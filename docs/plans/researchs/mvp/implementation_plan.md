# Kuberina v0.0.0 — Python MVP in `research/`

Implement a working Python MVP of the Kuberina 3-phase pipeline (Phase 0: DaemonSet pre-deduction → Phase 1: FFD warm-start → Phase 2: GA optimization with CSP forward checking), running against a sample homelab scenario, strictly derived from the project's markdown documentation.

## Source Documents Correlated

| Document | What it provides for the MVP |
|---|---|
| [PAPER_v1.md](file:///home/stella/workspace/kuberina/PAPER_v1.md) | Formal problem statement, objective function, hard/soft constraints, 3-phase pipeline description, gang scheduling model |
| [PAPER.md](file:///home/stella/workspace/kuberina/PAPER.md) | Formal notation (§3.2), $F(\mathbf{s})$ formula, hard constraint inequalities, DaemonSet pre-deduction formula |
| [DESIGN.md](file:///home/stella/workspace/kuberina/docs/DESIGN.md) | Data model structs (Node, Pod, Blueprint, PodGroup, DaemonSet), GA operators pseudocode (mutate, crossover, gangPenalty, canPlaceGang), resource model (K8s Quantity style) |
| [ga_estimation.md](file:///home/stella/workspace/kuberina/docs/ga_estimation.md) | Population sizing (128 small / 512 medium), chromosome encoding (`Assignment []int`), fitness eval complexity, convergence estimates (200-500 gens small), early stopping (50-100 gens), tournament selection size=3, mutation rate=5% |
| [AGENTS.md](file:///home/stella/workspace/kuberina/AGENTS.md) | Clean Code rules: 4-20 line functions, <500 line files, SRP, explicit types, DI, structured logging, tests for every function |
| [README.md](file:///home/stella/workspace/kuberina/README.md) | CLI interface: `kuberina plan --infra <file> --workloads <dir> --output <dir>` |

## Scope for v0.0.0

This is a **proof-of-concept** — validate that the math works, not a production tool.

### In scope
- Phase 0: DaemonSet pre-deduction ($C_j^r = C_{j,raw}^r - \sum \mathbb{1}[\text{eligible}] \cdot \text{res}_d^r$)
- Phase 1: FFD warm-start ($V_i = \alpha \cdot CPU_i + \beta \cdot RAM_i + \gamma \cdot GPU_i$, sort descending, first-fit)
- Phase 2: GA optimization (tournament selection k=3, uniform crossover + gang repair, mutation rate=5% + CSP rollback, early stopping)
- Fitness function: $F(s) = w_1 \cdot f_{nodes} + w_2 \cdot f_{frag} + w_3 \cdot f_{affinity} + w_4 \cdot f_{var} + \Phi(s)$
- Hard constraints: capacity, assignment, taint/toleration, nodeSelector, gang all-or-nothing
- Soft constraints: pod affinity (preferred), utilization balance
- Sample scenario: 3 ThinkCentre homelab (the user's dream scenario)
- CLI entry point: `python -m research.kuberina plan --infra <file> --workloads <file>`
- Console output: blueprint + fitness stats
- Tests: pytest for every module

### Out of scope
- YAML output generation (injecting NodeSelector/Affinity into manifests)
- K8s `resource.Quantity` parsing (use simple float internally)
- Island Model GA
- Topology Spread Constraints
- QoS class handling (Guaranteed vs Burstable)

## Proposed Changes

### Project Setup

#### [NEW] [pyproject.toml](file:///home/stella/workspace/kuberina/pyproject.toml)
- UV-managed Python project config
- Dependencies: `pyyaml` (YAML parsing), `pytest` (testing)
- No heavy ML/OR libs — pure Python + stdlib `random`

---

### Data Model (`research/kuberina/model/`)

Source: [DESIGN.md §4](file:///home/stella/workspace/kuberina/docs/DESIGN.md#L103-L177), [PAPER.md §3.2](file:///home/stella/workspace/kuberina/PAPER.md#L91-L175)

#### [NEW] [types.py](file:///home/stella/workspace/kuberina/research/kuberina/model/types.py)
- `ResourceVector` dataclass: `cpu: float`, `ram: float`, `gpu: float` with `fits()` and arithmetic ops
- `Node` dataclass: `name`, `labels`, `taints`, `allocatable: ResourceVector`, `zone`
- `Pod` dataclass: `name`, `namespace`, `requests: ResourceVector`, `tolerations`, `node_selector`, `affinity_targets`, `anti_affinity_targets`, `group_name`
- `PodGroup` dataclass: `name`, `pods: list[int]`, `min_members`, `node_selector`, `colocate`
- `DaemonSet` dataclass: `name`, `resources: ResourceVector`, `node_selector`, `tolerations`
- `Blueprint` dataclass: `assignment: list[int]`, `fitness: float`, `node_load: list[ResourceVector]`

---

### Phase 0 — DaemonSet Pre-deduction (`research/kuberina/phases/`)

Source: [PAPER.md §3.2 Phase 0](file:///home/stella/workspace/kuberina/PAPER.md#L108-L114), [DESIGN.md DaemonSet](file:///home/stella/workspace/kuberina/docs/DESIGN.md#L277-L335)

#### [NEW] [phase0.py](file:///home/stella/workspace/kuberina/research/kuberina/phases/phase0.py)
- `pre_deduct_daemonsets(nodes, daemonsets) -> list[Node]`: Implements $C_j^r = C_{j,raw}^r - \sum \mathbb{1}[eligible] \cdot res_d^r$
- `is_eligible(ds, node) -> bool`: Check nodeSelector + tolerations match

---

### Phase 1 — FFD Warm-Start

Source: [PAPER_v1.md §Giai đoạn 1](file:///home/stella/workspace/kuberina/PAPER_v1.md#L119-L123), [PAPER.md §4.2](file:///home/stella/workspace/kuberina/PAPER.md#L185-L196)

#### [NEW] [phase1_ffd.py](file:///home/stella/workspace/kuberina/research/kuberina/phases/phase1_ffd.py)
- `synthetic_volume(pod, alpha, beta, gamma) -> float`: Compute $V_i$
- `ffd_warmstart(pods, nodes, alpha, beta, gamma) -> Blueprint`: Sort pods by $V_i$ desc, first-fit place

---

### Phase 2 — Genetic Algorithm with CSP

Source: [DESIGN.md §GA Operators](file:///home/stella/workspace/kuberina/docs/DESIGN.md#L179-L273), [ga_estimation.md](file:///home/stella/workspace/kuberina/docs/ga_estimation.md), [PAPER.md §4.3-4.4](file:///home/stella/workspace/kuberina/PAPER.md#L198-L213)

#### [NEW] [phase2_ga.py](file:///home/stella/workspace/kuberina/research/kuberina/phases/phase2_ga.py)
- `run_ga(seed_blueprint, pods, nodes, groups, config) -> Blueprint`: Main GA loop
- `tournament_select(population, k=3) -> Blueprint`: Tournament selection
- `uniform_crossover(p1, p2) -> Blueprint`: Gene-level uniform crossover
- `repair_gangs(child, p1, p2, groups, nodes) -> Blueprint`: Gang repair from DESIGN.md crossover pseudocode
- `mutate(blueprint, pods, nodes, groups, rate) -> Blueprint`: Mutation with CSP rollback from DESIGN.md mutate pseudocode

#### [NEW] [csp.py](file:///home/stella/workspace/kuberina/research/kuberina/phases/csp.py)
- `check_capacity(assignment, pods, nodes) -> bool`: Hard constraint #1
- `check_taint_toleration(pod, node) -> bool`: Hard constraint #3
- `check_node_selector(pod, node) -> bool`: Hard constraint #4
- `can_place_pod(pod_idx, node_idx, assignment, pods, nodes) -> bool`: Combined pre-screen
- `can_place_gang(group, nodes, node_load) -> bool`: Forward checking from DESIGN.md canPlaceGang

---

### Fitness Function

Source: [PAPER.md §3.2 Objective Function](file:///home/stella/workspace/kuberina/PAPER.md#L124-L136)

#### [NEW] [fitness.py](file:///home/stella/workspace/kuberina/research/kuberina/fitness.py)
- `compute_fitness(blueprint, pods, nodes, groups, weights) -> float`: $F(s) = w_1 f_{nodes} + w_2 f_{frag} + w_3 f_{affinity} + w_4 f_{var} + \Phi(s)$
- `count_active_nodes(assignment) -> int`: $f_{nodes}$
- `compute_fragmentation(assignment, pods, nodes) -> float`: $f_{frag}$
- `compute_affinity_violations(assignment, pods) -> int`: $f_{affinity}$
- `compute_utilization_variance(assignment, pods, nodes) -> float`: $f_{var}$
- `compute_hard_penalty(blueprint, pods, nodes, groups) -> float`: $\Phi(s)$ — returns $-\infty$ or $0$

---

### Input Parser

#### [NEW] [parser.py](file:///home/stella/workspace/kuberina/research/kuberina/parser.py)
- `load_infra(path) -> tuple[list[Node], list[DaemonSet]]`: Parse cluster topology YAML
- `load_workloads(path) -> tuple[list[Pod], list[PodGroup]]`: Parse workload manifest YAML

---

### CLI Entry Point

#### [NEW] [__main__.py](file:///home/stella/workspace/kuberina/research/kuberina/__main__.py)
- `python -m research.kuberina plan --infra <file> --workloads <file>`
- Runs Phase 0 → Phase 1 → Phase 2 → prints blueprint with stats

---

### Sample Scenario

#### [NEW] [homelab_infra.yaml](file:///home/stella/workspace/kuberina/research/testdata/homelab_infra.yaml)
- 3 ThinkCentre M720q nodes: 16GB RAM, 4 cores each
- DaemonSets: kube-proxy (100m CPU, 64Mi RAM), calico-node (250m, 128Mi)

#### [NEW] [homelab_workloads.yaml](file:///home/stella/workspace/kuberina/research/testdata/homelab_workloads.yaml)
- Pi-hole, Home Assistant, Zigbee2MQTT (affinity to HA), Jellyfin, Nextcloud, Postgres (affinity to Nextcloud), Grafana, Prometheus, Mosquitto, Vaultwarden

---

### Tests

#### [NEW] [test_model.py](file:///home/stella/workspace/kuberina/research/tests/test_model.py)
#### [NEW] [test_phase0.py](file:///home/stella/workspace/kuberina/research/tests/test_phase0.py)
#### [NEW] [test_phase1.py](file:///home/stella/workspace/kuberina/research/tests/test_phase1.py)
#### [NEW] [test_phase2.py](file:///home/stella/workspace/kuberina/research/tests/test_phase2.py)
#### [NEW] [test_fitness.py](file:///home/stella/workspace/kuberina/research/tests/test_fitness.py)
#### [NEW] [test_csp.py](file:///home/stella/workspace/kuberina/research/tests/test_csp.py)
#### [NEW] [test_integration.py](file:///home/stella/workspace/kuberina/research/tests/test_integration.py)
- End-to-end: load homelab scenario → run pipeline → assert all pods placed, no capacity violations, gangs intact

## Verification Plan

### Automated Tests
```bash
cd /home/stella/workspace/kuberina
uv run pytest research/tests/ -v
```

### Manual Verification
- Run: `uv run python -m research.kuberina plan --infra research/testdata/homelab_infra.yaml --workloads research/testdata/homelab_workloads.yaml`
- Verify: all 10 homelab services placed across 3 nodes, no node over capacity, affinity pairs co-located

## Open Questions

> [!IMPORTANT]
> **Fitness weights**: The documents describe $w_1, w_2, w_3, w_4$ as tunable but don't specify default values. For the MVP I'll use: $w_1 = 10.0$ (node count, highest priority — minimize cost), $w_2 = 1.0$ (fragmentation), $w_3 = 5.0$ (affinity violations), $w_4 = 2.0$ (utilization variance). These are reasonable starting points but will need tuning with real workloads.

> [!IMPORTANT]
> **FFD alpha/beta/gamma**: Same gap — not specified. For homelab (no GPU), I'll use: $\alpha = 1.0$, $\beta = 1.0$, $\gamma = 10.0$ (GPU scarcity dominates when present). With no GPU pods, this degenerates to CPU+RAM weighting.
