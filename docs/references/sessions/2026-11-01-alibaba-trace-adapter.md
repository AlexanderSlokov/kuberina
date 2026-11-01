# Session record — Alibaba trace adapter — 2026-11-01

**Backlog item:** X-2 · **Issue:** [#24](https://github.com/AlexanderSlokov/kuberina/issues/24)
**Plan:** `docs/plans/benchmarks/alibaba-gpu-v2023/00_plan.md`, Stage 2
**Kuberina commit:** `df5895e`, which adds the `bench/alibaba/` adapter. The measurements
ran on its working tree before the commit; the adapter code is identical.
**Trace:** `alibaba/clusterdata` at `3858992ca41bf92082e6d6522ddf0bebd14e838b`, the
commit of the X-1 calibration.
**Simulator logs:** `docs/plans/benchmarks/alibaba-gpu-v2023/x1-seed42/*/simon.log.gz`,
from `2026-10-31-fgd-simulator-calibration.md`.
**Host:** Linux 7.0.0-38-generic x86_64, 8 vCPU, 13 GiB RAM.

## What was measured

The trace adapter (`bench/alibaba/`) converts the trace CSV files and the three seed-42
simulator logs into Kuberina IR. The X-2 condition is that the GPU demand written to
`workloads.yaml` equals the GPU demand the simulator logged as arrived. The adapter
checks it in two ways:

- Per arrival: the trace demand, summed in log order, equals the `Arrived GPU Milli`
  value of the `[Alloc]` line after each of the 10,866 Pods.
- Over the written file: `workloads.yaml`, read back with PyYAML, sums to the last
  logged value.

## Commands

```shell
make bench-alibaba-fetch   # downloads the two CSV files at the pinned trace commit
make bench-alibaba-ir      # runs python -m alibaba.adapter on the three seed-42 logs
```

The adapter printed:

```console
nodes: 1213
pods: 10866
GPU demand written and logged: 8075080 milli-GPU
wrote alibaba/out/seed42/infra.yaml, alibaba/out/seed42/workloads.yaml, alibaba/out/seed42/sequence.csv
```

## Results

The following table compares the emitted instance with the simulator's own figures:

| Quantity | Emitted by the adapter | Logged by the simulator |
|---|---|---|
| Nodes | 1,213 | 1,213 GPU nodes, 6,212 GPUs |
| Pods, in arrival order | 10,866 | 10,866 (`Num of Pods after Tuning`) |
| Original trace rows among them | 8,152, each once | 8,152 (`Num of Pods before Tuning`) |
| Tuned clones (`-tuned-<i>`) | 2,714 | — |
| GPU demand of the original rows | 6,086,800 milli-GPU | 6,086,800 (`milli gpu request`) |
| GPU demand of all Pods | 8,075,080 milli-GPU | 8,075,080 (last `Arrived GPU Milli`) |
| GPU demand as a share of capacity | 129.99% | target 130% (`1.30 * 6212000`) |

The X-2 condition holds: the emitted demand equals the logged demand at every arrival
and in total.

The Pod mix of the emitted instance is 1,428 CPU-only Pods, 4,137 fractional-GPU Pods
(under 1,000 milli-GPU), 5,207 one-GPU Pods, and 94 multi-GPU Pods.

The per-policy columns of `sequence.csv` reproduce the rejection counts of the
calibration record: FGD 2,532, Best-Fit 2,676, Random 2,690.

### Readers of the emitted files

Both IR readers accept the files:

| Reader | Result |
|---|---|
| `kuberina-solver plan` (release build) | `Loaded 1213 nodes, 0 daemonsets, 10866 pods, 0 groups`. The run was stopped after 40 seconds. |
| `inspector/ir_workloads.py`, `unroll_workloads` | 10,866 Pods. The first identity is `paib-gpu/openb-pod-0255-0`, the same as the `pod` column of `sequence.csv`. |

No plan was computed. Until X-3 and X-4 land, a plan of this instance is not meaningful:
GPU is one scalar per node, and every Pod must be placed although demand is 130% of
capacity.

### Cost and reproducibility

One adapter run took 11.5 seconds of wall clock and 144 MB peak resident memory
(`/usr/bin/time -v`). Two runs, one from scratch directories and one through the `make`
targets, wrote byte-identical files:

| File | SHA-256 |
|---|---|
| `openb_node_list_gpu_node.csv` (input) | `2beca64b4d3dfa342036a34b56a495c6cef9225db836c81f541282cb1df320b5` |
| `openb_pod_list_default.csv` (input) | `1ee7ed79c27a3b0861cda8ddba86a004c6aba904caafa329a76ae93ca63834a8` |
| `infra.yaml` | `fb09a46e63093cdc7195a9579a638a8b25bff6302042d029155c0277b7c08594` |
| `workloads.yaml` | `57e89f6278d44b3d2f81ad5875b42e1cf5dc9f44403f9cee3723f61b7cfa2ede` |
| `sequence.csv` | `e35a1139d67b943b72d490dae64f7f45f64ad3f0220f779b78623b0a9360d095` |

The emitted files are not committed. They are regenerated from the pinned trace and the
committed logs.

## Findings for later stages

- **IR v1 has no arrival field.** The adapter writes declarations in arrival order and
  states the order per Pod in `sequence.csv`. X-4 (`--admission in-order`) has to
  decide how the solver learns the order.
- **The simulator prefixes node names with its own index.** The log writes
  `0000-openb-node-0260` for trace node `openb-node-0260`. The prefix is not the trace
  number: 1,212 of the 1,213 logged prefixes differ from the node's own number. The
  adapter strips it.
- **The `[Alloc]` line follows every Pod, placed or rejected.** Its `Arrived GPU Milli`
  value includes a rejected Pod's demand: at the first FGD rejection, index 7,885, the
  arrived total exceeds the used total by exactly that Pod's 1,000 milli-GPU.
- **X-3 changes the adapter.** A fractional request is written as `gpu: 0.81` until IR
  v1 gains per-device fields.
