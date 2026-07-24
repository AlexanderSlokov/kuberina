#!/usr/bin/env python3
"""Generate MSC Irina-scale datacenter test data for Kuberina stress test.

Target: 150 nodes, 3000 pods, ~92% fill rate, hostile constraint matrix.
This script is a one-shot generator — run it once, commit the YAML.

Node types:
  - 100 Standard:        64C / 256G / 0GPU
  - 30 Memory-Optimized: 32C / 512G / 0GPU (label: disk=nvme)
  - 20 GPU-Optimized:    48C / 192G / 8GPU (label: gpu=nvidia-a100)

Workload design:
  - 50 microservices, each with 10-100 replicas = ~3000 pods total
  - 10 services with hard anti-affinity (no 2 pods on same node)
  - 5 affinity chains (A→B→C)
  - 500 GPU pods competing for 20 GPU nodes
  - Total resource ~92% of cluster capacity
"""

from __future__ import annotations

import math
import random
import yaml


SEED = 42
RNG = random.Random(SEED)

# ─── Zone distribution ───────────────────────────────────────────────
ZONES = ["us-east-1a", "us-east-1b", "us-east-1c"]


def generate_infra() -> dict:
    """Build 150-node datacenter topology with 3 node types."""
    nodes = []
    daemonsets = _build_daemonsets()

    # 120 Standard nodes: 64C / 256G (was 100, +20%)
    for i in range(120):
        nodes.append(_make_node(
            name=f"std-{i:03d}",
            cpu=64.0, ram=256.0, gpu=0.0,
            labels={"tier": "standard", "disk": "ssd"},
            zone=ZONES[i % 3],
        ))

    # 40 Memory-Optimized nodes: 32C / 512G (was 40)
    for i in range(36):
        nodes.append(_make_node(
            name=f"mem-{i:03d}",
            cpu=32.0, ram=512.0, gpu=0.0,
            labels={"tier": "memory", "disk": "nvme"},
            zone=ZONES[i % 3],
        ))

    # 30 GPU nodes: 48C / 192G / 8GPU (was 20)
    for i in range(30):
        nodes.append(_make_node(
            name=f"gpu-{i:03d}",
            cpu=48.0, ram=192.0, gpu=8.0,
            labels={"tier": "gpu", "gpu": "nvidia-a100", "disk": "nvme"},
            zone=ZONES[i % 3],
        ))

    return {"nodes": nodes, "daemonsets": daemonsets}


def _make_node(
    name: str,
    cpu: float,
    ram: float,
    gpu: float,
    labels: dict[str, str],
    zone: str,
) -> dict:
    """Create a single node entry."""
    return {
        "name": name,
        "allocatable": {"cpu": cpu, "ram": ram, "gpu": gpu},
        "labels": {**labels, "node-role": "worker"},
        "taints": [],
        "zone": zone,
    }


def _build_daemonsets() -> list[dict]:
    """System DaemonSets consuming ~3-5% overhead per node."""
    return [
        {"name": "kube-proxy", "resources": {"cpu": 0.5, "ram": 0.5, "gpu": 0.0},
         "nodeSelector": {}, "tolerations": []},
        {"name": "calico-node", "resources": {"cpu": 1.0, "ram": 1.0, "gpu": 0.0},
         "nodeSelector": {}, "tolerations": []},
        {"name": "node-exporter", "resources": {"cpu": 0.25, "ram": 0.25, "gpu": 0.0},
         "nodeSelector": {}, "tolerations": []},
        {"name": "fluentd", "resources": {"cpu": 0.5, "ram": 1.0, "gpu": 0.0},
         "nodeSelector": {}, "tolerations": []},
    ]


# ─── Workload generation ─────────────────────────────────────────────

