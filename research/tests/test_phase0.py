"""Unit tests for Phase 0 — DaemonSet pre-deduction."""

from __future__ import annotations

from kuberina.model.types import DaemonSet, Node, ResourceVector
from kuberina.phases.phase0 import is_eligible, pre_deduct_daemonsets


def test_is_eligible_no_selector_matches_all() -> None:
    """DaemonSet with empty selector should match every node."""
    ds = DaemonSet(name="kube-proxy", resources=ResourceVector(cpu=0.1))
    node = Node(name="n1", allocatable=ResourceVector(cpu=4.0))
    assert is_eligible(ds, node) is True


def test_is_eligible_selector_mismatch() -> None:
    """DaemonSet with GPU selector should NOT match CPU-only node."""
    ds = DaemonSet(name="gpu-plugin", resources=ResourceVector(cpu=0.1),
                   node_selector={"gpu": "true"})
    node = Node(name="cpu-node", allocatable=ResourceVector(cpu=4.0),
                labels={"gpu": "false"})
    assert is_eligible(ds, node) is False


def test_pre_deduct_subtracts_resources() -> None:
    """Phase 0 should reduce allocatable by DaemonSet overhead."""
    nodes = [
        Node(name="n1", allocatable=ResourceVector(cpu=4.0, ram=16.0)),
    ]
    daemon_sets = [
        DaemonSet(name="kube-proxy", resources=ResourceVector(cpu=0.1, ram=0.064)),
        DaemonSet(name="calico", resources=ResourceVector(cpu=0.25, ram=0.128)),
    ]
    result = pre_deduct_daemonsets(nodes, daemon_sets)

    assert len(result) == 1
    assert abs(result[0].allocatable.cpu - 3.65) < 0.001
    assert abs(result[0].allocatable.ram - 15.808) < 0.001


def test_pre_deduct_preserves_original() -> None:
    """Phase 0 should NOT mutate the original node list."""
    nodes = [Node(name="n1", allocatable=ResourceVector(cpu=4.0, ram=16.0))]
    daemon_sets = [DaemonSet(name="ds", resources=ResourceVector(cpu=1.0))]
    pre_deduct_daemonsets(nodes, daemon_sets)

    assert nodes[0].allocatable.cpu == 4.0  # original unchanged
