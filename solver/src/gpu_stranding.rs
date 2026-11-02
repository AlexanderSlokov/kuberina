//! Milli-GPU stranded on partly used devices (plan #24, §8 decision 7; backlog S-2a).
//!
//! A device with 600 milli-GPU in use keeps 400 free. Only a Pod asking for 400 or less
//! can use that room, so it is worth less than a free device. The fragmentation term
//! charges it, and the in-order walk prefers the node where a Pod strands the least.
//!
//! The device layout comes from the same best-fit decode as the feasibility check
//! (`gpu_devices::decode_devices`), so both agree on which device holds what.

use crate::gpu_devices::{DeviceLedger, MILLI_PER_DEVICE, decode_devices, device_demand};
use crate::model::{GpuShare, Node, Pod, placements};

/// Free milli-GPU on devices that are partly used; a free or full device strands nothing.
///
/// ```
/// # use kuberina_solver::gpu_stranding::stranded_milli;
/// assert_eq!(stranded_milli(&[400, 1000, 0, 250]), 650);
/// ```
pub fn stranded_milli(free_per_device: &[u32]) -> u64 {
    free_per_device
        .iter()
        .filter(|&&free| free > 0 && free < MILLI_PER_DEVICE)
        .map(|&free| u64::from(free))
        .sum()
}

/// Stranded milli-GPU when `demands` are decoded onto `device_count` devices.
fn stranded_on_devices(device_count: u32, demands: &[GpuShare]) -> u64 {
    stranded_milli(&decode_devices(device_count, demands).free)
}

/// Stranded GPUs summed over all nodes, for the fragmentation term; 0 without devices.
///
/// ```ignore
/// let f_frag = compute_fragmentation(&load, &nodes) + stranded_gpu(&assignment, &pods, &nodes);
/// ```
pub fn stranded_gpu(assignment: &[usize], pods: &[Pod], nodes: &[Node]) -> f64 {
    // WHY early return: a cluster without devices (MSC Irina) pays one scan, no decode.
    if nodes.iter().all(|n| n.gpu_devices.is_none()) {
        return 0.0;
    }
    let mut ledger = DeviceLedger::new(nodes.len());
    for (pod_idx, node_idx) in placements(assignment) {
        ledger.record(node_idx, &pods[pod_idx]);
    }
    let milli: u64 = nodes
        .iter()
        .enumerate()
        .filter_map(|(idx, node)| {
            Some(stranded_on_devices(
                node.gpu_devices?,
                ledger.demands_on(idx),
            ))
        })
        .sum();
    milli as f64 / f64::from(MILLI_PER_DEVICE)
}

/// How many milli-GPU placing `pod` on `node` adds to its stranded room; negative when
/// the Pod fills room that was stranded.
///
/// ```ignore
/// let delta = stranding_delta(&ledger, &nodes[j], j, &pod);
/// ```
pub fn stranding_delta(ledger: &DeviceLedger, node: &Node, node_idx: usize, pod: &Pod) -> i64 {
    let (Some(device_count), Some(demand)) = (node.gpu_devices, device_demand(pod)) else {
        return 0;
    };
    let before = ledger.demands_on(node_idx);
    let mut after = before.to_vec();
    after.push(demand);
    stranded_on_devices(device_count, &after) as i64
        - stranded_on_devices(device_count, before) as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_fixtures::{cpu_node, cpu_pod, eight_gpu_node, shared_gpu_pod};

    #[test]
    fn a_partly_used_device_strands_its_free_room() {
        let pods = [shared_gpu_pod("sixty", 600)];
        let nodes = [eight_gpu_node("n")];
        assert_eq!(stranded_gpu(&[0], &pods, &nodes), 0.4);
    }

    #[test]
    fn two_shares_that_fill_one_device_strand_nothing() {
        let pods = [shared_gpu_pod("sixty", 600), shared_gpu_pod("forty", 400)];
        let nodes = [eight_gpu_node("n")];
        assert_eq!(stranded_gpu(&[0, 0], &pods, &nodes), 0.0);
    }

    #[test]
    fn unplaced_pods_and_nodes_without_devices_strand_nothing() {
        let pods = [shared_gpu_pod("sixty", 600), cpu_pod("c", 1.0)];
        let nodes = [cpu_node("cpu", 4.0)];
        assert_eq!(
            stranded_gpu(&[crate::model::UNPLACED, 0], &pods, &nodes),
            0.0
        );
    }

    #[test]
    fn the_fitness_fragmentation_term_includes_stranded_gpus() {
        use crate::fitness::{compute_fitness, compute_fragmentation};
        use crate::model::{Blueprint, FitnessWeights};
        use crate::phase1_ffd::compute_node_loads;
        let pods = [shared_gpu_pod("sixty", 600)];
        let nodes = [eight_gpu_node("n")];
        let node_load = compute_node_loads(&[0], &pods, 1);
        let per_node_waste = compute_fragmentation(&node_load, &nodes);
        let bp = Blueprint {
            assignment: vec![0],
            fitness: 0.0,
            node_load,
            scorecard: Default::default(),
        };
        let (_, scorecard) = compute_fitness(&bp, &pods, &nodes, &[], &FitnessWeights::default());
        assert!((scorecard.fragmentation - (per_node_waste + 0.4)).abs() < 1e-9);
    }

    #[test]
    fn filling_stranded_room_has_a_negative_delta() {
        let pods = [shared_gpu_pod("sixty", 600), shared_gpu_pod("forty", 400)];
        let node = eight_gpu_node("n");
        let mut ledger = DeviceLedger::new(1);
        ledger.record(0, &pods[0]);
        assert_eq!(stranding_delta(&ledger, &node, 0, &pods[1]), -400);
    }

    #[test]
    fn a_share_on_an_empty_node_strands_the_rest_of_its_device() {
        let pod = shared_gpu_pod("sixty", 600);
        let ledger = DeviceLedger::new(1);
        assert_eq!(stranding_delta(&ledger, &eight_gpu_node("n"), 0, &pod), 400);
    }

    #[test]
    fn a_cpu_pod_strands_nothing() {
        let ledger = DeviceLedger::new(1);
        assert_eq!(
            stranding_delta(&ledger, &eight_gpu_node("n"), 0, &cpu_pod("c", 1.0)),
            0
        );
    }
}
