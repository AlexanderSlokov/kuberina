# Changelog

All notable changes to the Kuberina project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **Explicit feasibility verdict and exit codes for the solver.** A plan that does not
  fit is no longer printed under the heading "Final Blueprint" with exit code 0. The
  solver now classifies its result against real capacity first and the reserve second,
  and exits `0` when feasible, `2` when the plan fits the hardware but breaches the
  requested `--headroom` reserve, and `1` when it does not fit at all. A rejected plan
  prints under "Rejected Assignment (NOT a blueprint)" with the overflow broken out per
  dimension, and the exported YAML carries a warning header. CI pipelines can now branch
  on the exit code instead of parsing output. (#17)
- **`--headroom` and path overrides in `bench/mathematical_proof.py`.** The script had
  no command line at all; the three input paths were hardcoded. It now accepts
  `--headroom`, `--infra`, `--workloads` and `--solution`, and reports the LP lower
  bounds under both the real and the reserved capacity model with the approximation
  ratio for each, naming which one matches the run being verified. `make
  bench-proof-headroom-20` runs the reserved configuration. (#8)
- **Session records under `docs/references/sessions/`.** Benchmark runs, before/after
  comparisons and defect reproductions are now written down with the commit they were
  measured at and the command that produced them, before the session ends. `PAPER.md` is
  written from these records rather than from memory. The practice is documented in
  `AGENTS.md`; raw console output lives in `docs/plans/benchmarks/` and is cited by
  filename.
- **`BACKLOG.md`**, holding the concrete tasks that roadmap items decompose into. The
  documentation flow — `ROADMAP.md` for intent, `BACKLOG.md` for tasks, `CHANGELOG.md`
  for what shipped — is recorded in `AGENTS.md`.

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

- **BREAKING: `--pareto 80` is now `--headroom 20`.** The old flag named the cap and the
  new one names the reserve, so the number inverts: `--pareto 80` and `--headroom 20`
  request the same thing. Operators think in terms of how much they are holding back,
  not what fraction they are allowed to fill, and the old name promised a Pareto
  frontier the flag never computed. The value is now validated — a percentage in
  `[0, 100)` — where `--pareto 800` was previously accepted. The Makefile target
  `solver-irina-pareto-80` is now `solver-irina-headroom-20`.
- **FFD pod ordering is now scale-invariant.** `synthetic_volume` scored each dimension
  in its native units — cores against GiB against MB/s — so the weights were doing the
  work of unit conversion rather than expressing priority. Each dimension now contributes
  the share of the largest node's capacity that the pod consumes, and the default weights
  are uniform. Dimensions with unconstrained capacity contribute nothing. (#18)
- **GA mutation is parameterized by expected relocations per chromosome, not per-gene
  probability.** `GaConfig.mutation_rate` is replaced by `mutations_per_child` and
  `init_mutations`, from which the per-gene rate is derived. A per-gene rate that gives a
  local move on a 50-pod chromosome randomizes a 2,714-pod chromosome completely; the
  new knobs mean the same value behaves the same way at any cluster size. (#19)
- **`docs/references/PAPER.md` §6 and §7 rewritten against the current repository.** All
  published figures descended from a 186-node testbed that commit `f7520f0` replaced with
  620 nodes, and none of them reproduced. The sections now report the run at commit
  `1ab1ad7`: 539 of 620 nodes under full packing, 615 of 620 while holding a 20% reserve,
  zero violations across all eight dimensions in both, and approximation ratios of 1.53
  and 1.39 each measured against a bound computed under the capacity model that run
  planned in. §4.2, §4.3, §8.3 and §8.5 were corrected in the same pass, and the abstract
  and conclusion restated. Errata E-1 through E-7 retire with them. The testbed's binding
  dimension is disk write throughput, not CPU, and the paper now says so. (#15, #8, #11)

### Fixed
- **GA early stopping never fired, so every run cost its full generation budget.** The
  convergence check reset its stale counter on any improvement at all, including gains of
  0.0003 on a fitness of 1.58 million, so a run making arbitrarily small progress ran to
  the end no matter what. Both MSC Irina configurations spent all 1,000 generations,
  roughly fifteen minutes each. Progress is now measured against a relative threshold —
  `GaConfig.min_relative_improvement`, defaulting to 0.01% of current best fitness — and
  the counter resets only on cumulative gains that exceed it. Both configurations now
  converge at generation 200 in about three minutes, a 4.9× speedup, and the solver
  prints the generation it converged at so a wall-clock figure means something. The
  reserved-capacity run gives up nothing measurable for the saving; the full-packing run
  finishes on 540 active nodes instead of 539, because the GA's only real gain in 1,000
  generations was a single chance node evacuation at generation ≈581. The threshold value
  is derived from the benchmark rather than tuned, and the derivation, the proof that no
  patience setting below 581 behaves differently on this instance, and the supporting
  literature are recorded in `docs/references/papers/ga-termination-criteria.md`;
  before-and-after measurements are in
  `docs/references/sessions/2026-09-06-ga-early-stop-threshold.md`. (#20)
- `README.md` documented `make solver-inspect`, a target that never existed; the
  validator target is now correctly referenced as `make inspector-run`.
- `README.md` linked to `./DESIGN.md`; the file lives at `docs/DESIGN.md`.
- `docs/DESIGN.md` §1 still described the main engine as Go. The optimization engine
  has been Rust since v0.1.0; the section now states the actual three-language split
  and marks the Go forge as planned rather than present.
- `docs/references/PAPER.md` §6.1 and §6.5 referenced the testdata generator and
  verification scripts at their pre-move `research/` paths.
- **The solver was planning in four of its eight dimensions.** The benchmark generator
  emitted throughput flat (`disk_read`, `net_in`) while Kuberina IR v0.2.0 nests it under
  `disk` and `network`. `RawResources` did not deny unknown fields, so `serde` silently
  discarded all four keys: pod I/O demand parsed as zero and node I/O capacity fell
  through to the unconstrained `f64::MAX` default. Every result published before this was
  produced against CPU, RAM, GPU and storage only. The generator now emits the nested
  shape, the parser rejects unknown fields rather than dropping them, and both Python
  tools normalize either shape on load. Caught by `inspector/`, which shares no code with
  the solver. (#16)
- **The genetic algorithm never searched.** `init_population` perturbed at rate 0.2 and
  `mutate` at 0.03, both per-gene, on a 2,714-gene chromosome — roughly 543 random
  relocations per initial individual and 81 per child. No offspring landed anywhere near
  its parent, elitism preserved the FFD seed unchanged, and every run terminated on the
  early-stopping criterion with its final fitness identical to the seed. The published
  conclusion that "FFD alone found the optimal seed" was an artifact of this. On the
  headroom configuration the fix improves final fitness 66× and removes 99.2% of the
  capacity overflow. (#19)
- **15 pods were being placed on a node that could not hold them, silently.** The FFD
  ordering defect above pushed I/O-heavy pods to the end of the queue, where they reached
  a fallback in `ffd_warmstart` that assigns a pod to a node without capacity and without
  recording that it did so. All 15 landed on the same node. With the ordering fixed the
  fallback is not reached on this benchmark. (#18)
- `solver/testdata/irina_infra.yaml` carried a generated header reading "100 Standard + 30
  Memory + 20 GPU nodes" while the generator emitted 400/120/100. The header is now
  computed from the data it describes.

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
