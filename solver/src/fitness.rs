//! Fitness function — evaluate blueprint quality via weighted multi-objective sum.
//!
//! Ported from `research/src/kuberina/fitness.py`.
//! From PAPER.md §3.2 Objective Function:
//!     min F(s) = w1·f_nodes + w2·f_frag + w3·f_affinity + w4·f_var + Φ(s)
//!
//! Lower is better. Φ(s) = large scalar if any hard constraint violated.

use std::collections::{HashMap, HashSet};

use crate::csp::{compute_capacity_overflow, compute_selector_violations, node_within_capacity};
use crate::model::{Blueprint, FitnessWeights, Node, Pod, PodGroup, ResourceVector, Scorecard};

/// Compute weighted-sum fitness score for a blueprint.
///
/// Returns a f64 — lower is better. Massive penalty if infeasible.
pub fn compute_fitness(
    blueprint: &Blueprint,
    pods: &[Pod],
    nodes: &[Node],
    groups: &[PodGroup],
    weights: &FitnessWeights,
) -> (f64, Scorecard) {
    let mut sc = Scorecard::default();

    let penalty = compute_hard_penalty(blueprint, pods, nodes, groups, &mut sc);

    let f_nodes = count_active_nodes(&blueprint.assignment, nodes.len()) as f64;
    let f_frag = compute_fragmentation(&blueprint.node_load, nodes);
    let f_aff = compute_affinity_violations(&blueprint.assignment, pods) as f64;
    let f_var = compute_utilization_variance(&blueprint.node_load, nodes);
    let f_spread = compute_topology_spread_penalty(&blueprint.assignment, pods, nodes);

    sc.active_nodes = f_nodes;
    sc.fragmentation = f_frag;
    sc.affinity_violations = f_aff;
    sc.utilization_variance = f_var;
    sc.topology_spread_penalty = f_spread;

    let fitness = penalty
        + weights.node_count * f_nodes
        + weights.fragmentation * f_frag
        + weights.affinity_violation * f_aff
        + weights.utilization_variance * f_var
        + weights.topology_spread * f_spread;

    (fitness, sc)
}

/// f_nodes: count nodes that have at least one pod assigned.
///
/// PAPER.md: f_nodes = Σ y_j (number of active nodes).
///
/// ```
/// # use kuberina_solver::fitness::count_active_nodes;
/// assert_eq!(count_active_nodes(&[0, 0, 2], 3), 2);
/// ```
pub fn count_active_nodes(assignment: &[usize], _num_nodes: usize) -> usize {
    let mut seen = std::collections::HashSet::new();
    for &n in assignment {
        seen.insert(n);
    }
    seen.len()
}

/// f_frag: sum of wasted capacity on active nodes across all dimensions.
///
/// PAPER.md: f_frag = Σ_{j: y_j=1} Σ_r max(0, C_j^r − Σ req_i^r)
pub fn compute_fragmentation(node_load: &[ResourceVector], nodes: &[Node]) -> f64 {
    let mut total_waste = 0.0_f64;
    for (j, load) in node_load.iter().enumerate() {
        // WHY: skip empty nodes — they have no waste, they're just unused.
        if load.is_zero() {
            continue;
        }
        let cap = &nodes[j].allocatable;
        total_waste += (cap.cpu - load.cpu).max(0.0);
        total_waste += (cap.ram - load.ram).max(0.0);
        total_waste += (cap.gpu - load.gpu).max(0.0);
        // WHY: guard against f64::MAX unconstrained dimensions.
        // If node doesn't declare a limit (default=MAX), skip that dim
        // from waste calc — otherwise waste = MAX → poisons fitness.
        if cap.storage < f64::MAX {
            total_waste += (cap.storage - load.storage).max(0.0);
        }
        if cap.disk_read < f64::MAX {
            total_waste += (cap.disk_read - load.disk_read).max(0.0);
        }
        if cap.disk_write < f64::MAX {
            total_waste += (cap.disk_write - load.disk_write).max(0.0);
        }
        if cap.net_in < f64::MAX {
            total_waste += (cap.net_in - load.net_in).max(0.0);
        }
        if cap.net_out < f64::MAX {
            total_waste += (cap.net_out - load.net_out).max(0.0);
        }
    }
    total_waste
}

