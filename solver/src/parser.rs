//! YAML parser for Kuberina IR v0.2.0 format.
//!
//! Parses `_infra.yaml` (nodes + daemonsets) and `_workloads.yaml`
//! (namespace-grouped pods with replicas, K8s units, gang scheduling).
//!
//! Key transformations:
//! 1. Namespace flattening: `namespaces.monitoring[0]` → `Pod { namespace: "monitoring" }`
//! 2. Replica unrolling: `replicas: 3` → 3 independent pods (grafana-0000, -0001, -0002)
//! 3. Gang auto-grouping: inline `gang: "name"` → auto-generated PodGroup
//! 4. K8s unit parsing: "512Mi" → 0.5 GiB via quantity.rs

use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

use crate::model::{DaemonSet, Node, Pod, PodGroup, ResourceVector, TopologySpread};
use crate::quantity::{QuantityContext, parse_quantity};

// ─── Raw deserialization structs (YAML shape) ───────────────────────────

/// Accepts both `f64` and `String` from YAML, e.g. `cpu: 0.3` or `ram: "512Mi"`.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum ResourceValue {
    Numeric(f64),
    Text(String),
}

#[derive(Debug, Clone, Deserialize, Default)]
struct RawDiskIO {
    read: Option<ResourceValue>,
    write: Option<ResourceValue>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct RawNetworkIO {
    #[serde(rename = "in")]
    in_: Option<ResourceValue>,
    out: Option<ResourceValue>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct RawResources {
    cpu: Option<ResourceValue>,
    ram: Option<ResourceValue>,
    gpu: Option<ResourceValue>,
    storage: Option<ResourceValue>,
    disk: Option<RawDiskIO>,
    network: Option<RawNetworkIO>,
}

// ─── Infra raw structs ──────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct RawInfraFile {
    #[serde(default)]
    nodes: Vec<RawNode>,
    #[serde(default)]
    daemonsets: Vec<RawDaemonSet>,
}

#[derive(Debug, Deserialize)]
struct RawNode {
    name: String,
    #[serde(default)]
    zone: String,
    #[serde(default)]
    rack: String,
    #[serde(default)]
    labels: HashMap<String, String>,
    #[serde(default)]
    taints: Vec<RawTaint>,
    #[serde(default)]
    allocatable: RawResources,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct RawTaint {
    key: String,
    #[serde(default)]
    value: String,
    #[serde(default)]
    effect: String,
}

#[derive(Debug, Deserialize)]
struct RawDaemonSet {
    name: String,
    #[serde(default)]
    resources: RawResources,
    #[serde(default, rename = "nodeSelector")]
    node_selector: HashMap<String, String>,
    #[serde(default)]
    tolerations: Vec<String>,
}

// ─── Workload raw structs ───────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct RawWorkloadFile {
    #[serde(default)]
    namespaces: HashMap<String, Vec<RawPod>>,
}

#[derive(Debug, Deserialize)]
struct RawPod {
    name: String,
    #[serde(default = "default_replicas")]
    replicas: usize,
    #[serde(default)]
    gang: Option<String>,
    #[serde(default)]
    requests: RawResources,
    #[serde(default, rename = "nodeSelector")]
    node_selector: HashMap<String, String>,
    #[serde(default)]
    tolerations: Vec<RawToleration>,
    #[serde(default)]
    affinity: Vec<String>,
    #[serde(default, rename = "antiAffinity")]
    anti_affinity: Vec<String>,
    #[serde(default, rename = "topologySpread")]
    topology_spread: Option<TopologySpread>,
}

fn default_replicas() -> usize {
    1
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct RawToleration {
    key: String,
    #[serde(default)]
    operator: String,
    #[serde(default)]
    value: String,
    #[serde(default)]
    effect: String,
}

// ─── Conversion: Raw → Domain ───────────────────────────────────────────

/// Parse a ResourceValue with appropriate context.
fn resolve_value(
    val: &Option<ResourceValue>,
    ctx: QuantityContext,
    field_name: &str,
) -> Result<f64, String> {
    match val {
        None => Ok(0.0),
        Some(ResourceValue::Numeric(n)) => Ok(*n),
        Some(ResourceValue::Text(s)) => {
            parse_quantity(s, ctx).map_err(|e| format!("field '{field_name}': {e}"))
        }
    }
}

/// Parse a node's ResourceValue; missing dims default to f64::MAX (unconstrained).
fn resolve_node_value(
    val: &Option<ResourceValue>,
    ctx: QuantityContext,
    field_name: &str,
) -> Result<f64, String> {
    match val {
        None => Ok(f64::MAX),
        Some(ResourceValue::Numeric(n)) => Ok(*n),
        Some(ResourceValue::Text(s)) => {
            parse_quantity(s, ctx).map_err(|e| format!("field '{field_name}': {e}"))
        }
    }
}

/// Convert RawResources to domain ResourceVector for pods/daemonsets (default 0).
fn raw_to_pod_resources(raw: &RawResources) -> Result<ResourceVector, String> {
    let disk = raw.disk.as_ref();
    let net = raw.network.as_ref();

    Ok(ResourceVector::new_8d(
        resolve_value(&raw.cpu, QuantityContext::Cpu, "cpu")?,
        resolve_value(&raw.ram, QuantityContext::Memory, "ram")?,
        resolve_value(&raw.gpu, QuantityContext::Cpu, "gpu")?,
        resolve_value(&raw.storage, QuantityContext::Memory, "storage")?,
        resolve_value(
            &disk.and_then(|d| d.read.clone()),
            QuantityContext::Throughput,
            "disk.read",
        )?,
        resolve_value(
            &disk.and_then(|d| d.write.clone()),
            QuantityContext::Throughput,
            "disk.write",
        )?,
        resolve_value(
            &net.and_then(|n| n.in_.clone()),
            QuantityContext::Throughput,
            "network.in",
        )?,
        resolve_value(
            &net.and_then(|n| n.out.clone()),
            QuantityContext::Throughput,
            "network.out",
        )?,
    ))
}

/// Convert RawResources to domain ResourceVector for nodes (default f64::MAX).
fn raw_to_node_resources(raw: &RawResources) -> Result<ResourceVector, String> {
    let disk = raw.disk.as_ref();
    let net = raw.network.as_ref();

    // WHY: CPU/RAM/GPU default to 0 for nodes too (explicit is better),
    // but I/O dimensions default to f64::MAX so homelab users can omit them.
    Ok(ResourceVector::new_8d(
        resolve_value(&raw.cpu, QuantityContext::Cpu, "cpu")?,
        resolve_value(&raw.ram, QuantityContext::Memory, "ram")?,
        resolve_value(&raw.gpu, QuantityContext::Cpu, "gpu")?,
        resolve_node_value(&raw.storage, QuantityContext::Memory, "storage")?,
        resolve_node_value(
            &disk.and_then(|d| d.read.clone()),
            QuantityContext::Throughput,
            "disk.read",
        )?,
        resolve_node_value(
            &disk.and_then(|d| d.write.clone()),
            QuantityContext::Throughput,
            "disk.write",
        )?,
        resolve_node_value(
            &net.and_then(|n| n.in_.clone()),
            QuantityContext::Throughput,
            "network.in",
        )?,
        resolve_node_value(
            &net.and_then(|n| n.out.clone()),
            QuantityContext::Throughput,
            "network.out",
        )?,
    ))
}

/// Flatten RawTaint to a simple key string for CSP matching.
/// WHY: Existing taint/toleration model uses string comparison.
/// Structured taints from IR v0.2.0 are flattened to preserve this.
fn flatten_taint(taint: &RawTaint) -> String {
    taint.key.clone()
}

/// Flatten RawToleration to a simple key string.
fn flatten_toleration(tol: &RawToleration) -> String {
    tol.key.clone()
}

// ─── Public API ─────────────────────────────────────────────────────────

/// Parse cluster topology YAML (IR v0.2.0) into Node and DaemonSet lists.
///
/// ```no_run
/// # use kuberina_solver::parser::load_infra;
/// let (nodes, ds) = load_infra("testdata/homelab_infra.yaml").unwrap();
/// assert!(!nodes.is_empty());
/// ```
pub fn load_infra(path: impl AsRef<Path>) -> Result<(Vec<Node>, Vec<DaemonSet>), String> {
    let content = std::fs::read_to_string(path.as_ref())
        .map_err(|e| format!("cannot read {}: {e}", path.as_ref().display()))?;
    let raw: RawInfraFile = serde_yaml::from_str(&content)
        .map_err(|e| format!("YAML parse error in {}: {e}", path.as_ref().display()))?;

    let nodes = convert_nodes(&raw.nodes)?;
    let daemonsets = convert_daemonsets(&raw.daemonsets)?;
    Ok((nodes, daemonsets))
}

fn convert_nodes(raw_nodes: &[RawNode]) -> Result<Vec<Node>, String> {
    raw_nodes
        .iter()
        .map(|rn| {
            Ok(Node {
                name: rn.name.clone(),
                allocatable: raw_to_node_resources(&rn.allocatable)?,
                labels: rn.labels.clone(),
                taints: rn.taints.iter().map(flatten_taint).collect(),
                zone: rn.zone.clone(),
                rack: rn.rack.clone(),
            })
        })
        .collect()
}

fn convert_daemonsets(raw_ds: &[RawDaemonSet]) -> Result<Vec<DaemonSet>, String> {
    raw_ds
        .iter()
        .map(|rd| {
            Ok(DaemonSet {
                name: rd.name.clone(),
                resources: raw_to_pod_resources(&rd.resources)?,
                node_selector: rd.node_selector.clone(),
                tolerations: rd.tolerations.clone(),
            })
        })
        .collect()
}

/// Parse workload manifests YAML (IR v0.2.0) into Pod and PodGroup lists.
///
/// Performs namespace flattening, replica unrolling, and gang auto-grouping.
///
/// ```no_run
/// # use kuberina_solver::parser::load_workloads;
/// let (pods, groups) = load_workloads("testdata/homelab_workloads.yaml").unwrap();
/// assert!(!pods.is_empty());
/// ```
pub fn load_workloads(path: impl AsRef<Path>) -> Result<(Vec<Pod>, Vec<PodGroup>), String> {
    let content = std::fs::read_to_string(path.as_ref())
        .map_err(|e| format!("cannot read {}: {e}", path.as_ref().display()))?;
    let raw: RawWorkloadFile = serde_yaml::from_str(&content)
        .map_err(|e| format!("YAML parse error in {}: {e}", path.as_ref().display()))?;

    let pods = flatten_namespaces(&raw)?;
    let groups = auto_group_gangs(&pods);
    Ok((pods, groups))
}

/// Flatten namespace dict → flat pod list, unrolling replicas.
fn flatten_namespaces(raw: &RawWorkloadFile) -> Result<Vec<Pod>, String> {
    let mut pods = Vec::new();

    // WHY: sort namespace keys for deterministic pod ordering across runs.
    let mut ns_keys: Vec<&String> = raw.namespaces.keys().collect();
    ns_keys.sort();

    for ns_name in ns_keys {
        let raw_pods = &raw.namespaces[ns_name];
        for rp in raw_pods {
            let resources = raw_to_pod_resources(&rp.requests)?;
            let tolerations: Vec<String> = rp.tolerations.iter().map(flatten_toleration).collect();

            let replica_count = rp.replicas.max(1);
            for replica_idx in 0..replica_count {
                let pod_name = format_replica_name(&rp.name, replica_idx, replica_count);
                pods.push(Pod {
                    name: pod_name,
                    namespace: ns_name.clone(),
                    requests: resources,
                    tolerations: tolerations.clone(),
                    node_selector: rp.node_selector.clone(),
                    affinity_targets: rp.affinity.clone(),
                    anti_affinity_targets: rp.anti_affinity.clone(),
                    group_name: rp.gang.clone().unwrap_or_default(),
                    topology_spread: rp.topology_spread.clone(),
                });
            }
        }
    }
    Ok(pods)
}

/// Format replica name: "grafana" with 3 replicas → "grafana-0000", "grafana-0001", etc.
/// Single replica (replicas=1) → just "grafana-0" (no padding).
fn format_replica_name(base: &str, idx: usize, total: usize) -> String {
    if total <= 1 {
        format!("{base}-0")
    } else {
        format!("{base}-{idx:04}")
    }
}

/// Scan all pods for unique `gang` values and auto-generate PodGroup list.
///
/// ```
/// # use kuberina_solver::model::*;
/// // Two pods with gang = "training" → one PodGroup with min_members = 2
/// ```
fn auto_group_gangs(pods: &[Pod]) -> Vec<PodGroup> {
    let mut gang_map: HashMap<String, Vec<usize>> = HashMap::new();

    for (idx, pod) in pods.iter().enumerate() {
        if !pod.group_name.is_empty() {
            gang_map
                .entry(pod.group_name.clone())
                .or_default()
                .push(idx);
        }
    }

    let mut groups: Vec<PodGroup> = gang_map
        .into_iter()
        .map(|(name, indices)| {
            let min_members = indices.len();
            PodGroup {
                name,
                pod_indices: indices,
                min_members,
                node_selector: HashMap::new(),
                colocate: false,
            }
        })
        .collect();

    // WHY: sort for deterministic output ordering
    groups.sort_by(|a, b| a.name.cmp(&b.name));
    groups
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_homelab_infra() {
        let (nodes, ds) = load_infra("testdata/homelab_infra.yaml").unwrap();
        assert_eq!(nodes.len(), 3);
        assert_eq!(ds.len(), 3);
        assert_eq!(nodes[0].name, "thinkcentre-alpha");
        // "4.0" bare float → 4.0 cores
        assert!((nodes[0].allocatable.cpu - 4.0).abs() < 1e-9);
    }

    #[test]
    fn load_homelab_workloads() {
        let (pods, groups) = load_workloads("testdata/homelab_workloads.yaml").unwrap();
        assert_eq!(pods.len(), 10);
        assert!(groups.is_empty());
        // Pods are sorted by namespace, then by order in YAML
        // First pod should be from "cloud" namespace (alphabetical)
        assert_eq!(pods[0].namespace, "cloud");
    }

    #[test]
    fn missing_file_returns_error() {
        let result = load_infra("nonexistent.yaml");
        assert!(result.is_err());
    }

    #[test]
    fn replica_unrolling() {
        let yaml = r#"
namespaces:
  ai:
    - name: llm-worker
      replicas: 3
      requests:
        cpu: 2.0
        ram: "8Gi"
"#;
        let raw: RawWorkloadFile = serde_yaml::from_str(yaml).unwrap();
        let pods = flatten_namespaces(&raw).unwrap();
        assert_eq!(pods.len(), 3);
        assert_eq!(pods[0].name, "llm-worker-0000");
        assert_eq!(pods[1].name, "llm-worker-0001");
        assert_eq!(pods[2].name, "llm-worker-0002");
        assert_eq!(pods[0].namespace, "ai");
        assert!((pods[0].requests.ram - 8.0).abs() < 1e-9);
    }

    #[test]
    fn gang_auto_grouping() {
        let yaml = r#"
namespaces:
  ai:
    - name: trainer
      replicas: 4
      gang: "training-job"
      requests:
        cpu: 8.0
        gpu: 1
"#;
        let raw: RawWorkloadFile = serde_yaml::from_str(yaml).unwrap();
        let pods = flatten_namespaces(&raw).unwrap();
        let groups = auto_group_gangs(&pods);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].name, "training-job");
        assert_eq!(groups[0].min_members, 4);
        assert_eq!(groups[0].pod_indices.len(), 4);
    }

