# Termination Criteria for the Kuberina GA

**Status:** reference. Descriptive of the solver as measured at commit `1ab1ad7`.
**Occasion:** [issue #20](https://github.com/AlexanderSlokov/kuberina/issues/20), backlog
item S-3. This document states why the improvement threshold was set to the value it was
set to, why the generation budget was *not* changed alongside it, and what the benchmark
data says the GA is actually contributing.

Measurements come from `docs/references/sessions/2026-09-06-solver-audit.md` and the raw
console output in `docs/plans/benchmarks/v0.3.0-regen/`. Two runs are referenced
throughout:

- **P1 — full packing**, `make solver-irina`, 1,000 generations, 912.49 s.
- **P2 — 20% headroom**, `make solver-irina-headroom-20`, 1,000 generations, 895.07 s.

Both solve the same deterministic instance: 620 nodes, 2,714 pods, `SEED = 42`.

---

## 1. The improvement process has two components, not one

Fitness is minimized. Write $f_g$ for the best fitness at generation $g$, and

$$\rho(a, g) = \frac{f_a - f_g}{f_a}$$

for the relative improvement from an earlier generation $a$ to generation $g$.

Reading the two logs generation by generation, improvement arrives in two forms that
differ by six orders of magnitude:

| Component | What it is | Rate or size |
|---|---|---|
| **A — smooth drift** | utilization variance and fragmentation micro-tuning | P1: $5.00\times10^{-9}$/gen · P2: $1.43\times10^{-8}$/gen |
| **B — jump** | one node emptied completely | $6.20\times10^{-3}$, observed once in 1,000 generations |

**Component A, measured.** P1 between generations 200 and 580 moves from
$1{,}582{,}556.1257$ to $1{,}582{,}553.1170$: $3.0087$ absolute over 380 generations, or
$5.00\times10^{-9}$ relative per generation. P2 over its full 1,000 generations moves
$33.05$ absolute on a fitness of $2{,}305{,}971.13$, or $1.43\times10^{-8}$ per
generation. Take $r_A = 1.5\times10^{-8}$/gen as an upper bound on what the smooth
component delivers.

**Component B, measured.** P1 between generations 580 and 590 drops from
$1{,}582{,}553.1170$ to $1{,}572{,}736.6180$ — $9{,}816.50$ absolute,
$6.20\times10^{-3}$ relative. The scorecard identifies it exactly: fragmentation falls
$1{,}577{,}120.00 \to 1{,}567{,}313.50$ ($-9{,}806.50$) and active nodes fall
$540 \to 539$, contributing $w_{\text{nodes}} \times 1 = 10$. The two terms sum to
$9{,}816.50$, the whole of the jump. P2 records no such event: its active-node count is
flat at 615 from generation 0 to generation 1,000.

Everything the GA achieved on P1 in 1,000 generations, beyond noise, was this one event.

---

## 2. Sizing the threshold

The defect in #20 is that `stale_count` resets on any improvement whatsoever, so a run
sustained by component A alone never terminates. The fix is a relative threshold
$\varepsilon$: the counter resets only when cumulative improvement since the last reset
exceeds $\varepsilon$. Two constraints bracket the value.

**Lower bound — the criterion must fire on an A-only run.** Over $N$ generations the
smooth component accumulates $r_A N$. With the shipped $N = 200$:

$$r_A N = 1.5\times10^{-8} \times 200 = 3.0\times10^{-6}$$

Any $\varepsilon$ at or below this is absorbed by drift and the criterion stays dead —
which is exactly today's bug, only with a different constant.

**Upper bound — the criterion must never suppress a real jump.** A genuine improvement
of size $\Delta_B = 6.20\times10^{-3}$ must always reset the counter, so
$\varepsilon \ll \Delta_B$.

The admissible window spans a factor of $2{,}067$. Placing $\varepsilon$ at the geometric
mean equalizes the multiplicative safety margin on both sides:

$$\varepsilon^{*} = \sqrt{3.0\times10^{-6} \times 6.20\times10^{-3}} = 1.36\times10^{-4}$$

**Shipped value: $\varepsilon = 10^{-4}$** (0.01% of current best fitness) — 33× above the
noise floor, 62× below the jump. The margin is wide enough that neither bound is
sensitive to the imprecision in $r_A$ or $\Delta_B$, both of which are single-run
estimates.

---

## 3. Every patience value below 581 behaves identically

It is tempting to keep the jump by widening the patience window $N$ instead. On this
data that does not work, and the reason is arithmetic rather than empirical.

With $\varepsilon = 10^{-4}$ and the anchor semantics above, the anchor moves only on a
cumulative gain exceeding $10^{-4} \times 1{,}582{,}568.13 = 158.26$ absolute. P1's total
gain from generation 0 to generation 580 is

$$1{,}582{,}568.126 - 1{,}582{,}553.117 = 15.009 \quad (9.48\times10^{-6}\ \text{relative})$$

which is $10.5\times$ short of the threshold. The anchor therefore never moves before the
jump, `stale_count` equals the generation index for every $g < 581$, and the run
terminates at generation $N$ **for every $N < 581$**. Raising $N$ from 200 to 400 does
not reach the jump; it only spends 180 s more before stopping in the same place.

Capturing the jump requires $N \ge 581$, i.e. 58% of the entire budget. At that setting
the criterion stops being an early-stopping rule: after the jump resets the anchor at
generation 581, the next admissible stop is generation 1,162, past the budget, so P1
would run to completion regardless.

The same conclusion holds for the incremental variant of the rule (compare against the
previous generation instead of an anchor), because no single generation in $[0, 580]$
gains more than $\approx 3$ absolute, itself 50× short of the threshold.

**Corollary.** No stale-based rule can both stop P2 early and preserve P1's jump. P2's
smooth rate ($1.43\times10^{-8}$/gen) is *higher* than P1's ($5.00\times10^{-9}$/gen), so
the fitness trace carries no signal that separates the run worth continuing from the run
worth abandoning.

---

## 4. Staleness carries no information about jumps

Model component B as a rare event with per-generation probability $p$, independent of
history. Then for any $k$,

$$P(\text{jump within the next } k \text{ generations} \mid \text{no jump in the last } N) = 1 - (1-p)^k$$

which does not depend on $N$. The expected remaining gain at generation $g$ of a budget
$B$,

$$E[\text{gain} \mid \text{stale} = N] = (\Delta_B\, p + r_A)(B - g)$$

is likewise independent of $N$. The stale counter is uninformative about the only
component that matters.

Two consequences follow, and they are the substance of this document:

1. **At the present operator set, early stopping is an effort cap, not a convergence
   detector.** There is no generation at which the run has "converged"; there is only a
   generation at which we stop paying. $\varepsilon$ makes the criterion capable of
   firing at all; $N$ decides how much compute a rare event is worth.
2. **The marginal value of continuing is constant**, so a rule that trades wall clock for
   the chance of a jump has no interior optimum. Either the whole budget is worth
   spending or none of it past the first plateau is.

The point estimate is $\hat{p} = 1/1000 = 10^{-3}$ per generation from a single observed
event; the exact Poisson 95% interval for one event, $[0.0253,\ 5.572]$, puts $p$ in
$[2.5\times10^{-5},\ 5.6\times10^{-3}]$ — a factor of 200 of uncertainty. Any policy
tuned finely against $\hat{p}$ would be tuned against noise. At $\hat{p}$, continuing
from generation 200 to 1,000 buys an expected 0.8 node evacuations for 730 s of compute.

**Decision taken:** $\varepsilon = 10^{-4}$, and `early_stop_generations` left at its
existing 200 / 100 / 50 per tier. Widening it is provably ineffective (§3); narrowing it
is unjustified by any measurement. The compute freed is better spent on independent
restarts than on a longer wait in the same basin (§7).

---

## 5. What the GA actually contributes

The audit's D-2 table records the FFD warm-start alone reaching **540 active nodes** on
P1, under both weight orderings tested. The full pipeline finishes at **539**. Of the 81
nodes consolidated away from the 620-node cluster:

| Stage | Nodes eliminated | Share | Wall clock |
|---|---|---|---|
| Phase 1 (FFD warm-start) | 80 | 98.8% | seconds |
| Phase 2 (GA, 1,000 generations) | 1 | 1.2% | 912 s |

On P2 the GA eliminates **zero** nodes (615 before, 615 after); its contribution there is
utilization-variance smoothing, $0.0647 \to 0.0383$.

This is the honest reading of the benchmark, and it reframes what an early-stopping fix
can and cannot do. Stopping at generation 200 costs the 539th node — but that node was
won by a lottery, not by search. The instrument that would win it deliberately is a
node-evacuation operator: node count is the highest-weighted term in the objective, and
single-pod mutation reaches an emptied node only by coincidence, because emptying a node
requires relocating every pod on it within one generation. That work is filed under
ROADMAP Phase 1, *Search Quality and Termination*, and is not a termination-criterion
problem.

---

## 6. Scale: 2,714 pods is not 24,346 TEU, and neither is a TPU pod

`PAPER.md` §8.2 leaves open whether the early-stopping heuristic holds at 5,000+ pods.
The evidence assembled here suggests the question is posed at the wrong level: the
generation budget is not the variable that scales.

**Maritime practice decomposes rather than enlarges the search.** `PAPER.md` §2.3 already
states the reason: simultaneously optimizing 24,000+ units is intractable, so the CSPP is
split into the Master Bay Plan Problem (container groups to bays) and the Slot Plan
Problem (individual containers to coordinates). The strongest exact result located for
this document, Roberti and Pacino (2018), reports solving to optimality "instances with
up to 10 ports and 5,000 containers in a few minutes of computing time" — and does so via
a decomposition into two sets of decision variables with column generation, not by
running a longer population search. Kuberina currently encodes all 2,714 pods as a single
flat chromosome with one gene per pod, which is the Slot Plan formulation applied to the
whole vessel at once.

**Accelerator fleets are a different problem, not a bigger one.** A TPU v4 pod is 4,096
chips wired as a 3D mesh, and a workload requests capacity as a topology 3-tuple
`AxBxC`, with torus wrap-around available only on slices where `2A=B=C` or `2A=2B=C`
(Google Cloud TPU v4 documentation). Allocation is therefore a question of *shape* inside
an interconnect geometry, not of packing independent items into vector-valued bins; the
objective is interconnect locality, and the unit of allocation is a slice rather than a
chip. Google's own fleet analysis calls the resulting scheduling task an "NP-hard
bin-packing problem" (§3.2 of the ML Productivity Goodput paper) — and production systems
solve it with topology-aware schedulers, not population search.

**Implication for Kuberina.** Going from 2,714 to 24,000 units will not be answered by
more generations, and §3 above shows why: a flat encoding makes per-gene mutation shrink
as $1/|P|$ to stay local, so the reachable neighborhood per generation shrinks with the
instance while the space grows. The maritime answer — plan at node-pool granularity
first, then within pool — is the structural change that scales, and it is the same
two-phase shape the paper already credits to the stowage literature.

---

## 7. What the literature says about generation counts

- **A generation budget is not a convergence criterion.** Ravber, Liu, Mernik and
  Črepinšek argue the point in the title: *Maximum number of generations as a stopping
  criterion considered harmful*. Algorithms consume different numbers of evaluations per
  generation, so a generation cap silently advantages some and handicaps others, and the
  paper gives guidelines for stating budgets in evaluations instead. Kuberina's own
  budget is 1,000 generations × 1,024 individuals ≈ $1.02\times10^{6}$ evaluations; the
  evaluation count, not the generation count, is the figure worth reporting.
- **Theory does not license "run it longer" at this scale.** Aytug and Koehler derive a
  bound on the number of generations needed to have seen the optimum with a stated
  confidence, as a function of only mutation rate, string length and population size. For
  a 2,714-gene chromosome at a per-gene mutation rate of $\approx 0.00147$, that bound is
  astronomically large — it is a statement that mutation-driven sampling cannot reach the
  optimum here, not a target to tune toward.
- **The criterion being implemented is standard prior art.** Zielinski and Laur catalogue
  improvement-based stopping criteria — terminating when the improvement of the best
  objective value falls below a bound (`ImpBest`), or when the population's average
  improvement does (`ImpAv`) — alongside diversity-based criteria such as `MaxDist`,
  which terminate on the collapse of the population's spread rather than on its fitness.
  The threshold in §2 is `ImpBest` with an instance-calibrated bound. A diversity-based
  criterion is the natural next candidate if the improvement-based one proves too blunt.
- **Rare, memoryless progress argues for restarts, not patience.** Gomes, Selman, Crato
  and Kautz documented heavy-tailed run-time distributions in combinatorial search, where
  a small probability of a very long run dominates the mean. Luby, Sinclair and Zuckerman
  proved the matching result for Las Vegas algorithms: when the run-time distribution is
  known, a fixed cutoff with restarts is optimal, and when it is unknown, a universal
  restart sequence is optimal to within a logarithmic factor. §4's memorylessness is
  exactly the regime those results address, and it converts the compute freed by early
  stopping into a strictly better use: five independent seeds of 200 generations rather
  than one run of 1,000.
- **The 1,000-generation budget is an outlier for this problem class.** Recent industrial
  3D bin packing with a GA reports "a population size of 100, up to 50 generations, an
  elite rate of 10%" (GENPACK, §4.5) on 1,500 real-world orders. Kuberina runs twenty
  times as many generations, which is a symptom of a weak operator rather than of a
  harder instance.

---

## 8. Decisions and open items

| Decision | Where it lands |
|---|---|
| `min_relative_improvement = 1e-4`, reset the stale counter only on gains exceeding it | `solver/src/model.rs`, `solver/src/phase2_ga.rs`; mirrored in `research/` |
| `early_stop_generations` unchanged (200 / 100 / 50) | §3 — every value below 581 is equivalent on this instance |
| P1 is expected to finish at 540 active nodes rather than 539 | recorded in the session record for this change, not hidden |
| Node-evacuation operator | ROADMAP Phase 1 — the actual fix for §5 |
| Multi-restart (bet-and-run) driver | ROADMAP Phase 1 — the actual use for the compute freed here |

---

## References

Bibliographic fields were verified against the Semantic Scholar API records and the
publishers' landing pages. Where a claim rests on a paywalled full text that could not be
retrieved from this environment, that is stated inline; verbatim quotations are given
only for text actually fetched.

1. H. Aytug and G. J. Koehler, "New stopping criterion for genetic algorithms,"
   *European Journal of Operational Research*, vol. 126, no. 3, pp. 662–674, 2000.
   DOI: [10.1016/S0377-2217(99)00321-5](https://doi.org/10.1016/S0377-2217(99)00321-5).
   *Full text paywalled; the bound is described here from secondary summaries, not
   quoted.*

2. M. Ravber, S.-H. Liu, M. Mernik and M. Črepinšek, "Maximum number of generations as a
   stopping criterion considered harmful," *Applied Soft Computing*, vol. 128, art.
   109478, 2022.
   DOI: [10.1016/j.asoc.2022.109478](https://doi.org/10.1016/j.asoc.2022.109478).
   *Listed as hybrid open access (CC BY); the publisher's copy returned HTTP 403 from
   this environment, so no verbatim quotation is given.*

3. K. Zielinski and R. Laur, "Stopping Criteria for Differential Evolution in Constrained
   Single-Objective Optimization," in *Constraint-Handling in Evolutionary Optimization*
   (Studies in Computational Intelligence, vol. 198), E. Mezura-Montes, Ed. Springer,
   2009, pp. 111–138.
   DOI: [10.1007/978-3-540-68830-3_4](https://doi.org/10.1007/978-3-540-68830-3_4).
   *Chapter title, authors, pages and DOI verified; book placement taken from the
   publisher's listing. Full text paywalled.*

4. C. P. Gomes, B. Selman, N. Crato and H. Kautz, "Heavy-Tailed Phenomena in
   Satisfiability and Constraint Satisfaction Problems," *Journal of Automated
   Reasoning*, vol. 24, no. 1–2, pp. 67–100, 2000.
   DOI: [10.1023/A:1006314320276](https://doi.org/10.1023/A:1006314320276).

5. M. Luby, A. Sinclair and D. Zuckerman, "Optimal speedup of Las Vegas algorithms,"
   *Information Processing Letters*, vol. 47, no. 4, pp. 173–180, 1993.
   DOI: [10.1016/0020-0190(93)90029-9](https://doi.org/10.1016/0020-0190(93)90029-9).

6. R. Roberti and D. Pacino, "A Decomposition Method for Finding Optimal Container
   Stowage Plans," *Transportation Science*, vol. 52, no. 6, pp. 1444–1462, 2018.
   DOI: [10.1287/trsc.2017.0795](https://doi.org/10.1287/trsc.2017.0795).
   From the abstract, verbatim: "The proposed solution method outperforms the methods
   from the literature and can solve to optimality instances with up to 10 ports and
   5,000 containers in a few minutes of computing time."

7. D. Poolavaram, C. Markgraf and S. Dorn, "GENPACK: KPI-Guided Multi-Criteria Genetic
   Algorithm for Industrial 3D Bin Packing," arXiv:2601.11325, 16 Jan 2026.
   [arXiv:2601.11325](https://arxiv.org/abs/2601.11325). §4.5, verbatim: "We use a
   population size of 100, up to 50 generations, an elite rate of 10%, and adaptive Pc
   and Pm for crossover and mutation, respectively."

8. A. Wongpanich, T. Oguntebi, J. Baiocchi Paredes, Y. E. Wang, P. M. Phothilimthana,
   R. Mitra, Z. Zhou, N. Kumar and V. Janapa Reddi, "Machine Learning Fleet Efficiency:
   Analyzing and Optimizing Large-Scale Google TPU Systems with ML Productivity
   Goodput," arXiv:2502.06982, 2025.
   [arXiv:2502.06982](https://arxiv.org/abs/2502.06982). §3.2, verbatim: the scheduler
   faces an "NP-hard bin-packing problem."

9. Google Cloud, "TPU v4," Cloud TPU documentation.
   [docs.cloud.google.com/tpu/docs/v4](https://docs.cloud.google.com/tpu/docs/v4).
   Verbatim: "TPU Pod size: 4096 chips"; "v4 TPUs have a direct connection to the nearest
   neighboring chips in 3 dimensions, resulting in a 3D mesh of networking connections";
   connections "can be configured as a 3D torus on slices where the topology, AxBxC, is
   either 2A=B=C or 2A=2B=C."

10. Kuberina, `docs/references/sessions/2026-09-06-solver-audit.md` and
    `docs/plans/benchmarks/v0.3.0-regen/` — the run at commit `1ab1ad7` from which every
    measurement in §1, §3 and §5 is taken.
