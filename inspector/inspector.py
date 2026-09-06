#!/usr/bin/env python3
"""Kuberina Inspector
Independent Validator and Interactive HTML Heatmap Renderer for Kuberina solutions.
"""

import argparse
import json
import os
import sys

try:
    import yaml
except ImportError:
    print("Error: PyYAML is not installed. Run: uv pip install pyyaml")
    sys.exit(1)


def parse_args():
    parser = argparse.ArgumentParser(description="Kuberina Inspector")
    parser.add_argument("--infra", default="solver/testdata/irina_infra.yaml", help="Path to infra YAML")
    parser.add_argument("--workloads", default="solver/testdata/irina_workloads.yaml", help="Path to workloads YAML")
    parser.add_argument("--solution", default="solver/kuberina_solution.yaml", help="Path to solution YAML")
    parser.add_argument("--output", default="dashboard.html", help="Path to output HTML heatmap")
    return parser.parse_args()


def load_yaml(path: str):
    if not os.path.exists(path):
        print(f"Error: File not found: {path}")
        sys.exit(1)
    with open(path, "r") as f:
        return yaml.safe_load(f)


def _as_float(value, dimension):
    """Coerce one IR quantity to a float, naming it if the shape is wrong."""
    try:
        return float(value)
    except (TypeError, ValueError):
        raise ValueError(
            f"resource dimension {dimension!r} must be a number, got {value!r}. "
            "Kubernetes suffix notation (e.g. '256Mi') is understood by the solver "
            "but not by this validator."
        ) from None


