//! Kuberina CLI — pre-deployment scheduling optimizer.
//!
//! Ported from `research/src/kuberina/__main__.py`.
//!
//! Usage:
//!     kuberina plan --infra <file> --workloads <file>
//!
//! Pipeline: Phase 0 (DaemonSet pre-deduction) → Phase 1 (FFD) → Phase 2 (GA+CSP)

use std::time::Instant;

use clap::{Parser, Subcommand};

use kuberina_solver::csp::{compute_overflow_by_dimension, compute_selector_violations};
use kuberina_solver::fitness::compute_fitness;
use kuberina_solver::model::{
    Blueprint, FfdWeights, FitnessWeights, GaConfig, Node, Pod, ResourceVector,
};
use kuberina_solver::parser::{IR_API_VERSION, KIND_BLUEPRINT, load_infra, load_workloads};
use kuberina_solver::phase0::pre_deduct_daemonsets;
use kuberina_solver::phase1_ffd::ffd_warmstart;
use kuberina_solver::phase2_ga::{GaOutcome, run_ga};

#[derive(Parser)]
#[command(
    name = "kuberina",
    about = "Maritime stowage-inspired K8s scheduling optimizer"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate optimized scheduling blueprint
    Plan {
        /// Cluster topology YAML
        #[arg(long)]
        infra: String,
        /// Workload manifests YAML
        #[arg(long)]
        workloads: String,
        /// Reserve this percentage of every node's capacity (e.g., 20 keeps 20% free)
        #[arg(long)]
        headroom: Option<f64>,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Plan {
            infra,
            workloads,
            headroom,
        } => run_plan(&infra, &workloads, validate_headroom(headroom)),
    }
}

/// Reject a headroom percentage outside `[0, 100)`.
///
/// A reserve of 100% or more leaves no capacity to place into, and a negative one
/// would inflate nodes past their real size.
///
/// ```ignore
/// let pct = validate_headroom(Some(20.0)); // Some(20.0); Some(120.0) exits
/// ```
fn validate_headroom(headroom: Option<f64>) -> Option<f64> {
    let pct = headroom?;
    if !(0.0..100.0).contains(&pct) {
        eprintln!("Error: --headroom must be a percentage in [0, 100), got {pct}");
        std::process::exit(1);
    }
    Some(pct)
}

/// Scale one capacity dimension, leaving unconstrained dimensions untouched.
///
/// WHY: only scale constrained dimensions — f64::MAX * 0.8 is still MAX-ish
/// but could drift. Skip unconstrained dims entirely.
///
/// ```ignore
/// let mut cpu = 64.0;
/// scale_if_constrained(&mut cpu, 0.8); // 51.2
/// ```
fn scale_if_constrained(dimension: &mut f64, factor: f64) {
    if *dimension < f64::MAX {
        *dimension *= factor;
    }
}

/// Scale every capacity dimension of one node by `factor`.
///
/// ```ignore
/// scale_allocatable(&mut node.allocatable, 0.8);
/// ```
fn scale_allocatable(allocatable: &mut ResourceVector, factor: f64) {
    scale_if_constrained(&mut allocatable.cpu, factor);
    scale_if_constrained(&mut allocatable.ram, factor);
    scale_if_constrained(&mut allocatable.gpu, factor);
    scale_if_constrained(&mut allocatable.storage, factor);
    scale_if_constrained(&mut allocatable.disk_read, factor);
    scale_if_constrained(&mut allocatable.disk_write, factor);
    scale_if_constrained(&mut allocatable.net_in, factor);
    scale_if_constrained(&mut allocatable.net_out, factor);
}

/// Build a copy of `nodes` with `headroom_pct` of every capacity withheld.
///
/// The optimizer never sees the withheld fraction, which is what makes the reserve
/// structural rather than a target: no placement it produces can allocate into it.
///
/// ```ignore
/// let reserved = reserve_headroom(&net_nodes, 20.0); // every node keeps 20% free
/// ```
fn reserve_headroom(nodes: &[Node], headroom_pct: f64) -> Vec<Node> {
    let factor = 1.0 - headroom_pct / 100.0;
    let mut reserved = nodes.to_vec();
    for node in &mut reserved {
        scale_allocatable(&mut node.allocatable, factor);
    }
    reserved
}

