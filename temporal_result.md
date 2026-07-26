Tạo synthetiac data vói file research/gen_irina_testdata.py:

```bash
════════════════════════════════════════════════════════════
  MSC IRINA SCALE — Resource Summary
════════════════════════════════════════════════════════════
  Nodes:          186
  Pods:           2714
  Constraints:    4632 anti-affinity + 496 affinity
────────────────────────────────────────────────────────────
  Cluster CPU:    10272 cores (net: 9853.5)
  Cluster RAM:    54912 GiB  (net: 54400.5)
  Cluster GPU:    240 units
────────────────────────────────────────────────────────────
  Pod CPU demand: 7198 cores  (73.1% fill)
  Pod RAM demand: 27840 GiB   (51.2% fill)
  Pod GPU demand: 152 units  (63.3% fill)
════════════════════════════════════════════════════════════

  Written: testdata/irina_infra.yaml
  Written: testdata/irina_workloads.yaml
```

Chạy Kuberina để  lấy kết quả 100% (cố nhét đầy 100%):

```bash
stella@stella-HP-Pavilion-15:~/workspace/kuberina$ make solver-irina
cd solver && cargo run --release -- plan \
--infra testdata/irina_infra.yaml \
--workloads testdata/irina_workloads.yaml
    Finished `release` profile [optimized] target(s) in 0.10s
     Running `target/release/kuberina plan --infra testdata/irina_infra.yaml --workloads testdata/irina_workloads.yaml`
Loaded 186 nodes, 4 daemonsets, 2714 pods, 0 groups

═══ Phase 0: DaemonSet Pre-deduction (Ballast Water) ═══
  std-000: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-001: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-002: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  <Alot here but I cut off for the sake of the context>  
  gpu-027: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-028: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-029: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
Phase 1 (FFD): seed fitness = 23153.0747
Datacenter-scale detected (2714 pods) — cranking GA to maximum
Gen  190/1000 | best=23153.0747 | stale=191 | ███░░░░░░░░░░░░░░░░░ 19%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 18418.00 | Aff: 643 | Var: 0.0374
  [Nodes] Active: 152 / 186
Early stop at generation 199 (no improvement for 200 gens)

═══ Final Blueprint (Stowage Plan) ═══
  Fitness: 23153.0747
  Time: 43.67s
  Scorecard:
    Capacity Penalty: 0
    Selector Penalty: 0
    Gang Penalty: 0
    Active Nodes: 152
    Fragmentation: 18418.00
    Affinity Violations: 643
    Utilization Variance: 0.0374

  ─── Cluster Summary (186 nodes) ───
  Active nodes: 152 / 186
  Empty nodes:  34
  Pods placed:  2714
  Avg CPU util: 88.7%

  ─── Top 5 Busiest Nodes ───
  std-068: 61.5/61.8 CPU (100%), 43 pods
  std-069: 61.5/61.8 CPU (100%), 31 pods
  std-070: 61.5/61.8 CPU (100%), 31 pods
  std-071: 61.5/61.8 CPU (100%), 31 pods
  std-072: 61.5/61.8 CPU (100%), 31 pods

  ─── Top 5 Lightest Active Nodes ───
  mem-031: 3.0/29.8 CPU (10%), 1 pods
  std-099: 9.0/61.8 CPU (15%), 9 pods
  std-000: 26.0/61.8 CPU (42%), 11 pods
  std-001: 26.0/61.8 CPU (42%), 11 pods
  std-002: 26.0/61.8 CPU (42%), 11 pods

  ─── 34 nodes empty (available for shutdown) ───

  (Full stowage plan exported to kuberina_solution.yaml)
stella@stella-HP-Pavilion-15:~/workspace/kuberina$ 
```

Còn nếu chỉ chạy với ràng buộc 80/20 pareto:

