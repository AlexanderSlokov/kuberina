//! Free-admission swap: decline one placed Pod, place several unplaced Pods in its room
//! (plan #24, §8 decision 9; backlog S-2c).
//!
//! The other free-admission operators change one gene at a time. On the X-4 fixture,
//! declining one 4-GPU Pod and placing five small Pods on the same node raises the
//! placed Pods from 3 to 7 at the same 16 GPUs, and no single-gene move gets there:
//! each intermediate step leaves capacity unused and scores worse.

use rand::rngs::StdRng;
use rand::seq::IndexedRandom;

use crate::model::{Node, Pod, UNPLACED, placements};
use crate::packing::Packing;

/// Trade one placed GPU Pod, chosen at random, for unplaced Pods on its node.
///
/// ```ignore
/// trade_at_random(&mut blueprint.assignment, &pods, &nodes, &mut rng);
/// ```
pub fn trade_at_random(
    assignment: &mut Vec<usize>,
    pods: &[Pod],
    nodes: &[Node],
    rng: &mut StdRng,
) {
    let mut packing = Packing::from_assignment(pods, nodes, assignment);
    trade_random_gpu_pod(&mut packing, rng);
    *assignment = packing.into_assignment();
}

/// `trade_at_random` on a packing the caller already holds (the GA's mutation, X-8).
///
/// ```ignore
/// trade_random_gpu_pod(&mut packing, &mut rng);
/// ```
pub fn trade_random_gpu_pod(packing: &mut Packing, rng: &mut StdRng) {
    let pods = packing.pods();
    let gpu_pods: Vec<usize> = placements(packing.assignment())
        .map(|(pod_idx, _)| pod_idx)
        .filter(|&pod_idx| pods[pod_idx].requests.gpu > 0.0)
        .collect();
    if let Some(&declined) = gpu_pods.choose(rng) {
        trade_one_for_several(packing, declined);
    }
}

/// Decline Pod `declined` and refill its node with unplaced Pods, largest GPU first.
///
/// The declined Pod is not offered back to its own node. A Pod that does not fit is
/// skipped, and the next one is tried. Returns the Pods placed.
///
/// ```ignore
/// let placed = trade_one_for_several(&mut packing, train_c);
/// ```
pub fn trade_one_for_several(packing: &mut Packing, declined: usize) -> Vec<usize> {
    let node_idx = packing.node_of(declined);
    if node_idx == UNPLACED {
        return Vec::new();
    }
    packing.remove(declined);
    let mut placed = Vec::new();
    for pod_idx in refill_order(packing.pods(), packing.assignment(), declined) {
        if packing.fits(pod_idx, node_idx) {
            packing.place(pod_idx, node_idx);
            placed.push(pod_idx);
        }
    }
    placed
}

/// Unplaced Pods other than `declined`, largest aggregate GPU first, then by index.
fn refill_order(pods: &[Pod], assignment: &[usize], declined: usize) -> Vec<usize> {
    let mut order: Vec<usize> = (0..pods.len())
        .filter(|&p| p != declined && assignment[p] == UNPLACED)
        .collect();
    order.sort_by(|&a, &b| {
        pods[b]
            .requests
            .gpu
            .total_cmp(&pods[a].requests.gpu)
            .then(a.cmp(&b))
    });
    order
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::GpuShare;
    use crate::test_fixtures::{cpu_node, cpu_pod, eight_gpu_node, shared_gpu_pod};

    fn whole_gpu_pod(name: &str, gpus: u32) -> Pod {
        let mut pod = shared_gpu_pod(name, 1000);
        pod.requests.gpu = f64::from(gpus);
        pod.gpu_share = Some(GpuShare {
            devices: gpus,
            milli: 1000,
        });
        pod
    }

    #[test]
    fn declining_a_large_pod_places_the_small_ones_that_fill_its_devices() {
        // The X-4 finding in miniature: a 4-GPU Pod holds the node's last four devices.
        let pods = vec![
            whole_gpu_pod("train-a", 4),
            whole_gpu_pod("train-c", 4),
            whole_gpu_pod("train-d", 2),
            shared_gpu_pod("infer-e", 700),
            shared_gpu_pod("infer-a", 600),
            shared_gpu_pod("infer-f", 400),
            shared_gpu_pod("infer-d", 300),
        ];
        let nodes = [eight_gpu_node("n")];
        let start = [0, 0, UNPLACED, UNPLACED, UNPLACED, UNPLACED, UNPLACED];
        let mut packing = Packing::from_assignment(&pods, &nodes, &start);
        assert_eq!(trade_one_for_several(&mut packing, 1), vec![2, 3, 4, 5, 6]);
        assert_eq!(packing.node_of(1), UNPLACED);
    }

    #[test]
    fn a_pod_that_does_not_fit_is_skipped_and_the_next_one_is_tried() {
        let pods = vec![
            cpu_pod("big", 4.0),
            cpu_pod("huge", 9.0),
            cpu_pod("small", 3.0),
        ];
        let nodes = [cpu_node("n", 4.0)];
        let mut packing = Packing::from_assignment(&pods, &nodes, &[0, UNPLACED, UNPLACED]);
        assert_eq!(trade_one_for_several(&mut packing, 0), vec![2]);
    }

    #[test]
    fn an_unplaced_pod_cannot_be_traded() {
        let pods = vec![cpu_pod("a", 1.0)];
        let nodes = [cpu_node("n", 4.0)];
        let mut packing = Packing::from_assignment(&pods, &nodes, &[UNPLACED]);
        assert!(trade_one_for_several(&mut packing, 0).is_empty());
        assert_eq!(packing.node_of(0), UNPLACED);
    }
}
