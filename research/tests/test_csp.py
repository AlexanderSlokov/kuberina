"""Unit tests for CSP solver — hard constraint checks."""

from __future__ import annotations

from kuberina.model.types import Node, Pod, PodGroup, ResourceVector
from kuberina.phases.csp import (
    can_place_gang,
    can_place_pod_on_node,
    compute_capacity_overflow,
    compute_selector_violations,
    check_node_selector,
    check_taint_toleration,
)


def test_taint_toleration_passes_with_match() -> None:
    """Pod with correct toleration should pass taint check."""
    pod = Pod(name="p", namespace="ns", requests=ResourceVector(),
              tolerations=["gpu-only"])
    node = Node(name="n", allocatable=ResourceVector(), taints=["gpu-only"])
    assert check_taint_toleration(pod, node) is True


def test_taint_toleration_fails_without_match() -> None:
    """Pod without toleration should fail on tainted node."""
    pod = Pod(name="p", namespace="ns", requests=ResourceVector())
    node = Node(name="n", allocatable=ResourceVector(), taints=["gpu-only"])
    assert check_taint_toleration(pod, node) is False


def test_node_selector_matches() -> None:
    """Pod nodeSelector should match node labels."""
    pod = Pod(name="p", namespace="ns", requests=ResourceVector(),
              node_selector={"zone": "us-east"})
    node = Node(name="n", allocatable=ResourceVector(),
                labels={"zone": "us-east", "role": "worker"})
    assert check_node_selector(pod, node) is True


def test_node_selector_rejects_mismatch() -> None:
    """Pod nodeSelector should fail on node without matching label."""
    pod = Pod(name="p", namespace="ns", requests=ResourceVector(),
              node_selector={"zone": "us-east"})
    node = Node(name="n", allocatable=ResourceVector(),
                labels={"zone": "eu-west"})
    assert check_node_selector(pod, node) is False


def test_capacity_overflow_zero_when_valid() -> None:
    """Valid assignment should have 0 overflow."""
    pods = [Pod(name="p", namespace="ns", requests=ResourceVector(cpu=2.0, ram=8.0))]
    nodes = [Node(name="n", allocatable=ResourceVector(cpu=4.0, ram=16.0))]
    assert compute_capacity_overflow([0], pods, nodes) == 0.0


def test_capacity_overflow_positive_when_overloaded() -> None:
    """Overloaded node should return exact overflow amount."""
    pods = [
        Pod(name="a", namespace="ns", requests=ResourceVector(cpu=3.0, ram=10.0)),
        Pod(name="b", namespace="ns", requests=ResourceVector(cpu=3.0, ram=10.0)),
    ]
    nodes = [Node(name="n", allocatable=ResourceVector(cpu=4.0, ram=16.0))]
    # Demand = 6.0 CPU, 20.0 RAM
    # Capacity = 4.0 CPU, 16.0 RAM
    # Overflow = (6 - 4) + (20 - 16) = 2.0 + 4.0 = 6.0
    assert compute_capacity_overflow([0, 0], pods, nodes) == 6.0


def test_selector_violations_counted() -> None:
    """Unassigned pods or pods on violating nodes are counted."""
    pods = [
        Pod(name="a", namespace="ns", requests=ResourceVector(), node_selector={"gpu": "true"}),
        Pod(name="b", namespace="ns", requests=ResourceVector()),  # unassigned
    ]
    nodes = [Node(name="n", allocatable=ResourceVector())] # missing gpu=true label
    assert compute_selector_violations([0, -1], pods, nodes) == 2


def test_can_place_gang_sufficient_capacity() -> None:
    """Gang with 2 pods should pass when node has enough capacity."""
    pods = [
        Pod(name="w0", namespace="ns", requests=ResourceVector(cpu=1.0)),
        Pod(name="w1", namespace="ns", requests=ResourceVector(cpu=1.0)),
    ]
    nodes = [Node(name="n", allocatable=ResourceVector(cpu=4.0, ram=16.0))]
    group = PodGroup(name="g", pod_indices=[0, 1], min_members=2)
    loads = [ResourceVector.zero()]

    assert can_place_gang(group, nodes, loads, pods) is True


def test_can_place_gang_insufficient_capacity() -> None:
    """Gang should fail when total demand exceeds available capacity."""
    pods = [
        Pod(name="w0", namespace="ns", requests=ResourceVector(cpu=3.0)),
        Pod(name="w1", namespace="ns", requests=ResourceVector(cpu=3.0)),
    ]
    nodes = [Node(name="n", allocatable=ResourceVector(cpu=4.0, ram=16.0))]
    group = PodGroup(name="g", pod_indices=[0, 1], min_members=2)
    loads = [ResourceVector.zero()]

    assert can_place_gang(group, nodes, loads, pods) is False
