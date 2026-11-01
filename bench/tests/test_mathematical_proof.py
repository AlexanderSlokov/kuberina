"""Tests for the proof script's feasibility checks on IR v1 workloads."""

from mathematical_proof import (
    load_pods,
    verify_assignment_constraint,
    verify_capacity_constraint,
)

NODES = [
    {"name": "n1", "labels": {}, "allocatable": {"cpu": 4.0}},
    {"name": "n2", "labels": {}, "allocatable": {"cpu": 4.0}},
]
TWO_TEAMS = {"namespaces": {
    "team-a": [{"name": "web", "requests": {"cpu": 3}}],
    "team-b": [{"name": "web", "requests": {"cpu": 2}}],
}}


def test_pods_load_from_namespaces_keyed_by_identity():
    # Regression for #26: the script read a top-level `pods:` list, which IR v1 rejects.
    assert [p["identity"] for p in load_pods(TWO_TEAMS)] == ["team-a/web-0", "team-b/web-0"]


def test_capacity_counts_same_named_pods_in_two_namespaces():
    # Regression: pods were keyed by bare name, so web-0 in two namespaces merged.
    pods = load_pods(TWO_TEAMS)
    ok, overflow = verify_capacity_constraint(NODES, pods, {"team-a/web-0": "n1", "team-b/web-0": "n1"})
    assert not ok and overflow["cpu"] == 1.0


def test_assignment_finds_the_unassigned_pod_by_identity():
    pods = load_pods(TWO_TEAMS)
    assert verify_assignment_constraint(pods, {"team-a/web-0": "n1"}) == (False, 1)
