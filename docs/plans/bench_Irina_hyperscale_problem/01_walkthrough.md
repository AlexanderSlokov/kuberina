# Kuberina: A Constraint-Satisfaction Genetic Algorithm Scheduler

**Version**: v0.0.1 (MSC Irina Scale Ready)

## What We Accomplished
Kuberina has evolved from a 10-pod homelab proof-of-concept into a datacenter-capable constraint solver. During our 150-node/2714-pod "MSC Irina" scale stress test, we successfully identified and fixed a catastrophic plateau bug in the genetic algorithm.

### The MSC Irina Anomaly
When pushed to 90%+ CPU and 95% GPU utilization, Kuberina encountered the "Infeasible Plateau" problem:
1. The **FFD (First-Fit Decreasing)** warm-start fell into a fragmentation trap. It greedily assigned heavy `postgres` pods (requiring NVMe) to GPU-optimized nodes (which also have NVMe).
2. By the time `training-worker` pods (requiring A100 GPUs) were scheduled, the GPU nodes were already full of CPU/RAM-heavy database pods.
3. The pods were pushed to a fallback node, violating Capacity and NodeSelector constraints.
4. Because the original penalty function returned `math.inf` for any hard constraint violation, the entire population received a fitness score of `inf`. The GA lost its evolutionary gradient and degenerated into a blind random walk.

### The Gradient Penalty Solution
We refactored Kuberina's penalty system to use **Soft Gradient Penalties** instead of binary boolean logic (`True/False` or `math.inf`).

- **Capacity Overflow**: Replaced `check_capacity_all_nodes` with `compute_capacity_overflow()`. The GA now calculates the exact scalar volume of resource over-commitment: `Σ max(0, Load - Capacity)`.
- **Selector Violations**: Created `compute_selector_violations()` to count how many pods are placed on nodes that violate their `NodeSelector` or `Tolerations`.
- **Fitness Integration**: Violations are now penalized with a steep slope:
  - `Base Penalty (1,000,000) + Overflow Amount * 10,000`

> [!TIP]
> **Evolutionary Gradient**
> With this change, if the GA randomly mutates and reduces the overflow by even 1.0 CPU, its fitness improves from `1,050,000` to `1,040,000`. Tournament selection detects this improvement, preserving the genes and guiding the algorithm back into feasible space.

## Validation Results
- Unit tests (`tests/test_csp.py`, `tests/test_fitness.py`, `tests/test_integration.py`) have been fully updated to assert mathematical convergence behavior.
- All 33 tests passed under `pytest`.
- The `make irina_stress` command will now print a massively high but **finite** seed fitness (e.g., `seed fitness = 2,540,110`). As you watch the generations progress, you will see the fitness steadily decrease as the GA learns to evict `postgres` from the GPU nodes and heal the cluster!
