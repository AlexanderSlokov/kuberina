# Session record — affinity constraints never reached the solver — 2026-11-01

**Commits examined:** `24ffc52` (the run that PAPER §7 reports) and `0e30c94` (HEAD of
`24-challenge-alibaba`).
**Found while:** starting backlog item K-3.
**Host:** Linux 7.0.0-38-generic x86_64.

This record reproduces two defects in how Kuberina reads affinity and anti-affinity
rules. Both leave the solver with no affinity constraints while it reports zero
affinity violations.

## Defect 1: the MSC Irina constraints sit in a section the solver does not read

The generator `bench/gen_irina_testdata.py` writes each service once under
`namespaces:`, and writes every Pod a second time under a top-level `pods:` list. It
attaches the affinity and anti-affinity targets to the `pods:` copies only.

At `24ffc52`, the solver's `RawWorkloadFile` struct has a single field, `namespaces`,
and ignores unknown keys. It reads no affinity rules from this file.

```shell
git show 24ffc52:solver/src/parser.rs | grep -n "struct RawWorkloadFile" -A3
git show 24ffc52:solver/testdata/irina_workloads.yaml | awk '/^pods:/{exit} {print}' \
  | grep -cE "^\s+(affinity|antiAffinity):"
git show 24ffc52:solver/testdata/irina_workloads.yaml | awk '/^pods:/{f=1} f' \
  | grep -cE "^\s+- [a-z-]+-[0-9]{4}$"
```

| Quantity | Value |
|---|---|
| Fields in `RawWorkloadFile` at `24ffc52` | `namespaces` only |
| Affinity or anti-affinity keys under `namespaces:` | 0 |
| Affinity and anti-affinity targets under `pods:` | 14,258 |

The file is unchanged between `24ffc52` and `0e30c94`. The count of 14,258 equals the
constraint count in PAPER §7 and in `2026-09-06-solver-audit.md`.

Neither `inspector/inspector.py` nor `bench/mathematical_proof.py` contains a check for
affinity or anti-affinity (`grep -n -i affinity` returns no lines in either file). The
"Affinity violations: 0" figure in the 2026-09-06 records is the solver's count over
zero constraints.

At `0e30c94`, the parser rejects unknown keys, so the same file fails to load. That
failure is backlog item K-3.

## Defect 2: affinity targets from `kuberina-forge` match no Pod

The IR specification (`docs/references/ir-v1.md` §3.1) says affinity targets are matched
against unrolled identities, which have the form `namespace/podname`. Its own example
uses a declaration name, `ai/parameter-server`.

At `0e30c94`:

- The `kuberina-forge in` tool writes declaration-level targets. For
  `forge/testdata/manifests`, it writes `affinity: [ai/trainer]`.
- The parser stores the unrolled Pod name without its namespace, for example
  `trainer-0000` (`solver/src/parser.rs`, `format_replica_name`).
- The `compute_affinity_violations` function (`solver/src/fitness.rs`) looks each
  target up by exact Pod name. The target `ai/trainer` matches no Pod and is skipped.

In the tutorial sample, the `parameter-server` Pod lands next to the trainers because
the solver minimizes active nodes. Its affinity rule played no part.

## Consequence for PAPER.md

PAPER §7 states: "All 14,258 affinity and anti-affinity constraints satisfied." At
`24ffc52`, the solver did not receive these constraints, and no validator checked them.
Under the proof-of-work rule in `AGENTS.md`, the claim is unsupported and the paper must
be corrected.
