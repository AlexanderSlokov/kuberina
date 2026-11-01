"""Write `sequence.csv`: one row per arrival, with each online policy's decision.

Stage 6 of the plan compares Kuberina against the simulator's policies pod by pod. This
table is that comparison's input: the arrival index, the pod's IR identity and trace
row, its requests in trace units, and the node each policy chose (empty when the policy
rejected the pod).
"""

from __future__ import annotations

import csv
from collections.abc import Mapping, Sequence
from typing import TextIO

from alibaba.ir_documents import pod_identity
from alibaba.simulator_log import ArrivalRecord
from alibaba.trace_csv import TracePod, source_pod_name

SequenceRow = dict[str, str | int]


def build_sequence_rows(
    records_by_policy: Mapping[str, Sequence[ArrivalRecord]],
    pods: Mapping[str, TracePod],
) -> list[SequenceRow]:
    """One row per arrival; the first policy's log supplies the pod order.

    Call `check_same_sequence` first: this function trusts that every log offered the
    same pods in the same order.

    Example:
        rows = build_sequence_rows({"FGD": fgd_records}, read_trace_pods(pod_csv))
    """
    reference = next(iter(records_by_policy.values()))
    return [
        _row(position, record, records_by_policy, pods)
        for position, record in enumerate(reference)
    ]


def write_sequence_csv(rows: Sequence[SequenceRow], stream: TextIO) -> None:
    """Write the rows as CSV with a header; policy columns follow the fixed columns.

    Example:
        with open("sequence.csv", "w", newline="") as out:
            write_sequence_csv(rows, out)
    """
    writer = csv.DictWriter(stream, fieldnames=list(rows[0]), lineterminator="\n")
    writer.writeheader()
    writer.writerows(rows)


def _row(
    position: int,
    record: ArrivalRecord,
    records_by_policy: Mapping[str, Sequence[ArrivalRecord]],
    pods: Mapping[str, TracePod],
) -> SequenceRow:
    source = pods[source_pod_name(record.pod)]
    row: SequenceRow = {
        "arrival_index": record.index,
        "pod": pod_identity(record),
        "source_pod": source.name,
        "cpu_milli": source.cpu_milli,
        "memory_mib": source.memory_mib,
        "gpu_milli": source.gpu_milli_total,
    }
    for policy, records in records_by_policy.items():
        row[f"node_{policy}"] = records[position].node or ""
    return row