/// Optimize placement and print the resulting blueprint.
///
/// Under `--headroom h`, the FFD warm-start, the fitness function and the GA all
/// operate on capacities reduced by `h`, while the printed blueprint and every
/// reported utilization figure use the real post-DaemonSet capacities. Every node in
/// the output therefore carries at least `h%` of its real capacity unallocated.
///
/// ```ignore
/// run_plan("testdata/homelab_infra.yaml", "testdata/homelab_workloads.yaml", None);
/// ```
fn run_plan(infra_path: &str, workloads_path: &str, headroom: Option<f64>) {
    let start = Instant::now();

    let (raw_nodes, daemon_sets) = load_infra(infra_path).unwrap_or_else(|e| {
        eprintln!("Error loading infra: {e}");
        std::process::exit(1);
    });
    let (pods, groups) = load_workloads(workloads_path).unwrap_or_else(|e| {
        eprintln!("Error loading workloads: {e}");
        std::process::exit(1);
    });

    eprintln!(
        "Loaded {} nodes, {} daemonsets, {} pods, {} groups",
        raw_nodes.len(),
        daemon_sets.len(),
        pods.len(),
        groups.len(),
    );

    // Phase 0: DaemonSet pre-deduction (ballast water)
    let net_nodes = pre_deduct_daemonsets(&raw_nodes, &daemon_sets);
    print_phase0_summary(&raw_nodes, &net_nodes);

    let planning_nodes = match headroom {
        Some(pct) => {
            eprintln!(
                "\n[Headroom Mode] Reserving {:.1}% of every node — the optimizer sees {:.1}% of real capacity.",
                pct,
                100.0 - pct,
            );
            reserve_headroom(&net_nodes, pct)
        }
        None => net_nodes.clone(),
    };

    // Phase 1: FFD warm-start (stow heaviest containers first)
    let ffd_weights = FfdWeights::default();
    let mut seed = ffd_warmstart(&pods, &planning_nodes, &ffd_weights);
    let fitness_weights = FitnessWeights::default();
    let (fitness, sc) = compute_fitness(&seed, &pods, &planning_nodes, &groups, &fitness_weights);
    seed.fitness = fitness;
    seed.scorecard = sc.clone();
    eprintln!("Phase 1 (FFD): seed fitness = {:.4}", seed.fitness);

    // Phase 2: GA optimization (evolutionary stowage planning)
    // WHY: auto-scale GA params based on problem size (ga_estimation.md §3).
    let ga_config = select_ga_config(pods.len());
    let outcome = run_ga(
        &seed,
        &pods,
        &planning_nodes,
        &groups,
        &ga_config,
        &fitness_weights,
    );
    report_ga_termination(&outcome, &ga_config);
    let best = outcome.best;

    let elapsed = start.elapsed().as_secs_f64();
    // Print the blueprint against real capacity — the reserve is withheld from the
    // optimizer, not from the operator reading the result.
    let verdict = classify(&best, &pods, &net_nodes, &planning_nodes, headroom);
    let infeasible = matches!(verdict, Verdict::Infeasible { .. });
    print_blueprint(&best, &pods, &net_nodes, elapsed, infeasible);
    report_verdict(&verdict);
    if verdict.exit_code() != 0 {
        std::process::exit(verdict.exit_code());
    }
}

/// What the emitted blueprint is actually worth.
///
/// A nonzero hard penalty does not make the run fail today, which is how an
/// assignment violating seven capacity constraints came to be printed under the
/// heading "Final Blueprint" with exit code 0. See issue #17.
enum Verdict {
    /// Every hard constraint holds, including the reserve if one was requested.
    Feasible,
    /// Fits the cluster, but spends capacity the operator asked to keep free.
    ReserveNotMet { headroom: f64, over: ResourceVector },
    /// Does not fit the cluster. The blueprint is not applicable.
    Infeasible {
        over: ResourceVector,
        selector_violations: usize,
        gang_penalty: f64,
    },
}

impl Verdict {
    fn exit_code(&self) -> i32 {
        match self {
            Verdict::Feasible => 0,
            Verdict::ReserveNotMet { .. } => 2,
            Verdict::Infeasible { .. } => 1,
        }
    }
}