# WHY these exact numbers: we need total resource ~92% of cluster capacity
# after DaemonSet deduction.
#
# Cluster raw capacity:
#   Standard:  100 * 64C  = 6400C,  100 * 256G  = 25600G
#   Memory:     30 * 32C  =  960C,   30 * 512G  = 15360G
#   GPU:        20 * 48C  =  960C,   20 * 192G  =  3840G
#   Total:                  8320C                  44800G,  160 GPU
#
# DaemonSet overhead per node: 2.25C, 2.75G
#   Total overhead: 150 * 2.25 = 337.5C,  150 * 2.75 = 412.5G
#
# Net capacity: 7982.5C, 44387.5G, 160 GPU
# Target 92%:   7344C,   40836G,   147 GPU

# Service definitions: (name, namespace, replicas, cpu_per_pod, ram_per_pod,
#                        gpu_per_pod, node_selector, is_anti_affinity_spread)
SERVICES: list[tuple] = [
    # ── Tier 1: GPU workloads (~300 pods, 152 GPU units on 20 nodes) ───
    # 20 GPU nodes * 8 GPU = 160. Target 95% = 152 GPU.
    ("llm-inference",     "ai",        40, 6.0,  24.0, 1.0, {"gpu": "nvidia-a100"}, True),
    ("embedding-server",  "ai",        24, 5.0,  16.0, 1.0, {"gpu": "nvidia-a100"}, True),
    ("training-worker",   "ai",        16,12.0,  48.0, 4.0, {"gpu": "nvidia-a100"}, False),
    ("vision-pipeline",   "ai",        24, 4.0,  12.0, 1.0, {"gpu": "nvidia-a100"}, True),
    ("recommendation-ml", "ai",        80, 3.0,  10.0, 0.0, {}, True),
    ("feature-store",     "ai",       100, 2.0,   6.0, 0.0, {}, False),

    # ── Tier 2: Core platform (heavy CPU, anti-affinity spread) ────────
    ("api-gateway",       "platform", 100, 6.0,  12.0, 0.0, {},                     True),
    ("payment-service",   "fintech",  100, 4.0,   8.0, 0.0, {},                     True),
    ("order-service",     "commerce",  80, 4.0,  10.0, 0.0, {},                     True),
    ("inventory-service", "commerce",  60, 3.0,   6.0, 0.0, {},                     True),

    # ── Tier 3: Data layer (memory-heavy, nvme required) ──────────────
    ("postgres-primary",  "database",  30, 6.0,  96.0, 0.0, {"disk": "nvme"},       True),
    ("postgres-replica",  "database",  60, 3.0,  48.0, 0.0, {"disk": "nvme"},       False),
    ("redis-cache",       "cache",    100, 2.0,  24.0, 0.0, {},                     True),
    ("elasticsearch",     "search",    30, 6.0,  96.0, 0.0, {"disk": "nvme"},       True),
    ("kafka-broker",      "streaming", 30, 6.0,  48.0, 0.0, {"disk": "nvme"},       False),

    # ── Tier 4: Web frontend + BFF ─────────────────────────────────────
    ("web-frontend",      "frontend", 100, 3.0,   6.0, 0.0, {},                     True),
    ("mobile-bff",        "frontend",  80, 2.0,   4.0, 0.0, {},                     False),
    ("admin-dashboard",   "frontend",  30, 2.0,   4.0, 0.0, {},                     False),

    # ── Tier 5: Background workers ─────────────────────────────────────
    ("email-worker",      "workers",   80, 2.0,   4.0, 0.0, {},                     False),
    ("notification-svc",  "workers",   60, 2.0,   3.0, 0.0, {},                     False),
    ("report-generator",  "workers",   80, 3.0,   6.0, 0.0, {},                     False),
    ("image-processor",   "workers",   60, 4.0,   8.0, 0.0, {},                     False),
    ("pdf-renderer",      "workers",   40, 3.0,   6.0, 0.0, {},                     False),

    # ── Tier 6: Observability stack ────────────────────────────────────
    ("prometheus",        "monitoring", 30, 3.0,  12.0, 0.0, {},                    False),
    ("grafana",           "monitoring", 20, 2.0,   4.0, 0.0, {},                    False),
    ("jaeger-collector",  "monitoring", 30, 3.0,   6.0, 0.0, {},                    False),
    ("loki",              "monitoring", 30, 3.0,  12.0, 0.0, {},                    False),

    # ── Tier 7: Auth & Security ────────────────────────────────────────
    ("auth-service",      "security", 100, 3.0,   6.0, 0.0, {},                     True),
    ("vault",             "security",  20, 2.0,   4.0, 0.0, {},                     True),
    ("cert-manager",      "security",  20, 1.0,   2.0, 0.0, {},                     False),

    # ── Tier 8: Internal tools ─────────────────────────────────────────
    ("cronjob-scheduler", "internal",  40, 1.5,   3.0, 0.0, {},                     False),
    ("config-server",     "internal",  20, 1.0,   2.0, 0.0, {},                     False),
    ("service-mesh-proxy","internal", 100, 1.0,   2.0, 0.0, {},                     False),

    # ── Tier 9: Fillers (reach ~2700 pods, boost CPU fill to ~92%) ─────
    ("log-aggregator",    "logging",   80, 2.0,   6.0, 0.0, {},                     False),
    ("metrics-relay",     "logging",   60, 1.0,   2.0, 0.0, {},                     False),
    ("healthcheck-agent", "ops",      100, 1.0,   2.0, 0.0, {},                     False),
    ("dns-resolver",      "ops",       40, 1.0,   2.0, 0.0, {},                     False),
    ("rate-limiter",      "platform",  80, 2.0,   4.0, 0.0, {},                     False),
    ("session-store",     "platform",  80, 1.0,   6.0, 0.0, {},                     False),
    ("task-queue",        "workers",   60, 2.0,   4.0, 0.0, {},                     False),
    ("webhook-relay",     "platform",  40, 1.0,   2.0, 0.0, {},                     False),
    ("audit-logger",      "security",  40, 1.0,   2.0, 0.0, {},                     False),
    ("geo-service",       "platform",  40, 2.0,   4.0, 0.0, {},                     False),
    ("search-indexer",    "search",    40, 3.0,  12.0, 0.0, {"disk": "nvme"},       False),
    ("cdn-origin",        "frontend",  60, 2.0,   4.0, 0.0, {},                     False),
    ("ab-test-engine",    "platform",  20, 1.0,   2.0, 0.0, {},                     False),
    ("feature-flags",     "platform",  20, 1.0,   2.0, 0.0, {},                     False),
    ("user-profile-svc",  "platform",  80, 2.0,   4.0, 0.0, {},                     False),
    ("chat-service",      "comms",     60, 2.0,   4.0, 0.0, {},                     False),
]

