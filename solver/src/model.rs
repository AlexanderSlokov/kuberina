//! Core data types for Kuberina bin-packing optimizer.
//!
//! Ported from `research/src/kuberina/model/types.py`.
//! See docs/DESIGN.md §4 Data Model and PAPER.md §3.2 Formal Definition.

use serde::Deserialize;
use std::collections::HashMap;
use std::ops::Add;

/// Multi-dimensional resource capacity/request.
///
/// Maps to R = {CPU, RAM, GPU} from PAPER.md §3.2.
/// CPU in cores (f64), RAM in GiB (f64), GPU in units (f64).
///
/// `Copy` — stack-allocated, 3×f64 = 24 bytes. Cache-friendly for hot loops
/// (preplan.md §2: memory contiguity for L1/L2/L3 cache).
///
/// ```
/// # use kuberina_solver::model::ResourceVector;
/// let cap = ResourceVector::new(4.0, 16.0, 1.0);
/// let demand = ResourceVector::new(2.0, 8.0, 1.0);
/// assert!(cap.fits(demand));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub struct ResourceVector {
    #[serde(default)]
    pub cpu: f64,
    #[serde(default)]
    pub ram: f64,
    #[serde(default)]
    pub gpu: f64,
}

impl ResourceVector {
    pub fn new(cpu: f64, ram: f64, gpu: f64) -> Self {
        Self { cpu, ram, gpu }
    }

    pub fn zero() -> Self {
        Self { cpu: 0.0, ram: 0.0, gpu: 0.0 }
    }

    /// True if this vector has enough capacity for the demand.
    pub fn fits(self, demand: Self) -> bool {
        self.cpu >= demand.cpu - 1e-9 && self.ram >= demand.ram - 1e-9 && self.gpu >= demand.gpu - 1e-9
    }

    pub fn subtract(self, other: Self) -> Self {
        Self {
            cpu: self.cpu - other.cpu,
            ram: self.ram - other.ram,
            gpu: self.gpu - other.gpu,
        }
    }

    pub fn is_zero(self) -> bool {
        self.cpu == 0.0 && self.ram == 0.0 && self.gpu == 0.0
    }
}

impl Add for ResourceVector {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            cpu: self.cpu + other.cpu,
            ram: self.ram + other.ram,
            gpu: self.gpu + other.gpu,
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
#[derive(Debug, Clone, Deserialize)]
pub struct Node {
    pub name: String,
    #[serde(default)]
    pub allocatable: ResourceVector,
    #[serde(default)]
    pub labels: HashMap<String, String>,
    #[serde(default)]
    pub taints: Vec<String>,
    #[serde(default)]
    pub zone: String,
}

/// A Kubernetes pod to be scheduled.
///
/// Maps to p_i ∈ P from PAPER.md §3.2.
/// `requests` is req_i^r — the resource demand used for bin packing.
#[derive(Debug, Clone, Deserialize)]
pub struct Pod {
    pub name: String,
    #[serde(default = "default_namespace")]
    pub namespace: String,
    #[serde(default)]
    pub requests: ResourceVector,
    #[serde(default)]
    pub tolerations: Vec<String>,
    #[serde(default, rename = "nodeSelector")]
    pub node_selector: HashMap<String, String>,
    /// WHY list of pod names: affinity is a soft constraint evaluated in fitness,
    /// not a hard constraint. We track desired co-location partners by name.
    #[serde(default, rename = "affinity")]
    pub affinity_targets: Vec<String>,
    #[serde(default, rename = "antiAffinity")]
    pub anti_affinity_targets: Vec<String>,
    #[serde(default, rename = "group")]
    pub group_name: String,
}

fn default_namespace() -> String {
    "default".to_owned()
}

/// Gang-scheduled pod group — all-or-nothing placement.
///
/// Maps to G_q ∈ G from PAPER.md §3.2.
/// Maritime analogy: Block Booking (DESIGN.md §Pod group).
/// Each pod is a separate decision variable (coupled variable in CSP).
#[derive(Debug, Clone, Deserialize)]
pub struct PodGroup {
    pub name: String,
    /// Populated during parsing by resolving pod names → indices.
    #[serde(default)]
    pub pod_indices: Vec<usize>,
    #[serde(default, rename = "minMembers")]
    pub min_members: usize,
    #[serde(default, rename = "nodeSelector")]
    pub node_selector: HashMap<String, String>,
    #[serde(default)]
    pub colocate: bool,
}

/// System-level workload pre-deducted from node capacity.
///
/// Maps to d ∈ D from PAPER.md §3.2.
/// Maritime analogy: Ship's own systems — ballast pumps, comms, sensors.
/// NOT cargo. Pre-deducted in Phase 0 before optimization.
#[derive(Debug, Clone, Deserialize)]
pub struct DaemonSet {
    pub name: String,
    #[serde(default)]
    pub resources: ResourceVector,
    #[serde(default, rename = "nodeSelector")]
    pub node_selector: HashMap<String, String>,
    #[serde(default)]
    pub tolerations: Vec<String>,
}

/// A candidate scheduling solution (GA chromosome).
///
/// Maps to s = [x_1, ..., x_k] from PAPER.md §3.2.
/// `assignment[i]` = node index for pod i.
/// `node_load[j]` = total resource consumed on node j (cached for fast fitness).
#[derive(Debug, Clone)]
pub struct Blueprint {
    pub assignment: Vec<usize>,
    pub fitness: f64,
    pub node_load: Vec<ResourceVector>,
}

/// Hyperparameters for the genetic algorithm.
///
/// Defaults from docs/ga_estimation.md §3 (Small tier).
#[derive(Debug, Clone)]
pub struct GaConfig {
    pub population_size: usize,
    pub tournament_size: usize,
    pub mutation_rate: f64,
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
            mutation_rate: 0.05,
            crossover_rate: 0.8,
            max_generations: 500,
            early_stop_generations: 50,
            random_seed: 42,
        }
    }
}

/// Weights for the multi-objective fitness function.
///
/// F(s) = w1*f_nodes + w2*f_frag + w3*f_affinity + w4*f_var + Φ(s)
/// From PAPER.md §3.2 Objective Function.
#[derive(Debug, Clone)]
pub struct FitnessWeights {
    pub node_count: f64,
    pub fragmentation: f64,
    pub affinity_violation: f64,
    pub utilization_variance: f64,
}

impl Default for FitnessWeights {
    fn default() -> Self {
        Self {
            node_count: 10.0,
            fragmentation: 1.0,
            affinity_violation: 5.0,
            utilization_variance: 2.0,
        }
    }
}

/// Scalarization weights for FFD synthetic volume.
///
/// V_i = α·CPU_i + β·RAM_i + γ·GPU_i
/// From PAPER.md §4.2 / PAPER_v1.md §Giai đoạn 1.
#[derive(Debug, Clone)]
pub struct FfdWeights {
    pub alpha: f64,
    pub beta: f64,
    pub gamma: f64,
}

impl Default for FfdWeights {
    fn default() -> Self {
        Self {
            alpha: 1.0,
            beta: 1.0,
            gamma: 10.0,
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
}
