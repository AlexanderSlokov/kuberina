"""Tests for reading the Alibaba `cluster-trace-gpu-v2023` CSV files (backlog X-2)."""

from pathlib import Path

import pytest

from alibaba.trace_csv import (
    TraceNode,
    TracePod,
    read_trace_nodes,
    read_trace_pods,
    source_pod_name,
)

FIXTURES = Path(__file__).parent / "fixtures" / "alibaba"


def test_node_row_keeps_cpu_memory_gpu_count_and_model():
    nodes = read_trace_nodes(FIXTURES / "nodes.csv")
    assert nodes[2] == TraceNode("openb-node-0002", 96000, 393216, 8, "G2")


def test_pods_are_keyed_by_trace_name():
    pods = read_trace_pods(FIXTURES / "pods.csv")
    assert pods["openb-pod-0001"] == TracePod("openb-pod-0001", 3152, 5600, 1, 810)


def test_multi_gpu_pod_demand_is_device_count_times_milli_per_device():
    assert TracePod("p", 32000, 1024, 8, 1000).gpu_milli_total == 8000


def test_cpu_only_pod_has_zero_gpu_demand():
    assert TracePod("p", 8000, 1024, 0, 0).gpu_milli_total == 0


def test_tuned_clone_resolves_to_its_source_row():
    assert source_pod_name("paib-gpu/openb-pod-3888-tuned-0") == "openb-pod-3888"


def test_original_pod_resolves_to_itself():
    assert source_pod_name("paib-gpu/openb-pod-0255") == "openb-pod-0255"


def test_missing_column_is_an_error_naming_the_column(tmp_path):
    broken = tmp_path / "nodes.csv"
    broken.write_text("sn,cpu_milli,memory_mib,gpu\nopenb-node-0000,64000,262144,2\n")
    with pytest.raises(ValueError, match="model"):
        read_trace_nodes(broken)
