"""Phase 1: FFD Warm-Start — generate a feasible seed blueprint via greedy packing.

Maritime analogy: Stack the heaviest containers first, fill gaps with smaller ones.
Formula from PAPER.md §4.2:
    V_i = α·CPU_i + β·RAM_i + γ·GPU_i
    Sort pods descending by V_i, first-fit into nodes.
Complexity: O(k·log(k) + k·m) — from PAPER.md §4.2.
"""

from __future__ import annotations

from kuberina.model.types import (
    Blueprint,
    FFDWeights,
    Node,
    Pod,
    ResourceVector,
)
from kuberina.phases.csp import can_place_pod_on_node


def synthetic_volume(pod: Pod, weights: FFDWeights) -> float:
    """Compute scalar weight for a pod based on normalized resource scarcity.

    V_i = α·CPU_i + β·RAM_i + γ·GPU_i
    Higher V means the pod is "heavier" and should be placed first.

    Example:
        >>> pod = Pod(name="gpu-worker", namespace="ai",
        ...           requests=ResourceVector(cpu=8.0, ram=64.0, gpu=1.0))
        >>> synthetic_volume(pod, FFDWeights(alpha=1.0, beta=1.0, gamma=10.0))
        82.0
    """
    r = pod.requests
    return weights.alpha * r.cpu + weights.beta * r.ram + weights.gamma * r.gpu


def _compute_node_loads(
    assignment: list[int],
    pods: list[Pod],
    num_nodes: int,
) -> list[ResourceVector]:
    """Accumulate per-node resource usage from pod assignments."""
    loads = [ResourceVector.zero() for _ in range(num_nodes)]
    for pod_idx, node_idx in enumerate(assignment):
        if node_idx >= 0:
            loads[node_idx] = loads[node_idx].add(pods[pod_idx].requests)
    return loads


def ffd_warmstart(
    pods: list[Pod],
    nodes: list[Node],
    weights: FFDWeights,
) -> Blueprint:
    """Generate a feasible initial blueprint using First-Fit Decreasing.

    Pods are sorted by synthetic volume (descending) and greedily placed
    into the first node with sufficient residual capacity.
    Accelerates GA convergence 3-5x vs random init (ga_estimation.md §5).

    Example:
        >>> pods = [Pod(name="big", namespace="ns", requests=ResourceVector(cpu=2.0, ram=8.0))]
        >>> nodes = [Node(name="n1", allocatable=ResourceVector(cpu=4.0, ram=16.0))]
        >>> bp = ffd_warmstart(pods, nodes, FFDWeights())
        >>> bp.assignment
        [0]
    """
    num_pods = len(pods)
    num_nodes = len(nodes)

    # Sort pod indices by synthetic volume descending (heaviest first)
    sorted_indices = sorted(
        range(num_pods),
        key=lambda i: synthetic_volume(pods[i], weights),
        reverse=True,
    )

    assignment = [-1] * num_pods
    residual = [ResourceVector(cpu=n.allocatable.cpu, ram=n.allocatable.ram,
                               gpu=n.allocatable.gpu) for n in nodes]

    for pod_idx in sorted_indices:
        placed = False
        for node_idx in range(num_nodes):
            if not can_place_pod_on_node(pods[pod_idx], nodes[node_idx]):
                continue
            if not residual[node_idx].fits(pods[pod_idx].requests):
                continue

            assignment[pod_idx] = node_idx
            residual[node_idx] = residual[node_idx].subtract(
                pods[pod_idx].requests,
            )
            placed = True
            break

        if not placed:
            # WHY: pod cannot fit anywhere — assign to node 0 as fallback.
            # GA will try to fix this via mutation. Fitness penalty will apply.
            assignment[pod_idx] = 0

    node_load = _compute_node_loads(assignment, pods, num_nodes)
    return Blueprint(assignment=assignment, fitness=0.0, node_load=node_load)
