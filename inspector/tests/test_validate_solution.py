"""Tests for validate_solution in inspector.py."""

from inspector import validate_solution
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
