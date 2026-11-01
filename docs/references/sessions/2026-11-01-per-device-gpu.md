# Session record — per-device GPU model — 2026-11-01

**Backlog item:** X-3 · **Issue:** [#24](https://github.com/AlexanderSlokov/kuberina/issues/24)
**Plan:** `docs/plans/benchmarks/alibaba-gpu-v2023/00_plan.md`, Stage 3
**Kuberina commit:** `e6acca3`, which adds the X-3 changes. The measurements ran on its
working tree before the commit; the code is identical.
**Compared against:** `096177e`, recorded in `2026-11-01-irina-with-affinity.md`.
**Host:** Linux 7.0.0-38-generic x86_64, 8 vCPU, 13 GiB RAM.

## What was measured

X-3 adds the per-device GPU model of IR v1 §3.4: `gpuDevices` on a node, and
`gpuMilli` on a declaration. It is done when three conditions hold:

1. The solver rejects a half-GPU Pod on a node whose eight devices are each 60% used.
1. The inspector rejects the same placement with its own implementation.
1. The MSC Irina FFD seed fitness is unchanged to the digit.

The fixture for the first two conditions is `solver/testdata/gpu_devices_infra.yaml`
and `solver/testdata/gpu_devices_workloads.yaml`. It has one node with eight devices,
eight Pods of 600 milli-GPU, and one Pod of 500 milli-GPU. The GPU sum fits: 4.8 + 0.5
of 8 GPUs. No single device has 500 milli-GPU free.

## Commands

```shell
cd solver && cargo build --release
kuberina-solver plan --infra testdata/gpu_devices_infra.yaml \
    --workloads testdata/gpu_devices_workloads.yaml
uv run --with pyyaml python inspector/inspector.py \
    --infra solver/testdata/gpu_devices_infra.yaml \
    --workloads solver/testdata/gpu_devices_workloads.yaml \
    --solution kuberina_solution.yaml --output dashboard.html
kuberina-solver plan --infra testdata/irina_infra.yaml --workloads testdata/irina_workloads.yaml
kuberina-solver plan --infra testdata/irina_infra.yaml --workloads testdata/irina_workloads.yaml --headroom 20
make bench-alibaba-ir
```

The two MSC Irina runs were stopped after the FFD seed line, 90 seconds in.

## Results

### The half-GPU fixture

The solver reported:

```console
═══ ❌ INFEASIBLE — DO NOT APPLY ═══
  This assignment exceeds real node capacity. It is not a plan.
  Capacity exceeded, by dimension:
    gpu                  0.50
```

It exited with status 1 and wrote a blueprint marked `feasible: false`. The inspector,
given that blueprint, reported:

```console
Capacity Overflow (GPU): 0.00
GPU Device Violations: 1
  gpu-node-01
```

The two outputs agree: the per-node GPU sum is not exceeded, and the device check fails.

### MSC Irina

MSC Irina declares no `gpuDevices`, so the per-device code does not run on it. The FFD
seed fitness, compared with the session record of `096177e`:

| Configuration | `096177e` | This session |
|---|---|---|
| Full packing | 1,587,653.1260 | 1,587,653.1260 |
| `--headroom 20` | 2,311,466.1293 | 2,311,466.1293 |

Both values are identical to the digit.

### The Alibaba seed-42 instance

The adapter now writes `gpuDevices` on each node and `gpuMilli` on each Pod that needs
part of a device. The regenerated instance, compared with
`2026-11-01-alibaba-trace-adapter.md`:

| Quantity | Before X-3 | After X-3 |
|---|---|---|
| GPU demand, by the adapter's read-back | 8,075,080 milli-GPU | 8,075,080 milli-GPU |
| GPU demand, by the inspector's reader | — | 8,075,080 milli-GPU |
| Pods with `gpuMilli` | — | 4,137 |
| `infra.yaml` SHA-256 | `fb09a46e…7c08594` | `4019a82bf3f07cf552b3092f138736791bfebacb43cce77ead193f78a279ac0a` |
| `workloads.yaml` SHA-256 | `57e89f62…2ede` | `ad48f3b39ee80be6a2cb763a5837d095c5bee73f1d107e7d24aaf92f089bffd1` |
| `sequence.csv` SHA-256 | `e35a1139…d095` | unchanged |

The solver loads the instance and accepts its device inputs: `Loaded 1213 nodes, 0
daemonsets, 10866 pods, 0 groups`. No plan was computed; X-4 and S-2 come first.

## Findings

- **Phase 0 dropped the device count.** `pre_deduct_daemonsets` rebuilt every node
  with `gpu_devices: None`, so the first solver run of the fixture reported
  `FEASIBLE`. The integration test had skipped Phase 0 and passed. Phase 0 now keeps
  the field, a regression test covers it, and the integration test runs Phase 0.
- **The solver's device decode is conservative.** It places requests best-fit in
  descending order. On two devices it declines {500, 400, 300, 300, 300, 200}, which
  fits as {500, 300, 200} and {400, 300, 300}. IR v1 §3.4 allows a planner to decline
  a placement that fits. The inspector searches every assignment and accepts this set.
  Both behaviors are covered by tests.
- **Two departures from the plan.** The plan proposed `gpuDevices` as a device count
  and a capacity per device. The capacity is always 1,000 milli-GPU, so the field is a
  count. The plan also listed the fragmentation term among the changes. X-3 changes the
  capacity overflow term only. Whether fragmentation should count milli-GPU stranded
  on devices is left to S-2, which sets the weights on this instance.
