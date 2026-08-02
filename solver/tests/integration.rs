//! Integration test: end-to-end homelab scenario.
//!
//! Ported from `research/tests/test_integration.py`.
//! Load testdata → Phase 0 → Phase 1 → Phase 2 → verify constraints.

use kuberina_solver::csp::compute_capacity_overflow;
use kuberina_solver::fitness::{compute_affinity_violations, compute_fitness};
use kuberina_solver::model::{FfdWeights, FitnessWeights, GaConfig};
use kuberina_solver::parser::{load_infra, load_workloads};
use kuberina_solver::phase0::pre_deduct_daemonsets;
use kuberina_solver::phase1_ffd::ffd_warmstart;
use kuberina_solver::phase2_ga::run_ga;

const INFRA: &str = "testdata/homelab_infra.yaml";
const WORKLOADS: &str = "testdata/homelab_workloads.yaml";

/// Run the complete 3-phase pipeline on the homelab scenario.
///
/// Assertions:
/// 1. All 10 pods are assigned to valid nodes (0, 1, or 2)
/// 2. No node exceeds capacity (hard constraint #1)
/// 3. Home Assistant + Zigbee2MQTT are on thinkcentre-beta (USB dongle)
/// 4. GA improves upon FFD seed fitness
#[test]
fn full_pipeline_homelab() {
    let (raw_nodes, daemon_sets) = load_infra(INFRA).unwrap();
    let (pods, groups) = load_workloads(WORKLOADS).unwrap();

    // Phase 0
    let nodes = pre_deduct_daemonsets(&raw_nodes, &daemon_sets);

    // WHY: verify DaemonSet overhead was subtracted
    assert!(nodes[0].allocatable.cpu < raw_nodes[0].allocatable.cpu);
    assert!(nodes[0].allocatable.ram < raw_nodes[0].allocatable.ram);

    // Phase 1: FFD
    let mut seed = ffd_warmstart(&pods, &nodes, &FfdWeights::default());
    let weights = FitnessWeights::default();
    let (fitness, sc) = compute_fitness(&seed, &pods, &nodes, &groups, &weights);
    seed.fitness = fitness;
    seed.scorecard = sc;

    // All pods assigned to valid nodes
    assert!(seed.assignment.iter().all(|&idx| idx < nodes.len()));

    // Phase 2: GA
    let config = GaConfig {
        population_size: 64, // small for test speed
        max_generations: 100,
        early_stop_generations: 30,
        ..GaConfig::default()
    };
    let best = run_ga(&seed, &pods, &nodes, &groups, &config, &weights);

    // 1. All pods assigned
    assert_eq!(best.assignment.len(), pods.len());
    assert!(best.assignment.iter().all(|&idx| idx < nodes.len()));

    // 2. No capacity violation
    assert_eq!(
        compute_capacity_overflow(&best.assignment, &pods, &nodes),
        0.0,
    );

    // 3. USB-dongle pods are on node 1 (thinkcentre-beta)
    let ha_idx = pods
        .iter()
        .position(|p| p.name == "home-assistant-0")
        .unwrap();
    let z2m_idx = pods.iter().position(|p| p.name == "zigbee2mqtt-0").unwrap();
    assert_eq!(
        best.assignment[ha_idx], 1,
        "Home Assistant must be on beta (USB)"
    );
    assert_eq!(
        best.assignment[z2m_idx], 1,
        "Zigbee2MQTT must be on beta (USB)"
    );

    // 4. GA should improve (or at least not worsen) FFD seed
    assert!(best.fitness <= seed.fitness);
}

#[test]
fn ffd_produces_feasible_seed() {
    let (raw_nodes, daemon_sets) = load_infra(INFRA).unwrap();
    let (pods, _groups) = load_workloads(WORKLOADS).unwrap();
    let nodes = pre_deduct_daemonsets(&raw_nodes, &daemon_sets);
    let seed = ffd_warmstart(&pods, &nodes, &FfdWeights::default());

    assert_eq!(
        compute_capacity_overflow(&seed.assignment, &pods, &nodes),
        0.0
    );
}

#[test]
fn ga_reduces_affinity_violations() {
    let (raw_nodes, daemon_sets) = load_infra(INFRA).unwrap();
    let (pods, groups) = load_workloads(WORKLOADS).unwrap();
    let nodes = pre_deduct_daemonsets(&raw_nodes, &daemon_sets);

    let seed = ffd_warmstart(&pods, &nodes, &FfdWeights::default());
    let ffd_violations = compute_affinity_violations(&seed.assignment, &pods);

    let config = GaConfig {
        population_size: 64,
        max_generations: 100,
        early_stop_generations: 30,
        ..GaConfig::default()
    };
    let best = run_ga(
        &seed,
        &pods,
        &nodes,
        &groups,
        &config,
        &FitnessWeights::default(),
    );
    let ga_violations = compute_affinity_violations(&best.assignment, &pods);

    // GA should have same or fewer violations
    assert!(ga_violations <= ffd_violations);
}
