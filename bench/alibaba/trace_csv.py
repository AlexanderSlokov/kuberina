"""Read the node and pod tables of Alibaba `cluster-trace-gpu-v2023`.

Source: `github.com/alibaba/clusterdata`, directory `cluster-trace-gpu-v2023/csv`, at
commit `3858992ca41bf92082e6d6522ddf0bebd14e838b`, the commit the simulator calibration
used (`docs/references/sessions/2026-10-31-fgd-simulator-calibration.md`). The benchmark
cluster is `openb_node_list_gpu_node.csv` only; the simulator never loads the 310
CPU-only nodes.
"""

from __future__ import annotations

import csv
import re
from dataclasses import dataclass
from pathlib import Path

NODE_COLUMNS = ("sn", "cpu_milli", "memory_mib", "gpu", "model")
POD_COLUMNS = ("name", "cpu_milli", "memory_mib", "num_gpu", "gpu_milli")
TUNED_SUFFIX = re.compile(r"-tuned-\d+$")


@dataclass(frozen=True)
class TraceNode:
    """One GPU node of the benchmark cluster.

    Example:
        >>> TraceNode("openb-node-0260", 96000, 393216, 8, "G2").gpus
        8
    """

    name: str
    cpu_milli: int
    memory_mib: int
    gpus: int
    model: str


@dataclass(frozen=True)
class TracePod:
    """One row of a trace pod list. `gpu_milli` is per device, as in the trace.

    Example:
        >>> TracePod("openb-pod-0001", 6000, 12288, 1, 460).gpu_milli_total
        460
    """

    name: str
    cpu_milli: int
    memory_mib: int
    num_gpu: int
    gpu_milli: int

    @property
    def gpu_milli_total(self) -> int:
        """GPU demand in milli-GPU, summed over the pod's devices."""
        return self.num_gpu * self.gpu_milli


def read_trace_nodes(path: Path) -> list[TraceNode]:
    """Read `openb_node_list_gpu_node.csv` in file order.

    Example:
        nodes = read_trace_nodes(Path("csv/openb_node_list_gpu_node.csv"))
    """
    return [
        TraceNode(
            row["sn"],
            int(row["cpu_milli"]),
            int(row["memory_mib"]),
            int(row["gpu"]),
            row["model"],
        )
        for row in _read_rows(path, NODE_COLUMNS)
    ]


def read_trace_pods(path: Path) -> dict[str, TracePod]:
    """Read a pod list such as `openb_pod_list_default.csv`, keyed by pod name.

    Example:
        pods = read_trace_pods(Path("csv/openb_pod_list_default.csv"))
    """
    pods = (
        TracePod(
            row["name"],
            int(row["cpu_milli"]),
            int(row["memory_mib"]),
            int(row["num_gpu"]),
            int(row["gpu_milli"]),
        )
        for row in _read_rows(path, POD_COLUMNS)
    )
    return {pod.name: pod for pod in pods}


def source_pod_name(logged_pod: str) -> str:
    """Map a logged pod (`<namespace>/<name>[-tuned-<i>]`) to its trace row name.

    The simulator's `tuneUpPods` clones rows to reach 130% demand and names each clone
    `<source>-tuned-<i>`; the clone has the source row's requests.

    Example:
        >>> source_pod_name("paib-gpu/openb-pod-3888-tuned-0")
        'openb-pod-3888'
    """
    name = logged_pod.rsplit("/", 1)[-1]
    return TUNED_SUFFIX.sub("", name)


def _read_rows(path: Path, required: tuple[str, ...]) -> list[dict[str, str]]:
    with path.open(newline="", encoding="utf-8") as stream:
        reader = csv.DictReader(stream)
        missing = [
            column for column in required if column not in (reader.fieldnames or [])
        ]
        if missing:
            raise ValueError(
                f"{path} lacks columns {missing}; expected at least {list(required)}"
            )
        return list(reader)
