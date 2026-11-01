//! Render a blueprint as the IR v1 Blueprint document (§4), `unplaced:` included (X-4).
//!
//! Moved out of `main.rs` so that the document shape is tested. Without admission the
//! output is byte-identical to what the solver wrote before X-4.

use crate::admission::Admission;
use crate::model::{Blueprint, Node, Pod, UNPLACED, placements};
use crate::parser::{IR_API_VERSION, KIND_BLUEPRINT};

/// How the run that produced the blueprint ended, for its header.
#[derive(Debug, Clone, Copy)]
pub struct BlueprintVerdict {
    pub feasible: bool,
    pub admission: Option<Admission>,
}

/// The Blueprint document for `best` against `nodes`.
///
/// ```ignore
/// let yaml = render_blueprint_yaml(&best, &pods, &nodes, verdict);
/// std::fs::write("kuberina_solution.yaml", yaml)?;
/// ```
pub fn render_blueprint_yaml(
    best: &Blueprint,
    pods: &[Pod],
    nodes: &[Node],
    verdict: BlueprintVerdict,
) -> String {
    let mut yaml = String::new();
    if !verdict.feasible {
        // WHY: the file is written for inspection, not application. A reader who
        // opens it without having seen the console output must still be told.
        yaml.push_str("# INFEASIBLE — this assignment exceeds real node capacity.\n");
        yaml.push_str("# Written for inspection only. Do not apply. See the solver output.\n");
    }
    yaml.push_str(&header(best, verdict));
    yaml.push_str("solution:\n");
    for (pod_idx, node_idx) in placements(&best.assignment) {
        yaml.push_str(&format!(
            "  {}: {}\n",
            pods[pod_idx].identity(),
            nodes[node_idx].name
        ));
    }
    if verdict.admission.is_some() {
        yaml.push_str(&unplaced_list(best, pods));
    }
    yaml
}

/// `apiVersion`, `kind` and `metadata` (ADR-0002): `kuberina-forge out` checks them
/// before it links the plan to manifests.
fn header(best: &Blueprint, verdict: BlueprintVerdict) -> String {
    let placed = placements(&best.assignment).count();
    let mut header = format!(
        "apiVersion: {IR_API_VERSION}\nkind: {KIND_BLUEPRINT}\nmetadata:\n  feasible: {}\n  pods: {placed}\n  activeNodes: {}\n",
        verdict.feasible, best.scorecard.active_nodes as usize,
    );
    if let Some(admission) = verdict.admission {
        let declined = best.assignment.len() - placed;
        header.push_str(&format!(
            "  admission: {}\n  unplaced: {declined}\n",
            admission.label()
        ));
    }
    header
}

fn unplaced_list(best: &Blueprint, pods: &[Pod]) -> String {
    let declined: Vec<String> = best
        .assignment
        .iter()
        .zip(pods)
        .filter(|&(&node_idx, _)| node_idx == UNPLACED)
        .map(|(_, pod)| format!("  - {}\n", pod.identity()))
        .collect();
    if declined.is_empty() {
        return "unplaced: []\n".into();
    }
    format!("unplaced:\n{}", declined.concat())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_fixtures::{cpu_node, cpu_pod};

    fn blueprint(assignment: Vec<usize>, active_nodes: f64) -> Blueprint {
        let mut bp = Blueprint {
            assignment,
            fitness: 0.0,
            node_load: vec![],
            scorecard: Default::default(),
        };
        bp.scorecard.active_nodes = active_nodes;
        bp
    }

    #[test]
    fn without_admission_the_document_keeps_its_pre_x4_shape() {
        let pods = [cpu_pod("a", 1.0), cpu_pod("b", 1.0)];
        let nodes = [cpu_node("n1", 4.0), cpu_node("n2", 4.0)];
        let verdict = BlueprintVerdict {
            feasible: true,
            admission: None,
        };
        let yaml = render_blueprint_yaml(&blueprint(vec![0, 1], 2.0), &pods, &nodes, verdict);
        assert_eq!(
            yaml,
            "apiVersion: kuberina.io/v1\nkind: Blueprint\nmetadata:\n  feasible: true\n  pods: 2\n  activeNodes: 2\nsolution:\n  ns/a: n1\n  ns/b: n2\n"
        );
    }

    #[test]
    fn declined_pods_go_to_unplaced_and_the_header_says_how() {
        let pods = [cpu_pod("a", 1.0), cpu_pod("b", 1.0)];
        let nodes = [cpu_node("n1", 4.0)];
        let verdict = BlueprintVerdict {
            feasible: true,
            admission: Some(Admission::InOrder),
        };
        let yaml =
            render_blueprint_yaml(&blueprint(vec![0, UNPLACED], 1.0), &pods, &nodes, verdict);
        assert_eq!(
            yaml,
            "apiVersion: kuberina.io/v1\nkind: Blueprint\nmetadata:\n  feasible: true\n  pods: 1\n  activeNodes: 1\n  admission: in-order\n  unplaced: 1\nsolution:\n  ns/a: n1\nunplaced:\n  - ns/b\n"
        );
    }

    #[test]
    fn infeasible_document_starts_with_an_aligned_banner() {
        // The second banner line used to carry 13 spaces of source indentation.
        let pods = [cpu_pod("a", 9.0)];
        let nodes = [cpu_node("n1", 4.0)];
        let verdict = BlueprintVerdict {
            feasible: false,
            admission: None,
        };
        let yaml = render_blueprint_yaml(&blueprint(vec![0], 1.0), &pods, &nodes, verdict);
        assert!(yaml.starts_with(
            "# INFEASIBLE — this assignment exceeds real node capacity.\n# Written for inspection only. Do not apply. See the solver output.\napiVersion:"
        ));
    }
}
