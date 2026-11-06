//! Temporary measurement: time each fitness term on MSC Irina (not committed).
use kuberina_solver::fitness::*;
use kuberina_solver::model::{FfdWeights, FitnessWeights};
use kuberina_solver::parser::{load_infra, load_workloads};
use kuberina_solver::phase0::pre_deduct_daemonsets;
use kuberina_solver::phase1_ffd::ffd_warmstart;
use std::time::Instant;

fn time<T>(label: &str, n: usize, mut f: impl FnMut() -> T) {
    let t = Instant::now();
    for _ in 0..n {
        std::hint::black_box(f());
    }
    println!(
        "{label:<28} {:>9.3} ms/call",
        t.elapsed().as_secs_f64() * 1000.0 / n as f64
    );
}

fn main() {
    let (nodes, ds) = load_infra("testdata/irina_infra.yaml").unwrap();
    let nodes = pre_deduct_daemonsets(&nodes, &ds);
    let (pods, groups) = load_workloads("testdata/irina_workloads.yaml").unwrap();
    let bp = ffd_warmstart(&pods, &nodes, &FfdWeights::default());
    let w = FitnessWeights::default();
    let n = 50;
    time("compute_fitness (total)", n, || {
        compute_fitness(&bp, &pods, &nodes, &groups, &w)
    });
    time("affinity_violations", n, || {
        compute_affinity_violations(&bp.assignment, &pods)
    });
    time("topology_spread_penalty", n, || {
        compute_topology_spread_penalty(&bp.assignment, &pods, &nodes)
    });
    time("fragmentation", n, || {
        compute_fragmentation(&bp.node_load, &nodes)
    });
    time("utilization_variance", n, || {
        compute_utilization_variance(&bp.node_load, &nodes)
    });
    time("count_active_nodes", n, || {
        count_active_nodes(&bp.assignment, nodes.len())
    });
}
