"""Tests for the inspector's admission checks (ir-v1.md §4 and §4.1, backlog X-4)."""

from admission_check import (
    admission_report,
    arrival_ranks,
    gpu_allocation,
    in_order_violations,
    placement_problems,
)
from ir_workloads import unroll_workloads

NODE = {"name": "gpu-node-01", "labels": {}, "allocatable": {"gpu": 8.0}, "gpuDevices": 8}


def pods_with_gpus(*gpus):
    """One single-replica declaration per GPU count, arriving in the order given."""
    return unroll_workloads({"namespaces": {"ns": [
        {"name": name, "arrival": i, "requests": {"gpu": g}}
        for i, (name, g) in enumerate(zip("abcdefgh", gpus))
    ]}})


def test_a_consistent_blueprint_has_no_placement_problems():
    pods = pods_with_gpus(4, 4)
    assert placement_problems(pods, {"ns/a-0": "gpu-node-01"}, ["ns/b-0"]) == []


def test_pod_in_neither_list_is_missing():
    pods = pods_with_gpus(4, 4)
    assert placement_problems(pods, {"ns/a-0": "gpu-node-01"}, []) == ["ns/b-0 is in neither solution nor unplaced"]


def test_pod_in_both_lists_is_reported():
    pods = pods_with_gpus(4)
    problems = placement_problems(pods, {"ns/a-0": "gpu-node-01"}, ["ns/a-0"])
    assert problems == ["ns/a-0 is in both solution and unplaced"]


def test_unknown_identity_is_reported():
    pods = pods_with_gpus(4)
    problems = placement_problems(pods, {"ns/a-0": "gpu-node-01"}, ["ns/ghost-0"])
    assert problems == ["ns/ghost-0 is not a pod of the workloads"]


def test_spec_example_that_holds_has_no_violation():
    # ir-v1.md §4.1: A 5 GPUs, B 4 unplaced, C 1. Removing C leaves 3 free.
    pods = pods_with_gpus(5, 4, 1)
    solution = {"ns/a-0": "gpu-node-01", "ns/c-0": "gpu-node-01"}
    assert in_order_violations([NODE], pods, solution, ["ns/b-0"]) == []


def test_spec_example_that_breaks_the_rule_names_both_pods():
    # A 4 GPUs, B 4 unplaced, C 2: B could take the place of C.
    pods = pods_with_gpus(4, 4, 2)
    solution = {"ns/a-0": "gpu-node-01", "ns/c-0": "gpu-node-01"}
    assert in_order_violations([NODE], pods, solution, ["ns/b-0"]) == [("ns/b-0", "ns/c-0")]


def test_earlier_placed_pod_is_never_a_place_to_take():
    # B arrived after A, so B may not take A's place even though it would fit.
    pods = pods_with_gpus(8, 8)
    assert in_order_violations([NODE], pods, {"ns/a-0": "gpu-node-01"}, ["ns/b-0"]) == []


def test_swap_that_fits_the_sum_but_not_a_device_is_no_violation():
    # Eight 600-milli Pods, then a 500-milli Pod unplaced, then a 400-milli Pod placed
    # on a device that already holds 600. Taking its place frees 400 on that device:
    # not enough for 500, although the GPU sum would allow it.
    pods = unroll_workloads({"namespaces": {"ns": [
        {"name": "sixty", "arrival": 0, "replicas": 8, "requests": {"gpu": 1}, "gpuMilli": 600},
        {"name": "half", "arrival": 1, "requests": {"gpu": 1}, "gpuMilli": 500},
        {"name": "small", "arrival": 2, "requests": {"gpu": 1}, "gpuMilli": 400},
    ]}})
    solution = {p["identity"]: "gpu-node-01" for p in pods if p["identity"] != "ns/half-0"}
    assert in_order_violations([NODE], pods, solution, ["ns/half-0"]) == []


def test_replicas_arrive_one_after_another_in_replica_order():
    pods = unroll_workloads({"namespaces": {"ns": [
        {"name": "late", "arrival": 9, "replicas": 2},
        {"name": "early", "arrival": 2},
    ]}})
    assert arrival_ranks(pods) == {"ns/early-0": 0, "ns/late-0000": 1, "ns/late-0001": 2}


def test_gpu_allocation_counts_placed_gpu_against_capacity():
    pods = pods_with_gpus(6, 4)
    assert gpu_allocation([NODE], pods, {"ns/a-0": "gpu-node-01"}) == (6.0, 8.0)


def test_report_of_an_in_order_blueprint_states_problems_violations_and_allocation():
    pods = pods_with_gpus(4, 4, 2)
    blueprint = {
        "metadata": {"admission": "in-order"},
        "solution": {"ns/a-0": "gpu-node-01", "ns/c-0": "gpu-node-01"},
        "unplaced": ["ns/b-0"],
    }
    assert admission_report([NODE], pods, blueprint) == [
        "Placement Problems: 0",
        "GPU Allocation: 6.00 of 8 GPUs (75.00%)",
        "In-Order Violations: 1",
        "  ns/b-0 could take the place of ns/c-0",
    ]


def test_report_without_admission_checks_placement_only():
    pods = pods_with_gpus(4)
    assert admission_report([NODE], pods, {"solution": {"ns/a-0": "gpu-node-01"}}) == ["Placement Problems: 0"]
