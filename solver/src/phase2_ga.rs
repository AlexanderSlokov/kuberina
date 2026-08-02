//! Phase 2: Genetic Algorithm optimizer with CSP-integrated operators.
//!
//! Ported from `research/src/kuberina/phases/phase2_ga.py`.
//! From PAPER.md §4.3 and DESIGN.md §GA Operators.
//! Tournament selection k=3, uniform crossover + gang repair,
//! mutation rate=5% + CSP rollback, early stopping.
//!
//! Uses `rayon` for parallel fitness evaluation (preplan.md §3).

use rand::Rng;
use rand::SeedableRng;
use rand::rngs::StdRng;
use rand::seq::IndexedRandom;
use rayon::prelude::*;

use crate::csp::can_place_pod_on_node;
use crate::fitness::compute_fitness;
use crate::model::{Blueprint, FitnessWeights, GaConfig, Node, Pod, PodGroup};
use crate::phase1_ffd::compute_node_loads;

/// Execute the GA optimization loop starting from FFD seed.
///
/// Returns the best blueprint found after convergence or max generations.
/// From ga_estimation.md §5: with FFD warm-start, converges 3-5x faster.
pub fn run_ga(
    seed: &Blueprint,
    pods: &[Pod],
    nodes: &[Node],
    groups: &[PodGroup],
    config: &GaConfig,
    fitness_weights: &FitnessWeights,
) -> Blueprint {
    let mut rng = StdRng::seed_from_u64(config.random_seed);
    let mut population = init_population(seed, pods, nodes, groups, config, &mut rng);
    evaluate_all(&mut population, pods, nodes, groups, fitness_weights);

    let mut best = find_best(&population).clone();
    let mut stale_count = 0_usize;

    for generation in 0..config.max_generations {
        let mut offspring = breed_generation(
            &population, pods, nodes, groups, config, fitness_weights, &mut rng,
        );
        // WHY: evaluate BEFORE select — offspring start with fitness=0.0,
        // which would always beat parents in sorting without real scores.
        evaluate_all(&mut offspring, pods, nodes, groups, fitness_weights);
        let next = select_survivors(population, offspring, config);

        let current_best = find_best(&next);
        if current_best.fitness < best.fitness {
            best = current_best.clone();
            stale_count = 0;
        } else {
            stale_count += 1;
        }

        if stale_count >= config.early_stop_generations {
            eprintln!(
                "Early stop at generation {} (no improvement for {} gens)",
                generation, config.early_stop_generations,
            );
            break;
        }

        if generation % 10 == 0 {
            let pct = (generation + 1) as f64 / config.max_generations as f64 * 100.0;
            let filled = (pct / 5.0) as usize;
            let bar: String = "█".repeat(filled) + &"░".repeat(20 - filled);
            
            if generation > 0 {
                eprint!("\x1B[4A"); // Move cursor up 4 lines
            }
            
            eprintln!(
                "\x1B[2KGen {:>4}/{} | best={:.4} | stale={:<3} | {} {:.0}%",
                generation, config.max_generations, best.fitness, stale_count, bar, pct,
            );
            eprintln!(
                "\x1B[2K  [Scorecard] Cap: {:.0} | Sel: {:.0} | Gang: {:.0}",
                best.scorecard.capacity_penalty, best.scorecard.selector_penalty, best.scorecard.gang_penalty
            );
            eprintln!(
                "\x1B[2K              Frag: {:.2} | Aff: {:.0} | Var: {:.4}",
                best.scorecard.fragmentation, best.scorecard.affinity_violations, best.scorecard.utilization_variance
            );
            eprintln!(
                "\x1B[2K  [Nodes] Active: {:.0} / {}",
                best.scorecard.active_nodes, nodes.len()
            );
        }

        population = next;
    }

    best
}

/// Create initial population from FFD seed + random perturbations.
fn init_population(
    seed: &Blueprint,
    pods: &[Pod],
    nodes: &[Node],
    groups: &[PodGroup],
    config: &GaConfig,
    rng: &mut StdRng,
) -> Vec<Blueprint> {
    let mut population = Vec::with_capacity(config.population_size);
    population.push(seed.clone());

    for _ in 1..config.population_size {
        let mut variant = seed.clone();
        // WHY: perturb ~20% of assignments to inject diversity into the
        // FFD-derived population. Pure clones would make crossover useless.
        mutate(&mut variant, pods, nodes, groups, 0.2, rng);
        population.push(variant);
    }

    population
}

