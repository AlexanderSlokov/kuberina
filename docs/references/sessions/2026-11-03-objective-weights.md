# Session record — objective weights on the Alibaba instance (S-2) — 2026-11-03

**Backlog items:** S-2a to S-2d · **Issue:** [#24](https://github.com/AlexanderSlokov/kuberina/issues/24)
**Plan:** `docs/plans/benchmarks/alibaba-gpu-v2023/00_plan.md`, §8 decisions 7–11, and
[ADR-0003](../../explanation/adr/0003-ga-must-beat-the-walk.md)
**Kuberina commit:** `112be75`, which adds the S-2 changes. The measurements ran on its
working tree, on top of `05a0680`; the code is identical.
**Compared against:** `cfee303`, recorded in `2026-11-01-admission.md`.
**Host:** Linux 7.0.0-38-generic x86_64, 8 vCPU, 13 GiB RAM.

## What was measured

S-2 carries out the four decisions the maintainer made on 2026-11-02:

1. **S-2a.** The fragmentation term counts milli-GPU _stranded_ on devices: free room
   on a device that is partly used, such as the 400 milli-GPU left beside a 600
   milli-GPU Pod. The in-order walk chooses, among the nodes a Pod fits, the node where
   the Pod adds the least stranded room, then the best fit.
1. **S-2b.** The unplaced-Pod charge keeps its dominance formula. Its constants are
   computed on the Alibaba seed-42 instance.
1. **S-2c.** Free admission gains a trade operator: decline one placed GPU Pod and
   refill its node with unplaced Pods, largest GPU first.
1. **S-2d.** `--no-ga` stops the solver after Phase 1 or the admission walk.

## Commands

```shell
cd solver && cargo build --release
kuberina-solver plan --infra testdata/irina_infra.yaml --workloads testdata/irina_workloads.yaml
kuberina-solver plan --infra testdata/irina_infra.yaml --workloads testdata/irina_workloads.yaml --headroom 20
kuberina-solver plan --infra testdata/admission_infra.yaml \
    --workloads testdata/admission_workloads.yaml --admission in-order
kuberina-solver plan --infra testdata/admission_infra.yaml \
    --workloads testdata/admission_workloads.yaml --admission in-order --no-ga
kuberina-solver plan --infra testdata/admission_infra.yaml \
    --workloads testdata/admission_workloads.yaml --admission free
uv run --with pyyaml python inspector/inspector.py \
    --infra solver/testdata/admission_infra.yaml \
    --workloads solver/testdata/admission_workloads.yaml \
    --solution kuberina_solution.yaml --output dashboard.html
kuberina-solver plan --infra bench/alibaba/out/seed42/infra.yaml \
    --workloads bench/alibaba/out/seed42/workloads.yaml --admission in-order
```

The two MSC Irina runs were stopped after the FFD seed line. Wall clock and peak memory
come from `/usr/bin/time -f "%e s, %M KB"`.

## Results

### MSC Irina without `--admission`

MSC Irina declares no GPU devices, so the stranded term is zero and the FFD seed does not
use the walk. The seed fitness, compared with `cfee303`:

| Configuration | `cfee303` | This session |
|---|---|---|
| Full packing | 1,587,653.1260 | 1,587,653.1260 |
| `--headroom 20` | 2,311,466.1293 | 2,311,466.1293 |

### Weights on the Alibaba seed-42 instance (S-2b)

The instance is the one recorded in `2026-11-01-admission.md`: `infra.yaml` SHA-256
`4019a82bf3f07cf552b3092f138736791bfebacb43cce77ead193f78a279ac0a`, `workloads.yaml`
SHA-256 `ea5df9ae4e4d95147991b9f3137ba47fcc8120984c3368c5252b39b9b3b598f4`.

The derivation, with each input and where it comes from:

| Quantity | Value | Source |
|---|---|---|
| `w_node` | 10 | `FitnessWeights::default()` |
| `w_frag` | 1 | `FitnessWeights::default()` |
| Largest node | `openb-node-1032`: 128 CPU, 1,024 GiB RAM, 1 GPU, 1 device | `infra.yaml` |
| `C_max` | 128 + 1,024 + 1 + 1 = 1,154 | Capacity sum plus one GPU of stranded room per device |
| `g_min` | 50 milli-GPU = 0.05 GPU | Smallest positive request in `sequence.csv` |
| Per-Pod charge `W` | 2 × (10 + 1 × 1,154) = 2,328 | Solver: `Unplaced-Pod weights: 2328.0000 per Pod` |
| Per-GPU charge `W / g_min` | 2,328 / 0.05 = 46,560 | Solver: `46560.0000 per GPU` |

The solver printed the two charges at load. A script that read `infra.yaml` and
`sequence.csv` on its own reached the same `C_max` and `g_min`.

`C_max` adds the device count because the stranded term can charge a freed node up to
one GPU per device. With it, freeing any node saves at most `w_node + w_frag × C_max`,
and declining a Pod costs at least twice that.

### The 134% fixture

The following table compares the X-4 results with this session's:

| Quantity | `in-order`, `cfee303` | `in-order`, this session | `in-order --no-ga` | `free`, `cfee303` | `free`, this session |
|---|---|---|---|---|---|
| Pods placed | 9 of 11 | 9 of 11 | 9 of 11 | 3 of 11 | **9 of 11** |
| Unplaced | `train-c-0`, `train-d-0` | `train-c-0`, `train-d-0` | `train-c-0`, `train-d-0` | 8 Pods | `train-a-0`, `train-d-0` |
| GPU allocation (solver and inspector) | 15.50 of 16 (96.88%) | 15.50 of 16 (96.88%) | 15.50 of 16 (96.88%) | 16.00 of 16 (100%) | **15.50 of 16 (96.88%)** |
| Seed fitness | — | 22,865.0156 | 22,865.0156 | — | 27,309.3333 |
| Final fitness | — | 22,865.0156 | 22,865.0156 | — | 22,865.0156 |
| GA | — | converged at generation 50 of 500 | skipped | — | converged at generation 79 of 500 |
| Inspector: overflow, device violations, placement problems | 0, 0, 0 | 0, 0, 0 | 0, 0, 0 | 0, 0, 0 | 0, 0, 0 |
| Inspector: in-order violations | 0 | 0 | 0 | not checked | not checked |
| Wall clock, peak memory | 0.01 s, 3,260 KB | 0.02 s, 3,472 KB | 0.00 s, 3,160 KB | 0.01 s, 3,496 KB | 0.02 s, 3,468 KB |

The in-order blueprints with and without the GA are identical, Pod for Pod.

### Kuberina ran on the trace before this record was written

The weights were read from a run on the Alibaba instance that was meant to stop at the
weights line. The stop command waited for a 120-second timeout, so the run went on for
about 112 seconds, through the in-order walk and into the GA, and was stopped at
generation 100 of 1,000. Its console showed:

```console
Phase 1 (FFD): seed fitness = 109285356.4890
Gen  100/1000 | best=109285355.4821 | stale=101
  Frag: 205959.72 | Aff: 0 | Var: 0.0808
  [Nodes] Active: 1213 / 1213
```

Nothing was changed after this output was seen: the weights follow the formula fixed on
2026-11-02, and no setting was tuned. The walk and the GA were not timed, and X-5
measures them. The run is reported here because the plan states that weights are fixed
before Kuberina's first run on the trace.

## Findings

- **The objective trades GPU for Pod count at `g_min` per Pod.** Under the kept formula,
  placing one more Pod is worth `g_min` GPUs of placed demand. On the fixture,
  `g_min` is 0.3 GPU, so free admission prefers 9 Pods at 15.5 GPUs to 7 Pods at 16
  GPUs, and its GPU allocation fell from 100% to 96.88%. On the Alibaba instance,
  `g_min` is 0.05 GPU. E3 reports GPU allocation, so its result is read with this
  rule beside it.
- **The trade operator works.** At `cfee303`, free admission stopped at the FFD seed
  (3 Pods). With the trade, it reached the in-order result (9 Pods, fitness
  22,865.0156).
- **On the fixture, the GA changed nothing under in-order admission.** The fixture has
  11 Pods, so this says nothing about the trace. ADR-0003 decides from seed 42 (X-7).
- **The stranded term adds a decode per node to every fitness evaluation** on clusters
  with devices. Its cost on the trace is part of X-5.
- **`C_max` on the trace is dominated by memory.** The largest node holds 1,024 GiB
  of RAM, which is 89% of `C_max`. Fragmentation sums CPU cores, GiB, and GPUs in one
  number, so the GiB count dominates it.
