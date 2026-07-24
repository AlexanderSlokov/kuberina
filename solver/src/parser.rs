//! YAML parser for Kuberina cluster topology and workload manifests.
//!
//! Ported from `research/src/kuberina/parser.py`.
//! Uses serde_yaml for direct struct deserialization — less code than the
//! Python version because serde maps fields automatically.

use std::path::Path;

use crate::model::{DaemonSet, Node, Pod, PodGroup};

/// Raw YAML shape for infrastructure file.
#[derive(serde::Deserialize)]
struct InfraFile {
    #[serde(default)]
    nodes: Vec<Node>,
    #[serde(default)]
    daemonsets: Vec<DaemonSet>,
}

/// Raw YAML shape for workload file.
#[derive(serde::Deserialize)]
struct WorkloadFile {
    #[serde(default)]
    pods: Vec<Pod>,
    #[serde(default)]
    groups: Vec<PodGroup>,
}

/// Parse cluster topology YAML into Node and DaemonSet lists.
///
/// ```no_run
/// # use kuberina_solver::parser::load_infra;
/// let (nodes, ds) = load_infra("testdata/homelab_infra.yaml").unwrap();
/// assert!(!nodes.is_empty());
/// ```
pub fn load_infra(path: impl AsRef<Path>) -> Result<(Vec<Node>, Vec<DaemonSet>), String> {
    let content = std::fs::read_to_string(path.as_ref())
        .map_err(|e| format!("cannot read {}: {e}", path.as_ref().display()))?;
    let infra: InfraFile = serde_yaml::from_str(&content)
        .map_err(|e| format!("YAML parse error in {}: {e}", path.as_ref().display()))?;
    Ok((infra.nodes, infra.daemonsets))
}

/// Parse workload manifests YAML into Pod and PodGroup lists.
///
/// Resolves group membership: scans pod `group_name` fields to populate
/// `PodGroup.pod_indices`.
///
/// ```no_run
/// # use kuberina_solver::parser::load_workloads;
/// let (pods, groups) = load_workloads("testdata/homelab_workloads.yaml").unwrap();
/// assert!(!pods.is_empty());
/// ```
pub fn load_workloads(path: impl AsRef<Path>) -> Result<(Vec<Pod>, Vec<PodGroup>), String> {
    let content = std::fs::read_to_string(path.as_ref())
        .map_err(|e| format!("cannot read {}: {e}", path.as_ref().display()))?;
    let mut wf: WorkloadFile = serde_yaml::from_str(&content)
        .map_err(|e| format!("YAML parse error in {}: {e}", path.as_ref().display()))?;

    resolve_groups(&wf.pods, &mut wf.groups);
    Ok((wf.pods, wf.groups))
}

/// Build PodGroup.pod_indices by matching pod.group_name → group.name.
fn resolve_groups(pods: &[Pod], groups: &mut [PodGroup]) {
    for group in groups.iter_mut() {
        let indices: Vec<usize> = pods
            .iter()
            .enumerate()
            .filter(|(_, p)| p.group_name == group.name)
            .map(|(i, _)| i)
            .collect();
        if group.min_members == 0 {
            group.min_members = indices.len();
        }
        group.pod_indices = indices;
    }
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
        assert_eq!(nodes[0].allocatable.cpu, 4.0);
    }

    #[test]
    fn load_homelab_workloads() {
        let (pods, groups) = load_workloads("testdata/homelab_workloads.yaml").unwrap();
        assert_eq!(pods.len(), 10);
        assert!(groups.is_empty());
        assert_eq!(pods[0].name, "pihole");
        assert_eq!(pods[0].namespace, "dns");
    }

    #[test]
    fn missing_file_returns_error() {
        let result = load_infra("nonexistent.yaml");
        assert!(result.is_err());
    }
}
