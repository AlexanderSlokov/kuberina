# Session record — admission (unplaced Pods) — 2026-11-01

**Backlog items:** X-4a to X-4i · **Issue:** [#24](https://github.com/AlexanderSlokov/kuberina/issues/24)
**Plan:** `docs/plans/benchmarks/alibaba-gpu-v2023/00_plan.md`, Stage 4 and decisions 4–6
**Kuberina commit:** `cfee303`, which adds the X-4 changes. The measurements ran on its
working tree before the commit; the code is identical.
**Compared against:** `096177e`, recorded in `2026-11-01-irina-with-affinity.md`.
**Host:** Linux 7.0.0-38-generic x86_64, 8 vCPU, 13 GiB RAM.

## What was measured

X-4 lets the solver leave Pods unplaced, with `--admission in-order` or
`--admission free` (IR v1 §4.1). The Pods' order comes from the new `arrival` field
(§3.5). This session checked four things:

1. Both modes give a feasible blueprint with an `unplaced:` list at more than 100% GPU
   demand.
1. In-order admission skips a Pod that does not fit and admits later Pods, and the
   result keeps the in-order rule, by the solver's check and by the inspector's own.
1. `kuberina-forge out` refuses a blueprint with unplaced Pods.
1. Without `--admission`, the MSC Irina FFD seed fitness is unchanged to the digit.

The fixture is `solver/testdata/admission_infra.yaml` and
`solver/testdata/admission_workloads.yaml`: two nodes with 8 GPU devices each, and 11
Pods requesting 21.5 GPUs (134%). The arrival order mixes whole-GPU training Pods with
fractional inference Pods.

## Commands

```shell
cd solver && cargo build --release
kuberina-solver plan --infra testdata/admission_infra.yaml \
    --workloads testdata/admission_workloads.yaml --admission in-order
kuberina-solver plan --infra testdata/admission_infra.yaml \
    --workloads testdata/admission_workloads.yaml --admission free
uv run --with pyyaml python inspector/inspector.py \
    --infra solver/testdata/admission_infra.yaml \
    --workloads solver/testdata/admission_workloads.yaml \
    --solution kuberina_solution.yaml --output dashboard.html
kuberina-forge out --blueprint kuberina_solution.yaml --out planned forge/testdata/manifests
kuberina-solver plan --infra testdata/irina_infra.yaml --workloads testdata/irina_workloads.yaml
kuberina-solver plan --infra testdata/irina_infra.yaml --workloads testdata/irina_workloads.yaml --headroom 20
make bench-alibaba-ir
```

The two MSC Irina runs were stopped after the FFD seed line, 90 seconds in. Wall clock
and peak memory come from `/usr/bin/time -f "%e s, %M KB"`.

## Results

### The 134% fixture

The following table lists what the solver and the inspector reported for each mode:

| Quantity | `in-order` | `free` |
|---|---|---|
| Solver exit status | 0 | 0 |
| `metadata.feasible` | true | true |
| Pods unplaced | 2 of 11 (`train-c-0`, `train-d-0`) | 8 of 11 |
| GPU allocation, solver | 15.50 of 16 GPUs (96.88%) | 16.00 of 16 GPUs (100.00%) |
| GPU allocation, inspector | 15.50 of 16 GPUs (96.88%) | 16.00 of 16 GPUs (100.00%) |
| Inspector: GPU overflow, device violations, placement problems | 0, 0, 0 | 0, 0, 0 |
| Inspector: in-order violations | 0 | not checked |
| Wall clock, peak memory | 0.01 s, 3,260 KB | 0.01 s, 3,496 KB |

Under in-order admission, `train-c` (4 GPUs, arrival 4) was rejected, and the
inference Pods that arrived after it were admitted. This is the skip-and-continue rule.

A blueprint edited by hand to decline `infer-a-0` as well was reported by the
inspector:

```console
In-Order Violations: 1
  bench/infer-a-0 could take the place of bench/train-b-0
```

### The forge refuses a partial plan

`kuberina-forge out`, given the in-order blueprint, exited with status 1, wrote no
manifest, and printed:

```console
kuberina-forge: kuberina_solution.yaml: the planner left 2 Pod(s) unplaced (metadata.admission: in-order); a partial plan serves benchmarks and is not linked to manifests
```

### MSC Irina without `--admission`

The FFD seed fitness, compared with the session record of `096177e`:

| Configuration | `096177e` | This session |
|---|---|---|
| Full packing | 1,587,653.1260 | 1,587,653.1260 |
| `--headroom 20` | 2,311,466.1293 | 2,311,466.1293 |

Both values are identical to the digit.

### The Alibaba seed-42 instance

The adapter now writes `arrival` on every declaration. The regenerated instance still
sums to 8,075,080 milli-GPU, and each `arrival` equals its `sequence.csv` row (test
`test_every_arrival_equals_its_sequence_row`). `workloads.yaml` SHA-256 is
`ea5df9ae4e4d95147991b9f3137ba47fcc8120984c3368c5252b39b9b3b598f4`; `infra.yaml` and
`sequence.csv` are unchanged from `2026-11-01-per-device-gpu.md`.

The solver loads the instance with `--admission in-order` and accepts its arrival order:
`Loaded 1213 nodes, 0 daemonsets, 10866 pods, 0 groups`. The run was stopped after
20 seconds, and nothing it computed was read. By the plan's stance, the weights (S-2)
are fixed before Kuberina's first run on the trace.

## Findings

- **Free admission stops at the FFD seed on this fixture.** FFD places the three
  training Pods first, which fills all 16 GPUs, and the GA does not improve on that. A
  placement with the same 16 GPUs and 7 Pods exists: `train-b`, `train-a`, `train-d`,
  `infer-a`, `infer-b`, and both `infer-f` replicas, with each inference pair sharing a
  device at 1,000 milli-GPU. Reaching it needs one large Pod declined and several small
  Pods placed in one step. The mutation operators make one change per
  gene. The GPU allocation, which is what E3 reports, is already 100% here. S-2 reviews
  the free-mode operators together with the weights.
- **The objective weights are provisional.** For each unplaced Pod, the term charges
  `W × (1 + gpu / g_min)`, with `W = 2 × (w_node + w_frag × C_max)`. On the fixture,
  `C_max` is 96 + 384 + 8 = 488 and `g_min` is 0.3 GPU. S-2 replaces these values on
  the Alibaba instance.
- **The in-order rule is enforced after the GA.** The GA moves Pods without knowing the
  rule. The solver therefore repairs every breach after the GA: it admits the earlier
  Pod in the later Pod's place, then tries to place the later Pod elsewhere.
- **The infeasible-blueprint banner is fixed.** The second line of the banner carried 13
  spaces of source indentation. The writer moved to `solver/src/blueprint_yaml.rs`, with
  a test for the banner and a test that pins the pre-X-4 document shape.