/// f_affinity: count soft affinity/anti-affinity violations.
///
/// Follows Kubernetes semantics with the node as the topology domain (IR v1
/// §3.1.1, #26). An affinity entry is violated when none of the pods it names
/// shares the pod's node. An anti-affinity violation is one pair of pods on the
/// same node, counted once whichever pod declared it.
///
/// ```ignore
/// let violations = compute_affinity_violations(&blueprint.assignment, &pods);
/// ```
pub fn compute_affinity_violations(assignment: &[usize], pods: &[Pod]) -> usize {
    let index_of = identity_index(pods);
    unmet_affinity_entries(assignment, pods, &index_of)
        + colocated_anti_affinity_pairs(assignment, pods, &index_of)
}

/// Map each pod's `(namespace, name)` to its index, without allocating.
fn identity_index(pods: &[Pod]) -> HashMap<(&str, &str), usize> {
    pods.iter()
        .enumerate()
        .map(|(i, p)| ((p.namespace.as_str(), p.name.as_str()), i))
        .collect()
}

/// Affinity entries with no named pod on the declaring pod's node.
fn unmet_affinity_entries(
    assignment: &[usize],
    pods: &[Pod],
    index_of: &HashMap<(&str, &str), usize>,
) -> usize {
    let mut unmet = 0;
    for (i, pod) in pods.iter().enumerate() {
        for entry in &pod.affinity_targets {
            let named: Vec<usize> = entry
                .iter()
                .filter_map(|t| lookup_identity(index_of, t))
                .collect();
            // WHY skip an empty entry: it names no other pod (for example, a
            // single-replica declaration naming itself), so there is nothing to be near.
            if !named.is_empty() && named.iter().all(|&j| assignment[j] != assignment[i]) {
                unmet += 1;
            }
        }
    }
    unmet
}

/// Distinct anti-affinity pairs that share a node.
fn colocated_anti_affinity_pairs(
    assignment: &[usize],
    pods: &[Pod],
    index_of: &HashMap<(&str, &str), usize>,
) -> usize {
    let mut pairs = HashSet::new();
    for (i, pod) in pods.iter().enumerate() {
        for target in &pod.anti_affinity_targets {
            let Some(j) = lookup_identity(index_of, target) else {
                continue;
            };
            if i != j && assignment[i] == assignment[j] {
                pairs.insert((i.min(j), i.max(j)));
            }
        }
    }
    pairs.len()
}

/// Find the pod whose `namespace/podname` identity equals `target`.
fn lookup_identity(index_of: &HashMap<(&str, &str), usize>, target: &str) -> Option<usize> {
    let (namespace, name) = target.split_once('/')?;
    index_of.get(&(namespace, name)).copied()
}

/// f_var: variance of CPU utilization across active nodes.
///
/// PAPER.md: Var({U_j^r : y_j = 1}) — vessel trim & stability.
pub fn compute_utilization_variance(node_load: &[ResourceVector], nodes: &[Node]) -> f64 {
    let utilizations: Vec<f64> = node_load
        .iter()
        .enumerate()
        .filter(|(j, load)| {
            let cap = &nodes[*j].allocatable;
            cap.cpu > 0.0 && !load.is_zero()
        })
        .map(|(j, load)| load.cpu / nodes[j].allocatable.cpu)
        .collect();

    if utilizations.len() < 2 {
        return 0.0;
    }

    let mean = utilizations.iter().sum::<f64>() / utilizations.len() as f64;
    utilizations.iter().map(|u| (u - mean).powi(2)).sum::<f64>() / (utilizations.len() - 1) as f64
}

