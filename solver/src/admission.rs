//! Admission: leaving Pods unplaced when demand exceeds capacity (IR v1 §4.1, X-4).
//!
//! Two modes, decided by the Founder on 2026-11-01:
//!
//! - **In-order** walks the Pods by arrival. A Pod is admitted when the Pods admitted
//!   before it, plus it, can be placed; Pods admitted earlier may move. A rejected Pod
//!   stays rejected, and the next Pod is tried: FGD's rule, plus re-planning.
//! - **Free** may choose any set of Pods. FFD admits what fits, and the GA may place or
//!   unplace Pods under the unplaced-Pod objective term.
//!
//! Re-planning here is a best-fit insertion followed by a depth-1 repair: move one
//! admitted Pod to another node so that the new Pod fits where it was. IR v1 §4.1 allows
//! a planner to reject a Pod that a deeper re-packing could admit.

use std::collections::HashMap;
use std::str::FromStr;

use crate::model::{Blueprint, FfdWeights, GpuShare, Node, Pod, UNPLACED, placements};
use crate::packing::Packing;
use crate::phase1_ffd::ffd_order;

/// How the planner may leave Pods unplaced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Admission {
    InOrder,
    Free,
}

impl Admission {
    /// The spelling used on the command line and in `metadata.admission`.
    pub fn label(self) -> &'static str {
        match self {
            Admission::InOrder => "in-order",
            Admission::Free => "free",
        }
    }
}

impl FromStr for Admission {
    type Err = String;

    /// ```
    /// # use kuberina_solver::admission::Admission;
    /// assert_eq!("in-order".parse::<Admission>(), Ok(Admission::InOrder));
    /// ```
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "in-order" => Ok(Admission::InOrder),
            "free" => Ok(Admission::Free),
            other => Err(format!(
                "admission {other:?} is not a mode; expected in-order or free"
            )),
        }
    }
}

/// In-order admission over `order` (Pod indices by arrival); returns the placement.
///
/// ```ignore
/// let seed = admit_in_order(&pods, &nodes, &arrival_order(&pods)?);
/// ```
pub fn admit_in_order(pods: &[Pod], nodes: &[Node], order: &[usize]) -> Blueprint {
    let mut packing = Packing::new(pods, nodes);
    for &pod_idx in order {
        admit_one(&mut packing, pod_idx);
    }
    // WHY: a repair move can open room that an earlier rejected Pod could use.
    enforce_on(&mut packing, order);
    packing.into_blueprint()
}

/// Place Pod `pod_idx` best-fit, or by moving one admitted Pod; false if neither works.
fn admit_one(packing: &mut Packing, pod_idx: usize) -> bool {
    if let Some(node_idx) = packing.best_fit_node(pod_idx) {
        packing.place(pod_idx, node_idx);
        return true;
    }
    relocate_one(packing, pod_idx)
}

/// Depth-1 repair: move one admitted Pod off a node so that `incoming` fits there.
fn relocate_one(packing: &mut Packing, incoming: usize) -> bool {
    let mut destinations: HashMap<Shape, Vec<usize>> = HashMap::new();
    for node_idx in 0..packing.nodes().len() {
        for leaving in packing.pods_on(node_idx).to_vec() {
            if !packing.fits_in_place_of(incoming, leaving) {
                continue;
            }
            let pod = &packing.pods()[leaving];
            let targets = destinations
                .entry(shape_of(pod))
                .or_insert_with(|| fitting_nodes(packing, leaving));
            let Some(&target) = targets.iter().find(|&&t| t != node_idx) else {
                continue;
            };
            packing.remove(leaving);
            packing.place(incoming, node_idx);
            packing.place(leaving, target);
            return true;
        }
    }
    false
}

/// Nodes Pod `pod_idx` fits as the packing stands.
///
/// WHY it is exact for the repair: the repair changes only the node the moving Pod
/// leaves, and that node is never its destination.
fn fitting_nodes(packing: &Packing, pod_idx: usize) -> Vec<usize> {
    (0..packing.nodes().len())
        .filter(|&node_idx| packing.fits(pod_idx, node_idx))
        .collect()
}

/// What decides where a Pod fits; Pods of one shape fit the same nodes.
#[derive(Hash, PartialEq, Eq)]
struct Shape {
    requests: [u64; 8],
    gpu_share: Option<(u32, u32)>,
    node_selector: Vec<(String, String)>,
    tolerations: Vec<String>,
}

fn shape_of(pod: &Pod) -> Shape {
    let r = pod.requests;
    let mut node_selector: Vec<(String, String)> = pod.node_selector.clone().into_iter().collect();
    node_selector.sort();
    Shape {
        requests: [
            r.cpu,
            r.ram,
            r.gpu,
            r.storage,
            r.disk_read,
            r.disk_write,
            r.net_in,
            r.net_out,
        ]
        .map(f64::to_bits),
        gpu_share: pod
            .gpu_share
            .map(|GpuShare { devices, milli }| (devices, milli)),
        node_selector,
        tolerations: pod.tolerations.clone(),
    }
}

