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

use kuberina_solver::fitness::compute_fitness;
use kuberina_solver::model::{FfdWeights, FitnessWeights, GaConfig, Node, ResourceVector};
use kuberina_solver::parser::{load_infra, load_workloads};
use kuberina_solver::phase0::pre_deduct_daemonsets;
use kuberina_solver::phase1_ffd::ffd_warmstart;
use kuberina_solver::phase2_ga::run_ga;

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
    let best = run_ga(
        &seed,
        &pods,
        &planning_nodes,
        &groups,
        &ga_config,
        &fitness_weights,
    );

    let elapsed = start.elapsed().as_secs_f64();
    // Print the blueprint against real capacity — the reserve is withheld from the
    // optimizer, not from the operator reading the result.
    print_blueprint(&best, &pods, &net_nodes, elapsed);
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
            mutation_rate: 0.03,
            crossover_rate: 0.85,
            max_generations: 1000,
            early_stop_generations: 200,
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

fn print_blueprint(
    best: &kuberina_solver::model::Blueprint,
    pods: &[kuberina_solver::model::Pod],
    nodes: &[kuberina_solver::model::Node],
    elapsed: f64,
) {
    println!("\n═══ Final Blueprint (Stowage Plan) ═══");
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