/// Evaluate fitness for every individual in the population.
///
/// ponytail: this is the rayon hot path — .par_iter_mut() vắt kiệt CPU
/// (preplan.md §3). Switching to .iter_mut() reverts to single-threaded.
fn evaluate_all(
    population: &mut [Blueprint],
    pods: &[Pod],
    nodes: &[Node],
    groups: &[PodGroup],
    weights: &FitnessWeights,
) {
    population.par_iter_mut().for_each(|bp| {
        bp.node_load = compute_node_loads(&bp.assignment, pods, nodes.len());
        let (fitness, sc) = compute_fitness(bp, pods, nodes, groups, weights);
        bp.fitness = fitness;
        bp.scorecard = sc;
    });
}

/// Produce offspring via selection, crossover, and mutation.
fn breed_generation(
    population: &[Blueprint],
    pods: &[Pod],
    nodes: &[Node],
    groups: &[PodGroup],
    config: &GaConfig,
    _weights: &FitnessWeights,
    rng: &mut StdRng,
) -> Vec<Blueprint> {
    let mut offspring = Vec::with_capacity(config.population_size);

    while offspring.len() < config.population_size {
        let p1 = tournament_select(population, config.tournament_size, rng);
        let p2 = tournament_select(population, config.tournament_size, rng);

        let mut child = if rng.random::<f64>() < config.crossover_rate {
            let c = uniform_crossover(p1, p2, rng);
            repair_gangs_on(c, p1, p2, groups, pods, nodes)
        } else {
            if p1.fitness < p2.fitness { p1.clone() } else { p2.clone() }
        };

        mutate(&mut child, pods, nodes, groups, config.mutation_rate, rng);
        offspring.push(child);
    }

    offspring.truncate(config.population_size);
    offspring
}

/// Elitism: keep best from parents + offspring combined.
fn select_survivors(
    parents: Vec<Blueprint>,
    offspring: Vec<Blueprint>,
    config: &GaConfig,
) -> Vec<Blueprint> {
    let mut combined: Vec<Blueprint> = parents.into_iter().chain(offspring).collect();
    combined.sort_by(|a, b| a.fitness.partial_cmp(&b.fitness).unwrap_or(std::cmp::Ordering::Equal));
    combined.truncate(config.population_size);
    combined
}

/// Return the individual with the lowest (best) fitness.
fn find_best(population: &[Blueprint]) -> &Blueprint {
    population
        .iter()
        .min_by(|a, b| a.fitness.partial_cmp(&b.fitness).unwrap_or(std::cmp::Ordering::Equal))
        .expect("population must not be empty")
}

/// Pick k random individuals, return the one with best (lowest) fitness.
///
/// From ga_estimation.md §4: tournament selection with k_tour=3.
fn tournament_select<'a>(
    population: &'a [Blueprint],
    k: usize,
    rng: &mut StdRng,
) -> &'a Blueprint {
    let k = k.min(population.len());
    let candidates: Vec<&Blueprint> = (0..k)
        .map(|_| {
            let idx = rng.random_range(0..population.len());
            &population[idx]
        })
        .collect();
    candidates
        .into_iter()
        .min_by(|a, b| a.fitness.partial_cmp(&b.fitness).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap()
}

/// Gene-level uniform crossover: each gene from p1 or p2 with 50% chance.
///
/// From PAPER.md §4.3 point 3: "Uniform Crossover".
fn uniform_crossover(p1: &Blueprint, p2: &Blueprint, rng: &mut StdRng) -> Blueprint {
    let assignment: Vec<usize> = p1
        .assignment
        .iter()
        .zip(p2.assignment.iter())
        .map(|(&a, &b)| if rng.random::<f64>() < 0.5 { a } else { b })
        .collect();
    Blueprint {
        assignment,
        fitness: 0.0,
        node_load: Vec::new(),
        scorecard: Default::default(),
    }
}

