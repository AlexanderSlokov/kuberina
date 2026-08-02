//! Phase 0: DaemonSet pre-deduction — subtract system overhead from node capacity.
//!
//! Ported from `research/src/kuberina/phases/phase0.py`.
//! Maritime analogy: Fill ballast tanks to establish baseline draft before cargo loading.
//! Formula from PAPER.md §3.2:
//!     C_j^r = C_{j,raw}^r − Σ 𝟙[eligible(d, n_j)] · res_d^r

use crate::model::{DaemonSet, Node, ResourceVector};

/// Check if a DaemonSet should run on a given node.
///
/// Matches nodeSelector labels and checks taint tolerations.
/// An empty selector means "run on all nodes" (typical for kube-proxy).
///
/// ```
/// # use kuberina_solver::model::*;
/// # use kuberina_solver::phase0::is_eligible;
/// # use std::collections::HashMap;
/// let ds = DaemonSet {
///     name: "gpu-plugin".into(),
///     resources: ResourceVector::zero(),
///     node_selector: HashMap::from([("gpu".into(), "true".into())]),
///     tolerations: vec![],
/// };
/// let node = Node {
///     name: "n1".into(),
///     allocatable: ResourceVector::zero(),
///     labels: HashMap::from([("gpu".into(), "true".into())]),
///     taints: vec![],
///     zone: String::new(),
/// };
/// assert!(is_eligible(&ds, &node));
/// ```
pub fn is_eligible(ds: &DaemonSet, node: &Node) -> bool {
    for (key, value) in &ds.node_selector {
        if node.labels.get(key) != Some(value) {
            return false;
        }
    }
    // WHY: DaemonSets typically tolerate all taints (they MUST run).
    // If DS has no tolerations specified, it tolerates everything.
    if ds.tolerations.is_empty() {
        return true;
    }
    node.taints.iter().all(|t| ds.tolerations.contains(t))
}

/// Subtract DaemonSet resource consumption from every eligible node.
///
/// After this, each node's allocatable = REAL capacity available for workload pods.
/// The optimizer never sees DaemonSet pods — they are fixed variables.
///
/// ```
/// # use kuberina_solver::model::*;
/// # use kuberina_solver::phase0::pre_deduct_daemonsets;
/// let nodes = vec![Node {
///     name: "n1".into(),
///     allocatable: ResourceVector::new(4.0, 16.0, 0.0),
///     labels: Default::default(), taints: vec![], zone: String::new(),
/// }];
/// let ds = vec![DaemonSet {
///     name: "kube-proxy".into(),
///     resources: ResourceVector::new(0.1, 0.064, 0.0),
///     node_selector: Default::default(), tolerations: vec![],
/// }];
/// let result = pre_deduct_daemonsets(&nodes, &ds);
/// assert!((result[0].allocatable.cpu - 3.9).abs() < 1e-9);
/// ```
pub fn pre_deduct_daemonsets(nodes: &[Node], daemon_sets: &[DaemonSet]) -> Vec<Node> {
    nodes
        .iter()
        .map(|node| {
            let overhead = daemon_sets
                .iter()
                .filter(|ds| is_eligible(ds, node))
                .fold(ResourceVector::zero(), |acc, ds| acc + ds.resources);

            Node {
                name: node.name.clone(),
                allocatable: node.allocatable.subtract(overhead),
                labels: node.labels.clone(),
                taints: node.taints.clone(),
                zone: node.zone.clone(),
                rack: node.rack.clone(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_node(name: &str, cpu: f64, ram: f64) -> Node {
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
    fn eligible_empty_selector_matches_all() {
        let ds = DaemonSet {
            name: "kube-proxy".into(),
            resources: ResourceVector::zero(),
            node_selector: HashMap::new(),
            tolerations: vec![],
        };
        let node = make_node("n1", 4.0, 16.0);
        assert!(is_eligible(&ds, &node));
    }

    #[test]
    fn eligible_selector_mismatch() {
        let ds = DaemonSet {
            name: "gpu-plugin".into(),
            resources: ResourceVector::zero(),
            node_selector: HashMap::from([("gpu".into(), "true".into())]),
            tolerations: vec![],
        };
        let node = make_node("n1", 4.0, 16.0);
        assert!(!is_eligible(&ds, &node));
    }

    #[test]
    fn pre_deduct_subtracts_overhead() {
        let nodes = vec![make_node("n1", 4.0, 16.0)];
        let ds = vec![DaemonSet {
            name: "kp".into(),
            resources: ResourceVector::new(0.5, 1.0, 0.0),
            node_selector: HashMap::new(),
            tolerations: vec![],
        }];
        let result = pre_deduct_daemonsets(&nodes, &ds);
        assert_eq!(result[0].allocatable.cpu, 3.5);
        assert_eq!(result[0].allocatable.ram, 15.0);
    }
}
