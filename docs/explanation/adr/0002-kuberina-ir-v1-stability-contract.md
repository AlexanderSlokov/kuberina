# ADR-0002 — Kuberina IR v1 stability contract

**Status:** Accepted — 2026-09-06
**Deciders:** Dinh Tan Dung
**Affects:** `solver/src/parser.rs`, `kuberina-forge`, `bench/gen_irina_testdata.py`,
`research/`, issues #9 and #13
**Depends on:** [ADR-0001](0001-go-component-is-a-cli.md)

## Context

The IR is the seam of the whole system. The solver reads it, `kuberina-forge` writes it
([ADR-0001](0001-go-component-is-a-cli.md)), the benchmark generator emits it, the
inspector re-derives capacity from it, the Python reference implementation parses it,
and — per ROADMAP Phase 3 — Kalena will eventually populate part of it. Six producers
and consumers, in three languages, with no version marker in the format they share.

Two events made the cost of that concrete.

**The eight-dimension defect.** The generator emitted throughput flat (`disk_read`,
`net_in`) while the IR nested it under `disk` and `network`. `serde` discarded the
unknown keys silently: pod demand parsed as zero, node capacity fell through to
`f64::MAX`, and the solver planned in four dimensions while reporting eight. Every
result published before the fix was wrong, and only the inspector caught it. The
response — `deny_unknown_fields` on `RawResources` — was correct but local.

**Reachability gaps.** `PodGroup.min_members` and `PodGroup.colocate` exist in the
domain model, the CSP branches on them and the fitness function penalizes them, yet no
input file can set either: the parser auto-generates groups with fixed values
(issue [#9](https://github.com/AlexanderSlokov/kuberina/issues/9)). Separately, nothing
in the IR can express observed usage, so the runtime-feedback gap PAPER §8.1 names has
nowhere to land (issue [#13](https://github.com/AlexanderSlokov/kuberina/issues/13)).

ROADMAP Phase 1 states that v0.3.0 is the last release before `kuberina-forge` fixes
these interfaces in place. Writing forge against a schema that is still moving means
writing it twice.

## Decision

**Kuberina IR v1 is declared, and `docs/references/ir-v1.md` is its normative
specification.** Where an implementation and that document disagree, the document is
right and the implementation has a bug.

The contract has four clauses.

**1. Documents identify themselves.** Every IR document carries `apiVersion:
kuberina.io/v1` and a `kind` — `Infrastructure`, `Workloads`, or `Blueprint`. Both
fields are optional on input for compatibility with files written before this record, in
which case v1 is assumed; every document Kuberina *writes* carries them. An
`apiVersion` naming a version the binary does not implement is a parse error, not a
warning: a tool that guesses at an unknown schema is how the eight-dimension defect
happened.

**2. Unknown fields are errors, everywhere.** `deny_unknown_fields` extends from
`RawResources` to every IR structure. A typo, a stale field name, or a document written
for a newer version fails at parse time with the offending key named. Silence is not an
acceptable response to input a parser does not understand.

**3. v1 evolves by addition only.** Within v1, new *optional* fields may be added, and
their absence must reproduce today's behavior byte for byte. What may not change: the
meaning of an existing field, its units, its default, whether it is required, or the
shape of a document. Two additions land with this decision and are the last ones that
change the surface for a while — the explicit `groups:` block (#9) and the optional
`observed:` block (#13).

**4. Breaking changes go to v2, and v1 stays readable.** A change that violates clause 3
requires `apiVersion: kuberina.io/v2` and a new specification. When v2 arrives, the
solver keeps reading v1 for at least one minor release, and `kuberina-forge` gains a
conversion path. Version bumps are announced in `CHANGELOG.md` under a **BREAKING**
heading, as `--pareto` → `--headroom` was.

**Units are part of the contract, not a convention.** CPU in cores, RAM and storage in
GiB, GPU in whole units, disk and network throughput in MB/s. String quantities in
Kubernetes notation (`512Mi`, `2500m`, `1Gi`) are accepted wherever a number is, and
parse through one shared implementation. A future dimension arrives with its unit
declared in the specification or it does not arrive.

## Consequences

**What this makes easy.**

- Forge can be written once. The schema it targets is stated in one document rather than
  inferred from `parser.rs`, and a change to that document is a reviewable event.
- Cross-implementation drift becomes detectable. The Rust parser, the Python reference
  implementation and the Go forge can each be checked against the same specification
  instead of against each other.
- The gang machinery and the runtime-feedback path stop being unreachable code.

**What this makes hard, and we accept.**

- Adding a resource dimension is now a deliberate act with a specification edit, not an
  incidental struct field. Given that the last incidental change cost every published
  result its validity, this is the intended trade.
- Files written before this record that contain typos will now fail to parse instead of
  quietly losing a field. That is the correction, not a regression.
- `deny_unknown_fields` means a v1 solver rejects a v1.1 document that uses a new
  optional field. Additive evolution is safe forward, not backward; operators upgrade
  the solver before the producer.

**What it forecloses.** Reinterpreting an existing field is off the table inside v1 —
including "clarifying" a unit. If `ram` should have been GiB and is not, that is a v2
change with a conversion path, not a patch.

## Alternatives considered

**Leave the IR unversioned and keep it stable by convention.** Rejected. That was the
policy in force when the eight-dimension defect shipped, and the defect was invisible
precisely because no document could say which schema a file claimed to be.

**Adopt Kubernetes' own group/version/kind machinery.** Rejected as overkill. Kuberina
IR is a planning input, not an API served to controllers; it needs a version marker and
a written schema, not conversion webhooks and a scheme registry. The `apiVersion`/`kind`
*spelling* is borrowed because operators already read it fluently — the machinery behind
it is not.

**Require `apiVersion` immediately.** Rejected for v1. Every existing testdata file,
every example in the README, and both benchmark inputs would break at once for no gain
in correctness — the assumed default is unambiguous while only one version exists. v2
may require it.

**Publish a JSON Schema and validate against it.** Deferred rather than rejected. It is
the right long-term shape, especially with three implementations, but the schema is
worth writing once the v1 surface has survived a release rather than the week it was
declared.

## References

- `docs/references/ir-v1.md` — the specification this record declares
- `CHANGELOG.md`, `[Unreleased]` → Fixed — the eight-dimension defect and the
  `deny_unknown_fields` response
- `ROADMAP.md` Phase 1, "CLI and IR Surface Stabilization"
- Issues [#9](https://github.com/AlexanderSlokov/kuberina/issues/9) (explicit pod
  groups), [#13](https://github.com/AlexanderSlokov/kuberina/issues/13) (`observed`
  block)
- [ADR-0001](0001-go-component-is-a-cli.md) — why forge, the IR's main producer, is a
  CLI
