"""Fitness function — evaluate blueprint quality via weighted multi-objective sum.

From PAPER.md §3.2 Objective Function:
    min F(s) = w1·f_nodes + w2·f_frag + w3·f_affinity + w4·f_var + Φ(s)

Lower is better. Φ(s) = -inf if any hard constraint violated.
"""

from __future__ import annotations

import math
from statistics import variance

from kuberina.model.types import (
    Blueprint,
    FitnessWeights,
    Node,
    Pod,
    PodGroup,
    ResourceVector,
)
from kuberina.phases.csp import (
    compute_capacity_overflow,
    compute_selector_violations,
)


def compute_fitness(
    blueprint: Blueprint,
    pods: list[Pod],
    nodes: list[Node],
    groups: list[PodGroup],
    weights: FitnessWeights,
) -> float:
    """Compute weighted-sum fitness score for a blueprint.

    Calls each sub-function, applies weights, and checks hard constraints.
    Returns a float — lower is better. -inf means infeasible.

    Example:
        >>> # (see test_fitness.py for full examples)
    """
    penalty = compute_hard_penalty(blueprint, pods, nodes, groups)
    # WHY: we removed math.inf check because penalties are now continuous scalars.

    f_nodes = count_active_nodes(blueprint.assignment, len(nodes))
    f_frag = compute_fragmentation(blueprint.node_load, nodes)
    f_aff = compute_affinity_violations(blueprint.assignment, pods)
    f_var = compute_utilization_variance(blueprint.node_load, nodes)

    return (
        penalty
        + weights.node_count * f_nodes
        + weights.fragmentation * f_frag
        + weights.affinity_violation * f_aff
        + weights.utilization_variance * f_var
    )


def count_active_nodes(assignment: list[int], num_nodes: int) -> int:
    """f_nodes: count nodes that have at least one pod assigned.

    PAPER.md: f_nodes = Σ y_j (number of active nodes).

    Example:
        >>> count_active_nodes([0, 0, 2], 3)
        2
    """
    active = set(assignment)
    return len(active)


def compute_fragmentation(
    node_load: list[ResourceVector],
    nodes: list[Node],
) -> float:
    """f_frag: sum of wasted capacity on active nodes across all dimensions.

    PAPER.md: f_frag = Σ_{j: y_j=1} Σ_r max(0, C_j^r - Σ req_i^r)

    Example:
        >>> loads = [ResourceVector(cpu=2.0, ram=8.0)]
        >>> ns = [Node(name="n", allocatable=ResourceVector(cpu=4.0, ram=16.0))]
        >>> compute_fragmentation(loads, ns)
        10.0
    """
    total_waste = 0.0
    for j, load in enumerate(node_load):
        # WHY: skip empty nodes — they have no waste, they're just unused.
        if load.cpu == 0 and load.ram == 0 and load.gpu == 0:
            continue
        cap = nodes[j].allocatable
        total_waste += max(0.0, cap.cpu - load.cpu)
        total_waste += max(0.0, cap.ram - load.ram)
        total_waste += max(0.0, cap.gpu - load.gpu)
    return total_waste


def compute_affinity_violations(
    assignment: list[int],
    pods: list[Pod],
) -> int:
    """f_affinity: count soft affinity/anti-affinity violations.

    Affinity: pods that WANT to be on the same node but aren't.
    Anti-affinity: pods that DON'T want to be on the same node but are.

    Example:
        >>> pods = [Pod(name="a", namespace="ns", requests=ResourceVector(),
        ...             affinity_targets=["b"]),
        ...         Pod(name="b", namespace="ns", requests=ResourceVector())]
        >>> compute_affinity_violations([0, 1], pods)
        1
    """
    name_to_idx = {pod.name: i for i, pod in enumerate(pods)}
    violations = 0

    for i, pod in enumerate(pods):
        for target_name in pod.affinity_targets:
            target_idx = name_to_idx.get(target_name)
            if target_idx is None:
                continue
            # WHY: only count once per pair — check i < target_idx
            if i < target_idx and assignment[i] != assignment[target_idx]:
                violations += 1

        for target_name in pod.anti_affinity_targets:
            target_idx = name_to_idx.get(target_name)
            if target_idx is None:
                continue
            if i < target_idx and assignment[i] == assignment[target_idx]:
                violations += 1

    return violations


def compute_utilization_variance(
    node_load: list[ResourceVector],
    nodes: list[Node],
) -> float:
    """f_var: variance of CPU utilization across active nodes.

    PAPER.md: Var({U_j^r : y_j = 1}) — vessel trim & stability.

    Example:
        >>> loads = [ResourceVector(cpu=2.0), ResourceVector(cpu=4.0)]
        >>> ns = [Node(name="a", allocatable=ResourceVector(cpu=4.0, ram=16.0)),
        ...       Node(name="b", allocatable=ResourceVector(cpu=4.0, ram=16.0))]
        >>> compute_utilization_variance(loads, ns)  # 0.5, 1.0 -> var = 0.125
        0.125
    """
    utilizations: list[float] = []
    for j, load in enumerate(node_load):
        cap = nodes[j].allocatable
        if cap.cpu <= 0:
            continue
        if load.cpu == 0 and load.ram == 0 and load.gpu == 0:
            continue
        utilizations.append(load.cpu / cap.cpu)

    if len(utilizations) < 2:
        return 0.0

    return variance(utilizations)


def compute_hard_penalty(
    blueprint: Blueprint,
    pods: list[Pod],
    nodes: list[Node],
    groups: list[PodGroup],
) -> float:
    """Φ(s): gradient penalty scalar for hard constraints.

    Instead of math.inf, returns a massive but differentiable scalar.
    This allows the GA to climb out of infeasible search spaces (like
    the MSC Irina scale test) rather than plateauing in infinity.

    Base penalty weight: 1,000,000 to ensure any invalid solution
    is ranked strictly worse than any valid solution.
    """
    total_penalty = 0.0

    capacity_overflow = compute_capacity_overflow(blueprint.assignment, pods, nodes)
    if capacity_overflow > 0:
        total_penalty += 1_000_000 + (capacity_overflow * 10_000)

    selector_violations = compute_selector_violations(blueprint.assignment, pods, nodes)
    if selector_violations > 0:
        total_penalty += 500_000 + (selector_violations * 50_000)

    total_penalty += _gang_penalty(blueprint, pods, nodes, groups)

    return total_penalty


def _gang_penalty(
    blueprint: Blueprint,
    pods: list[Pod],
    nodes: list[Node],
    groups: list[PodGroup],
) -> float:
    """Soft penalty if any gang pod can't fit on its assigned node.

    Penalty scales linearly with the number of missing pods to reach min_members.
    Base penalty: 500,000.
    """
    penalty = 0.0
    for group in groups:
        placed = 0
        for pod_idx in group.pod_indices:
            node_idx = blueprint.assignment[pod_idx]
            # Handle unassigned pods fallback
            if node_idx < 0:
                continue
            node_cap = nodes[node_idx].allocatable
            if node_cap.fits(blueprint.node_load[node_idx]):
                placed += 1
        if placed < group.min_members:
            missing = group.min_members - placed
            penalty += 500_000 + (missing * 10_000)
    return penalty
