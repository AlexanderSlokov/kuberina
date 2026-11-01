"""Acceptance checks for the trace adapter (backlog X-2).

X-2 is done when the GPU demand written to `workloads.yaml` equals the GPU demand the
simulator logged as arrived. The adapter checks this twice: per arrival against the
running total in each `[Alloc]` line, and once over the file it wrote, read back.
"""

from __future__ import annotations

from collections.abc import Mapping, Sequence

from alibaba.simulator_log import ArrivalRecord
from alibaba.trace_csv import TracePod, source_pod_name

MILLI = 1000

# A Workloads document as `yaml.safe_load` returns it.
LoadedWorkloads = Mapping[str, Mapping[str, Sequence[Mapping[str, object]]]]


def check_arrived_demand(
    records: Sequence[ArrivalRecord], pods: Mapping[str, TracePod]
) -> None:
    """Raise unless the trace demand, summed in arrival order, equals each logged total.

    Example:
        check_arrived_demand(read_simulator_log(log), read_trace_pods(pod_csv))
    """
    running_total = 0
    for record in records:
        running_total += pods[source_pod_name(record.pod)].gpu_milli_total
        if running_total != record.arrived_gpu_milli:
            raise ValueError(
                f"at arrival index {record.index} ({record.pod}) the trace sums to "
                f"{running_total} milli-GPU, the log says {record.arrived_gpu_milli}"
            )


def check_same_sequence(
    records_by_policy: Mapping[str, Sequence[ArrivalRecord]],
) -> None:
    """Raise unless every policy's log offered the same pods in the same order.

    Example:
        check_same_sequence({"FGD": fgd_records, "Random": random_records})
    """
    (reference_policy, reference), *others = records_by_policy.items()
    for policy, records in others:
        if len(records) != len(reference):
            raise ValueError(
                f"{reference_policy} logged {len(reference)} pods, {policy} logged {len(records)}"
            )
        _check_same_names(reference_policy, reference, policy, records)


def emitted_gpu_milli(workloads: LoadedWorkloads) -> int:
    """Total GPU demand of a Workloads document in milli-GPU, replicas included.

    Example:
        >>> emitted_gpu_milli({"namespaces": {"a": [{"name": "x", "requests": {"gpu": 0.5}}]}})
        500
    """
    return sum(
        _declaration_gpu_milli(declaration)
        for declarations in workloads["namespaces"].values()
        for declaration in declarations
    )


def _check_same_names(
    reference_policy: str,
    reference: Sequence[ArrivalRecord],
    policy: str,
    records: Sequence[ArrivalRecord],
) -> None:
    for expected, actual in zip(reference, records):
        if expected.pod != actual.pod:
            raise ValueError(
                f"arrival index {expected.index}: {reference_policy} offered {expected.pod!r}, "
                f"{policy} offered {actual.pod!r}"
            )


def _declaration_gpu_milli(declaration: Mapping[str, object]) -> int:
    requests = declaration.get("requests", {})
    gpu = float(requests.get("gpu", 0)) if isinstance(requests, Mapping) else 0.0
    replicas = int(declaration.get("replicas", 1))  # type: ignore[call-overload]
    # Rounding recovers the integer the adapter divided by 1000; no trace value is finer.
    return round(gpu * MILLI) * replicas
