"""Core data types for Kuberina bin-packing optimizer.

Derived from:
  - docs/DESIGN.md §4 Data Model (Node, Pod, Blueprint, PodGroup, DaemonSet)
  - PAPER.md §3.2 Formal Definition (notation table, decision variables)
  - docs/ga_estimation.md §2 Chromosome Encoding (Blueprint.Assignment []int)
"""

from __future__ import annotations

from dataclasses import dataclass, field


@dataclass
class ResourceVector:
    """Multi-dimensional resource capacity/request.

    Maps to R = {CPU, RAM, GPU} from PAPER.md §3.2 notation table.
    CPU in cores (float), RAM in GiB (float), GPU in units (int-like float).

    Example:
        >>> rv = ResourceVector(cpu=4.0, ram=16.0, gpu=0.0)
        >>> rv.fits(ResourceVector(cpu=2.0, ram=8.0, gpu=0.0))
        True
    """

    cpu: float = 0.0
    ram: float = 0.0
    gpu: float = 0.0

    def fits(self, demand: ResourceVector) -> bool:
        """True if this vector has enough capacity for the demand."""
        return (
            self.cpu >= demand.cpu
            and self.ram >= demand.ram
            and self.gpu >= demand.gpu
        )

    def subtract(self, other: ResourceVector) -> ResourceVector:
        """Return a new vector with other's resources subtracted."""
        return ResourceVector(
            cpu=self.cpu - other.cpu,
            ram=self.ram - other.ram,
            gpu=self.gpu - other.gpu,
        )

    def add(self, other: ResourceVector) -> ResourceVector:
        """Return a new vector with other's resources added."""
        return ResourceVector(
            cpu=self.cpu + other.cpu,
            ram=self.ram + other.ram,
            gpu=self.gpu + other.gpu,
        )

    @staticmethod
    def zero() -> ResourceVector:
        """Return a zero-valued resource vector."""
        return ResourceVector(cpu=0.0, ram=0.0, gpu=0.0)


@dataclass
class Node:
    """A Kubernetes node with physical resource boundaries.

    Maps to n_j ∈ N from PAPER.md §3.2.
    `allocatable` is C_j^r AFTER DaemonSet pre-deduction (Phase 0).

    Example:
        >>> node = Node(name="worker-1", allocatable=ResourceVector(cpu=4.0, ram=16.0))
    """

    name: str
    allocatable: ResourceVector
    labels: dict[str, str] = field(default_factory=dict)
    taints: list[str] = field(default_factory=list)
    zone: str = ""


@dataclass
class Pod:
    """A Kubernetes pod to be scheduled.

    Maps to p_i ∈ P from PAPER.md §3.2.
    `requests` is req_i^r — the resource demand used for bin packing.

    Example:
        >>> pod = Pod(name="nginx", namespace="default",
        ...           requests=ResourceVector(cpu=0.5, ram=0.512))
    """

    name: str
    namespace: str
    requests: ResourceVector
    tolerations: list[str] = field(default_factory=list)
    node_selector: dict[str, str] = field(default_factory=dict)
    # WHY list of pod names: affinity is a soft constraint evaluated in fitness,
    # not a hard constraint. We track desired co-location partners by name.
    affinity_targets: list[str] = field(default_factory=list)
    anti_affinity_targets: list[str] = field(default_factory=list)
    group_name: str = ""


@dataclass
class PodGroup:
    """Gang-scheduled pod group — all-or-nothing placement.

    Maps to G_q ∈ G from PAPER.md §3.2.
    Maritime analogy: Block Booking (DESIGN.md §Pod group).
    Each pod is a separate decision variable (coupled variable in CSP).

    Example:
        >>> group = PodGroup(name="training-job", pod_indices=[0, 1, 2],
        ...                  min_members=3)
    """

    name: str
    pod_indices: list[int]
    min_members: int
    node_selector: dict[str, str] = field(default_factory=dict)
    colocate: bool = False


@dataclass
class DaemonSet:
    """System-level workload pre-deducted from node capacity.

    Maps to d ∈ D from PAPER.md §3.2.
    Maritime analogy: Ship's own systems — ballast pumps, comms, sensors.
    NOT cargo. Pre-deducted in Phase 0 before optimization.

    Example:
        >>> ds = DaemonSet(name="kube-proxy",
        ...                resources=ResourceVector(cpu=0.1, ram=0.064))
    """

    name: str
    resources: ResourceVector
    node_selector: dict[str, str] = field(default_factory=dict)
    tolerations: list[str] = field(default_factory=list)


@dataclass
class Blueprint:
    """A candidate scheduling solution (GA chromosome).

    Maps to s = [x_1, ..., x_k] from PAPER.md §3.2.
    assignment[i] = node index for pod i.
    node_load[j] = total resource consumed on node j (cached for fast fitness).

    Example:
        >>> bp = Blueprint(assignment=[0, 1, 0, 2], fitness=0.0,
        ...                node_load=[ResourceVector()])
    """

    assignment: list[int]
    fitness: float = 0.0
    node_load: list[ResourceVector] = field(default_factory=list)


@dataclass
class GAConfig:
    """Hyperparameters for the genetic algorithm.

    Defaults from docs/ga_estimation.md §3 (Small tier):
    population=128, tournament_size=3, mutation_rate=0.05, early_stop=50.

    Example:
        >>> config = GAConfig(population_size=128)
    """

    population_size: int = 128
    tournament_size: int = 3
    mutation_rate: float = 0.05
    crossover_rate: float = 0.8
    max_generations: int = 500
    early_stop_generations: int = 50
    random_seed: int = 42


@dataclass
class FitnessWeights:
    """Weights for the multi-objective fitness function.

    F(s) = w1*f_nodes + w2*f_frag + w3*f_affinity + w4*f_var + Φ(s)
    From PAPER.md §3.2 Objective Function.

    Defaults are MVP starting points (see research/README.md).

    Example:
        >>> weights = FitnessWeights(node_count=10.0)
    """

    node_count: float = 10.0
    fragmentation: float = 1.0
    affinity_violation: float = 5.0
    utilization_variance: float = 2.0


@dataclass
class FFDWeights:
    """Scalarization weights for FFD synthetic volume.

    V_i = α·CPU_i + β·RAM_i + γ·GPU_i
    From PAPER.md §4.2 / PAPER_v1.md §Giai đoạn 1.

    Example:
        >>> ffd = FFDWeights(alpha=1.0, beta=1.0, gamma=10.0)
    """

    alpha: float = 1.0
    beta: float = 1.0
    gamma: float = 10.0
