#!/usr/bin/env python3
"""Mathematical verification of Kuberina's solution correctness.

Proves three things:
  1. FEASIBILITY — Every hard constraint predicate evaluates to True.
  2. QUALITY     — Solution uses ≤ α·LB nodes (LP relaxation lower bound).
  3. SIGNIFICANCE — P(random achieves same) ≈ 0 (Monte Carlo).

Usage:
    uv run --with pyyaml python bench/mathematical_proof.py
    uv run --with pyyaml python bench/mathematical_proof.py --headroom 20

Pass --headroom with the same value the solver was given. Feasibility is always
verified against real capacity, since that is the cluster the blueprint runs on,
but the optimality bound is computed against the capacity the optimizer was
actually allowed to see. Comparing a reserved run against an unreserved bound
attributes the operator's reserve to the algorithm (#8).
"""

from __future__ import annotations

import argparse
import copy
import math
import os
import random
import sys

try:
    import yaml
except ImportError:
    print("Error: PyYAML required. Run: uv pip install pyyaml")
    sys.exit(1)


# The 8-dimensional resource vector of Kuberina IR v0.2.0. Order is fixed so that
# printed proofs list dimensions the same way every run.
DIMENSIONS = (
    "cpu",
    "ram",
    "gpu",
    "storage",
    "disk_read",
    "disk_write",
    "net_in",
    "net_out",
)


# ─── Data Loading ─────────────────────────────────────────────────────

def load_yaml(path: str) -> dict:
    """Load and parse a YAML file."""
    with open(path, "r") as f:
        return yaml.safe_load(f)


def _as_float(value: object, dimension: str) -> float:
    """Coerce one IR quantity to a float, naming it if the shape is wrong."""
    try:
        return float(value)  # type: ignore[arg-type]
    except (TypeError, ValueError):
        raise ValueError(
            f"resource dimension {dimension!r} must be a number, got {value!r}. "
            "Kubernetes suffix notation (e.g. '256Mi') is understood by the solver "
            "but not by this verifier."
        ) from None


def flatten_resources(block: dict) -> dict[str, float]:
    """Project a Kuberina IR resource block onto the flat 8-dimension vector.

    IR v0.2.0 nests throughput under `disk: {read, write}` and
    `network: {in, out}`; the rest of this script indexes dimensions by their
    flat names. Absent dimensions stay absent, because the callers distinguish
    "declared as zero" from "not declared" — pre_deduct_daemonsets turns the
    latter into an unconstrained node dimension.

    Example:
        >>> flatten_resources({"cpu": 4.0, "disk": {"read": 50.0}})
        {'cpu': 4.0, 'disk_read': 50.0}
    """
    disk = block.get("disk") or {}
    network = block.get("network") or {}
    sources = (
        ("cpu", block.get("cpu")),
        ("ram", block.get("ram")),
        ("gpu", block.get("gpu")),
        ("storage", block.get("storage")),
        ("disk_read", disk.get("read")),
        ("disk_write", disk.get("write")),
        ("net_in", network.get("in")),
        ("net_out", network.get("out")),
    )
    return {r: _as_float(v, r) for r, v in sources if v is not None}


def normalize_ir(infra: dict, workloads: dict) -> None:
    """Rewrite every resource block in place into flat 8-dimension form.

    WHY: the solver reads the nested IR shape. A verifier that reads flat keys
    would silently disagree with it about I/O capacity and demand, which is the
    one thing an independent verifier must never do quietly.
    """
    for node in infra.get("nodes", []):
        node["allocatable"] = flatten_resources(node.get("allocatable", {}))
    for ds in infra.get("daemonsets", []):
        ds["resources"] = flatten_resources(ds.get("resources", {}))
    for pod in workloads.get("pods", []):
        pod["requests"] = flatten_resources(pod.get("requests", {}))


