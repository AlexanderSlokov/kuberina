"""Kuberina CLI — pre-deployment scheduling optimizer.

Usage:
    python -m kuberina plan --infra <file> --workloads <file>

Pipeline: Phase 0 (DaemonSet pre-deduction) → Phase 1 (FFD) → Phase 2 (GA+CSP)
"""

from __future__ import annotations

import argparse
import logging
import sys
import time

from kuberina.fitness import compute_fitness
from kuberina.model.types import (
    FFDWeights,
    FitnessWeights,
    GAConfig,
)
from kuberina.parser import load_infra, load_workloads
from kuberina.phases.phase0 import pre_deduct_daemonsets
from kuberina.phases.phase1_ffd import ffd_warmstart
from kuberina.phases.phase2_ga import run_ga


def main() -> None:
    """Entry point for the Kuberina CLI."""
    args = _parse_args()

    logging.basicConfig(
        level=logging.INFO,
        format="%(asctime)s [%(levelname)s] %(message)s",
    )

    if args.command == "plan":
        _run_plan(args)
    else:
        print(f"Unknown command: {args.command}", file=sys.stderr)
        sys.exit(1)


def _parse_args() -> argparse.Namespace:
    """Parse CLI arguments matching README.md interface."""
    parser = argparse.ArgumentParser(
        prog="kuberina",
        description="Maritime stowage-inspired K8s scheduling optimizer",
    )
    sub = parser.add_subparsers(dest="command")

    plan = sub.add_parser("plan", help="Generate optimized scheduling blueprint")
    plan.add_argument("--infra", required=True, help="Cluster topology YAML")
    plan.add_argument("--workloads", required=True, help="Workload manifests YAML")

    return parser.parse_args()


def _run_plan(args: argparse.Namespace) -> None:
    """Execute the full 3-phase pipeline and print results."""
    logger = logging.getLogger("kuberina")
    start = time.perf_counter()

    # Load inputs
    raw_nodes, daemon_sets = load_infra(args.infra)
    pods, groups = load_workloads(args.workloads)

    logger.info("Loaded %d nodes, %d daemonsets, %d pods, %d groups",
                len(raw_nodes), len(daemon_sets), len(pods), len(groups))

    # Phase 0: DaemonSet pre-deduction (ballast water)
    nodes = pre_deduct_daemonsets(raw_nodes, daemon_sets)
    _print_phase0_summary(raw_nodes, nodes, daemon_sets)

    # Phase 1: FFD warm-start (stow heaviest containers first)
    ffd_weights = FFDWeights()
    seed = ffd_warmstart(pods, nodes, ffd_weights)
    fitness_weights = FitnessWeights()
    seed.fitness = compute_fitness(seed, pods, nodes, groups, fitness_weights)
    logger.info("Phase 1 (FFD): seed fitness = %.4f", seed.fitness)

    # Phase 2: GA optimization (evolutionary stowage planning)
    # WHY: auto-scale GA params based on problem size (ga_estimation.md §3).
    # Small clusters converge quickly; datacenter-scale needs brute force.
    ga_config = _select_ga_config(len(pods))
    best = run_ga(seed, pods, nodes, groups, ga_config, fitness_weights)

    elapsed = time.perf_counter() - start
    _print_blueprint(best, pods, nodes, elapsed)


def _select_ga_config(num_pods: int) -> GAConfig:
    """Auto-scale GA parameters based on problem size.

    Tiers from ga_estimation.md §3:
    - Small  (<100 pods):  quick convergence, population 128
    - Medium (100-500):    standard GA, population 256
    - Large  (>500 pods):  datacenter-scale brute force, population 1024

    Example:
        >>> config = _select_ga_config(2714)
        >>> config.population_size
        1024
    """
    if num_pods > 500:
        logger = logging.getLogger("kuberina")
        logger.info("Datacenter-scale detected (%d pods) — cranking GA to maximum", num_pods)
        return GAConfig(
            population_size=1024,
            tournament_size=5,
            mutation_rate=0.03,
            crossover_rate=0.85,
            max_generations=1000,
            early_stop_generations=200,
        )

    if num_pods > 100:
        return GAConfig(
            population_size=256,
            max_generations=500,
            early_stop_generations=100,
        )

    return GAConfig()


def _print_phase0_summary(
    raw_nodes: list,
    deducted_nodes: list,
    daemon_sets: list,
) -> None:
    """Show DaemonSet pre-deduction results."""
    print("\n═══ Phase 0: DaemonSet Pre-deduction (Ballast Water) ═══")
    for raw, ded in zip(raw_nodes, deducted_nodes):
        overhead_cpu = raw.allocatable.cpu - ded.allocatable.cpu
        overhead_ram = raw.allocatable.ram - ded.allocatable.ram
        print(f"  {ded.name}: {raw.allocatable.cpu:.1f} → {ded.allocatable.cpu:.2f} CPU, "
              f"{raw.allocatable.ram:.1f} → {ded.allocatable.ram:.3f} GiB RAM "
              f"(-{overhead_cpu:.2f} CPU, -{overhead_ram:.3f} GiB overhead)")


