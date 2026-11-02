//! Cost of the admission walk's parts and of one fitness evaluation (backlog X-5).
//!
//! Run from `solver/`:
//!
//! ```shell
//! cargo run --release --example trace_cost -- <infra.yaml> <workloads.yaml>
//! ```
//!
//! The walk's depth-1 repair is private to `admission.rs`, so its cost is reported as
//! the full walk minus an insertion-only replay and the final rule check.

use std::time::Instant;

use kuberina_solver::admission::{admit_in_order, in_order_violation};
use kuberina_solver::admission_ir::arrival_order;
use kuberina_solver::fitness::compute_fitness;
use kuberina_solver::gpu_devices::device_overflow_per_node;
use kuberina_solver::gpu_stranding::stranded_gpu;
use kuberina_solver::model::{Blueprint, FitnessWeights, Node, Pod};
use kuberina_solver::packing::Packing;
use kuberina_solver::parser::{load_infra, load_workloads};
use kuberina_solver::phase0::pre_deduct_daemonsets;
use kuberina_solver::unplaced_term::derive_unplaced_weights;

/// Seconds `work` takes, averaged over `calls` calls.
fn seconds_per_call<T>(calls: usize, mut work: impl FnMut() -> T) -> f64 {
    let start = Instant::now();
    for _ in 0..calls {
        std::hint::black_box(work());
    }
    start.elapsed().as_secs_f64() / calls as f64
}

/// The walk without the repair: best fit or reject. Returns the Pods it rejected.
fn insertion_only_walk(pods: &[Pod], nodes: &[Node], order: &[usize]) -> usize {
    let mut packing = Packing::new(pods, nodes);
    let mut rejected = 0;
    for &pod_idx in order {
        match packing.best_fit_node(pod_idx) {
            Some(node_idx) => packing.place(pod_idx, node_idx),
            None => rejected += 1,
        }
    }
    rejected
}

fn report_walk(pods: &[Pod], nodes: &[Node], order: &[usize]) -> Blueprint {
    let start = Instant::now();
    let walked = admit_in_order(pods, nodes, order);
    println!(
        "walk, total                  {:>10.3} s",
        start.elapsed().as_secs_f64()
    );
    let start = Instant::now();
    let rejected = insertion_only_walk(pods, nodes, order);
    println!(
        "walk, insertion only         {:>10.3} s ({rejected} Pods rejected without repair)",
        start.elapsed().as_secs_f64()
    );
    let packing = Packing::from_assignment(pods, nodes, &walked.assignment);
    let check = seconds_per_call(1, || in_order_violation(&packing, order));
    println!("in-order rule, one full check {:>9.3} s", check);
    walked
}

fn report_fitness(walked: &Blueprint, pods: &[Pod], nodes: &[Node]) {
    let base = FitnessWeights::default();
    let weights = FitnessWeights {
        unplaced: derive_unplaced_weights(pods, nodes, &base),
        ..base
    };
    let calls = 20;
    let total = seconds_per_call(calls, || {
        compute_fitness(walked, pods, nodes, &[], &weights)
    });
    let stranded = seconds_per_call(calls, || stranded_gpu(&walked.assignment, pods, nodes));
    let overflow = seconds_per_call(calls, || {
        device_overflow_per_node(&walked.assignment, pods, nodes)
    });
    println!("fitness, one evaluation      {:>10.3} ms", total * 1e3);
    println!("  of which stranded GPU       {:>9.3} ms", stranded * 1e3);
    println!("  of which device overflow    {:>9.3} ms", overflow * 1e3);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let [_, infra, workloads] = args.as_slice() else {
        panic!(
            "expected <infra.yaml> <workloads.yaml>, got {:?}",
            &args[1..]
        );
    };
    let (raw_nodes, daemon_sets) = load_infra(infra).expect("infrastructure document");
    let nodes = pre_deduct_daemonsets(&raw_nodes, &daemon_sets);
    let (pods, _) = load_workloads(workloads).expect("workloads document");
    let order = arrival_order(&pods).expect("every Pod has an arrival");
    let walked = report_walk(&pods, &nodes, &order);
    report_fitness(&walked, &pods, &nodes);
}
