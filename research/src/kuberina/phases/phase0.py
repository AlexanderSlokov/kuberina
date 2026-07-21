"""Phase 0: DaemonSet pre-deduction — subtract system overhead from node capacity.

Maritime analogy: Fill ballast tanks to establish baseline draft before cargo loading.
Formula from PAPER.md §3.2:
    C_j^r = C_{j,raw}^r - Σ 𝟙[eligible(d, n_j)] · res_d^r
"""

from __future__ import annotations

from kuberina.model.types import DaemonSet, Node, ResourceVector


def is_eligible(daemon_set: DaemonSet, node: Node) -> bool:
    """Check if a DaemonSet should run on a given node.

    Matches nodeSelector labels and checks taint tolerations.
    An empty selector means "run on all nodes" (typical for kube-proxy).

    Example:
        >>> ds = DaemonSet(name="gpu-plugin", resources=ResourceVector(),
        ...                node_selector={"gpu": "true"})
        >>> node = Node(name="n1", allocatable=ResourceVector(), labels={"gpu": "true"})
        >>> is_eligible(ds, node)
        True
    """
    for key, value in daemon_set.node_selector.items():
        if node.labels.get(key) != value:
            return False

    # WHY: DaemonSets typically tolerate all taints (they MUST run).
    # If DS has no tolerations specified, it tolerates everything.
    if not daemon_set.tolerations:
        return True

    for taint in node.taints:
        if taint not in daemon_set.tolerations:
            return False

    return True


def pre_deduct_daemonsets(
    nodes: list[Node],
    daemon_sets: list[DaemonSet],
) -> list[Node]:
    """Subtract DaemonSet resource consumption from every eligible node.

    After this, nodes[j].allocatable = REAL capacity available for workload pods.
    The optimizer never sees DaemonSet pods — they are fixed variables.

    Example:
        >>> nodes = [Node(name="n1", allocatable=ResourceVector(cpu=4.0, ram=16.0))]
        >>> ds_list = [DaemonSet(name="kube-proxy", resources=ResourceVector(cpu=0.1, ram=0.064))]
        >>> result = pre_deduct_daemonsets(nodes, ds_list)
        >>> result[0].allocatable.cpu
        3.9
    """
    deducted_nodes: list[Node] = []

    for node in nodes:
        total_overhead = ResourceVector.zero()
        for ds in daemon_sets:
            if is_eligible(ds, node):
                total_overhead = total_overhead.add(ds.resources)

        deducted = Node(
            name=node.name,
            allocatable=node.allocatable.subtract(total_overhead),
            labels=node.labels,
            taints=node.taints,
            zone=node.zone,
        )
        deducted_nodes.append(deducted)

    return deducted_nodes
