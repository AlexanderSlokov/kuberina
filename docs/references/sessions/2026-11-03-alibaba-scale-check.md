# Session record — scale check on the Alibaba seed-42 instance (X-5, partial) — 2026-11-03

**Backlog item:** X-5 · **Issue:** [#24](https://github.com/AlexanderSlokov/kuberina/issues/24)
**Plan:** `docs/plans/benchmarks/alibaba-gpu-v2023/00_plan.md`, Stage 5
**Kuberina commit:** `624db06`, which adds the phase timing lines. The cost example
`solver/examples/trace_cost.rs` is added by the commit of this record. The free-admission
run found a defect, fixed in `e5f520f`; the solves are repeated at that commit in the
next session (see *Status*).
**Instance:** seed 42, tuned default set, 1,213 nodes and 10,866 Pods. `infra.yaml`
SHA-256 `4019a82bf3f07cf552b3092f138736791bfebacb43cce77ead193f78a279ac0a`,
`workloads.yaml` SHA-256
`ea5df9ae4e4d95147991b9f3137ba47fcc8120984c3368c5252b39b9b3b598f4`, as in
`2026-11-03-objective-weights.md`.
**Host:** Linux 7.0.0-38-generic x86_64, 8 vCPU, 13 GiB RAM.
**Raw output:** `docs/plans/benchmarks/alibaba-gpu-v2023/x5-seed42/`.

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
time, with nothing else on the host.

## Results

### Whole solves

The following table lists each solve:

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

| Part | Cost |
|---|---|
| In-order walk, total | 15.844 s |
| Walk replayed with best-fit insertion only, no repair | 1.509 s; it rejects 2,741 Pods |
| Depth-1 repair, by difference | about 14.3 s, 90% of the walk |
| In-order rule, one full check of the walk's result | 0.067 s |
| One fitness evaluation | 3.676 ms |
| Of which, the stranded-GPU term | 0.796 ms (22%) |
| Of which, the device overflow check | 0.775 ms (21%) |

The replay without repair rejects 2,741 Pods, and the walk with repair leaves 2,554
unplaced. The two packings diverge after the first repair, so the difference, 187 Pods,
approximates what the repair admits.

### The inspector

The inspector checked both in-order blueprints with its own code:

| Quantity | `--no-ga` blueprint | GA blueprint |
|---|---|---|
| Wall clock, whole inspection | 23.79 s | 27.70 s |
| Peak memory | 141,976 KB | 141,740 KB |
| Capacity overflow, every dimension | 0 | 0 |
| GPU device violations | 0 | 0 |
| Placement problems | 0 | 0 |
| GPU allocation | 5,860.27 of 6,212 GPUs (94.34%) | 5,860.27 of 6,212 GPUs (94.34%) |
| In-order violations | 0 | 0 |

## Budget for Stage 6

These figures come from `624db06`, before the fix. They give a first budget for the
experiment grid, to be confirmed at `e5f520f`:

| Experiment | Configuration | One seed | Ten seeds |
|---|---|---|---|
| E1 | `in-order --no-ga` | 17 s | about 3 minutes |
| E2 | `in-order` | 213 s | about 36 minutes |
| E3 | `free` | 1,286 s, without converging | about 3.6 hours |
| X-7 | about 14 probes of Phase 1 and the GA, seed 42 only | not measured | — |

## Findings

- **Free admission wrote an infeasible blueprint.** The GA placed Pods onto full nodes:
  the capacity penalty, 1,000,000 plus 10,000 per unit of overflow, was smaller than
  the unplaced charges it saved, 46,560 per GPU. The capacity overflow reached about
  12,000,000 in the scorecard, and the verdict was INFEASIBLE with exit status 1. The
  fix, `e5f520f`, adds the charge for leaving every Pod unplaced to any breach.
  In-order admission was not affected in its result: the GA cannot change the admitted
  set there.
- **The GA does not change the in-order result on the trace.** It ran 196.47 seconds,
  92% of the solve, and lowered the fitness by about 9 parts in a billion. ADR-0003
  decides from the seed-42 cross-check (X-7).
- **The depth-1 repair is 90% of the walk.** By difference, about 14.3 of 15.8 seconds,
  for about 187 Pods admitted. The walk stays under 20 seconds, so E1 is affordable.
- **The stranded-GPU term is 22% of a fitness evaluation**, 0.796 of 3.676 ms.
- **The inspector checks a trace blueprint in under 30 seconds**, in-order rule
  included, at 142 MB peak.
- **Next to the published FGD figures for seed 42** (`2026-10-31-fgd-simulator-calibration.md`),
  the walk's GPU allocation at 130%, 94.34%, lies between FGD's 95.29% and Best-Fit's
  93.13%. Its first rejection is at arrival index 7,775; FGD's is at 7,885 and
  Best-Fit's at 7,727. These come from one seed and precede Stage 6, which reports the
  comparison.

## Status

X-5 is not done. The next session repeats the three solves at `e5f520f`, confirms that
free admission writes a feasible blueprint, and completes the budget.
