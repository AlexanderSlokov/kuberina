"""Phase 2 GA: early-stopping improvement threshold.

Regression coverage for #20, mirroring `solver/src/phase2_ga.rs`. The Rust solver is
the shipping implementation; these tests keep the reference implementation honest
about the same semantics.
"""

from __future__ import annotations

import logging

from kuberina.fitness import compute_fitness
from kuberina.model.types import (
    FFDWeights,
    FitnessWeights,
    GAConfig,
)
from kuberina.parser import load_infra, load_workloads
from kuberina.phases.phase0 import pre_deduct_daemonsets
from kuberina.phases.phase1_ffd import ffd_warmstart
from kuberina.phases.phase2_ga import _exceeds_improvement_threshold, run_ga


INFRA_PATH = "testdata/homelab_infra.yaml"
WORKLOADS_PATH = "testdata/homelab_workloads.yaml"


def test_threshold_ignores_gains_below_the_bound() -> None:
    """0.0003 on a fitness of 1.58 million is the gain that broke #20."""
    assert not _exceeds_improvement_threshold(1582568.126, 1582568.1257, 1e-4)
    # One node emptied: 6.2e-3 relative, comfortably over the bound.
    assert _exceeds_improvement_threshold(1582553.117, 1572736.618, 1e-4)


def test_threshold_rejects_non_gains_and_scales_to_the_anchor() -> None:
    assert not _exceeds_improvement_threshold(100.0, 100.0, 1e-4)
    assert not _exceeds_improvement_threshold(100.0, 101.0, 1e-4)
    # Exactly at the bound is not exceeding it. Binary-exact values, so the
    # assertion tests the comparison rather than float representation.
    assert not _exceeds_improvement_threshold(128.0, 64.0, 0.5)
    assert _exceeds_improvement_threshold(128.0, 63.0, 0.5)
    # A zero anchor has no scale to measure against, so any gain counts.
    assert _exceeds_improvement_threshold(0.0, -1e-9, 1e-4)


def test_run_stops_early_when_gains_stay_under_the_threshold(
    caplog: logging.LogCaptureFixture,
) -> None:
    """A run whose gains never exceed the threshold must terminate early (#20).

    The threshold is set to 50% so no realistic generation qualifies as progress,
    which is the situation the MSC Irina benchmark was in at 1e-4.
    """
    raw_nodes, daemon_sets = load_infra(INFRA_PATH)
    pods, groups = load_workloads(WORKLOADS_PATH)
    nodes = pre_deduct_daemonsets(raw_nodes, daemon_sets)

    weights = FitnessWeights()
    seed = ffd_warmstart(pods, nodes, FFDWeights())
    seed.fitness = compute_fitness(seed, pods, nodes, groups, weights)

    config = GAConfig(
        population_size=16,
        max_generations=200,
        early_stop_generations=5,
        min_relative_improvement=0.5,
    )

    with caplog.at_level(logging.INFO, logger="kuberina.phases.phase2_ga"):
        best = run_ga(seed, pods, nodes, groups, config, weights)

    stops = [r for r in caplog.records if r.getMessage().startswith("Early stop")]
    assert stops, "run burned its whole budget instead of stopping early"
    # Generation index is zero-based, so stopping on the Nth stale generation
    # reports N - 1.
    assert stops[0].args[0] <= config.early_stop_generations
    assert best.fitness <= seed.fitness
