//! Integration test: end-to-end homelab scenario.
//!
//! Ported from `research/tests/test_integration.py`.
//! Load testdata → Phase 0 → Phase 1 → Phase 2 → verify constraints.

use kuberina_solver::admission::{
    admit_in_order, admit_in_order_without_repair, ffd_with_admission, in_order_violation,
};
use kuberina_solver::admission_ir::arrival_order;
use kuberina_solver::csp::{compute_capacity_overflow, compute_overflow_by_dimension};
use kuberina_solver::fitness::{compute_affinity_violations, compute_fitness};
use kuberina_solver::gpu_ir::check_device_inputs;
use kuberina_solver::model::{FfdWeights, FitnessWeights, GaConfig};
use kuberina_solver::model::{Node, Pod, UNPLACED};
use kuberina_solver::packing::Packing;
use kuberina_solver::parser::{load_infra, load_workloads};
use kuberina_solver::phase0::pre_deduct_daemonsets;
use kuberina_solver::phase1_ffd::ffd_warmstart;
use kuberina_solver::phase2_ga::run_ga;
use kuberina_solver::unplaced_term::derive_unplaced_weights;

const INFRA: &str = "testdata/homelab_infra.yaml";
const WORKLOADS: &str = "testdata/homelab_workloads.yaml";
const GPU_DEVICES_INFRA: &str = "testdata/gpu_devices_infra.yaml";
const GPU_DEVICES_WORKLOADS: &str = "testdata/gpu_devices_workloads.yaml";
const ADMISSION_INFRA: &str = "testdata/admission_infra.yaml";
const ADMISSION_WORKLOADS: &str = "testdata/admission_workloads.yaml";

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
    let best = run_ga(&seed, &pods, &nodes, &groups, &config, &weights).best;

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
    )
    .best;
    let ga_violations = compute_affinity_violations(&best.assignment, &pods);

    // GA should have same or fewer violations
    assert!(ga_violations <= ffd_violations);
}

/// Every committed testdata pair must load under the IR v1 parser.
///
/// Regression for #26: `irina_workloads.yaml` carried a top-level `pods:` list that
/// the stricter IR v1 parser rejects, and no test loaded it, so `make solver-irina`
/// broke without a failing test.
#[test]
fn every_committed_testdata_file_loads() {
    for (infra, workloads) in [
        (
            "testdata/homelab_infra.yaml",
            "testdata/homelab_workloads.yaml",
        ),
        ("testdata/irina_infra.yaml", "testdata/irina_workloads.yaml"),
        (GPU_DEVICES_INFRA, GPU_DEVICES_WORKLOADS),
        (ADMISSION_INFRA, ADMISSION_WORKLOADS),
    ] {
        load_infra(infra).unwrap_or_else(|e| panic!("{infra}: {e}"));
        load_workloads(workloads).unwrap_or_else(|e| panic!("{workloads}: {e}"));
    }
}

/// X-3 done-when, solver half: a half-GPU Pod cannot join eight 600-milli Pods on an
/// eight-device node, although the node's GPU sum (4.8 + 0.5 of 8) would accept it.
#[test]
fn half_gpu_pod_on_eight_sixty_percent_devices_is_gpu_overflow() {
    let (raw_nodes, daemon_sets) = load_infra(GPU_DEVICES_INFRA).unwrap();
    // Through Phase 0, as the CLI runs: it once dropped the device count.
    let nodes = pre_deduct_daemonsets(&raw_nodes, &daemon_sets);
    let (pods, _) = load_workloads(GPU_DEVICES_WORKLOADS).unwrap();
    check_device_inputs(&nodes, &pods).unwrap();

    let seed = ffd_warmstart(&pods, &nodes, &FfdWeights::default());
    let gpu_sum: f64 = pods.iter().map(|p| p.requests.gpu).sum();
    assert!((gpu_sum - 5.3).abs() < 1e-9 && gpu_sum < nodes[0].allocatable.gpu);

    let over = compute_overflow_by_dimension(&seed.assignment, &pods, &nodes);
    assert!((over.gpu - 0.5).abs() < 1e-9, "GPU overflow {}", over.gpu);
}

