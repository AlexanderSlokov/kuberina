//! CSP Solver — hard constraint pre-screening for every placement decision.
//!
//! Ported from `research/src/kuberina/phases/csp.py`.
//! Integrated into FFD (Phase 1) and GA mutation/repair (Phase 2).
//! From PAPER.md §4.4 and DESIGN.md §CSP Forward Checking.
//!
//! Hard constraints (PAPER.md §3.2):
//!   1. Capacity:        Σ x_ij · req_i^r ≤ C_j^r
//!   2. Assignment:      Σ x_ij = 1 (handled by chromosome encoding)
//!   3. Taint/Toleration: Taints(n_j) ⊆ Tolerations(p_i)
//!   4. NodeSelector:    Labels(n_j) ⊇ Selector(p_i)
//!   5. Gang:            All pods in group feasible, or none

use crate::model::{Node, Pod, PodGroup, ResourceVector};

/// Pod can only land on a tainted node if it has matching tolerations.
///
/// From PAPER.md §3.2 Hard Constraint #3.
pub fn check_taint_toleration(pod: &Pod, node: &Node) -> bool {
    node.taints.iter().all(|t| pod.tolerations.contains(t))
}

/// Pod can only land on nodes matching its selector labels.
///
/// From PAPER.md §3.2 Hard Constraint #4.
pub fn check_node_selector(pod: &Pod, node: &Node) -> bool {
    pod.node_selector
        .iter()
        .all(|(k, v)| node.labels.get(k) == Some(v))
}

/// Combined pre-screen: taint + nodeSelector (excludes capacity check).
///
/// Used by FFD and GA mutation before checking residual capacity.
pub fn can_place_pod_on_node(pod: &Pod, node: &Node) -> bool {
    check_taint_toleration(pod, node) && check_node_selector(pod, node)
}

/// Compute continuous penalty scalar for capacity overflow on all nodes.
///
/// Instead of boolean pass/fail, returns total resource over-commitment.
/// Overflow = Σ max(0, load − cap) for CPU, RAM, GPU.
pub fn compute_capacity_overflow(assignment: &[usize], pods: &[Pod], nodes: &[Node]) -> f64 {
    let by_dim = compute_overflow_by_dimension(assignment, pods, nodes);
    by_dim.cpu
        + by_dim.ram
        + by_dim.gpu
        + by_dim.storage
        + by_dim.disk_read
        + by_dim.disk_write
        + by_dim.net_in
        + by_dim.net_out
}

/// Capacity overflow per dimension, summed across every node.
///
/// The scalar `compute_capacity_overflow` collapses this into one number for the
/// fitness function. Reporting needs the breakdown: "over by 11,328 on disk_write"
/// tells an operator what to change, where a single total does not.
///
/// ```ignore
/// let over = compute_overflow_by_dimension(&assignment, &pods, &nodes);
/// assert_eq!(over.disk_write, 0.0);
/// ```
pub fn compute_overflow_by_dimension(
    assignment: &[usize],
    pods: &[Pod],
    nodes: &[Node],
) -> ResourceVector {
    let mut loads = vec![ResourceVector::zero(); nodes.len()];
    for (pod_idx, &node_idx) in assignment.iter().enumerate() {
        loads[node_idx] = loads[node_idx] + pods[pod_idx].requests;
    }

    let mut over = ResourceVector::zero();
    for (node_idx, load) in loads.iter().enumerate() {
        let cap = &nodes[node_idx].allocatable;
        over.cpu += (load.cpu - cap.cpu).max(0.0);
        over.ram += (load.ram - cap.ram).max(0.0);
        over.gpu += (load.gpu - cap.gpu).max(0.0);
        over.storage += (load.storage - cap.storage).max(0.0);
        // WHY: Disk/Network overflow is the Noisy Neighbor signal —
        // 10 DB pods crushing a node's IOPS triggers massive penalty.
        over.disk_read += (load.disk_read - cap.disk_read).max(0.0);
        over.disk_write += (load.disk_write - cap.disk_write).max(0.0);
        over.net_in += (load.net_in - cap.net_in).max(0.0);
        over.net_out += (load.net_out - cap.net_out).max(0.0);
    }
    over
}

/// Count number of pods placed on nodes violating Taint or NodeSelector.
pub fn compute_selector_violations(assignment: &[usize], pods: &[Pod], nodes: &[Node]) -> usize {
    assignment
        .iter()
        .enumerate()
        .filter(|&(pod_idx, &node_idx)| !can_place_pod_on_node(&pods[pod_idx], &nodes[node_idx]))
        .count()
}

/// Forward checking for gang scheduling — verify group is feasible.
///
/// From DESIGN.md canPlaceGang pseudocode.
/// 1. Enough eligible nodes for all pods in gang?
/// 2. Total residual capacity >= total demand?
/// 3. If colocate: any single node fits the entire gang?
pub fn can_place_gang(
    group: &PodGroup,
    nodes: &[Node],
    node_load: &[ResourceVector],
    pods: &[Pod],
) -> bool {
    let eligible = filter_eligible_nodes(group, nodes, pods);
    if eligible.is_empty() {
        return false;
    }

    let total_demand = sum_gang_demand(group, pods);
    let total_avail = sum_residual_capacity(&eligible, nodes, node_load);
    if !total_avail.fits(total_demand) {
        return false;
    }

    if group.colocate {
        return eligible.iter().any(|&idx| {
            nodes[idx]
                .allocatable
                .subtract(node_load[idx])
                .fits(total_demand)
        });
    }

    true
}

