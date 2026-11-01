"""Read the pod arrival sequence from one FGD simulator log (`simon.log`).

The simulator draws its pod order with Go's `math/rand`. Rebuilding that order in
Python is fragile, so the adapter reads the order the simulator logged instead (plan
`docs/plans/benchmarks/alibaba-gpu-v2023/00_plan.md`, section 2). For every pod the
log holds three lines, in this order:

    [i] attempt to create pod(<namespace>/<pod>)
    pod(<namespace>/<pod>) is scheduled to node(<simulator index>-<trace node>)
      or: [i] failed to schedule pod(<namespace>/<pod>): <...>
    [Alloc]; ... Arrived GPU Milli: <cumulative milli-GPU offered so far>

Every other line is ignored.
"""

from __future__ import annotations

import gzip
import re
from collections.abc import Iterable
from dataclasses import dataclass
from pathlib import Path

ATTEMPT_LINE = re.compile(r"\[(\d+)\] attempt to create pod\(([^)]+)\)")
PLACED_LINE = re.compile(r"pod\(([^)]+)\) is scheduled to node\(\d+-([^)]+)\)")
FAILED_LINE = re.compile(r"\[\d+\] failed to schedule pod\(([^)]+)\)")
ALLOC_LINE = re.compile(r"\[Alloc\];.*Arrived GPU Milli: (\d+)")
REJECTED = ""  # outcome marker for a pod the policy did not place


@dataclass(frozen=True)
class ArrivalRecord:
    """One pod offered to the simulator, and what the policy did with it.

    `node` is the trace node name (`openb-node-0260`), or None when the policy
    rejected the pod. `arrived_gpu_milli` is the simulator's running total of GPU
    demand offered up to and including this pod.

    Example:
        >>> ArrivalRecord(0, "paib-gpu/openb-pod-0255", None, 0).node is None
        True
    """

    index: int
    pod: str
    node: str | None
    arrived_gpu_milli: int


@dataclass
class _PendingPod:
    index: int
    pod: str
    outcome: str | None = None  # trace node name, REJECTED, or None before the outcome


def read_simulator_log(path: Path) -> list[ArrivalRecord]:
    """Read a `simon.log` or `simon.log.gz` file into its arrival records.

    Example:
        records = read_simulator_log(Path("x1-seed42/06-FGD/simon.log.gz"))
    """
    opener = gzip.open if path.suffix == ".gz" else open
    with opener(path, "rt", encoding="utf-8") as stream:
        return parse_simulator_log(stream)


def parse_simulator_log(lines: Iterable[str]) -> list[ArrivalRecord]:
    """Fold the log lines into one ArrivalRecord per attempted pod.

    Example:
        >>> parse_simulator_log(['msg="[Tune] Num of Pods after Tuning: 0"'])
        []
    """
    records: list[ArrivalRecord] = []
    pending: _PendingPod | None = None
    for line in lines:
        pending = _consume_line(line, pending, records)
    if pending is not None:
        raise ValueError(f"log ends before the [Alloc] line of pod {pending.pod!r}")
    return records


def _consume_line(
    line: str, pending: _PendingPod | None, records: list[ArrivalRecord]
) -> _PendingPod | None:
    """Apply one log line to the pod in flight; return the pod still in flight."""
    if match := ATTEMPT_LINE.search(line):
        return _start_pod(int(match[1]), match[2], pending, len(records))
    if match := PLACED_LINE.search(line):
        return _record_outcome(pending, match[1], match[2])
    if match := FAILED_LINE.search(line):
        return _record_outcome(pending, match[1], REJECTED)
    if match := ALLOC_LINE.search(line):
        records.append(_finish_pod(pending, int(match[1])))
        return None
    return pending


def _start_pod(
    index: int, pod: str, pending: _PendingPod | None, expected: int
) -> _PendingPod:
    if pending is not None:
        raise ValueError(
            f"pod {pod!r} starts before pod {pending.pod!r} has an [Alloc] line"
        )
    if index != expected:
        raise ValueError(f"attempt for {pod!r} has index {index}, expected {expected}")
    return _PendingPod(index, pod)


def _record_outcome(pending: _PendingPod | None, pod: str, outcome: str) -> _PendingPod:
    expected = pending.pod if pending else None
    if pending is None or pending.pod != pod:
        raise ValueError(
            f"outcome line names pod {pod!r}, expected the pod in flight {expected!r}"
        )
    pending.outcome = outcome
    return pending


def _finish_pod(pending: _PendingPod | None, arrived_gpu_milli: int) -> ArrivalRecord:
    if pending is None:
        raise ValueError(
            f"[Alloc] line ({arrived_gpu_milli} milli) has no pod in flight"
        )
    if pending.outcome is None:
        raise ValueError(
            f"[Alloc] line for pod {pending.pod!r} precedes its outcome line"
        )
    node = pending.outcome or None
    return ArrivalRecord(pending.index, pending.pod, node, arrived_gpu_milli)
