"""CSP Solver — hard constraint pre-screening for every placement decision.

Integrated into FFD (Phase 1) and GA mutation/repair (Phase 2).
From PAPER.md §4.4 and DESIGN.md §CSP Forward Checking.

Hard constraints (PAPER.md §3.2):
  1. Capacity:        Σ x_ij · req_i^r ≤ C_j^r
  2. Assignment:      Σ x_ij = 1 (handled by chromosome encoding)
  3. Taint/Toleration: Taints(n_j) ⊆ Tolerations(p_i)
  4. NodeSelector:    Labels(n_j) ⊇ Selector(p_i)
  5. Gang:            All pods in group feasible, or none
"""

from __future__ import annotations

from kuberina.model.types import Node, Pod, PodGroup, ResourceVector


def check_taint_toleration(pod: Pod, node: Node) -> bool:
    """Pod can only land on a tainted node if it has matching tolerations.

    From PAPER.md §3.2 Hard Constraint #3.

    Example:
        >>> pod = Pod(name="p", namespace="ns", requests=ResourceVector(),
        ...           tolerations=["gpu-only"])
        >>> node = Node(name="n", allocatable=ResourceVector(), taints=["gpu-only"])
        >>> check_taint_toleration(pod, node)
        True
    """
    for taint in node.taints:
        if taint not in pod.tolerations:
            return False
    return True


def check_node_selector(pod: Pod, node: Node) -> bool:
    """Pod can only land on nodes matching its selector labels.

    From PAPER.md §3.2 Hard Constraint #4.

    Example:
        >>> pod = Pod(name="p", namespace="ns", requests=ResourceVector(),
        ...           node_selector={"zone": "us-east"})
        >>> node = Node(name="n", allocatable=ResourceVector(),
        ...             labels={"zone": "us-east"})
        >>> check_node_selector(pod, node)
        True
    """
    for key, value in pod.node_selector.items():
        if node.labels.get(key) != value:
            return False
    return True


def can_place_pod_on_node(pod: Pod, node: Node) -> bool:
    """Combined pre-screen: taint + nodeSelector (excludes capacity check).

    Used by FFD and GA mutation before checking residual capacity.

    Example:
        >>> pod = Pod(name="p", namespace="ns", requests=ResourceVector())
        >>> node = Node(name="n", allocatable=ResourceVector(cpu=4.0, ram=16.0))
        >>> can_place_pod_on_node(pod, node)
        True
    """
    if not check_taint_toleration(pod, node):
        return False
    return check_node_selector(pod, node)


def check_capacity_all_nodes(
    assignment: list[int],
    pods: list[Pod],
    nodes: list[Node],
) -> bool:
    """Verify no node exceeds allocatable resources on any dimension.

    Hard Constraint #1 from PAPER.md §3.2:
        ∀j, ∀r: Σ x_ij · req_i^r ≤ C_j^r

    Example:
        >>> pods = [Pod(name="p", namespace="ns", requests=ResourceVector(cpu=2.0, ram=8.0))]
        >>> nodes = [Node(name="n", allocatable=ResourceVector(cpu=4.0, ram=16.0))]
        >>> check_capacity_all_nodes([0], pods, nodes)
        True
    """
    loads = [ResourceVector.zero() for _ in nodes]
    for pod_idx, node_idx in enumerate(assignment):
        loads[node_idx] = loads[node_idx].add(pods[pod_idx].requests)

    for node_idx, load in enumerate(loads):
        if not nodes[node_idx].allocatable.fits(load):
            return False
    return True


def can_place_gang(
    group: PodGroup,
    nodes: list[Node],
    node_load: list[ResourceVector],
    pods: list[Pod],
) -> bool:
    """Forward checking for gang scheduling — verify group is feasible.

    From DESIGN.md canPlaceGang pseudocode.
    1. Enough eligible nodes for all pods in gang?
    2. Total residual capacity >= total demand?
    3. If colocate: any single node fits the entire gang?

    Example:
        >>> group = PodGroup(name="g", pod_indices=[0, 1], min_members=2)
        >>> pods = [Pod(name="p0", namespace="ns", requests=ResourceVector(cpu=1.0)),
        ...         Pod(name="p1", namespace="ns", requests=ResourceVector(cpu=1.0))]
        >>> nodes = [Node(name="n0", allocatable=ResourceVector(cpu=4.0, ram=16.0))]
        >>> can_place_gang(group, nodes, [ResourceVector.zero()], pods)
        True
    """
    eligible = _filter_eligible_nodes(group, nodes, pods)
    if len(eligible) < len(group.pod_indices):
        return False

    total_demand = _sum_gang_demand(group, pods)
    total_avail = _sum_residual_capacity(eligible, nodes, node_load)
    if not total_avail.fits(total_demand):
        return False

    if group.colocate:
        return _any_node_fits_entire_gang(eligible, nodes, node_load, total_demand)

    return True


def _filter_eligible_nodes(
    group: PodGroup,
    nodes: list[Node],
    pods: list[Pod],
) -> list[int]:
    """Return indices of nodes eligible for all pods in this gang."""
    eligible: list[int] = []
    # WHY: use group's nodeSelector, not individual pod selectors,
    # because gang pods share the same hardware requirement (e.g., GPU+NVLink).
    for node_idx, node in enumerate(nodes):
        for key, value in group.node_selector.items():
            if node.labels.get(key) != value:
                break
        else:
            # Also check that at least one gang pod can pass taint check
            sample_pod = pods[group.pod_indices[0]]
            if check_taint_toleration(sample_pod, node):
                eligible.append(node_idx)
    return eligible


def _sum_gang_demand(group: PodGroup, pods: list[Pod]) -> ResourceVector:
    """Sum resource requests of all pods in a gang."""
    total = ResourceVector.zero()
    for pod_idx in group.pod_indices:
        total = total.add(pods[pod_idx].requests)
    return total


def _sum_residual_capacity(
    eligible_indices: list[int],
    nodes: list[Node],
    node_load: list[ResourceVector],
) -> ResourceVector:
    """Sum remaining capacity across all eligible nodes."""
    total = ResourceVector.zero()
    for idx in eligible_indices:
        residual = nodes[idx].allocatable.subtract(node_load[idx])
        total = total.add(residual)
    return total


def _any_node_fits_entire_gang(
    eligible_indices: list[int],
    nodes: list[Node],
    node_load: list[ResourceVector],
    total_demand: ResourceVector,
) -> bool:
    """Check if any single eligible node can host the entire gang."""
    for idx in eligible_indices:
        residual = nodes[idx].allocatable.subtract(node_load[idx])
        if residual.fits(total_demand):
            return True
    return False
