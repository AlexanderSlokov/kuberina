# Refactor Hard Constraints to Gradient Penalties

The current GA implementation uses `math.inf` to penalize hard constraint violations (Capacity, NodeSelector, Gang constraints). This causes the GA to plateau in a zero-gradient space (`fitness=inf`) during highly constrained "datacenter-scale" packing scenarios like the `MSC Irina` test, where initial FFD warm-starts can fall into fragmentation traps.

We will refactor Kuberina to use a continuous "Soft Penalty" gradient for hard constraints.

## Proposed Changes

### [MODIFY] [csp.py](file:///home/stella/workspace/kuberina/research/src/kuberina/phases/csp.py)
- Replace `check_capacity_all_nodes` with `compute_capacity_overflow(assignment, pods, nodes) -> float`.
  - Instead of returning a boolean, this function will compute the exact scalar amount of resource overflow across all nodes.
  - Formula: `Σ (max(0, load.cpu - cap.cpu) * weight_cpu + ...)`
- Add `compute_selector_violations(assignment, pods, nodes) -> int`
  - Counts the number of pods assigned to nodes that do not satisfy their NodeSelector or Taint tolerations.

### [MODIFY] [fitness.py](file:///home/stella/workspace/kuberina/research/src/kuberina/fitness.py)
- Remove `math.inf` returns in `compute_fitness`.
- Introduce steep scalar penalties:
  - `OVERCAPACITY_WEIGHT = 1,000,000`
  - `SELECTOR_VIOLATION_WEIGHT = 500,000`
- Integrate `compute_capacity_overflow` and `compute_selector_violations` into the main `compute_fitness` formula.
- Refactor `_gang_penalty` to return a scalable penalty `(group.min_members - placed) * 500,000` instead of `math.inf`.

### [MODIFY] [test_csp.py](file:///home/stella/workspace/kuberina/research/tests/test_csp.py) & [test_fitness.py](file:///home/stella/workspace/kuberina/research/tests/test_fitness.py)
- Update unit tests to assert finite scalar penalty values instead of `math.inf`.

## Verification Plan

### Automated Tests
- Run `pytest tests/ -v` to ensure all 32 unit and integration tests pass with the new penalty architecture.

### Manual Verification
- Re-run the `make irina_stress` command.
- We expect the FFD seed to still have a very high fitness penalty (e.g. `fitness = 15,200,400`), but as the GA progresses through generations (Gen 10, Gen 50, etc.), we should see the fitness rapidly decreasing as the GA's mutations push `postgres` out of the GPU nodes and place `training-worker` into the valid nodes, eventually reaching a valid state where the penalty drops below the 1,000,000 threshold.