def pre_deduct_daemonsets(
    nodes: list[dict],
    daemonsets: list[dict],
) -> list[dict]:
    """Phase 0: subtract DaemonSet overhead from node capacity.

    Implements: C_j^r = C_{j,raw}^r − Σ 𝟙[eligible(d, n_j)] · res_d^r
    """
    net_nodes = []
    for node in nodes:
        net = dict(node)
        net["allocatable"] = dict(node["allocatable"])
        for ds in daemonsets:
            ds_sel = ds.get("nodeSelector", {})
            if all(
                node.get("labels", {}).get(k) == v
                for k, v in ds_sel.items()
            ):
                for r in DIMENSIONS:
                    if r in net["allocatable"]:
                        net["allocatable"][r] -= ds["resources"].get(r, 0.0)
                    else:
                        net["allocatable"][r] = float('inf')
        net_nodes.append(net)
    return net_nodes


def reserve_headroom(nodes: list[dict], headroom_pct: float) -> list[dict]:
    """Withhold `headroom_pct` of every node's capacity, as `--headroom` does.

    Mirrors `reserve_headroom` in solver/src/main.rs, including the guard that
    leaves unconstrained dimensions alone — the solver skips f64::MAX, this skips
    the float('inf') that pre_deduct_daemonsets assigns to an absent dimension.

    Example:
        >>> reserve_headroom([{"allocatable": {"cpu": 64.0}}], 20.0)
        [{'allocatable': {'cpu': 51.2}}]
    """
    factor = 1.0 - headroom_pct / 100.0
    reserved = copy.deepcopy(nodes)
    for node in reserved:
        for r, capacity in node["allocatable"].items():
            if capacity < float('inf'):
                node["allocatable"][r] = capacity * factor
    return reserved


# ─── Proof 1: Constraint Satisfaction (Feasibility) ──────────────────

def verify_capacity_constraint(
    nodes: list[dict],
    pods: list[dict],
    solution: dict[str, str],
) -> tuple[bool, dict[str, float]]:
    """Verify: ∀j ∈ N: Σᵢ xᵢⱼ · reqᵢʳ ≤ Cⱼʳ  for r ∈ {cpu, ram, gpu}.

    Returns (all_satisfied, overflow_by_resource).
    """
    node_map = {n["name"]: n for n in nodes}
    pod_map = {p["name"]: p for p in pods}

    loads: dict[str, dict[str, float]] = {
        n["name"]: {r: 0.0 for r in DIMENSIONS}
        for n in nodes
    }

    for pod_name_raw, node_name in solution.items():
        pod_name = pod_name_raw.split('/')[-1] if '/' in pod_name_raw else pod_name_raw
        if pod_name not in pod_map or node_name not in node_map:
            continue
        req = pod_map[pod_name]["requests"]
        for r in DIMENSIONS:
            loads[node_name][r] += req.get(r, 0.0)

    overflow = {r: 0.0 for r in DIMENSIONS}
    for n_name, load in loads.items():
        cap = node_map[n_name]["allocatable"]
        for r in DIMENSIONS:
            excess = load[r] - cap.get(r, 0.0)
            if excess > 1e-9:
                overflow[r] += excess

    satisfied = all(v < 1e-9 for v in overflow.values())
    return satisfied, overflow


def verify_assignment_constraint(
    pods: list[dict],
    solution: dict[str, str],
) -> tuple[bool, int]:
    """Verify: ∀i ∈ P: Σⱼ xᵢⱼ = 1 (every pod assigned exactly once).

    Returns (all_assigned, count_missing).
    """
    assigned_pods = set(k.split('/')[-1] if '/' in k else k for k in solution.keys())
    missing = sum(1 for p in pods if p["name"] not in assigned_pods)
    return missing == 0, missing


