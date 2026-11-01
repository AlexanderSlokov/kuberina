"""Read Kuberina IR v1 resource blocks and workloads documents for the inspector.

Written from docs/references/ir-v1.md, without reference to the solver's parser. The
inspector's verdict is only worth something while the two implementations stay
independent.
"""


def _as_float(value, dimension):
    """Coerce one IR quantity to a float, naming it if the shape is wrong."""
    try:
        return float(value)
    except (TypeError, ValueError):
        raise ValueError(
            f"resource dimension {dimension!r} must be a number, got {value!r}. "
            "Kubernetes suffix notation (e.g. '256Mi') is understood by the solver "
            "but not by this validator."
        ) from None


def flatten_resources(block):
    """Project a Kuberina IR resource block onto the flat 8-dimension vector.

    IR v0.2.0 nests throughput under `disk: {read, write}` and
    `network: {in, out}`. Absent dimensions stay absent so that
    pre_deduct_daemonsets can tell "declared zero" from "not declared".

    Example:
        >>> flatten_resources({"cpu": 4.0, "network": {"in": 500.0}})
        {'cpu': 4.0, 'net_in': 500.0}
    """
    disk = block.get("disk") or {}
    network = block.get("network") or {}
    sources = (
        ("cpu", block.get("cpu")),
        ("ram", block.get("ram")),
        ("gpu", block.get("gpu")),
        ("storage", block.get("storage")),
        ("disk_read", disk.get("read")),
        ("disk_write", disk.get("write")),
        ("net_in", network.get("in")),
        ("net_out", network.get("out")),
    )
    return {r: _as_float(v, r) for r, v in sources if v is not None}


def replica_name(name, index, total):
    """Name one unrolled replica (ir-v1.md §3.1).

    Example:
        >>> replica_name("worker", 3, 18)
        'worker-0003'
    """
    return f"{name}-0" if total <= 1 else f"{name}-{index:04d}"


def unroll_workloads(workloads):
    """Turn every declaration into one pod dict per replica, in document order.

    Each pod carries its `identity` (`namespace/podname`), its `declaration`
    (`namespace/name`), flattened `requests`, and its targets as written.

    Example:
        >>> unroll_workloads({"namespaces": {"ai": [{"name": "web"}]}})[0]["identity"]
        'ai/web-0'
    """
    pods = []
    for namespace in sorted(workloads.get("namespaces") or {}):
        for declaration in workloads["namespaces"][namespace]:
            pods.extend(_replicas_of(namespace, declaration))
    return pods


def _replicas_of(namespace, declaration):
    """The unrolled pods of one declaration."""
    total = max(int(declaration.get("replicas", 1)), 1)
    requests = flatten_resources(declaration.get("requests") or {})
    name = f"{namespace}/{declaration['name']}"
    device_fields = _gpu_device_fields(name, declaration.get("gpuMilli"), requests)
    return [
        {
            "identity": f"{namespace}/{replica_name(declaration['name'], i, total)}",
            "declaration": name,
            "requests": requests,
            **device_fields,
            **({"arrival": declaration["arrival"]} if "arrival" in declaration else {}),
            "nodeSelector": declaration.get("nodeSelector") or {},
            "affinity": list(declaration.get("affinity") or []),
            "antiAffinity": list(declaration.get("antiAffinity") or []),
            **({"topologySpread": declaration["topologySpread"]}
               if declaration.get("topologySpread") else {}),
        }
        for i in range(total)
    ]


def _gpu_device_fields(declaration, gpu_milli, requests):
    """Read `gpuMilli` (ir-v1.md §3.4); rewrite `requests["gpu"]` to the aggregate.

    With `gpuMilli: m`, `requests.gpu` counts devices: a whole number of at least 1.
    The pod's GPU demand in the per-node sum becomes `devices * m / 1000`.

    Example:
        >>> requests = {"gpu": 1.0}
        >>> _gpu_device_fields("ai/inference", 500, requests), requests["gpu"]
        ({'gpuDevices': 1, 'gpuMilli': 500}, 0.5)
    """
    if gpu_milli is None:
        return {}
    if not isinstance(gpu_milli, int) or not 1 <= gpu_milli <= 1000:
        raise ValueError(f"{declaration} has gpuMilli: {gpu_milli!r}; expected an integer in 1..=1000")
    devices = requests.get("gpu", 0.0)
    if devices < 1 or devices != int(devices):
        raise ValueError(
            f"{declaration} has gpuMilli with requests.gpu: {devices!r}; "
            "expected a whole number of devices, at least 1"
        )
    requests["gpu"] = int(devices) * gpu_milli / 1000
    return {"gpuDevices": int(devices), "gpuMilli": gpu_milli}


def resolve_targets(pods):
    """Replace each target with the identities it names (ir-v1.md §3.1.1).

    A target names a declaration (every replica) or one unrolled identity. The pod
    itself is never its own target. Each `affinity` entry becomes its own group of
    identities, since one of them on the node satisfies it; `antiAffinity` becomes one
    flat list. Raises ValueError for a target that names nothing.

    Example:
        >>> pods = unroll_workloads({"namespaces": {"ai": [
        ...     {"name": "web", "replicas": 2, "antiAffinity": ["ai/web"]}]}})
        >>> resolve_targets(pods)[0]["antiAffinity"]
        ['ai/web-0001']
    """
    replicas_of = {}
    for pod in pods:
        replicas_of.setdefault(pod["declaration"], []).append(pod["identity"])
    identities = {pod["identity"] for pod in pods}
    for pod in pods:
        tables = (replicas_of, identities)
        pod["affinity"] = [_others(t, pod, *tables) for t in pod["affinity"]]
        pod["antiAffinity"] = [i for t in pod["antiAffinity"] for i in _others(t, pod, *tables)]
    return pods


def _others(target, pod, replicas_of, identities):
    """Identities a target names, leaving out the pod that declares it."""
    named = _named_by(target, pod, replicas_of, identities)
    return [i for i in named if i != pod["identity"]]


def _named_by(target, pod, replicas_of, identities):
    """Identities that one target names: a declaration first, then one pod."""
    if target in replicas_of:
        return replicas_of[target]
    if target in identities:
        return [target]
    raise ValueError(
        f"pod {pod['identity']} has target {target!r}, which matches no declaration "
        "and no pod; expected namespace/name or namespace/name-NNNN"
    )