/// Fix gang assignments broken by crossover.
///
/// If crossover split a gang and broke capacity, rollback the entire
/// gang assignment to the better parent.
fn repair_gangs_on(
    mut child: Blueprint,
    p1: &Blueprint,
    p2: &Blueprint,
    groups: &[PodGroup],
    pods: &[Pod],
    nodes: &[Node],
) -> Blueprint {
    for group in groups {
        child.node_load = compute_node_loads(&child.assignment, pods, nodes.len());
        if gang_is_feasible(&child, group, nodes) {
            continue;
        }

        let p1_feasible = gang_is_feasible(p1, group, nodes);
        let p2_feasible = gang_is_feasible(p2, group, nodes);

        if !p1_feasible && !p2_feasible {
            // WHY: If both parents were infeasible (e.g. trapped on fallback node 0),
            // don't rollback. Rollback would just trap the child too. Let it explore!
            continue;
        }

        // Rollback to the parent that had this gang feasible
        let source = match (p1_feasible, p2_feasible) {
            (true, true) => if p1.fitness <= p2.fitness { p1 } else { p2 },
            (true, false) => p1,
            (false, true) => p2,
            (false, false) => unreachable!(),
        };

        for &pod_idx in &group.pod_indices {
            child.assignment[pod_idx] = source.assignment[pod_idx];
        }
    }
    child
}

/// Random reset mutation with CSP rollback for gang pods.
///
/// - Regular pod: move to random eligible node
/// - Gang pod: try move, rollback if gang becomes infeasible
fn mutate(
    blueprint: &mut Blueprint,
    pods: &[Pod],
    nodes: &[Node],
    groups: &[PodGroup],
    rate: f64,
    rng: &mut StdRng,
) {
    let group_lookup = build_group_lookup(groups, pods.len());
    let num_nodes = nodes.len();

    for i in 0..blueprint.assignment.len() {
        if rng.random::<f64>() > rate {
            continue;
        }

        let is_swap = rng.random_bool(0.5);

        if is_swap {
            // Swap operator
            let j = rng.random_range(0..pods.len());
            if i == j {
                continue;
            }

            let node_i = blueprint.assignment[i];
            let node_j = blueprint.assignment[j];
            if node_i == node_j {
                continue;
            }

            // Check if nodes are eligible for each other
            if !can_place_pod_on_node(&pods[i], &nodes[node_j]) || 
               !can_place_pod_on_node(&pods[j], &nodes[node_i]) {
                continue;
            }

            let group_i = group_lookup[i];
            let group_j = group_lookup[j];

            let mut was_feasible_i = false;
            let mut was_feasible_j = false;

            if let Some(g_idx) = group_i {
                was_feasible_i = gang_is_feasible(blueprint, &groups[g_idx], nodes);
            }
            if let Some(g_idx) = group_j {
                was_feasible_j = gang_is_feasible(blueprint, &groups[g_idx], nodes);
            }

            // Perform swap
            blueprint.assignment[i] = node_j;
            blueprint.assignment[j] = node_i;

            let mut rollback = false;

            if group_i.is_some() || group_j.is_some() {
                blueprint.node_load = compute_node_loads(&blueprint.assignment, pods, num_nodes);
                
                if let Some(g_idx) = group_i {
                    if was_feasible_i && !gang_is_feasible(blueprint, &groups[g_idx], nodes) {
                        rollback = true;
                    }
                }
                
                if let Some(g_idx) = group_j {
                    if was_feasible_j && !gang_is_feasible(blueprint, &groups[g_idx], nodes) {
                        rollback = true;
                    }
                }
            }

            if rollback {
                blueprint.assignment[i] = node_i;
                blueprint.assignment[j] = node_j;
            }

        } else {
            // Move operator
            let old_node = blueprint.assignment[i];
            let eligible = eligible_nodes_for_pod(&pods[i], nodes);
            if eligible.is_empty() {
                continue;
            }

            let &new_node = eligible.choose(rng).unwrap();
            let group_idx = group_lookup[i];
            let mut was_feasible = false;
            
            if let Some(g_idx) = group_idx {
                was_feasible = gang_is_feasible(blueprint, &groups[g_idx], nodes);
            }

            blueprint.assignment[i] = new_node;

            if let Some(g_idx) = group_idx {
                blueprint.node_load = compute_node_loads(&blueprint.assignment, pods, num_nodes);
                if was_feasible && !gang_is_feasible(blueprint, &groups[g_idx], nodes) {
                    blueprint.assignment[i] = old_node; // rollback
                }
            }
        }
    }
}

