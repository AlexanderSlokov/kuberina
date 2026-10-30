# Session record — FGD simulator calibration — 2026-10-31

**Backlog item:** X-1 · **Issue:** [#24](https://github.com/AlexanderSlokov/kuberina/issues/24)
**Plan:** `docs/plans/benchmarks/alibaba-gpu-v2023/00_plan.md`, Stage 1
**Kuberina commit:** `c15a04c`. No Kuberina code ran in this session's measurements.
**Simulator:** `hkust-adsl/kubernetes-scheduler-simulator` at `8f3d6417353c5083c6d56617f255fadd8dc306bc` (2024-07-31).
**Trace:** `alibaba/clusterdata` at `3858992ca41bf92082e6d6522ddf0bebd14e838b` (2026-10-13).
**Host:** Linux 7.0.0-38-generic x86_64, 8 vCPU, 13 GiB RAM.
**Raw output:** `docs/plans/benchmarks/alibaba-gpu-v2023/x1-seed42/`.

## Setup

The simulator was built with Go 1.26.1. Its `go.mod` declares `go 1.15`, so the legacy
`math/rand` seeding applies; `GODEBUG=randseednop=0` was also set to make that explicit.
The Python scripts ran in a virtual environment with the pinned `requirements.txt`
(Python 3.10, pandas 2.0.1).

```shell
make                                                    # in the simulator checkout
cd data && mkdir -p openb_pod_list_default \
  && cp node_yaml/openb_node_list_gpu_node.yaml openb_pod_list_default/ \
  && python pod_csv_to_yaml.py csv/openb_pod_list_default.csv
cd .. && GODEBUG=randseednop=0 bash cells_seed42.sh     # three cells, run in parallel
```

`cells_seed42.sh` holds the three lines for `openb_pod_list_default` × {01-Random,
05-BestFit, 06-FGD} × seed 42, copied from the upstream
`experiments/run_scripts/expected_run_scripts_0511.sh`. The allocation figures were
computed from each cell's `analysis_allo.csv` with the arithmetic of the upstream
`experiments/analysis/merge_alloc_discrete.py`.

## Results

GPU allocation ratio once arrived GPU demand reaches 100% and 130% of the 6,212 GPUs,
against the upstream `expected_results/analysis_allo_discrete.csv` for the same seed:

| Policy | At 100% | Published | At 130% | Published | Difference at 130% |
|---|---|---|---|---|---|
| 06-FGD | 95.14% | 95.14% | 95.29% | 95.29% | 0.00 |
| 05-BestFit | 92.94% | 92.94% | 93.13% | 93.13% | 0.00 |
| 01-Random | 86.16% | 86.17% | 87.58% | 87.26% | +0.32 |

The X-1 condition, FGD within ±0.5 points of the published seed-42 value, holds with a
difference of zero. Random differs by 0.32 points; it is the only policy that draws
random numbers while it places Pods, which is where a different Go runtime can diverge.

### Workload and first rejection

| Policy | Pods offered | Pods rejected | First rejection, arrival index | Arrived GPU demand at first rejection |
|---|---|---|---|---|
| 06-FGD | 10,866 | 2,532 | 7,885 | 94.89% |
| 05-BestFit | 10,866 | 2,676 | 7,727 | 92.86% |
| 01-Random | 10,866 | 2,690 | 3,552 | 42.60% |

Tuning grew the default list from 8,152 to 10,866 Pods. The logged sequence of Pod names
is identical across the three policies, so every policy saw the same workload in the
same order. The last column is the first E1 data point for each policy.

### Cost

| Policy | Wall clock | Peak memory |
|---|---|---|
| 05-BestFit | 2,138 s | 2.4 GB |
| 06-FGD | 2,164 s | 2.8 GB |
| 01-Random | 2,294 s | 3.2 GB |

The three cells ran in parallel. The simulator README estimates about 10 minutes per
cell on 2 vCPU; each cell here took about 36 minutes.

## Other findings

- **No departures.** `data/pod_csv_to_yaml.py` writes no deletion-time annotation (lines
  119–120 are commented out), so the benchmark contains arrivals only.
- **Rejected Pods leave the cluster.** After a failed placement, the log shows
  `[deletePod] attempt to delete a non-scheduled pod`. A rejected Pod is not retried.
