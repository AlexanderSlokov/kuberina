"""Tests for the X-2 acceptance checks: emitted GPU demand equals logged arrivals."""

import pytest

from alibaba.demand_check import (
    check_arrived_demand,
    check_same_sequence,
    emitted_gpu_milli,
)
from alibaba.simulator_log import ArrivalRecord
from alibaba.trace_csv import TracePod

PODS = {"openb-pod-0001": TracePod("openb-pod-0001", 3152, 5600, 1, 810)}
GOOD = [
    ArrivalRecord(0, "paib-gpu/openb-pod-0001", "openb-node-0000", 810),
    ArrivalRecord(1, "paib-gpu/openb-pod-0001-tuned-0", None, 1620),
]


def test_matching_running_totals_pass():
    check_arrived_demand(GOOD, PODS)


def test_running_total_mismatch_names_index_and_both_totals():
    bad = [GOOD[0], ArrivalRecord(1, "paib-gpu/openb-pod-0001-tuned-0", None, 1700)]
    with pytest.raises(ValueError, match=r"index 1.*1620.*1700"):
        check_arrived_demand(bad, PODS)


def test_identical_sequences_across_policies_pass():
    check_same_sequence({"FGD": GOOD, "Random": GOOD})


def test_different_pod_at_one_index_names_index_and_policies():
    other = [GOOD[0], ArrivalRecord(1, "paib-gpu/openb-pod-0002", None, 1620)]
    with pytest.raises(ValueError, match=r"index 1.*FGD.*Random"):
        check_same_sequence({"FGD": GOOD, "Random": other})


def test_different_lengths_name_both_lengths():
    with pytest.raises(ValueError, match=r"2.*1"):
        check_same_sequence({"FGD": GOOD, "Random": GOOD[:1]})


def test_emitted_demand_sums_gpu_over_declarations_and_replicas():
    workloads = {
        "namespaces": {
            "a": [
                {"name": "x", "requests": {"gpu": 0.81}},
                {"name": "y", "requests": {"cpu": 1}},
            ],
            "b": [{"name": "z", "replicas": 2, "requests": {"gpu": 8}}],
        }
    }
    assert emitted_gpu_milli(workloads) == 16810


def test_emitted_demand_reads_gpu_as_devices_when_gpu_milli_is_present():
    workloads = {
        "namespaces": {
            "a": [
                {"name": "x", "requests": {"gpu": 1}, "gpuMilli": 810},
                {"name": "y", "replicas": 3, "requests": {"gpu": 2}, "gpuMilli": 250},
            ]
        }
    }
    assert emitted_gpu_milli(workloads) == 810 + 3 * 500