/// f_spread: penalize uneven pod distribution across topology zones/racks.
///
/// For each pod with a topologySpread constraint, counts how many pods of
/// the same "spread group" (same name prefix before replica suffix) land in
/// each topology domain. Penalty = Σ max(0, actual_skew - maxSkew).
///
/// ```
/// # use kuberina_solver::fitness::compute_topology_spread_penalty;
/// // Tested via compute_fitness integration
/// ```
pub fn compute_topology_spread_penalty(assignment: &[usize], pods: &[Pod], nodes: &[Node]) -> f64 {
    let mut penalty = 0.0_f64;

    // Collect pods that have topology_spread set
    for (i, pod) in pods.iter().enumerate() {
        let ts = match &pod.topology_spread {
            Some(ts) => ts,
            None => continue,
        };

        // WHY: only count from the first replica to avoid double-counting.
        // All replicas share the same constraint, but we evaluate the
        // distribution of ALL pods with the same group_name + spread key.
        if !is_first_with_spread(pods, i) {
            continue;
        }

        let mut counts: HashMap<&str, usize> = HashMap::new();
        // Pre-populate with all available domains to correctly calculate skew
        // even if some domains end up with 0 pods.
        for node in nodes {
            let domain = match ts.topology_key.as_str() {
                "zone" => &node.zone,
                "rack" => &node.rack,
                _ => "",
            };
            if !domain.is_empty() {
                counts.insert(domain, 0);
            }
        }

        // Count pods sharing the same spread group
        for (j, other) in pods.iter().enumerate() {
            let same_spread = match &other.topology_spread {
                Some(ots) => {
                    ots.topology_key == ts.topology_key
                        && other.namespace == pod.namespace
                        && share_base_name(&pod.name, &other.name)
                }
                None => false,
            };
            if !same_spread {
                continue;
            }
            let node = &nodes[assignment[j]];
            let domain = match ts.topology_key.as_str() {
                "zone" => &node.zone,
                "rack" => &node.rack,
                _ => "",
            };
            if !domain.is_empty() {
                *counts.entry(domain).or_insert(0) += 1;
            }
        }

        if counts.len() < 2 {
            continue;
        }

        let max_count = counts.values().copied().max().unwrap_or(0);
        let min_count = counts.values().copied().min().unwrap_or(0);
        let skew = max_count.saturating_sub(min_count);
        if skew > ts.max_skew {
            penalty += (skew - ts.max_skew) as f64;
        }
    }
    penalty
}

/// True if this is the first pod in the list with a topology_spread
/// and the same base name (i.e., don't re-evaluate for each replica).
fn is_first_with_spread(pods: &[Pod], idx: usize) -> bool {
    let pod = &pods[idx];
    let ts = match &pod.topology_spread {
        Some(ts) => ts,
        None => return false,
    };
    for (j, other) in pods.iter().enumerate() {
        if j >= idx {
            return true;
        }
        if other.topology_spread.as_ref().is_some_and(|ots| {
            ots.topology_key == ts.topology_key
                && other.namespace == pod.namespace
                && share_base_name(&pod.name, &other.name)
        }) {
            return false;
        }
    }
    true
}

/// Check if two pod names share the same base (before replica suffix).
/// "worker-0000" and "worker-0003" share base "worker".
fn share_base_name(a: &str, b: &str) -> bool {
    let base_a = a.rsplit_once('-').map(|(b, _)| b).unwrap_or(a);
    let base_b = b.rsplit_once('-').map(|(b, _)| b).unwrap_or(b);
    base_a == base_b
}

/// Φ(s): gradient penalty scalar for hard constraints.
///
/// Instead of infinity, returns a massive but differentiable scalar.
/// This allows the GA to climb out of infeasible search spaces rather
/// than plateauing in infinity.
///
/// Base penalty weight: 1,000,000 to ensure any invalid solution
/// is ranked strictly worse than any valid solution.
fn compute_hard_penalty(
    blueprint: &Blueprint,
    pods: &[Pod],
    nodes: &[Node],
    groups: &[PodGroup],
    sc: &mut Scorecard,
) -> f64 {
    let mut total = 0.0_f64;

    let cap_overflow = compute_capacity_overflow(&blueprint.assignment, pods, nodes);
    if cap_overflow > 1e-9 {
        let p = 1_000_000.0 + cap_overflow * 10_000.0;
        total += p;
        sc.capacity_penalty = p;
    }

    let sel_violations = compute_selector_violations(&blueprint.assignment, pods, nodes);
    if sel_violations > 0 {
        let p = 500_000.0 + sel_violations as f64 * 50_000.0;
        total += p;
        sc.selector_penalty = p;
    }

    let gp = gang_penalty(blueprint, pods, nodes, groups);
    sc.gang_penalty = gp;
    total += gp;

    total
}

