"""Unit tests for Phase 1 — FFD warm-start."""

from __future__ import annotations

from kuberina.model.types import FFDWeights, Node, Pod, ResourceVector
from kuberina.phases.phase1_ffd import ffd_warmstart, synthetic_volume


def test_synthetic_volume_gpu_dominant() -> None:
    """GPU pod should have much higher synthetic volume than CPU pod."""
    gpu_pod = Pod(name="gpu", namespace="ai",
                  requests=ResourceVector(cpu=4.0, ram=32.0, gpu=1.0))
    cpu_pod = Pod(name="cpu", namespace="web",
                  requests=ResourceVector(cpu=2.0, ram=4.0))
    weights = FFDWeights(alpha=1.0, beta=1.0, gamma=10.0)

    assert synthetic_volume(gpu_pod, weights) > synthetic_volume(cpu_pod, weights)


def test_ffd_places_all_pods() -> None:
    """FFD should assign every pod to a valid node index."""
    pods = [
        Pod(name="a", namespace="ns", requests=ResourceVector(cpu=1.0, ram=4.0)),
        Pod(name="b", namespace="ns", requests=ResourceVector(cpu=2.0, ram=8.0)),
    ]
    nodes = [
        Node(name="n1", allocatable=ResourceVector(cpu=4.0, ram=16.0)),
        Node(name="n2", allocatable=ResourceVector(cpu=4.0, ram=16.0)),
    ]
    bp = ffd_warmstart(pods, nodes, FFDWeights())

    assert len(bp.assignment) == 2
    assert all(0 <= idx < 2 for idx in bp.assignment)


def test_ffd_heaviest_first() -> None:
    """FFD should place the heaviest pod first (on the first available node)."""
    pods = [
        Pod(name="light", namespace="ns", requests=ResourceVector(cpu=0.5)),
        Pod(name="heavy", namespace="ns", requests=ResourceVector(cpu=3.0)),
    ]
    nodes = [
        Node(name="n1", allocatable=ResourceVector(cpu=4.0, ram=16.0)),
    ]
    bp = ffd_warmstart(pods, nodes, FFDWeights())

    # Both should fit on node 0
    assert bp.assignment == [0, 0]


def test_ffd_respects_node_selector() -> None:
    """Pod with nodeSelector should only be placed on matching nodes."""
    pods = [
        Pod(name="gpu-pod", namespace="ns",
            requests=ResourceVector(cpu=1.0),
            node_selector={"gpu": "true"}),
    ]
    nodes = [
        Node(name="cpu-node", allocatable=ResourceVector(cpu=4.0, ram=16.0),
             labels={}),
        Node(name="gpu-node", allocatable=ResourceVector(cpu=4.0, ram=16.0),
             labels={"gpu": "true"}),
    ]
    bp = ffd_warmstart(pods, nodes, FFDWeights())

    assert bp.assignment[0] == 1  # must be on gpu-node
