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

## Benchmark and testdata (`bench/`)

### B-2 — Populate the benchmark with pod groups

**Issue:** [#9](https://github.com/AlexanderSlokov/kuberina/issues/9) ·
**ROADMAP:** Phase 1 (v0.3.0)

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

### K-2 — Honor Kubernetes topology keys in topology spread

**Found:** 2026-10-31, while writing the first tutorial, at commit `c15a04c`.

The `kuberina-forge in` tool copies `topologyKey` verbatim, so a Deployment that
spreads by zone arrives in the IR as `topologyKey: topology.kubernetes.io/zone`. The
`compute_topology_spread_penalty` function (`solver/src/fitness.rs`) recognizes only
`zone` and `rack`, and scores every other key as zero. On `forge/testdata/manifests`,
the solver places both `trainer` replicas on `gpu-b` in zone `az-2`, leaves the eligible
`gpu-a` in zone `az-1` empty, and reports a topology spread penalty of 0.00. The
declared `maxSkew` is 1 and the actual skew is 2. `docs/references/ir-v1.md` does not
state which keys are valid, and `inspector/inspector.py` does not check spread.

**Touches:** `docs/references/ir-v1.md` §3, `forge/internal/forgein/convert.go` or
`solver/src/fitness.rs`, `inspector/inspector.py`.
**Done when:** the IR specification lists the accepted keys, a spread on
`topology.kubernetes.io/zone` from a manifest is scored against node zones, a regression
test covers the `forge/testdata` case, and the inspector reports skew per spread group.

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

## Solver search quality (`solver/src/`)

**ROADMAP:** Phase 1 (v0.3.0), *Search Quality and Termination* ·
**Evidence:** `docs/references/sessions/2026-11-01-irina-with-affinity.md`

| ID | Task | Touches | Depends on | Done when |
|---|---|---|---|---|
| S-1 | Resolve affinity targets and spread groups to indices once at load, then evaluate only what a mutation changes | `solver/src/model.rs`, `target_resolution.rs`, `fitness.rs` | — | MSC Irina seed and final fitness match `096177e` to the digit in both configurations, and `compute_fitness` takes under 1 ms per call on the FFD seed |

S-1 gets its own GitHub issue and its own work session, after #24. It does not block
#24: the Alibaba `openb_pod_list_default` trace carries no affinity or topology spread
rule, and those two terms hold all of the measured fitness cost. The cost on the Alibaba
instance is measured separately by X-5.

---

## Paper corrections (`docs/references/PAPER.md`)

These items run last, by the rule at the top of this file.

| ID | Task | Issue | Done when |
|---|---|---|---|
| P-1 | Replace the §7 claim that 14,258 affinity constraints were satisfied | [#26](https://github.com/AlexanderSlokov/kuberina/issues/26) | §7 reports the instance and results of `2026-11-01-irina-with-affinity.md`, and #26 is closed |

---

## External validation on the Alibaba GPU trace

**Issue:** [#24](https://github.com/AlexanderSlokov/kuberina/issues/24) ·
**ROADMAP:** Phase 2 (v0.4.0) · **Plan:** `docs/plans/benchmarks/alibaba-gpu-v2023/00_plan.md`

| ID | Task | Touches | Depends on | Done when |
|---|---|---|---|---|
| X-3 | Per-device GPU model, and the adapter writes the new device fields | `ir-v1.md`, `solver/src/`, `inspector/`, `forge/internal/ir/`, `bench/alibaba/ir_documents.py` | — | 0.5-GPU pod rejected on 8×60%-used devices by solver and inspector; Irina FFD fitness unchanged; the regenerated seed-42 IR still sums to 8,075,080 milli-GPU |
| X-4 | Admission (unplaced pods); decide how the solver learns arrival order, which IR v1 cannot state | `ir-v1.md` §3–§4, `solver/src/`, `inspector/`, `forge/internal/forgeout/` | X-3 | 130% fixture gives a feasible blueprint with `unplaced:` |
| S-2 | Derive objective weights from measured term ranges on the Alibaba instance, including X-4's unplaced-GPU term | `solver/src/model.rs`, `docs/references/` | X-4 | Each weight cites its derivation in a session record, fixed before Kuberina's first run on the trace (plan, *Purpose and stance*) |
| X-5 | Scale check | `docs/references/sessions/` | S-2 | Measured wall clock and memory for one full solve |
| X-6 | Experiments E1–E5 | `bench/alibaba/`, `docs/references/sessions/` | X-5 | One session record per experiment, seeds 42–51 |

---

## Documentation (`README.md`, `docs/DESIGN.md`, component READMEs)

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

### D-4 — Resolve the dangling `Appendix C.1` citation

`inspector/README.md:13` and `CHANGELOG.md:23` both cite "the Kalena interface contract
(Appendix C.1)" in `PAPER.md`. The paper has ten numbered sections and no appendices.

Under the proof-of-work rule the citation is wrong, not the paper. Either drop the
reference or write the appendix — but the appendix can only be written once the Kalena
contract exists, which is Phase 3 work.

**Done when:** the citation either points at a section that exists or is removed.
