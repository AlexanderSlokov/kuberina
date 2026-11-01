//! Per-device GPU feasibility (IR v1 §3.4, backlog X-3).
//!
//! Summing GPU per node accepts a half-GPU pod onto a node whose eight devices are
//! each 60% used, because 3.2 GPUs are free in total. That placement is impossible on
//! the real node. On a node with `gpu_devices`, the pods placed there must also fit
//! device by device.
//!
//! Finding a device assignment is itself a packing problem, so this module decodes
//! with a fixed heuristic: requests in descending `milli` order, each onto the devices
//! with the least room that still fits it (best fit). The heuristic may decline a set
//! that fits; it never accepts one that does not, which is what IR v1 §3.4 requires.
//! The inspector checks the same rule with an exhaustive search of its own.

use crate::model::{GpuShare, Node, Pod};

/// Capacity of one GPU device in milli-GPU.
pub const MILLI_PER_DEVICE: u32 = 1000;

/// The per-device demand of a pod, or `None` when it requests no GPU.
///
/// A pod without `gpuMilli` uses whole devices. Its `requests.gpu` is a whole number
/// whenever a node declares devices, which `check_device_inputs` enforces at load.
///
/// ```ignore
/// assert_eq!(device_demand(&trainer), Some(GpuShare { devices: 8, milli: 1000 }));
/// ```
pub fn device_demand(pod: &Pod) -> Option<GpuShare> {
    if pod.gpu_share.is_some() {
        return pod.gpu_share;
    }
    if pod.requests.gpu <= 0.0 {
        return None;
    }
    Some(GpuShare {
        devices: pod.requests.gpu.round() as u32,
        milli: MILLI_PER_DEVICE,
    })
}

/// Milli-GPU of the demands that the decode could not place on `device_count` devices.
///
/// ```
/// # use kuberina_solver::gpu_devices::unplaced_device_milli;
/// # use kuberina_solver::model::GpuShare;
/// let sixty_percent = GpuShare { devices: 1, milli: 600 };
/// let half = GpuShare { devices: 1, milli: 500 };
/// let mut demands = vec![sixty_percent; 8];
/// demands.push(half);
/// assert_eq!(unplaced_device_milli(8, &demands), 500);
/// ```
pub fn unplaced_device_milli(device_count: u32, demands: &[GpuShare]) -> u64 {
    let mut ordered = demands.to_vec();
    // WHY descending: large requests placed late find every device partly used.
    ordered.sort_by(|a, b| b.milli.cmp(&a.milli).then(b.devices.cmp(&a.devices)));
    let mut free = vec![MILLI_PER_DEVICE; device_count as usize];
    ordered
        .iter()
        .map(|&share| place_share(&mut free, share))
        .sum()
}

/// Place one share best-fit; return its milli-GPU if it found too few devices.
fn place_share(free: &mut [u32], share: GpuShare) -> u64 {
    let mut candidates: Vec<usize> = (0..free.len())
        .filter(|&d| free[d] >= share.milli)
        .collect();
    if candidates.len() < share.devices as usize {
        return share.total_milli();
    }
    candidates.sort_by_key(|&d| (free[d], d));
    for &d in &candidates[..share.devices as usize] {
        free[d] -= share.milli;
    }
    0
}

/// GPUs per node that found no device, in devices; zero on nodes without devices.
///
/// The capacity penalty and the feasibility verdict add this to the per-node GPU sum
/// (`csp::compute_overflow_by_dimension`).
///
/// ```ignore
/// let stranded = device_overflow_per_node(&blueprint.assignment, &pods, &nodes);
/// ```
pub fn device_overflow_per_node(assignment: &[usize], pods: &[Pod], nodes: &[Node]) -> Vec<f64> {
    let mut over = vec![0.0; nodes.len()];
    // WHY early return: a cluster without devices (MSC Irina) pays one scan, no decode.
    if nodes.iter().all(|n| n.gpu_devices.is_none()) {
        return over;
    }
    let mut ledger = DeviceLedger::new(nodes.len());
    for (pod_idx, &node_idx) in assignment.iter().enumerate() {
        ledger.record(node_idx, &pods[pod_idx]);
    }
    for (node_idx, node) in nodes.iter().enumerate() {
        over[node_idx] = ledger.unplaced_milli(node, node_idx) as f64 / f64::from(MILLI_PER_DEVICE);
    }
    over
}

