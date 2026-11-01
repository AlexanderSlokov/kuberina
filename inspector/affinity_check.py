"""Count affinity and anti-affinity violations in a blueprint (ir-v1.md §3.1.1).

Written from the specification, without reference to the solver's fitness code.
"""


def count_affinity_violations(pods, solution):
    """Count the rules that a placement breaks, with Kubernetes semantics.

    `pods` holds resolved targets (see ir_workloads.resolve_targets). `solution` maps
    each identity to a node name. An affinity entry is broken when none of the pods it
    names shares the pod's node. An anti-affinity violation is one pair on the same
    node, counted once even when both pods name each other. Rules that involve an
    unplaced pod are not judged; the unplaced pod is reported elsewhere.

    Example:
        >>> count_affinity_violations(
        ...     [{"identity": "ns/a", "affinity": [["ns/b"]], "antiAffinity": []},
        ...      {"identity": "ns/b", "affinity": [], "antiAffinity": []}],
        ...     {"ns/a": "n1", "ns/b": "n2"})
        1
    """
    return _unmet_affinity_entries(pods, solution) + _colocated_pairs(pods, solution)


def _unmet_affinity_entries(pods, solution):
    """Affinity entries with no named pod on the declaring pod's node."""
    unmet = 0
    for pod in pods:
        node = solution.get(pod["identity"])
        for entry in pod["affinity"]:
            placed = [solution[t] for t in entry if t in solution]
            if node is not None and placed and node not in placed:
                unmet += 1
    return unmet


def _colocated_pairs(pods, solution):
    """Distinct anti-affinity pairs whose two pods share a node."""
    pairs = set()
    for pod in pods:
        node = solution.get(pod["identity"])
        for target in pod["antiAffinity"]:
            if node is not None and solution.get(target) == node:
                pairs.add(tuple(sorted((pod["identity"], target))))
    return len(pairs)
