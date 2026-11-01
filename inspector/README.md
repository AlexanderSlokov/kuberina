# Kuberina Inspector

Independent constraint validator and heatmap dashboard generator.

The inspector re-reads the infrastructure, workload, and solution YAML files and
re-checks every hard constraint from scratch. It **shares no code with the Rust
solver** — that independence is what makes it a verification tool rather than a
self-report. Any disagreement between the solver and the inspector is a bug in
one of the two.

This is a shipping component, not research scratch. `PAPER.md` §6.5
(Verification Methodology) cites it as the external validator behind every
published result, and the Kalena interface contract (Appendix C.1) relies on it
to detect divergence in Phase 0 capacity computation between the two systems.

## Usage

```bash
make inspector-run
```

Or directly, from the repository root:

```bash
uv run --with pyyaml python inspector/inspector.py \
    --infra solver/testdata/irina_infra.yaml \
    --workloads solver/testdata/irina_workloads.yaml \
    --solution solver/kuberina_solution.yaml \
    --output kuberina_dashboard.html
```

All path defaults are resolved relative to the repository root. Open the emitted
HTML file in a browser for the interactive cluster heatmap.

## What it checks

| Check | Constraint |
|---|---|
| Phase 0 | Re-derives net allocatable capacity by re-applying DaemonSet pre-deduction |
| Capacity | Per-node sum of assigned pod requests ≤ allocatable, on all 8 dimensions |
| Node selector | Every pod's `nodeSelector` matches the labels of the node it landed on |
| Assignment | Every entry in the solution names a node that exists |

Gang grouping and `topologySpread` skew are **not** checked here. Gang is enforced
by the solver's CSP phase, and topology spread is a soft penalty in the fitness
function rather than a feasibility bound — see
`docs/plans/core/MDBP_expansion/implementation_plan_refactor_research.md` for the
reasoning behind that split.
