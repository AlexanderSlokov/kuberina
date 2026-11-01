"""Tests for reading an IR v1 workloads document (ir-v1.md §3.1 and §3.1.1)."""

import pytest

from ir_workloads import replica_name, resolve_targets, unroll_workloads

TRAINING_JOB = {
    "namespaces": {
        "ai": [
            {"name": "trainer", "replicas": 2, "requests": {"cpu": 8, "gpu": 1}},
            {
                "name": "parameter-server",
                "requests": {"cpu": 4, "network": {"in": 100}},
                "affinity": ["ai/trainer"],
            },
        ]
    }
}


def test_replica_names_follow_the_unrolling_rule():
    assert replica_name("worker", 0, 18) == "worker-0000"
    assert replica_name("grafana", 0, 1) == "grafana-0"


def test_unrolling_yields_one_pod_per_replica_keyed_by_identity():
    pods = unroll_workloads(TRAINING_JOB)
    assert [p["identity"] for p in pods] == [
        "ai/trainer-0000",
        "ai/trainer-0001",
        "ai/parameter-server-0",
    ]
    assert pods[2]["requests"] == {"cpu": 4.0, "net_in": 100.0}


def test_declaration_target_resolves_to_every_replica():
    pods = resolve_targets(unroll_workloads(TRAINING_JOB))
    assert pods[2]["affinity"] == ["ai/trainer-0000", "ai/trainer-0001"]


def test_own_declaration_excludes_the_pod_itself():
    doc = {"namespaces": {"ai": [{"name": "web", "replicas": 2, "antiAffinity": ["ai/web"]}]}}
    pods = resolve_targets(unroll_workloads(doc))
    assert pods[0]["antiAffinity"] == ["ai/web-0001"]


def test_unresolvable_target_names_the_value():
    doc = {"namespaces": {"ai": [{"name": "web", "affinity": ["web"]}]}}
    with pytest.raises(ValueError, match="'web'"):
        resolve_targets(unroll_workloads(doc))
