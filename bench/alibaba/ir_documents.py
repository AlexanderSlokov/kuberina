"""Build Kuberina IR v1 Infrastructure and Workloads documents from the Alibaba trace.

Units follow `docs/references/ir-v1.md` §1: cores, GiB, and GPU devices. Quantities are
written as plain numbers because the inspector reads numbers only. Throughput and
storage are left out, which IR v1 reads as unconstrained on nodes and zero on pods: the
simulator models CPU, memory and GPU only.

GPU is the scalar of IR v1 until backlog X-3 adds per-device fields, so a 0.81-GPU pod
is written as `gpu: 0.81`.
"""

from __future__ import annotations

from collections.abc import Iterable, Mapping
from typing import TypedDict

from alibaba.simulator_log import ArrivalRecord
from alibaba.trace_csv import TraceNode, TracePod, source_pod_name

IR_API_VERSION = "kuberina.io/v1"
GPU_MODEL_LABEL = "gpu-model"
MIB_PER_GIB = 1024
MILLI = 1000

Resources = dict[str, float]


class NodeEntry(TypedDict):
    name: str
    labels: dict[str, str]
    allocatable: Resources


class Declaration(TypedDict):
    name: str
    requests: Resources


class Infrastructure(TypedDict):
    apiVersion: str
    kind: str
    nodes: list[NodeEntry]


class Workloads(TypedDict):
    apiVersion: str
    kind: str
    namespaces: dict[str, list[Declaration]]


def build_infrastructure(nodes: Iterable[TraceNode]) -> Infrastructure:
    """One IR node per trace GPU node, labeled with its GPU model.

    Example:
        >>> build_infrastructure([TraceNode("n", 8000, 32768, 1, "T4")])["nodes"][0]["allocatable"]
        {'cpu': 8.0, 'ram': 32.0, 'gpu': 1}
    """
    return {
        "apiVersion": IR_API_VERSION,
        "kind": "Infrastructure",
        "nodes": [_node_entry(node) for node in nodes],
    }


def build_workloads(
    records: Iterable[ArrivalRecord], pods: Mapping[str, TracePod]
) -> Workloads:
    """One declaration per logged pod, in arrival order, grouped by namespace.

    Each tuned clone takes the requests of its source row. IR v1 has no arrival field,
    so the arrival order is the declaration order; `sequence.csv` states it per pod.

    Example:
        workloads = build_workloads(read_simulator_log(log), read_trace_pods(pod_csv))
    """
    namespaces: dict[str, list[Declaration]] = {}
    for record in records:
        namespace, name = record.pod.split("/", 1)
        requests = _pod_requests(_trace_row(record.pod, pods))
        namespaces.setdefault(namespace, []).append(
            {"name": name, "requests": requests}
        )
    return {"apiVersion": IR_API_VERSION, "kind": "Workloads", "namespaces": namespaces}


def pod_identity(record: ArrivalRecord) -> str:
    """The IR pod identity of a logged pod: one replica unrolls to `<name>-0` (§3.1).

    Example:
        >>> pod_identity(ArrivalRecord(0, "paib-gpu/openb-pod-0255", None, 0))
        'paib-gpu/openb-pod-0255-0'
    """
    return f"{record.pod}-0"


def _trace_row(logged_pod: str, pods: Mapping[str, TracePod]) -> TracePod:
    source = source_pod_name(logged_pod)
    if source not in pods:
        raise ValueError(
            f"logged pod {logged_pod!r} maps to trace row {source!r}, absent from the pod list"
        )
    return pods[source]


def _node_entry(node: TraceNode) -> NodeEntry:
    allocatable: Resources = {
        "cpu": node.cpu_milli / MILLI,
        "ram": node.memory_mib / MIB_PER_GIB,
        "gpu": node.gpus,
    }
    return {
        "name": node.name,
        "labels": {GPU_MODEL_LABEL: node.model},
        "allocatable": allocatable,
    }


def _pod_requests(pod: TracePod) -> Resources:
    requests: Resources = {
        "cpu": pod.cpu_milli / MILLI,
        "ram": pod.memory_mib / MIB_PER_GIB,
    }
    if pod.gpu_milli_total:
        requests["gpu"] = _whole_or_fraction(pod.gpu_milli_total)
    return requests


def _whole_or_fraction(gpu_milli: int) -> float:
    """Write whole devices as integers (`8`) and fractions as decimals (`0.81`)."""
    whole, remainder = divmod(gpu_milli, MILLI)
    return whole if remainder == 0 else gpu_milli / MILLI