/// Whether the pods assigned to one node fit its devices; true without devices.
///
/// ```ignore
/// let ok = devices_fit_on_node(node_idx, &blueprint.assignment, &pods, &nodes);
/// ```
pub fn devices_fit_on_node(
    node_idx: usize,
    assignment: &[usize],
    pods: &[Pod],
    nodes: &[Node],
) -> bool {
    let Some(device_count) = nodes[node_idx].gpu_devices else {
        return true;
    };
    let demands: Vec<GpuShare> = assignment
        .iter()
        .zip(pods)
        .filter(|&(&assigned, _)| assigned == node_idx)
        .filter_map(|(_, pod)| device_demand(pod))
        .collect();
    unplaced_device_milli(device_count, &demands) == 0
}

/// The device demands placed on each node so far, for greedy placement (FFD).
///
/// ```ignore
/// let mut ledger = DeviceLedger::new(nodes.len());
/// if ledger.admits(&nodes[j], j, &pod) { ledger.record(j, &pod); }
/// ```
pub struct DeviceLedger {
    demands: Vec<Vec<GpuShare>>,
}

impl DeviceLedger {
    /// An empty ledger for `num_nodes` nodes.
    pub fn new(num_nodes: usize) -> Self {
        Self {
            demands: vec![Vec::new(); num_nodes],
        }
    }

    /// Whether `pod` still fits the devices of `node` (index `node_idx`).
    pub fn admits(&self, node: &Node, node_idx: usize, pod: &Pod) -> bool {
        let (Some(device_count), Some(demand)) = (node.gpu_devices, device_demand(pod)) else {
            return true;
        };
        let mut trial = self.demands[node_idx].clone();
        trial.push(demand);
        unplaced_device_milli(device_count, &trial) == 0
    }

    /// Add `pod`'s device demand to node `node_idx`, whether or not it fits.
    pub fn record(&mut self, node_idx: usize, pod: &Pod) {
        if let Some(demand) = device_demand(pod) {
            self.demands[node_idx].push(demand);
        }
    }