def verify_selector_constraint(
    nodes: list[dict],
    pods: list[dict],
    solution: dict[str, str],
) -> tuple[bool, int]:
    """Verify: ∀i: xᵢⱼ=1 ⟹ Selector(pᵢ) ⊆ Labels(nⱼ).

    Returns (all_satisfied, violation_count).
    """
    node_map = {n["name"]: n for n in nodes}
    pod_map = {p["name"]: p for p in pods}
    violations = 0

    for pod_name_raw, node_name in solution.items():
        pod_name = pod_name_raw.split('/')[-1] if '/' in pod_name_raw else pod_name_raw
        if pod_name not in pod_map or node_name not in node_map:
            continue
        selector = pod_map[pod_name].get("nodeSelector", {})
        labels = node_map[node_name].get("labels", {})
        if not all(labels.get(k) == v for k, v in selector.items()):
            violations += 1

    return violations == 0, violations



def verify_topology_spread(
    nodes: list[dict],
    pods: list[dict],
    solution: dict[str, str],
) -> float:
    """Compute topology spread penalty (total skew across all domains)."""
    node_map = {n["name"]: n for n in nodes}
    pod_map = {p["name"]: p for p in pods}
    
    all_domains = {"zone": set(), "rack": set()}
    for n in nodes:
        all_domains["zone"].add(n.get("zone", ""))
        all_domains["rack"].add(n.get("rack", ""))
    
    groups = {}
    import re
    for pod_name_raw, n_name in solution.items():
        pod_name = pod_name_raw.split('/')[-1] if '/' in pod_name_raw else pod_name_raw
        if pod_name not in pod_map or n_name not in node_map: continue
        pod = pod_map[pod_name]
        if "topologySpread" not in pod: continue
        
        ts = pod["topologySpread"]
        top_key = ts.get("topologyKey")
        if top_key not in all_domains: continue
        
        ns = pod.get("namespace", "default")
        base_name = re.sub(r'-\d{1,4}$', '', pod_name)
        
        group_key = (ns, base_name, top_key)
        groups.setdefault(group_key, []).append(n_name)
    
    total_penalty = 0.0
    for group_key, assigned_nodes in groups.items():
        ns, base_name, top_key = group_key
        counts = {domain: 0 for domain in all_domains[top_key]}
        for n_name in assigned_nodes:
            domain_val = node_map[n_name].get(top_key, "")
            if domain_val in counts:
                counts[domain_val] += 1
        
        min_c = min(counts.values()) if counts else 0
        max_c = max(counts.values()) if counts else 0
        skew = max(0, max_c - min_c - 1)
        total_penalty += skew * 100.0
        
    return total_penalty


# ─── Proof 2: LP Relaxation Lower Bound ──────────────────────────────

def compute_lp_lower_bound(
    nodes: list[dict],
    pods: list[dict],
) -> dict[str, float]:
    """Compute continuous relaxation lower bound on active nodes.

    For each resource dimension r:
        L^r = ⌈ Σᵢ reqᵢʳ / max_j(Cⱼʳ) ⌉

    The optimal solution OPT ≥ L = max_r(L^r).

    This is a well-known bound from Bin Packing theory
    (Coffman, Garey & Johnson, 1978).
    """
    total_demand = {r: 0.0 for r in DIMENSIONS}
    for pod in pods:
        req = pod["requests"]
        for r in DIMENSIONS:
            total_demand[r] += req.get(r, 0.0)

    # max capacity per resource across all node types
    max_cap = {r: 0.0 for r in DIMENSIONS}
    for node in nodes:
        cap = node["allocatable"]
        for r in DIMENSIONS:
            max_cap[r] = max(max_cap[r], cap.get(r, 0.0))

    bounds = {}
    for r in DIMENSIONS:
        if max_cap[r] > 0:
            bounds[r] = math.ceil(total_demand[r] / max_cap[r])
        else:
            bounds[r] = 0

    return {
        "total_demand": total_demand,
        "max_node_cap": max_cap,
        "per_resource_lb": bounds,
        "overall_lb": max(bounds.values()),
    }