/// FFD that leaves a Pod unplaced when it fits nowhere, instead of overflowing a node.
///
/// ```ignore
/// let seed = ffd_with_admission(&pods, &nodes, &FfdWeights::default());
/// ```
pub fn ffd_with_admission(pods: &[Pod], nodes: &[Node], weights: &FfdWeights) -> Blueprint {
    let mut packing = Packing::new(pods, nodes);
    for pod_idx in ffd_order(pods, nodes, weights) {
        if let Some(node_idx) = packing.first_fit_node(pod_idx) {
            packing.place(pod_idx, node_idx);
        }
    }
    packing.into_blueprint()
}

/// The first breach of the in-order rule: an unplaced Pod that could take the place of a
/// placed Pod arriving after it. Returns `(unplaced, later)`.
///
/// ```ignore
/// assert_eq!(in_order_violation(&packing, &order), None);
/// ```
pub fn in_order_violation(packing: &Packing, order: &[usize]) -> Option<(usize, usize)> {
    let mut rank = vec![0; order.len()];
    for (position, &pod_idx) in order.iter().enumerate() {
        rank[pod_idx] = position;
    }
    order
        .iter()
        .filter(|&&unplaced| packing.node_of(unplaced) == UNPLACED)
        .find_map(|&unplaced| later_pod_it_could_replace(packing, unplaced, &rank))
}

/// A placed Pod that arrived after `unplaced` and whose place `unplaced` could take.
fn later_pod_it_could_replace(
    packing: &Packing,
    unplaced: usize,
    rank: &[usize],
) -> Option<(usize, usize)> {
    (0..packing.nodes().len())
        .flat_map(|node_idx| packing.pods_on(node_idx).iter().copied())
        .find(|&later| rank[later] > rank[unplaced] && packing.fits_in_place_of(unplaced, later))
        .map(|later| (unplaced, later))
}

/// Repair every breach of the in-order rule, earliest unplaced Pod first.
///
/// Each repair admits an earlier Pod and evicts a later one, then tries to place the
/// evicted Pod elsewhere. Every step admits an earlier arrival, so the loop ends. Run it
/// after the GA, which moves Pods without knowing the rule.
///
/// ```ignore
/// let best = enforce_in_order_rule(&best.assignment, &pods, &nodes, &order);
/// ```
pub fn enforce_in_order_rule(
    assignment: &[usize],
    pods: &[Pod],
    nodes: &[Node],
    order: &[usize],
) -> Blueprint {
    let mut packing = Packing::from_assignment(pods, nodes, assignment);
    enforce_on(&mut packing, order);
    packing.into_blueprint()
}

fn enforce_on(packing: &mut Packing, order: &[usize]) {
    while let Some((earlier, later)) = in_order_violation(packing, order) {
        let node_idx = packing.node_of(later);
        packing.remove(later);
        packing.place(earlier, node_idx);
        if let Some(elsewhere) = packing.best_fit_node(later) {
            packing.place(later, elsewhere);
        }
    }
}

