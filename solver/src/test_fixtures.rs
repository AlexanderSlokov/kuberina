//! Shared test fixtures: per-device GPU (X-3), admission (X-4), and GA moves within
//! capacity (X-8). Compiled for tests only.

use std::collections::HashMap;

use crate::model::{GpuShare, Node, Pod, ResourceVector, UNPLACED};
use crate::phase1_ffd::compute_node_loads;

/// A pod with one CPU, one GiB, and the given per-device GPU share.
pub fn shared_gpu_pod(name: &str, milli: u32) -> Pod {
    Pod {
        name: name.into(),
        namespace: "ns".into(),
        requests: ResourceVector::new(1.0, 1.0, f64::from(milli) / 1000.0),
        tolerations: vec![],
        node_selector: HashMap::new(),
        affinity_targets: vec![],
        anti_affinity_targets: vec![],
        group_name: String::new(),
        topology_spread: None,
        observed: None,
        gpu_share: Some(GpuShare { devices: 1, milli }),
        arrival: None,
    }
}

/// Eight 600-milli pods, then one 500-milli pod: the X-3 done-when case.
pub fn sixty_percent_then_half() -> Vec<Pod> {
    let mut pods: Vec<Pod> = (0..8)
        .map(|i| shared_gpu_pod(&format!("sixty-{i}"), 600))
        .collect();
    pods.push(shared_gpu_pod("half", 500));
    pods
}

/// A node with 64 cores, 256 GiB, and eight GPU devices.
pub fn eight_gpu_node(name: &str) -> Node {
    Node {
        name: name.into(),
        allocatable: ResourceVector::new(64.0, 256.0, 8.0),
        labels: HashMap::new(),
        taints: vec![],
        zone: String::new(),
        rack: String::new(),
        gpu_devices: Some(8),
    }
}

/// A CPU-only pod requesting `cpu` cores and nothing else.
pub fn cpu_pod(name: &str, cpu: f64) -> Pod {
    Pod {
        requests: ResourceVector::new(cpu, 0.0, 0.0),
        gpu_share: None,
        ..shared_gpu_pod(name, 1)
    }
}

/// A node with `cpu` cores and no other declared capacity.
pub fn cpu_node(name: &str, cpu: f64) -> Node {
    Node {
        allocatable: ResourceVector::new(cpu, 0.0, 0.0),
        gpu_devices: None,
        ..eight_gpu_node(name)
    }
}

/// Two full four-core nodes and one with room for one more two-core Pod (X-8).
///
/// Five two-core Pods are placed; the last Pod, `wide`, needs three cores and fits nowhere.
pub fn nearly_full_cluster() -> (Vec<Pod>, Vec<Node>, Vec<usize>) {
    let mut pods: Vec<Pod> = (0..5).map(|i| cpu_pod(&format!("p{i}"), 2.0)).collect();
    pods.push(cpu_pod("wide", 3.0));
    let nodes = vec![cpu_node("a", 4.0), cpu_node("b", 4.0), cpu_node("c", 4.0)];
    (pods, nodes, vec![0, 0, 1, 1, 2, UNPLACED])
}

/// Whether any node carries more than its allocatable capacity.
pub fn breaches_a_node(assignment: &[usize], pods: &[Pod], nodes: &[Node]) -> bool {
    let loads = compute_node_loads(assignment, pods, nodes.len());
    nodes
        .iter()
        .zip(loads)
        .any(|(n, load)| !n.allocatable.fits(load))
}