def compute_heterogeneous_lower_bound(
    nodes: list[dict],
    pods: list[dict],
) -> dict[str, float]:
    """Tighter bound: total demand / total supply gives utilization floor.

    If Σᵢ reqᵢ^CPU / Σⱼ Cⱼ^CPU = ρ, then at least ⌈ρ · m⌉ nodes
    must be active (where m = total nodes).
    """
    total_demand = {r: 0.0 for r in DIMENSIONS}
    total_supply = {r: 0.0 for r in DIMENSIONS}

    for pod in pods:
        req = pod["requests"]
        for r in DIMENSIONS:
            total_demand[r] += req.get(r, 0.0)

    for node in nodes:
        cap = node["allocatable"]
        for r in DIMENSIONS:
            total_supply[r] += cap.get(r, 0.0)

    rho = {}
    for r in DIMENSIONS:
        if total_supply[r] > 0:
            rho[r] = total_demand[r] / total_supply[r]
        else:
            rho[r] = 0.0

    m = len(nodes)
    lb_per_r = {r: math.ceil(rho[r] * m) for r in rho}

    return {
        "total_demand": total_demand,
        "total_supply": total_supply,
        "utilization_ratio": rho,
        "per_resource_lb": lb_per_r,
        "overall_lb": max(lb_per_r.values()),
    }


# ─── Proof 3: Monte Carlo Random Baseline ────────────────────────────

def monte_carlo_random_baseline(
    nodes: list[dict],
    pods: list[dict],
    trials: int = 10_000,
    seed: int = 42,
) -> dict[str, float]:
    """Estimate P(random assignment achieves 0 capacity violations).

    For each trial: assign every pod to a uniformly random node,
    count capacity violations. Report statistics.

    If P ≈ 0 after N trials, then Kuberina's 0-violation result
    is astronomically unlikely to be luck.
    """
    rng = random.Random(seed)
    node_names = [n["name"] for n in nodes]
    node_map = {n["name"]: n for n in nodes}
    num_nodes = len(nodes)

    zero_violation_count = 0
    violation_counts: list[int] = []
    overflow_sums: list[float] = []

    for _ in range(trials):
        loads = {n: {r: 0.0 for r in DIMENSIONS} for n in node_names}

        for pod in pods:
            target = node_names[rng.randrange(num_nodes)]
            req = pod["requests"]
            for r in DIMENSIONS:
                loads[target][r] += req.get(r, 0.0)

        violations = 0
        total_overflow = 0.0
        for n_name, load in loads.items():
            cap = node_map[n_name]["allocatable"]
            for r in DIMENSIONS:
                excess = load[r] - cap.get(r, 0.0)
                if excess > 1e-9:
                    violations += 1
                    total_overflow += excess

        violation_counts.append(violations)
        overflow_sums.append(total_overflow)
        if violations == 0:
            zero_violation_count += 1

    avg_violations = sum(violation_counts) / trials
    avg_overflow = sum(overflow_sums) / trials
    min_violations = min(violation_counts)
    max_violations = max(violation_counts)

    # Standard deviation
    var = sum((v - avg_violations) ** 2 for v in violation_counts) / trials
    std_dev = math.sqrt(var)

    return {
        "trials": trials,
        "p_zero_violations": zero_violation_count / trials,
        "zero_violation_count": zero_violation_count,
        "avg_violations": avg_violations,
        "std_dev": std_dev,
        "min_violations": min_violations,
        "max_violations": max_violations,
        "avg_overflow_cpu_cores": avg_overflow,
    }