/// Soft penalty if any gang pod can't fit on its assigned node.
///
/// Penalty scales linearly with the number of missing pods to reach min_members.
fn gang_penalty(blueprint: &Blueprint, pods: &[Pod], nodes: &[Node], groups: &[PodGroup]) -> f64 {
    let mut penalty = 0.0_f64;
    for group in groups {
        let placed = group
            .pod_indices
            .iter()
            .filter(|&&pod_idx| {
                let node_idx = blueprint.assignment[pod_idx];
                node_within_capacity(
                    node_idx,
                    &blueprint.assignment,
                    &blueprint.node_load,
                    pods,
                    nodes,
                )
            })
            .count();
        if placed < group.min_members {
            let missing = group.min_members - placed;
            penalty += 500_000.0 + missing as f64 * 10_000.0;
        }
    }
    penalty
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ResourceVector;
    use std::collections::HashMap;

    fn pod(name: &str) -> Pod {
        Pod {
            name: name.into(),
            namespace: "ns".into(),
            requests: ResourceVector::zero(),
            tolerations: vec![],
            node_selector: HashMap::new(),
            affinity_targets: vec![],
            anti_affinity_targets: vec![],
            group_name: String::new(),
            topology_spread: None,
            observed: None,
            gpu_share: None,
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
            gpu_devices: None,
        }
    }

    #[test]
    fn count_active_nodes_basic() {
        assert_eq!(count_active_nodes(&[0, 0, 2], 3), 2);
        assert_eq!(count_active_nodes(&[1, 1, 1], 3), 1);
    }

    #[test]
    fn fragmentation_zero_on_perfect_fit() {
        let loads = [ResourceVector::new(4.0, 16.0, 0.0)];
        let nodes = [node("n", 4.0, 16.0)];
        assert_eq!(compute_fragmentation(&loads, &nodes), 0.0);
    }

    #[test]
    fn fragmentation_measures_waste() {
        let loads = [ResourceVector::new(2.0, 8.0, 0.0)];
        let nodes = [node("n", 4.0, 16.0)];
        // waste = (4-2) + (16-8) = 10.0
        assert_eq!(compute_fragmentation(&loads, &nodes), 10.0);
    }

    #[test]
    fn affinity_target_is_matched_by_namespaced_identity() {
        // Regression for #26: targets were looked up by bare pod name, so a
        // resolved identity such as "ns/b" never matched and no rule counted.
        let pods = vec![
            Pod {
                affinity_targets: vec![vec!["ns/b".into()]],
                ..pod("a")
            },
            pod("b"),
        ];
        assert_eq!(compute_affinity_violations(&[0, 1], &pods), 1);
    }

    #[test]
    fn rule_declared_by_the_later_pod_is_counted() {
        // Regression for #26: the pair was counted only from the pod with the lower
        // index, so a rule that only the later pod declared was never scored.
        let pods = vec![
            pod("a"),
            Pod {
                anti_affinity_targets: vec!["ns/a".into()],
                ..pod("b")
            },
        ];
        assert_eq!(compute_affinity_violations(&[0, 0], &pods), 1);
    }

    #[test]
    fn symmetric_anti_affinity_pair_is_counted_once() {
        let pods = vec![
            Pod {
                anti_affinity_targets: vec!["ns/b".into()],
                ..pod("a")
            },
            Pod {
                anti_affinity_targets: vec!["ns/a".into()],
                ..pod("b")
            },
        ];
        assert_eq!(compute_affinity_violations(&[0, 0], &pods), 1);
    }

    #[test]
    fn affinity_entry_is_satisfied_by_one_of_its_pods() {
        // Kubernetes podAffinity semantics (#26): one named pod on the node suffices.
        let pods = vec![
            Pod {
                affinity_targets: vec![vec!["ns/b".into(), "ns/c".into()]],
                ..pod("a")
            },
            pod("b"),
            pod("c"),
        ];
        assert_eq!(compute_affinity_violations(&[0, 0, 1], &pods), 0);
    }

    #[test]
    fn affinity_entry_with_no_named_pod_nearby_is_one_violation() {
        let pods = vec![
            Pod {
                affinity_targets: vec![vec!["ns/b".into(), "ns/c".into()]],
                ..pod("a")
            },
            pod("b"),
            pod("c"),
        ];
        assert_eq!(compute_affinity_violations(&[0, 1, 2], &pods), 1);
    }

    #[test]
    fn affinity_violation_detected() {
        let pods = vec![
            Pod {
                affinity_targets: vec![vec!["ns/b".into()]],
                ..pod("a")
            },
            pod("b"),
        ];
        assert_eq!(compute_affinity_violations(&[0, 1], &pods), 1);
    }

    #[test]
    fn affinity_satisfied_no_violation() {
        let pods = vec![
            Pod {
                affinity_targets: vec![vec!["ns/b".into()]],
                ..pod("a")
            },
            pod("b"),
        ];
        assert_eq!(compute_affinity_violations(&[0, 0], &pods), 0);
    }

    #[test]
    fn anti_affinity_violation() {
        let pods = vec![
            Pod {
                anti_affinity_targets: vec!["ns/b".into()],
                ..pod("a")
            },
            pod("b"),
        ];
        assert_eq!(compute_affinity_violations(&[0, 0], &pods), 1);
    }

    #[test]
    fn utilization_variance_balanced() {
        let loads = [
            ResourceVector::new(2.0, 0.0, 0.0),
            ResourceVector::new(2.0, 0.0, 0.0),
        ];
        let nodes = [node("a", 4.0, 16.0), node("b", 4.0, 16.0)];
        assert_eq!(compute_utilization_variance(&loads, &nodes), 0.0);
    }

    #[test]
    fn hard_penalty_scalar_gradient() {
        let pods = vec![Pod {
            requests: ResourceVector::new(5.0, 0.0, 0.0),
            ..pod("p")
        }];
        let nodes = vec![node("n", 4.0, 16.0)];
        let bp = Blueprint {
            assignment: vec![0],
            fitness: 0.0,
            node_load: vec![ResourceVector::new(5.0, 0.0, 0.0)],
            scorecard: Default::default(),
        };
        let (f, _) = compute_fitness(&bp, &pods, &nodes, &[], &FitnessWeights::default());
        assert!(f > 1_000_000.0);
        assert!(f.is_finite());
    }

    #[test]
    fn topology_spread_penalty_skew() {
        use crate::model::TopologySpread;
        let ts = Some(TopologySpread {
            max_skew: 1,
            topology_key: "zone".into(),
        });
        let pods = vec![
            Pod {
                name: "w-0000".into(),
                topology_spread: ts.clone(),
                ..pod("w-0000")
            },
            Pod {
                name: "w-0001".into(),
                topology_spread: ts.clone(),
                ..pod("w-0001")
            },
            Pod {
                name: "w-0002".into(),
                topology_spread: ts.clone(),
                ..pod("w-0002")
            },
        ];
        let nodes = vec![
            Node {
                zone: "us-east-1a".into(),
                ..node("n0", 4.0, 16.0)
            },
            Node {
                zone: "us-east-1b".into(),
                ..node("n1", 4.0, 16.0)
            },
        ];
        // 3 pods across 2 zones: [0,0,0] → zone-a=3, zone-b=0 → skew=3, max_skew=1 → penalty=2
        let penalty = compute_topology_spread_penalty(&[0, 0, 0], &pods, &nodes);
        assert!((penalty - 2.0).abs() < 1e-9);

        // Better: [0,1,0] → zone-a=2, zone-b=1 → skew=1 → no penalty
        let penalty = compute_topology_spread_penalty(&[0, 1, 0], &pods, &nodes);
        assert!((penalty - 0.0).abs() < 1e-9);
    }

    // ─── Per-device GPU (IR v1 §3.4, X-3) ───────────────────────────────

    use crate::gpu_test_fixtures::{eight_gpu_node, sixty_percent_then_half};

    /// All nine done-when pods on one eight-device node, as one gang of nine.
    fn device_bound_gang() -> (Blueprint, Vec<Pod>, Vec<Node>, PodGroup) {
        let pods = sixty_percent_then_half();
        let nodes = vec![eight_gpu_node("a")];
        let assignment = vec![0; pods.len()];
        let node_load = crate::phase1_ffd::compute_node_loads(&assignment, &pods, 1);
        let blueprint = Blueprint {
            assignment,
            fitness: 0.0,
            node_load,
            scorecard: Default::default(),
        };
        let group = PodGroup {
            name: "g".into(),
            pod_indices: (0..pods.len()).collect(),
            min_members: pods.len(),
            node_selector: HashMap::new(),
            colocate: false,
        };
        (blueprint, pods, nodes, group)
    }

    #[test]
    fn gang_on_a_node_whose_devices_cannot_hold_it_is_penalized() {
        let (blueprint, pods, nodes, group) = device_bound_gang();
        assert!(gang_penalty(&blueprint, &pods, &nodes, &[group]) > 0.0);
    }
}
