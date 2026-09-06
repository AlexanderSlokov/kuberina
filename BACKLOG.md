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

### S-3 — Give early stopping an improvement threshold

**Issue:** [#20](https://github.com/AlexanderSlokov/kuberina/issues/20) ·
**ROADMAP:** Phase 1 (v0.3.0)

`stale_count` resets whenever the best fitness improves at all, including by ~0.0003 in
absolute terms. On the MSC Irina benchmark neither configuration ever triggers the
`N_stop` criterion: both burn the full 1,000-generation budget, ~900 s each, for total
gains of 0.62% (full packing) and 0.0014% (20% headroom). The run at commit `1ab1ad7`
is recorded in `docs/references/sessions/2026-09-06-solver-audit.md`, and the behavior
is stated as an open defect in `docs/references/PAPER.md` §7.4.

The criterion should test for improvement that matters, not improvement that exists —
a relative threshold against current best fitness, so that a run stops when progress
falls below it.

**Done when:** `GaConfig` carries a relative improvement threshold with a documented
default, `stale_count` resets only on gains exceeding it, and a regression test asserts
that a run whose fitness improves by less than the threshold for `N_stop` generations
terminates early.

---

## Benchmark and testdata (`bench/`)

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
