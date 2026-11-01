"""Tests for turning the trace and one simulator log into Kuberina IR v1 (backlog X-2)."""

import pytest

from alibaba.ir_documents import build_infrastructure, build_workloads, pod_identity
from alibaba.simulator_log import ArrivalRecord
from alibaba.trace_csv import TraceNode, TracePod

NODE = TraceNode("openb-node-0260", 96000, 393216, 8, "G2")
PODS = {
    "openb-pod-0255": TracePod("openb-pod-0255", 8000, 30517, 0, 0),
    "openb-pod-3888": TracePod("openb-pod-3888", 11400, 40960, 1, 810),
}
RECORDS = [
    ArrivalRecord(0, "paib-gpu/openb-pod-3888", "openb-node-0260", 810),
    ArrivalRecord(1, "paib-gpu/openb-pod-0255", None, 810),
    ArrivalRecord(2, "paib-gpu/openb-pod-3888-tuned-0", None, 1620),
]


def test_infrastructure_is_an_ir_v1_document():
    infra = build_infrastructure([NODE])
    assert (infra["apiVersion"], infra["kind"]) == ("kuberina.io/v1", "Infrastructure")


def test_node_allocatable_is_cores_gib_and_whole_gpus():
    node = build_infrastructure([NODE])["nodes"][0]
    assert node["allocatable"] == {"cpu": 96.0, "ram": 384.0, "gpu": 8}


def test_node_carries_its_gpu_model_as_a_label():
    assert build_infrastructure([NODE])["nodes"][0]["labels"] == {"gpu-model": "G2"}


def test_workloads_declare_one_pod_per_arrival_in_log_order():
    declarations = build_workloads(RECORDS, PODS)["namespaces"]["paib-gpu"]
    assert [d["name"] for d in declarations] == [
        "openb-pod-3888",
        "openb-pod-0255",
        "openb-pod-3888-tuned-0",
    ]


def test_clone_has_the_requests_of_its_source_row():
    declarations = build_workloads(RECORDS, PODS)["namespaces"]["paib-gpu"]
    assert declarations[2]["requests"] == declarations[0]["requests"]


def test_fractional_gpu_request_is_milli_over_one_thousand():
    request = build_workloads(RECORDS, PODS)["namespaces"]["paib-gpu"][0]["requests"]
    assert request == {"cpu": 11.4, "ram": 40.0, "gpu": 0.81}


def test_cpu_only_pod_omits_gpu():
    request = build_workloads(RECORDS, PODS)["namespaces"]["paib-gpu"][1]["requests"]
    assert "gpu" not in request


def test_logged_pod_missing_from_the_trace_is_an_error_naming_it():
    stray = [ArrivalRecord(0, "paib-gpu/openb-pod-9999", None, 0)]
    with pytest.raises(ValueError, match="openb-pod-9999"):
        build_workloads(stray, PODS)


def test_pod_identity_follows_ir_replica_unrolling():
    # ir-v1.md §3.1: a declaration with one replica unrolls to `<name>-0`.
    assert pod_identity(RECORDS[2]) == "paib-gpu/openb-pod-3888-tuned-0-0"
