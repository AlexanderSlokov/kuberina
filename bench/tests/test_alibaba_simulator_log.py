"""Tests for reading the FGD simulator's per-pod log (backlog X-2)."""

import gzip
from pathlib import Path

import pytest

from alibaba.simulator_log import ArrivalRecord, parse_simulator_log, read_simulator_log

FIXTURES = Path(__file__).parent / "fixtures" / "alibaba"
ATTEMPT = 'level=info msg="[{i}] attempt to create pod(paib-gpu/{pod})\\n"'
PLACED = 'level=info msg="pod(paib-gpu/{pod}) is scheduled to node(0000-{node})\\n"'
ALLOC = 'level=info msg="[Alloc]; Used nodes: 1; Arrived GPU Milli: {milli}\\n"'


def fgd_records():
    return read_simulator_log(FIXTURES / "fgd.simon.log")


def test_one_record_per_attempt_in_arrival_order():
    names = [record.pod for record in fgd_records()]
    assert names == [
        "paib-gpu/openb-pod-0000",
        "paib-gpu/openb-pod-0001",
        "paib-gpu/openb-pod-0002",
        "paib-gpu/openb-pod-0003",
        "paib-gpu/openb-pod-0002-tuned-0",
    ]


def test_placed_pod_carries_trace_node_name_without_simulator_prefix():
    # The simulator prefixes node names with its own index: 0000-openb-node-0002.
    assert fgd_records()[0] == ArrivalRecord(
        0, "paib-gpu/openb-pod-0000", "openb-node-0002", 0
    )


def test_rejected_pod_has_no_node():
    assert fgd_records()[4].node is None


def test_arrived_gpu_milli_is_read_from_the_alloc_line_after_each_pod():
    assert [r.arrived_gpu_milli for r in fgd_records()] == [0, 810, 8810, 9810, 17810]


def test_gzipped_log_reads_the_same_as_plain(tmp_path):
    packed = tmp_path / "simon.log.gz"
    packed.write_bytes(gzip.compress((FIXTURES / "fgd.simon.log").read_bytes()))
    assert read_simulator_log(packed) == fgd_records()


def test_out_of_order_attempt_index_is_an_error_naming_both_indices():
    lines = [ATTEMPT.format(i=1, pod="openb-pod-0000")]
    with pytest.raises(ValueError, match=r"index 1.*expected 0"):
        parse_simulator_log(lines)


def test_alloc_line_without_an_outcome_is_an_error_naming_the_pod():
    lines = [ATTEMPT.format(i=0, pod="openb-pod-0000"), ALLOC.format(milli=0)]
    with pytest.raises(ValueError, match="openb-pod-0000"):
        parse_simulator_log(lines)


def test_log_ending_mid_pod_is_an_error_naming_the_pod():
    lines = [
        ATTEMPT.format(i=0, pod="openb-pod-0000"),
        PLACED.format(pod="openb-pod-0000", node="openb-node-0002"),
    ]
    with pytest.raises(ValueError, match="openb-pod-0000"):
        parse_simulator_log(lines)


def test_outcome_for_a_different_pod_is_an_error_naming_both_pods():
    lines = [
        ATTEMPT.format(i=0, pod="openb-pod-0000"),
        PLACED.format(pod="openb-pod-0009", node="openb-node-0002"),
    ]
    with pytest.raises(ValueError, match="openb-pod-0009.*openb-pod-0000"):
        parse_simulator_log(lines)
