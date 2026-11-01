"""Count affinity and anti-affinity violations in a blueprint (ir-v1.md §3.1.1).

Written from the specification, without reference to the solver's fitness code.
"""


def count_affinity_violations(pods, solution):
    """Count the distinct rules that a placement breaks.

    `pods` holds resolved targets (see ir_workloads.resolve_targets). `solution` maps
    each identity to a node name. A rule declared from both sides counts once.

    Example:
        >>> count_affinity_violations(
        ...     [{"identity": "ns/a", "affinity": ["ns/b"], "antiAffinity": []},
        ...      {"identity": "ns/b", "affinity": [], "antiAffinity": []}],
        ...     {"ns/a": "n1", "ns/b": "n2"})
        1
    """
    rules = set()
    for pod in pods:
        for target in pod["affinity"]:
            rules.add((*sorted((pod["identity"], target)), True))
        for target in pod["antiAffinity"]:
            rules.add((*sorted((pod["identity"], target)), False))
    return sum(1 for a, b, together in rules if _breaks(solution, a, b, together))


def _breaks(solution, a, b, wants_together):
    """Whether one rule is broken by where its two pods landed.

    A rule with an unplaced pod is not judged: the unplaced pod is reported elsewhere.
    """
    if a not in solution or b not in solution:
        return False
    same_node = solution[a] == solution[b]
    return same_node != wants_together
