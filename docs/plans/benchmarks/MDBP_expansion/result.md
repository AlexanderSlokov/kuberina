```bash
stella@stella-HP-Pavilion-15:~/workspace/kuberina$ make research-full-pipeline
=> Generating 8D testdata...
cd research && uv run python gen_irina_testdata.py
════════════════════════════════════════════════════════════
  MSC IRINA SCALE — Resource Summary
════════════════════════════════════════════════════════════
  Nodes:          620
  Pods:           2714
  Constraints:    13762 anti-affinity + 496 affinity
────────────────────────────────────────────────────────────
  Cluster CPU:    34240 cores (net: 32845.0)
  Cluster RAM:    183040 GiB  (net: 181335.0)
  Cluster GPU:    800 units
────────────────────────────────────────────────────────────
  Pod CPU demand: 7198 cores  (21.9% fill)
  Pod RAM demand: 27840 GiB   (15.4% fill)
  Pod GPU demand: 152 units  (19.0% fill)
════════════════════════════════════════════════════════════

  Written: testdata/irina_infra.yaml
  Written: testdata/irina_workloads.yaml
=> Running solver on generated testdata...
cd solver && cargo run --release -- plan --infra ../research/testdata/irina_infra.yaml --workloads ../research/testdata/irina_workloads.yaml --pareto 80
    Finished `release` profile [optimized] target(s) in 0.05s
     Running `target/release/kuberina plan --infra ../research/testdata/irina_infra.yaml --workloads ../research/testdata/irina_workloads.yaml --pareto 80`
Loaded 620 nodes, 4 daemonsets, 2714 pods, 0 groups

═══ Phase 0: DaemonSet Pre-deduction (Ballast Water) ═══
  std-000: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-001: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-002: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  < And a lots of nodes. I cut off for the sake of long output >
  gpu-097: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-098: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-099: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)

[Pareto Mode] Capping node capacities to 80.0% for placement optimization.
Phase 1 (FFD): seed fitness = 104817.6781
Datacenter-scale detected (2714 pods) — cranking GA to maximum
Gen  190/1000 | best=104817.6781 | stale=191 | ███░░░░░░░░░░░░░░░░░ 19%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 102481.60 | Aff: 0 | Var: 0.0391
  [Nodes] Active: 200 / 620
Early stop at generation 199 (no improvement for 200 gens)

═══ Final Blueprint (Stowage Plan) ═══
  Fitness: 104817.6781
  Time: 230.36s
  Scorecard:
    Capacity Penalty: 0
    Selector Penalty: 0
    Gang Penalty: 0
    Active Nodes: 200
    Fragmentation: 102481.60
    Affinity Violations: 0
    Utilization Variance: 0.0391
    Topology Spread Penalty: 112.00

  ─── Cluster Summary (620 nodes) ───
  Active nodes: 200 / 620
  Empty nodes:  420
  Pods placed:  2714
  Avg CPU util: 67.4%

  ─── Top 5 Busiest Nodes ───
  std-016: 49.0/61.8 CPU (79%), 14 pods
  std-017: 49.0/61.8 CPU (79%), 11 pods
  std-018: 49.0/61.8 CPU (79%), 11 pods
  std-019: 49.0/61.8 CPU (79%), 11 pods
  std-020: 49.0/61.8 CPU (79%), 12 pods

  ─── Top 5 Lightest Active Nodes ───
  gpu-025: 8.0/45.8 CPU (17%), 2 pods
  mem-020: 9.0/29.8 CPU (30%), 3 pods
  mem-021: 9.0/29.8 CPU (30%), 3 pods
  mem-022: 9.0/29.8 CPU (30%), 3 pods
  mem-023: 9.0/29.8 CPU (30%), 3 pods

  ─── 420 nodes empty (available for shutdown) ───

  (Full stowage plan exported to kuberina_solution.yaml)
=> Running Inspector heatmap & validation...
cd research && uv run python inspector.py --infra testdata/irina_infra.yaml --workloads testdata/irina_workloads.yaml --solution ../solver/kuberina_solution.yaml
Loading infra: testdata/irina_infra.yaml
Loading workloads: testdata/irina_workloads.yaml
Loading solution: ../solver/kuberina_solution.yaml

--- Validation Results ---
Selector Violations: 0
Capacity Overflow (CPU): 0.00
Capacity Overflow (RAM): 0.00
Capacity Overflow (GPU): 0.00
Capacity Overflow (Storage): 0.00
Capacity Overflow (Disk Read): 27790.00
Capacity Overflow (Disk Write): 98161.00
Capacity Overflow (Net In): 296560.00
Capacity Overflow (Net Out): 313250.00
Unassigned Pods: 0
--------------------------

Heatmap dashboard generated: dashboard.html
=> Running Formal Mathematical Proof...
cd research && uv run python mathematical_proof.py
======================================================================
  KUBERINA MATHEMATICAL VERIFICATION
======================================================================
  Dataset: 186 nodes, 2714 pods
  Solution: 2714 assignments

──────────────────────────────────────────────────────────────────────
  PROOF 1: CONSTRAINT SATISFACTION (FEASIBILITY)
──────────────────────────────────────────────────────────────────────
\n  Predicate 1 — Capacity:
    ∀j ∈ N: Σᵢ xᵢⱼ · reqᵢʳ ≤ Cⱼʳ
    CPU overflow: 0.000000
    RAM overflow: 0.000000
    GPU overflow: 0.000000
    STORAGE overflow: 0.000000
    DISK_READ overflow: 0.000000
    DISK_WRITE overflow: 0.000000
    NET_IN overflow: 0.000000
    NET_OUT overflow: 0.000000
    Verdict: ✅ SATISFIED
\n  Predicate 2 — Assignment:
    ∀i ∈ P: Σⱼ xᵢⱼ = 1
    Missing pods: 0
    Verdict: ✅ SATISFIED
\n  Predicate 3 — Node Selector:
    ∀i: xᵢⱼ=1 ⟹ Selector(pᵢ) ⊆ Labels(nⱼ)
    Violations: 0
    Verdict: ✅ SATISFIED
\n  Objective — Topology Spread:
    Soft Penalty (Total Skew): 0.00
    Verdict: ✅ PERFECTLY BALANCED

  ══ FEASIBILITY: ✅ PROVEN ══

──────────────────────────────────────────────────────────────────────
  PROOF 2: OPTIMALITY BOUND (LP RELAXATION)
──────────────────────────────────────────────────────────────────────

  Total pod demand:
    CPU: 7198 cores
    RAM: 27840 GiB
    GPU: 152 units

  Max single-node capacity (after DaemonSet deduction):
    CPU: 61.75 cores
    RAM: 509.25 GiB
    GPU: 8.00 units

  ── Homogeneous LP Lower Bound (Coffman-Garey-Johnson 1978) ──
    L^CPU = ⌈7198 / 61.75⌉ = 117
    L^RAM = ⌈27840 / 509.25⌉ = 55
    L^GPU = ⌈152 / 8.00⌉ = 19
    L = max(L^r) = 117

  ── Heterogeneous Utilization Bound ──
    ρ^CPU = 0.7305
    ρ^RAM = 0.5118
    ρ^GPU = 0.6333
    L_het = max(⌈ρʳ · m⌉) = 136

  Kuberina used: 200 active nodes
  Theoretical lower bound: 136
  Approximation ratio α = 200/136 = 1.4706
  ══ α = 1.4706 — above FFD 1D guarantee, expected for multi-D ══

──────────────────────────────────────────────────────────────────────
  PROOF 3: STATISTICAL SIGNIFICANCE (MONTE CARLO)
──────────────────────────────────────────────────────────────────────

  Running 10000 random uniform trials...
    P(0 violations | random uniform) = 0.0
    Zero-violation trials: 0 / 10000
    Avg violations: 125.7 ± 5.8
    Range: [105, 146]

  Running 10000 selector-aware random trials...
    P(0 cap violations | selector-aware random) = 0.0
    Avg capacity violations: 89.1 ± 3.3
    Best random trial: 76 violations

  Kuberina: 0 violations
  Best random (selector-aware): 76 violations

  ══ P(random achieves Kuberina's result) < 1/10000 = 1.0e-04 ══
  ══ STATISTICALLY SIGNIFICANT: p < 1.0e-04 ══

======================================================================
  SUMMARY
======================================================================
  1. Feasibility:    PROVEN ✅
  2. Approx ratio:   α = 1.4706 (LB = 136 nodes)
  3. Significance:   p < 1.0e-04
======================================================================
=> Pipeline Complete.
```