    #[test]
    fn namespace_flattening() {
        let yaml = r#"
namespaces:
  monitoring:
    - name: grafana
      replicas: 1
      requests:
        cpu: 0.3
  database:
    - name: postgres
      replicas: 1
      requests:
        cpu: 1.0
"#;
        let raw: RawWorkloadFile = serde_yaml::from_str(yaml).unwrap();
        let pods = flatten_namespaces(&raw).unwrap();
        assert_eq!(pods.len(), 2);
        // "database" comes before "monitoring" alphabetically
        assert_eq!(pods[0].namespace, "database");
        assert_eq!(pods[0].name, "postgres-0");
        assert_eq!(pods[1].namespace, "monitoring");
        assert_eq!(pods[1].name, "grafana-0");
    }

    #[test]
    fn k8s_units_in_resources() {
        let yaml = r#"
namespaces:
  test:
    - name: heavy-io
      replicas: 1
      requests:
        cpu: "300m"
        ram: "512Mi"
        storage: "10Gi"
        disk:
          read: "500M"
          write: "100M"
        network:
          in: "1G"
          out: "500M"
"#;
        let raw: RawWorkloadFile = serde_yaml::from_str(yaml).unwrap();
        let pods = flatten_namespaces(&raw).unwrap();
        let r = &pods[0].requests;
        assert!((r.cpu - 0.3).abs() < 1e-9);
        assert!((r.ram - 0.5).abs() < 1e-9);
        assert!((r.storage - 10.0).abs() < 1e-9);
        assert!((r.disk_read - 500.0).abs() < 1e-9);
        assert!((r.disk_write - 100.0).abs() < 1e-9);
        assert!((r.net_in - 1000.0).abs() < 1e-9);
        assert!((r.net_out - 500.0).abs() < 1e-9);
    }