    /// Milli-GPU on node `node_idx` that the decode could not place; 0 without devices.
    fn unplaced_milli(&self, node: &Node, node_idx: usize) -> u64 {
        node.gpu_devices.map_or(0, |count| {
            unplaced_device_milli(count, &self.demands[node_idx])
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ResourceVector;
    use std::collections::HashMap;

    const WHOLE: GpuShare = GpuShare {
        devices: 1,
        milli: 1000,
    };

    fn share(devices: u32, milli: u32) -> GpuShare {
        GpuShare { devices, milli }
    }

    fn gpu_pod(gpu: f64, gpu_share: Option<GpuShare>) -> Pod {
        Pod {
            name: "p".into(),
            namespace: "ns".into(),
            requests: ResourceVector::new(1.0, 1.0, gpu),
            tolerations: vec![],
            node_selector: HashMap::new(),
            affinity_targets: vec![],
            anti_affinity_targets: vec![],
            group_name: String::new(),
            topology_spread: None,
            observed: None,
            gpu_share,
        }
    }

    #[test]
    fn half_gpu_is_rejected_on_eight_devices_each_sixty_percent_used() {
        // X-3 done-when: 3.2 GPUs are free in total, but no device has 500 milli.
        let mut demands = vec![share(1, 600); 8];
        demands.push(share(1, 500));
        assert_eq!(unplaced_device_milli(8, &demands), 500);
    }

    #[test]
    fn fractions_that_sum_to_one_device_share_it() {
        assert_eq!(unplaced_device_milli(1, &[share(1, 600), share(1, 400)]), 0);
    }

    #[test]
    fn multi_device_request_needs_that_many_distinct_devices() {
        // Two devices with 500 free each hold 1,000 milli in total, not one 2×500 pod
        // on a single device; the pod needs both devices.
        assert_eq!(unplaced_device_milli(2, &[share(2, 500), share(2, 500)]), 0);
        assert_eq!(
            unplaced_device_milli(2, &[share(2, 500), share(2, 600)]),
            1000
        );
    }

    #[test]
    fn whole_device_requests_take_empty_devices_before_fractions() {
        // 2,200 milli on 2,000: the two whole requests get the devices, and the two
        // 100-milli requests are what is left over.
        assert_eq!(
            unplaced_device_milli(2, &[share(1, 100), share(1, 100), WHOLE, WHOLE]),
            200
        );
    }

    #[test]
    fn larger_requests_are_placed_first_whatever_the_input_order() {
        // In input order, the two 300s would share a device and strand 400 milli on
        // it, leaving no device for the second 700.
        let demands = [share(1, 300), share(1, 300), share(1, 700), share(1, 700)];
        assert_eq!(unplaced_device_milli(2, &demands), 0);
    }

    #[test]
    fn decode_may_decline_a_set_that_fits() {
        // IR v1 §3.4 allows this and forbids the opposite. {500, 300, 200} and
        // {400, 300, 300} fill two devices exactly; best fit puts 400 next to 500 and
        // strands the 200. The inspector's exhaustive search accepts this set.
        let demands: Vec<GpuShare> = [500, 400, 300, 300, 300, 200].map(|m| share(1, m)).to_vec();
        assert_eq!(unplaced_device_milli(2, &demands), 200);
    }

    #[test]
    fn no_devices_leave_every_request_unplaced() {
        assert_eq!(unplaced_device_milli(0, &[share(2, 1000)]), 2000);
    }

    #[test]
    fn device_demand_of_a_shared_pod_is_its_share() {
        assert_eq!(
            device_demand(&gpu_pod(0.5, Some(share(1, 500)))),
            Some(share(1, 500))
        );
    }

    #[test]
    fn device_demand_of_a_whole_gpu_pod_is_whole_devices() {
        assert_eq!(device_demand(&gpu_pod(8.0, None)), Some(share(8, 1000)));
    }

    #[test]
    fn cpu_only_pod_has_no_device_demand() {
        assert_eq!(device_demand(&gpu_pod(0.0, None)), None);
    }

    fn device_node(devices: Option<u32>) -> Node {
        Node {
            name: "gpu-node-01".into(),
            allocatable: ResourceVector::new(64.0, 256.0, 8.0),
            labels: HashMap::new(),
            taints: vec![],
            zone: String::new(),
            rack: String::new(),
            gpu_devices: devices,
        }
    }

    use crate::gpu_test_fixtures::sixty_percent_then_half;

    #[test]
    fn overflow_reports_the_stranded_half_gpu_in_devices() {
        let nodes = [device_node(Some(8))];
        let over = device_overflow_per_node(&[0; 9], &sixty_percent_then_half(), &nodes);
        assert_eq!(over, vec![0.5]);
    }

    #[test]
    fn node_without_devices_has_no_device_overflow() {
        let nodes = [device_node(None)];
        assert_eq!(
            device_overflow_per_node(&[0; 9], &sixty_percent_then_half(), &nodes),
            vec![0.0]
        );
    }

    #[test]
    fn node_fit_check_sees_only_the_pods_on_that_node() {
        let nodes = [device_node(Some(8)), device_node(Some(8))];
        let mut assignment = [0; 9];
        assert!(!devices_fit_on_node(
            0,
            &assignment,
            &sixty_percent_then_half(),
            &nodes
        ));
        assignment[8] = 1;
        assert!(devices_fit_on_node(
            0,
            &assignment,
            &sixty_percent_then_half(),
            &nodes
        ));
    }

    #[test]
    fn ledger_declines_the_half_gpu_after_eight_sixty_percent_pods() {
        let node = device_node(Some(8));
        let pods = sixty_percent_then_half();
        let mut ledger = DeviceLedger::new(1);
        for pod in &pods[..8] {
            assert!(ledger.admits(&node, 0, pod));
            ledger.record(0, pod);
        }
        assert!(!ledger.admits(&node, 0, &pods[8]));
    }

    #[test]
    fn ledger_admits_anything_on_a_node_without_devices() {
        let node = device_node(None);
        let mut ledger = DeviceLedger::new(1);
        let pods = sixty_percent_then_half();
        for pod in &pods[..8] {
            ledger.record(0, pod);
        }
        assert!(ledger.admits(&node, 0, &pods[8]));
    }
}
