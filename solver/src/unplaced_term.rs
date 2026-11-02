//! The objective term for Pods the planner declines (IR v1 §4.1, backlog X-4).
//!
//! Once Pods may stay unplaced, an empty node is cheaper than a full one, and the
//! optimizer could improve its score by declining Pods (plan §3, gap G3). This term
//! charges every unplaced Pod enough that placing any Pod outweighs freeing any node.
//!
//! The maintainer kept this dominance formula on 2026-11-02 (plan #24, §8 decision 8).
//! The weights derive from the instance at load; backlog S-2b recorded their values on
//! the Alibaba seed-42 instance in `docs/references/sessions/`.

use crate::model::{FitnessWeights, Node, Pod, placements};

/// Per-Pod and per-GPU charges for an unplaced Pod.
///
/// ```
/// # use kuberina_solver::unplaced_term::UnplacedWeights;
/// let off = UnplacedWeights::default();
/// assert_eq!((off.per_pod, off.per_gpu), (0.0, 0.0));
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct UnplacedWeights {
    pub per_pod: f64,
    pub per_gpu: f64,
}

/// Weights from the instance: `per_pod = 2 × (w_node + w_frag × C_max)`, `per_gpu = per_pod / g_min`.
///
/// `C_max` is the largest node's capacity summed over its constrained dimensions, plus
/// its GPU device count, which bounds the milli-GPU it can strand (`gpu_stranding`).
/// It bounds what freeing one node can save in the node-count and fragmentation terms.
/// `g_min` is the smallest positive GPU request. The factor 2 keeps the inequality
/// strict.
///
/// ```ignore
/// let w = derive_unplaced_weights(&pods, &nodes, &FitnessWeights::default());
/// ```
pub fn derive_unplaced_weights(
    pods: &[Pod],
    nodes: &[Node],
    base: &FitnessWeights,
) -> UnplacedWeights {
    let largest_node = nodes.iter().map(constrained_capacity).fold(0.0, f64::max);
    let per_pod = 2.0 * (base.node_count + base.fragmentation * largest_node);
    let smallest_gpu = pods
        .iter()
        .map(|p| p.requests.gpu)
        .filter(|&gpu| gpu > 0.0)
        .fold(f64::INFINITY, f64::min);
    let per_gpu = if smallest_gpu.is_finite() {
        per_pod / smallest_gpu
    } else {
        0.0
    };
    UnplacedWeights { per_pod, per_gpu }
}

/// Capacity summed over the dimensions a node constrains (`f64::MAX` means unconstrained),
/// plus one GPU of stranded room per device.
fn constrained_capacity(node: &Node) -> f64 {
    let a = node.allocatable;
    [
        a.cpu,
        a.ram,
        a.gpu,
        a.storage,
        a.disk_read,
        a.disk_write,
        a.net_in,
        a.net_out,
    ]
    .into_iter()
    .filter(|&dimension| dimension < f64::MAX)
    .sum::<f64>()
        + f64::from(node.gpu_devices.unwrap_or(0))
}

/// The charge for every unplaced Pod in `assignment`; zero when every Pod is placed.
///
/// ```ignore
/// let charge = compute_unplaced_penalty(&blueprint.assignment, &pods, &weights);
/// ```
pub fn compute_unplaced_penalty(
    assignment: &[usize],
    pods: &[Pod],
    weights: &UnplacedWeights,
) -> f64 {
    let mut placed = vec![false; pods.len()];
    for (pod_idx, _) in placements(assignment) {
        placed[pod_idx] = true;
    }
    pods.iter()
        .zip(placed)
        .filter(|&(_, is_placed)| !is_placed)
        .map(|(pod, _)| weights.per_pod + weights.per_gpu * pod.requests.gpu)
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fitness::compute_fitness;
    use crate::model::{Blueprint, ResourceVector, UNPLACED};
    use crate::phase1_ffd::compute_node_loads;
    use crate::test_fixtures::{eight_gpu_node, shared_gpu_pod};

    fn fitness_of(
        assignment: Vec<usize>,
        pods: &[Pod],
        nodes: &[Node],
        weights: &FitnessWeights,
    ) -> f64 {
        let node_load = compute_node_loads(&assignment, pods, nodes.len());
        let bp = Blueprint {
            assignment,
            fitness: 0.0,
            node_load,
            scorecard: Default::default(),
        };
        compute_fitness(&bp, pods, nodes, &[], weights).0
    }

    #[test]
    fn placing_the_smallest_gpu_pod_beats_freeing_an_active_node() {
        // The 50-milli Pod alone on node b: declining it frees b entirely.
        let pods = [shared_gpu_pod("big", 1000), shared_gpu_pod("smallest", 50)];
        let nodes = [eight_gpu_node("a"), eight_gpu_node("b")];
        let base = FitnessWeights::default();
        let weights = FitnessWeights {
            unplaced: derive_unplaced_weights(&pods, &nodes, &base),
            ..base
        };
        let placed = fitness_of(vec![0, 1], &pods, &nodes, &weights);
        let declined = fitness_of(vec![0, UNPLACED], &pods, &nodes, &weights);
        assert!(
            placed < declined,
            "placed {placed} must beat declined {declined}"
        );
    }

    #[test]
    fn weights_follow_the_formula() {
        let pods = [shared_gpu_pod("a", 500), shared_gpu_pod("b", 50)];
        let nodes = [eight_gpu_node("n")];
        let base = FitnessWeights::default();
        let w = derive_unplaced_weights(&pods, &nodes, &base);
        // C_max = 64 cores + 256 GiB + 8 GPUs + 8 devices' stranded room; unconstrained
        // dimensions are skipped.
        let per_pod = 2.0 * (base.node_count + base.fragmentation * (64.0 + 256.0 + 8.0 + 8.0));
        assert_eq!(
            w,
            UnplacedWeights {
                per_pod,
                per_gpu: per_pod / 0.05
            }
        );
    }

    #[test]
    fn cluster_without_gpu_requests_charges_nothing_per_gpu() {
        let mut pod = shared_gpu_pod("cpu-only", 1);
        pod.requests = ResourceVector::new(1.0, 1.0, 0.0);
        let w = derive_unplaced_weights(&[pod], &[eight_gpu_node("n")], &FitnessWeights::default());
        assert_eq!(w.per_gpu, 0.0);
    }

    #[test]
    fn penalty_charges_each_unplaced_pod_its_share_and_nothing_when_all_are_placed() {
        let pods = [shared_gpu_pod("a", 500), shared_gpu_pod("b", 250)];
        let w = UnplacedWeights {
            per_pod: 10.0,
            per_gpu: 100.0,
        };
        assert_eq!(
            compute_unplaced_penalty(&[0, UNPLACED], &pods, &w),
            10.0 + 100.0 * 0.25
        );
        assert_eq!(compute_unplaced_penalty(&[0, 0], &pods, &w), 0.0);
    }
}
