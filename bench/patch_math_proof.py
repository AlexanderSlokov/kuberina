import re

with open("bench/mathematical_proof.py", "r") as f:
    content = f.read()

# 1. pre_deduct_daemonsets
content = re.sub(
    r'net\["allocatable"\]\["cpu"\] -= ds\["resources"\].*?\n.*?net\["allocatable"\]\["gpu"\].*?\n',
    '                for r in ("cpu", "ram", "gpu", "storage", "disk_read", "disk_write", "net_in", "net_out"):\n                    net["allocatable"][r] -= ds["resources"].get(r, 0.0)\n',
    content,
    flags=re.DOTALL
)

# 2. loads initialization
content = re.sub(
    r'\{"cpu": 0\.0, "ram": 0\.0, "gpu": 0\.0\}',
    '{r: 0.0 for r in ("cpu", "ram", "gpu", "storage", "disk_read", "disk_write", "net_in", "net_out")}',
    content
)

# 3. For loop over cpu, ram, gpu
content = re.sub(
    r'\("cpu", "ram", "gpu"\)',
    '("cpu", "ram", "gpu", "storage", "disk_read", "disk_write", "net_in", "net_out")',
    content
)

# 4. load accumulation
content = re.sub(
    r'loads\[.*?\]\["cpu"\] \+= req\.get\("cpu", 0\.0\)\n\s+loads\[.*?\]\["ram"\].*?\n\s+loads\[.*?\]\["gpu"\].*?\n',
    'for r in ("cpu", "ram", "gpu", "storage", "disk_read", "disk_write", "net_in", "net_out"):\n            loads[target if "target" in locals() or "target" in globals() else node_name][r] += req.get(r, 0.0)\n',
    content
)
content = content.replace('loads[target if "target" in locals() or "target" in globals() else node_name]', 'loads[target if "target" in vars() else node_name]')
content = re.sub(
    r'loads\[node_name\]\["cpu"\] \+= req\.get\("cpu", 0\.0\)\n\s+loads\[node_name\]\["ram"\].*?\n\s+loads\[node_name\]\["gpu"\].*?\n',
    'for r in ("cpu", "ram", "gpu", "storage", "disk_read", "disk_write", "net_in", "net_out"):\n            loads[node_name][r] += req.get(r, 0.0)\n',
    content
)
content = re.sub(
    r'total_demand\["cpu"\] \+= req\.get\("cpu", 0\.0\)\n\s+total_demand\["ram"\].*?\n\s+total_demand\["gpu"\].*?\n',
    'for r in ("cpu", "ram", "gpu", "storage", "disk_read", "disk_write", "net_in", "net_out"):\n            total_demand[r] += req.get(r, 0.0)\n',
    content
)

# Topology Spread function
topology_spread_func = """
def verify_topology_spread(
    nodes: list[dict],
    pods: list[dict],
    solution: dict[str, str],
) -> float:
    \"\"\"Compute topology spread penalty (total skew across all domains).\"\"\"
    node_map = {n["name"]: n for n in nodes}
    pod_map = {p["name"]: p for p in pods}
    
    all_domains = {"zone": set(), "rack": set()}
    for n in nodes:
        all_domains["zone"].add(n.get("zone", ""))
        all_domains["rack"].add(n.get("rack", ""))
    
    groups = {}
    import re
    for p_name, n_name in solution.items():
        if p_name not in pod_map or n_name not in node_map: continue
        pod = pod_map[p_name]
        if "topologySpread" not in pod: continue
        
        ts = pod["topologySpread"]
        top_key = ts.get("topologyKey")
        if top_key not in all_domains: continue
        
        ns = pod.get("namespace", "default")
        base_name = re.sub(r'-\d{1,4}$', '', p_name)
        
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

"""
content = content.replace("# ─── Proof 2", topology_spread_func + "\n# ─── Proof 2")

# Main output updates
main_updates = """
    cap_ok, cap_overflow = verify_capacity_constraint(nodes, pods, solution)
    assign_ok, missing = verify_assignment_constraint(pods, solution)
    sel_ok, sel_violations = verify_selector_constraint(nodes, pods, solution)
    topology_penalty = verify_topology_spread(nodes, pods, solution)
"""
content = content.replace(
    "cap_ok, cap_overflow = verify_capacity_constraint(nodes, pods, solution)\n    assign_ok, missing = verify_assignment_constraint(pods, solution)\n    sel_ok, sel_violations = verify_selector_constraint(nodes, pods, solution)",
    main_updates
)

main_print_updates = """
    print(f"\\n  Predicate 1 — Capacity:")
    print(f"    ∀j ∈ N: Σᵢ xᵢⱼ · reqᵢʳ ≤ Cⱼʳ")
    for r in ("cpu", "ram", "gpu", "storage", "disk_read", "disk_write", "net_in", "net_out"):
        print(f"    {r.upper()} overflow: {cap_overflow[r]:.6f}")
    print(f"    Verdict: {'✅ SATISFIED' if cap_ok else '❌ VIOLATED'}")

    print(f"\\n  Predicate 2 — Assignment:")
    print(f"    ∀i ∈ P: Σⱼ xᵢⱼ = 1")
    print(f"    Missing pods: {missing}")
    print(f"    Verdict: {'✅ SATISFIED' if assign_ok else '❌ VIOLATED'}")

    print(f"\\n  Predicate 3 — Node Selector:")
    print(f"    ∀i: xᵢⱼ=1 ⟹ Selector(pᵢ) ⊆ Labels(nⱼ)")
    print(f"    Violations: {sel_violations}")
    print(f"    Verdict: {'✅ SATISFIED' if sel_ok else '❌ VIOLATED'}")

    print(f"\\n  Objective — Topology Spread:")
    print(f"    Soft Penalty (Total Skew): {topology_penalty:.2f}")
    if topology_penalty == 0.0:
        print("    Verdict: ✅ PERFECTLY BALANCED")
    else:
        print("    Verdict: ⚠️ SOFT PENALTY APPLIED")
"""
content = re.sub(
    r'print\(f"\\n  Predicate 1 — Capacity:".*?print\(f"    Verdict: \{\'✅ SATISFIED\' if sel_ok else \'❌ VIOLATED\'\}"\)',
    main_print_updates.strip(),
    content,
    flags=re.DOTALL
)

with open("bench/mathematical_proof.py", "w") as f:
    f.write(content)
