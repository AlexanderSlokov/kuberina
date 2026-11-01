"""Tests for validate_solution in inspector.py."""

import pytest

from inspector import normalize_ir, validate_solution
from ir_workloads import unroll_workloads

NODES = [
    {"name": "n1", "labels": {}, "allocatable": {"cpu": 4.0}},
    {"name": "n2", "labels": {}, "allocatable": {"cpu": 4.0}},
]


def test_pods_with_the_same_name_in_two_namespaces_are_kept_apart():
    # Regression: the inspector keyed pods by bare name, so web-0 in team-a and
    # web-0 in team-b collapsed into one pod and one load went uncounted.
    pods = unroll_workloads({"namespaces": {
        "team-a": [{"name": "web", "requests": {"cpu": 3}}],
        "team-b": [{"name": "web", "requests": {"cpu": 2}}],
    }})
    result = validate_solution(NODES, pods, {"team-a/web-0": "n1", "team-b/web-0": "n1"})
    assert result["node_loads"]["n1"]["cpu"] == 5.0
    assert result["capacity_overflow"]["cpu"] == 1.0


def test_solution_breaking_a_device_but_not_the_gpu_sum_is_a_device_violation():
    # X-3 done-when, inspector half: 4.8 + 0.5 of 8 GPUs fits the sum, not a device.
    pods = unroll_workloads({"namespaces": {"inference": [
        {"name": "sixty", "replicas": 8, "requests": {"gpu": 1}, "gpuMilli": 600},
        {"name": "half", "requests": {"gpu": 1}, "gpuMilli": 500},
    ]}})
    nodes = [{"name": "gpu-node-01", "labels": {}, "allocatable": {"gpu": 8.0}, "gpuDevices": 8}]
    result = validate_solution(nodes, pods, {p["identity"]: "gpu-node-01" for p in pods})
    assert result["capacity_overflow"]["gpu"] == 0.0
    assert result["gpu_device_violations"] == ["gpu-node-01"]


def test_gpu_devices_set_an_absent_allocatable_gpu():
    infra = {"nodes": [{"name": "gpu-node-01", "allocatable": {"cpu": 96}, "gpuDevices": 8}]}
    normalize_ir(infra)
    assert infra["nodes"][0]["allocatable"]["gpu"] == 8.0


def test_gpu_devices_disagreeing_with_allocatable_gpu_names_the_node():
    infra = {"nodes": [{"name": "gpu-node-01", "allocatable": {"gpu": 4}, "gpuDevices": 8}]}
    with pytest.raises(ValueError, match="gpu-node-01"):
        normalize_ir(infra)
