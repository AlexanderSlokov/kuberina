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

/// Largest capacity present in the cluster, per dimension.
///
/// Used to make `synthetic_volume` scale-invariant. A dimension no node declares
/// stays at `f64::MAX`, which divides pod demand down to nothing — correct, since
/// an unconstrained dimension cannot make a pod hard to place.
///
/// ```ignore
/// let scale = max_node_capacity(&nodes);
/// ```
pub fn max_node_capacity(nodes: &[Node]) -> ResourceVector {
    let mut max = ResourceVector::zero();
    for node in nodes {
        let c = &node.allocatable;
        max.cpu = max.cpu.max(c.cpu);
        max.ram = max.ram.max(c.ram);
        max.gpu = max.gpu.max(c.gpu);
        max.storage = max.storage.max(c.storage);
        max.disk_read = max.disk_read.max(c.disk_read);
        max.disk_write = max.disk_write.max(c.disk_write);
        max.net_in = max.net_in.max(c.net_in);
        max.net_out = max.net_out.max(c.net_out);
    }
    max
}

/// One dimension's share of the largest node's capacity in that dimension.
///
/// ```ignore
/// capacity_share(32.0, 64.0)  // 0.5 — half of the biggest node's CPU
/// ```
fn capacity_share(demand: f64, max_capacity: f64) -> f64 {
    if max_capacity <= 0.0 || !max_capacity.is_finite() {
        return 0.0;
    }
    demand / max_capacity
}

/// Compute scalar weight for a pod based on normalized resource scarcity.
///
/// V_i = Σ_r w_r · (req_i^r / max_j C_j^r)
///
/// Higher V means the pod is "heavier" and should be placed first.
///
/// WHY normalize: summing raw magnitudes across dimensions compares quantities in
/// different units — 500 units of net_in is not 500 GiB of RAM. The previous form
/// compensated with small I/O weights (0.01 against 1.0 for CPU), which hardcoded
/// a belief about which dimension binds. On a testbed where disk_write is the
/// scarce dimension that ordering placed the I/O-heavy pods last, by which point
/// no node had I/O headroom left, and 15 of them fell through to the overloading
/// fallback below. Dividing by the largest node capacity in each dimension makes
/// the terms comparable, so the weights express preference rather than unit
/// conversion. See issue #18.
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
///     observed: None,
/// };
/// // Against a cluster whose biggest node is 64 CPU / 256 RAM / 8 GPU:
/// let scale = ResourceVector::new(64.0, 256.0, 8.0);
/// let v = synthetic_volume(&pod, &FfdWeights::default(), &scale);
/// // 1.0*(8/64) + 1.0*(64/256) + 10.0*(1/8) = 0.125 + 0.25 + 1.25
/// assert!((v - 1.625).abs() < 1e-9);
/// ```
pub fn synthetic_volume(pod: &Pod, weights: &FfdWeights, scale: &ResourceVector) -> f64 {
    let r = &pod.requests;
    weights.alpha * capacity_share(r.cpu, scale.cpu)
        + weights.beta * capacity_share(r.ram, scale.ram)
        + weights.gamma * capacity_share(r.gpu, scale.gpu)
        + weights.delta * capacity_share(r.storage, scale.storage)
        + weights.epsilon_r * capacity_share(r.disk_read, scale.disk_read)
        + weights.epsilon_w * capacity_share(r.disk_write, scale.disk_write)
        + weights.zeta_in * capacity_share(r.net_in, scale.net_in)
        + weights.zeta_out * capacity_share(r.net_out, scale.net_out)
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
///     observed: None,
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
    let scale = max_node_capacity(nodes);
    let mut sorted_indices: Vec<usize> = (0..num_pods).collect();
    sorted_indices.sort_by(|&a, &b| {
        synthetic_volume(&pods[b], weights, &scale)
            .partial_cmp(&synthetic_volume(&pods[a], weights, &scale))
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
            observed: None,
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
        let pods = vec![pod("small", 1.0, 2.0), pod("big", 3.0, 12.0)];
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
    fn synthetic_volume_scales_by_cluster_capacity() {
        let p = pod("gpu", 8.0, 64.0);
        let p_gpu = Pod {
            requests: ResourceVector::new(8.0, 64.0, 1.0),
            ..p
        };
        let scale = ResourceVector::new(64.0, 256.0, 8.0);
        let v = synthetic_volume(&p_gpu, &FfdWeights::default(), &scale);
        // 1.0*(8/64) + 1.0*(64/256) + 10.0*(1/8) = 0.125 + 0.25 + 1.25
        assert!((v - 1.625).abs() < 1e-9);
    }

    #[test]
    fn synthetic_volume_ranks_by_scarcity_not_magnitude() {
        // A pod wanting most of the cluster's disk_write should outrank one
        // wanting a sliver of its much larger network budget, even though the
        // latter's raw numbers are an order of magnitude bigger. This ordering
        // is what the pre-#18 formula got backwards.
        let base = pod("x", 1.0, 1.0);
        let io_heavy = Pod {
            requests: ResourceVector::new_8d(1.0, 1.0, 0.0, 0.0, 0.0, 180.0, 0.0, 0.0),
            ..base.clone()
        };
        let net_light = Pod {
            requests: ResourceVector::new_8d(1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 500.0, 0.0),
            ..base
        };
        let scale =
            ResourceVector::new_8d(64.0, 256.0, 8.0, 1000.0, 500.0, 200.0, 10000.0, 10000.0);
        let w = FfdWeights::default();
        assert!(synthetic_volume(&io_heavy, &w, &scale) > synthetic_volume(&net_light, &w, &scale));
    }

    #[test]
    fn unconstrained_dimension_contributes_nothing() {
        let p = Pod {
            requests: ResourceVector::new_8d(1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 900.0, 0.0),
            ..pod("x", 1.0, 1.0)
        };
        let mut scale = ResourceVector::new(64.0, 256.0, 8.0);
        scale.net_in = f64::MAX;
        let w = FfdWeights::default();
        let with_net = synthetic_volume(&p, &w, &scale);
        let expected = 1.0 / 64.0 + 1.0 / 256.0;
        assert!((with_net - expected).abs() < 1e-9);
    }

    #[test]
    fn max_node_capacity_takes_the_largest_per_dimension() {
        let mut gpu_node = node("gpu", 32.0, 192.0);
        gpu_node.allocatable.gpu = 8.0;
        let nodes = vec![
            node("small", 16.0, 128.0),
            node("big-cpu", 64.0, 64.0),
            gpu_node,
        ];
        let max = max_node_capacity(&nodes);
        assert_eq!(max.cpu, 64.0);
        assert_eq!(max.ram, 192.0);
        assert_eq!(max.gpu, 8.0);
    }
}
