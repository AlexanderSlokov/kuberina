# TODO

Hand-off for the next session. Branch `24-challenge-alibaba`, updated 2026-11-06.
Remove an item once it is done or moved into `BACKLOG.md`.

The plan of record is `docs/plans/benchmarks/alibaba-gpu-v2023/00_plan.md`, issue
[#24](https://github.com/AlexanderSlokov/kuberina/issues/24). Its *Progress* table shows
the stage status, and `BACKLOG.md` holds the remaining tasks.

## For the Founder

These tasks belong to the Founder. Agents prepare material for them and do not decide
them:

1. **Discuss and complete
   [RFC-0001](docs/explanation/rfc/0001-chia-tang-va-lo-trinh-thay-doi.md).** It records,
   in Vietnamese, the ideas of 2026-11-06: decompose the problem as stowage planning does,
   leave the second tier's algorithm open, and plan the order of workload changes. Its
   section *Câu hỏi mở* lists the questions to settle. The RFC stays out of `ROADMAP.md`
   until this debate ends.
1. **Decide how E3 runs, before its runs start.** At `e5f520f`, free admission is feasible
   but ends at 6,799 Pods and 93.05% GPU, behind the in-order walk's 8,312 Pods and 94.34%,
   after 1,314 s and all 1,000 generations. It starts from FFD. Two options: seed it from
   the walk, which elitism keeps from ending worse, and make its place move pick only nodes
   the Pod fits. Either one changes E3's meaning.
1. **Decide on an FGD-scored walk (proposed "E2b").** At `e5f520f`, the walk reaches 94.34%
   GPU allocation at 130%, between FGD (95.29%) and Best-Fit (93.13%); first rejection at
   index 7,775 against FGD's 7,885. A walk that scores nodes by FGD's fragmentation
   measure, plus the depth-1 repair, would isolate what re-planning is worth. It is a
   post-hoc variant, so it is reported beside the pre-registered result. Open it in plan
   mode if approved.
1. **Decide whether a tie counts as the GA losing.** ADR-0003 says "at or after"; X-7
   applies this rule.
1. **Decide on three derived files and folders the repository keeps:**
    - `docs/references/PAPER.pdf` dates from 2026-07-26, and `PAPER.md` has changed since.
      No document links to the PDF. Regenerate it or remove it.
    - `research/testdata/irina_*.yaml` comes from `make research-generate-testdata`. The
      `full-pipeline` target also regenerates it, and it differs from the solver's copy.
    - `*.out` in `.gitignore` also hides six small raw files in
      `docs/plans/benchmarks/alibaba-gpu-v2023/x1-seed42/`. Add an exception to keep them
      as evidence, or leave them out.

## Done

Everything below is committed and pushed. Each item names its commit and its session
record in `docs/references/sessions/`:

| Item | What it delivered | Commit | Record |
|---|---|---|---|
| X-1 | The FGD simulator reproduces the published seed-42 figures exactly | `0e30c94` | `2026-10-31-fgd-simulator-calibration.md` |
| K-3 (#26) | Affinity rules reach the solver, the inspector checks them, MSC Irina is IR v1 | `096177e` | `2026-11-01-irina-with-affinity.md` |
| X-2 | Trace adapter `bench/alibaba/`; seed-42 IR: 1,213 nodes, 10,866 Pods, 8,075,080 milli-GPU | `df5895e` | `2026-11-01-alibaba-trace-adapter.md` |
| X-3 | `gpuDevices` and `gpuMilli` (IR v1 §2, §3.4): GPUs checked device by device | `e6acca3` | `2026-11-01-per-device-gpu.md` |
| X-4 | `arrival`, `unplaced:`, `--admission in-order \| free` (IR v1 §3.5, §4.1) | `cfee303` | `2026-11-01-admission.md` |
| X-5 | Phase timing lines; breach fix `e5f520f`; measured cost: walk 15 s, GA solve 196 s, free solve 1,314 s | `624db06`, `e5f520f` | `2026-11-03-alibaba-scale-check.md` |
| S-2 | Stranded GPU in fragmentation and the walk; weights 2,328 per Pod and 46,560 per GPU on seed 42; free-admission trade; `--no-ga` | `112be75` | `2026-11-03-objective-weights.md` |

Decisions the maintainer made, recorded in the plan, §8:

- 2026-11-01: finish #24 before the performance work (S-1). In-order admission means
  skip and continue. Arrival order is the IR field `arrival`. `kuberina-forge out`
  refuses a blueprint with unplaced Pods.
- 2026-11-02: fragmentation counts milli-GPU stranded on devices, in fitness and in the
  walk. The unplaced-Pod charge keeps its dominance formula, computed on the instance.
  Free admission gains a swap operator. E1 is one in-order walk per seed, with a
  binary-search cross-check on seed 42 only. The GA is deprecated, sunset, and removed
  if it finds no later first rejection than the walk
  ([ADR-0003](docs/explanation/adr/0003-ga-must-beat-the-walk.md)).

## Next, in order

The task rows in `BACKLOG.md`, section *External validation on the Alibaba GPU trace*,
hold the files and done-when conditions:

1. **X-6: experiments E1–E5** over seeds 42–51. The simulator needs 27 more cells for
   seeds 43–51 (see *Things to know*). E3 waits on the Founder's decision above.
1. **X-7: E1 cross-check on seed 42.** Its result decides the GA's future (ADR-0003).

## Later, outside #24

- **S-1:** open a GitHub issue, then resolve affinity targets and spread groups to
  indices once at load. The temporary timing example `solver/examples/fitness_timing.rs`
  stays uncommitted on purpose until then.
- **K-2:** topology spread ignores `topology.kubernetes.io/zone`.
- **P-1:** correct PAPER §7 with the MSC Irina results of `096177e`, then close #26.
- **The research Python parser** (`research/src/kuberina/parser.py`) does not read
  `gpuDevices`, `gpuMilli`, or `arrival`. `ir-v1.md` still lists it as an
  implementation. Either update it or say in the spec that it covers the pre-X-3
  surface only.
- **Documentation for users:** README describes shared GPUs and admission in one
  bullet each. A how-to guide for either does not exist yet (`docs/how-to/`).

## Things to know before continuing

- **The simulator lived in `/tmp` and is gone after a reboot.** The committed logs in
  `docs/plans/benchmarks/alibaba-gpu-v2023/x1-seed42/` are enough for the adapter. To
  rebuild the simulator:

    ```shell
    git clone https://github.com/hkust-adsl/kubernetes-scheduler-simulator.git fgd-sim
    git -C fgd-sim checkout 8f3d6417353c5083c6d56617f255fadd8dc306bc
    cd fgd-sim && PATH=$HOME/sdk/go1.26.1/bin:$PATH make
    uv venv --python 3.10 .venv && VIRTUAL_ENV=.venv uv pip install -r requirements.txt
    ```

    Then prepare the trace and run cells as described in
    `2026-10-31-fgd-simulator-calibration.md`. The command file is
    `x1-seed42/cells_seed42.sh`. Set `GODEBUG=randseednop=0`.
- **The trace CSV files are not in the repository.** `make bench-alibaba-fetch`
  downloads them at the pinned commit into `bench/alibaba/data/`, and Git ignores it.
- **Go is not on the default `PATH`.** The SDK is at `~/sdk/go1.26.1/bin`; run
  `PATH=$HOME/sdk/go1.26.1/bin:$PATH make test`.
- **One simulator cell takes about 36 minutes and 2.4–3.2 GB of RAM.** Ten seeds for
  three policies is 30 cells, about 6 hours with three in parallel on this machine.
- **The E1 baseline exists for seed 42.** Each policy's first rejection: FGD at 94.89%
  of arrived GPU demand, Best-Fit at 92.86%, Random at 42.60%.
- **The trace has arrivals only,** so E1 can use arrival prefixes.
- **Stopping a background solver from zsh:** `grep -m1 <pattern> <(tail -f err.txt)`
  waits for the `tail` to end in zsh, so the solver kept running. Kill by PID, and
  never `pkill -f kuberina-solver`: it matches the shell running the command.
- **MSC Irina is the regression check for every solver change.** Without
  `--admission`, the FFD seed fitness must stay 1,587,653.1260 (full packing) and
  2,311,466.1293 (`--headroom 20`). The line appears within 90 seconds of the start.
- **Run `black` only on the files you changed.** Running it over `bench/tests` reformats
  unrelated files.
