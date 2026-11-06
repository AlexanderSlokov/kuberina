//! Why free admission ends behind the in-order walk on the trace: how often each GA
//! move survives, from the FFD seed and from the walk.
//!
//! Run from `solver/`:
//!
//! ```shell
//! cargo run --release --example free_ga_probe -- <infra.yaml> <workloads.yaml> <generations>
//! ```
//!
//! The GA's mutation is private to `phase2_ga.rs`, so this example rebuilds each move
//! from the same rule and scores it alone with the solver's fitness.

use rand::rngs::StdRng;
use rand::seq::IndexedRandom;
use rand::{Rng, SeedableRng};

use kuberina_solver::admission::{admit_in_order, ffd_with_admission};
use kuberina_solver::admission_ir::arrival_order;
use kuberina_solver::admission_swap::trade_at_random;
use kuberina_solver::csp::can_place_pod_on_node;
use kuberina_solver::fitness::compute_fitness;
use kuberina_solver::model::{
    Blueprint, FfdWeights, FitnessWeights, GaConfig, Node, Pod, UNPLACED,
};
use kuberina_solver::packing::Packing;
use kuberina_solver::parser::{load_infra, load_workloads};
use kuberina_solver::phase0::pre_deduct_daemonsets;
use kuberina_solver::phase1_ffd::compute_node_loads;
use kuberina_solver::phase2_ga::run_ga;
use kuberina_solver::unplaced_term::derive_unplaced_weights;

const TRIALS: usize = 300;

/// The instance and the weights every score in this example uses.
struct Probe<'a> {
    pods: &'a [Pod],
    nodes: &'a [Node],
    weights: FitnessWeights,
}

impl Probe<'_> {
    fn score(&self, assignment: &[usize]) -> (f64, bool) {
        let blueprint = Blueprint {
            assignment: assignment.to_vec(),
            fitness: 0.0,
            node_load: compute_node_loads(assignment, self.pods, self.nodes.len()),
            scorecard: Default::default(),
        };
        let (fitness, card) =
            compute_fitness(&blueprint, self.pods, self.nodes, &[], &self.weights);
        (fitness, card.capacity_penalty > 0.0)
    }

    fn random_eligible_node(&self, pod_idx: usize, rng: &mut StdRng) -> Option<usize> {
        let eligible: Vec<usize> = (0..self.nodes.len())
            .filter(|&n| can_place_pod_on_node(&self.pods[pod_idx], &self.nodes[n]))
            .collect();
        eligible.choose(rng).copied()
    }
}

fn indices_where(assignment: &[usize], placed: bool) -> Vec<usize> {
    (0..assignment.len())
        .filter(|&p| (assignment[p] != UNPLACED) == placed)
        .collect()
}

fn place_at_random(probe: &Probe, genes: &mut [usize], rng: &mut StdRng) {
    if let Some(&pod) = indices_where(genes, false).choose(rng) {
        genes[pod] = probe.random_eligible_node(pod, rng).unwrap_or(UNPLACED);
    }
}

fn move_at_random(probe: &Probe, genes: &mut [usize], rng: &mut StdRng) {
    if let Some(&pod) = indices_where(genes, true).choose(rng) {
        genes[pod] = probe.random_eligible_node(pod, rng).unwrap_or(genes[pod]);
    }
}

fn swap_at_random(_probe: &Probe, genes: &mut [usize], rng: &mut StdRng) {
    let placed = indices_where(genes, true);
    if let (Some(&a), Some(&b)) = (placed.choose(rng), placed.choose(rng)) {
        genes.swap(a, b);
    }
}

fn unplace_at_random(_probe: &Probe, genes: &mut [usize], rng: &mut StdRng) {
    if let Some(&pod) = indices_where(genes, true).choose(rng) {
        genes[pod] = UNPLACED;
    }
}

fn trade(probe: &Probe, genes: &mut [usize], rng: &mut StdRng) {
    let mut assignment = genes.to_vec();
    trade_at_random(&mut assignment, probe.pods, probe.nodes, rng);
    genes.copy_from_slice(&assignment);
}

/// Each gene from one of two traded parents, as the GA's uniform crossover does.
fn crossover_of_two_trades(probe: &Probe, genes: &mut [usize], rng: &mut StdRng) {
    let (mut first, mut second) = (genes.to_vec(), genes.to_vec());
    trade(probe, &mut first, rng);
    trade(probe, &mut second, rng);
    for (p, gene) in genes.iter_mut().enumerate() {
        *gene = if rng.random_bool(0.5) {
            first[p]
        } else {
            second[p]
        };
    }
}

type Operator = fn(&Probe, &mut [usize], &mut StdRng);

fn report_operator(name: &str, operator: Operator, probe: &Probe, seed: &[usize]) {
    let mut rng = StdRng::seed_from_u64(42);
    let (base, _) = probe.score(seed);
    let (mut breached, mut improved, mut best_gain) = (0, 0, 0.0_f64);
    for _ in 0..TRIALS {
        let mut genes = seed.to_vec();
        operator(probe, &mut genes, &mut rng);
        let (fitness, breach) = probe.score(&genes);
        breached += usize::from(breach);
        improved += usize::from(fitness < base - 1e-9);
        best_gain = best_gain.max(base - fitness);
    }
    println!(
        "  {name:<24} breach {breached:>3}/{TRIALS}  better {improved:>3}/{TRIALS}  best gain {best_gain:>12.1}"
    );
}

