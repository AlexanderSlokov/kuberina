//! Phase 1: FFD Warm-Start — generate a feasible seed blueprint via greedy packing.
//!
//! Ported from `research/src/kuberina/phases/phase1_ffd.py`.
//! Maritime analogy: Stack the heaviest containers first, fill gaps with smaller ones.
//! Formula from PAPER.md §4.2:
//!     V_i = α·CPU_i + β·RAM_i + γ·GPU_i
//!     Sort pods descending by V_i, first-fit into nodes.
//! Complexity: O(k·log(k) + k·m) — from PAPER.md §4.2.

use crate::csp::can_place_pod_on_node;
use crate::model::{Blueprint, FfdWeights, Node, Pod, ResourceVector};

/// Compute scalar weight for a pod based on normalized resource scarcity.
///
/// V_i = α·CPU_i + β·RAM_i + γ·GPU_i
/// Higher V means the pod is "heavier" and should be placed first.
///
/// ```
/// # use kuberina_solver::model::*;
/// # use kuberina_solver::phase1_ffd::synthetic_volume;
/// let pod = Pod {
///     name: "gpu-worker".into(), namespace: "ai".into(),
///     requests: ResourceVector::new(8.0, 64.0, 1.0),
///     tolerations: vec![], node_selector: Default::default(),
///     affinity_targets: vec![],
///     anti_affinity_targets: vec![],
///     group_name: String::new(),
///     topology_spread: None,
/// };
/// let v = synthetic_volume(&pod, &FfdWeights::default());
/// assert!((v - 82.0).abs() < 1e-9);
/// ```
pub fn synthetic_volume(pod: &Pod, weights: &FfdWeights) -> f64 {
    let r = &pod.requests;
    weights.alpha * r.cpu
        + weights.beta * r.ram
        + weights.gamma * r.gpu
        + weights.delta * r.storage
        + weights.epsilon_r * r.disk_read
        + weights.epsilon_w * r.disk_write
        + weights.zeta_in * r.net_in
        + weights.zeta_out * r.net_out
}

/// Accumulate per-node resource usage from pod assignments.
pub fn compute_node_loads(
    assignment: &[usize],
    pods: &[Pod],
    num_nodes: usize,
) -> Vec<ResourceVector> {
    let mut loads = vec![ResourceVector::zero(); num_nodes];
    for (pod_idx, &node_idx) in assignment.iter().enumerate() {
        loads[node_idx] = loads[node_idx] + pods[pod_idx].requests;
    }
    loads
}

/// Generate a feasible initial blueprint using First-Fit Decreasing.
///
/// Pods are sorted by synthetic volume (descending) and greedily placed
/// into the first node with sufficient residual capacity.
/// Accelerates GA convergence 3-5x vs random init (ga_estimation.md §5).
///
/// ```
/// # use kuberina_solver::model::*;
/// # use kuberina_solver::phase1_ffd::ffd_warmstart;
/// # use std::collections::HashMap;
/// let pods = vec![Pod {
///     name: "big".into(), namespace: "ns".into(),
///     requests: ResourceVector::new(2.0, 8.0, 0.0),
///     tolerations: vec![], node_selector: HashMap::new(),
///     affinity_targets: vec![],
///     anti_affinity_targets: vec![],
///     group_name: String::new(),
///     topology_spread: None,
/// }];
/// let nodes = vec![Node {
///     name: "n1".into(), allocatable: ResourceVector::new(4.0, 16.0, 0.0),
///     labels: HashMap::new(),
///     taints: vec![],
///     zone: String::new(),
///     rack: String::new(),
/// }];
/// let bp = ffd_warmstart(&pods, &nodes, &FfdWeights::default());
/// assert_eq!(bp.assignment, vec![0]);
/// ```
pub fn ffd_warmstart(pods: &[Pod], nodes: &[Node], weights: &FfdWeights) -> Blueprint {
    let num_pods = pods.len();
    let num_nodes = nodes.len();

    // Sort pod indices by synthetic volume descending (heaviest first)
    let mut sorted_indices: Vec<usize> = (0..num_pods).collect();
    sorted_indices.sort_by(|&a, &b| {
        synthetic_volume(&pods[b], weights)
            .partial_cmp(&synthetic_volume(&pods[a], weights))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut assignment = vec![0_usize; num_pods];
    let mut residual: Vec<ResourceVector> = nodes.iter().map(|n| n.allocatable).collect();

    for pod_idx in sorted_indices {
        let mut placed = false;
        for node_idx in 0..num_nodes {
            if !can_place_pod_on_node(&pods[pod_idx], &nodes[node_idx]) {
                continue;
            }
            if !residual[node_idx].fits(pods[pod_idx].requests) {
                continue;
            }

            assignment[pod_idx] = node_idx;
            residual[node_idx] = residual[node_idx].subtract(pods[pod_idx].requests);
            placed = true;
            break;
        }

        if !placed {
            // WHY: pod cannot fit anywhere — fallback to the first node that matches selector/taint.
            // This prevents adding artificial Selector violations which the GA cannot mutate out of
            // due to massive capacity penalty spikes.
            let fallback = (0..num_nodes)
                .find(|&i| crate::csp::can_place_pod_on_node(&pods[pod_idx], &nodes[i]))
                .unwrap_or(0);
                
            assignment[pod_idx] = fallback;
            residual[fallback] = residual[fallback].subtract(pods[pod_idx].requests);
        }
    }

    let node_load = compute_node_loads(&assignment, pods, num_nodes);
    Blueprint {
        assignment,
        fitness: 0.0,
        node_load,
        scorecard: Default::default(),
    }
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
    fn ffd_single_pod_single_node() {
        let pods = vec![pod("p", 2.0, 8.0)];
        let nodes = vec![node("n", 4.0, 16.0)];
        let bp = ffd_warmstart(&pods, &nodes, &FfdWeights::default());
        assert_eq!(bp.assignment, vec![0]);
    }

    #[test]
    fn ffd_heaviest_first_fills_tightly() {
        let pods = vec![
            pod("small", 1.0, 2.0),
            pod("big", 3.0, 12.0),
        ];
        let nodes = vec![node("n", 4.0, 16.0)];
        let bp = ffd_warmstart(&pods, &nodes, &FfdWeights::default());
        // Both should fit on node 0 (big=3+12=15 volume, small=1+2=3)
        assert_eq!(bp.assignment[0], 0);
        assert_eq!(bp.assignment[1], 0);
    }

    #[test]
    fn ffd_overflow_falls_back_to_node_0() {
        let pods = vec![pod("huge", 10.0, 100.0)];
        let nodes = vec![node("n", 4.0, 16.0)];
        let bp = ffd_warmstart(&pods, &nodes, &FfdWeights::default());
        assert_eq!(bp.assignment, vec![0]); // fallback
    }

    #[test]
    fn synthetic_volume_calculation() {
        let p = pod("gpu", 8.0, 64.0);
        let p_gpu = Pod {
            requests: ResourceVector::new(8.0, 64.0, 1.0),
            ..p
        };
        let v = synthetic_volume(&p_gpu, &FfdWeights::default());
        // 1.0*8 + 1.0*64 + 10.0*1 = 82.0
        assert!((v - 82.0).abs() < 1e-9);
    }
}
