"""Tests for the inspector's independent affinity check (ir-v1.md §3.1.1)."""

from affinity_check import count_affinity_violations


def pod(identity, affinity=(), anti_affinity=()):
    """A resolved pod; each affinity entry is a group of identities."""
    return {"identity": identity, "affinity": [list(e) for e in affinity],
            "antiAffinity": list(anti_affinity)}


def test_separated_affinity_pair_is_one_violation():
    pods = [pod("ns/a", affinity=[["ns/b"]]), pod("ns/b")]
    assert count_affinity_violations(pods, {"ns/a": "n1", "ns/b": "n2"}) == 1


def test_colocated_anti_affinity_pair_is_one_violation():
    pods = [pod("ns/a"), pod("ns/b", anti_affinity=["ns/a"])]
    assert count_affinity_violations(pods, {"ns/a": "n1", "ns/b": "n1"}) == 1


def test_anti_affinity_pair_declared_from_both_sides_counts_once():
    pods = [pod("ns/a", anti_affinity=["ns/b"]), pod("ns/b", anti_affinity=["ns/a"])]
    assert count_affinity_violations(pods, {"ns/a": "n1", "ns/b": "n1"}) == 1


def test_affinity_entry_is_satisfied_by_one_of_its_pods():
    pods = [pod("ns/a", affinity=[["ns/b", "ns/c"]]), pod("ns/b"), pod("ns/c")]
    assert count_affinity_violations(pods, {"ns/a": "n1", "ns/b": "n1", "ns/c": "n2"}) == 0


def test_affinity_entry_with_no_named_pod_nearby_is_one_violation():
    pods = [pod("ns/a", affinity=[["ns/b", "ns/c"]]), pod("ns/b"), pod("ns/c")]
    assert count_affinity_violations(pods, {"ns/a": "n1", "ns/b": "n2", "ns/c": "n3"}) == 1


def test_satisfied_rules_are_not_violations():
    pods = [pod("ns/a", affinity=[["ns/b"]], anti_affinity=["ns/c"]), pod("ns/b"), pod("ns/c")]
    assert count_affinity_violations(pods, {"ns/a": "n1", "ns/b": "n1", "ns/c": "n2"}) == 0


def test_rule_with_an_unplaced_pod_is_not_judged():
    pods = [pod("ns/a", affinity=[["ns/c"]], anti_affinity=["ns/b"]), pod("ns/b"), pod("ns/c")]
    assert count_affinity_violations(pods, {}) == 0