/// Decide whether the blueprint holds against real capacity, the reserve, or neither.
///
/// ```ignore
/// match classify(&best, &pods, &net_nodes, &planning_nodes, Some(20.0)) { .. }
/// ```
fn classify(
    best: &Blueprint,
    pods: &[Pod],
    net_nodes: &[Node],
    planning_nodes: &[Node],
    headroom: Option<f64>,
) -> Verdict {
    let real_over = compute_overflow_by_dimension(&best.assignment, pods, net_nodes);
    let selector_violations = compute_selector_violations(&best.assignment, pods, net_nodes);

    if total_of(&real_over) > 1e-9 || selector_violations > 0 || best.scorecard.gang_penalty > 1e-9
    {
        return Verdict::Infeasible {
            over: real_over,
            selector_violations,
            gang_penalty: best.scorecard.gang_penalty,
        };
    }

    if let Some(pct) = headroom {
        let reserved_over = compute_overflow_by_dimension(&best.assignment, pods, planning_nodes);
        if total_of(&reserved_over) > 1e-9 {
            return Verdict::ReserveNotMet {
                headroom: pct,
                over: reserved_over,
            };
        }
    }

    Verdict::Feasible
}

fn total_of(v: &ResourceVector) -> f64 {
    v.cpu + v.ram + v.gpu + v.storage + v.disk_read + v.disk_write + v.net_in + v.net_out
}

/// List the dimensions carrying overflow, largest first.
fn overflowing_dimensions(over: &ResourceVector) -> Vec<(&'static str, f64)> {
    let mut dims = vec![
        ("cpu", over.cpu),
        ("ram", over.ram),
        ("gpu", over.gpu),
        ("storage", over.storage),
        ("disk_read", over.disk_read),
        ("disk_write", over.disk_write),
        ("net_in", over.net_in),
        ("net_out", over.net_out),
    ];
    dims.retain(|&(_, amount)| amount > 1e-9);
    dims.sort_by(|a, b| b.1.total_cmp(&a.1));
    dims
}

/// State the verdict where the operator cannot miss it.
fn report_verdict(verdict: &Verdict) {
    match verdict {
        Verdict::Feasible => {
            println!("  ✅ FEASIBLE — every hard constraint holds.");
        }
        Verdict::ReserveNotMet { headroom, over } => {
            println!(
                "
═══ ⚠️  RESERVE NOT MET ═══"
            );
            println!(
                "  The blueprint fits the cluster, but spends capacity you asked to keep free.
                   Requested reserve: {:.1}%. Over the reserve by:",
                headroom,
            );
            for (dim, amount) in overflowing_dimensions(over) {
                println!("    {dim:<10} {amount:>14.2}");
            }
            println!(
                "
  Lower --headroom, or add nodes. The exported plan is applicable as-is;
                   it simply leaves less runtime margin than you specified."
            );
        }
        Verdict::Infeasible {
            over,
            selector_violations,
            gang_penalty,
        } => {
            println!(
                "
═══ ❌ INFEASIBLE — DO NOT APPLY ═══"
            );
            println!("  This assignment exceeds real node capacity. It is not a plan.");
            let dims = overflowing_dimensions(over);
            if !dims.is_empty() {
                println!(
                    "
  Capacity exceeded, by dimension:"
                );
                for (dim, amount) in dims {
                    println!("    {dim:<10} {amount:>14.2}");
                }
            }
            if *selector_violations > 0 {
                println!(
                    "
  NodeSelector violations: {selector_violations}"
                );
            }
            if *gang_penalty > 1e-9 {
                println!("  Gang penalty: {gang_penalty:.0}");
            }
            println!(
                "
  kuberina_solution.yaml was written for inspection and is marked infeasible.
                   Applying it would schedule pods onto nodes that cannot hold them."
            );
        }
    }
}

