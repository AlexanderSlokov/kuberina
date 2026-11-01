# Todo — resume here

Hand-off for the next session, written 2026-10-31. Branch `24-challenge-alibaba`. The
work below is uncommitted. Delete this file once its items are done or moved into
`BACKLOG.md`.

## State at hand-off

- X-1 is done. The FGD simulator reproduces the published seed-42 figures exactly. See
  `docs/references/sessions/2026-10-31-fgd-simulator-calibration.md`. Raw output is in
  `docs/plans/benchmarks/alibaba-gpu-v2023/x1-seed42/`.
- The README is rewritten in the Kubernetes documentation style, and
  `docs/tutorials/plan-your-first-cluster.md` is new. `AGENTS.md` gained a
  *Writing documentation* section, and `CLAUDE.md` imports `AGENTS.md`.
- Two defects found while writing the tutorial are filed in `BACKLOG.md` as K-2
  (topology spread ignores `topology.kubernetes.io/zone`) and K-3 (the MSC Irina testdata
  and the inspector fail under IR v1).

## Next, in order

1. Fix K-3 first ([#26](https://github.com/AlexanderSlokov/kuberina/issues/26)). Affinity
   has no effect on any plan, `make solver-irina` and `make inspector-run` fail, and X-3
   cannot meet its done-when condition, which needs the Irina FFD fitness. Start with
   the target semantics in `docs/references/ir-v1.md` §3.1.
1. Start X-2, the trace adapter in `bench/alibaba/`. Its input is a simulator log; the
   three seed-42 logs are in `x1-seed42/*/simon.log.gz`.
1. Start X-3, the per-device GPU model. Write the IR fields into
   `docs/references/ir-v1.md` before any code.
1. Fix K-2.

## Things to know before continuing

- **The simulator lived in `/tmp`, which may not survive a reboot.** To rebuild it:

    ```shell
    git clone https://github.com/hkust-adsl/kubernetes-scheduler-simulator.git fgd-sim
    git -C fgd-sim checkout 8f3d6417353c5083c6d56617f255fadd8dc306bc
    cd fgd-sim && PATH=$HOME/sdk/go1.26.1/bin:$PATH make
    uv venv --python 3.10 .venv && VIRTUAL_ENV=.venv uv pip install -r requirements.txt
    ```

    Then prepare the trace and run cells as described in the session record. The command
    file is `x1-seed42/cells_seed42.sh`. Set `GODEBUG=randseednop=0`.
- **Go is not on the default `PATH`.** The SDK is at `~/sdk/go1.26.1/bin`.
- **One simulator cell takes about 36 minutes and 2.4–3.2 GB of RAM.** All ten seeds
  for three policies is 30 cells, about 6 hours with three in parallel on this machine.
- **The E1 baseline already exists for seed 42.** Each policy's first rejection: FGD at
  94.89% arrived GPU demand, Best-Fit at 92.86%, Random at 42.60%.
- **The plan's open question about departures is closed.** The trace has arrivals only,
  so E1 can use arrival prefixes.
