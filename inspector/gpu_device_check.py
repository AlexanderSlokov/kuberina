"""Check GPU placement device by device (ir-v1.md §3.4, backlog X-3).

Written from the specification, without reference to the solver. The solver decodes
devices with a greedy heuristic that may decline a placement that fits. This module
searches every device assignment, so it accepts exactly the placements the
specification accepts. A solver blueprint that this check rejects is a solver bug.
"""

from __future__ import annotations

from collections.abc import Iterable, Mapping, Sequence
from functools import lru_cache
from itertools import combinations

MILLI_PER_DEVICE = 1000

Shape = tuple[int, int]  # (devices, milli-GPU on each device)


def device_shape(pod: Mapping) -> Shape | None:
    """The `(devices, milli)` a pod needs, or None when it requests no GPU.

    Example:
        >>> device_shape({"requests": {"gpu": 0.5}, "gpuDevices": 1, "gpuMilli": 500})
        (1, 500)
    """
    if "gpuMilli" in pod:
        return pod["gpuDevices"], pod["gpuMilli"]
    gpu = pod.get("requests", {}).get("gpu", 0.0)
    if gpu <= 0:
        return None
    if gpu != int(gpu):
        raise ValueError(
            f"pod {pod.get('identity')} requests gpu: {gpu} without gpuMilli on a node "
            "with gpuDevices; expected a whole number of devices, or gpuMilli"
        )
    return int(gpu), MILLI_PER_DEVICE


def fits_on_devices(device_count: int, shapes: Iterable[Shape]) -> bool:
    """Whether some assignment of every shape to distinct devices fits.

    Example:
        >>> fits_on_devices(1, [(1, 600), (1, 400)])
        True
    """
    ordered = tuple(sorted(shapes, key=lambda s: (s[1], s[0]), reverse=True))
    if sum(d * m for d, m in ordered) > device_count * MILLI_PER_DEVICE:
        return False
    return _search(ordered, 0, (MILLI_PER_DEVICE,) * device_count)


def device_violations(nodes: Sequence[Mapping], pods: Sequence[Mapping], solution: Mapping[str, str]) -> list[str]:
    """Names of nodes with `gpuDevices` whose assigned pods do not fit device by device.

    Example:
        bad_nodes = device_violations(nodes, pods, blueprint["solution"])
    """
    shapes_on = _shapes_by_node(pods, solution)
    return [
        node["name"]
        for node in nodes
        if "gpuDevices" in node and not fits_on_devices(node["gpuDevices"], shapes_on.get(node["name"], []))
    ]


def _shapes_by_node(pods: Sequence[Mapping], solution: Mapping[str, str]) -> dict[str, list[Shape]]:
    shapes_on: dict[str, list[Shape]] = {}
    for pod in pods:
        node = solution.get(pod["identity"])
        shape = device_shape(pod) if node is not None else None
        if shape is not None:
            shapes_on.setdefault(node, []).append(shape)
    return shapes_on


@lru_cache(maxsize=None)
def _search(shapes: tuple[Shape, ...], index: int, free: tuple[int, ...]) -> bool:
    """Depth-first search; `free` is sorted, so equivalent device states share a cache entry."""
    if index == len(shapes):
        return True
    devices, milli = shapes[index]
    return any(_search(shapes, index + 1, after) for after in _placements(free, devices, milli))


def _placements(free: tuple[int, ...], devices: int, milli: int) -> set[tuple[int, ...]]:
    """Every distinct device state after giving `milli` on `devices` distinct devices."""
    roomy = [k for k, room in enumerate(free) if room >= milli]
    states = set()
    for chosen in combinations(roomy, devices):
        after = list(free)
        for k in chosen:
            after[k] -= milli
        states.add(tuple(sorted(after)))
    return states
