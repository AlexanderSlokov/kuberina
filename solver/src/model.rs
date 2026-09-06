//! Core data types for Kuberina bin-packing optimizer.
//!
//! Ported from `research/src/kuberina/model/types.py`.
//! See docs/DESIGN.md §4 Data Model and PAPER.md §3.2 Formal Definition.

use serde::Deserialize;
use std::collections::HashMap;
use std::ops::Add;

/// Multi-dimensional resource capacity/request (8D).
///
/// Maps to R = {CPU, RAM, GPU, Storage, DiskR, DiskW, NetIn, NetOut}
/// from PAPER.md §3.2 (v0.2.0 expansion).
///
/// CPU in cores, RAM/Storage in GiB, GPU in units,
/// Disk R/W and Net I/O in MB/s.
///
/// `Copy` — stack-allocated, 8×f64 = 64 bytes. Cache-friendly for hot loops
/// (preplan.md §2: memory contiguity for L1/L2/L3 cache).
///
/// ```
/// # use kuberina_solver::model::ResourceVector;
/// let cap = ResourceVector::new(4.0, 16.0, 1.0);
/// let demand = ResourceVector::new(2.0, 8.0, 1.0);
/// assert!(cap.fits(demand));
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResourceVector {
    pub cpu: f64,
    pub ram: f64,
    pub gpu: f64,
    pub storage: f64,
    pub disk_read: f64,
    pub disk_write: f64,
    pub net_in: f64,
    pub net_out: f64,
}

impl ResourceVector {
    /// Construct with the original 3 dimensions; new dims default to 0.
    /// Keeps backward compat for existing tests.
    pub fn new(cpu: f64, ram: f64, gpu: f64) -> Self {
        Self {
            cpu,
            ram,
            gpu,
            storage: 0.0,
            disk_read: 0.0,
            disk_write: 0.0,
            net_in: 0.0,
            net_out: 0.0,
        }
    }

    /// Full 8-dimensional constructor.
    ///
    /// ```
    /// # use kuberina_solver::model::ResourceVector;
    /// let v = ResourceVector::new_8d(4.0, 16.0, 1.0, 100.0, 500.0, 500.0, 1000.0, 1000.0);
    /// assert_eq!(v.storage, 100.0);
    /// ```
    #[allow(clippy::too_many_arguments)]
    pub fn new_8d(
        cpu: f64,
        ram: f64,
        gpu: f64,
        storage: f64,
        disk_read: f64,
        disk_write: f64,
        net_in: f64,
        net_out: f64,
    ) -> Self {
        Self {
            cpu,
            ram,
            gpu,
            storage,
            disk_read,
            disk_write,
            net_in,
            net_out,
        }
    }

    pub fn zero() -> Self {
        Self {
            cpu: 0.0,
            ram: 0.0,
            gpu: 0.0,
            storage: 0.0,
            disk_read: 0.0,
            disk_write: 0.0,
            net_in: 0.0,
            net_out: 0.0,
        }
    }

    /// True if this vector has enough capacity for the demand on ALL 8 dims.
    pub fn fits(self, demand: Self) -> bool {
        self.cpu >= demand.cpu - 1e-9
            && self.ram >= demand.ram - 1e-9
            && self.gpu >= demand.gpu - 1e-9
            && self.storage >= demand.storage - 1e-9
            && self.disk_read >= demand.disk_read - 1e-9
            && self.disk_write >= demand.disk_write - 1e-9
            && self.net_in >= demand.net_in - 1e-9
            && self.net_out >= demand.net_out - 1e-9
    }

    pub fn subtract(self, other: Self) -> Self {
        Self {
            cpu: self.cpu - other.cpu,
            ram: self.ram - other.ram,
            gpu: self.gpu - other.gpu,
            storage: self.storage - other.storage,
            disk_read: self.disk_read - other.disk_read,
            disk_write: self.disk_write - other.disk_write,
            net_in: self.net_in - other.net_in,
            net_out: self.net_out - other.net_out,
        }
    }

