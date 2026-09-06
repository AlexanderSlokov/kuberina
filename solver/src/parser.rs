//! YAML parser for Kuberina IR v1.
//!
//! Parses `_infra.yaml` (nodes + daemonsets) and `_workloads.yaml`
//! (namespace-grouped pods with replicas, K8s units, gang scheduling).
//! The normative schema is `docs/references/ir-v1.md`; where this file and that
//! document disagree, the document is right.
//!
//! Key transformations:
//! 1. Namespace flattening: `namespaces.monitoring[0]` → `Pod { namespace: "monitoring" }`
//! 2. Replica unrolling: `replicas: 3` → 3 independent pods (grafana-0000, -0001, -0002)
//! 3. Gang grouping: an explicit `groups:` block, or inline `gang: "name"` shorthand
//! 4. K8s unit parsing: "512Mi" → 0.5 GiB via quantity.rs

use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

use crate::model::{DaemonSet, Node, ObservedUsage, Pod, PodGroup, ResourceVector, TopologySpread};
use crate::quantity::{QuantityContext, parse_quantity};

/// The IR version this binary implements. See ADR-0002.
pub const IR_API_VERSION: &str = "kuberina.io/v1";

/// `kind` of an infrastructure document.
pub const KIND_INFRASTRUCTURE: &str = "Infrastructure";
/// `kind` of a workload document.
pub const KIND_WORKLOADS: &str = "Workloads";
/// `kind` of a blueprint document, written by the solver and read by the forge.
pub const KIND_BLUEPRINT: &str = "Blueprint";

// ─── Raw deserialization structs (YAML shape) ───────────────────────────