/// Check if all gang pods fit on their assigned nodes.
fn gang_is_feasible(blueprint: &Blueprint, group: &PodGroup, nodes: &[Node]) -> bool {
    group.pod_indices.iter().all(|&pod_idx| {
        let node_idx = blueprint.assignment[pod_idx];
        nodes[node_idx].allocatable.fits(blueprint.node_load[node_idx])
    })
}

/// Map pod_idx → Some(group_idx) or None if pod is not in any gang.
fn build_group_lookup(groups: &[PodGroup], num_pods: usize) -> Vec<Option<usize>> {
    let mut lookup = vec![None; num_pods];
    for (g_idx, group) in groups.iter().enumerate() {
        for &pod_idx in &group.pod_indices {
            lookup[pod_idx] = Some(g_idx);
        }
    }
    lookup
}

/// Return indices of nodes that pass taint+selector checks for a pod.
fn eligible_nodes_for_pod(pod: &Pod, nodes: &[Node]) -> Vec<usize> {
    nodes
        .iter()
        .enumerate()
        .filter(|(_, n)| can_place_pod_on_node(pod, n))
        .map(|(i, _)| i)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ResourceVector;
    use std::collections::HashMap;

    fn pod(name: &str, cpu: f64, ram: f64) -> Pod {
        Pod {
            name: name.into(),
            namespace: "ns".into(),
            requests: ResourceVector::new(cpu, ram, 0.0),
            tolerations: vec![],
            node_selector: HashMap::new(),
            affinity_targets: vec![],
            anti_affinity_targets: vec![],
            group_name: String::new(),
            topology_spread: None,
        }
    }

    fn node(name: &str, cpu: f64, ram: f64) -> Node {
        Node {
            name: name.into(),
            allocatable: ResourceVector::new(cpu, ram, 0.0),
            labels: HashMap::new(),
            taints: vec![],
            zone: String::new(),
            rack: String::new(),
        }
    }

    #[test]
    fn group_lookup_maps_correctly() {
        let groups = vec![PodGroup {
            name: "g".into(),
            pod_indices: vec![1, 3],
            min_members: 2,
            node_selector: HashMap::new(),
            colocate: false,
        }];
        let lookup = build_group_lookup(&groups, 5);
        assert_eq!(lookup[0], None);
        assert_eq!(lookup[1], Some(0));
        assert_eq!(lookup[2], None);
        assert_eq!(lookup[3], Some(0));
    }

    #[test]
    fn eligible_nodes_filters_by_selector() {
        let p = Pod {
            node_selector: HashMap::from([("gpu".into(), "true".into())]),
            ..pod("p", 1.0, 1.0)
        };
        let nodes = vec![
            node("n0", 4.0, 16.0),
            Node {
                labels: HashMap::from([("gpu".into(), "true".into())]),
                ..node("n1", 4.0, 16.0)
            },
        ];
        let eligible = eligible_nodes_for_pod(&p, &nodes);
        assert_eq!(eligible, vec![1]);
    }

    #[test]
    fn uniform_crossover_produces_valid_length() {
        let p1 = Blueprint {
            assignment: vec![0, 1, 2],
            fitness: 1.0,
            node_load: vec![],
            scorecard: Default::default(),
        };
        let p2 = Blueprint {
            assignment: vec![2, 1, 0],
            fitness: 2.0,
            node_load: vec![],
            scorecard: Default::default(),
        };
        let mut rng = StdRng::seed_from_u64(42);
        let child = uniform_crossover(&p1, &p2, &mut rng);
        assert_eq!(child.assignment.len(), 3);
    }

    #[test]
    fn ga_improves_or_maintains_seed_fitness() {
        let pods = vec![pod("a", 1.0, 2.0), pod("b", 1.0, 2.0)];
        let nodes = vec![node("n0", 4.0, 16.0), node("n1", 4.0, 16.0)];
        let seed = Blueprint {
            assignment: vec![0, 0],
            fitness: 0.0,
            node_load: compute_node_loads(&[0, 0], &pods, 2),
            scorecard: Default::default(),
        };
        let config = GaConfig {
            population_size: 16,
            max_generations: 20,
            early_stop_generations: 10,
            ..GaConfig::default()
        };
        let weights = FitnessWeights::default();
        let (seed_fitness, _) = compute_fitness(&seed, &pods, &nodes, &[], &weights);

        let best = run_ga(&seed, &pods, &nodes, &[], &config, &weights);
        assert!(best.fitness <= seed_fitness + 1e-9);
    }
}
