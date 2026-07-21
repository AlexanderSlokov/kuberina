"""YAML parser for Kuberina cluster topology and workload manifests.

Reads two YAML files:
1. Infrastructure topology (nodes + daemonsets)
2. Workload manifests (pods + pod groups)

Resource format: simple numeric (CPU in cores, RAM in GiB, GPU in units).
MVP does not parse K8s resource.Quantity notation — that's for the Go version.
"""

from __future__ import annotations

from pathlib import Path
from typing import Any

import yaml

from kuberina.model.types import (
    DaemonSet,
    Node,
    Pod,
    PodGroup,
    ResourceVector,
)


def load_infra(path: str | Path) -> tuple[list[Node], list[DaemonSet]]:
    """Parse cluster topology YAML into Node and DaemonSet lists.

    Expected format: see research/testdata/homelab_infra.yaml

    Example:
        >>> nodes, ds = load_infra("research/testdata/homelab_infra.yaml")
        >>> len(nodes) > 0
        True
    """
    raw = _read_yaml(path)
    nodes = [_parse_node(n) for n in raw.get("nodes", [])]
    daemon_sets = [_parse_daemonset(d) for d in raw.get("daemonsets", [])]
    return nodes, daemon_sets


def load_workloads(path: str | Path) -> tuple[list[Pod], list[PodGroup]]:
    """Parse workload manifests YAML into Pod and PodGroup lists.

    Expected format: see research/testdata/homelab_workloads.yaml

    Example:
        >>> pods, groups = load_workloads("research/testdata/homelab_workloads.yaml")
        >>> len(pods) > 0
        True
    """
    raw = _read_yaml(path)
    pods = [_parse_pod(p) for p in raw.get("pods", [])]
    groups = _resolve_groups(raw.get("groups", []), pods)
    return pods, groups


def _read_yaml(path: str | Path) -> dict[str, Any]:
    """Read and parse a YAML file, raising clear errors."""
    file_path = Path(path)
    if not file_path.exists():
        raise FileNotFoundError(
            f"YAML file not found: {file_path} (expected absolute or relative path)",
        )
    with file_path.open() as f:
        return yaml.safe_load(f)


def _parse_resources(raw: dict[str, Any]) -> ResourceVector:
    """Parse a resource dict into a ResourceVector."""
    return ResourceVector(
        cpu=float(raw.get("cpu", 0.0)),
        ram=float(raw.get("ram", 0.0)),
        gpu=float(raw.get("gpu", 0.0)),
    )


def _parse_node(raw: dict[str, Any]) -> Node:
    """Parse a single node entry from YAML."""
    return Node(
        name=raw["name"],
        allocatable=_parse_resources(raw.get("allocatable", {})),
        labels=raw.get("labels", {}),
        taints=raw.get("taints", []),
        zone=raw.get("zone", ""),
    )


def _parse_daemonset(raw: dict[str, Any]) -> DaemonSet:
    """Parse a single daemonset entry from YAML."""
    return DaemonSet(
        name=raw["name"],
        resources=_parse_resources(raw.get("resources", {})),
        node_selector=raw.get("nodeSelector", {}),
        tolerations=raw.get("tolerations", []),
    )


def _parse_pod(raw: dict[str, Any]) -> Pod:
    """Parse a single pod entry from YAML."""
    return Pod(
        name=raw["name"],
        namespace=raw.get("namespace", "default"),
        requests=_parse_resources(raw.get("requests", {})),
        tolerations=raw.get("tolerations", []),
        node_selector=raw.get("nodeSelector", {}),
        affinity_targets=raw.get("affinity", []),
        anti_affinity_targets=raw.get("antiAffinity", []),
        group_name=raw.get("group", ""),
    )


def _resolve_groups(
    raw_groups: list[dict[str, Any]],
    pods: list[Pod],
) -> list[PodGroup]:
    """Build PodGroup objects by resolving pod names to indices."""
    name_to_idx = {pod.name: i for i, pod in enumerate(pods)}
    groups: list[PodGroup] = []

    for raw in raw_groups:
        group_name = raw["name"]
        member_indices = [
            name_to_idx[pod.name]
            for pod in pods
            if pod.group_name == group_name
        ]
        groups.append(PodGroup(
            name=group_name,
            pod_indices=member_indices,
            min_members=raw.get("minMembers", len(member_indices)),
            node_selector=raw.get("nodeSelector", {}),
            colocate=raw.get("colocate", False),
        ))

    return groups