/// Accepts both `f64` and `String` from YAML, e.g. `cpu: 0.3` or `ram: "512Mi"`.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum ResourceValue {
    Numeric(f64),
    Text(String),
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct RawDiskIO {
    read: Option<ResourceValue>,
    write: Option<ResourceValue>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct RawNetworkIO {
    #[serde(rename = "in")]
    in_: Option<ResourceValue>,
    out: Option<ResourceValue>,
}

/// WHY `deny_unknown_fields`: the I/O dimensions nest under `disk` and `network`.
/// A file writing them flat as `disk_read`/`net_in` used to deserialize cleanly —
/// serde dropped the unknown keys — leaving pod demand at 0 and node capacity at
/// f64::MAX, so the solver planned in four dimensions while reporting eight. That
/// shipped in the MSC Irina testdata and was caught only by the inspector. Schema
/// drift must fail at parse time, not become a silently smaller problem.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
struct RawInfraFile {
    #[serde(default, rename = "apiVersion")]
    api_version: Option<String>,
    #[serde(default)]
    kind: Option<String>,
    #[serde(default)]
    nodes: Vec<RawNode>,
    #[serde(default)]
    daemonsets: Vec<RawDaemonSet>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
#[allow(dead_code)]
struct RawTaint {
    key: String,
    #[serde(default)]
    value: String,
    #[serde(default)]
    effect: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
struct RawWorkloadFile {
    #[serde(default, rename = "apiVersion")]
    api_version: Option<String>,
    #[serde(default)]
    kind: Option<String>,
    #[serde(default)]
    namespaces: HashMap<String, Vec<RawPod>>,
    #[serde(default)]
    groups: Vec<RawPodGroup>,
}

/// An explicitly declared gang, as opposed to the inline `gang:` shorthand.
///
/// WHY it exists (#9): `min_members` and `colocate` are branched on by the CSP layer
/// and the fitness function, but `auto_group_gangs` hardcodes both, so partial gang
/// admission and forced co-location had no reachable input.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPodGroup {
    name: String,
    /// Pod declarations to enrol, as `namespace/name`. Every replica joins.
    members: Vec<String>,
    /// Smallest admissible gang. Defaults to every resolved member.
    #[serde(default)]
    min_members: Option<usize>,
    #[serde(default)]
    colocate: bool,
    #[serde(default, rename = "nodeSelector")]
    node_selector: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct RawObservedUsage {
    #[serde(default)]
    window: String,
    #[serde(default)]
    p50: Option<RawResources>,
    #[serde(default)]
    p95: Option<RawResources>,
    #[serde(default)]
    p99: Option<RawResources>,
    #[serde(default)]
    peak: Option<RawResources>,
    #[serde(default)]
    exceeded_request_fraction: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
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
    #[serde(default)]
    observed: Option<RawObservedUsage>,
}

fn default_replicas() -> usize {
    1
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
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

// ─── Document header ────────────────────────────────────────────────────

/// Reject a document whose `apiVersion` or `kind` is not the one we implement.
///
/// Both fields are optional for files written before IR v1 was declared, and an
/// absent field means "v1" (ADR-0002 clause 1). A *present* but unknown value is an
/// error rather than a warning: guessing at an unrecognized schema is how the solver
/// once planned in four dimensions while reporting eight.
fn check_document_header(
    api_version: &Option<String>,
    kind: &Option<String>,
    expected_kind: &str,
    source: &str,
) -> Result<(), String> {
    if let Some(v) = api_version.as_deref().filter(|v| *v != IR_API_VERSION) {
        return Err(format!(
            "{source}: unsupported apiVersion '{v}', this binary implements '{IR_API_VERSION}'"
        ));
    }
    if let Some(k) = kind.as_deref().filter(|k| *k != expected_kind) {
        return Err(format!(
            "{source}: expected kind '{expected_kind}', found '{k}'"
        ));
    }
    Ok(())
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

    check_document_header(
        &raw.api_version,
        &raw.kind,
        KIND_INFRASTRUCTURE,
        &path.as_ref().display().to_string(),
    )?;

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

    check_document_header(
        &raw.api_version,
        &raw.kind,
        KIND_WORKLOADS,
        &path.as_ref().display().to_string(),
    )?;

    let (mut pods, declarations) = flatten_namespaces(&raw)?;
    let declared = resolve_declared_groups(&raw.groups, &declarations, &mut pods)?;
    let groups = merge_groups(declared, &pods);
    Ok((pods, groups))
}

/// Flatten namespace dict → flat pod list, unrolling replicas.
///
/// Also returns, for each `namespace/name` declaration, the indices of the replicas
/// it produced. An explicit `groups:` block names declarations, not unrolled replica
/// names, so resolving membership needs that map.
type DeclarationIndex = HashMap<String, Vec<usize>>;

fn flatten_namespaces(raw: &RawWorkloadFile) -> Result<(Vec<Pod>, DeclarationIndex), String> {
    let mut pods = Vec::new();
    let mut declarations: DeclarationIndex = HashMap::new();

    // WHY: sort namespace keys for deterministic pod ordering across runs.
    let mut ns_keys: Vec<&String> = raw.namespaces.keys().collect();
    ns_keys.sort();

    for ns_name in ns_keys {
        let raw_pods = &raw.namespaces[ns_name];
        for rp in raw_pods {
            let resources = raw_to_pod_resources(&rp.requests)?;
            let tolerations: Vec<String> = rp.tolerations.iter().map(flatten_toleration).collect();

            let observed = rp.observed.as_ref().map(raw_to_observed).transpose()?;

            let replica_count = rp.replicas.max(1);
            let slot = declarations
                .entry(format!("{ns_name}/{}", rp.name))
                .or_default();
            for replica_idx in 0..replica_count {
                let pod_name = format_replica_name(&rp.name, replica_idx, replica_count);
                slot.push(pods.len());
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
                    observed: observed.clone(),
                });
            }
        }
    }
    Ok((pods, declarations))
}

/// Convert the raw `observed` block, parsing every percentile as a pod-side vector.
fn raw_to_observed(raw: &RawObservedUsage) -> Result<ObservedUsage, String> {
    Ok(ObservedUsage {
        window: raw.window.clone(),
        p50: optional_vector(&raw.p50)?,
        p95: optional_vector(&raw.p95)?,
        p99: optional_vector(&raw.p99)?,
        peak: optional_vector(&raw.peak)?,
        exceeded_request_fraction: raw.exceeded_request_fraction,
    })
}

fn optional_vector(raw: &Option<RawResources>) -> Result<Option<ResourceVector>, String> {
    raw.as_ref().map(raw_to_pod_resources).transpose()
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

/// Turn `groups:` entries into domain `PodGroup`s and stamp membership onto pods.
///
/// The inline `gang:` shorthand keeps working alongside this; see `merge_groups`.
fn resolve_declared_groups(
    raw_groups: &[RawPodGroup],
    declarations: &DeclarationIndex,
    pods: &mut [Pod],
) -> Result<Vec<PodGroup>, String> {
    let mut groups = Vec::with_capacity(raw_groups.len());
    for rg in raw_groups {
        let pod_indices = resolve_members(rg, declarations)?;
        let min_members = resolve_min_members(rg, pod_indices.len())?;
        for &idx in &pod_indices {
            claim_pod(&mut pods[idx], &rg.name)?;
        }
        groups.push(PodGroup {
            name: rg.name.clone(),
            pod_indices,
            min_members,
            node_selector: rg.node_selector.clone(),
            colocate: rg.colocate,
        });
    }
    Ok(groups)
}

/// Resolve `members` entries to pod indices. Every replica of a declaration joins.
fn resolve_members(
    rg: &RawPodGroup,
    declarations: &DeclarationIndex,
) -> Result<Vec<usize>, String> {
    let mut indices = Vec::new();
    for member in &rg.members {
        let found = declarations.get(member).ok_or_else(|| {
            format!(
                "group '{}': member '{member}' matches no pod declaration; expected 'namespace/name'",
                rg.name
            )
        })?;
        indices.extend(found);
    }
    if indices.is_empty() {
        return Err(format!("group '{}': resolved to no pods", rg.name));
    }
    indices.sort_unstable();
    indices.dedup();
    Ok(indices)
}

/// Default `min_members` to the whole gang, and reject a value it cannot admit.
fn resolve_min_members(rg: &RawPodGroup, member_count: usize) -> Result<usize, String> {
    let min = rg.min_members.unwrap_or(member_count);
    if min == 0 || min > member_count {
        return Err(format!(
            "group '{}': min_members is {min}, expected 1..={member_count}",
            rg.name
        ));
    }
    Ok(min)
}

/// Record group membership on a pod, rejecting a pod claimed by two groups.
fn claim_pod(pod: &mut Pod, group: &str) -> Result<(), String> {
    if !pod.group_name.is_empty() && pod.group_name != group {
        return Err(format!(
            "pod '{}/{}' is claimed by group '{}' and group '{group}'; a pod belongs to at most one group",
            pod.namespace, pod.name, pod.group_name
        ));
    }
    pod.group_name = group.to_string();
    Ok(())
}

/// Combine declared groups with those implied by the inline `gang:` shorthand.
///
/// A name declared in `groups:` wins: `resolve_declared_groups` has already stamped
/// those pods, so auto-grouping would otherwise rebuild the same gang with the
/// shorthand's defaults and overwrite `min_members` and `colocate`.
fn merge_groups(declared: Vec<PodGroup>, pods: &[Pod]) -> Vec<PodGroup> {
    let claimed: Vec<String> = declared.iter().map(|g| g.name.clone()).collect();
    let mut groups = declared;
    groups.extend(
        auto_group_gangs(pods)
            .into_iter()
            .filter(|g| !claimed.contains(&g.name)),
    );
    groups.sort_by(|a, b| a.name.cmp(&b.name));
    groups
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
        let (pods, _) = flatten_namespaces(&raw).unwrap();
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
        let (pods, _) = flatten_namespaces(&raw).unwrap();
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
        let (pods, _) = flatten_namespaces(&raw).unwrap();
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
        let (pods, _) = flatten_namespaces(&raw).unwrap();
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
        let (pods, _) = flatten_namespaces(&raw).unwrap();
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

    // ─── IR v1 surface (ADR-0002) ───────────────────────────────────────

    #[test]
    fn header_accepts_v1_and_an_absent_header() {
        assert!(check_document_header(&None, &None, KIND_WORKLOADS, "t").is_ok());
        assert!(
            check_document_header(
                &Some(IR_API_VERSION.into()),
                &Some(KIND_WORKLOADS.into()),
                KIND_WORKLOADS,
                "t",
            )
            .is_ok()
        );
    }

    #[test]
    fn header_rejects_unknown_version_and_wrong_kind() {
        let bad_version = check_document_header(
            &Some("kuberina.io/v2".into()),
            &None,
            KIND_WORKLOADS,
            "f.yaml",
        )
        .unwrap_err();
        assert!(bad_version.contains("kuberina.io/v2"), "{bad_version}");
        assert!(bad_version.contains(IR_API_VERSION), "{bad_version}");

        let bad_kind = check_document_header(
            &None,
            &Some("Infrastructure".into()),
            KIND_WORKLOADS,
            "f.yaml",
        )
        .unwrap_err();
        assert!(bad_kind.contains("Workloads"), "{bad_kind}");
    }

    #[test]
    fn unknown_field_is_a_parse_error() {
        // The eight-dimension defect in miniature: a stale key must not be dropped.
        let yaml = "nodes:\n  - name: n0\n    allocatable:\n      disk_read: 500\n";
        let err = serde_yaml::from_str::<RawInfraFile>(yaml).unwrap_err();
        assert!(err.to_string().contains("disk_read"), "{err}");
    }

    #[test]
    fn declared_group_carries_min_members_and_colocate() {
        let yaml = r#"
namespaces:
  ai:
    - name: trainer
      replicas: 4
      requests: { cpu: 8.0, gpu: 1 }
groups:
  - name: training-job
    members: ["ai/trainer"]
    min_members: 3
    colocate: true
    nodeSelector: { accelerator: "h100" }
"#;
        let raw: RawWorkloadFile = serde_yaml::from_str(yaml).unwrap();
        let (mut pods, decls) = flatten_namespaces(&raw).unwrap();
        let groups = resolve_declared_groups(&raw.groups, &decls, &mut pods).unwrap();

        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].pod_indices.len(), 4);
        assert_eq!(
            groups[0].min_members, 3,
            "partial admission must be reachable"
        );
        assert!(groups[0].colocate, "forced co-location must be reachable");
        assert_eq!(groups[0].node_selector["accelerator"], "h100");
        assert!(pods.iter().all(|p| p.group_name == "training-job"));
    }

    #[test]
    fn declared_group_defaults_min_members_to_the_whole_gang() {
        let yaml = r#"
namespaces:
  ai:
    - name: trainer
      replicas: 2
groups:
  - name: g
    members: ["ai/trainer"]
"#;
        let raw: RawWorkloadFile = serde_yaml::from_str(yaml).unwrap();
        let (mut pods, decls) = flatten_namespaces(&raw).unwrap();
        let groups = resolve_declared_groups(&raw.groups, &decls, &mut pods).unwrap();
        assert_eq!(groups[0].min_members, 2);
    }

    #[test]
    fn declared_group_rejects_bad_members_and_bad_min_members() {
        let missing = r#"
namespaces:
  ai:
    - name: trainer
groups:
  - name: g
    members: ["ai/typo"]
"#;
        let raw: RawWorkloadFile = serde_yaml::from_str(missing).unwrap();
        let (mut pods, decls) = flatten_namespaces(&raw).unwrap();
        let err = resolve_declared_groups(&raw.groups, &decls, &mut pods).unwrap_err();
        assert!(err.contains("ai/typo"), "{err}");
        assert!(err.contains("namespace/name"), "{err}");

        let oversized = r#"
namespaces:
  ai:
    - name: trainer
      replicas: 2
groups:
  - name: g
    members: ["ai/trainer"]
    min_members: 5
"#;
        let raw: RawWorkloadFile = serde_yaml::from_str(oversized).unwrap();
        let (mut pods, decls) = flatten_namespaces(&raw).unwrap();
        let err = resolve_declared_groups(&raw.groups, &decls, &mut pods).unwrap_err();
        assert!(err.contains("min_members is 5"), "{err}");
        assert!(err.contains("1..=2"), "{err}");
    }

    #[test]
    fn declared_group_and_inline_gang_coexist() {
        let yaml = r#"
namespaces:
  ai:
    - name: trainer
      replicas: 2
    - name: sidecar
      replicas: 2
      gang: "helpers"
groups:
  - name: training-job
    members: ["ai/trainer"]
    min_members: 1
"#;
        let raw: RawWorkloadFile = serde_yaml::from_str(yaml).unwrap();
        let (mut pods, decls) = flatten_namespaces(&raw).unwrap();
        let declared = resolve_declared_groups(&raw.groups, &decls, &mut pods).unwrap();
        let groups = merge_groups(declared, &pods);

        assert_eq!(groups.len(), 2);
        // Sorted by name: helpers keeps the shorthand's defaults, training-job keeps its own.
        assert_eq!(groups[0].name, "helpers");
        assert_eq!(groups[0].min_members, 2);
        assert_eq!(groups[1].name, "training-job");
        assert_eq!(groups[1].min_members, 1);
    }

    #[test]
    fn a_pod_cannot_belong_to_two_groups() {
        let yaml = r#"
namespaces:
  ai:
    - name: trainer
      gang: "helpers"
groups:
  - name: training-job
    members: ["ai/trainer"]
"#;
        let raw: RawWorkloadFile = serde_yaml::from_str(yaml).unwrap();
        let (mut pods, decls) = flatten_namespaces(&raw).unwrap();
        let err = resolve_declared_groups(&raw.groups, &decls, &mut pods).unwrap_err();
        assert!(err.contains("helpers"), "{err}");
        assert!(err.contains("training-job"), "{err}");
    }

    #[test]
    fn observed_block_parses_and_is_optional() {
        let yaml = r#"
namespaces:
  edge:
    - name: api-gateway
      requests: { cpu: 4.0, ram: "8Gi" }
      observed:
        window: 720h
        p95: { cpu: 2.4, ram: "5Gi" }
        peak: { cpu: 4.6, ram: "7Gi" }
        exceeded_request_fraction: 0.004
    - name: plain
      requests: { cpu: 1.0 }
"#;
        let raw: RawWorkloadFile = serde_yaml::from_str(yaml).unwrap();
        let (pods, _) = flatten_namespaces(&raw).unwrap();

        let observed = pods[0].observed.as_ref().expect("observed block dropped");
        assert_eq!(observed.window, "720h");
        assert!(
            (observed.p95.unwrap().ram - 5.0).abs() < 1e-9,
            "quantities must parse"
        );
        assert!((observed.peak.unwrap().cpu - 4.6).abs() < 1e-9);
        assert!((observed.exceeded_request_fraction - 0.004).abs() < 1e-12);
        assert!(observed.p50.is_none(), "absent percentiles stay absent");
        assert!(pods[1].observed.is_none(), "absent block must stay absent");
    }
}
