"""Check a blueprint that declines Pods (ir-v1.md §4 and §4.1, backlog X-4).

Written from the specification, without reference to the solver. Three checks:

- every Pod appears exactly once, in `solution` or in `unplaced`;
- under in-order admission, no unplaced Pod can take the place of a placed Pod that
  arrived after it (capacity on every dimension, GPU devices, and node selector);
- the GPU allocation: placed GPU over cluster GPU, the ratio the FGD simulator reports.
"""

from __future__ import annotations

from collections.abc import Mapping, Sequence

from gpu_device_check import device_shape, fits_on_devices

DIMENSIONS = ("cpu", "ram", "gpu", "storage", "disk_read", "disk_write", "net_in", "net_out")
TOLERANCE = 1e-9


def placement_problems(pods: Sequence[Mapping], solution: Mapping[str, str], unplaced: Sequence[str]) -> list[str]:
    """Pods missing from both lists, Pods in both, and names that are not Pods.

    Example:
        >>> pods = [{"identity": "ns/a-0"}]
        >>> placement_problems(pods, {}, ["ns/a-0"])
        []
    """
    declined = set(unplaced)
    problems = []
    for pod in pods:
        identity = pod["identity"]
        if identity in solution and identity in declined:
            problems.append(f"{identity} is in both solution and unplaced")
        elif identity not in solution and identity not in declined:
            problems.append(f"{identity} is in neither solution nor unplaced")
    known = {pod["identity"] for pod in pods}
    strangers = dict.fromkeys(i for i in [*solution, *unplaced] if i not in known)
    problems.extend(f"{identity} is not a pod of the workloads" for identity in strangers)
    return problems


def arrival_ranks(pods: Sequence[Mapping]) -> dict[str, int]:
    """Each Pod's position in arrival order (ir-v1.md §3.5).

    Replicas of a declaration share its `arrival` and come one after another in
    replica order, which is their order in `pods`.

    Example:
        >>> arrival_ranks([{"identity": "ns/b-0", "arrival": 7}, {"identity": "ns/a-0", "arrival": 1}])
        {'ns/a-0': 0, 'ns/b-0': 1}
    """
    for pod in pods:
        if "arrival" not in pod:
            raise ValueError(f"pod {pod['identity']} has no arrival; in-order admission needs one on every declaration")
    ordered = sorted(range(len(pods)), key=lambda k: (pods[k]["arrival"], k))
    return {pods[k]["identity"]: rank for rank, k in enumerate(ordered)}


def in_order_violations(
    nodes: Sequence[Mapping], pods: Sequence[Mapping], solution: Mapping[str, str], unplaced: Sequence[str]
) -> list[tuple[str, str]]:
    """`(unplaced, later)` pairs where the unplaced Pod could take the later Pod's place.

    At most one pair is reported per unplaced Pod.

    Example:
        bad = in_order_violations(nodes, pods, blueprint["solution"], blueprint["unplaced"])
    """
    rank = arrival_ranks(pods)
    pod_of = {pod["identity"]: pod for pod in pods}
    node_of = {node["name"]: node for node in nodes}
    loads = _node_loads(pod_of, solution)
    later_first = sorted(solution, key=rank.__getitem__)
    violations = []
    for early in sorted(unplaced, key=rank.__getitem__):
        taken = next(
            (later for later in later_first
             if rank[later] > rank[early]
             and _could_take_place(pod_of[early], pod_of[later], node_of[solution[later]], loads, pod_of, solution)),
            None,
        )
        if taken is not None:
            violations.append((early, taken))
    return violations


def gpu_allocation(nodes: Sequence[Mapping], pods: Sequence[Mapping], solution: Mapping[str, str]) -> tuple[float, float]:
    """GPU placed and GPU capacity, in devices.

    Example:
        >>> gpu_allocation([{"allocatable": {"gpu": 8.0}}], [{"identity": "a", "requests": {"gpu": 2.0}}], {"a": "n"})
        (2.0, 8.0)
    """
    placed = sum(pod["requests"].get("gpu", 0.0) for pod in pods if pod["identity"] in solution)
    capacity = sum(node["allocatable"].get("gpu", 0.0) for node in nodes if node["allocatable"].get("gpu", 0.0) != float("inf"))
    return placed, capacity


def _node_loads(pod_of: Mapping[str, Mapping], solution: Mapping[str, str]) -> dict[str, dict[str, float]]:
    loads: dict[str, dict[str, float]] = {}
    for identity, node in solution.items():
        load = loads.setdefault(node, dict.fromkeys(DIMENSIONS, 0.0))
        for dim in DIMENSIONS:
            load[dim] += pod_of[identity]["requests"].get(dim, 0.0)
    return loads


def _could_take_place(incoming, leaving, node, loads, pod_of, solution) -> bool:
    """Whether `incoming` fits `node` with `leaving` removed from it."""
    labels = node.get("labels") or {}
    if any(labels.get(key) != value for key, value in incoming.get("nodeSelector", {}).items()):
        return False
    load, capacity = loads[node["name"]], node["allocatable"]
    for dim in DIMENSIONS:
        after = load[dim] - leaving["requests"].get(dim, 0.0) + incoming["requests"].get(dim, 0.0)
        if after > capacity.get(dim, float("inf")) + TOLERANCE:
            return False
    if "gpuDevices" not in node:
        return True
    return fits_on_devices(node["gpuDevices"], _shapes_after_swap(node["name"], incoming, leaving, pod_of, solution))


def _shapes_after_swap(node_name, incoming, leaving, pod_of, solution):
    staying = [pod_of[i] for i, n in solution.items() if n == node_name and i != leaving["identity"]]
    shapes = [device_shape(pod) for pod in [*staying, incoming]]
    return [shape for shape in shapes if shape is not None]


def admission_report(nodes: Sequence[Mapping], pods: Sequence[Mapping], blueprint: Mapping) -> list[str]:
    """The inspector's admission lines for one blueprint, ready to print.

    Placement is checked for every blueprint. Allocation is reported when
    `metadata.admission` is set, and the in-order rule is checked when it is `in-order`.

    Example:
        for line in admission_report(nodes, pods, blueprint):
            print(line)
    """
    solution = blueprint.get("solution") or {}
    unplaced = blueprint.get("unplaced") or []
    problems = placement_problems(pods, solution, unplaced)
    lines = [f"Placement Problems: {len(problems)}", *(f"  {p}" for p in problems)]
    admission = (blueprint.get("metadata") or {}).get("admission")
    if admission is None:
        return lines
    placed, capacity = gpu_allocation(nodes, pods, solution)
    if capacity > 0:
        lines.append(f"GPU Allocation: {placed:.2f} of {capacity:.0f} GPUs ({100 * placed / capacity:.2f}%)")
    if admission == "in-order":
        violations = in_order_violations(nodes, pods, solution, unplaced)
        lines.append(f"In-Order Violations: {len(violations)}")
        lines.extend(f"  {early} could take the place of {later}" for early, later in violations)
    return lines
