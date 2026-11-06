# ADR-0003 — The genetic algorithm stays only if it beats the in-order walk

**Status:** Accepted — 2026-11-02, amended 2026-11-07
**Deciders:** Dinh Tan Dung
**Affects:** `solver/src/phase2_ga.rs`, `solver/src/admission.rs`, the #24 plan
(`docs/plans/benchmarks/alibaba-gpu-v2023/00_plan.md`), experiment E1, issue
[#24](https://github.com/AlexanderSlokov/kuberina/issues/24)

## Context

The solver plans in two phases. Phase 1 builds a first-fit decreasing (FFD) placement,
the _seed_. Phase 2, a genetic algorithm (GA), searches for a better placement from
that seed. On every recorded MSC Irina run, the seed already holds nearly all of the
final result.

The following table lists what the GA changed after the seed, on MSC Irina (620 nodes,
2,714 Pods). Figures come from `2026-11-01-irina-with-affinity.md` (commit `096177e`)
and `2026-09-06-ga-early-stop-threshold.md` (commits `1ab1ad7` and `24ffc52`):

| Run | Seed fitness | Final fitness | Gain | Active nodes, seed → final | Wall clock |
|---|---|---|---|---|---|
| `096177e`, full packing | 1,587,653.1260 | 1,587,391.1255 | 0.0165% | 540 → 540 | 864.67 s |
| `096177e`, `--headroom 20` | 2,311,466.1293 | 2,310,768.1128 | 0.0302% | 615 → 615 | 1,097.14 s |
| `24ffc52`, full packing | 1,582,568.1260 | 1,582,556.1257 | 0.0008% | 540 → 540 | 185.23 s |
| `1ab1ad7`, full packing, 1,000 generations | 1,582,568.1260 | 1,572,736.6090 | 0.62% | 540 → 539 | 912.49 s |

The one change of size in these runs is a single node evacuated near generation 581 of
the 1,000-generation run. At `096177e`, the GA removed 53 of 1,017 affinity violations
in full packing and 139 of 1,099 with headroom, and it moved neither the node count nor
fragmentation.

When the operator relaxes the global resource limit with `--headroom`, the seed places
every Pod, and the node count leaves the GA nothing to win: 615 active nodes before and
after it. The GA's gain in that run, 0.0302%, comes from soft constraints.

The GA's weakness may come partly from the weights that steer it. At `096177e`,
fragmentation contributes 99.35% of the final fitness and affinity 0.30%, so a move that
removes one affinity violation changes the fitness by a fraction the GA barely
distinguishes. S-2 fixes the weights on the trace before the GA is judged there.

Admission (X-4) narrows the GA's role further. Under `--admission in-order`, the GA may
not change which Pods are admitted. The admitted set, which decides experiments E1 and
E2, comes from the in-order _walk_: Pods are taken in arrival order, each goes to the
best-fitting node, and a Pod that does not fit may displace one admitted Pod to another
node. A Pod that still does not fit is rejected, and the walk continues.

Experiment E1 asks which arriving Pod is the first one each planner rejects. The plan
proposed a binary search over arrival prefixes, with one full solve, GA included, per
probe: about 14 solves for each of ten seeds. The cost of one solve on the trace is
unknown until X-5 measures it.

## Decision

1. **The walk measures E1.** For each seed from 42 to 51, one in-order walk gives the
   first rejected Pod. E1 stays the headline comparison (plan, decision 1).
1. **The binary search runs on seed 42 only, after X-5.** Each probe records two
   verdicts on the same prefix: Phase 1 alone, and Phase 1 followed by the GA. The
   difference between the two is the GA's measured contribution on the trace.
1. **The GA's future follows the seed-42 result.** If the walk's first rejection comes
   at or after the boundary that the GA-based binary search finds, the GA does not earn
   its cost. It is then planned for removal in three steps, each a ROADMAP item:
    1. Deprecate: the solver prints a deprecation notice when the GA runs.
    1. Sunset: the GA is off by default, and a flag turns it on.
    1. Remove: the GA code and its flags leave the solver.
1. If the GA-based search finds a later boundary, the GA stays, and the size of its gain
   is recorded next to its cost.

### Amendment of 2026-11-07

A probe on the seed-42 instance showed that the GA's place move breached capacity in 300
of 300 trials, and that its move and swap operators breached in 179 to 295 of 300, because
they choose targets without checking the fit
(`docs/references/sessions/2026-11-06-free-ga-moves.md`). A verdict at that point would
judge those moves. The following changes apply:

1. The moves are repaired before the seed-42 cross-check runs (plan, decision 12). The
   weights and GA settings stay.
1. E1 reports the walk with and without repair (plan, decision 13). The verdict in this
   ADR keeps comparing the GA with the walk with repair, because both may re-plan.

## Consequences

- E1 costs two walks per seed since the amendment. The solver needs a way to stop after the walk, so that
  E1 does not pay for a GA that cannot change its result.
- The binary search changes from the headline method to a cross-check. Its probe count
  and cost are reported from the seed-42 run.
- The GA stays in the solver until the seed-42 result exists. Its removal, if the
  condition holds, goes through ROADMAP and BACKLOG like any other change.
- MSC Irina remains the regression check. A solver without the GA must reproduce the
  FFD seed fitness of 1,587,653.1260 (full packing) and 2,311,466.1293
  (`--headroom 20`), because those values come from Phase 1.

## Alternatives considered

- **Binary search with the GA on all ten seeds.** It measures full re-planning on every
  seed, at about 14 times the cost of the walk. An estimate scaled from MSC Irina gives
  about 12 minutes per solve; this is not a measurement. The seed-42 run answers the
  question that decides the GA's future at one tenth of that cost.
- **Remove the GA now.** The MSC Irina records show small gains, but none was measured
  on the trace, where fractional GPUs and admission change the problem. The decision
  waits for that number.

## References

- `docs/references/sessions/2026-11-01-irina-with-affinity.md`
- `docs/references/sessions/2026-09-06-ga-early-stop-threshold.md`
- `docs/references/sessions/2026-11-01-admission.md`
- `docs/references/sessions/2026-11-06-free-ga-moves.md`
- `docs/plans/benchmarks/alibaba-gpu-v2023/00_plan.md`, §4 Stage 6 and §8
