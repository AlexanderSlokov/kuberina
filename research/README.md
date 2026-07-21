# Kuberina Research — Python MVP (v0.0.0)

Proof-of-concept implementation of the Kuberina 3-phase scheduling pipeline,
validating the mathematical model from the whitepaper.

## Quick Start

```bash
cd research/
uv sync
uv run python -m kuberina plan \
    --infra testdata/homelab_infra.yaml \
    --workloads testdata/homelab_workloads.yaml
```

## Run Tests

```bash
uv run pytest tests/ -v
```

## Default Hyperparameters

### Fitness Weights

From PAPER.md §3.2 — `F(s) = w1·f_nodes + w2·f_frag + w3·f_affinity + w4·f_var + Φ(s)`:

| Weight | Default | Rationale |
|---|---|---|
| `w1` (node count) | 10.0 | Highest priority — minimize infrastructure cost |
| `w2` (fragmentation) | 1.0 | Low weight — secondary concern to node count |
| `w3` (affinity violation) | 5.0 | Medium — respect co-location preferences |
| `w4` (utilization variance) | 2.0 | Low-medium — promote balanced load |

These are MVP starting points. Tune with real workloads.

### FFD Scalarization (Synthetic Volume)

From PAPER.md §4.2 — `V_i = α·CPU + β·RAM + γ·GPU`:

| Coefficient | Default | Rationale |
|---|---|---|
| α (CPU) | 1.0 | Standard weight |
| β (RAM) | 1.0 | Standard weight |
| γ (GPU) | 10.0 | GPU scarcity dominates when present |

For homelabs without GPUs, this degenerates to equal CPU+RAM weighting.

### GA Configuration

From ga_estimation.md §3 (Small tier):

| Parameter | Default | Source |
|---|---|---|
| Population | 128 | ga_estimation.md §3 |
| Tournament size | 3 | ga_estimation.md §4 |
| Mutation rate | 5% | ga_estimation.md §4 |
| Crossover rate | 80% | Standard GA literature |
| Max generations | 500 | ga_estimation.md §5 |
| Early stop | 50 gens | ga_estimation.md §5 |