    pub fn is_zero(self) -> bool {
        self.cpu == 0.0
            && self.ram == 0.0
            && self.gpu == 0.0
            && self.storage == 0.0
            && self.disk_read == 0.0
            && self.disk_write == 0.0
            && self.net_in == 0.0
            && self.net_out == 0.0
    }
}

impl Add for ResourceVector {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            cpu: self.cpu + other.cpu,
            ram: self.ram + other.ram,
            gpu: self.gpu + other.gpu,
            storage: self.storage + other.storage,
            disk_read: self.disk_read + other.disk_read,
            disk_write: self.disk_write + other.disk_write,
            net_in: self.net_in + other.net_in,
            net_out: self.net_out + other.net_out,
        }
    }
}

impl Default for ResourceVector {
    fn default() -> Self {
        Self::zero()
    }
}

/// A Kubernetes node with physical resource boundaries.
///
/// Maps to n_j ∈ N from PAPER.md §3.2.
/// `allocatable` is C_j^r AFTER DaemonSet pre-deduction (Phase 0).
#[derive(Debug, Clone)]
pub struct Node {
    pub name: String,
    pub allocatable: ResourceVector,
    pub labels: HashMap<String, String>,
    pub taints: Vec<String>,
    pub zone: String,
    /// Rack topology — used by topologySpread with topologyKey "rack".
    pub rack: String,
}

/// Topology spread constraint — distribute pods evenly across zones/racks.
///
/// From K8s topologySpreadConstraints. Implemented as soft penalty in v0.2.0.
/// `max_skew` = maximum allowed difference in pod count between any two
/// topology domains (zones or racks).
///
/// ```
/// # use kuberina_solver::model::TopologySpread;
/// let ts = TopologySpread { max_skew: 1, topology_key: "zone".into() };
/// assert_eq!(ts.max_skew, 1);
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct TopologySpread {
    #[serde(default = "default_max_skew", rename = "maxSkew")]
    pub max_skew: usize,
    #[serde(default, rename = "topologyKey")]
    pub topology_key: String,
}

fn default_max_skew() -> usize {
    1
}

/// A Kubernetes pod to be scheduled.
///
/// Maps to p_i ∈ P from PAPER.md §3.2.
/// `requests` is req_i^r — the resource demand used for bin packing.
#[derive(Debug, Clone)]
pub struct Pod {
    pub name: String,
    pub namespace: String,
    pub requests: ResourceVector,
    pub tolerations: Vec<String>,
    pub node_selector: HashMap<String, String>,
    /// WHY list of pod names: affinity is a soft constraint evaluated in fitness,
    /// not a hard constraint. We track desired co-location partners by name.
    pub affinity_targets: Vec<String>,
    pub anti_affinity_targets: Vec<String>,
    pub group_name: String,
    /// Topology spread constraint (soft penalty in v0.2.0).
    pub topology_spread: Option<TopologySpread>,
}

/// Gang-scheduled pod group — all-or-nothing placement.
///
/// Maps to G_q ∈ G from PAPER.md §3.2.
/// Maritime analogy: Block Booking (DESIGN.md §Pod group).
/// Each pod is a separate decision variable (coupled variable in CSP).
#[derive(Debug, Clone)]
pub struct PodGroup {
    pub name: String,
    /// Populated during parsing by resolving pod names → indices.
    pub pod_indices: Vec<usize>,
    pub min_members: usize,
    pub node_selector: HashMap<String, String>,
    pub colocate: bool,
}

/// System-level workload pre-deducted from node capacity.
///
/// Maps to d ∈ D from PAPER.md §3.2.
/// Maritime analogy: Ship's own systems — ballast pumps, comms, sensors.
/// NOT cargo. Pre-deducted in Phase 0 before optimization.
#[derive(Debug, Clone)]
pub struct DaemonSet {
    pub name: String,
    pub resources: ResourceVector,
    pub node_selector: HashMap<String, String>,
    pub tolerations: Vec<String>,
}