/// GPU placed and GPU in the cluster, in devices: the allocation ratio the FGD
/// simulator reports (plan, §2), on `nodes` as given.
///
/// ```ignore
/// let (placed, capacity) = gpu_allocation(&best.assignment, &pods, &net_nodes);
/// ```
pub fn gpu_allocation(assignment: &[usize], pods: &[Pod], nodes: &[Node]) -> (f64, f64) {
    let placed = placements(assignment)
        .map(|(pod_idx, _)| pods[pod_idx].requests.gpu)
        .sum();
    let capacity = nodes
        .iter()
        .map(|n| n.allocatable.gpu)
        .filter(|&gpu| gpu < f64::MAX)
        .sum();
    (placed, capacity)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ResourceVector;
    use crate::test_fixtures::{cpu_node, cpu_pod, eight_gpu_node, shared_gpu_pod};

    fn whole_gpu_pod(name: &str, gpus: f64) -> Pod {
        Pod {
            requests: ResourceVector::new(1.0, 1.0, gpus),
            gpu_share: None,
            ..shared_gpu_pod(name, 1)
        }
    }

    #[test]
    fn in_order_skips_a_pod_that_does_not_fit_and_admits_a_later_one() {
        // 6 GPUs, then 4 (2 free: rejected), then 2 (fits): skip and continue.
        let pods = [
            whole_gpu_pod("a", 6.0),
            whole_gpu_pod("b", 4.0),
            whole_gpu_pod("c", 2.0),
        ];
        let nodes = [eight_gpu_node("n")];
        let bp = admit_in_order(&pods, &nodes, &[0, 1, 2]);
        assert_eq!(bp.assignment, vec![0, UNPLACED, 0]);
    }

    #[test]
    fn in_order_follows_the_arrival_order_given() {
        // Same Pods, the 4-GPU Pod first: now the 6-GPU Pod is the one left out.
        let pods = [
            whole_gpu_pod("a", 6.0),
            whole_gpu_pod("b", 4.0),
            whole_gpu_pod("c", 2.0),
        ];
        let nodes = [eight_gpu_node("n")];
        let bp = admit_in_order(&pods, &nodes, &[1, 0, 2]);
        assert_eq!(bp.assignment, vec![UNPLACED, 0, 0]);
    }

    #[test]
    fn in_order_moves_one_admitted_pod_to_make_room() {
        // Best fit puts 3 cores on the 4-core node and 2 on the 5-core node. A 4-core Pod
        // then fits nowhere, until the 3-core Pod moves to the 5-core node.
        let pods = [
            cpu_pod("three", 3.0),
            cpu_pod("two", 2.0),
            cpu_pod("four", 4.0),
        ];
        let nodes = [cpu_node("five", 5.0), cpu_node("four", 4.0)];
        let bp = admit_in_order(&pods, &nodes, &[0, 1, 2]);
        assert_eq!(bp.assignment, vec![0, 0, 1]);
    }

    #[test]
    fn free_admission_places_what_fits_and_overflows_nothing() {
        // 13 GPUs offered to 8: about 160%.
        let pods: Vec<Pod> = [6.0, 4.0, 2.0, 1.0]
            .iter()
            .map(|&g| whole_gpu_pod("p", g))
            .collect();
        let nodes = [eight_gpu_node("n")];
        let bp = ffd_with_admission(&pods, &nodes, &FfdWeights::default());
        assert!(bp.assignment.contains(&UNPLACED));
        assert!(bp.node_load[0].gpu <= 8.0);
    }

    #[test]
    fn spec_example_that_holds_has_no_violation() {
        // ir-v1.md §4.1: A 5 GPUs (arrival 0), B 4 unplaced (1), C 1 (2).
        let pods = [
            whole_gpu_pod("a", 5.0),
            whole_gpu_pod("b", 4.0),
            whole_gpu_pod("c", 1.0),
        ];
        let nodes = [eight_gpu_node("n")];
        let packing = Packing::from_assignment(&pods, &nodes, &[0, UNPLACED, 0]);
        assert_eq!(in_order_violation(&packing, &[0, 1, 2]), None);
    }

    #[test]
    fn spec_example_that_breaks_the_rule_is_found_and_repaired() {
        // A 4 GPUs, B 4 unplaced, C 2: B can take the place of C.
        let pods = [
            whole_gpu_pod("a", 4.0),
            whole_gpu_pod("b", 4.0),
            whole_gpu_pod("c", 2.0),
        ];
        let nodes = [eight_gpu_node("n")];
        let packing = Packing::from_assignment(&pods, &nodes, &[0, UNPLACED, 0]);
        assert_eq!(in_order_violation(&packing, &[0, 1, 2]), Some((1, 2)));
        let repaired = enforce_in_order_rule(&[0, UNPLACED, 0], &pods, &nodes, &[0, 1, 2]);
        assert_eq!(repaired.assignment, vec![0, 0, UNPLACED]);
    }

    #[test]
    fn repair_places_the_evicted_pod_elsewhere_when_it_fits() {
        let pods = [
            whole_gpu_pod("a", 4.0),
            whole_gpu_pod("b", 4.0),
            whole_gpu_pod("c", 2.0),
        ];
        let nodes = [
            eight_gpu_node("n"),
            Node {
                allocatable: ResourceVector::new(64.0, 256.0, 2.0),
                gpu_devices: Some(2),
                ..eight_gpu_node("small")
            },
        ];
        let repaired = enforce_in_order_rule(&[0, UNPLACED, 0], &pods, &nodes, &[0, 1, 2]);
        assert_eq!(repaired.assignment, vec![0, 0, 1]);
    }

    #[test]
    fn in_order_result_satisfies_the_rule() {
        let gpus = [3.0, 6.0, 2.0, 5.0, 1.0, 4.0, 2.0, 3.0];
        let pods: Vec<Pod> = gpus.iter().map(|&g| whole_gpu_pod("p", g)).collect();
        let nodes = [eight_gpu_node("a"), eight_gpu_node("b")];
        let order: Vec<usize> = (0..pods.len()).collect();
        let bp = admit_in_order(&pods, &nodes, &order);
        let packing = Packing::from_assignment(&pods, &nodes, &bp.assignment);
        assert_eq!(in_order_violation(&packing, &order), None);
        assert!(bp.assignment.contains(&UNPLACED));
    }

    #[test]
    fn unknown_mode_names_the_value_and_both_modes() {
        let err = "greedy".parse::<Admission>().unwrap_err();
        assert!(
            err.contains("greedy") && err.contains("in-order") && err.contains("free"),
            "{err}"
        );
    }

    #[test]
    fn allocation_counts_placed_gpu_against_constrained_capacity() {
        let pods = [whole_gpu_pod("a", 6.0), whole_gpu_pod("b", 4.0)];
        let nodes = [eight_gpu_node("n"), cpu_node("cpu-only", 4.0)];
        assert_eq!(gpu_allocation(&[0, UNPLACED], &pods, &nodes), (6.0, 8.0));
    }
}