/// Return indices of nodes eligible for all pods in this gang.
fn filter_eligible_nodes(group: &PodGroup, nodes: &[Node], pods: &[Pod]) -> Vec<usize> {
    // WHY: use group's nodeSelector, not individual pod selectors,
    // because gang pods share the same hardware requirement (e.g., GPU+NVLink).
    let sample_pod = &pods[group.pod_indices[0]];
    nodes
        .iter()
        .enumerate()
        .filter(|(_, node)| {
            group
                .node_selector
                .iter()
                .all(|(k, v)| node.labels.get(k) == Some(v))
                && check_taint_toleration(sample_pod, node)
        })
        .map(|(i, _)| i)
        .collect()
}

/// Sum resource requests of all pods in a gang.
fn sum_gang_demand(group: &PodGroup, pods: &[Pod]) -> ResourceVector {
    group
        .pod_indices
        .iter()
        .fold(ResourceVector::zero(), |acc, &i| acc + pods[i].requests)
}

/// Sum remaining capacity across all eligible nodes.
fn sum_residual_capacity(
    eligible: &[usize],
    nodes: &[Node],
    node_load: &[ResourceVector],
) -> ResourceVector {
    eligible.iter().fold(ResourceVector::zero(), |acc, &idx| {
        acc + nodes[idx].allocatable.subtract(node_load[idx])
    })
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
    fn taint_toleration_no_taint_passes() {
        assert!(check_taint_toleration(&pod("p"), &node("n", 4.0, 16.0)));
    }

    #[test]
    fn taint_toleration_without_matching_toleration_fails() {
        let n = Node {
            taints: vec!["gpu-only".into()],
            ..node("n", 4.0, 16.0)
        };
        assert!(!check_taint_toleration(&pod("p"), &n));
    }

    #[test]
    fn taint_toleration_with_matching_toleration_passes() {
        let p = Pod {
            tolerations: vec!["gpu-only".into()],
            ..pod("p")
        };
        let n = Node {
            taints: vec!["gpu-only".into()],
            ..node("n", 4.0, 16.0)
        };
        assert!(check_taint_toleration(&p, &n));
    }

    #[test]
    fn node_selector_empty_passes() {
        assert!(check_node_selector(&pod("p"), &node("n", 4.0, 16.0)));
    }

    #[test]
    fn node_selector_match_passes() {
        let p = Pod {
            node_selector: HashMap::from([("zone".into(), "us-east".into())]),
            ..pod("p")
        };
        let n = Node {
            labels: HashMap::from([("zone".into(), "us-east".into())]),
            ..node("n", 4.0, 16.0)
        };
        assert!(check_node_selector(&p, &n));
    }

    #[test]
    fn node_selector_mismatch_fails() {
        let p = Pod {
            node_selector: HashMap::from([("zone".into(), "us-east".into())]),
            ..pod("p")
        };
        assert!(!check_node_selector(&p, &node("n", 4.0, 16.0)));
    }

    #[test]
    fn capacity_overflow_no_overflow() {
        let pods = vec![Pod {
            requests: ResourceVector::new(2.0, 8.0, 0.0),
            ..pod("p")
        }];
        let nodes = vec![node("n", 4.0, 16.0)];
        assert_eq!(compute_capacity_overflow(&[0], &pods, &nodes), 0.0);
    }

    #[test]
    fn capacity_overflow_detects_cpu_overflow() {
        let pods = vec![Pod {
            requests: ResourceVector::new(6.0, 8.0, 0.0),
            ..pod("p")
        }];
        let nodes = vec![node("n", 4.0, 16.0)];
        assert_eq!(compute_capacity_overflow(&[0], &pods, &nodes), 2.0);
    }

    #[test]
    fn selector_violations_count() {
        let p = Pod {
            node_selector: HashMap::from([("gpu".into(), "true".into())]),
            ..pod("p")
        };
        let n = node("n", 4.0, 16.0); // no labels
        assert_eq!(compute_selector_violations(&[0], &[p], &[n]), 1);
    }

    #[test]
    fn gang_feasibility_basic() {
        let pods = vec![
            Pod {
                requests: ResourceVector::new(1.0, 1.0, 0.0),
                ..pod("p0")
            },
            Pod {
                requests: ResourceVector::new(1.0, 1.0, 0.0),
                ..pod("p1")
            },
        ];
        let nodes = vec![node("n0", 4.0, 16.0)];
        let group = PodGroup {
            name: "g".into(),
            pod_indices: vec![0, 1],
            min_members: 2,
            node_selector: HashMap::new(),
            colocate: false,
        };
        assert!(can_place_gang(
            &group,
            &nodes,
            &[ResourceVector::zero()],
            &pods,
        ));
    }

    #[test]
    fn capacity_overflow_detects_disk_read_overflow() {
        let pods = vec![Pod {
            requests: ResourceVector::new_8d(1.0, 1.0, 0.0, 0.0, 600.0, 0.0, 0.0, 0.0),
            ..pod("db")
        }];
        let nodes = vec![Node {
            allocatable: ResourceVector::new_8d(
                4.0, 16.0, 0.0, 100.0, 500.0, 500.0, 1000.0, 1000.0,
            ),
            ..node("n", 4.0, 16.0)
        }];
        // disk_read overflow: 600 - 500 = 100
        assert!((compute_capacity_overflow(&[0], &pods, &nodes) - 100.0).abs() < 1e-9);
    }
}
