"""Integration test: end-to-end homelab scenario.

Load testdata → Phase 0 → Phase 1 → Phase 2 → verify constraints.
"""

from __future__ import annotations

from kuberina.fitness import (
    compute_affinity_violations,
    compute_fitness,
)
from kuberina.model.types import (
    FFDWeights,
    FitnessWeights,
    GAConfig,
)
from kuberina.parser import load_infra, load_workloads
from kuberina.phases.csp import compute_capacity_overflow
from kuberina.phases.phase0 import pre_deduct_daemonsets
from kuberina.phases.phase1_ffd import ffd_warmstart
from kuberina.phases.phase2_ga import run_ga


INFRA_PATH = "testdata/homelab_infra.yaml"
WORKLOADS_PATH = "testdata/homelab_workloads.yaml"


def test_full_pipeline_homelab() -> None:
    """Run the complete 3-phase pipeline on the homelab scenario.

    Assertions:
    1. All 10 pods are assigned to valid nodes (0, 1, or 2)
    2. No node exceeds capacity (hard constraint #1)
    3. Home Assistant + Zigbee2MQTT are on thinkcentre-beta (USB dongle)
    4. GA improves upon FFD seed fitness
    """
    raw_nodes, daemon_sets = load_infra(INFRA_PATH)
    pods, groups = load_workloads(WORKLOADS_PATH)

    # Phase 0
    nodes = pre_deduct_daemonsets(raw_nodes, daemon_sets)

    # WHY: verify DaemonSet overhead was subtracted
    assert nodes[0].allocatable.cpu < raw_nodes[0].allocatable.cpu
    assert nodes[0].allocatable.ram < raw_nodes[0].allocatable.ram

    # Phase 1: FFD
    seed = ffd_warmstart(pods, nodes, FFDWeights())
    weights = FitnessWeights()
    seed.fitness = compute_fitness(seed, pods, nodes, groups, weights)

    # All pods assigned to valid nodes
    assert all(0 <= idx < len(nodes) for idx in seed.assignment)

    # Phase 2: GA
    config = GAConfig(
        population_size=64,   # small for test speed
        max_generations=100,
        early_stop_generations=30,
    )
    best = run_ga(seed, pods, nodes, groups, config, weights)

    # 1. All pods assigned
    assert len(best.assignment) == len(pods)
    assert all(0 <= idx < len(nodes) for idx in best.assignment)

    # 2. No capacity violation
    assert compute_capacity_overflow(best.assignment, pods, nodes) == 0.0

    # 3. USB-dongle pods are on node 1 (thinkcentre-beta)
    pod_names = [p.name for p in pods]
    ha_idx = pod_names.index("home-assistant")
    z2m_idx = pod_names.index("zigbee2mqtt")
    assert best.assignment[ha_idx] == 1, "Home Assistant must be on beta (USB)"
    assert best.assignment[z2m_idx] == 1, "Zigbee2MQTT must be on beta (USB)"

    # 4. GA should improve (or at least not worsen) FFD seed
    assert best.fitness <= seed.fitness


def test_ffd_produces_feasible_seed() -> None:
    """FFD seed should have no capacity violations on the homelab scenario."""
    raw_nodes, daemon_sets = load_infra(INFRA_PATH)
    pods, groups = load_workloads(WORKLOADS_PATH)
    nodes = pre_deduct_daemonsets(raw_nodes, daemon_sets)
    seed = ffd_warmstart(pods, nodes, FFDWeights())

    assert compute_capacity_overflow(seed.assignment, pods, nodes) == 0.0


def test_ga_reduces_affinity_violations() -> None:
    """GA should reduce affinity violations compared to random assignment."""
    raw_nodes, daemon_sets = load_infra(INFRA_PATH)
    pods, groups = load_workloads(WORKLOADS_PATH)
    nodes = pre_deduct_daemonsets(raw_nodes, daemon_sets)

    seed = ffd_warmstart(pods, nodes, FFDWeights())
    ffd_violations = compute_affinity_violations(seed.assignment, pods)

    config = GAConfig(
        population_size=64,
        max_generations=100,
        early_stop_generations=30,
    )
    best = run_ga(seed, pods, nodes, groups, config, FitnessWeights())
    ga_violations = compute_affinity_violations(best.assignment, pods)

    # GA should have same or fewer violations
    assert ga_violations <= ffd_violations
