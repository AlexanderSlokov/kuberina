# Changelog

All notable changes to the Kuberina project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
- **Component naming disambiguation.** Three distinct artifacts were all named
  `kuberina`, so which one a bare `kuberina plan` invoked depended on `PATH`
  ordering. The Rust binary is now `kuberina-solver` (`solver/Cargo.toml`) and the
  Python reference implementation's console script is now `kuberina-poc`
  (`research/pyproject.toml`). The importable Python module remains `kuberina`, so
  `python -m kuberina` and all existing imports are unaffected. The bare name
  `kuberina` is reserved for `kuberina-forge`, the user-facing CLI planned in v0.3.0.
- **Repository restructured into four named components.** `research/` was carrying
  three unrelated responsibilities under a name that implied disposable scratch work.
  It now holds only the Python reference implementation.
  - `inspector/` — the independent constraint validator and heatmap dashboard,
    promoted to a first-class component. It is cited in PAPER.md §6.5 as the external
    validator behind every published result, and the Kalena interface contract
    (Appendix C.1) depends on it to detect Phase 0 divergence between the two systems.
  - `bench/` — MSC Irina testdata generation and the formal feasibility, quality, and
    significance proofs.
- **Makefile targets realigned to component prefixes:** `solver-*`, `inspector-*`,
  `bench-*`, `research-*`. `research-inspect` is now `inspector-run`, and
  `research-full-pipeline` is now `full-pipeline`. `research-generate-testdata`
  now writes into `research/testdata/` for the reference implementation; the target
  that writes into `solver/testdata/` is now `bench-generate-testdata`.

### Fixed
- `README.md` documented `make solver-inspect`, a target that never existed; the
  validator target is now correctly referenced as `make inspector-run`.
- `README.md` linked to `./DESIGN.md`; the file lives at `docs/DESIGN.md`.
- `docs/DESIGN.md` §1 still described the main engine as Go. The optimization engine
  has been Rust since v0.1.0; the section now states the actual three-language split
  and marks the Go forge as planned rather than present.
- `docs/references/PAPER.md` §6.1 and §6.5 referenced the testdata generator and
  verification scripts at their pre-move `research/` paths.

## [0.2.0] - 2026-08-02

### Added
- **8-Dimensional MDBP Matrix:** Expanded the multi-dimensional bin packing model from 3 dimensions (CPU, RAM, GPU) to 8 dimensions by introducing `storage`, `disk_read`, `disk_write`, `net_in`, and `net_out` fields. This provides granular control over disk and network throughput to mitigate noisy neighbor issues.
- **Kuberina IR v0.2.0 Format:** Introduced a new intermediate representation (IR) structure for Kubernetes topology and workloads. Features include automated namespace flattening, replica unrolling, and gang auto-grouping based on annotations.
- **Topology Spread Constraints:** Added support for `TopologySpread` constraints (`maxSkew` and `topologyKey`) modeled as a soft penalty in the fitness function, promoting even distribution of replicas across zones and racks.
- **Kubernetes Quantity Parser:** Implemented a new context-aware parsing module (`quantity.rs`) to accurately convert standard Kubernetes resource notations (e.g., `512Mi`, `300m`, `10G`) into `f64` representations used by the solver.
- **Rack Topology Domain:** Added `rack` attribute to `Node` models to support hierarchical topology spread calculations.

### Changed
- **Fitness Function Overhaul:** Re-weighted FFD synthetic volume and GA fitness scalarization to accommodate the 8-dimensional space. Default unconstrained node dimensions (e.g., omitted I/O capacities) now gracefully default to `f64::MAX` to prevent artificial fragmentation penalties.
- **DaemonSet Deduction Phase:** Pre-deduction of DaemonSet overhead now properly propagates all 8 dimensions and new topology fields to the effective node capacity.

## [0.1.0] - Initial Release

### Added
- Initial Rust implementation of the genetic algorithm solver engine for Kubernetes pod scheduling.
- **Soft Gradient Penalties:** Introduced a differentiable penalty model for Hard Constraints (Capacity, NodeSelector) replacing previous `math.inf` implementations. This resolved the "Infeasible Plateau Anomaly" during large-scale `MSC Irina` stress tests, allowing the GA to navigate away from invalid states effectively.
- FFD (First Fit Decreasing) Warm-start for generating high-quality initial seeds.
- Basic 3-dimensional ResourceVector models (CPU, RAM, GPU).
