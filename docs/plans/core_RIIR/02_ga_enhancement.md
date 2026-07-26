# Phase 2 GA Enhancements for Dense Clusters

The MSC Irina workload has a 95.2% CPU utilization. The current Genetic Algorithm is trapped in a local minimum because the mutation operator only moves one pod at a time. In a nearly full cluster, moving a pod to a new node almost always overflows that node, triggering a massive capacity penalty.

Furthermore, we lack visibility into the exact penalties making up the 10-million fitness score.

## User Review Required
> [!IMPORTANT]
> The Swap operator is O(1) but adds logic to the mutation phase. Is a 50/50 split between `move` and `swap` acceptable, or would you prefer a dynamic rate based on cluster utilization?

## Open Questions
- Do we want to print the Scorecard on every 50 generations, or just at Gen 0 and at the very end? (Proposed: Just Gen 0 and the final output to avoid log spam).

## Proposed Changes

### Visibility (Scorecard)
Add a `Scorecard` struct to `fitness.rs` to track the exact breakdown of constraints.
#### [MODIFY] [fitness.rs](file:///home/stella/workspace/kuberina/solver/src/fitness.rs)
- Create `Scorecard` struct with fields: `capacity_penalty`, `selector_penalty`, `gang_penalty`, `fragmentation_score`, `affinity_score`, `variance_score`.
- Refactor `compute_fitness` to return `(f64, Scorecard)`.
#### [MODIFY] [model.rs](file:///home/stella/workspace/kuberina/solver/src/model.rs)
- Update `Blueprint` to store `Scorecard` alongside `fitness`.
#### [MODIFY] [main.rs](file:///home/stella/workspace/kuberina/solver/src/main.rs)
- Print the Scorecard for the FFD seed and the final best Blueprint.

### Swap Mutation (Escape the Trap)
Introduce a Swap operator in `phase2_ga.rs`.
#### [MODIFY] [phase2_ga.rs](file:///home/stella/workspace/kuberina/solver/src/phase2_ga.rs)
- In `mutate`, for each pod selected for mutation, randomly choose between a `move` operation (current logic) or a `swap` operation.
- `swap` operation: Pick a random second pod (from a different node) and swap their assignments. Check if this breaks gang constraints for either pod; if so, rollback the swap.

## Verification Plan
### Automated Tests
- `make solver-test` to ensure the new Scorecard and Swap logic passes unit tests.
### Manual Verification
- Run `make solver-irina`. We expect to see the Scorecard printed, explicitly showing a 0 penalty for capacity/gangs at the end, and the total fitness dropping to ~1500.