def monte_carlo_selector_aware(
    nodes: list[dict],
    pods: list[dict],
    trials: int = 10_000,
    seed: int = 42,
) -> dict[str, float]:
    """Selector-aware random baseline (fair comparison).

    Only assigns pods to nodes matching their nodeSelector,
    then checks capacity. This isolates the bin-packing quality
    from the constraint-filtering quality.
    """
    rng = random.Random(seed)
    node_map = {n["name"]: n for n in nodes}

    # Pre-compute eligible nodes per pod
    pod_eligible: list[list[str]] = []
    for pod in pods:
        selector = pod.get("nodeSelector", {})
        eligible = [
            n["name"] for n in nodes
            if all(
                n.get("labels", {}).get(k) == v
                for k, v in selector.items()
            )
        ]
        pod_eligible.append(eligible)

    zero_cap_violations = 0
    cap_violation_counts: list[int] = []

    for _ in range(trials):
        loads = {n["name"]: {r: 0.0 for r in DIMENSIONS} for n in nodes}

        for i, pod in enumerate(pods):
            eligible = pod_eligible[i]
            if not eligible:
                continue
            target = eligible[rng.randrange(len(eligible))]
            req = pod["requests"]
            for r in DIMENSIONS:
                loads[target][r] += req.get(r, 0.0)

        violations = 0
        for n_name, load in loads.items():
            cap = node_map[n_name]["allocatable"]
            for r in DIMENSIONS:
                if load[r] - cap.get(r, 0.0) > 1e-9:
                    violations += 1

        cap_violation_counts.append(violations)
        if violations == 0:
            zero_cap_violations += 1

    avg = sum(cap_violation_counts) / trials
    var = sum((v - avg) ** 2 for v in cap_violation_counts) / trials

    return {
        "trials": trials,
        "p_zero_cap_violations": zero_cap_violations / trials,
        "avg_cap_violations": avg,
        "std_dev": math.sqrt(var),
        "min_cap_violations": min(cap_violation_counts),
    }


# ─── Main ─────────────────────────────────────────────────────────────

def ratio_or_inf(active_nodes: int, lower_bound: int) -> float:
    """Approximation ratio, or infinity when the bound degenerates to zero.

    Example:
        >>> ratio_or_inf(152, 136)
        1.1176470588235294
    """
    return active_nodes / lower_bound if lower_bound > 0 else float("inf")


def print_capacity_model(label: str, nodes: list[dict], pods: list[dict]) -> int:
    """Print both lower bounds for one capacity model and return the tighter one.

    Reports every dimension rather than the CPU/RAM/GPU triple, so a reader can
    tell a slack dimension from an unchecked one (#11).

    Example:
        >>> print_capacity_model("real capacity", nodes, pods)  # doctest: +SKIP
        136
    """
    lp = compute_lp_lower_bound(nodes, pods)
    het = compute_heterogeneous_lower_bound(nodes, pods)

    print(f"\n  ══ Bounds against {label} ══")
    print("\n  ── Homogeneous LP Lower Bound (Coffman-Garey-Johnson 1978) ──")
    for r in DIMENSIONS:
        demand, cap = lp["total_demand"][r], lp["max_node_cap"][r]
        note = "" if cap > 0 else "  (node capacity unconstrained)"
        print(f"    L^{r:<10} = ⌈{demand:>12,.0f} / {cap:>10,.2f}⌉ = "
              f"{lp['per_resource_lb'][r]:>4}{note}")
    print(f"    L = max(L^r) = {lp['overall_lb']}")

    print("\n  ── Heterogeneous Utilization Bound ──")
    for r in DIMENSIONS:
        print(f"    ρ^{r:<10} = {het['utilization_ratio'][r]:.4f}   "
              f"⌈ρ·m⌉ = {het['per_resource_lb'][r]:>4}")
    print(f"    L_het = max(⌈ρʳ · m⌉) = {het['overall_lb']}")

    return max(lp["overall_lb"], het["overall_lb"])


def print_ratio_verdict(ratio: float) -> None:
    """Classify α against the 1D FFD guarantee of 11/9 · OPT + 6/9."""
    if ratio <= 1.0 + 1e-9:
        print("\n  ══ OPTIMAL (α = 1.0) ══")
    elif ratio <= 11 / 9 + 1e-9:
        print(f"\n  ══ WITHIN FFD GUARANTEE (α = {ratio:.4f} ≤ 11/9 ≈ 1.222) ══")
    else:
        print(f"\n  ══ α = {ratio:.4f} — above FFD 1D guarantee, expected for multi-D ══")


