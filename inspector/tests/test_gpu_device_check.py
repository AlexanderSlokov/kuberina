"""Tests for the inspector's per-device GPU check (ir-v1.md §3.4, backlog X-3)."""

import pytest

from gpu_device_check import device_shape, device_violations, fits_on_devices
from ir_workloads import unroll_workloads

SIXTY = (1, 600)
HALF = (1, 500)
WHOLE = (1, 1000)


def test_half_gpu_does_not_fit_eight_devices_each_sixty_percent_used():
    # X-3 done-when: 3.2 GPUs free in total, no device with 500 milli.
    assert not fits_on_devices(8, [SIXTY] * 8 + [HALF])


def test_eight_sixty_percent_pods_fit_eight_devices():
    assert fits_on_devices(8, [SIXTY] * 8)


def test_search_finds_an_assignment_that_descending_best_fit_misses():
    # {500, 300, 200} and {400, 300, 300} fill two devices exactly. Descending best
    # fit puts 400 next to 500 and strands the 200; the solver's decode does that
    # (gpu_devices.rs, decode_may_decline_a_set_that_fits), and the inspector must not.
    assert fits_on_devices(2, [(1, m) for m in (500, 400, 300, 300, 300, 200)])


def test_multi_device_pod_needs_distinct_devices():
    assert fits_on_devices(2, [(2, 500), (2, 500)])
    assert not fits_on_devices(1, [(2, 500)])


def test_whole_device_pods_need_empty_devices():
    assert not fits_on_devices(2, [(1, 100), WHOLE, WHOLE])


def test_shape_of_a_shared_pod_is_devices_and_milli():
    pod = {"requests": {"gpu": 0.5}, "gpuDevices": 1, "gpuMilli": 500}
    assert device_shape(pod) == HALF


def test_shape_of_a_whole_gpu_pod_uses_whole_devices():
    assert device_shape({"requests": {"gpu": 2.0}}) == (2, 1000)


def test_cpu_only_pod_has_no_shape():
    assert device_shape({"requests": {"cpu": 4.0}}) is None


def test_fractional_gpu_without_gpu_milli_has_no_device_shape():
    with pytest.raises(ValueError, match=r"ns/p-0.*0.5.*gpuMilli"):
        device_shape({"identity": "ns/p-0", "requests": {"gpu": 0.5}})


def test_violations_name_only_device_nodes_whose_pods_do_not_fit():
    pods = unroll_workloads({"namespaces": {"inference": [
        {"name": "sixty", "replicas": 8, "requests": {"gpu": 1}, "gpuMilli": 600},
        {"name": "half", "requests": {"gpu": 1}, "gpuMilli": 500},
    ]}})
    nodes = [
        {"name": "full", "allocatable": {"gpu": 8.0}, "gpuDevices": 8},
        {"name": "summed", "allocatable": {"gpu": 8.0}},
    ]
    on_full = {p["identity"]: "full" for p in pods}
    on_summed = {p["identity"]: "summed" for p in pods}
    assert device_violations(nodes, pods, on_full) == ["full"]
    assert device_violations(nodes, pods, on_summed) == []