/// Auto-scale GA parameters based on problem size.
///
/// Tiers from ga_estimation.md §3:
/// - Small  (<100 pods):  quick convergence, population 128
/// - Medium (100-500):    standard GA, population 256
/// - Large  (>500 pods):  datacenter-scale brute force, population 1024
fn select_ga_config(num_pods: usize) -> GaConfig {
    if num_pods > 500 {
        eprintln!(
            "Datacenter-scale detected ({} pods) — cranking GA to maximum",
            num_pods,
        );
        GaConfig {
            population_size: 1024,
            tournament_size: 5,
            mutations_per_child: 4.0,
            init_mutations: 16.0,
            crossover_rate: 0.85,
            max_generations: 1000,
            early_stop_generations: 200,
            min_relative_improvement: 1e-4,
            random_seed: 42,
        }
    } else if num_pods > 100 {
        GaConfig {
            population_size: 256,
            max_generations: 500,
            early_stop_generations: 100,
            ..GaConfig::default()
        }
    } else {
        GaConfig::default()
    }
}

/// Tell the operator how the GA run ended, not just what it produced.
///
/// WHY: a wall-clock figure means one thing for a run that converged and another
/// for one that hit its budget. PAPER §7.4 could not tell the two apart (#20).
fn report_ga_termination(outcome: &GaOutcome, config: &GaConfig) {
    if outcome.stopped_early {
        eprintln!(
            "Phase 2 (GA): converged at generation {} of {} — gain stayed under {:.4}% for {} generations",
            outcome.generations_run,
            config.max_generations,
            config.min_relative_improvement * 100.0,
            config.early_stop_generations,
        );
        return;
    }
    eprintln!(
        "Phase 2 (GA): ran the full budget of {} generations without converging",
        config.max_generations,
    );
}

fn print_phase0_summary(
    raw_nodes: &[kuberina_solver::model::Node],
    deducted_nodes: &[kuberina_solver::model::Node],
) {
    println!("\n═══ Phase 0: DaemonSet Pre-deduction (Ballast Water) ═══");
    for (raw, ded) in raw_nodes.iter().zip(deducted_nodes.iter()) {
        let oh_cpu = raw.allocatable.cpu - ded.allocatable.cpu;
        let oh_ram = raw.allocatable.ram - ded.allocatable.ram;
        println!(
            "  {}: {:.1} → {:.2} CPU, {:.1} → {:.3} GiB RAM (-{:.2} CPU, -{:.3} GiB overhead)",
            ded.name,
            raw.allocatable.cpu,
            ded.allocatable.cpu,
            raw.allocatable.ram,
            ded.allocatable.ram,
            oh_cpu,
            oh_ram,
        );
    }
}

fn print_blueprint(best: &Blueprint, pods: &[Pod], nodes: &[Node], elapsed: f64, infeasible: bool) {
    if infeasible {
        println!("\n═══ Rejected Assignment (NOT a blueprint) ═══");
    } else {
        println!("\n═══ Final Blueprint (Stowage Plan) ═══");
    }
    println!("  Fitness: {:.4}", best.fitness);
    println!("  Time: {:.2}s", elapsed);
    println!("  Scorecard:");
    println!(
        "    Capacity Penalty: {:.0}",
        best.scorecard.capacity_penalty
    );
    println!(
        "    Selector Penalty: {:.0}",
        best.scorecard.selector_penalty
    );
    println!("    Gang Penalty: {:.0}", best.scorecard.gang_penalty);
    println!("    Active Nodes: {:.0}", best.scorecard.active_nodes);
    println!("    Fragmentation: {:.2}", best.scorecard.fragmentation);
    println!(
        "    Affinity Violations: {:.0}",
        best.scorecard.affinity_violations
    );
    println!(
        "    Utilization Variance: {:.4}",
        best.scorecard.utilization_variance
    );
    println!(
        "    Topology Spread Penalty: {:.2}",
        best.scorecard.topology_spread_penalty
    );
    println!();

    let num_nodes = nodes.len();
    let mut node_pods: Vec<Vec<&str>> = vec![vec![]; num_nodes];
    for (pod_idx, &node_idx) in best.assignment.iter().enumerate() {
        node_pods[node_idx].push(&pods[pod_idx].name);
    }

    if num_nodes <= 10 && pods.len() <= 50 {
        print_all_nodes(nodes, best, &node_pods);
    } else {
        print_summary_mode(nodes, best, &node_pods);
    }

    // Export full solution to YAML
    let mut yaml_out = String::new();
    if infeasible {
        // WHY: the file is written for inspection, not application. A reader who
        // opens it without having seen the console output must still be told.
        yaml_out.push_str(
            "# INFEASIBLE — this assignment exceeds real node capacity.\n             # Written for inspection only. Do not apply. See the solver output.\n",
        );
    }
    // WHY the header (ADR-0002): `kuberina-forge out` links this file against the
    // manifests it came from. A bare map of names cannot say what it is, which
    // version it speaks, or whether it was safe to apply.
    yaml_out.push_str(&format!(
        "apiVersion: {IR_API_VERSION}\nkind: {KIND_BLUEPRINT}\nmetadata:\n  feasible: {}\n  pods: {}\n  activeNodes: {}\n",
        !infeasible,
        pods.len(),
        best.scorecard.active_nodes as usize,
    ));
    yaml_out.push_str("solution:\n");
    for (pod_idx, &node_idx) in best.assignment.iter().enumerate() {
        yaml_out.push_str(&format!(
            "  {}/{}: {}\n",
            pods[pod_idx].namespace, pods[pod_idx].name, nodes[node_idx].name,
        ));
    }
    if let Err(e) = std::fs::write("kuberina_solution.yaml", yaml_out) {
        eprintln!("Failed to export solution: {}", e);
    } else {
        println!("  (Full stowage plan exported to kuberina_solution.yaml)");
    }
}

