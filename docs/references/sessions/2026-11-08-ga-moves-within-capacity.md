# Session record — GA moves within capacity and the walk without repair (X-8, X-9) — 2026-11-08

**Backlog items:** X-8, X-9 · **Issue:** [#24](https://github.com/AlexanderSlokov/kuberina/issues/24)
**Plan:** `docs/plans/benchmarks/alibaba-gpu-v2023/00_plan.md`, Stage 5a
**Kuberina commit:** `7da3068`. The binaries were built from the working tree that became
this commit, and `solver/src` did not change between the build and the commit.
**Instances:** the Alibaba seed-42 instance of `2026-11-03-alibaba-scale-check.md`
(1,213 nodes, 10,866 Pods), and MSC Irina (620 nodes, 2,714 Pods) at
`solver/testdata/irina_*.yaml`, unchanged since `096177e`.
**Host:** Linux 7.0.0-42-generic x86_64, 8 vCPU, 13 GiB RAM.
**Raw output:** `docs/plans/benchmarks/alibaba-gpu-v2023/x8-x9/`. The folder `7da3068/`
holds the runs at the commit. The folder `serial-mutation/` holds one run before the
mutation ran in parallel, from a working tree that was not committed.

## What was measured

Plan decision 12 makes the GA's place, move, and swap operators choose only nodes where
the Pod fits (X-8). Decision 13 adds `--no-repair`, which walks in arrival order by best
fit alone (X-9). This session measured:

1. How often each GA move breaches capacity after the change, with the probe of
   `2026-11-06-free-ga-moves.md`.
1. The in-order solve with the GA on seed 42, against `e5f520f`.
1. The walk without repair on seed 42, against the probe's figures.
1. MSC Irina full packing with the GA, against `096177e`, and the FFD seed fitness of
   both MSC Irina configurations.

X-10, free admission from the walk, is in `7da3068`, and its seed-42 run was stopped
before it finished. A later session runs it.

## Commands

From the repository root, with the solver built by `cargo build --release` in
`solver/`:

```shell
kuberina-solver plan --infra bench/alibaba/out/seed42/infra.yaml \
    --workloads bench/alibaba/out/seed42/workloads.yaml --admission in-order --no-repair --no-ga
kuberina-solver plan --infra bench/alibaba/out/seed42/infra.yaml \
    --workloads bench/alibaba/out/seed42/workloads.yaml --admission in-order
kuberina-solver plan --infra solver/testdata/irina_infra.yaml \
    --workloads solver/testdata/irina_workloads.yaml
kuberina-solver plan --infra solver/testdata/irina_infra.yaml \
    --workloads solver/testdata/irina_workloads.yaml --headroom 20 --no-ga
uv run --with pyyaml python inspector/inspector.py \
    --infra bench/alibaba/out/seed42/infra.yaml \
    --workloads bench/alibaba/out/seed42/workloads.yaml \
    --solution kuberina_solution.yaml --output dashboard.html
cd solver && cargo run --release --example free_ga_probe -- \
    ../bench/alibaba/out/seed42/infra.yaml ../bench/alibaba/out/seed42/workloads.yaml 100
```

Wall clock and peak memory come from `/usr/bin/time -f "%e s, %M KB"`. The runs ran one
at a time. During the MSC Irina run, `make test` ran once, and the agent posted to GitHub.
The first rejection of each blueprint comes from
`docs/plans/benchmarks/alibaba-gpu-v2023/x8-x9/first_rejection.py`, which reads the
arrival order from `workloads.yaml` and the `unplaced:` list from the blueprint.

## Results

### Each move, 300 trials

The probe applies each move 300 times to the FFD seed and to the walk. Place, move, and
swap are the solver's operators from `ga_mutation.rs`. The other rows are unchanged from
`99a4b8a`:

| Move | FFD seed: breach | FFD seed: better | Walk: breach | Walk: better | `99a4b8a` breaches, FFD seed and walk |
|---|---|---|---|---|---|
| Place where it fits | 0 | 0 | 0 | 0 | 300 and 300 |
| Move where it fits | 0 | 71 | 0 | 58 | 295 and 290 |
| Swap if both fit | 0 | 97 | 0 | 18 | 179 and 249 |
| Unplace | 0 | 0 | 0 | 0 | 0 and 0 |
| Trade | 0 | 75 | 0 | 163 | 0 and 0 |
| Uniform crossover of two trades | 160 | 21 | 135 | 36 | 160 and 135 |

Uniform crossover still breaches capacity in 135 to 160 of 300 trials.

The probe also ran free admission from the walk for 100 generations. It ended at fitness
108,179,398.9360 with no breach, 8,327 Pods, and 5,883.28 GPUs (94.71%). At `99a4b8a`,
the same run ended at 109,276,509.7090, with 8,312 Pods and 5,860.46 GPUs (94.34%).
The probe took 142.14 s and 536,008 KB.

### In-order solves on seed 42

The following table compares the solves with `e5f520f`, from
`2026-11-03-alibaba-scale-check.md`:

| Quantity | `--no-repair --no-ga` | In-order with the GA, `e5f520f` | In-order with the GA, `7da3068` |
|---|---|---|---|
| Wall clock | 1.58 s | 196.42 s | 244.44 s |
| Peak memory | 40,328 KB | 450,472 KB | 548,836 KB |
| Phase 1 (walk) | 1.37 s | 14.11 s | 15.05 s |
| Phase 2 (GA) | skipped | 182.06 s, stopped at generation 200 of 1,000 | 229.13 s, stopped at generation 200 of 1,000 |
| In-order rule after Phase 2 | 0.08 s | 0.08 s | 0.07 s |
| Seed fitness | 115,627,725.9790 | 109,285,356.4890 | 109,285,356.4890 |
| Final fitness | 115,627,725.9790 | 109,285,355.4699 | 109,041,102.5884 |
| Pods placed, unplaced | 8,125, 2,741 | 8,312, 2,554 | 8,318, 2,548 |
| GPU allocation | 5,733.55 of 6,212 GPUs (92.30%) | 5,860.27 (94.34%) | 5,865.21 (94.42%) |
| First rejection, arrival index | 7,653 | 7,775 | 7,775 |
| Arrived GPU at first rejection | 92.00% | 93.40% | 93.40% |
| `metadata.feasible`, exit status | true, 0 | true, 0 | true, 0 |
| Inspector violations | 0 | 0 | 0 |

The inspector reported zero violations of every kind for both `7da3068` blueprints: no
capacity overflow on any dimension, no selector, GPU device, or affinity violation, and
no in-order violation.

The GA's best fitness before the in-order rule ran was 109,284,895.4167 at generation
190, a gain of about 461 over the walk. Under in-order admission, the GA does not change
the admitted set, so the in-order rule step after Phase 2 made the change below. Compared with the `e5f520f` blueprint, the `7da3068` blueprint
moves 295 Pods, admits 7 Pods that `e5f520f` left out, and leaves out 1 Pod that it placed.

One run before the mutation ran in parallel (`serial-mutation/`) took 327.15 s in
Phase 2 and ended at 109,139,357.2224, with 2,549 Pods unplaced and 94.38%. The parallel
mutation draws different random numbers, so the two results are different runs and are
not a repeat.

### MSC Irina

The FFD seed fitness is 1,587,653.1260 for full packing and 2,311,466.1293 with
`--headroom 20`, as at `096177e`. The full-packing solve with the GA, against
`2026-11-01-irina-with-affinity.md`:

| Quantity | `096177e` | `7da3068` |
|---|---|---|
| Generations | 330 of 1,000, early stop | 1,000 of 1,000, no early stop |
| Wall clock | 864.67 s | 2,726.98 s |
| Peak memory | 171,652 KiB | 243,272 KB |
| Final fitness | 1,587,391.1255 | 1,518,026.6505 |
| Active nodes | 540 of 620 | 527 of 620 |
| Fragmentation | 1,577,120.00 | 1,509,875.50 |
| Affinity violations (solver) | 964 | 566 |
| Topology spread penalty | 17.00 | 17.00 |
| Capacity penalty, verdict | 0, FEASIBLE | 0, FEASIBLE |

The `--headroom 20` solve with the GA was not run. The inspector was not run on the
MSC Irina blueprint.

## Findings

- **The three moves no longer breach capacity.** Place, move, and swap breached in 0 of
  300 trials from both seeds, against 179 to 300 at `99a4b8a`. Uniform crossover is
  unchanged and still breaches in about half of its trials.
- **The walk without repair matches the probe.** On seed 42, `--no-repair` rejects first
  at arrival index 7,653 with 92.00% of the cluster's GPU arrived, and ends at 92.30%,
  the figures of `2026-11-06-free-ga-moves.md`. The inspector found no in-order violation.
  The whole solve took 1.58 s, against 14.62 s for the walk with repair at `e5f520f`.
- **The GA now improves the in-order solve, by a small amount.** Its own gain over the
  walk is about 461 in fitness, against about 1 at `e5f520f`. The repacking then lets the
  in-order rule admit 6 more Pods, for 94.42% of the GPUs. The first rejection does not
  move, so this session does not change the ADR-0003 question; X-7 answers it.
- **The GA now improves MSC Irina by a large amount.** It emptied 13 more nodes and
  removed 398 of 964 affinity violations, and it did not converge within 1,000
  generations. This session did not run the inspector on that blueprint, so the solver's
  figures are not yet checked independently.
- **A generation costs about the same on MSC Irina and more on seed 42.** On MSC Irina, a
  generation took 2.73 s against 2.62 s at `096177e`; the run took longer because the GA
  kept improving. On seed 42, Phase 2 took 229.13 s against 182.06 s for the same 200
  generations, 26% more. Running the mutation serially took 327.15 s.

## Not done in this session

- The seed-42 free-admission solve from the walk (X-10 done-when).
- The MSC Irina `--headroom 20` solve with the GA, and the inspector on the MSC Irina
  full-packing blueprint.
