# Todo — resume here

Hand-off for the next session, written 2026-10-31. Branch `24-challenge-alibaba`. The
work below is uncommitted. Delete this file once its items are done or moved into
`BACKLOG.md`.

## State at hand-off

Updated 2026-11-01, evening. Branch `24-challenge-alibaba`, committed and pushed
through `e6acca3` and its follow-up.

- X-1 is done. The FGD simulator reproduces the published seed-42 figures exactly. See
  `docs/references/sessions/2026-10-31-fgd-simulator-calibration.md`.
- K-3 is done (#26): affinity rules reach the solver, the inspector checks them, and the
  MSC Irina testdata is IR v1 with declaration-level rules. See
  `docs/references/sessions/2026-11-01-irina-with-affinity.md`.
- X-2 is done at `df5895e`: the trace adapter in `bench/alibaba/`, its tests, the
  `bench-alibaba-fetch` and `bench-alibaba-ir` targets, and
  `docs/references/sessions/2026-11-01-alibaba-trace-adapter.md`. The seed-42 IR has
  1,213 nodes and 10,866 Pods, and its GPU demand equals the logged arrivals at every
  Pod.
- X-3 is done at `e6acca3`: `gpuDevices` and `gpuMilli` in `ir-v1.md` §2 and §3.4, the
  solver, the inspector, the forge IR types, and the adapter. See
  `docs/references/sessions/2026-11-01-per-device-gpu.md`.
- Uncommitted on purpose: the temporary timing example
  `solver/examples/fitness_timing.rs`, kept for S-1.

## Next, in order

Decided 2026-11-01: finish #24 first. Performance work (S-1) waits for its own issue and
session, because the Alibaba default trace has no affinity or spread rule.

1. X-4: admission. Pods may stay unplaced, with an `unplaced:` list in the blueprint and
   `--admission in-order | free`. IR v1 has no arrival field: the adapter writes
   declarations in arrival order and `sequence.csv` states the order. Decide which one
   the solver reads before writing the `in-order` mode. An unplaced Pod must take no
   device: `gpu_devices::device_overflow_per_node` and `DeviceLedger` count every
   assigned Pod.
1. S-2: derive the objective weights on the Alibaba instance and fix them before the
   first run. Decide whether fragmentation counts milli-GPU stranded on devices; X-3
   left that term as a per-node sum.
1. X-5: one full solve of the seed-42 tuned set; measure wall clock and memory.
1. X-6: experiments E1–E5 over seeds 42–51.

Later, outside #24: open a GitHub issue for S-1, fix K-2 (topology spread keys), and P-1
(correct PAPER §7, then close #26).

## Things to know before continuing

- **The simulator lived in `/tmp` and is gone after a reboot (checked 2026-11-01).** X-2
  needed only its committed logs. To rebuild it:

    ```shell
    git clone https://github.com/hkust-adsl/kubernetes-scheduler-simulator.git fgd-sim
    git -C fgd-sim checkout 8f3d6417353c5083c6d56617f255fadd8dc306bc
    cd fgd-sim && PATH=$HOME/sdk/go1.26.1/bin:$PATH make
    uv venv --python 3.10 .venv && VIRTUAL_ENV=.venv uv pip install -r requirements.txt
    ```

    Then prepare the trace and run cells as described in the session record. The command
    file is `x1-seed42/cells_seed42.sh`. Set `GODEBUG=randseednop=0`.
- **The trace CSV files are not in the repository.** `make bench-alibaba-fetch`
  downloads them at the pinned commit into `bench/alibaba/data/` (Git ignores it).
- **Go is not on the default `PATH`.** The SDK is at `~/sdk/go1.26.1/bin`.
- **One simulator cell takes about 36 minutes and 2.4–3.2 GB of RAM.** All ten seeds
  for three policies is 30 cells, about 6 hours with three in parallel on this machine.
- **The E1 baseline already exists for seed 42.** Each policy's first rejection: FGD at
  94.89% arrived GPU demand, Best-Fit at 92.86%, Random at 42.60%.
- **The plan's open question about departures is closed.** The trace has arrivals only,
  so E1 can use arrival prefixes.
