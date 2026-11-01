"""Phase 2: Genetic Algorithm optimizer with CSP-integrated operators.

From PAPER.md §4.3 and DESIGN.md §GA Operators.
Population from ga_estimation.md §3: 128 (small), 512 (medium).
Tournament selection k=3, uniform crossover + gang repair,
mutation rate=5% + CSP rollback, early stopping.
"""

from __future__ import annotations

import copy
import logging
import random
import sys

from kuberina.fitness import compute_fitness
from kuberina.model.types import (
    Blueprint,
    FitnessWeights,
    GAConfig,
    Node,
    Pod,
    PodGroup,
    ResourceVector,
)
from kuberina.phases.csp import can_place_pod_on_node

logger = logging.getLogger(__name__)


def run_ga(
    seed: Blueprint,
    pods: list[Pod],
    nodes: list[Node],
    groups: list[PodGroup],
    config: GAConfig,
    fitness_weights: FitnessWeights,
) -> Blueprint:
    """Execute the GA optimization loop starting from FFD seed.

    Returns the best blueprint found after convergence or max generations.
    From ga_estimation.md §5: with FFD warm-start, converges 3-5x faster.

    Example:
        >>> # (see test_integration.py for full example)
    """
    rng = random.Random(config.random_seed)
    population = _init_population(seed, pods, nodes, groups, config, rng)
    _evaluate_all(population, pods, nodes, groups, fitness_weights)

    best = _find_best(population)
    # WHY two variables (#20): `best` tracks every improvement so the run never
    # returns a worse plan, but the stale counter measures progress against the last
    # gain that mattered. Comparing staleness to `best` let 0.0003 on a fitness of
    # 1.58 million reset the counter forever, so the criterion never fired.
    anchor_fitness = best.fitness
    stale_count = 0

    for gen in range(config.max_generations):
        offspring = _breed_generation(
            population, pods, nodes, groups, config, fitness_weights, rng,
        )
        _evaluate_all(offspring, pods, nodes, groups, fitness_weights)

        population = _select_survivors(population, offspring, config)
        current_best = _find_best(population)

        if current_best.fitness < best.fitness:
            best = copy.deepcopy(current_best)

        if _exceeds_improvement_threshold(
            anchor_fitness, best.fitness, config.min_relative_improvement,
        ):
            anchor_fitness = best.fitness
            stale_count = 0
        else:
            stale_count += 1

        if stale_count >= config.early_stop_generations:
            logger.info("Early stop at generation %d (gain under %.4f%% for %d gens)",
                        gen, config.min_relative_improvement * 100.0,
                        config.early_stop_generations)
            break

        if gen % 50 == 0:
            logger.info("Gen %d: best_fitness=%.4f", gen, best.fitness)

    return best


def _exceeds_improvement_threshold(
    anchor: float, candidate: float, min_relative: float,
) -> bool:
    """Is the gain from `anchor` to `candidate` worth resetting the stale counter for?

    Fitness is minimized, so a gain is a decrease. The test is relative because an
    absolute gain means nothing across instance sizes: 0.0003 is progress on a
    fitness of 1.0 and rounding error on a fitness of 1.58 million (#20).

    Example:
        >>> _exceeds_improvement_threshold(1_000_000.0, 999_000.0, 1e-4)
        True
        >>> _exceeds_improvement_threshold(1_000_000.0, 999_999.0, 1e-4)
        False
    """
    gain = anchor - candidate
    if gain <= 0.0:
        return False
    scale = abs(anchor)
    if scale <= sys.float_info.epsilon:
        # A zero anchor has no scale to measure against, so any gain counts.
        return True
    return gain / scale > min_relative


def _init_population(
    seed: Blueprint,
    pods: list[Pod],
    nodes: list[Node],
    groups: list[PodGroup],
    config: GAConfig,
    rng: random.Random,
) -> list[Blueprint]:
    """Create initial population from FFD seed + random perturbations."""
    population = [copy.deepcopy(seed)]

    for _ in range(config.population_size - 1):
        variant = copy.deepcopy(seed)
        # WHY: perturb ~20% of assignments to inject diversity into the
        # FFD-derived population. Pure clones would make crossover useless.
        mutate(variant, pods, nodes, groups, rate=0.2, rng=rng)
        population.append(variant)

    return population


def _evaluate_all(
    population: list[Blueprint],
    pods: list[Pod],
    nodes: list[Node],
    groups: list[PodGroup],
    weights: FitnessWeights,
) -> None:
    """Evaluate fitness for every individual in the population."""
    for bp in population:
        bp.node_load = _recompute_node_loads(bp.assignment, pods, len(nodes))
        bp.fitness = compute_fitness(bp, pods, nodes, groups, weights)


def _breed_generation(
    population: list[Blueprint],
    pods: list[Pod],
    nodes: list[Node],
    groups: list[PodGroup],
    config: GAConfig,
    weights: FitnessWeights,
    rng: random.Random,
) -> list[Blueprint]:
    """Produce offspring via selection, crossover, and mutation."""
    offspring: list[Blueprint] = []
    target_size = config.population_size

    while len(offspring) < target_size:
        p1 = tournament_select(population, config.tournament_size, rng)
        p2 = tournament_select(population, config.tournament_size, rng)

        if rng.random() < config.crossover_rate:
            child = uniform_crossover(p1, p2, rng)
            repair_gangs(child, p1, p2, groups, pods, nodes)
        else:
            child = copy.deepcopy(p1 if p1.fitness < p2.fitness else p2)

        mutate(child, pods, nodes, groups, config.mutation_rate, rng)
        offspring.append(child)

    return offspring[:target_size]


