# Kuberina Backlog

Concrete, actionable tasks. Where [ROADMAP.md](ROADMAP.md) states *what capability
Kuberina will gain and in which release*, this file states *what has to be edited to
get there*. Every item names the files it touches and the condition under which it is
done. Completed items leave this file and are described in
[CHANGELOG.md](CHANGELOG.md) under `[Unreleased]`.

Items are grouped by the artifact they modify, and the groups are ordered by
dependency. Per `AGENTS.md`, a task that adds capability descends from a ROADMAP item;
a task that corrects an artifact which already exists (a wrong figure, a stale path, a
dangling citation) enters here directly.

**The white paper is last, and that ordering is a rule, not a preference.**
`docs/references/PAPER.md` reports on the repository; it does not direct it. Every
section above the paper must be settled before the paper section can be worked, because
regenerating §6–§7 against a repository that is still moving means regenerating them
twice. The Borg paper followed Borg by a decade for the same reason.

---

## Solver (`solver/`)

### S-1 — Rename `--pareto p` to `--headroom h`

**Issue:** [#12](https://github.com/AlexanderSlokov/kuberina/issues/12) ·
**ROADMAP:** Phase 1 (v0.3.0) · **Blocks:** W-1, W-4

The flag reserves capacity headroom. "Pareto" names neither a Pareto front nor an 80/20
distributional claim, and the direction of the number is ambiguous — `--pareto 80`
reserves 20%, which reads backwards. `--headroom 20` says what is reserved.

The semantics invert with the name: today `factor = p / 100.0`, after the rename
`factor = 1.0 - h / 100.0`.

Touches `solver/src/main.rs` only (arg definition at `main.rs:41-43`, destructuring at
`main.rs:54-59`, the capped-node construction at `main.rs:83-95`) plus the Makefile
target `solver-irina-pareto-80`. No test references the flag.

Two defects to fix in the same pass:

- The current code accepts `--pareto 800` and silently multiplies capacity by 8.
  Validate `0.0 <= h < 100.0` and exit naming the offending value.
- The doc comment on `run_plan` does not describe the dual-capacity model, which is why
  W-4 exists. State it there while the function is open.

Preserve the `f64::MAX` guard comment at `main.rs:91` — it records why unconstrained
dimensions are skipped rather than scaled.

**Done when:** the flag is `--headroom`, out-of-range values are rejected with the
value in the message, `cargo test && cargo clippy -- -D warnings && cargo fmt --check`
passes, and `grep -rn pareto --include=*.rs .` returns nothing.

**Breaking CLI change.** It lands at a release boundary and needs its own CHANGELOG
callout.

### S-2 — Make gang IR fields reachable from a workload file

**Issue:** [#9](https://github.com/AlexanderSlokov/kuberina/issues/9) ·
**ROADMAP:** Phase 1 (v0.3.0)

`PodGroup` carries `min_members` and `colocate` (`solver/src/model.rs:226,228`), the
CSP layer branches on `colocate` (`csp.rs:98`) and the fitness function penalizes
`placed < min_members` (`fitness.rs:321`). Neither field can be set from YAML.
`auto_group_gangs` hardcodes both — `min_members = indices.len()` (`parser.rs:391`) and
`colocate: false` (`parser.rs:397`) — so partial gang admission and forced co-location
are unreachable outside unit tests.

This is a stronger statement than issue #9 makes. The machinery is not merely
unexercised by the benchmark; there is no input that would exercise it.

**Done when:** the IR accepts an explicit `groups:` block carrying `name`, `members`,
`min_members` and `colocate`, the inline `gang:` shorthand continues to auto-group with
today's defaults, and parser tests cover `min_members < |G|` and `colocate: true`.

---

## Benchmark and testdata (`bench/`)

### B-1 — Teach the proof script the capacity reserve

**Issue:** [#8](https://github.com/AlexanderSlokov/kuberina/issues/8) ·
**Depends on:** S-1 · **Blocks:** W-2

`bench/mathematical_proof.py` has no CLI. Its three input paths are hardcoded
(`mathematical_proof.py:415-417`) and it has no way to express the capacity reserve the
solver applied. The consequence is W-2: §7.3 divides an active-node count from a
reserved-capacity run by a lower bound computed from full capacity.

Add `argparse` to `main()` with `--headroom`, plus `--infra`, `--workloads` and
`--solution` overrides so the script can verify a run other than the default one.
Scale `total_supply` in `compute_heterogeneous_lower_bound`
(`mathematical_proof.py:243`) and per-node capacity in `compute_lp_lower_bound` by the
same `1 - h/100` factor the solver uses, guarding `float('inf')` the way the solver
guards `f64::MAX`.

Add a `bench-proof-headroom-20` Makefile target. `make bench-proof --headroom 20` would
be parsed as a flag to `make` itself, so the argument needs a target of its own.

**Done when:** both ratios can be produced from the same script, each labeled with the
capacity model it assumes, and the reserved run reports a strictly smaller
`total_supply` and a larger `L_het` than the unreserved one.

### B-2 — Populate the benchmark with pod groups

**Issue:** [#9](https://github.com/AlexanderSlokov/kuberina/issues/9) ·
**Depends on:** S-2 · **ROADMAP:** Phase 1 (v0.3.0)

`bench/gen_irina_testdata.py` contains no gang generation logic, and the MSC Irina
workload carries zero pod groups. Every published result therefore runs with `|G| = 0`,
which makes Gang Repair a no-op in every generation and leaves Forward Checking with no
group capacity to look ahead for. PAPER §8.5 acknowledges this; nothing in the
repository fails if the gang code regresses.

The motivating example in §4.5 — a 64-GPU training job where aggregating 60 pods is
worse than placing none — is precisely the scenario the benchmark omits.

**Done when:** the generator emits large GPU training gangs sized near but under the
available GPU units, medium gangs that fit on one node (to exercise co-location
distinctly from spread), and at least one gang deliberately too large to place, so
all-or-nothing rejection is observable. `inspector/inspector.py` verifies the
all-or-nothing predicate independently, as it already does for capacity and selectors.

Blocked by S-2: there is no IR syntax to express any of this yet.

### B-3 — Correct the generated infra header comment

`solver/testdata/irina_infra.yaml` opens with
`# 100 Standard + 30 Memory + 20 GPU nodes`. The generator emits 400 standard, 120
memory-optimized and 100 GPU (`bench/gen_irina_testdata.py:40,51,62`), for 620 total.
The comment matches neither the file it heads nor the code that wrote it.

The line is emitted by the generator, so fix it there and regenerate.

**Done when:** the header reports the counts the generator actually produces.

---

## Intermediate representation and Kalena interoperability

### K-1 — Optional `observed` usage block in the workload IR

**Issue:** [#13](https://github.com/AlexanderSlokov/kuberina/issues/13) ·
**ROADMAP:** Phase 1 (v0.3.0) for the schema, Phase 3 (v0.5.0) for the pairing

PAPER §8.1 names "No Runtime Feedback Loop" as a limitation and asks operators to
choose a re-planning trigger, including "event-driven (when utilization deviation
exceeds a threshold)". Nothing in the IR can express observed usage, so neither the
feedback nor the trigger has anywhere to land. `docs/DESIGN.md:469` lists the same idea
as a future consideration.

Add an optional `observed` block alongside `requests`, carrying a retention `window`,
`p50`/`p95`/`p99`/`peak` vectors in the existing 8-dimensional `ResourceVector` shape,
and `exceeded_request_fraction`. Absent, behavior is byte-identical to today.

Kuberina must not consume telemetry directly or re-plan itself. The blueprint's value
rests on being a reviewed artifact; a planner that silently regenerates from live
metrics gives that up. The block is ingested from a file on the same terms as every
other input.

**Done when:** the parser round-trips the block, an absent block changes nothing, and
`inspector/inspector.py` reports per-node request-versus-observed divergence alongside
the existing utilization heatmap — which is what makes blueprint staleness visible in
the tool operators already open.

Selecting what to pack against (`--pack-against p95`) is deliberately **not** part of
this item. Changing the packing input changes what the feasibility proof guarantees,
and that decision waits for real data.

---

## Documentation (`README.md`, `docs/DESIGN.md`, component READMEs)

### D-1 — Declare the `uv` prerequisite

**Issue:** [#6](https://github.com/AlexanderSlokov/kuberina/issues/6)

The README walkthrough runs three commands. The first two need Rust; `make
inspector-run` needs `uv`, and nothing says so. It is the step that produces
`kuberina_dashboard.html`, which the next line tells the reader to open, so a reader
without `uv` fails at the payoff.

**Done when:** a Prerequisites block precedes the walkthrough (Rust for `solver-*`;
`uv` for `inspector-*`, `bench-*`, `research-*`; both for `full-pipeline`), each
command is annotated with what it needs, and the README states that `inspector-*` and
`bench-*` provision dependencies ad hoc via `uv run --with pyyaml` while `research-*`
runs against the locked environment in `research/pyproject.toml`.

### D-2 — Close the requests-versus-limits question

**Issue:** [#10](https://github.com/AlexanderSlokov/kuberina/issues/10)

`docs/DESIGN.md:357` still poses "Kuberina dùng requests hay limits để tính toán bin
packing?" as open. The solver answered it: `parser.rs` reads `requests` and `limits`
appears nowhere in the parser, the model, or any testdata file.

The decision is correct and should be recorded with its consequence. Packing against
requests matches `kube-scheduler`'s own admission arithmetic, so a blueprint is
feasible under exactly the conditions the cluster would accept. It also means the
feasibility proof covers reserved capacity, not peak consumption — a node at 100% of
requests can still be driven past capacity by Burstable pods bursting toward their
limits. The paper's "zero constraint violations" reads as a stronger claim than a
requests-based model supports.

**Done when:** §5 states the decision, its rationale, and that consequence, and names
`--headroom` as the mitigation for burst exposure. The flag is currently presented
only as a Resource Canal / autoscaling feature; this is its second and more concrete
role.

**Follow-on, not required here:** if `limits` are ever ingested, the natural use is not
to pack against them but to report per-node burst exposure — `Σ(limit − request)` over
the pods placed on each node — which tells an operator which nodes are most exposed to
runtime overcommit without changing the packing model.

### D-3 — Restate Core Features as what exists, and add Non-Goals

**Issues:** [#14](https://github.com/AlexanderSlokov/kuberina/issues/14),
[#7](https://github.com/AlexanderSlokov/kuberina/issues/7)

Two claims in the README describe `kuberina-forge`, which is scheduled for v0.3.0 and
does not exist:

- "Auto-Inject Constraints" (`README.md:82`) promises `NodeSelector`, `PodAffinity`,
  `PodAntiAffinity` and `Tolerations` written into an output YAML. The solver emits a
  flat `namespace/pod: node` map. Affinity and toleration data flows *into* the solver
  (`parser.rs:93-125`) and is consumed by the CSP and fitness layers; nothing is
  written back out.
- The workflow example (`README.md:61-70`) runs `kuberina plan → blueprint.yaml` and
  ends in `kubectl apply -f blueprint-final.yaml`. `main.go` prints one line, and the
  solver writes `kuberina_solution.yaml`, not `blueprint.yaml`. The current output
  would be rejected as an unrecognized resource.

The repository-layout table already labels `main.go` as a placeholder stub, so the two
halves of the README disagree with each other.

**Done when:** the feature is restated as constraint-aware placement over a
pod-to-node assignment map with manifest rendering named as forge work; the workflow
snippet is kept but labeled as the target UX with a pointer to what runs today; the
artifact name is reconciled in both directions; and a Non-Goals section states that
Kuberina is not a runtime scheduler, does not replace `kube-scheduler`, does not handle
autoscaling, and does not yet render manifests. The `[Q-Claude]` note at `README.md:86`
already proposes exactly this section.

### D-4 — Resolve the dangling `Appendix C.1` citation

`inspector/README.md:13` and `CHANGELOG.md:23` both cite "the Kalena interface contract
(Appendix C.1)" in `PAPER.md`. The paper has ten numbered sections and no appendices.

Under the proof-of-work rule the citation is wrong, not the paper. Either drop the
reference or write the appendix — but the appendix can only be written once the Kalena
contract exists, which is Phase 3 work.

**Done when:** the citation either points at a section that exists or is removed.

---

## White paper (`docs/references/PAPER.md`)

**Gated on every section above.** These items are corrections to a document that
reports results; they cannot be completed while the results are still moving.

The published figures in §6 and §7 were measured against a testbed that is no longer in
the repository, and three accuracy gaps sit on top of that. W-1 is a prerequisite for
W-2 and W-3: correcting the ratio or the dimension coverage before the underlying run
is regenerated means computing both twice.

### W-1 — Regenerate §6.1 and §7 against the current testbed

**Blocks:** W-2, W-3 · **Depends on:** S-1, B-1, B-3

Commit `f7520f0` regenerated `solver/testdata/` to IR v0.2.0 and, in doing so, moved
the testbed from 186 nodes to 620. Its own message records the consequence: "The
paper's numbers are not reproducible against this file and the benchmark section needs
re-running before the next release."

| Quantity | §6.1 states | `solver/testdata/` holds |
|---|---|---|
| Total nodes | 186 | 620 |
| Total pods | 2,714 | 2,714 |
| Cluster CPU (raw) | 10,272 | 34,240 |
| Cluster RAM (raw) | 54,912 | 183,040 |
| Cluster GPU | 240 | 800 |
| CPU fill | 73.1% | 21.0% |
| RAM fill | 51.2% | 15.2% |

Pod demand is unchanged while capacity grew roughly 3.3×, so the published run
describes a materially harder instance than `make solver-irina` solves today. Every
figure in §7.1, §7.2 and §7.3 descends from the 186-node run: 152 active nodes, 88.7%
average CPU utilization, fragmentation 18,418.00, `L = 117`, `L_het = 136`,
`α = 1.3382`. None of them reproduce.

**The correction is to the paper, not the testbed.** `AGENTS.md` is explicit that the
paper follows the repository and that the repository is not to be changed to serve the
paper's claims. The regenerated numbers will describe a looser instance and will read
as less impressive; that is the honest result of the benchmark as it currently stands.
Making the benchmark denser is a legitimate goal on its own merits — a 21%-fill
instance under-exercises multi-dimensional bin packing — but it is benchmark work,
judged as benchmark work, and never a step taken to protect a published figure.

**Done when:** §6.1 and all of §7 are regenerated from a single recorded run of the
current testdata, that run's console output is committed under
`docs/plans/benchmarks/`, and the run reproduces via `make solver-irina`,
`make solver-irina-headroom-20`, `make inspector-run` and `make bench-proof`.

Note that `make solver-irina` overwrites `solver/kuberina_solution.yaml` and both
verifiers read that path, so each configuration must be verified before the next runs.

### W-2 — Report the approximation ratio against a matching capacity model

**Issue:** [#8](https://github.com/AlexanderSlokov/kuberina/issues/8) ·
**Depends on:** W-1, B-1

§7.3 computes `α = 182 / 136`, where the numerator is an active-node count from a
capacity-reserved run and the denominator is a lower bound derived from full node
capacities. The two halves come from different capacity models.

The error understates the result. On the same published data the full-packing
configuration gives `152 / 136 = 1.118`, which sits *below* the 11/9 ≈ 1.222 FFD bound
that the following sentence explains the result as exceeding.

**Done when:** §7.3 reports α for the full-packing configuration, whose objective
matches the bound, and reports the reserved configuration against the reserved bound
produced by B-1. The abstract names the configuration its α came from.

### W-3 — Extend the feasibility proof and LP bound to all 8 dimensions

**Issue:** [#11](https://github.com/AlexanderSlokov/kuberina/issues/11) ·
**Depends on:** W-1

`bench/mathematical_proof.py` iterates all eight dimensions in `compute_lp_lower_bound`,
`compute_heterogeneous_lower_bound` and the capacity verifier. §7.3 reports three:
overflow "on CPU, RAM, GPU", and `L^r` for CPU, RAM and GPU only. §6.1's testbed table
has the same 3-of-8 gap. A reader cannot distinguish "checked and slack" from "not
checked" — which is the distinction a proof section exists to settle.

The five dimensions added in v0.2.0 are not slack on the current testbed. All 620 nodes
carry finite capacity in all eight dimensions, and four of the new dimensions show
higher fill than CPU:

| Dimension | Fill |
|---|---|
| disk_write | 54.1% |
| net_out | 33.4% |
| net_in | 31.2% |
| disk_read | 30.3% |
| cpu | 21.0% |
| gpu | 19.0% |
| storage | 15.9% |
| ram | 15.2% |

**Done when:** Proof 1 reports overflow per dimension and Proof 2 reports `L^r` per
dimension, both across all eight; §6.1 carries cluster totals for storage, disk and
network; and any dimension left unconstrained is marked as such rather than omitted.

**Note:** the `float('inf')` default at `mathematical_proof.py:57` (mirroring the
solver's `f64::MAX`) does not trigger on this testbed — no node omits a dimension. The
branch stays correct for hand-written infra files; it is simply not what §7.3 is
currently hiding.

### W-4 — Document the dual-capacity model behind `--headroom`

**Issue:** [#12](https://github.com/AlexanderSlokov/kuberina/issues/12) ·
**Depends on:** S-1

Under a capacity reserve, `solver/src/main.rs` builds a reduced copy of the node set and
hands it to the FFD warm-start (`main.rs:116`), the fitness function (`main.rs:118`) and
the GA (`main.rs:129`), then prints the blueprint against the true post-DaemonSet
capacities (`main.rs:137`). That asymmetry is the flag's defining property and is
documented only in the source comment at `main.rs:136`.

It is worth stating because the guarantee is stronger than §7.2 currently claims. "No
node exceeds 79% utilization" reads as an observed outcome; the mechanism makes it
structural and per-node — the optimizer never sees the reserve, so every node in the
blueprint carries at least the reserved fraction of its real capacity unallocated.

Left undocumented, two misreadings are available: that the cap is a post-hoc rejection
filter, making the reserve a soft target; or that reported utilization percentages are
relative to reduced capacity, making 74.6% mean 59.7% of real capacity.

**Done when:** §6.2 states which components see reduced capacity and which see real
capacity, and §7.2 notes that its fragmentation row is measured against reduced
capacity in the reserved column and real capacity in the full-packing column — the two
are not comparable as printed. Reporting both against real capacity would make the
table comparable and would show the reserved run's true waste, which is the more honest
number.
