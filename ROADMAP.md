# Kuberina Roadmap

This document outlines the development path for Kuberina. It is structured around major architectural milestones.

## Phase 1: Kuberina Forge & Hexagonal Architecture (v0.3.0)

**Goal:** Evolve Kuberina into a complete compiler pipeline (analogous to `rustc` emitting LLVM IR) and isolate the core solver domain.

### 1. `kuberina-forge` (Go CLI) 

The frontend "Forge" component. Scope is fixed by [ADR-0001](docs/explanation/adr/0001-go-component-is-a-cli.md). The in-cluster operator belongs to the Kuberina Integration Platform (KIP), a separate closed-source repository.

[x] *Forge-in (Frontend):* Ingest Kubernetes manifests from files, directories or stdin and emit standardized Kuberina IR. Helm and Kustomize are served by the pipe — `helm template | kuberina-forge in -`

[x] *Forge-out (Backend/Linker):* Consume the solver's blueprint and render the original manifests back out, pinned to their planned nodes with a required `nodeAffinity`.

[x] Rename `--pareto p` to `--headroom h`.

[x] *Explicit pod groups in the IR:* a `groups:` block makes `min_members` and `colocate` reachable from a workload file, so partial gang admission and forced co-location are expressible.


[ ] *Hexagonal Architecture Refactor:* Restructure the `kuberina-solver` (Rust) codebase utilizing Ports and Adapters. This will fully decouple the Domain Core (FFD, GA, CSP) from edge I/O concerns (Parsers, File Writers).

[ ] *CLI and IR Surface Stabilization.* Breaking changes to the interfaces operators write against belong at a release boundary, and v0.3.0 is the last one before `kuberina-forge` fixes them in place. The surface is now written down normatively in [`docs/references/ir-v1.md`](docs/references/ir-v1.md) and governed by [ADR-0002](docs/explanation/adr/0002-kuberina-ir-v1-stability-contract.md).

[ ] *Native Helm and Kustomize rendering:* an optional convenience over the pipe, deferred until operators ask for it. Cloud API state (AWS, GCP) ingestion likewise.

  - *Optional `observed` usage block:* an additive per-pod record of steady-state consumption (percentiles, peak, and the fraction of the window spent above request), in the existing 8-dimensional vector shape. Absent, behavior is unchanged. **Schema shipped**; packing against it (`--pack-against p95`) remains a separate decision, to be made once real data exists.

- **Search Quality and Termination.** The optimizer's own search behavior, as distinct from the interfaces around it.
  - *Improvement threshold on early stopping:* the convergence test accepted any improvement, however small, as progress, so a run that was still gaining ~0.0003 in fitness never terminated early and spent its full generation budget for a fraction of a percent. Termination now tests for improvement that matters, against a relative threshold. **Shipped.**
  - *Multi-restart (bet-and-run) driver:* the analysis behind the threshold above found that what progress the GA makes past the first plateau arrives as a rare, memoryless event rather than as convergence, which is the regime where restarts dominate a single long run. With the same total budget, several independent seeds of a few hundred generations explore several basins instead of waiting in one. See `docs/references/papers/ga-termination-criteria.md` §4 and §7.
  - *Fitness evaluation cost:* resolve affinity targets and topology spread groups to indices once at load, and evaluate only what a mutation changes. With affinity in force, fitness takes 12.3 ms per evaluation on MSC Irina, 73% of it in the affinity term. See `docs/references/sessions/2026-11-01-irina-with-affinity.md`.
  - *Objective weights from term ranges:* fragmentation contributes 99.35% of MSC Irina's fitness and affinity 0.30%, so the GA removes 5% of affinity violations. Derive each weight from the measured range of its term and record the derivation.
  - *An operator that can empty a node:* node count is the highest-weighted term in the objective, but reducing it requires relocating every pod off a node at once, and single-pod mutation reaches that state with vanishing probability. Both benchmark configurations hold their active-node count constant from the first generation to the last. A coordinated node-evacuation operator is what would make the GA contribute to the objective it is weighted toward.

## Phase 2: Distributed Solving & Agnostic Targets (v0.4.0)

**Goal:** Offload heavy optimization workloads from expensive operational environments and generalize the solver for non-Kubernetes targets.

- **gRPC & Protobuf Integration:** Transition the communication layer between `kuberina-forge` (Agent) and `kuberina-solver` (Engine) from static YAML exchanges to streaming gRPC over Protobuf.
- **Platform Agnosticism:** Given that the solver strictly operates on Kuberina IR, extend optimization capabilities beyond Kubernetes to orchestrators like Proxmox, HashiCorp Nomad, HPC environments (Slurm), or IoT edge provisioning.
- **Cloud-to-Edge Distributed Computation:** 
  - Deploy lightweight `kuberina-forge` agents in expensive cloud environments to collect state and telemetry.
  - Transmit IR via gRPC to localized/on-premise `kuberina-solver` instances where heavy Genetic Algorithm computations run at zero cloud cost.
  - Stream the final optimal `blueprint` back to the cloud for execution, maximizing operational security and minimizing compute expenditure.

[ ] **External Validation on Production Traces** ([#24](https://github.com/AlexanderSlokov/kuberina/issues/24)). Run Kuberina on Alibaba's production-scale cluster data, and report what its results mean next to the Alibaba team's published results.
  - Plan: `docs/plans/benchmarks/alibaba-gpu-v2023/00_plan.md`.
  - *Alibaba `cluster-trace-gpu-v2023`:* compare with FGD and its baselines in the FGD authors' simulator, after reproducing FGD's published result. The simulator reproduces FGD's seed-42 result, and the trace adapter writes the instance as IR. **Shipped.** The scale check is done, and the comparison runs (E1–E5) are next.
  - *Per-device GPU model:* a fractional GPU request must fit on a single device. **Shipped.**
  - *Admission:* the solver can leave pods unplaced when demand exceeds capacity, in arrival order or freely. **Shipped.**
  - *Objective weights for the trace:* derive every weight, the unplaced-Pod charge included, from the Alibaba instance, count milli-GPU stranded on devices, and fix them before Kuberina's first run on it. **Shipped.**
  - *GA moves that respect capacity:* place, move, and swap choose targets where the Pod fits, before the GA is judged on the trace.
  - *The GA's verdict:* on seed 42, compare the in-order walk with a GA-based re-plan. If the GA finds no later first rejection, deprecate, sunset, and remove it ([ADR-0003](docs/explanation/adr/0003-ga-must-beat-the-walk.md)).
  - *Later traces:* `cluster-trace-gpu-v2026`, then `cluster-trace-v2018`.

## Phase 3: Kalena Interoperability (v0.5.0)

**Goal:** Kuberina plans where work goes, and Kalena reclaims unused capacity at runtime. Each ships independently and exchanges data with the other through a documented contract.

- **Shared Phase 0 capacity contract:** both systems compute allocatable capacity after DaemonSet overhead with the same rules, and `inspector/` checks both.
- **Runtime telemetry as `observed`:** Kuberina reads Kalena's usage statistics from a file.
- **Drift-triggered re-planning:** operators set a threshold on request-versus-observed divergence that triggers a new plan (PAPER §8.1).
- **Divergence in the dashboard:** the inspector shows per-node divergence alongside the utilization heatmap.

**Non-goal:** merging the two products. Each keeps working on its own.
