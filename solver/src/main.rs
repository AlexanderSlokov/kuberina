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
use kuberina_solver::model::{FfdWeights, FitnessWeights, GaConfig};
use kuberina_solver::parser::{load_infra, load_workloads};
use kuberina_solver::phase0::pre_deduct_daemonsets;
use kuberina_solver::phase1_ffd::ffd_warmstart;
use kuberina_solver::phase2_ga::run_ga;

#[derive(Parser)]
#[command(name = "kuberina", about = "Maritime stowage-inspired K8s scheduling optimizer")]
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
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Plan { infra, workloads } => run_plan(&infra, &workloads),
    }
}

fn run_plan(infra_path: &str, workloads_path: &str) {
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
        raw_nodes.len(), daemon_sets.len(), pods.len(), groups.len(),
    );

    // Phase 0: DaemonSet pre-deduction (ballast water)
    let nodes = pre_deduct_daemonsets(&raw_nodes, &daemon_sets);
    print_phase0_summary(&raw_nodes, &nodes);

    // Phase 1: FFD warm-start (stow heaviest containers first)
    let ffd_weights = FfdWeights::default();
    let mut seed = ffd_warmstart(&pods, &nodes, &ffd_weights);
    let fitness_weights = FitnessWeights::default();
    seed.fitness = compute_fitness(&seed, &pods, &nodes, &groups, &fitness_weights);
    eprintln!("Phase 1 (FFD): seed fitness = {:.4}", seed.fitness);

    // Phase 2: GA optimization (evolutionary stowage planning)
    // WHY: auto-scale GA params based on problem size (ga_estimation.md §3).
    let ga_config = select_ga_config(pods.len());
    let best = run_ga(&seed, &pods, &nodes, &groups, &ga_config, &fitness_weights);

    let elapsed = start.elapsed().as_secs_f64();
    print_blueprint(&best, &pods, &nodes, elapsed);
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
            ded.name, raw.allocatable.cpu, ded.allocatable.cpu,
            raw.allocatable.ram, ded.allocatable.ram, oh_cpu, oh_ram,
        );
    }
}

fn print_blueprint(
    blueprint: &kuberina_solver::model::Blueprint,
    pods: &[kuberina_solver::model::Pod],
    nodes: &[kuberina_solver::model::Node],
    elapsed: f64,
) {
    println!("\n═══ Final Blueprint (Stowage Plan) ═══");
    println!("  Fitness: {:.4}", blueprint.fitness);
    println!("  Time: {:.2}s", elapsed);
    println!();

    let num_nodes = nodes.len();
    let mut node_pods: Vec<Vec<&str>> = vec![vec![]; num_nodes];
    for (pod_idx, &node_idx) in blueprint.assignment.iter().enumerate() {
        node_pods[node_idx].push(&pods[pod_idx].name);
    }

    if num_nodes <= 20 {
        print_all_nodes(nodes, blueprint, &node_pods);
    } else {
        print_summary_mode(nodes, blueprint, &node_pods);
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
        let cpu_pct = if cap.cpu > 0.0 { load.cpu / cap.cpu * 100.0 } else { 0.0 };
        let pod_count = node_pods[j].len();
        if pod_count > 0 { active_count += 1; } else { empty_count += 1; }
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
        stats.iter().filter(|s| s.pod_count > 0).map(|s| s.cpu_pct).sum::<f64>()
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
    by_cpu.sort_by(|a, b| b.cpu_pct.partial_cmp(&a.cpu_pct).unwrap_or(std::cmp::Ordering::Equal));
    println!("  ─── Top 5 Busiest Nodes ───");
    for s in by_cpu.iter().take(5) {
        println!(
            "  {}: {:.1}/{:.1} CPU ({:.0}%), {} pods",
            s.name, s.load_cpu, s.cap_cpu, s.cpu_pct, s.pod_count,
        );
    }

    // Bottom 5 least loaded (with at least 1 pod)
    let mut active_stats: Vec<&NodeStat> = stats.iter().filter(|s| s.pod_count > 0).collect();
    active_stats.sort_by(|a, b| a.cpu_pct.partial_cmp(&b.cpu_pct).unwrap_or(std::cmp::Ordering::Equal));
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
        println!("\n  ─── {} nodes empty (available for shutdown) ───", empty_count);
    }
    println!();
}