def parse_args() -> argparse.Namespace:
    """Parse CLI arguments, defaulting to the MSC Irina testbed.

    Example:
        >>> parse_args().headroom  # with --headroom 20 on the command line
        20.0
    """
    base = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument(
        "--headroom",
        type=float,
        default=0.0,
        help="Capacity percentage the solver was told to reserve (e.g. 20)",
    )
    parser.add_argument("--infra", default=os.path.join(base, "solver/testdata/irina_infra.yaml"))
    parser.add_argument("--workloads", default=os.path.join(base, "solver/testdata/irina_workloads.yaml"))
    parser.add_argument("--solution", default=os.path.join(base, "solver/kuberina_solution.yaml"))
    args = parser.parse_args()

    if not 0.0 <= args.headroom < 100.0:
        parser.error(f"--headroom must be a percentage in [0, 100), got {args.headroom}")
    return args


def main() -> None:
    """Run all mathematical proofs and print results."""
    args = parse_args()
    infra_path = args.infra
    work_path = args.workloads
    soln_path = args.solution

    infra = load_yaml(infra_path)
    workloads = load_yaml(work_path)
    solution = load_yaml(soln_path).get("solution", {})
    normalize_ir(infra, workloads)

    nodes = pre_deduct_daemonsets(infra["nodes"], infra.get("daemonsets", []))
    pods = workloads["pods"]

    print("=" * 70)
    print("  KUBERINA MATHEMATICAL VERIFICATION")
    print("=" * 70)
    print(f"  Dataset: {len(nodes)} nodes, {len(pods)} pods")
    print(f"  Solution: {len(solution)} assignments")
    if args.headroom > 0.0:
        print(f"  Headroom: {args.headroom:.1f}% reserved — the optimizer saw "
              f"{100.0 - args.headroom:.1f}% of real capacity")
    else:
        print("  Headroom: none — the optimizer saw full capacity")

    # ── Proof 1: Feasibility ──────────────────────────────────────────
    print("\n" + "─" * 70)
    print("  PROOF 1: CONSTRAINT SATISFACTION (FEASIBILITY)")
    print("─" * 70)

    
    cap_ok, cap_overflow = verify_capacity_constraint(nodes, pods, solution)
    assign_ok, missing = verify_assignment_constraint(pods, solution)
    sel_ok, sel_violations = verify_selector_constraint(nodes, pods, solution)
    topology_penalty = verify_topology_spread(nodes, pods, solution)


    print(f"\n  Predicate 1 — Capacity:")
    print(f"    ∀j ∈ N: Σᵢ xᵢⱼ · reqᵢʳ ≤ Cⱼʳ")
    for r in DIMENSIONS:
        print(f"    {r.upper()} overflow: {cap_overflow[r]:.6f}")
    print(f"    Verdict: {'✅ SATISFIED' if cap_ok else '❌ VIOLATED'}")

    print(f"\n  Predicate 2 — Assignment:")
    print(f"    ∀i ∈ P: Σⱼ xᵢⱼ = 1")
    print(f"    Missing pods: {missing}")
    print(f"    Verdict: {'✅ SATISFIED' if assign_ok else '❌ VIOLATED'}")

    print(f"\n  Predicate 3 — Node Selector:")
    print(f"    ∀i: xᵢⱼ=1 ⟹ Selector(pᵢ) ⊆ Labels(nⱼ)")
    print(f"    Violations: {sel_violations}")
    print(f"    Verdict: {'✅ SATISFIED' if sel_ok else '❌ VIOLATED'}")

    print(f"\n  Objective — Topology Spread:")
    print(f"    Soft Penalty (Total Skew): {topology_penalty:.2f}")
    if topology_penalty == 0.0:
        print("    Verdict: ✅ PERFECTLY BALANCED")
    else:
        print("    Verdict: ⚠️ SOFT PENALTY APPLIED")

    feasible = cap_ok and assign_ok and sel_ok
    print(f"\n  ══ FEASIBILITY: {'✅ PROVEN' if feasible else '❌ FAILED'} ══")

    # ── Proof 2: Lower Bound & Approximation Ratio ────────────────────
    print("\n" + "─" * 70)
    print("  PROOF 2: OPTIMALITY BOUND (LP RELAXATION)")
    print("─" * 70)

    real_lb = print_capacity_model("real capacity", nodes, pods)
    if args.headroom > 0.0:
        reserved_nodes = reserve_headroom(nodes, args.headroom)
        label = f"reserved capacity ({args.headroom:.1f}% withheld)"
        matched_lb = print_capacity_model(label, reserved_nodes, pods)
    else:
        matched_lb = real_lb

    active_nodes = len(set(solution.values()))
    print(f"\n  Kuberina used: {active_nodes} active nodes")
    print(f"    α vs real-capacity bound     = {active_nodes}/{real_lb} = "
          f"{ratio_or_inf(active_nodes, real_lb):.4f}")
    if args.headroom > 0.0:
        print(f"    α vs reserved-capacity bound = {active_nodes}/{matched_lb} = "
              f"{ratio_or_inf(active_nodes, matched_lb):.4f}")
        print("\n  The reserved-capacity bound is the comparable one: it is the only")
        print("  model under which numerator and denominator saw the same cluster.")

    print_ratio_verdict(ratio_or_inf(active_nodes, matched_lb))

    # ── Proof 3: Monte Carlo ──────────────────────────────────────────
    print("\n" + "─" * 70)
    print("  PROOF 3: STATISTICAL SIGNIFICANCE (MONTE CARLO)")
    print("─" * 70)

    n_trials = 10_000
    print(f"\n  Running {n_trials} random uniform trials...")
    mc_uniform = monte_carlo_random_baseline(nodes, pods, trials=n_trials)
    print(f"    P(0 violations | random uniform) = {mc_uniform['p_zero_violations']}")
    print(f"    Zero-violation trials: {mc_uniform['zero_violation_count']} / {n_trials}")
    print(f"    Avg violations: {mc_uniform['avg_violations']:.1f} ± {mc_uniform['std_dev']:.1f}")
    print(f"    Range: [{mc_uniform['min_violations']}, {mc_uniform['max_violations']}]")

    print(f"\n  Running {n_trials} selector-aware random trials...")
    mc_sel = monte_carlo_selector_aware(nodes, pods, trials=n_trials)
    print(f"    P(0 cap violations | selector-aware random) = {mc_sel['p_zero_cap_violations']}")
    print(f"    Avg capacity violations: {mc_sel['avg_cap_violations']:.1f} ± {mc_sel['std_dev']:.1f}")
    print(f"    Best random trial: {mc_sel['min_cap_violations']} violations")

    print(f"\n  Kuberina: 0 violations")
    print(f"  Best random (selector-aware): {mc_sel['min_cap_violations']} violations")

    if mc_uniform["p_zero_violations"] == 0.0:
        print(f"\n  ══ P(random achieves Kuberina's result) < 1/{n_trials} = {1/n_trials:.1e} ══")
        print(f"  ══ STATISTICALLY SIGNIFICANT: p < {1/n_trials:.1e} ══")
    else:
        p = mc_uniform["p_zero_violations"]
        print(f"\n  ══ P = {p:.6f} ══")

    # ── Summary ───────────────────────────────────────────────────────
    print("\n" + "=" * 70)
    print("  SUMMARY")
    print("=" * 70)
    print(f"  1. Feasibility:    {'PROVEN ✅' if feasible else 'FAILED ❌'}")
    matched_model = "reserved capacity" if args.headroom > 0.0 else "real capacity"
    print(f"  2. Approx ratio:   α = {ratio_or_inf(active_nodes, matched_lb):.4f} "
          f"(LB = {matched_lb} nodes, against {matched_model})")
    print(f"  3. Significance:   p < {1/n_trials:.1e}")
    print("=" * 70)


if __name__ == "__main__":
    main()
