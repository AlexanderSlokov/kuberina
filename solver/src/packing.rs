//! A mutable placement for the admission engines (backlog X-4).
//!
//! In-order admission places Pods one at a time, moves Pods it placed earlier, and
//! takes Pods out again. `Packing` keeps the bookkeeping those moves need in step: the
//! assignment, each node's residual capacity, its GPU device demands, and the Pods on
//! it. A Pod fits a node when the selector and taints allow it, the residual holds its
//! requests, and its devices fit (IR v1 §3.4).

use crate::csp::can_place_pod_on_node;
use crate::gpu_devices::DeviceLedger;
use crate::model::{Blueprint, Node, Pod, ResourceVector, UNPLACED, placements};
use crate::phase1_ffd::{compute_node_loads, max_node_capacity};

pub struct Packing<'a> {
    pods: &'a [Pod],
    nodes: &'a [Node],
    assignment: Vec<usize>,
    residual: Vec<ResourceVector>,
    devices: DeviceLedger,
    on_node: Vec<Vec<usize>>,
    scale: ResourceVector,
}

impl<'a> Packing<'a> {
    /// A packing with every Pod unplaced.
    ///
    /// ```ignore
    /// let mut packing = Packing::new(&pods, &nodes);
    /// ```
    pub fn new(pods: &'a [Pod], nodes: &'a [Node]) -> Self {
        Self {
            pods,
            nodes,
            assignment: vec![UNPLACED; pods.len()],
            residual: nodes.iter().map(|n| n.allocatable).collect(),
            devices: DeviceLedger::new(nodes.len()),
            on_node: vec![Vec::new(); nodes.len()],
            scale: max_node_capacity(nodes),
        }
    }

    /// A packing that starts from an existing assignment, `UNPLACED` entries included.
    pub fn from_assignment(pods: &'a [Pod], nodes: &'a [Node], assignment: &[usize]) -> Self {
        let mut packing = Self::new(pods, nodes);
        for (pod_idx, node_idx) in placements(assignment) {
            packing.place(pod_idx, node_idx);
        }
        packing
    }

    /// Whether Pod `pod_idx` fits node `node_idx` as the packing stands.
    pub fn fits(&self, pod_idx: usize, node_idx: usize) -> bool {
        let (pod, node) = (&self.pods[pod_idx], &self.nodes[node_idx]);
        can_place_pod_on_node(pod, node)
            && self.residual[node_idx].fits(pod.requests)
            && self.devices.admits(node, node_idx, pod)
    }

    /// Whether Pod `incoming` fits node `node_idx` once Pod `leaving`, placed there, leaves.
    pub fn fits_in_place_of(&self, incoming: usize, leaving: usize) -> bool {
        let node_idx = self.assignment[leaving];
        let (pod_in, pod_out) = (&self.pods[incoming], &self.pods[leaving]);
        let node = &self.nodes[node_idx];
        let room = self.residual[node_idx] + pod_out.requests;
        can_place_pod_on_node(pod_in, node)
            && room.fits(pod_in.requests)
            && self
                .devices
                .admits_in_place_of(node, node_idx, pod_in, pod_out)
    }

    /// Put Pod `pod_idx`, currently unplaced, on node `node_idx`.
    pub fn place(&mut self, pod_idx: usize, node_idx: usize) {
        let pod = &self.pods[pod_idx];
        self.assignment[pod_idx] = node_idx;
        self.residual[node_idx] = self.residual[node_idx].subtract(pod.requests);
        self.devices.record(node_idx, pod);
        self.on_node[node_idx].push(pod_idx);
    }

    /// Take Pod `pod_idx` off its node; it becomes unplaced.
    pub fn remove(&mut self, pod_idx: usize) {
        let node_idx = self.assignment[pod_idx];
        let pod = &self.pods[pod_idx];
        self.assignment[pod_idx] = UNPLACED;
        self.residual[node_idx] = self.residual[node_idx] + pod.requests;
        self.devices.remove(node_idx, pod);
        self.on_node[node_idx].retain(|&p| p != pod_idx);
    }

