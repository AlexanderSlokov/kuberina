# Session record — Alibaba `cluster-trace-gpu-v2023` profile — 2026-10-30

**Issue:** [#24](https://github.com/AlexanderSlokov/kuberina/issues/24).
**Kuberina commit:** `7e5609f`. This session measured the
external trace and read the published baseline, to decide what Kuberina needs before it
can run on the trace.
**Sources:** `github.com/alibaba/clusterdata`, directory `cluster-trace-gpu-v2023/csv`,
and `github.com/hkust-adsl/kubernetes-scheduler-simulator`, both shallow-cloned from
their default branches on 2026-10-30. The upstream commit SHAs were not recorded in this
session and must be added before any figure here is cited in `PAPER.md`.

## What was measured

The node inventory (`openb_node_list_all_node.csv`) and the default pod list
(`openb_pod_list_default.csv`), with pandas:

```bash
uv run --with pandas python -I -c "<profile script>"   # columns: num_gpu, gpu_milli, gpu_spec, model
```

### Cluster

| Quantity | Value |
|---|---|
| Nodes | 1,523 |
| Nodes with GPUs | 1,213 |
| GPUs | 6,212 |
| GPU models | G2 549 · T4 404 · P100 134 · V100M16 55 · G3 39 · V100M32 30 · A10 2 · none 310 |

### Default pod list

| Quantity | Value |
|---|---|
| Pods | 8,152 |
| CPU-only pods | 1,088 |
| GPU pods | 7,064 |
| Fractional-GPU pods (`num_gpu == 1`, `gpu_milli < 1000`) | **3,078** (43.6% of GPU pods) |
| Multi-GPU pods (`num_gpu > 1`) | 75 |
| Pods with a `gpu_spec` type constraint | 0 |
| Total GPU demand | 6,086.8 GPUs against 6,212 in the cluster (98.0%) |

The default list carries no GPU-type constraints. Those appear only in the `gpuspec*`
variants, as an OR over models (`V100M16|V100M32`).

## Published baseline

The simulator repository ships its own results in
`experiments/analysis/expected_results/analysis_allo_discrete.csv`. The key indicator is
the GPU allocation ratio once arrived GPU requests reach 130% of cluster capacity
(`tune = 1.3`). For `openb_pod_list_default`, averaged over the ten seeds 42–51:

| Policy | GPU allocation at 130% |
|---|---|
| 06-FGD | 95.39% |
| 05-BestFit | 93.08% |
| 04-GpuPacking | 92.00% |
| 03-GpuClustering | 91.88% |
| 02-DotProd | 90.85% |
| 01-Random | 87.47% |

These are the upstream authors' numbers, read from their file. Reproducing them is
backlog item X-1.

## What the numbers say about Kuberina

1. **The GPU dimension is the wrong shape for this trace.** IR v1 models `gpu` as one
   scalar per node (`docs/references/ir-v1.md` §1: "fractional values are accepted but
   not meaningful to the packer"), and `ResourceVector::fits` compares sums. A pod asking
   for 0.5 GPU fits a node with eight GPUs each 60% used, because 3.2 GPUs are free in
   aggregate, yet no single device can host it. 3,078 pods are of this kind. Any
   Kuberina result on this trace is invalid until the model tracks GPUs per device.
2. **Kuberina cannot decline a pod.** `Blueprint::assignment` is a `Vec<usize>` with one
   node per pod, and `ffd_warmstart` falls back to an overflowing node when nothing fits
   (`solver/src/phase1_ffd.rs`). The benchmark offers 130% of capacity by construction,
   so every Kuberina run would end in an infeasible blueprint.
3. **The comparison is offline against online.** The simulator places pods one at a time
   in arrival order, with no knowledge of what comes next. Kuberina sees the whole set. Given
   free choice of which pods to admit at 130%, an offline planner can approach 100%
   allocation by admitting small pods first. That figure would not be comparable to
   95.39%.
