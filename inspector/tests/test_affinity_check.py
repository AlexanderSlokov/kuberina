"""Tests for the inspector's independent affinity check (ir-v1.md §3.1.1)."""

from affinity_check import count_affinity_violations


def pod(identity, affinity=(), anti_affinity=()):
    return {"identity": identity, "affinity": list(affinity), "antiAffinity": list(anti_affinity)}


def test_separated_affinity_pair_is_one_violation():
    pods = [pod("ns/a", affinity=["ns/b"]), pod("ns/b")]
    assert count_affinity_violations(pods, {"ns/a": "n1", "ns/b": "n2"}) == 1


def test_colocated_anti_affinity_pair_is_one_violation():
    pods = [pod("ns/a"), pod("ns/b", anti_affinity=["ns/a"])]
    assert count_affinity_violations(pods, {"ns/a": "n1", "ns/b": "n1"}) == 1


def test_rule_declared_from_both_sides_counts_once():
    pods = [pod("ns/a", affinity=["ns/b"]), pod("ns/b", affinity=["ns/a"])]
    assert count_affinity_violations(pods, {"ns/a": "n1", "ns/b": "n2"}) == 1


def test_satisfied_rules_are_not_violations():
    pods = [pod("ns/a", affinity=["ns/b"], anti_affinity=["ns/c"]), pod("ns/b"), pod("ns/c")]
    assert count_affinity_violations(pods, {"ns/a": "n1", "ns/b": "n1", "ns/c": "n2"}) == 0


def test_rule_with_an_unplaced_pod_is_not_judged():
    pods = [pod("ns/a", anti_affinity=["ns/b"]), pod("ns/b")]
    assert count_affinity_violations(pods, {}) == 0
