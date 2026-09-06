# Kuberina Backlog

Concrete, actionable tasks. Where [ROADMAP.md](ROADMAP.md) states *what capability
Kuberina will gain and in which release*, this file states *what has to be edited to
get there*. Every item names the file it touches and the condition under which it is
done. Completed items move to [CHANGELOG.md](CHANGELOG.md) under `[Unreleased]`.

Items are grouped by the artifact they modify. An item that changes solver behavior
belongs to a ROADMAP phase first; an item that corrects or completes an existing
artifact does not.

---

## White paper (`docs/references/PAPER.md`)

The published results in §6 and §7 were generated against a testbed that no longer
exists in the repository, and three separate accuracy gaps sit on top of that. W-1 is
a prerequisite for W-2 and W-3: correcting the ratio or the dimension coverage before
the underlying run is regenerated means computing both twice.

### W-1 — Regenerate §6.1 and §7 against the current testbed

**Blocks:** W-2, W-3.

`bench/gen_irina_testdata.py` now emits 620 nodes (400 standard, 120 memory-optimized,
100 GPU); `docs/references/PAPER.md` §6.1 still reports the 186-node testbed the
results were measured on. Pod demand is unchanged, so cluster capacity grew by roughly
3.3× while the workload did not — the published run describes a materially harder
instance than `make solver-irina` solves today.

| Quantity | §6.1 states | `solver/testdata/` holds |
|---|---|---|
| Total nodes | 186 | 620 |
| Total pods | 2,714 | 2,714 |
| Cluster CPU (raw) | 10,272 | 34,240 |
| Cluster RAM (raw) | 54,912 | 183,040 |
| Cluster GPU | 240 | 800 |
| CPU fill | 73.1% | 21.0% |
| RAM fill | 51.2% | 15.2% |

Every figure in §7.1, §7.2 and §7.3 descends from the 186-node run: 152 active nodes,
88.7% average CPU utilization, fragmentation 18,418.00, `L = 117`, `L_het = 136`,
`α = 1.3382`. None of them reproduce.

**Done when:** §6.1 and all of §7 are regenerated from a single recorded run of the
current testdata, and the run is reproducible via `make solver-irina`,
`make solver-irina-pareto-80`, `make inspector-run` and `make bench-proof`.

**Also:** the header comment in `solver/testdata/irina_infra.yaml` reads
`# 100 Standard + 30 Memory + 20 GPU nodes`, which matches neither the file nor the
generator. Fix it in the same pass.

### W-2 — Report the approximation ratio against a matching capacity model

**Issue:** [#8](https://github.com/AlexanderSlokov/kuberina/issues/8) · **Depends on:** W-1

§7.3 computes `α = 182 / 136`, where the numerator is the active-node count from a
`--pareto 80` run and the denominator is a lower bound derived from uncapped node
capacities. `bench/mathematical_proof.py` has no `--pareto` flag and no CLI arguments
at all, so the bound cannot express the cap. The two halves of the ratio come from
different capacity models.

The error understates the result. On the same published data the full-packing
configuration gives `152 / 136 = 1.118`, which sits *below* the 11/9 ≈ 1.222 FFD bound
that the following sentence explains the result as exceeding.

**Done when:** §7.3 reports α for the full-packing configuration, whose objective
matches the bound, and the Pareto configuration is handled by one of:

1. a `--pareto` flag on `bench/mathematical_proof.py` that scales `total_supply`
   exactly as the solver scales node capacity, with both ratios reported side by side; or
2. an explicit statement that no optimality bound is claimed for the Pareto
   configuration, because minimizing active nodes is deliberately not its objective.

The abstract must name the configuration its α came from.

### W-3 — Extend the feasibility proof and LP bound to all 8 dimensions

**Issue:** [#11](https://github.com/AlexanderSlokov/kuberina/issues/11) · **Depends on:** W-1

`bench/mathematical_proof.py` iterates all eight dimensions in
`compute_lp_lower_bound`, `compute_heterogeneous_lower_bound` and the capacity
verifier. §7.3 reports three: capacity overflow "on CPU, RAM, GPU", and `L^r` for CPU,
RAM and GPU only. §6.1's testbed table has the same 3-of-8 gap. A reader cannot
distinguish "checked and slack" from "not checked" — which is the distinction a proof
section exists to settle.

The five dimensions added in v0.2.0 are not slack on the current testbed. All 620
nodes carry finite capacity in all eight dimensions, and measured fill ratios put four
of the new dimensions above RAM and GPU:

| Dimension | Fill |
|---|---|
| disk_write | 54.1% |
| net_out | 33.4% |
| net_in | 31.2% |
| disk_read | 30.3% |
| cpu | 21.0% |
| gpu | 19.0% |
| storage | 15.9% |
| ram | 15.2% |

**Done when:** Proof 1 reports overflow per dimension and Proof 2 reports `L^r` per
dimension, both across all eight; §6.1's testbed table carries cluster totals for
storage, disk and network; and any dimension left unconstrained is marked as such
rather than omitted.

**Note:** the `float('inf')` default at `bench/mathematical_proof.py:57` (mirroring the
solver's `f64::MAX`) does not trigger on this testbed — no node omits a dimension. The
branch stays correct for hand-written infra files; it is simply not what §7.3 is
currently hiding.

### W-4 — Document the dual-capacity model behind `--pareto`

**Issue:** [#12](https://github.com/AlexanderSlokov/kuberina/issues/12)

Under `--pareto p`, `solver/src/main.rs` builds a capped copy of the node set and hands
it to the FFD warm-start (`main.rs:116`), the fitness function (`main.rs:118`) and the
GA (`main.rs:129`), then prints the blueprint against the true post-DaemonSet
capacities (`main.rs:137`). That asymmetry is the flag's defining property and is
documented only in the source comment on `main.rs:136`.

It is worth stating because the guarantee is stronger than what §7.2 currently claims.
"No node exceeds 79% utilization" reads as an observed outcome; the mechanism makes it
structural and per-node — the optimizer never sees the reserve, so every node in the
blueprint carries at least `(100 − p)%` of its real capacity unallocated.

Left undocumented, two misreadings are available: that the cap is a post-hoc rejection
filter (making the reserve a soft target), or that reported utilization percentages are
relative to capped capacity (making 74.6% mean 59.7% of real capacity).

**Done when:** §6.2 states which components see capped capacity and which see real
capacity, and §7.2 notes that its fragmentation row is measured against capped capacity
in the Pareto column and real capacity in the full-packing column — the two are not
directly comparable as printed. Reporting both against real capacity would make the
table comparable and would show the Pareto run's true waste.

**Deferred sub-item:** renaming the flag to `--reserve 20` / `--headroom 20`. "Pareto"
suggests a Pareto front or an 80/20 distributional claim and the mechanism is neither,
but a rename is a breaking CLI change and belongs to a release boundary, not to a
documentation pass.