/// The admission fixture through Phase 0, as the CLI loads it.
fn admission_fixture() -> (Vec<Node>, Vec<Pod>) {
    let (raw_nodes, daemon_sets) = load_infra(ADMISSION_INFRA).unwrap();
    let (pods, _) = load_workloads(ADMISSION_WORKLOADS).unwrap();
    check_device_inputs(&raw_nodes, &pods).unwrap();
    (pre_deduct_daemonsets(&raw_nodes, &daemon_sets), pods)
}

/// X-4 done-when: at 134% demand, in-order admission declines Pods, overflows
/// nothing, admits a Pod after an earlier one was rejected, and keeps the rule.
#[test]
fn in_order_admission_at_134_percent_is_feasible_and_keeps_the_rule() {
    let (nodes, pods) = admission_fixture();
    let order = arrival_order(&pods).unwrap();
    let seed = admit_in_order(&pods, &nodes, &order);

    assert_eq!(
        compute_capacity_overflow(&seed.assignment, &pods, &nodes),
        0.0
    );
    let first_rejected = order
        .iter()
        .position(|&p| seed.assignment[p] == UNPLACED)
        .unwrap();
    assert!(
        order[first_rejected..]
            .iter()
            .any(|&p| seed.assignment[p] != UNPLACED)
    );
    let packing = Packing::from_assignment(&pods, &nodes, &seed.assignment);
    assert_eq!(in_order_violation(&packing, &order), None);
}

#[test]
fn free_admission_at_134_percent_is_feasible_and_declines_pods() {
    let (nodes, pods) = admission_fixture();
    let seed = ffd_with_admission(&pods, &nodes, &FfdWeights::default());
    assert_eq!(
        compute_capacity_overflow(&seed.assignment, &pods, &nodes),
        0.0
    );
    assert!(seed.assignment.contains(&UNPLACED));
}

/// S-2c done-when: with the swap operator, free admission trades a training Pod for
/// several inference Pods. At X-4, it stopped at the FFD seed: 16 GPUs in 3 Pods.
///
/// The objective (plan #24, §8 decision 8) values one Pod at `g_min` GPUs, 0.3 here, so
/// 9 Pods at 15.5 GPUs scores better than 7 Pods at 16 GPUs, and the GA finds the former.
#[test]
fn free_admission_trades_one_large_pod_for_several_small_ones() {
    let (nodes, pods) = admission_fixture();
    let base = FitnessWeights::default();
    let weights = FitnessWeights {
        unplaced: derive_unplaced_weights(&pods, &nodes, &base),
        ..base
    };
    let mut seed = ffd_with_admission(&pods, &nodes, &FfdWeights::default());
    seed.fitness = compute_fitness(&seed, &pods, &nodes, &[], &weights).0;
    let config = GaConfig {
        free_admission: true,
        ..GaConfig::default()
    };
    let best = run_ga(&seed, &pods, &nodes, &[], &config, &weights).best;

    let placed = best.assignment.iter().filter(|&&n| n != UNPLACED).count();
    assert_eq!(
        compute_capacity_overflow(&best.assignment, &pods, &nodes),
        0.0
    );
    assert!(placed > 3, "placed {placed} Pods");
    assert!(best.fitness < seed.fitness);
}