    #[test]
    fn topology_spread_parsed() {
        let yaml = r#"
namespaces:
  ai:
    - name: inference
      replicas: 1
      requests:
        cpu: 1.0
      topologySpread:
        maxSkew: 2
        topologyKey: zone
"#;
        let raw: RawWorkloadFile = serde_yaml::from_str(yaml).unwrap();
        let pods = flatten_namespaces(&raw).unwrap();
        let ts = pods[0].topology_spread.as_ref().unwrap();
        assert_eq!(ts.max_skew, 2);
        assert_eq!(ts.topology_key, "zone");
    }

    #[test]
    fn single_replica_naming() {
        assert_eq!(format_replica_name("grafana", 0, 1), "grafana-0");
    }

    #[test]
    fn multi_replica_naming() {
        assert_eq!(format_replica_name("worker", 0, 18), "worker-0000");
        assert_eq!(format_replica_name("worker", 17, 18), "worker-0017");
    }

    #[test]
    fn node_unconstrained_defaults() {
        let yaml = r#"
nodes:
  - name: simple-node
    allocatable:
      cpu: 4.0
      ram: "16Gi"
"#;
        let raw: RawInfraFile = serde_yaml::from_str(yaml).unwrap();
        let nodes = convert_nodes(&raw.nodes).unwrap();
        assert!((nodes[0].allocatable.cpu - 4.0).abs() < 1e-9);
        assert!((nodes[0].allocatable.ram - 16.0).abs() < 1e-9);
        // Unspecified I/O dims → f64::MAX (unconstrained)
        assert_eq!(nodes[0].allocatable.disk_read, f64::MAX);
        assert_eq!(nodes[0].allocatable.net_in, f64::MAX);
    }
}