fn report_state(label: &str, probe: &Probe, seed: &Blueprint) {
    let genes = &seed.assignment;
    let unplaced = indices_where(genes, false);
    let packing = Packing::from_assignment(probe.pods, probe.nodes, genes);
    let could_fit = unplaced
        .iter()
        .filter(|&&p| packing.first_fit_node(p).is_some())
        .count();
    let gpu: f64 = indices_where(genes, true)
        .iter()
        .map(|&p| probe.pods[p].requests.gpu)
        .sum();
    let (fitness, _) = probe.score(genes);
    println!(
        "{label}: fitness {fitness:.4}, {} Pods placed, {gpu:.2} GPUs",
        genes.len() - unplaced.len()
    );
    println!(
        "  unplaced Pods that fit on some node as it stands: {could_fit} of {}",
        unplaced.len()
    );
    report_operator("place on random node", place_at_random, probe, genes);
    report_operator("move to random node", move_at_random, probe, genes);
    report_operator("swap two Pods", swap_at_random, probe, genes);
    report_operator("unplace", unplace_at_random, probe, genes);
    report_operator("trade", trade, probe, genes);
    report_operator(
        "crossover of two trades",
        crossover_of_two_trades,
        probe,
        genes,
    );
}

/// Best-fit in arrival order without moving any Pod: the same rule an online policy has.
fn report_online_walk(pods: &[Pod], nodes: &[Node], order: &[usize], cluster_gpu: f64) {
    let mut packing = Packing::new(pods, nodes);
    let (mut arrived, mut first, mut rejected) = (0.0, None, 0);
    for (index, &pod) in order.iter().enumerate() {
        match packing.best_fit_node(pod) {
            Some(node) => packing.place(pod, node),
            None => {
                rejected += 1;
                first = first.or(Some((index, arrived / cluster_gpu * 100.0)));
            }
        }
        arrived += pods[pod].requests.gpu;
    }
    let placed: f64 = indices_where(packing.assignment(), true)
        .iter()
        .map(|&p| pods[p].requests.gpu)
        .sum();
    println!(
        "walk without repair: first rejection {first:?}, {rejected} rejected, {placed:.2} GPUs ({:.2}%)",
        placed / cluster_gpu * 100.0
    );
}

fn first_rejection(walk: &Blueprint, pods: &[Pod], order: &[usize], cluster_gpu: f64) {
    let mut arrived = 0.0;
    for (index, &pod) in order.iter().enumerate() {
        if walk.assignment[pod] == UNPLACED {
            println!(
                "walk with repair: first rejection at {index}, {:.2}% of cluster GPU arrived",
                arrived / cluster_gpu * 100.0
            );
            return;
        }
        arrived += pods[pod].requests.gpu;
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let [_, infra, workloads, generations] = args.as_slice() else {
        panic!(
            "expected <infra.yaml> <workloads.yaml> <generations>, got {:?}",
            &args[1..]
        );
    };
    let (raw_nodes, daemon_sets) = load_infra(infra).expect("infrastructure document");
    let nodes = pre_deduct_daemonsets(&raw_nodes, &daemon_sets);
    let (pods, groups) = load_workloads(workloads).expect("workloads document");
    let order = arrival_order(&pods).expect("every Pod has an arrival");
    let base = FitnessWeights::default();
    let weights = FitnessWeights {
        unplaced: derive_unplaced_weights(&pods, &nodes, &base),
        ..base
    };
    let cluster_gpu: f64 = nodes.iter().map(|n| n.allocatable.gpu).sum();
    let probe = Probe {
        pods: &pods,
        nodes: &nodes,
        weights: weights.clone(),
    };

    let ffd = ffd_with_admission(&pods, &nodes, &FfdWeights::default());
    let walk = admit_in_order(&pods, &nodes, &order);
    report_state("FFD seed", &probe, &ffd);
    report_state("Walk seed", &probe, &walk);
    first_rejection(&walk, &pods, &order, cluster_gpu);
    report_online_walk(&pods, &nodes, &order, cluster_gpu);

    // The settings `select_ga_config` in main.rs picks above 500 Pods.
    let config = GaConfig {
        population_size: 1024,
        tournament_size: 5,
        crossover_rate: 0.85,
        max_generations: generations.parse().expect("generations is an integer"),
        early_stop_generations: usize::MAX,
        free_admission: true,
        ..GaConfig::default()
    };
    let outcome = run_ga(&walk, &pods, &nodes, &groups, &config, &weights);
    let (fitness, breach) = probe.score(&outcome.best.assignment);
    let placed = indices_where(&outcome.best.assignment, true);
    let gpu: f64 = placed.iter().map(|&p| pods[p].requests.gpu).sum();
    println!(
        "free GA from the walk, {} generations: fitness {fitness:.4}, breach {breach}, {} Pods, {gpu:.2} GPUs",
        outcome.generations_run,
        placed.len()
    );
}
