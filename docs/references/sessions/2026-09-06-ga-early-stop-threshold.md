# Session record — GA early-stopping threshold — 2026-09-06

**Change measured:** commit `24ffc52`, "give GA early stopping a relative improvement
threshold (#20)".
**Baseline compared against:** commit `1ab1ad7`, recorded in
`2026-09-06-solver-audit.md` and `docs/plans/benchmarks/v0.3.0-regen/`.
**Host:** Linux 7.0.0-31-generic x86_64. Same machine and same instance as the baseline:
620 nodes, 4 DaemonSets, 2,714 pods, 0 pod groups, `SEED = 42`.

Raw console output: `docs/plans/benchmarks/v0.3.0-early-stop/`.

## Commands

```bash
cd solver && cargo build --release
./target/release/kuberina-solver plan \
  --infra testdata/irina_infra.yaml --workloads testdata/irina_workloads.yaml
./target/release/kuberina-solver plan \
  --infra testdata/irina_infra.yaml --workloads testdata/irina_workloads.yaml --headroom 20
uv run --with pyyaml python inspector/inspector.py \
  --infra solver/testdata/irina_infra.yaml \
  --workloads solver/testdata/irina_workloads.yaml \
  --solution solver/kuberina_solution.yaml
uv run --with pyyaml python bench/mathematical_proof.py            # full packing
uv run --with pyyaml python bench/mathematical_proof.py --headroom 20
```

The testdata files were not regenerated; the committed `solver/testdata/irina_*.yaml`
are the ones the baseline ran against, and the FFD seed fitness reproduces to the digit
in both configurations, which confirms the instance is identical.

## Full packing

| | `1ab1ad7` | `24ffc52` | Change |
|---|---|---|---|
| Generations | 1,000 / 1,000 | **200** / 1,000 | stops early |
| Wall clock | 912.49 s | **185.23 s** | −79.7% (4.9× faster) |
| FFD seed fitness | 1,582,568.1260 | 1,582,568.1260 | identical |
| Final fitness | 1,572,736.6090 | 1,582,556.1257 | **0.62% worse** |
| Active nodes | 539 / 620 | **540 / 620** | **+1 node** |
| Fragmentation | 1,567,313.50 | 1,577,120.00 | +9,806.50 |
| Utilization variance | 0.0537 | 0.0628 | worse |
| Capacity / selector / gang penalty | 0 | 0 | unchanged |
| Affinity violations | 0 | 0 | unchanged |
| Verdict | FEASIBLE | FEASIBLE | unchanged |
| Approximation ratio | α = 1.5269 (LB 353) | **α = 1.5297** (LB 353) | +0.0028 |
| Peak RSS | not recorded | 164,884 KiB | — |

The run terminated with `Early stop at generation 199 (gain under 0.0100% for 200 gens)`.

**The 539th node is gone, and that is the expected outcome, not a surprise.** The
baseline reached 539 because a single lucky mutation emptied a node at generation ≈581 —
the only improvement of any size in the entire 1,000-generation run. Everything before it
was drift of order $10^{-9}$ per generation, which is precisely what the threshold is
built to ignore. `docs/references/papers/ga-termination-criteria.md` §3 predicted this
before the run: with ε = 1e-4 the anchor cannot move before the jump, so the run stops at
generation $N$ for every $N < 581$, and §5 predicted 540 nodes and ~185 s. Both landed.

## 20% headroom

| | `1ab1ad7` | `24ffc52` | Change |
|---|---|---|---|
| Generations | 1,000 / 1,000 | **200** / 1,000 | stops early |
| Wall clock | 895.07 s | **184.23 s** | −79.4% (4.9× faster) |
| FFD seed fitness | 2,305,971.1293 | 2,305,971.1293 | identical |
| Final fitness | 2,305,938.0766 | 2,305,938.1245 | +0.0479 absolute ($2\times10^{-8}$) |
| Active nodes | 615 / 620 | 615 / 620 | unchanged |
| Fragmentation | 2,299,752.00 | 2,299,752.00 | unchanged |
| Utilization variance | 0.0383 | 0.0622 | worse |
| Capacity / selector / gang penalty | 0 | 0 | unchanged |
| Verdict | FEASIBLE | FEASIBLE | unchanged |
| Approximation ratio | α = 1.3946 (LB 441) | α = 1.3946 (LB 441) | unchanged |

This is the configuration the fix was written for, and it costs nothing: 710 seconds
saved for a fitness difference of 0.0479 ($2\times10^{-8}$ relative), with the same 615 nodes and the same
approximation ratio.

## Independent verification

The inspector, which shares no code with the solver, re-checked both plans from the
source YAML:

| | Full packing | 20% headroom |
|---|---|---|
| Selector violations | 0 | 0 |
| Capacity overflow, all 8 dimensions | 0.00 | 0.00 |
| Unassigned pods | 0 | 0 |

Monte Carlo significance is unchanged at $p < 10^{-4}$ for both: across 10,000
selector-aware random trials the best random assignment still leaves 379 capacity
violations against Kuberina's zero.

## What this measurement says

1. **The criterion works.** It fired in both configurations, where before it fired in
   neither. Wall clock now measures convergence rather than the generation budget, which
   is what `PAPER.md` §7.4 said it could not.
2. **On the headroom configuration the saving is free.** Nothing measurable was given up.
3. **On full packing the saving costs one node**, and the honest attribution is not the
   threshold. Phase 1 (FFD) reaches 540 active nodes on its own; the GA's entire
   contribution to node count in 1,000 generations was the single lucky evacuation that
   took it to 539. A search whose only real gain is a lottery win cannot be rescued by a
   termination rule — the node-evacuation operator on ROADMAP Phase 1 is what would make
   that gain reproducible, and the multi-restart item added in the same commit is what
   would make better use of the 710 seconds now freed.
4. **`PAPER.md` is now stale in two places.** §7.4 describes the missing threshold as an
   open defect, and the §7 tables report 539 nodes, α = 1.5269, and ~15-minute run times
   for both configurations. All of those were measured at `1ab1ad7`. Filed as an issue
   rather than corrected here, per the rule that the paper is written last and from
   session records.
