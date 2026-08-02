# Changelog

All notable changes to the Kuberina project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
