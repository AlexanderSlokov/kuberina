//! Fitness function — evaluate blueprint quality via weighted multi-objective sum.
//!
//! Ported from `research/src/kuberina/fitness.py`.
//! From PAPER.md §3.2 Objective Function:
//!     min F(s) = w1·f_nodes + w2·f_frag + w3·f_affinity + w4·f_var + Φ(s)
//!
//! Lower is better. Φ(s) = large scalar if any hard constraint violated.

use std::collections::HashMap;

use crate::csp::{compute_capacity_overflow, compute_selector_violations};
use crate::model::{Blueprint, FitnessWeights, Node, Pod, PodGroup, ResourceVector};

/// Compute weighted-sum fitness score for a blueprint.
///
/// Returns a f64 — lower is better. Massive penalty if infeasible.
pub fn compute_fitness(
    blueprint: &Blueprint,
    pods: &[Pod],
    nodes: &[Node],
    groups: &[PodGroup],
    weights: &FitnessWeights,
) -> f64 {
    let penalty = compute_hard_penalty(blueprint, pods, nodes, groups);

    let f_nodes = count_active_nodes(&blueprint.assignment, nodes.len()) as f64;
    let f_frag = compute_fragmentation(&blueprint.node_load, nodes);
    let f_aff = compute_affinity_violations(&blueprint.assignment, pods) as f64;
    let f_var = compute_utilization_variance(&blueprint.node_load, nodes);

    penalty
        + weights.node_count * f_nodes
        + weights.fragmentation * f_frag
        + weights.affinity_violation * f_aff
        + weights.utilization_variance * f_var
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
    }
    total_waste
}

/// f_affinity: count soft affinity/anti-affinity violations.
///
/// Affinity: pods that WANT to be on the same node but aren't.
/// Anti-affinity: pods that DON'T want to be on the same node but are.
pub fn compute_affinity_violations(assignment: &[usize], pods: &[Pod]) -> usize {
    let name_to_idx: HashMap<&str, usize> = pods
        .iter()
        .enumerate()
        .map(|(i, p)| (p.name.as_str(), i))
        .collect();

    let mut violations = 0_usize;
    for (i, pod) in pods.iter().enumerate() {
        for target_name in &pod.affinity_targets {
            if let Some(&target_idx) = name_to_idx.get(target_name.as_str()) {
                // WHY: only count once per pair — check i < target_idx
                if i < target_idx && assignment[i] != assignment[target_idx] {
                    violations += 1;
                }
            }
        }
        for target_name in &pod.anti_affinity_targets {
            if let Some(&target_idx) = name_to_idx.get(target_name.as_str())
                && i < target_idx && assignment[i] == assignment[target_idx]
            {
                violations += 1;
            }
        }
    }
    violations
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
) -> f64 {
    let mut total = 0.0_f64;

    let cap_overflow = compute_capacity_overflow(&blueprint.assignment, pods, nodes);
    if cap_overflow > 0.0 {
        total += 1_000_000.0 + cap_overflow * 10_000.0;
    }

    let sel_violations = compute_selector_violations(&blueprint.assignment, pods, nodes);
    if sel_violations > 0 {
        total += 500_000.0 + sel_violations as f64 * 50_000.0;
    }

    total += gang_penalty(blueprint, pods, nodes, groups);
    total
}

/// Soft penalty if any gang pod can't fit on its assigned node.
///
/// Penalty scales linearly with the number of missing pods to reach min_members.
fn gang_penalty(
    blueprint: &Blueprint,
    _pods: &[Pod],
    nodes: &[Node],
    groups: &[PodGroup],
) -> f64 {
    let mut penalty = 0.0_f64;
    for group in groups {
        let placed = group
            .pod_indices
            .iter()
            .filter(|&&pod_idx| {
                let node_idx = blueprint.assignment[pod_idx];
                nodes[node_idx].allocatable.fits(blueprint.node_load[node_idx])
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
        }
    }

    fn node(name: &str, cpu: f64, ram: f64) -> Node {
        Node {
            name: name.into(),
            allocatable: ResourceVector::new(cpu, ram, 0.0),
            labels: HashMap::new(),
            taints: vec![],
            zone: String::new(),
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
    fn affinity_violation_detected() {
        let pods = vec![
            Pod { affinity_targets: vec!["b".into()], ..pod("a") },
            pod("b"),
        ];
        assert_eq!(compute_affinity_violations(&[0, 1], &pods), 1);
    }

    #[test]
    fn affinity_satisfied_no_violation() {
        let pods = vec![
            Pod { affinity_targets: vec!["b".into()], ..pod("a") },
            pod("b"),
        ];
        assert_eq!(compute_affinity_violations(&[0, 0], &pods), 0);
    }

    #[test]
    fn anti_affinity_violation() {
        let pods = vec![
            Pod { anti_affinity_targets: vec!["b".into()], ..pod("a") },
            pod("b"),
        ];
        assert_eq!(compute_affinity_violations(&[0, 0], &pods), 1);
    }

    #[test]
    fn utilization_variance_balanced() {
        let loads = [ResourceVector::new(2.0, 0.0, 0.0), ResourceVector::new(2.0, 0.0, 0.0)];
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
        };
        let f = compute_fitness(&bp, &pods, &nodes, &[], &FitnessWeights::default());
        assert!(f > 1_000_000.0);
        assert!(f.is_finite());
    }
}