def flatten_resources(block):
    """Project a Kuberina IR resource block onto the flat 8-dimension vector.

    IR v0.2.0 nests throughput under `disk: {read, write}` and
    `network: {in, out}`. Absent dimensions stay absent so that
    pre_deduct_daemonsets can tell "declared zero" from "not declared".

    Example:
        >>> flatten_resources({"cpu": 4.0, "network": {"in": 500.0}})
        {'cpu': 4.0, 'net_in': 500.0}
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


def normalize_ir(infra, workloads):
    """Rewrite every resource block in place into flat 8-dimension form.

    WHY: the solver reads the nested IR shape. A validator reading flat keys
    would disagree with it about I/O capacity and demand without saying so,
    which defeats the point of validating independently.
    """
    for node in infra.get("nodes", []):
        node["allocatable"] = flatten_resources(node.get("allocatable", {}))
    for ds in infra.get("daemonsets", []):
        ds["resources"] = flatten_resources(ds.get("resources", {}))
    for pod in workloads.get("pods", []):
        pod["requests"] = flatten_resources(pod.get("requests", {}))


def pre_deduct_daemonsets(nodes, daemonsets):
    """Subtract DaemonSet resources from node allocatable capacity (Phase 0)."""
    net_nodes = []
    for node in nodes:
        net_node = dict(node)
        net_node["allocatable"] = dict(node["allocatable"])
        for ds in daemonsets:
            # Check ds nodeSelector
            ds_sel = ds.get("nodeSelector", {})
            matches = True
            for k, v in ds_sel.items():
                if node.get("labels", {}).get(k) != v:
                    matches = False
                    break
            if matches:
                for r in ("cpu", "ram", "gpu", "storage", "disk_read", "disk_write", "net_in", "net_out"):
                    if r in net_node["allocatable"]:
                        net_node["allocatable"][r] -= ds["resources"].get(r, 0.0)
                    else:
                        net_node["allocatable"][r] = float('inf')
        net_nodes.append(net_node)
    return net_nodes


def validate_solution(nodes, pods, solution):
    node_map = {n["name"]: n for n in nodes}
    pod_map = {p["name"]: p for p in pods}
    
    node_loads = {n["name"]: {"cpu": 0.0, "ram": 0.0, "gpu": 0.0, "storage": 0.0, "disk_read": 0.0, "disk_write": 0.0, "net_in": 0.0, "net_out": 0.0, "pods": []} for n in nodes}
    
    selector_violations = 0
    unassigned_pods = 0
    
    for pod_name_raw, node_name in solution.items():
        pod_name = pod_name_raw.split('/')[-1] if '/' in pod_name_raw else pod_name_raw
        if pod_name not in pod_map:
            continue
        pod = pod_map[pod_name]
        
        if node_name not in node_map:
            unassigned_pods += 1
            continue
            
        node = node_map[node_name]
        node_loads[node_name]["pods"].append(pod_name)
        
        # Add to load
        req = pod.get("requests", {})
        node_loads[node_name]["cpu"] += req.get("cpu", 0.0)
        node_loads[node_name]["ram"] += req.get("ram", 0.0)
        node_loads[node_name]["gpu"] += req.get("gpu", 0.0)
        node_loads[node_name]["storage"] += req.get("storage", 0.0)
        node_loads[node_name]["disk_read"] += req.get("disk_read", 0.0)
        node_loads[node_name]["disk_write"] += req.get("disk_write", 0.0)
        node_loads[node_name]["net_in"] += req.get("net_in", 0.0)
        node_loads[node_name]["net_out"] += req.get("net_out", 0.0)
        
        # Check selector
        sel = pod.get("nodeSelector", {})
        for k, v in sel.items():
            if node.get("labels", {}).get(k) != v:
                selector_violations += 1
                break
                
    capacity_overflow = {"cpu": 0.0, "ram": 0.0, "gpu": 0.0, "storage": 0.0, "disk_read": 0.0, "disk_write": 0.0, "net_in": 0.0, "net_out": 0.0}
    for n_name, load in node_loads.items():
        cap = node_map[n_name]["allocatable"]
        for res in ["cpu", "ram", "gpu", "storage", "disk_read", "disk_write", "net_in", "net_out"]:
            if load[res] > cap.get(res, float('inf')): 
                capacity_overflow[res] += load[res] - cap.get(res, float('inf'))
        
    return {
        "node_loads": node_loads,
        "selector_violations": selector_violations,
        "unassigned_pods": unassigned_pods,
        "capacity_overflow": capacity_overflow,
    }


def render_html(nodes, validation, output_path):
    html_template = """<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Kuberina Inspector Dashboard</title>
    <style>
        :root {{
            --bg-color: #0f172a;
            --text-color: #f8fafc;
            --panel-bg: rgba(30, 41, 59, 0.7);
            --border-color: rgba(255, 255, 255, 0.1);
        }}
        body {{
            font-family: 'Inter', system-ui, -apple-system, sans-serif;
            background-color: var(--bg-color);
            color: var(--text-color);
            margin: 0;
            padding: 2rem;
            line-height: 1.5;
        }}
        h1 {{
            font-weight: 600;
            margin-bottom: 0.5rem;
            background: linear-gradient(90deg, #38bdf8, #818cf8);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
        }}
        .summary-bar {{
            display: flex;
            gap: 2rem;
            background: var(--panel-bg);
            padding: 1.5rem;
            border-radius: 12px;
            backdrop-filter: blur(10px);
            border: 1px solid var(--border-color);
            margin-bottom: 2rem;
        }}
        .stat {{
            display: flex;
            flex-direction: column;
        }}
        .stat-label {{
            font-size: 0.875rem;
            color: #94a3b8;
            text-transform: uppercase;
            letter-spacing: 0.05em;
        }}
        .stat-value {{
            font-size: 1.5rem;
            font-weight: 600;
        }}
        .error-value {{ color: #ef4444; }}
        .success-value {{ color: #10b981; }}
        
        .grid {{
            display: grid;
            grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
            gap: 1rem;
        }}
        .node-card {{
            position: relative;
            background: rgba(30, 41, 59, 0.4);
            border: 1px solid var(--border-color);
            border-radius: 8px;
            padding: 1rem;
            transition: all 0.2s ease;
            cursor: pointer;
            display: flex;
            flex-direction: column;
            gap: 0.5rem;
        }}
        .node-card:hover {{
            transform: translateY(-2px);
            box-shadow: 0 4px 12px rgba(0,0,0,0.2);
            border-color: rgba(255,255,255,0.3);
            z-index: 10;
        }}
        .node-name {{
            font-size: 0.9rem;
            font-weight: 600;
            margin: 0;
        }}
        .node-tier {{
            font-size: 0.75rem;
            color: #cbd5e1;
            background: rgba(255,255,255,0.1);
            padding: 2px 6px;
            border-radius: 4px;
            align-self: flex-start;
        }}
        .progress-container {{
            width: 100%;
            background: rgba(0,0,0,0.3);
            border-radius: 4px;
            height: 6px;
            overflow: hidden;
            margin-top: 4px;
        }}
        .progress-bar {{
            height: 100%;
            border-radius: 4px;
        }}
        .tooltip {{
            display: none;
            position: absolute;
            top: 100%;
            left: 50%;
            transform: translateX(-50%);
            margin-top: 10px;
            background: #1e293b;
            border: 1px solid var(--border-color);
            border-radius: 8px;
            padding: 1rem;
            min-width: 250px;
            box-shadow: 0 10px 25px rgba(0,0,0,0.5);
            z-index: 100;
        }}
        .node-card:hover .tooltip {{
            display: block;
        }}
        .tooltip-row {{
            display: flex;
            justify-content: space-between;
            font-size: 0.85rem;
            margin-bottom: 4px;
            border-bottom: 1px solid rgba(255,255,255,0.05);
            padding-bottom: 4px;
        }}
        .pod-list {{
            margin-top: 0.5rem;
            max-height: 150px;
            overflow-y: auto;
            font-size: 0.75rem;
            color: #94a3b8;
            background: rgba(0,0,0,0.2);
            padding: 0.5rem;
            border-radius: 4px;
        }}
        .pod-item {{
            margin-bottom: 2px;
            white-space: nowrap;
            overflow: hidden;
            text-overflow: ellipsis;
        }}
    </style>
</head>
<body>
    <h1>Kuberina Inspector</h1>
    
    <div class="summary-bar">
        <div class="stat">
            <span class="stat-label">Total Nodes</span>
            <span class="stat-value">{total_nodes}</span>
        </div>
        <div class="stat">
            <span class="stat-label">Total Pods</span>
            <span class="stat-value">{total_pods}</span>
        </div>
        <div class="stat">
            <span class="stat-label">CPU Overflow</span>
            <span class="stat-value {cpu_class}">{cpu_overflow:.2f}</span>
        </div>
        <div class="stat">
            <span class="stat-label">Selector Violations</span>
            <span class="stat-value {sel_class}">{sel_violations}</span>
        </div>
        <div class="stat">
            <span class="stat-label">Unassigned Pods</span>
            <span class="stat-value {un_class}">{un_pods}</span>
        </div>
    </div>
    
    <div class="grid">
        {grid_html}
    </div>
</body>
</html>"""

    node_loads = validation["node_loads"]
    total_pods = sum(len(n["pods"]) for n in node_loads.values())
    cpu_overflow = validation["capacity_overflow"]["cpu"]
    sel_violations = validation["selector_violations"]
    
    def get_color(pct):
        if pct < 50: return "#10b981" # Green
        if pct < 85: return "#f59e0b" # Yellow
        if pct < 100: return "#f97316" # Orange
        return "#ef4444" # Red
        
    grid_html = ""
    for node in nodes:
        name = node["name"]
        load = node_loads.get(name, {"cpu": 0, "ram": 0, "gpu": 0, "pods": []})
        cap = node["allocatable"]
        
        cpu_pct = (load["cpu"] / cap["cpu"] * 100) if cap["cpu"] > 0 else 0
        ram_pct = (load["ram"] / cap["ram"] * 100) if cap["ram"] > 0 else 0
        
        tier = node.get("labels", {}).get("tier", "unknown")
        cpu_color = get_color(cpu_pct)
        ram_color = get_color(ram_pct)
        
        pod_items = "".join(f"<div class='pod-item'>• {p}</div>" for p in load["pods"])
        
        bg_opacity = min(0.8, max(0.1, cpu_pct / 100))
        bg_color = f"rgba({int(cpu_color[1:3], 16)}, {int(cpu_color[3:5], 16)}, {int(cpu_color[5:7], 16)}, {bg_opacity * 0.4})"
        
        grid_html += f"""
        <div class="node-card" style="background: {bg_color}; border-top: 3px solid {cpu_color}">
            <h3 class="node-name">{name}</h3>
            <span class="node-tier">{tier}</span>
            
            <div style="font-size: 0.75rem; margin-top: 8px;">CPU: {load["cpu"]:.1f} / {cap["cpu"]:.1f} ({cpu_pct:.0f}%)</div>
            <div class="progress-container">
                <div class="progress-bar" style="width: {min(100, cpu_pct)}%; background-color: {cpu_color}"></div>
            </div>
            
            <div class="tooltip">
                <div class="tooltip-row"><span>Node</span><span>{name} ({tier})</span></div>
                <div class="tooltip-row"><span>CPU</span><span style="color: {cpu_color}">{load["cpu"]:.1f} / {cap["cpu"]:.1f}</span></div>
                <div class="tooltip-row"><span>RAM</span><span style="color: {ram_color}">{load["ram"]:.1f} / {cap["ram"]:.1f} GiB</span></div>
                <div class="tooltip-row"><span>GPU</span><span>{load["gpu"]:.0f} / {cap["gpu"]:.0f}</span></div>
                <div class="tooltip-row"><span>Pods Count</span><span>{len(load["pods"])}</span></div>
                
                <div class="pod-list">
                    {pod_items if pod_items else "<i>Empty</i>"}
                </div>
            </div>
        </div>
        """

    final_html = html_template.format(
        total_nodes=len(nodes),
        total_pods=total_pods,
        cpu_overflow=cpu_overflow,
        cpu_class="error-value" if cpu_overflow > 0 else "success-value",
        sel_violations=sel_violations,
        sel_class="error-value" if sel_violations > 0 else "success-value",
        un_pods=validation["unassigned_pods"],
        un_class="error-value" if validation["unassigned_pods"] > 0 else "success-value",
        grid_html=grid_html
    )
    
    with open(output_path, "w") as f:
        f.write(final_html)
    print(f"Heatmap dashboard generated: {output_path}")


def main():
    args = parse_args()
    
    print(f"Loading infra: {args.infra}")
    infra_data = load_yaml(args.infra)
    
    print(f"Loading workloads: {args.workloads}")
    workloads_data = load_yaml(args.workloads)
    
    print(f"Loading solution: {args.solution}")
    solution_data = load_yaml(args.solution)
    solution_map = solution_data.get("solution", {})
    
    normalize_ir(infra_data, workloads_data)

    # Pre-deduct daemonsets
    nodes = pre_deduct_daemonsets(infra_data["nodes"], infra_data.get("daemonsets", []))
    pods = workloads_data["pods"]
    
    # Validate
    validation = validate_solution(nodes, pods, solution_map)
    
    print("\n--- Validation Results ---")
    print(f"Selector Violations: {validation['selector_violations']}")
    print(f"Capacity Overflow (CPU): {validation['capacity_overflow']['cpu']:.2f}")
    print(f"Capacity Overflow (RAM): {validation['capacity_overflow']['ram']:.2f}")
    print(f"Capacity Overflow (GPU): {validation['capacity_overflow']['gpu']:.2f}")
    print(f"Capacity Overflow (Storage): {validation['capacity_overflow']['storage']:.2f}")
    print(f"Capacity Overflow (Disk Read): {validation['capacity_overflow']['disk_read']:.2f}")
    print(f"Capacity Overflow (Disk Write): {validation['capacity_overflow']['disk_write']:.2f}")
    print(f"Capacity Overflow (Net In): {validation['capacity_overflow']['net_in']:.2f}")
    print(f"Capacity Overflow (Net Out): {validation['capacity_overflow']['net_out']:.2f}")
    print(f"Unassigned Pods: {validation['unassigned_pods']}")
    print("--------------------------\n")
    
    # Render
    render_html(nodes, validation, args.output)


if __name__ == "__main__":
    main()
