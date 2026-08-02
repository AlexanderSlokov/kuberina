# Kuberina v0.2.0: MDBP Expansion & IR

The v0.2.0 release focuses on expanding the core scheduling problem space from 3 dimensions to 8 dimensions and standardizing the inputs/outputs with the new Kuberina Intermediate Representation (IR).

## What We Built

### 1. 8-Dimensional Multi-Dimensional Bin Packing (MDBP)
- Upgraded the domain models (`ResourceVector`) to support `CPU`, `RAM`, `GPU`, `Storage`, `Disk Read`, `Disk Write`, `Network In`, and `Network Out`.
- **Why?** To proactively solve the "Noisy Neighbor" problem where IO-heavy workloads (like databases and streaming servers) saturate a node's disk or network bandwidth despite having sufficient CPU/RAM.

### 2. Kuberina IR v0.2.0 Format
- Developed a highly readable YAML specification for Kuberina.
- Implemented `quantity.rs` to seamlessly convert standard Kubernetes resource notations (e.g., `512Mi`, `300m`, `10G`) into f64 representations used by the engine.
- Added intelligent features to the parser:
  - **Replica Unrolling**: Automatically expands `replicas: 3` into `pod-0000`, `pod-0001`, `pod-0002`.
  - **Namespace Flattening**: Groups workloads by namespace for clearer YAML structure, flattening them at runtime.
  - **Gang Auto-grouping**: Automatically links pods with the same `gang` annotation into strict `PodGroup` co-scheduling units.

### 3. Topology Spread Soft Penalty
- Implemented a topology spread logic that calculates the skew of pod replicas across specified topology domains (like `zone` or `rack`).
- By applying this as a soft gradient penalty in the Genetic Algorithm's fitness function, the solver organically learns to distribute workloads evenly for high availability, rather than blindly failing if it can't find a perfect placement.

## Verification
- Wrote extensive Unit and Doc-tests covering the new `quantity` parser, 8D logic, and `topology_spread_penalty` edge cases.
- Validated with the new `testdata/homelab_workloads.yaml` which successfully isolates heavy I/O services (`jellyfin`, `postgres`) and intelligently respects topology rules.
- Addressed all strict `clippy` linter warnings and enforced code formatting with `rustfmt`.
- Added missing `make` targets (`solver-lint`, `solver-fmt`) and streamlined `solver-homelab` to use `--release` for instant results.

## Next Steps
- We are now ready to embark on **Phase 1: Kuberina Forge & Hexagonal Architecture (v0.3.0)**, where we will build the Go CLI to ingest real Kubernetes states and decouple the Rust core engine.