# WHY: 5 affinity chains create cascading dependencies that force the GA
# to co-locate entire stacks — exponentially harder than pairwise affinity.
AFFINITY_CHAINS: list[list[str]] = [
    ["web-frontend", "api-gateway", "redis-cache"],
    ["order-service", "payment-service", "postgres-primary"],
    ["mobile-bff", "auth-service", "vault"],
    ["llm-inference", "embedding-server", "feature-store"],
    ["prometheus", "grafana", "loki"],
]

# WHY: anti-affinity pairs across tiers create cross-cutting constraints
# that prevent the GA from simply partitioning by tier.
ANTI_AFFINITY_CROSS: list[tuple[str, str]] = [
    ("postgres-primary", "elasticsearch"),  # both heavy — separate them
    ("kafka-broker", "postgres-primary"),   # I/O isolation
    ("training-worker", "llm-inference"),   # GPU bandwidth contention
    ("api-gateway", "image-processor"),     # CPU contention
    ("redis-cache", "kafka-broker"),        # memory contention
]


def generate_workloads() -> dict:
    """Build 3000-pod workload manifest with hostile constraint matrix."""
    pods: list[dict] = []
    pod_name_registry: dict[str, list[str]] = {}  # service -> [pod_names]

    for svc_def in SERVICES:
        svc_name, namespace, replicas = svc_def[0], svc_def[1], svc_def[2]
        cpu, ram, gpu = svc_def[3], svc_def[4], svc_def[5]
        node_selector, is_spread = svc_def[6], svc_def[7]

        svc_pod_names: list[str] = []

        for r in range(replicas):
            pod_name = f"{svc_name}-{r:04d}"
            svc_pod_names.append(pod_name)

            pod: dict = {
                "name": pod_name,
                "namespace": namespace,
                "requests": {"cpu": cpu, "ram": ram, "gpu": gpu},
            }

            if node_selector:
                pod["nodeSelector"] = node_selector

            pods.append(pod)

        pod_name_registry[svc_name] = svc_pod_names

    # WHY: apply constraints AFTER all pods are created, so we can
    # reference cross-service pod names for affinity targets.
    _apply_anti_affinity_spread(pods, pod_name_registry)
    _apply_affinity_chains(pods, pod_name_registry)
    _apply_cross_anti_affinity(pods, pod_name_registry)

    return {"pods": pods, "groups": []}


