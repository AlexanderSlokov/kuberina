from kuberina.model.types import Blueprint, Node, Pod, ResourceVector, FitnessWeights
from kuberina.fitness import compute_fitness, compute_hard_penalty
from kuberina.phases.csp import compute_capacity_overflow

pods = [Pod(name="p", namespace="ns", requests=ResourceVector(cpu=5.0))]
nodes = [Node(name="n", allocatable=ResourceVector(cpu=4.0, ram=16.0))]
bp = Blueprint(
    assignment=[0],
    node_load=[ResourceVector(cpu=5.0)],
)
overflow = compute_capacity_overflow(bp.assignment, pods, nodes)
print("Overflow:", overflow)
penalty = compute_hard_penalty(bp, pods, nodes, [])
print("Penalty:", penalty)
fit = compute_fitness(bp, pods, nodes, [], FitnessWeights())
print("Fitness:", fit)