def _select_survivors(
    parents: list[Blueprint],
    offspring: list[Blueprint],
    config: GAConfig,
) -> list[Blueprint]:
    """Elitism: keep best parents, fill rest with offspring."""
    combined = parents + offspring
    combined.sort(key=lambda bp: bp.fitness)
    return combined[:config.population_size]


def _find_best(population: list[Blueprint]) -> Blueprint:
    """Return the individual with the lowest (best) fitness."""
    return min(population, key=lambda bp: bp.fitness)


def tournament_select(
    population: list[Blueprint],
    k: int,
    rng: random.Random,
) -> Blueprint:
    """Pick k random individuals, return the one with best (lowest) fitness.

    From ga_estimation.md §4: tournament selection with k_tour=3.

    Example:
        >>> # (tournament_select is stochastic, tested in test_phase2.py)
    """
    candidates = rng.sample(population, min(k, len(population)))
    return min(candidates, key=lambda bp: bp.fitness)


def uniform_crossover(
    p1: Blueprint,
    p2: Blueprint,
    rng: random.Random,
) -> Blueprint:
    """Gene-level uniform crossover: each gene from p1 or p2 with 50% chance.

    From PAPER.md §4.3 point 3: "Uniform Crossover".

    Example:
        >>> # (tested in test_phase2.py)
    """
    child_assignment = [
        p1.assignment[i] if rng.random() < 0.5 else p2.assignment[i]
        for i in range(len(p1.assignment))
    ]
    return Blueprint(assignment=child_assignment, fitness=0.0)


def repair_gangs(
    child: Blueprint,
    p1: Blueprint,
    p2: Blueprint,
    groups: list[PodGroup],
    pods: list[Pod],
    nodes: list[Node],
) -> None:
    """Fix gang assignments broken by crossover.

    From DESIGN.md crossover pseudocode:
    If crossover split a gang and broke capacity, rollback the entire
    gang assignment to the better parent.

    Example:
        >>> # (tested in test_phase2.py)
    """
    for group in groups:
        child.node_load = _recompute_node_loads(
            child.assignment, pods, len(nodes),
        )
        if _gang_is_feasible(child, group, pods, nodes):
            continue

        # Rollback to better parent
        source = p1 if p1.fitness <= p2.fitness else p2
        for pod_idx in group.pod_indices:
            child.assignment[pod_idx] = source.assignment[pod_idx]


def mutate(
    blueprint: Blueprint,
    pods: list[Pod],
    nodes: list[Node],
    groups: list[PodGroup],
    rate: float,
    rng: random.Random,
) -> None:
    """Random reset mutation with CSP rollback for gang pods.

    From DESIGN.md mutate pseudocode:
    - Regular pod: move to random eligible node
    - Gang pod: try move, rollback if gang becomes infeasible

    Example:
        >>> # (tested in test_phase2.py)
    """
    group_lookup = _build_group_lookup(groups, len(pods))
    num_nodes = len(nodes)

    for i in range(len(blueprint.assignment)):
        if rng.random() > rate:
            continue

        old_node = blueprint.assignment[i]
        eligible = _eligible_nodes_for_pod(pods[i], nodes)
        if not eligible:
            continue

        new_node = rng.choice(eligible)
        blueprint.assignment[i] = new_node

        group_idx = group_lookup[i]
        if group_idx < 0:
            continue

        # Pod belongs to a gang — check if gang is still feasible
        blueprint.node_load = _recompute_node_loads(
            blueprint.assignment, pods, num_nodes,
        )
        if not _gang_is_feasible(blueprint, groups[group_idx], pods, nodes):
            blueprint.assignment[i] = old_node  # rollback


def _gang_is_feasible(
    blueprint: Blueprint,
    group: PodGroup,
    pods: list[Pod],
    nodes: list[Node],
) -> bool:
    """Check if all gang pods fit on their assigned nodes."""
    for pod_idx in group.pod_indices:
        node_idx = blueprint.assignment[pod_idx]
        load = blueprint.node_load[node_idx]
        if not nodes[node_idx].allocatable.fits(load):
            return False
    return True


def _build_group_lookup(
    groups: list[PodGroup],
    num_pods: int,
) -> list[int]:
    """Map pod_idx -> group_idx (-1 if pod is not in any gang)."""
    lookup = [-1] * num_pods
    for g_idx, group in enumerate(groups):
        for pod_idx in group.pod_indices:
            lookup[pod_idx] = g_idx
    return lookup


def _eligible_nodes_for_pod(pod: Pod, nodes: list[Node]) -> list[int]:
    """Return indices of nodes that pass taint+selector checks for a pod."""
    return [
        j for j, node in enumerate(nodes)
        if can_place_pod_on_node(pod, node)
    ]


def _recompute_node_loads(
    assignment: list[int],
    pods: list[Pod],
    num_nodes: int,
) -> list[ResourceVector]:
    """Recompute per-node resource loads from scratch."""
    loads = [ResourceVector.zero() for _ in range(num_nodes)]
    for pod_idx, node_idx in enumerate(assignment):
        loads[node_idx] = loads[node_idx].add(pods[pod_idx].requests)
    return loads