/// A candidate scheduling solution (GA chromosome).
///
/// Maps to s = [x_1, ..., x_k] from PAPER.md §3.2.
/// `assignment[i]` = node index for pod i.
/// `node_load[j]` = total resource consumed on node j (cached for fast fitness).
#[derive(Debug, Clone, Default)]
pub struct Scorecard {
    pub capacity_penalty: f64,
    pub selector_penalty: f64,
    pub gang_penalty: f64,
    pub active_nodes: f64,
    pub fragmentation: f64,
    pub affinity_violations: f64,
    pub utilization_variance: f64,
    pub topology_spread_penalty: f64,
}

#[derive(Debug, Clone)]
pub struct Blueprint {
    pub assignment: Vec<usize>,
    pub fitness: f64,
    pub node_load: Vec<ResourceVector>,
    pub scorecard: Scorecard,
}

/// Hyperparameters for the genetic algorithm.
///
/// Defaults from docs/ga_estimation.md §3 (Small tier).
#[derive(Debug, Clone)]
pub struct GaConfig {
    pub population_size: usize,
    pub tournament_size: usize,
    /// Expected number of pods relocated per child, NOT a per-gene probability.
    ///
    /// WHY a count: a fixed per-gene rate is a different operator at different
    /// problem sizes. At 10 pods, 0.03 relocates 0.3 assignments — a local move.
    /// At 2,714 pods it relocates 81, which is a random restart: no child lands
    /// near its parent, so elitism preserves the seed and the search never starts.
    /// A count keeps the operator local as |P| grows.
    pub mutations_per_child: f64,
    /// Expected number of pods relocated when seeding each initial individual.
    ///
    /// Large enough that crossover has material to work with, small enough that
    /// the population starts as a neighborhood of the FFD seed rather than noise.
    pub init_mutations: f64,
    pub crossover_rate: f64,
    pub max_generations: usize,
    pub early_stop_generations: usize,
    pub random_seed: u64,
}

impl Default for GaConfig {
    fn default() -> Self {
        Self {
            population_size: 128,
            tournament_size: 3,
            mutations_per_child: 4.0,
            init_mutations: 16.0,
            crossover_rate: 0.8,
            max_generations: 500,
            early_stop_generations: 50,
            random_seed: 42,
        }
    }
}

/// Weights for the multi-objective fitness function (v0.2.0: +topologySpread).
///
/// F(s) = w1*f_nodes + w2*f_frag + w3*f_affinity + w4*f_var + w5*f_spread + Φ(s)
/// From PAPER.md §3.2 Objective Function.
#[derive(Debug, Clone)]
pub struct FitnessWeights {
    pub node_count: f64,
    pub fragmentation: f64,
    pub affinity_violation: f64,
    pub utilization_variance: f64,
    pub topology_spread: f64,
}

impl Default for FitnessWeights {
    fn default() -> Self {
        Self {
            node_count: 10.0,
            fragmentation: 1.0,
            affinity_violation: 5.0,
            utilization_variance: 2.0,
            topology_spread: 3.0,
        }
    }
}

/// Scalarization weights for FFD synthetic volume (v0.2.0: 8D).
///
/// V_i = α·CPU + β·RAM + γ·GPU + δ·Storage + ε_r·DiskR + ε_w·DiskW + ζ_in·NetIn + ζ_out·NetOut
/// From PAPER.md §4.2.
#[derive(Debug, Clone)]
pub struct FfdWeights {
    pub alpha: f64,
    pub beta: f64,
    pub gamma: f64,
    pub delta: f64,
    pub epsilon_r: f64,
    pub epsilon_w: f64,
    pub zeta_in: f64,
    pub zeta_out: f64,
}

