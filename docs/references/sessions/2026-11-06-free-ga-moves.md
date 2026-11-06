# Session record — how often each GA move survives on the Alibaba seed-42 instance — 2026-11-06

**Issue:** [#24](https://github.com/AlexanderSlokov/kuberina/issues/24)
**Plan:** `docs/plans/benchmarks/alibaba-gpu-v2023/00_plan.md`, Stage 6
**Kuberina commit:** `99a4b8a`, with `solver/examples/free_ga_probe.rs` in the working tree.
`solver/src` is identical to `e5f520f`.
**Instance:** the seed-42 instance of `2026-11-03-alibaba-scale-check.md`.
**Host:** Linux 7.0.0-42-generic x86_64, 8 vCPU, 13 GiB RAM.
**Raw output:** `docs/plans/benchmarks/alibaba-gpu-v2023/free-ga-probe/seed42-100-generations.txt`.

## What was measured

Free admission ends behind the in-order walk on seed 42, although every in-order blueprint
is also a valid free-admission blueprint (`2026-11-03-alibaba-scale-check.md`). This session
looks for the cause. It measured:

1. Whether the objective ranks the two blueprints correctly.
1. How often each GA move breaches capacity or improves the fitness, applied alone to the
   FFD seed and to the walk. A breach costs the whole child, because `e5f520f` adds the
   charge for leaving every Pod unplaced to any breach.
1. What the walk reaches without its depth-1 repair, the rule an online policy has.
1. What 100 generations of free admission reach when the GA starts from the walk.

## Command

From `solver/`:

```shell
cargo run --release --example free_ga_probe -- \
    ../bench/alibaba/out/seed42/infra.yaml ../bench/alibaba/out/seed42/workloads.yaml 100
```

The whole run took 174.62 s, with a peak of 438,300 KB. The example rebuilds each move from
the rule in `phase2_ga.rs`, because the GA's mutation is private, and applies it 300 times.

## Results

### The objective

The unplaced charge, recomputed by hand from the Pod and GPU counts, gives 109,067,266 for
the walk and 116,304,086 for free admission. These match the solver's scorecards to the
rounding of the GPU figures. A lower fitness goes with more placed Pods and more placed GPUs,
so the objective ranks the two blueprints correctly.

### One move at a time

The following table lists 300 trials of each move. A gain below 0.05 prints as 0.0, so a
"better" count beside a best gain of 0.0 means a gain too small to matter:

| Move | FFD seed: breach | FFD seed: better | FFD seed: best gain | Walk: breach | Walk: better | Walk: best gain |
|---|---|---|---|---|---|---|
| Place an unplaced Pod on a random eligible node | 300 | 0 | 0.0 | 300 | 0 | 0.0 |
| Move a placed Pod to a random eligible node | 295 | 5 | 0.0 | 290 | 10 | 0.0 |
| Swap two placed Pods | 179 | 97 | 0.0 | 249 | 18 | 0.0 |
| Unplace a Pod | 0 | 0 | 0.0 | 0 | 0 | 0.0 |
| Trade one GPU Pod for unplaced Pods | 0 | 75 | 195,459.7 | 0 | 163 | 92,677.0 |
| Uniform crossover of two traded parents | 160 | 21 | 146,614.1 | 135 | 36 | 51,686.2 |

At both starting points, no unplaced Pod fits on any node as the packing stands: 0 of 4,110
at the FFD seed and 0 of 2,554 at the walk.

### Placement rules on one seed

The following table compares the walk with and without its repair, beside the published
seed-42 figures in `2026-10-31-fgd-simulator-calibration.md`:

| Policy | First rejection, arrival index | Arrived GPU at first rejection | GPU allocation at 130% |
|---|---|---|---|
| FGD | 7,885 | 94.89% | 95.29% |
| Walk with depth-1 repair | 7,775 | 93.40% | 94.34% |
| Best-Fit | 7,727 | 92.86% | 93.13% |
| Walk without repair | 7,653 | 92.00% | 92.30% |

The FFD seed, which sees every Pod before it starts, places 6,756 Pods and 5,753.83 GPUs
(92.62%).

### Free admission from the walk

The GA, with the settings `select_ga_config` picks above 500 Pods and the walk as its seed,
ran 100 generations. Its fitness fell from 109,285,356.4890 to 109,276,509.7090, with no
breach, 8,312 Pods, and 5,860.46 GPUs (94.34%).

## Findings

- **The objective is not the cause.** It ranks the blueprints correctly.
- **The place move cannot succeed from either seed.** It breached capacity in 300 of 300
  trials, and no unplaced Pod fits anywhere as the packing stands. The move and swap
  operators also pick targets without checking the fit, and they breached in 60% to 98% of
  trials. These blind moves share a child, so one breach discards the others beside it,
  including a good trade.
- **The trade is the only move with real gains,** and the fitness gain in the full free run
  is consistent with trades carrying the search. This session did not attribute the gain to
  a move directly.
- **The first generations are likely wasted, by inference.** `init_population` applies an
  expected 16 blind moves to every individual except the seed, so nearly all of them start
  in breach. The full free run shows no gain until after generation 90. This session did not
  count the breached individuals.
- **The same blind moves run under in-order admission,** where the GA gains about 9 parts in
  a billion. ADR-0003 judges the GA on the trace, so its verdict would judge these operators.
  This session did not measure how much of that small gain comes from them.
- **Starting from the walk removes the inversion and adds almost nothing.** Free admission
  then ends ahead of the walk by 8,846.78 in fitness and 0.19 GPU in 100 generations. It is
  still a short run.
- **The repair supplies the walk's lead over Best-Fit on this seed.** With repair, the walk
  is ahead of Best-Fit. Without it, the walk is behind on both first rejection and allocation.
  An online policy cannot move a running Pod, so E1 as planned compares two different rules.
  One seed; this session did not find why the insertion alone trails Best-Fit.

## Decisions that followed

The maintainer settled three decisions on 2026-11-07, recorded in the plan, §8, decisions
12 to 14: the moves are repaired, E1 reports the walk with and without repair, and E3 starts
from the walk.