def _apply_anti_affinity_spread(
    pods: list[dict],
    registry: dict[str, list[str]],
) -> None:
    """For spread services, each pod anti-affines with all siblings.

    WHY: this forces the GA to spread pods across nodes — the most
    expensive constraint type because it creates O(n^2) pairwise checks.
    We limit to first 5 siblings to avoid YAML bloat while keeping
    the constraint pressure high.
    """
    spread_services = [s[0] for s in SERVICES if s[7]]

    for svc_name in spread_services:
        svc_pods = registry[svc_name]
        for pod in pods:
            if not pod["name"].startswith(svc_name + "-"):
                continue

            # WHY: pick up to 5 random siblings to anti-affine with.
            # Full O(n^2) would create 10000-entry lists for 100 replicas.
            siblings = [n for n in svc_pods if n != pod["name"]]
            targets = RNG.sample(siblings, min(5, len(siblings)))
            pod.setdefault("antiAffinity", []).extend(targets)


def _apply_affinity_chains(
    pods: list[dict],
    registry: dict[str, list[str]],
) -> None:
    """Link services in affinity chains: A[i] → B[i] → C[i].

    WHY: creates cascading co-location pressure. If A must be with B
    and B must be with C, then A-B-C must all land on the same node.
    This is transitively explosive for the constraint solver.
    """
    for chain in AFFINITY_CHAINS:
        min_replicas = min(len(registry[svc]) for svc in chain)
        link_count = min(min_replicas, 30)  # cap at 30 links per chain

        for i in range(link_count):
            for j in range(len(chain) - 1):
                src_svc = chain[j]
                dst_svc = chain[j + 1]
                src_pod_name = registry[src_svc][i]
                dst_pod_name = registry[dst_svc][i]

                # Find the actual pod dict and add affinity
                for pod in pods:
                    if pod["name"] == src_pod_name:
                        pod.setdefault("affinity", []).append(dst_pod_name)
                    elif pod["name"] == dst_pod_name:
                        pod.setdefault("affinity", []).append(src_pod_name)


def _apply_cross_anti_affinity(
    pods: list[dict],
    registry: dict[str, list[str]],
) -> None:
    """Cross-tier anti-affinity: pods from different services repel each other.

    WHY: prevents the GA from simply partitioning workloads by tier.
    A postgres-primary pod must not share a node with an elasticsearch pod.
    """
    for svc_a, svc_b in ANTI_AFFINITY_CROSS:
        pods_a = registry[svc_a]
        pods_b = registry[svc_b]
        link_count = min(len(pods_a), len(pods_b), 20)

        for i in range(link_count):
            for pod in pods:
                if pod["name"] == pods_a[i]:
                    pod.setdefault("antiAffinity", []).append(pods_b[i])
                elif pod["name"] == pods_b[i]:
                    pod.setdefault("antiAffinity", []).append(pods_a[i])