/// S-2d done-when (ADR-0003): `--no-ga` stops after the in-order walk, writes the
/// blueprint, and the in-order rule holds on it.
#[test]
fn no_ga_stops_after_the_walk_and_writes_a_blueprint() {
    let dir = std::env::temp_dir().join(format!("kuberina-no-ga-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let fixture = |name: &str| format!("{}/{name}", env!("CARGO_MANIFEST_DIR"));
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_kuberina-solver"))
        .current_dir(&dir)
        .args(["plan", "--infra", &fixture(ADMISSION_INFRA), "--workloads"])
        .args([
            &fixture(ADMISSION_WORKLOADS),
            "--admission",
            "in-order",
            "--no-ga",
        ])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    let blueprint = std::fs::read_to_string(dir.join("kuberina_solution.yaml")).unwrap();
    std::fs::remove_dir_all(&dir).unwrap();

    assert!(output.status.success(), "{stderr}");
    assert!(
        stderr.contains("Phase 2 (GA): skipped (--no-ga)"),
        "{stderr}"
    );
    assert!(!stderr.contains("generation"), "{stderr}");
    assert!(
        blueprint.contains("admission: in-order\n  unplaced: 2\n"),
        "{blueprint}"
    );
    // X-5 times each phase from these lines.
    for phase in [
        "Timing: load ",
        "Timing: Phase 1 ",
        "Timing: in-order rule ",
    ] {
        assert!(stderr.contains(phase), "missing {phase:?} in {stderr}");
    }
}

/// What one CLI run printed and wrote.
struct CliRun {
    succeeded: bool,
    stderr: String,
    blueprint: Option<String>,
}

/// Run `kuberina-solver plan` on two fixtures in a fresh directory, then remove it.
fn run_plan_cli(tag: &str, infra: &str, workloads: &str, flags: &[&str]) -> CliRun {
    let dir = std::env::temp_dir().join(format!("kuberina-{tag}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let fixture = |name: &str| format!("{}/{name}", env!("CARGO_MANIFEST_DIR"));
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_kuberina-solver"))
        .current_dir(&dir)
        .args([
            "plan",
            "--infra",
            &fixture(infra),
            "--workloads",
            &fixture(workloads),
        ])
        .args(flags)
        .output()
        .unwrap();
    let blueprint = std::fs::read_to_string(dir.join("kuberina_solution.yaml")).ok();
    std::fs::remove_dir_all(&dir).unwrap();
    CliRun {
        succeeded: output.status.success(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        blueprint,
    }
}

/// X-9 done-when, on the fixture: `--no-repair` walks by best fit alone, and the in-order
/// rule holds without enforcement.
#[test]
fn no_repair_walks_by_best_fit_alone() {
    let flags = ["--admission", "in-order", "--no-repair", "--no-ga"];
    let run = run_plan_cli("no-repair", ADMISSION_INFRA, ADMISSION_WORKLOADS, &flags);
    assert!(run.succeeded, "{}", run.stderr);
    assert!(
        run.stderr
            .contains("Phase 1 seed: the in-order walk without repair"),
        "{}",
        run.stderr
    );
    assert!(
        run.blueprint
            .is_some_and(|b| b.contains("admission: in-order"))
    );

    let (nodes, pods) = admission_fixture();
    let order = arrival_order(&pods).unwrap();
    let walk = admit_in_order_without_repair(&pods, &nodes, &order);
    let packing = Packing::from_assignment(&pods, &nodes, &walk.assignment);
    assert_eq!(in_order_violation(&packing, &order), None);
}

#[test]
fn no_repair_on_a_run_that_does_not_walk_is_an_error() {
    let run = run_plan_cli("no-repair-ffd", INFRA, WORKLOADS, &["--no-repair"]);
    assert!(!run.succeeded);
    assert!(
        run.stderr.contains("--no-repair changes the in-order walk"),
        "{}",
        run.stderr
    );
}

/// X-10 done-when, on the fixture: free admission starts from the walk when every Pod has
/// `arrival`, and the in-order rule is not enforced on the result.
#[test]
fn free_admission_starts_from_the_walk_when_every_pod_has_arrival() {
    let flags = ["--admission", "free", "--no-ga"];
    let run = run_plan_cli("free-walk", ADMISSION_INFRA, ADMISSION_WORKLOADS, &flags);
    assert!(run.succeeded, "{}", run.stderr);
    assert!(
        run.stderr
            .contains("Phase 1 seed: the in-order walk with depth-1 repair"),
        "{}",
        run.stderr
    );
    assert!(
        !run.stderr.contains("Timing: in-order rule"),
        "{}",
        run.stderr
    );
    assert!(run.blueprint.is_some_and(|b| b.contains("admission: free")));
}

#[test]
fn free_admission_without_arrivals_starts_from_ffd() {
    let run = run_plan_cli(
        "free-ffd",
        INFRA,
        WORKLOADS,
        &["--admission", "free", "--no-ga"],
    );
    assert!(run.succeeded, "{}", run.stderr);
    assert!(
        !run.stderr.contains("Phase 1 seed: the in-order walk"),
        "{}",
        run.stderr
    );
}