```bash
stella@stella-HP-Pavilion-15:~/workspace/kuberina$ make solver-irina-pareto-80
cd solver && cargo run --release -- plan \
--infra testdata/irina_infra.yaml \
--workloads testdata/irina_workloads.yaml \
--pareto 80
    Finished `release` profile [optimized] target(s) in 0.04s
     Running `target/release/kuberina plan --infra testdata/irina_infra.yaml --workloads testdata/irina_workloads.yaml --pareto 80`
Loaded 186 nodes, 4 daemonsets, 2714 pods, 0 groups

═══ Phase 0: DaemonSet Pre-deduction (Ballast Water) ═══
  std-000: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-001: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  <Alot here but I cut off for the sake of the context>  
  gpu-028: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-029: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)

[Pareto Mode] Capping node capacities to 80.0% for placement optimization.
Phase 1 (FFD): seed fitness = 20192.6512
Datacenter-scale detected (2714 pods) — cranking GA to maximum
Gen  190/1000 | best=20192.6512 | stale=191 | ███░░░░░░░░░░░░░░░░░ 19%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 15627.60 | Aff: 549 | Var: 0.0256
  [Nodes] Active: 182 / 186
Early stop at generation 199 (no improvement for 200 gens)

═══ Final Blueprint (Stowage Plan) ═══
  Fitness: 20192.6512
  Time: 43.71s
  Scorecard:
    Capacity Penalty: 0
    Selector Penalty: 0
    Gang Penalty: 0
    Active Nodes: 182
    Fragmentation: 15627.60
    Affinity Violations: 549
    Utilization Variance: 0.0256

  ─── Cluster Summary (186 nodes) ───
  Active nodes: 182 / 186
  Empty nodes:  4
  Pods placed:  2714
  Avg CPU util: 74.6%

  ─── Top 5 Busiest Nodes ───
  std-012: 49.0/61.8 CPU (79%), 12 pods
  std-013: 49.0/61.8 CPU (79%), 9 pods
  std-014: 49.0/61.8 CPU (79%), 9 pods
  std-015: 49.0/61.8 CPU (79%), 9 pods
  std-016: 49.0/61.8 CPU (79%), 9 pods

  ─── Top 5 Lightest Active Nodes ───
  gpu-025: 8.0/45.8 CPU (17%), 2 pods
  std-000: 20.0/61.8 CPU (32%), 9 pods
  std-001: 20.0/61.8 CPU (32%), 9 pods
  std-002: 20.0/61.8 CPU (32%), 9 pods
  std-003: 20.0/61.8 CPU (32%), 9 pods

  ─── Empty Nodes (4) ───
  gpu-026: 0 pods (available for shutdown)
  gpu-027: 0 pods (available for shutdown)
  gpu-028: 0 pods (available for shutdown)
  gpu-029: 0 pods (available for shutdown)

  (Full stowage plan exported to kuberina_solution.yaml)
```

Chạy validator (xem file python) dể check kết quả file và sinh html xem cho vui:

```bash

stella@stella-HP-Pavilion-15:~/workspace/kuberina$ make research-inspect
uv run --with pyyaml python research/inspector.py \
        --infra solver/testdata/irina_infra.yaml \
        --workloads solver/testdata/irina_workloads.yaml \
        --solution solver/kuberina_solution.yaml \
        --output kuberina_dashboard.html
Loading infra: solver/testdata/irina_infra.yaml
Loading workloads: solver/testdata/irina_workloads.yaml
Loading solution: solver/kuberina_solution.yaml

--- Validation Results ---
Selector Violations: 0
Capacity Overflow (CPU): 0.00
Capacity Overflow (RAM): 0.00
Capacity Overflow (GPU): 0.00
Unassigned Pods: 0
--------------------------

Heatmap dashboard generated: kuberina_dashboard.html
stella@stella-HP-Pavilion-15:~/workspace/kuberina$ 
```

Còn nếu chạy luôn file từ `research/mathematical_proof.py` ta có:

```bash
stella@stella-HP-Pavilion-15:~/workspace/kuberina$ make research-verify
uv run --with pyyaml python research/mathematical_proof.py
======================================================================
  KUBERINA MATHEMATICAL VERIFICATION
======================================================================
  Dataset: 186 nodes, 2714 pods
  Solution: 2714 assignments

──────────────────────────────────────────────────────────────────────
  PROOF 1: CONSTRAINT SATISFACTION (FEASIBILITY)
──────────────────────────────────────────────────────────────────────

  Predicate 1 — Capacity:
    ∀j ∈ N: Σᵢ xᵢⱼ · reqᵢʳ ≤ Cⱼʳ
    CPU overflow: 0.000000
    RAM overflow: 0.000000
    GPU overflow: 0.000000
    Verdict: ✅ SATISFIED

  Predicate 2 — Assignment:
    ∀i ∈ P: Σⱼ xᵢⱼ = 1
    Missing pods: 0
    Verdict: ✅ SATISFIED

  Predicate 3 — Node Selector:
    ∀i: xᵢⱼ=1 ⟹ Selector(pᵢ) ⊆ Labels(nⱼ)
    Violations: 0
    Verdict: ✅ SATISFIED

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

  Kuberina used: 182 active nodes
  Theoretical lower bound: 136
  Approximation ratio α = 182/136 = 1.3382
  ══ α = 1.3382 — above FFD 1D guarantee, expected for multi-D ══

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
  2. Approx ratio:   α = 1.3382 (LB = 136 nodes)
  3. Significance:   p < 1.0e-04
======================================================================
stella@stella-HP-Pavilion-15:~/workspace/kuberina$ 
```