    /// The fitting node that leaves the least normalized capacity behind, if any.
    pub fn best_fit_node(&self, pod_idx: usize) -> Option<usize> {
        let demand = self.pods[pod_idx].requests;
        (0..self.nodes.len())
            .filter(|&node_idx| self.fits(pod_idx, node_idx))
            .map(|node_idx| {
                (
                    leftover_share(self.residual[node_idx], demand, self.scale),
                    node_idx,
                )
            })
            .min_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)))
            .map(|(_, node_idx)| node_idx)
    }

    /// The fitting node with the lowest index, if any (FFD's rule).
    pub fn first_fit_node(&self, pod_idx: usize) -> Option<usize> {
        (0..self.nodes.len()).find(|&node_idx| self.fits(pod_idx, node_idx))
    }

    pub fn node_of(&self, pod_idx: usize) -> usize {
        self.assignment[pod_idx]
    }

    pub fn pods_on(&self, node_idx: usize) -> &[usize] {
        &self.on_node[node_idx]
    }

    pub fn assignment(&self) -> &[usize] {
        &self.assignment
    }

    pub fn pods(&self) -> &'a [Pod] {
        self.pods
    }

    pub fn nodes(&self) -> &'a [Node] {
        self.nodes
    }

    /// The packing as a blueprint, with node loads computed and fitness unset.
    pub fn into_blueprint(self) -> Blueprint {
        let node_load = compute_node_loads(&self.assignment, self.pods, self.nodes.len());
        Blueprint {
            assignment: self.assignment,
            fitness: 0.0,
            node_load,
            scorecard: Default::default(),
        }
    }
}

/// Capacity left after placing `demand`, as shares of the largest node per dimension.
///
/// Unconstrained dimensions (`f64::MAX`) and dimensions no node declares are skipped,
/// as in `phase1_ffd::synthetic_volume`.
fn leftover_share(residual: ResourceVector, demand: ResourceVector, scale: ResourceVector) -> f64 {
    let dims = [
        (residual.cpu, demand.cpu, scale.cpu),
        (residual.ram, demand.ram, scale.ram),
        (residual.gpu, demand.gpu, scale.gpu),
        (residual.storage, demand.storage, scale.storage),
        (residual.disk_read, demand.disk_read, scale.disk_read),
        (residual.disk_write, demand.disk_write, scale.disk_write),
        (residual.net_in, demand.net_in, scale.net_in),
        (residual.net_out, demand.net_out, scale.net_out),
    ];
    dims.into_iter()
        .filter(|&(left, _, max)| left < f64::MAX && max > 0.0 && max < f64::MAX)
        .map(|(left, need, max)| (left - need) / max)
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_fixtures::{cpu_node, cpu_pod, eight_gpu_node, sixty_percent_then_half};

    #[test]
    fn new_packing_places_nothing() {
        let pods = [cpu_pod("a", 1.0)];
        let nodes = [cpu_node("n", 4.0)];
        assert_eq!(Packing::new(&pods, &nodes).assignment(), &[UNPLACED]);
    }

    #[test]
    fn placing_and_removing_restores_room() {
        let pods = [cpu_pod("a", 3.0), cpu_pod("b", 3.0)];
        let nodes = [cpu_node("n", 4.0)];
        let mut packing = Packing::new(&pods, &nodes);
        packing.place(0, 0);
        assert!(!packing.fits(1, 0));
        packing.remove(0);
        assert!(packing.fits(1, 0));
        assert!(packing.pods_on(0).is_empty());
    }

    #[test]
    fn fit_checks_devices_as_well_as_the_sum() {
        let pods = sixty_percent_then_half();
        let nodes = [eight_gpu_node("a")];
        let mut packing = Packing::new(&pods, &nodes);
        for pod_idx in 0..8 {
            packing.place(pod_idx, 0);
        }
        assert!(!packing.fits(8, 0));
        assert!(packing.fits_in_place_of(8, 0));
    }

    #[test]
    fn fit_respects_the_node_selector() {
        let mut pod = cpu_pod("a", 1.0);
        pod.node_selector.insert("disk".into(), "ssd".into());
        let pods = [pod];
        let nodes = [cpu_node("n", 4.0)];
        assert!(!Packing::new(&pods, &nodes).fits(0, 0));
    }

    #[test]
    fn best_fit_picks_the_tightest_node_and_first_fit_the_lowest_index() {
        let pods = [cpu_pod("a", 3.0)];
        let nodes = [cpu_node("roomy", 8.0), cpu_node("tight", 4.0)];
        let packing = Packing::new(&pods, &nodes);
        assert_eq!(packing.best_fit_node(0), Some(1));
        assert_eq!(packing.first_fit_node(0), Some(0));
    }

    #[test]
    fn from_assignment_rebuilds_the_bookkeeping() {
        let pods = [cpu_pod("a", 3.0), cpu_pod("b", 3.0)];
        let nodes = [cpu_node("n", 4.0)];
        let packing = Packing::from_assignment(&pods, &nodes, &[0, UNPLACED]);
        assert_eq!(packing.pods_on(0), &[0]);
        assert!(!packing.fits(1, 0));
    }

    #[test]
    fn blueprint_carries_the_assignment_and_loads() {
        let pods = [cpu_pod("a", 3.0), cpu_pod("b", 3.0)];
        let nodes = [cpu_node("n", 4.0)];
        let packing = Packing::from_assignment(&pods, &nodes, &[0, UNPLACED]);
        let bp = packing.into_blueprint();
        assert_eq!(
            (bp.assignment, bp.node_load[0].cpu),
            (vec![0, UNPLACED], 3.0)
        );
    }
}
