"""Unit tests for fitness function components."""

from __future__ import annotations

import math

from kuberina.model.types import (
    Blueprint,
    FitnessWeights,
    Node,
    Pod,
    PodGroup,
    ResourceVector,
)
from kuberina.fitness import (
    compute_affinity_violations,
    compute_fitness,
    compute_fragmentation,
    compute_utilization_variance,
    count_active_nodes,
)


def test_count_active_nodes() -> None:
    """Only nodes with pods assigned should be counted."""
    assert count_active_nodes([0, 0, 2], 3) == 2
    assert count_active_nodes([1, 1, 1], 3) == 1


def test_fragmentation_zero_on_perfect_fit() -> None:
    """No waste when pod fills node capacity exactly."""
    loads = [ResourceVector(cpu=4.0, ram=16.0)]
    nodes = [Node(name="n", allocatable=ResourceVector(cpu=4.0, ram=16.0))]
    assert compute_fragmentation(loads, nodes) == 0.0


def test_fragmentation_measures_waste() -> None:
    """Waste = cap - load on each active dimension."""
    loads = [ResourceVector(cpu=2.0, ram=8.0)]
    nodes = [Node(name="n", allocatable=ResourceVector(cpu=4.0, ram=16.0))]
    # waste = (4-2) + (16-8) = 10.0
    assert compute_fragmentation(loads, nodes) == 10.0


def test_affinity_violation_detected() -> None:
    """Pod wanting co-location but placed on different node → violation."""
    pods = [
        Pod(name="a", namespace="ns", requests=ResourceVector(),
            affinity_targets=["b"]),
        Pod(name="b", namespace="ns", requests=ResourceVector()),
    ]
    assert compute_affinity_violations([0, 1], pods) == 1


def test_affinity_satisfied_no_violation() -> None:
    """Co-located affinity pair → no violation."""
    pods = [
        Pod(name="a", namespace="ns", requests=ResourceVector(),
            affinity_targets=["b"]),
        Pod(name="b", namespace="ns", requests=ResourceVector()),
    ]
    assert compute_affinity_violations([0, 0], pods) == 0


def test_anti_affinity_violation() -> None:
    """Pods that should be apart but are on same node → violation."""
    pods = [
        Pod(name="a", namespace="ns", requests=ResourceVector(),
            anti_affinity_targets=["b"]),
        Pod(name="b", namespace="ns", requests=ResourceVector()),
    ]
    assert compute_affinity_violations([0, 0], pods) == 1


def test_utilization_variance_balanced() -> None:
    """Identical utilization → zero variance."""
    loads = [ResourceVector(cpu=2.0), ResourceVector(cpu=2.0)]
    nodes = [
        Node(name="a", allocatable=ResourceVector(cpu=4.0, ram=16.0)),
        Node(name="b", allocatable=ResourceVector(cpu=4.0, ram=16.0)),
    ]
    assert compute_utilization_variance(loads, nodes) == 0.0


def test_hard_penalty_on_overcapacity() -> None:
    """Blueprint violating capacity should get +inf fitness."""
    pods = [Pod(name="p", namespace="ns", requests=ResourceVector(cpu=5.0))]
    nodes = [Node(name="n", allocatable=ResourceVector(cpu=4.0, ram=16.0))]
    bp = Blueprint(
        assignment=[0],
        node_load=[ResourceVector(cpu=5.0)],
    )
    fitness = compute_fitness(bp, pods, nodes, [], FitnessWeights())
    assert math.isinf(fitness) and fitness > 0
