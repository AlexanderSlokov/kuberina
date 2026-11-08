# TODO

This page tells the Founder and the next agent what to do next on branch
`24-challenge-alibaba`. The branch works on
[issue #24](https://github.com/AlexanderSlokov/kuberina/issues/24), which runs Kuberina on
Alibaba's production GPU trace and reports the result beside the published figures. To use
this page, find your section and start at its first item. Remove an item when it is done or
when `BACKLOG.md` holds it. Updated 2026-11-08.

The [plan for #24](docs/plans/benchmarks/alibaba-gpu-v2023/00_plan.md) is the plan of
record. Its *Progress* table shows the status of each stage, and its section 8 lists the
Founder's decisions. `BACKLOG.md` holds the files and the done-when condition of each task.

## Terms

The following table defines the terms that this page uses:

| Term | Meaning |
|---|---|
| _Walk_ | The solver's in-order placement. It takes Pods in arrival order and gives each one the node that fits it best. |
| _Repair_ | A step of the walk. When a Pod fits nowhere, the walk moves one admitted Pod to another node to make room. |
| _In-order admission_ | A rejected Pod stays rejected, and the next Pod is tried. Set it with `--admission in-order`. |
| _Free admission_ | The solver may choose any set of Pods to place. Set it with `--admission free`. |
| _First-fit decreasing (FFD)_ | The solver's other placement, which takes large Pods first. |
| _Genetic algorithm (GA)_ | The solver's search phase, which runs after the walk or FFD. |
| _Fragmentation Gradient Descent (FGD)_ | The scheduling policy published with the Alibaba trace. It is the comparison target. |
| _E1 to E5_ | The five experiments of #24, defined in the plan, Stage 6. |
| _MSC Irina_ | Kuberina's own benchmark cluster, which has 620 nodes and 2,714 Pods. |

## For the Founder

These tasks belong to the Founder. Agents prepare material for them and do not decide them:

1. **Discuss and complete
   [RFC-0001](docs/explanation/rfc/0001-chia-tang-va-lo-trinh-thay-doi.md).** The RFC records,
   in Vietnamese, the ideas of 2026-11-06: split the problem into tiers as stowage planning
   does, leave the second tier's algorithm open, and plan the order of workload changes. Its
   section *Câu hỏi mở* lists the questions to settle. The RFC enters `ROADMAP.md` after this
   discussion.
1. **Decide on a walk scored by FGD's measure (proposed as E2b).** It would use FGD's
   fragmentation score to choose nodes, and so isolate what the repair is worth. It came after
   the first results, so it is reported beside the planned experiments. Agents open it in
   plan mode if you approve it.
1. **Confirm that a tie counts as the GA losing.**
   [ADR-0003](docs/explanation/adr/0003-ga-must-beat-the-walk.md) says that the GA loses when
   the walk's first rejection comes "at or after" the GA's boundary. X-7 applies this rule.
1. **Decide on crossover.** After X-8, uniform crossover still breaches capacity in 135 to
   160 of 300 trials, while place, move, and swap breach in none
   (`2026-11-08-ga-moves-within-capacity.md`, at `7da3068`).
1. **Settle [#28](https://github.com/AlexanderSlokov/kuberina/issues/28)**, which blocks
   [#29](https://github.com/AlexanderSlokov/kuberina/issues/29) (`--pack-memory-by`) and
   [#30](https://github.com/AlexanderSlokov/kuberina/issues/30) (`kuberina-manifest`).
1. **Decide on three sets of derived files:**
    - `docs/references/PAPER.pdf` dates from 2026-07-26, and `PAPER.md` has changed since. No
      document links to the PDF. Regenerate it or remove it.
    - `research/testdata/irina_*.yaml` comes from `make research-generate-testdata`. The
      `full-pipeline` target regenerates it too, and it differs from the solver's copy. Keep
      it or generate it on demand.
    - The `*.out` rule in `.gitignore` also hides six small raw files in
      `docs/plans/benchmarks/alibaba-gpu-v2023/x1-seed42/`. Add an exception to keep them as
      evidence, or leave them out.

## For the next agent

Work through these tasks in order. E1 and X-7 no longer wait on anything:

1. **X-10:** the code is in `7da3068`. Run the seed-42 free-admission solve from the walk,
   run the inspector on it, and record it. The run takes about 20 to 25 minutes.
1. **Finish the checks of `2026-11-08-ga-moves-within-capacity.md`:** the inspector on the
   MSC Irina full-packing blueprint, and the MSC Irina `--headroom 20` solve with the GA.
1. **Replace the E3 row** in the plan, section 6, with the time of the X-10 run.
1. **X-6:** run E1 to E5 over seeds 42 to 51. The simulator needs 27 more cells for seeds 43
   to 51.
1. **X-7:** run the E1 cross-check on seed 42. Its result decides the GA's future under
   ADR-0003.

## Later, outside #24

The following tasks wait until #24 ends:

- **S-1:** open a GitHub issue, then resolve affinity targets and spread groups to indices
  once at load. The example `solver/examples/fitness_timing.rs` measures the cost.
- **K-2:** make topology spread read `topology.kubernetes.io/zone`, which it ignores.
- **P-1:** correct PAPER section 7 with the MSC Irina results of `096177e`, then close
  [issue #26](https://github.com/AlexanderSlokov/kuberina/issues/26).
- **Research parser:** `research/src/kuberina/parser.py` does not read `gpuDevices`,
  `gpuMilli`, or `arrival`, and `docs/references/ir-v1.md` still lists it as an
  implementation. Update the parser, or state in the specification that it covers the
  surface before X-3.
- **User documentation:** `README.md` describes shared GPUs and admission in one bullet
  each. Write a how-to guide for each in `docs/how-to/`.

## Before you continue

The following facts save time on this branch.

### Run the tests

Go is not on the default `PATH`. To run every test, use this command:

```shell
PATH=$HOME/sdk/go1.26.1/bin:$PATH make test
```

Run `black` only on the files you changed. Running it over `bench/tests` reformats unrelated
files.

### Check the solver for regressions

MSC Irina is the regression check for every solver change. Without `--admission`, the FFD
seed fitness stays 1,587,653.1260 for full packing and 2,311,466.1293 with `--headroom 20`.
The line appears within 90 seconds of the start.

### Stop a background solver

Stop a background solver by its process ID. The command `pkill -f kuberina-solver` also
matches the shell that runs it, and it kills that shell. In `zsh`, the pattern
`grep -m1 <pattern> <(tail -f <log>)` waits for `tail` to end, so the solver keeps running.

### Get the trace

The trace CSV files are not in the repository. The command `make bench-alibaba-fetch`
downloads them at the pinned commit into `bench/alibaba/data/`, which Git ignores. The
experiments use arrivals only, because the simulator's script drops the trace's
`deletion_time` column (`2026-10-31-fgd-simulator-calibration.md`).

### Rebuild the simulator

The FGD simulator lived in `/tmp`, so it is gone after a reboot. The logs in
`docs/plans/benchmarks/alibaba-gpu-v2023/x1-seed42/` are enough for the adapter. X-6 needs
the simulator, so rebuild it in the repository root, where Git ignores `fgd-sim/`. These
commands come from X-1, and they have not been run again since:

```shell
git clone https://github.com/hkust-adsl/kubernetes-scheduler-simulator.git fgd-sim
git -C fgd-sim checkout 8f3d6417353c5083c6d56617f255fadd8dc306bc
cd fgd-sim && PATH=$HOME/sdk/go1.26.1/bin:$PATH make
uv venv --python 3.10 .venv && VIRTUAL_ENV=.venv uv pip install -r requirements.txt
```

To prepare the trace and run cells, follow `2026-10-31-fgd-simulator-calibration.md`. The
command file is `docs/plans/benchmarks/alibaba-gpu-v2023/x1-seed42/cells_seed42.sh`. Set
`GODEBUG=randseednop=0`. One cell takes about 36 minutes and 2.4 to 3.2 GB of RAM. Ten seeds
for three policies is 30 cells, about 6 hours with three in parallel on this machine.

## Completed

The following table lists finished work. Each row names the commit that delivered or measured
it, and its session record in `docs/references/sessions/`:

| Item | Result | Commit | Record |
|---|---|---|---|
| X-1 | The FGD simulator reproduces the published seed-42 figures | `0e30c94` | `2026-10-31-fgd-simulator-calibration.md` |
| K-3 | Affinity rules reach the solver, the inspector checks them, and MSC Irina is IR v1 | `096177e` | `2026-11-01-irina-with-affinity.md` |
| X-2 | The trace adapter `bench/alibaba/` writes seed 42 as IR: 1,213 nodes, 10,866 Pods, 8,075,080 milli-GPU | `df5895e` | `2026-11-01-alibaba-trace-adapter.md` |
| X-3 | `gpuDevices` and `gpuMilli` check GPUs device by device | `e6acca3` | `2026-11-01-per-device-gpu.md` |
| X-4 | `arrival`, `unplaced:`, and `--admission in-order` or `free` | `cfee303` | `2026-11-01-admission.md` |
| S-2 | Stranded GPU counts in fragmentation and the walk; seed-42 weights are 2,328 per Pod and 46,560 per GPU; free admission has a trade move; `--no-ga` exists | `112be75` | `2026-11-03-objective-weights.md` |
| X-5 | One seed-42 solve costs 14.62 s for the walk, 196.42 s with the GA, and 1,313.81 s under free admission | `e5f520f` | `2026-11-03-alibaba-scale-check.md` |
| GA probe | Place, move, and swap breach capacity on a nearly full cluster; the walk's lead over Best-Fit comes from its repair | `99a4b8a` | `2026-11-06-free-ga-moves.md` |
| X-8, X-9 | GA moves stay within capacity; `--no-repair` reproduces 7,653 and 92.30% on seed 42; MSC Irina ends at 527 active nodes | `7da3068` | `2026-11-08-ga-moves-within-capacity.md` |
