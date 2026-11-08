//! GA mutation whose moves keep every node within capacity (backlog X-8).
//!
//! On the Alibaba seed-42 instance, the blind place, move, and swap operators breached
//! capacity in 179 to 300 of 300 trials, and a breach costs the whole child
//! (`docs/references/sessions/2026-11-06-free-ga-moves.md`, at `99a4b8a`). Plan #24, §8
//! decision 12 repairs them: each operator here checks the fit on a `Packing` before it
//! moves a Pod, so a child that starts within capacity stays within it. Uniform
//! crossover is not covered; the Founder decides on it after X-8.

use rand::Rng;
use rand::rngs::StdRng;
use rand::seq::IndexedRandom;

use crate::admission_swap::trade_random_gpu_pod;
use crate::model::{Blueprint, Node, Pod, UNPLACED};
use crate::packing::Packing;

/// Share of selected placed genes that free admission unplaces (X-4).
///
/// WHY 0.2: untuned. It keeps most selected genes on the relocation operators that
/// improve packing, while the unplaced-Pod term (`unplaced_term`) judges each unplace.
/// S-2 measures it with the other settings on the Alibaba instance.
pub const UNPLACE_SHARE: f64 = 0.2;

/// Share of free-admission mutations that end with one trade of a placed GPU Pod for
/// several unplaced Pods (`admission_swap`, S-2c).
///
/// WHY 0.2: untuned, like `UNPLACE_SHARE`; plan #24, §8 decision 9 records both as
/// untuned.
pub const TRADE_SHARE: f64 = 0.2;

/// Mutate `blueprint` for the run's admission mode; each gene is selected with `rate`.
///
/// Free admission first places or unplaces selected genes, then relocates, and in a
/// share of calls ends with a trade. Without free admission, only relocation runs, and
/// admission stays as the seed decided it (X-4). Node loads are left for the caller to
/// recompute, as `evaluate_all` does.
///
/// WHY no gang rollback: a move that fits leaves no node past capacity, so a gang that
/// was within capacity stays within it. The blind operators needed the rollback.
///
/// ```ignore
/// mutate_within_capacity(config.free_admission, &mut child, &pods, &nodes, rate, &mut rng);
/// ```
pub fn mutate_within_capacity(
    free_admission: bool,
    blueprint: &mut Blueprint,
    pods: &[Pod],
    nodes: &[Node],
    rate: f64,
    rng: &mut StdRng,
) {
    let mut packing = Packing::from_assignment(pods, nodes, &blueprint.assignment);
    if free_admission {
        place_or_unplace_selected(&mut packing, rate, rng);
    }
    relocate_selected(&mut packing, rate, rng);
    // WHY last: relocation would scatter the refilled node before scoring.
    if free_admission && rng.random_bool(TRADE_SHARE) {
        trade_random_gpu_pod(&mut packing, rng);
    }
    blueprint.assignment = packing.into_assignment();
}

/// Free admission's genes: place a selected unplaced Pod where it fits, and unplace a
/// share of selected placed Pods.
fn place_or_unplace_selected(packing: &mut Packing, rate: f64, rng: &mut StdRng) {
    for pod_idx in 0..packing.pods().len() {
        if rng.random::<f64>() > rate {
            continue;
        }
        if packing.node_of(pod_idx) == UNPLACED {
            place_on_fitting_node(packing, pod_idx, rng);
        } else if rng.random_bool(UNPLACE_SHARE) {
            packing.remove(pod_idx);
        }
    }
}

/// Move or swap each selected placed Pod, half of them each way.
fn relocate_selected(packing: &mut Packing, rate: f64, rng: &mut StdRng) {
    let num_pods = packing.pods().len();
    for pod_idx in 0..num_pods {
        if rng.random::<f64>() > rate || packing.node_of(pod_idx) == UNPLACED {
            continue;
        }
        if rng.random_bool(0.5) {
            let other = rng.random_range(0..num_pods);
            swap_if_both_fit(packing, pod_idx, other);
        } else {
            move_to_fitting_node(packing, pod_idx, rng);
        }
    }
}

/// Put unplaced Pod `pod_idx` on a random node where it fits; false if none has room.
///
/// ```ignore
/// let placed = place_on_fitting_node(&mut packing, pod_idx, &mut rng);
/// ```
pub fn place_on_fitting_node(packing: &mut Packing, pod_idx: usize, rng: &mut StdRng) -> bool {
    let Some(&target) = packing.fitting_nodes(pod_idx).choose(rng) else {
        return false;
    };
    packing.place(pod_idx, target);
    true
}

/// Move placed Pod `pod_idx` to a random other node where it fits; false if none has room.
///
/// ```ignore
/// let moved = move_to_fitting_node(&mut packing, pod_idx, &mut rng);
/// ```
pub fn move_to_fitting_node(packing: &mut Packing, pod_idx: usize, rng: &mut StdRng) -> bool {
    let source = packing.node_of(pod_idx);
    // WHY with the Pod still placed: it occupies only `source`, which is excluded.
    let targets: Vec<usize> = packing
        .fitting_nodes(pod_idx)
        .into_iter()
        .filter(|&node_idx| node_idx != source)
        .collect();
    let Some(&target) = targets.choose(rng) else {
        return false;
    };
    packing.remove(pod_idx);
    packing.place(pod_idx, target);
    true
}