def _print_blueprint(
    blueprint,
    pods: list,
    nodes: list,
    elapsed: float,
) -> None:
    """Print the final optimized blueprint to stdout."""
    print("\n═══ Final Blueprint (Stowage Plan) ═══")
    print(f"  Fitness: {blueprint.fitness:.4f}")
    print(f"  Time: {elapsed:.2f}s")
    print()

    num_nodes = len(nodes)
    node_pods: dict[int, list[str]] = {}
    for pod_idx, node_idx in enumerate(blueprint.assignment):
        node_pods.setdefault(node_idx, []).append(pods[pod_idx].name)

    if num_nodes <= 20:
        _print_all_nodes(nodes, blueprint, node_pods)
    else:
        _print_summary_mode(nodes, blueprint, node_pods)


def _print_all_nodes(nodes: list, blueprint, node_pods: dict) -> None:
    """Print every node with its pods (for small clusters)."""
    for node_idx in range(len(nodes)):
        node = nodes[node_idx]
        load = blueprint.node_load[node_idx]
        pod_names = node_pods.get(node_idx, [])

        cpu_pct = (load.cpu / node.allocatable.cpu * 100) if node.allocatable.cpu > 0 else 0
        ram_pct = (load.ram / node.allocatable.ram * 100) if node.allocatable.ram > 0 else 0

        print(f"  {node.name}:")
        for name in pod_names:
            print(f"    - {name}")
        print(f"    CPU: {load.cpu:.2f}/{node.allocatable.cpu:.2f} ({cpu_pct:.0f}%)")
        print(f"    RAM: {load.ram:.3f}/{node.allocatable.ram:.3f} GiB ({ram_pct:.0f}%)")
        print()


def _print_summary_mode(nodes: list, blueprint, node_pods: dict) -> None:
    """Print aggregate stats + top/bottom nodes (for datacenter scale)."""
    # WHY: printing 150 nodes with 2700 pods individually would
    # produce 3000+ lines of unreadable output.
    num_nodes = len(nodes)

    node_stats = []
    active_count = 0
    empty_count = 0
    for j in range(num_nodes):
        load = blueprint.node_load[j]
        cap = nodes[j].allocatable
        cpu_pct = (load.cpu / cap.cpu * 100) if cap.cpu > 0 else 0
        pod_count = len(node_pods.get(j, []))
        if pod_count > 0:
            active_count += 1
        else:
            empty_count += 1
        node_stats.append((j, nodes[j].name, cpu_pct, pod_count, load.cpu, cap.cpu))

    total_pods = sum(s[3] for s in node_stats)
    avg_cpu = sum(s[2] for s in node_stats if s[3] > 0) / max(active_count, 1)

    print(f"  ─── Cluster Summary ({num_nodes} nodes) ───")
    print(f"  Active nodes: {active_count} / {num_nodes}")
    print(f"  Empty nodes:  {empty_count}")
    print(f"  Pods placed:  {total_pods}")
    print(f"  Avg CPU util: {avg_cpu:.1f}%")
    print()

    # Top 5 busiest
    by_cpu = sorted(node_stats, key=lambda s: s[2], reverse=True)
    print("  ─── Top 5 Busiest Nodes ───")
    for _, name, cpu_pct, pod_count, load_cpu, cap_cpu in by_cpu[:5]:
        print(f"  {name}: {load_cpu:.1f}/{cap_cpu:.1f} CPU ({cpu_pct:.0f}%), {pod_count} pods")

    # Bottom 5 least loaded (with at least 1 pod)
    active_stats = [s for s in node_stats if s[3] > 0]
    by_cpu_asc = sorted(active_stats, key=lambda s: s[2])
    print("\n  ─── Top 5 Lightest Active Nodes ───")
    for _, name, cpu_pct, pod_count, load_cpu, cap_cpu in by_cpu_asc[:5]:
        print(f"  {name}: {load_cpu:.1f}/{cap_cpu:.1f} CPU ({cpu_pct:.0f}%), {pod_count} pods")

    # Empty nodes
    if empty_count > 0 and empty_count <= 10:
        print(f"\n  ─── Empty Nodes ({empty_count}) ───")
        for s in node_stats:
            if s[3] == 0:
                print(f"  {s[1]}: 0 pods (available for shutdown)")
    elif empty_count > 10:
        print(f"\n  ─── {empty_count} nodes empty (available for shutdown) ───")
    print()


if __name__ == "__main__":
    main()
