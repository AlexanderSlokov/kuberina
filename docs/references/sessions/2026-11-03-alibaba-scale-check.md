# Session record — scale check on the Alibaba seed-42 instance (X-5) — 2026-11-03 and 2026-11-06

**Backlog item:** X-5 · **Issue:** [#24](https://github.com/AlexanderSlokov/kuberina/issues/24)
**Plan:** `docs/plans/benchmarks/alibaba-gpu-v2023/00_plan.md`, Stage 5
**Kuberina commits:** The first round, on 2026-11-03, ran at `624db06`, which adds the
phase timing lines. Its free-admission run found a defect, fixed in `e5f520f`. The second
round, on 2026-11-06, ran on `8865a43`, whose `solver/src` is identical to `e5f520f`, and
it is the round to cite. `8865a43` adds the cost example `solver/examples/trace_cost.rs`.
**Instance:** seed 42, tuned default set, 1,213 nodes and 10,866 Pods. `infra.yaml`
SHA-256 `4019a82bf3f07cf552b3092f138736791bfebacb43cce77ead193f78a279ac0a`,
`workloads.yaml` SHA-256
`ea5df9ae4e4d95147991b9f3137ba47fcc8120984c3368c5252b39b9b3b598f4`, as in
`2026-11-03-objective-weights.md`.
**Host:** 8 vCPU, 13 GiB RAM, Linux x86_64. Kernel 7.0.0-38-generic in the first round,
7.0.0-42-generic in the second.
**Raw output:** `docs/plans/benchmarks/alibaba-gpu-v2023/x5-seed42/`. The folder
`e5f520f/` holds the second round. The other folders hold the first.

## What was measured

Stage 5 asks what one Kuberina solve costs on the trace, before the experiment grid is
budgeted. This session measured:

1. One in-order solve without the GA (`--no-ga`), the configuration E1 uses
   (ADR-0003).
1. One in-order solve with the GA, the configuration E2 uses.
1. One free-admission solve, the configuration E3 uses.
1. The parts that the backlog named as possibly not scaling: the depth-1 repair in the
   walk, the in-order rule check, the stranded-GPU decode in each fitness evaluation,
   and the inspector's in-order check.

## Commands

From the repository root, with the solver built by `cargo build --release` in
`solver/`:

```shell
kuberina-solver plan --infra bench/alibaba/out/seed42/infra.yaml \
    --workloads bench/alibaba/out/seed42/workloads.yaml --admission in-order --no-ga
kuberina-solver plan --infra bench/alibaba/out/seed42/infra.yaml \
    --workloads bench/alibaba/out/seed42/workloads.yaml --admission in-order
kuberina-solver plan --infra bench/alibaba/out/seed42/infra.yaml \
    --workloads bench/alibaba/out/seed42/workloads.yaml --admission free
uv run --with pyyaml python inspector/inspector.py \
    --infra bench/alibaba/out/seed42/infra.yaml \
    --workloads bench/alibaba/out/seed42/workloads.yaml \
    --solution kuberina_solution.yaml --output dashboard.html
cd solver && cargo run --release --example trace_cost -- \
    ../bench/alibaba/out/seed42/infra.yaml ../bench/alibaba/out/seed42/workloads.yaml
```

Wall clock and peak memory come from `/usr/bin/time -v`. The solver runs ran one at a
time, with no other benchmark on the host. In the second round, the desktop session and
an editor stayed open, and light file reads and web requests ran during the free solve.

## Results

### Whole solves at `e5f520f`

The following table lists each solve of the second round:

| Quantity | `in-order --no-ga` | `in-order` | `free` |
|---|---|---|---|
| Wall clock | 14.62 s | 196.42 s | 1,313.81 s |
| Peak memory | 40,080 KB | 450,472 KB | 439,656 KB |
| Load | 0.14 s | 0.12 s | 0.12 s |
| Phase 1 (walk or FFD) | 14.40 s | 14.11 s | 0.09 s |
| Phase 2 (GA) | skipped | 182.06 s, stopped at generation 200 of 1,000 | 1,313.53 s, no early stop |
| In-order rule after Phase 2 | 0.06 s | 0.08 s | — |
| Seed fitness | 109,285,356.4890 | 109,285,356.4890 | 117,862,657.8318 |
| Final fitness | 109,285,356.4890 | 109,285,355.4699 | 116,521,226.9378 |
| Pods placed, unplaced | 8,312, 2,554 | 8,312, 2,554 | 6,799, 4,067 |
| GPU allocation | 5,860.27 of 6,212 GPUs (94.34%) | 5,860.27 of 6,212 GPUs (94.34%) | 5,780.49 of 6,212 GPUs (93.05%) |
| Active nodes | 1,213 of 1,213 | 1,213 of 1,213 | 1,213 of 1,213 |
| `metadata.feasible`, exit status | true, 0 | true, 0 | true, 0 |

A comparison of the blueprints, parsed by a script, gives these results:

- The `--no-ga` blueprint is identical, Pod for Pod, to the `624db06` one.
- The GA moved 50 Pods from the walk's placement and kept the admitted set. It lowered
  the fitness by 1.0191, about 9 parts in a billion. Its blueprint differs from the
  `624db06` one, which moved 43 Pods.
- A second GA solve at the same commit wrote a byte-identical blueprint with the same
  final fitness, so the GA repeats at one commit on this host. The difference from
  `624db06` therefore most likely comes from `e5f520f`, which changes how breaching
  offspring score. Only the wall clock varied: 217.53 s against 196.42 s, with file edits
  running beside the second solve.
- Free admission and in-order admission both place 5,324 Pods. Free admission alone
  places 1,475, and in-order admission alone places 2,988.

### Whole solves at `624db06`

The following table lists each solve of the first round. Its free-admission column shows
the defect that `e5f520f` fixes:

| Quantity | `in-order --no-ga` | `in-order` | `free` |
|---|---|---|---|
| Wall clock | 17.09 s | 213.39 s | 1,286.05 s |
| Peak memory | 40,376 KB | 454,296 KB | 437,984 KB |
| Load | 0.16 s | 0.16 s | 0.13 s |
| Phase 1 (walk or FFD) | 16.84 s | 16.61 s | 0.10 s |
| Phase 2 (GA) | skipped | 196.47 s, stopped at generation 200 of 1,000 | 1,285.76 s, the full 1,000 generations |
| In-order rule after Phase 2 | 0.07 s | 0.07 s | — |
| Seed fitness | 109,285,356.4890 | 109,285,356.4890 | 117,862,657.8318 |
| Final fitness | 109,285,356.4890 | 109,285,355.4720 | 85,207,878.6684 |
| Pods placed, unplaced | 8,312, 2,554 | 8,312, 2,554 | 8,046, 2,820 |
| GPU allocation | 5,860.27 of 6,212 GPUs (94.34%) | 5,860.27 of 6,212 GPUs (94.34%) | 6,649.25 of 6,212 GPUs (107.04%) |
| Active nodes | 1,213 of 1,213 | 1,213 of 1,213 | 1,213 of 1,213 |
| `metadata.feasible` | true | true | **false**, exit status 1 |

The GA stopped because no gain above 0.01% appeared in 200 generations. It lowered the
fitness by 1.017, about 9 parts in a billion. It moved 43 Pods to other nodes and left
the admitted set and the GPU allocation unchanged.

### Parts of the cost

`trace_cost` measured the parts on the same instance:

| Part | `e5f520f` | `624db06` |
|---|---|---|
| In-order walk, total | 15.233 s | 15.844 s |
| Walk replayed with best-fit insertion only, no repair | 1.453 s; it rejects 2,741 Pods | 1.509 s; it rejects 2,741 Pods |
| Depth-1 repair, by difference | about 13.8 s, 90% of the walk | about 14.3 s, 90% of the walk |
| In-order rule, one full check of the walk's result | 0.068 s | 0.067 s |
| One fitness evaluation | 3.568 ms | 3.676 ms |
| Of which, the stranded-GPU term | 0.787 ms (22%) | 0.796 ms (22%) |
| Of which, the device overflow check | 0.765 ms (21%) | 0.775 ms (21%) |

The replay without repair rejects 2,741 Pods, and the walk with repair leaves 2,554
unplaced. The two packings diverge after the first repair, so the difference, 187 Pods,
approximates what the repair admits.

### The inspector

The inspector checked each blueprint of the second round with its own code. It checks
the in-order rule only on in-order blueprints:

| Quantity | `--no-ga` blueprint | GA blueprint | Free blueprint |
|---|---|---|---|
| Wall clock, whole inspection | 23.11 s | 22.10 s | 10.02 s |
| Peak memory | 141,400 KB | 141,688 KB | 141,004 KB |
| Capacity overflow, every dimension | 0 | 0 | 0 |
| GPU device violations | 0 | 0 | 0 |
| Placement problems | 0 | 0 | 0 |
| GPU allocation | 5,860.27 of 6,212 GPUs (94.34%) | 5,860.27 of 6,212 GPUs (94.34%) | 5,780.49 of 6,212 GPUs (93.05%) |
| In-order violations | 0 | 0 | not checked |

In the first round, the inspector took 23.79 s and 27.70 s on the two in-order
blueprints, with the same zero counts.

## Budget for Stage 6

These figures come from the second round. Ten-seed figures multiply the seed-42 cost, and
other seeds may differ. They exclude the inspector, which adds about 23 seconds per
in-order solve and 10 seconds per free solve:

| Experiment | Configuration | One seed | Ten seeds |
|---|---|---|---|
| E1 | `in-order --no-ga` | 15 s | about 2.5 minutes |
| E2 | `in-order` | 196 s | about 33 minutes |
| E3 | `free` | 1,314 s, no early stop | about 3.7 hours |
| X-7 | about 14 probes of Phase 1 and the GA, seed 42 only | not measured | — |

## Findings

- **The fix holds: free admission writes a feasible blueprint.** At `624db06`, the GA
  placed Pods onto full nodes: the capacity penalty, 1,000,000 plus 10,000 per unit of
  overflow, was smaller than the unplaced charges it saved, 46,560 per GPU. The capacity
  overflow reached about 12,000,000 in the scorecard, the GPU allocation reached 107.04%,
  and the verdict was INFEASIBLE with exit status 1. `e5f520f` adds the charge for leaving
  every Pod unplaced to any breach. The second round is `feasible: true` with exit status
  0, and the inspector reports no violation. The in-order walk did not change: its
  blueprint is identical at both commits.
- **Free admission ends worse than the in-order walk.** Its final fitness, 116,521,226.9378,
  is 6.62% above the walk's 109,285,356.4890, and it places 1,513 fewer Pods and 79.78 fewer
  GPUs. Every in-order blueprint is also a valid free-admission blueprint, so this gap
  measures the search and its FFD seed. The GA gained 1.14% from that seed in 1,313.53
  seconds. As configured, E3 therefore reports how far the GA climbs from FFD. It does not
  report the best that free admission can reach. A walk-seeded free solve could not end
  worse than the walk under elitism; that variant has not been run, and it is a decision
  for the maintainer before E3.
- **The GA does not change the in-order result on the trace.** It ran 182.06 seconds, 93%
  of the solve, and lowered the fitness by about 9 parts in a billion, while moving 50
  Pods. ADR-0003 decides from the seed-42 cross-check (X-7).
- **The depth-1 repair is 90% of the walk.** By difference, about 13.8 of 15.2 seconds,
  for about 187 Pods admitted. The walk stays under 20 seconds, so E1 is affordable.
- **The stranded-GPU term is 22% of a fitness evaluation**, 0.787 of 3.568 ms.
- **The inspector checks a trace blueprint in under 30 seconds**, in-order rule
  included, at 142 MB peak.
- **Next to the published FGD figures for seed 42** (`2026-10-31-fgd-simulator-calibration.md`),
  the walk's GPU allocation at 130%, 94.34%, lies between FGD's 95.29% and Best-Fit's
  93.13%. Its first rejection is at arrival index 7,775; FGD's is at 7,885 and
  Best-Fit's at 7,727. These come from one seed and precede Stage 6, which reports the
  comparison.

## Status

X-5 is done. Stage 6 can use the budget above.