impl Default for FfdWeights {
    fn default() -> Self {
        Self {
            // WHY uniform: synthetic_volume divides each dimension by the largest
            // node capacity in that dimension, so the terms are already comparable
            // and a weight expresses preference, not unit conversion. The previous
            // 0.01 defaults on I/O were compensating for raw magnitudes and made
            // FFD blind to whichever I/O dimension actually binds (#18).
            alpha: 1.0,
            beta: 1.0,
            // GPU stays elevated: it is indivisible and confined to a small node
            // subset, so a GPU pod placed late has nowhere left to go.
            gamma: 10.0,
            delta: 1.0,
            epsilon_r: 1.0,
            epsilon_w: 1.0,
            zeta_in: 1.0,
            zeta_out: 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_vector_fits_within_capacity() {
        let cap = ResourceVector::new(4.0, 16.0, 1.0);
        let demand = ResourceVector::new(2.0, 8.0, 1.0);
        assert!(cap.fits(demand));
    }

    #[test]
    fn resource_vector_rejects_overcapacity() {
        let cap = ResourceVector::new(4.0, 16.0, 0.0);
        let demand = ResourceVector::new(2.0, 20.0, 0.0);
        assert!(!cap.fits(demand));
    }

    #[test]
    fn resource_vector_subtract() {
        let a = ResourceVector::new(4.0, 16.0, 2.0);
        let b = ResourceVector::new(1.0, 4.0, 1.0);
        let r = a.subtract(b);
        assert_eq!(r.cpu, 3.0);
        assert_eq!(r.ram, 12.0);
        assert_eq!(r.gpu, 1.0);
    }

    #[test]
    fn resource_vector_add() {
        let a = ResourceVector::new(1.0, 4.0, 0.0);
        let b = ResourceVector::new(0.5, 2.0, 0.0);
        let r = a + b;
        assert_eq!(r.cpu, 1.5);
        assert_eq!(r.ram, 6.0);
    }

    #[test]
    fn resource_vector_zero() {
        let z = ResourceVector::zero();
        assert_eq!(z.cpu, 0.0);
        assert_eq!(z.ram, 0.0);
        assert_eq!(z.gpu, 0.0);
    }

    #[test]
    fn resource_vector_8d_fits() {
        let cap = ResourceVector::new_8d(4.0, 16.0, 1.0, 100.0, 500.0, 500.0, 1000.0, 1000.0);
        let demand = ResourceVector::new_8d(2.0, 8.0, 1.0, 50.0, 200.0, 100.0, 500.0, 500.0);
        assert!(cap.fits(demand));
    }

    #[test]
    fn resource_vector_8d_rejects_disk_overflow() {
        let cap = ResourceVector::new_8d(4.0, 16.0, 1.0, 100.0, 500.0, 500.0, 1000.0, 1000.0);
        let demand = ResourceVector::new_8d(1.0, 1.0, 0.0, 0.0, 600.0, 0.0, 0.0, 0.0);
        assert!(!cap.fits(demand));
    }

    #[test]
    fn resource_vector_8d_add_subtract() {
        let a = ResourceVector::new_8d(4.0, 16.0, 1.0, 100.0, 500.0, 500.0, 1000.0, 1000.0);
        let b = ResourceVector::new_8d(1.0, 4.0, 0.0, 20.0, 100.0, 50.0, 200.0, 300.0);
        let sum = a + b;
        assert_eq!(sum.storage, 120.0);
        assert_eq!(sum.disk_read, 600.0);
        assert_eq!(sum.net_out, 1300.0);

        let diff = a.subtract(b);
        assert_eq!(diff.storage, 80.0);
        assert_eq!(diff.net_in, 800.0);
    }

    #[test]
    fn resource_vector_8d_is_zero() {
        assert!(ResourceVector::zero().is_zero());
        let not_zero = ResourceVector::new_8d(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0);
        assert!(!not_zero.is_zero());
    }
}
