# Changelog

All notable changes to the Kuberina project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **One solve of the Alibaba seed-42 instance has a measured cost, and the solver prints
  how long each phase takes** ([#24](https://github.com/AlexanderSlokov/kuberina/issues/24),
  X-5). On 1,213 nodes and 10,866 Pods at `e5f520f`, an in-order walk without the GA took
  14.62 seconds and 40 MB, a solve with the GA 196.42 seconds and 450 MB, and a
  free-admission solve 1,313.81 seconds and 440 MB, and the inspector found no violation in
  any of them. The GA changed neither the admitted set nor the GPU allocation of 94.34%.
  Free admission ended at 93.05%, behind the walk, so read its results with that gap in
  mind. The record is `docs/references/sessions/2026-11-03-alibaba-scale-check.md`.
- **The objective weights for the Alibaba trace are fixed, and fragmentation counts
  stranded GPU room** ([#24](https://github.com/AlexanderSlokov/kuberina/issues/24),
  S-2). Free room on a partly used GPU device, such as 400 milli-GPU beside a 600
  milli-GPU Pod, serves only small Pods. The fragmentation term now charges it, and
  in-order admission places each Pod on the node where it strands the least. The
  unplaced-Pod charge keeps its formula, and on the seed-42 instance it is 2,328 per
  Pod and 46,560 per GPU. Free admission gains a move that declines one GPU Pod and
  places several smaller Pods on its node; on the 134% fixture it raised the placed
  Pods from 3 to 9, at 96.88% of the GPUs instead of 100%, because the objective values
  one Pod at the smallest GPU request. The new `--no-ga` flag stops the solver after its
  first placement, which experiment E1 uses ([ADR-0003](docs/explanation/adr/0003-ga-must-beat-the-walk.md)).
  Clusters without GPU devices plan as before: the MSC Irina seed fitness is unchanged
  to the digit. The record is `docs/references/sessions/2026-11-03-objective-weights.md`.
- **The solver can leave Pods unplaced when demand exceeds capacity**
  ([#24](https://github.com/AlexanderSlokov/kuberina/issues/24), X-4). With
  `--admission in-order`, the solver decides Pod by Pod in the order of the new
  `arrival` field: a Pod is admitted when it fits beside the Pods admitted before it,
  after moving at most one of them, and a rejected Pod stays rejected while later Pods
  are still tried. With `--admission free`, the solver may choose any set of Pods. Both
  write a blueprint that is `feasible: true` when no capacity is breached, with an
  `unplaced:` list and `metadata.admission`, and print the GPU allocation. On a fixture
  that asks for 134% of its GPUs, in-order admission placed 96.88% of the GPUs and free
  admission 100%, and the inspector, which checks the in-order rule with its own code,
  agreed. `kuberina-forge out` refuses a blueprint with unplaced Pods, because a
  partial plan serves benchmarks and is not applied to a cluster. Runs without
  `--admission` plan exactly as before: the MSC Irina seed fitness is unchanged to the
  digit. The fields are
  specified in `docs/references/ir-v1.md` §3.5 and §4.1, and the record is
  `docs/references/sessions/2026-11-01-admission.md`.
- **GPUs can be checked device by device**
  ([#24](https://github.com/AlexanderSlokov/kuberina/issues/24), X-3). A node may
  declare `gpuDevices`, and a declaration may state `gpuMilli`, the share of each device
  it needs. On such a node, the solver and the inspector check that every Pod fits on
  real devices, in addition to the per-node GPU sum. Before this change, a half-GPU Pod
  was accepted onto a node whose eight GPUs were each 60% used, because 3.2 GPUs were
  free in total. The solver now reports that placement as 0.50 GPU over capacity, and
  the inspector reports a GPU device violation. Files without the two fields plan
  exactly as before: the MSC Irina seed fitness is unchanged to the digit. The fields
  are specified in `docs/references/ir-v1.md` §2 and §3.4, and the Alibaba trace
  adapter now writes them. The record is
  `docs/references/sessions/2026-11-01-per-device-gpu.md`.
- **A trace adapter converts the Alibaba GPU trace into Kuberina IR**
  ([#24](https://github.com/AlexanderSlokov/kuberina/issues/24), X-2). The
  `bench/alibaba/` package reads the trace CSV files and the FGD simulator's logs and
  writes `infra.yaml`, `workloads.yaml`, and `sequence.csv`. The workload holds the
  Pods in the order the simulator offered them, and `sequence.csv` records which node
  each simulator policy chose for each Pod. For seed 42, the adapter writes 1,213 nodes
  and 10,866 Pods whose GPU demand, 8,075,080 milli-GPU, equals the simulator's logged
  arrivals at every Pod. The adapter writes nothing when the logs disagree with each
  other or with the trace. Run `make bench-alibaba-fetch` and then
  `make bench-alibaba-ir`. The record is
  `docs/references/sessions/2026-11-01-alibaba-trace-adapter.md`. The output is not yet
  a meaningful planning input: GPU stays one number per node until X-3, and every Pod
  must be placed until X-4.
- **The FGD scheduler simulator reproduces its published results on this workstation**
  ([#24](https://github.com/AlexanderSlokov/kuberina/issues/24), X-1). On the Alibaba
  `openb_pod_list_default` trace with seed 42, FGD reaches 95.29% GPU allocation at
  130% arrived demand and Best-Fit reaches 93.13%, both identical to the published
  figures. The comparison of Kuberina against these schedulers rests on this check. The
  record is `docs/references/sessions/2026-10-31-fgd-simulator-calibration.md`.
- **A first tutorial, [Plan your first cluster](docs/tutorials/plan-your-first-cluster.md).**
  It takes a reader with no Kuberina experience from the sample manifests in
  `forge/testdata/manifests` to a blueprint and pinned manifests, without a running
  cluster. Every command was run at commit `c15a04c`, and each step shows the output the
  reader should see. Writing it surfaced two defects, now tracked as K-2 (topology
  spread ignores the standard Kubernetes zone key) and K-3 (the MSC Irina testdata and
  the inspector predate IR v1) in `BACKLOG.md`. The tutorial and the README state both
  limits where a reader meets them.
- **A documentation style guide for this repository,** in `AGENTS.md` under *Writing
  documentation*. It adapts the Kubernetes documentation style guide and page content
  types to GitHub-rendered Markdown, and `CLAUDE.md` loads it for Claude Code.
- **`kuberina-forge`, the Go frontend and linker, is real.** The repository root no
  longer holds a seven-line stub; `forge/` holds a working CLI with two commands.
  `kuberina-forge in` reads Kubernetes manifests — files, directories, or stdin — and
  emits Kuberina IR: Nodes and DaemonSets become the infrastructure document,
  Deployments, StatefulSets, ReplicaSets, Jobs and bare Pods become workload
  declarations. Container requests are summed the way the kubelet sums them, Kubernetes
  quantities are converted to IR units (`2Ti` → 2048 GiB, `16000m` → 16 cores),
  `nvidia.com/gpu` and friends map onto the GPU dimension, and `podAffinity`
  `matchLabels` selectors are resolved into the `namespace/name` identities the solver
  works in. A selector it cannot express — `matchExpressions` — is reported rather than
  dropped. Reading stdin is what puts Helm and Kustomize in reach without linking
  either: `helm template ./chart | kuberina-forge in -`.
  `kuberina-forge out` links a blueprint back onto the manifests it was planned from,
  writing them out unchanged except for a required `nodeAffinity` restricting each
  workload to the nodes its replicas were planned onto. It refuses a blueprint the
  solver marked infeasible, one whose entries do not match the given manifests, and one
  whose own metadata disagrees with its body. Your manifests survive intact — images,
  env, probes, and any field Kuberina does not model are preserved, because the linker
  edits the document tree rather than re-serializing a lossy view of it. (#7)
- **Kuberina IR v1 is declared, with a normative specification.**
  `docs/references/ir-v1.md` states every field of every document kind, its type, unit,
  default, and whether it is required; where an implementation disagrees with it, the
  implementation has the bug. Documents now identify themselves with
  `apiVersion: kuberina.io/v1` and a `kind` — optional on input for older files, always
  written on output — and an `apiVersion` the binary does not implement is a parse
  error rather than a guess. `deny_unknown_fields` extends from the resource block to
  every IR structure, so a typo or a stale field name fails at parse time with the
  offending key named. Blueprints gain the same header plus a `metadata` block
  recording feasibility, pod count and active nodes, which is what lets the forge check
  a plan before acting on it. The stability contract — additive-only evolution inside
  v1, breaking changes to v2 — is [ADR-0002](docs/explanation/adr/0002-kuberina-ir-v1-stability-contract.md).
- **Explicit pod groups in the IR.** A `groups:` block declares a gang directly,
  carrying `name`, `members`, `min_members`, `colocate` and `nodeSelector`. Members name
  declarations as `namespace/name` and every replica joins. `min_members` and
  `colocate` were branched on by the CSP layer and the fitness function but no input
  file could set either, so partial gang admission and forced co-location were
  unreachable outside unit tests. The inline `gang:` shorthand keeps working with its
  existing defaults, an explicit declaration wins over it, and a pod claimed by two
  groups is an error. (#9)
- **Optional `observed` usage block in the IR.** A per-pod record of steady-state
  consumption — `window`, `p50`/`p95`/`p99`/`peak` in the same eight-dimensional shape
  as `requests`, and `exceeded_request_fraction` — parsed and carried but not yet packed
  against. Absent, behavior is byte-identical to before. It exists so any producer of
  telemetry can record usage on the same terms as every other input: a file the
  operator reviews, not an endpoint the planner polls. (#13)
- **Architecture decision records**, under `docs/explanation/adr/`.
  [ADR-0001](docs/explanation/adr/0001-go-component-is-a-cli.md) settles a
  contradiction the repository had carried in writing: `preplan.md` described an
  in-cluster controller while ROADMAP Phase 3 forbids a planner that re-plans itself.
  The Go component is a CLI with no control loop, no cluster writes, and no
  self-triggered re-planning; the operator concern moves to the Kuberina Integration
  Platform, a separate closed-source repository that talks to this one through the IR
  like any other producer.
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
- **Free admission no longer trades a capacity breach for placed Pods**
  ([#24](https://github.com/AlexanderSlokov/kuberina/issues/24), found by X-5). On the
  Alibaba seed-42 instance, free admission wrote an infeasible blueprint at 107.04% GPU
  allocation: a breach paid its 1,000,000 base penalty once, while each placed GPU
  saved 46,560 in unplaced charges. A breach now also pays the charge for leaving every
  Pod unplaced, so any feasible blueprint scores better. Runs without `--admission`
  score as before: the MSC Irina seed fitness is unchanged to the digit.
- **The banner of an infeasible blueprint is aligned.** Its second line, "# Written for
  inspection only", carried 13 spaces of source indentation into the file. The writer
  now lives in `solver/src/blueprint_yaml.rs`, with tests for the banner and for the
  document shape.
- **Affinity and anti-affinity rules now affect plans.**
  ([#26](https://github.com/AlexanderSlokov/kuberina/issues/26)) Before this change, no
  rule reached the solver: the fitness function looked targets up by bare Pod name, so
  `ai/trainer` as written by `kuberina-forge` matched nothing, and the MSC Irina testdata
  kept all its rules in a `pods:` list that the solver did not read. Targets now name a
  declaration (`ai/trainer`, every replica) or one Pod (`ai/trainer-0003`), and a target
  that names nothing is a parse error. Affinity follows Kubernetes semantics: one named
  Pod on the node satisfies the rule. The inspector reads IR v1 workloads and checks
  affinity with its own code, and the MSC Irina testdata is regenerated with
  declaration-level rules. `make solver-irina` and `make inspector-run` work again. With the rules in force, a plan of MSC Irina takes 865
  seconds instead of 185, uses the same 540 nodes, and leaves 964 rules unmet. Operators
  whose workloads name targets without a namespace must add it; the error message names
  the target. See `docs/references/sessions/2026-11-01-irina-with-affinity.md`.
- **`make test` runs every test suite.** It covers `kuberina-forge`, the solver, the
  inspector, and the benchmark scripts, which had no tests before.
- **The README describes what Kuberina does today, and its commands run.**
  ([#6](https://github.com/AlexanderSlokov/kuberina/issues/6),
  [#14](https://github.com/AlexanderSlokov/kuberina/issues/14))
  The `kuberina-forge in` example placed its flags after the source paths. The Go flag
  parser stops at the first source, so the command exited with "nothing to do" and wrote
  no IR. The example now puts flags first, and the README says why. The feature list no
  longer claims that Kuberina injects `nodeSelector`, affinity, and toleration rules into
  manifests: `kuberina-forge out` adds one required `nodeAffinity` per workload, and that
  is what the README now states. A *Before you begin* section lists the tools each
  `make` target needs, including `uv` for the inspector, benchmark, and research targets,
  which the README had not mentioned. A *What Kuberina is not* section states that
  Kuberina does not replace `kube-scheduler`, connect to a cluster, scale workloads, or
  re-plan by itself. The README follows the Kubernetes documentation style guide:
  second person, present tense, sentence-case headings, and no unmeasured performance
  claims. The one runtime figure it quotes, 185 seconds for MSC Irina, cites its session
  record. The licensing text is unchanged.
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