def compute_resource_summary(infra: dict, workloads: dict) -> None:
    """Print resource utilization analysis for verification."""
    # Node capacity
    total_cpu = sum(n["allocatable"]["cpu"] for n in infra["nodes"])
    total_ram = sum(n["allocatable"]["ram"] for n in infra["nodes"])
    total_gpu = sum(n["allocatable"]["gpu"] for n in infra["nodes"])

    # DaemonSet overhead
    ds_cpu = sum(d["resources"]["cpu"] for d in infra["daemonsets"])
    ds_ram = sum(d["resources"]["ram"] for d in infra["daemonsets"])
    overhead_cpu = ds_cpu * len(infra["nodes"])
    overhead_ram = ds_ram * len(infra["nodes"])

    net_cpu = total_cpu - overhead_cpu
    net_ram = total_ram - overhead_ram

    # Pod requests
    pod_cpu = sum(p["requests"]["cpu"] for p in workloads["pods"])
    pod_ram = sum(p["requests"]["ram"] for p in workloads["pods"])
    pod_gpu = sum(p["requests"]["gpu"] for p in workloads["pods"])

    fill_cpu = pod_cpu / net_cpu * 100
    fill_ram = pod_ram / net_ram * 100
    fill_gpu = pod_gpu / total_gpu * 100 if total_gpu > 0 else 0

    total_pods = len(workloads["pods"])
    total_nodes = len(infra["nodes"])

    anti_aff_count = sum(
        len(p.get("antiAffinity", [])) for p in workloads["pods"]
    )
    aff_count = sum(
        len(p.get("affinity", [])) for p in workloads["pods"]
    )

    print(f"\n{'═' * 60}")
    print(f"  MSC IRINA SCALE — Resource Summary")
    print(f"{'═' * 60}")
    print(f"  Nodes:          {total_nodes}")
    print(f"  Pods:           {total_pods}")
    print(f"  Constraints:    {anti_aff_count} anti-affinity + {aff_count} affinity")
    print(f"{'─' * 60}")
    print(f"  Cluster CPU:    {total_cpu:.0f} cores (net: {net_cpu:.1f})")
    print(f"  Cluster RAM:    {total_ram:.0f} GiB  (net: {net_ram:.1f})")
    print(f"  Cluster GPU:    {total_gpu:.0f} units")
    print(f"{'─' * 60}")
    print(f"  Pod CPU demand: {pod_cpu:.0f} cores  ({fill_cpu:.1f}% fill)")
    print(f"  Pod RAM demand: {pod_ram:.0f} GiB   ({fill_ram:.1f}% fill)")
    print(f"  Pod GPU demand: {pod_gpu:.0f} units  ({fill_gpu:.1f}% fill)")
    print(f"{'═' * 60}\n")


def main() -> None:
    """Generate and write the MSC Irina test data files."""
    infra = generate_infra()
    workloads = generate_workloads()

    compute_resource_summary(infra, workloads)

    infra_path = "testdata/irina_infra.yaml"
    workloads_path = "testdata/irina_workloads.yaml"

    with open(infra_path, "w") as f:
        f.write("# MSC Irina Scale — 150-node datacenter topology\n")
        f.write("# Generated by gen_irina_testdata.py\n")
        f.write(f"# 100 Standard + 30 Memory + 20 GPU nodes\n\n")
        yaml.dump(infra, f, default_flow_style=False, sort_keys=False,
                  allow_unicode=True, width=120)

    with open(workloads_path, "w") as f:
        f.write("# MSC Irina Scale — 3000-pod workload manifest\n")
        f.write("# Generated by gen_irina_testdata.py\n")
        f.write(f"# 50 microservices with hostile constraint matrix\n\n")
        yaml.dump(workloads, f, default_flow_style=False, sort_keys=False,
                  allow_unicode=True, width=120)

    print(f"  Written: {infra_path}")
    print(f"  Written: {workloads_path}")


if __name__ == "__main__":
    main()
