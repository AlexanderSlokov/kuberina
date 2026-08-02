# Update Research Scripts to Kuberina v0.2.0 (8D MDBP)

This plan outlines the steps required to update the Python research scripts (`gen_irina_testdata.py`, `inspector.py`, and `mathematical_proof.py`) to align with the new 8-Dimensional Multi-Dimensional Bin Packing (MDBP) model and Kuberina IR v0.2.0.

## User Review Required

> [!IMPORTANT]  
> The Python verification scripts currently expect purely float values for `cpu`, `ram`, and `gpu`. With the new Kuberina IR, resources can theoretically be Kubernetes unit strings (e.g., `512Mi`, `1G`). To keep the Python scripts simple and focused on mathematical verification, I propose that `gen_irina_testdata.py` continues to emit raw `float` values for the 8 dimensions. The Rust core's `quantity.rs` unit parsing is already thoroughly tested in unit tests. Do you agree with this approach?

## Open Questions

> [!WARNING]
> Do we need to mathematically verify the `topologySpread` penalty in `mathematical_proof.py`? Currently, the mathematical proof only verifies Hard Constraints (Capacity, Node Selector, Assignment). Since `topologySpread` is a Soft Penalty (part of the objective function, not feasibility bounds), I propose we skip adding it to the strict mathematical proof of *feasibility*, but we can include it in the `inspector.py` heatmap for visual validation.

## Proposed Changes

### `research/gen_irina_testdata.py`
We need to expand the generated test data to populate the 8-dimensional space.
- **Node Topology:** Add `rack` to the `_make_node` topology. Add `storage`, `disk_read`, `disk_write`, `net_in`, `net_out` to `allocatable`.
  - Standard Nodes: high storage, moderate I/O.
  - Memory-Optimized: extreme `disk_read` and `disk_write` for NVMe.
  - GPU-Optimized: extremely high `net_in` and `net_out` for distributed training.
- **DaemonSets:** Update DaemonSets to include small 8D overheads.
- **Workloads:** 
  - Add `storage`, `disk_read`, `disk_write`, `net_in`, `net_out` to service definitions.
  - E.g., `postgres-primary` gets massive `disk_write`; `llm-inference` gets massive `net_out`.
- **Kuberina IR v0.2.0:** Apply the new `topologySpread` structures to workloads that need high availability.

### `research/inspector.py`
Update the heatmap generator and validator to recognize 8 dimensions.
- **Pre-deduction Phase 0:** Update `pre_deduct_daemonsets` to subtract all 8 resource dimensions.
- **Validation Check:** Update `validate_solution` to check overflow across all 8 dimensions instead of just 3.
- **Output:** Add summary metrics for Disk and Network I/O to the terminal output.

### `research/mathematical_proof.py`
Update the formal mathematical proof of feasibility.
- **Capacity Constraint (Proof 1):** Expand `verify_capacity_constraint` to loop over all 8 resources (`cpu`, `ram`, `gpu`, `storage`, `disk_read`, `disk_write`, `net_in`, `net_out`).
- **Phase 0 Deductions:** Update `pre_deduct_daemonsets` to properly process the full 8D vector.

## Verification Plan

### Automated Tests
- We will run the python scripts sequentially:
  1. `uv run --with pyyaml python research/gen_irina_testdata.py`
  2. `cd solver && cargo run --release -- plan --infra ../research/testdata/irina_infra.yaml --workloads ../research/testdata/irina_workloads.yaml`
  3. `uv run --with pyyaml python research/inspector.py --infra research/testdata/irina_infra.yaml --workloads research/testdata/irina_workloads.yaml --solution solver/kuberina_solution.yaml`
  4. `uv run --with pyyaml python research/mathematical_proof.py`

### Manual Verification
- Review the terminal output of `inspector.py` to ensure it successfully identifies 8D overflows (which should be zero).
