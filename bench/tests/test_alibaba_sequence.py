"""Tests for `sequence.csv`, the per-arrival table Stage 6 compares against (X-2)."""

import io

from alibaba.sequence import build_sequence_rows, write_sequence_csv
from alibaba.simulator_log import ArrivalRecord
from alibaba.trace_csv import TracePod

PODS = {"openb-pod-0001": TracePod("openb-pod-0001", 3152, 5600, 1, 810)}
FGD = [
    ArrivalRecord(0, "paib-gpu/openb-pod-0001", "openb-node-0000", 810),
    ArrivalRecord(1, "paib-gpu/openb-pod-0001-tuned-0", None, 1620),
]
RANDOM = [
    ArrivalRecord(0, "paib-gpu/openb-pod-0001", None, 810),
    ArrivalRecord(1, "paib-gpu/openb-pod-0001-tuned-0", "openb-node-0002", 1620),
]


def rows():
    return build_sequence_rows({"FGD": FGD, "Random": RANDOM}, PODS)


def test_row_names_ir_identity_and_source_row():
    row = rows()[1]
    assert (row["pod"], row["source_pod"]) == (
        "paib-gpu/openb-pod-0001-tuned-0-0",
        "openb-pod-0001",
    )


def test_row_carries_requests_in_trace_units():
    row = rows()[0]
    assert (row["cpu_milli"], row["memory_mib"], row["gpu_milli"]) == (3152, 5600, 810)


def test_each_policy_column_holds_its_node_or_empty_when_rejected():
    assert [(r["node_FGD"], r["node_Random"]) for r in rows()] == [
        ("openb-node-0000", ""),
        ("", "openb-node-0002"),
    ]


def test_csv_header_lists_policy_columns_in_the_order_given():
    stream = io.StringIO()
    write_sequence_csv(rows(), stream)
    header = stream.getvalue().splitlines()[0]
    assert (
        header
        == "arrival_index,pod,source_pod,cpu_milli,memory_mib,gpu_milli,node_FGD,node_Random"
    )