/// Exchange the nodes of placed Pods `a` and `b` when each fits in the other's place.
///
/// Returns false, and changes nothing, when they share a node, one is unplaced, or
/// either does not fit.
///
/// ```ignore
/// let swapped = swap_if_both_fit(&mut packing, a, b);
/// ```
pub fn swap_if_both_fit(packing: &mut Packing, a: usize, b: usize) -> bool {
    let (node_a, node_b) = (packing.node_of(a), packing.node_of(b));
    if node_a == UNPLACED || node_b == UNPLACED || node_a == node_b {
        return false;
    }
    if !packing.fits_in_place_of(a, b) || !packing.fits_in_place_of(b, a) {
        return false;
    }
    packing.remove(a);
    packing.remove(b);
    packing.place(a, node_b);
    packing.place(b, node_a);
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_fixtures::{breaches_a_node, cpu_node, cpu_pod, nearly_full_cluster};
    use rand::SeedableRng;

    fn blueprint_of(assignment: Vec<usize>) -> Blueprint {
        Blueprint {
            assignment,
            fitness: 0.0,
            node_load: Vec::new(),
            scorecard: Default::default(),
        }
    }

    fn mutated(free_admission: bool, assignment: &[usize], seed: u64) -> Vec<usize> {
        let (pods, nodes, _) = nearly_full_cluster();
        let mut bp = blueprint_of(assignment.to_vec());
        let mut rng = StdRng::seed_from_u64(seed);
        mutate_within_capacity(free_admission, &mut bp, &pods, &nodes, 1.0, &mut rng);
        bp.assignment
    }

    #[test]
    fn relocation_never_breaches_a_nearly_full_cluster() {
        let (pods, nodes, assignment) = nearly_full_cluster();
        for seed in 0..64 {
            let after = mutated(false, &assignment, seed);
            assert!(
                !breaches_a_node(&after, &pods, &nodes),
                "seed {seed}: {after:?}"
            );
        }
    }

    #[test]
    fn free_admission_never_places_a_pod_where_it_does_not_fit() {
        let (pods, nodes, assignment) = nearly_full_cluster();
        for seed in 0..64 {
            let after = mutated(true, &assignment, seed);
            assert!(
                !breaches_a_node(&after, &pods, &nodes),
                "seed {seed}: {after:?}"
            );
        }
    }

    #[test]
    fn mutation_leaves_unplaced_pods_unplaced() {
        let assignment = vec![0, UNPLACED, 1, UNPLACED, 2, UNPLACED];
        let after = mutated(false, &assignment, 7);
        assert_eq!(
            (after[1], after[3], after[5]),
            (UNPLACED, UNPLACED, UNPLACED)
        );
        assert!(after[0] != UNPLACED && after[2] != UNPLACED && after[4] != UNPLACED);
    }

    #[test]
    fn free_admission_mutation_places_unplaced_pods() {
        let after = mutated(true, &[UNPLACED; 6], 7);
        assert!(after[..5].iter().any(|&n| n != UNPLACED), "{after:?}");
    }

    #[test]
    fn free_admission_mutation_can_unplace_a_pod() {
        let pods: Vec<Pod> = (0..64).map(|i| cpu_pod(&format!("p{i}"), 1.0)).collect();
        let nodes = vec![cpu_node("a", 99.0), cpu_node("b", 99.0)];
        let mut bp = blueprint_of(vec![0; 64]);
        let mut rng = StdRng::seed_from_u64(7);
        mutate_within_capacity(true, &mut bp, &pods, &nodes, 1.0, &mut rng);
        assert!(bp.assignment.contains(&UNPLACED));
    }

    #[test]
    fn a_move_goes_to_the_only_other_node_with_room() {
        let (pods, nodes, assignment) = nearly_full_cluster();
        let mut packing = Packing::from_assignment(&pods, &nodes, &assignment);
        let mut rng = StdRng::seed_from_u64(1);
        assert!(move_to_fitting_node(&mut packing, 0, &mut rng));
        assert_eq!(packing.node_of(0), 2);
        // Now c is full and a has room, so a Pod from b can only go to a.
        assert!(move_to_fitting_node(&mut packing, 2, &mut rng));
        assert_eq!(packing.node_of(2), 0);
    }

    #[test]
    fn a_pod_that_fits_nowhere_stays_unplaced() {
        let (pods, nodes, assignment) = nearly_full_cluster();
        let mut packing = Packing::from_assignment(&pods, &nodes, &assignment);
        let mut rng = StdRng::seed_from_u64(1);
        assert!(!place_on_fitting_node(&mut packing, 5, &mut rng));
        assert_eq!(packing.node_of(5), UNPLACED);
    }

    #[test]
    fn a_swap_happens_only_when_each_pod_fits_the_other_place() {
        let pods = vec![
            cpu_pod("big", 3.0),
            cpu_pod("small", 1.0),
            cpu_pod("fill", 3.0),
        ];
        let nodes = vec![cpu_node("a", 4.0), cpu_node("b", 4.0)];
        let mut packing = Packing::from_assignment(&pods, &nodes, &[0, 1, 1]);
        // `big` cannot take `small`'s place: `fill` leaves one core on b.
        assert!(!swap_if_both_fit(&mut packing, 0, 1));
        assert_eq!(packing.assignment(), &[0, 1, 1]);
        // `big` and `fill` are the same size, so they can trade nodes.
        assert!(swap_if_both_fit(&mut packing, 0, 2));
        assert_eq!(packing.assignment(), &[1, 1, 0]);
    }
}