fn print_all_nodes(
    nodes: &[kuberina_solver::model::Node],
    blueprint: &kuberina_solver::model::Blueprint,
    node_pods: &[Vec<&str>],
) {
    for (node_idx, node) in nodes.iter().enumerate() {
        let load = &blueprint.node_load[node_idx];
        let cpu_pct = if node.allocatable.cpu > 0.0 {
            load.cpu / node.allocatable.cpu * 100.0
        } else {
            0.0
        };
        let ram_pct = if node.allocatable.ram > 0.0 {
            load.ram / node.allocatable.ram * 100.0
        } else {
            0.0
        };

        println!("  {}:", node.name);
        for name in &node_pods[node_idx] {
            println!("    - {}", name);
        }
        println!(
            "    CPU: {:.2}/{:.2} ({:.0}%)",
            load.cpu, node.allocatable.cpu, cpu_pct,
        );
        println!(
            "    RAM: {:.3}/{:.3} GiB ({:.0}%)",
            load.ram, node.allocatable.ram, ram_pct,
        );
        println!();
    }
}

fn print_summary_mode(
    nodes: &[kuberina_solver::model::Node],
    blueprint: &kuberina_solver::model::Blueprint,
    node_pods: &[Vec<&str>],
) {
    // WHY: printing 150 nodes with 2700 pods individually would
    // produce 3000+ lines of unreadable output.
    let num_nodes = nodes.len();

    struct NodeStat {
        name: String,
        cpu_pct: f64,
        pod_count: usize,
        load_cpu: f64,
        cap_cpu: f64,
    }

    let mut stats: Vec<NodeStat> = Vec::with_capacity(num_nodes);
    let mut active_count = 0_usize;
    let mut empty_count = 0_usize;

    for j in 0..num_nodes {
        let load = &blueprint.node_load[j];
        let cap = &nodes[j].allocatable;
        let cpu_pct = if cap.cpu > 0.0 {
            load.cpu / cap.cpu * 100.0
        } else {
            0.0
        };
        let pod_count = node_pods[j].len();
        if pod_count > 0 {
            active_count += 1;
        } else {
            empty_count += 1;
        }
        stats.push(NodeStat {
            name: nodes[j].name.clone(),
            cpu_pct,
            pod_count,
            load_cpu: load.cpu,
            cap_cpu: cap.cpu,
        });
    }

    let total_pods: usize = stats.iter().map(|s| s.pod_count).sum();
    let avg_cpu: f64 = if active_count > 0 {
        stats
            .iter()
            .filter(|s| s.pod_count > 0)
            .map(|s| s.cpu_pct)
            .sum::<f64>()
            / active_count as f64
    } else {
        0.0
    };

    println!("  ─── Cluster Summary ({} nodes) ───", num_nodes);
    println!("  Active nodes: {} / {}", active_count, num_nodes);
    println!("  Empty nodes:  {}", empty_count);
    println!("  Pods placed:  {}", total_pods);
    println!("  Avg CPU util: {:.1}%", avg_cpu);
    println!();

    // Top 5 busiest
    let mut by_cpu: Vec<&NodeStat> = stats.iter().collect();
    by_cpu.sort_by(|a, b| {
        b.cpu_pct
            .partial_cmp(&a.cpu_pct)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    println!("  ─── Top 5 Busiest Nodes ───");
    for s in by_cpu.iter().take(5) {
        println!(
            "  {}: {:.1}/{:.1} CPU ({:.0}%), {} pods",
            s.name, s.load_cpu, s.cap_cpu, s.cpu_pct, s.pod_count,
        );
    }

    // Bottom 5 least loaded (with at least 1 pod)
    let mut active_stats: Vec<&NodeStat> = stats.iter().filter(|s| s.pod_count > 0).collect();
    active_stats.sort_by(|a, b| {
        a.cpu_pct
            .partial_cmp(&b.cpu_pct)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    println!("\n  ─── Top 5 Lightest Active Nodes ───");
    for s in active_stats.iter().take(5) {
        println!(
            "  {}: {:.1}/{:.1} CPU ({:.0}%), {} pods",
            s.name, s.load_cpu, s.cap_cpu, s.cpu_pct, s.pod_count,
        );
    }

    // Empty nodes
    if empty_count > 0 && empty_count <= 10 {
        println!("\n  ─── Empty Nodes ({}) ───", empty_count);
        for s in &stats {
            if s.pod_count == 0 {
                println!("  {}: 0 pods (available for shutdown)", s.name);
            }
        }
    } else if empty_count > 10 {
        println!(
            "\n  ─── {} nodes empty (available for shutdown) ───",
            empty_count
        );
    }
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn node_with(allocatable: ResourceVector) -> Node {
        Node {
            name: "n-000".to_string(),
            allocatable,
            labels: HashMap::new(),
            taints: Vec::new(),
            zone: "us-east-1a".to_string(),
            rack: "rack-0".to_string(),
        }
    }

    #[test]
    fn validate_headroom_passes_in_range_values() {
        assert_eq!(validate_headroom(None), None);
        assert_eq!(validate_headroom(Some(0.0)), Some(0.0));
        assert_eq!(validate_headroom(Some(20.0)), Some(20.0));
        assert_eq!(validate_headroom(Some(99.9)), Some(99.9));
    }

    #[test]
    fn scale_if_constrained_leaves_unconstrained_dimension_at_max() {
        let mut unconstrained = f64::MAX;
        scale_if_constrained(&mut unconstrained, 0.8);
        assert_eq!(unconstrained, f64::MAX);

        let mut cpu = 64.0;
        scale_if_constrained(&mut cpu, 0.8);
        assert!((cpu - 51.2).abs() < 1e-9);
    }

    #[test]
    fn scale_allocatable_touches_every_dimension() {
        let mut v = ResourceVector::new_8d(64.0, 256.0, 8.0, 1000.0, 500.0, 200.0, 1000.0, 1000.0);
        scale_allocatable(&mut v, 0.5);
        assert!((v.cpu - 32.0).abs() < 1e-9);
        assert!((v.ram - 128.0).abs() < 1e-9);
        assert!((v.gpu - 4.0).abs() < 1e-9);
        assert!((v.storage - 500.0).abs() < 1e-9);
        assert!((v.disk_read - 250.0).abs() < 1e-9);
        assert!((v.disk_write - 100.0).abs() < 1e-9);
        assert!((v.net_in - 500.0).abs() < 1e-9);
        assert!((v.net_out - 500.0).abs() < 1e-9);
    }

    #[test]
    fn reserve_headroom_withholds_the_requested_fraction() {
        let nodes = vec![node_with(ResourceVector::new_8d(
            64.0, 256.0, 8.0, 1000.0, 500.0, 200.0, 1000.0, 1000.0,
        ))];
        let reserved = reserve_headroom(&nodes, 20.0);
        assert!((reserved[0].allocatable.cpu - 51.2).abs() < 1e-9);
        assert!((reserved[0].allocatable.ram - 204.8).abs() < 1e-9);
        // The source stays untouched: the blueprint is printed against real capacity.
        assert!((nodes[0].allocatable.cpu - 64.0).abs() < 1e-9);
    }

    #[test]
    fn reserve_headroom_of_zero_is_the_identity() {
        let nodes = vec![node_with(ResourceVector::new_8d(
            64.0, 256.0, 8.0, 1000.0, 500.0, 200.0, 1000.0, 1000.0,
        ))];
        let reserved = reserve_headroom(&nodes, 0.0);
        assert!((reserved[0].allocatable.cpu - 64.0).abs() < 1e-9);
        assert!((reserved[0].allocatable.net_out - 1000.0).abs() < 1e-9);
    }
}

#[cfg(test)]
mod verdict_tests {
    use super::*;
    use std::collections::HashMap;

    fn node(name: &str, cpu: f64) -> Node {
        Node {
            name: name.into(),
            allocatable: ResourceVector::new(cpu, 1000.0, 0.0),
            labels: HashMap::new(),
            taints: Vec::new(),
            zone: String::new(),
            rack: String::new(),
        }
    }

    fn pod(name: &str, cpu: f64) -> Pod {
        Pod {
            name: name.into(),
            namespace: "ns".into(),
            requests: ResourceVector::new(cpu, 1.0, 0.0),
            tolerations: Vec::new(),
            node_selector: HashMap::new(),
            affinity_targets: Vec::new(),
            anti_affinity_targets: Vec::new(),
            group_name: String::new(),
            topology_spread: None,
            observed: None,
        }
    }

    fn blueprint(assignment: Vec<usize>, pods: &[Pod], num_nodes: usize) -> Blueprint {
        Blueprint {
            node_load: kuberina_solver::phase1_ffd::compute_node_loads(
                &assignment,
                pods,
                num_nodes,
            ),
            assignment,
            fitness: 0.0,
            scorecard: Default::default(),
        }
    }

    #[test]
    fn fitting_assignment_is_feasible() {
        let nodes = vec![node("n", 10.0)];
        let pods = vec![pod("p", 4.0)];
        let bp = blueprint(vec![0], &pods, 1);
        let v = classify(&bp, &pods, &nodes, &nodes, None);
        assert_eq!(v.exit_code(), 0);
    }

    #[test]
    fn exceeding_real_capacity_is_infeasible() {
        let nodes = vec![node("n", 5.0)];
        let pods = vec![pod("a", 4.0), pod("b", 4.0)];
        let bp = blueprint(vec![0, 0], &pods, 1);
        let v = classify(&bp, &pods, &nodes, &nodes, None);
        assert_eq!(v.exit_code(), 1);
        match v {
            Verdict::Infeasible { over, .. } => assert!((over.cpu - 3.0).abs() < 1e-9),
            _ => panic!("expected Infeasible"),
        }
    }

    #[test]
    fn fitting_the_cluster_but_not_the_reserve_is_its_own_verdict() {
        // 9 of 10 cores used: fits the node, spends into a 20% reserve.
        let nodes = vec![node("n", 10.0)];
        let reserved = vec![node("n", 8.0)];
        let pods = vec![pod("p", 9.0)];
        let bp = blueprint(vec![0], &pods, 1);
        let v = classify(&bp, &pods, &nodes, &reserved, Some(20.0));
        assert_eq!(v.exit_code(), 2);
        match v {
            Verdict::ReserveNotMet { over, .. } => assert!((over.cpu - 1.0).abs() < 1e-9),
            _ => panic!("expected ReserveNotMet"),
        }
    }

    #[test]
    fn overflowing_dimensions_are_ranked_largest_first() {
        let mut over = ResourceVector::zero();
        over.cpu = 5.0;
        over.disk_write = 100.0;
        over.net_in = 40.0;
        let dims = overflowing_dimensions(&over);
        assert_eq!(dims.len(), 3);
        assert_eq!(dims[0].0, "disk_write");
        assert_eq!(dims[2].0, "cpu");
    }
}
