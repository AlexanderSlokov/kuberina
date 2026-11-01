//! Read and validate the arrival order of IR v1 §3.5 (`arrival`), backlog X-4.
//!
//! In-order admission (IR v1 §4.1) decides Pod by Pod in arrival order. The parser
//! records each declaration's `arrival` here, so that two declarations claiming the
//! same position fail at load with both names in the error.

use std::collections::HashMap;

use crate::model::Pod;

/// The declarations seen so far, keyed by their `arrival`.
pub type ArrivalRegistry = HashMap<u64, String>;

/// Record that `declaration` arrives at position `arrival`; refuse a taken position.
///
/// ```
/// # use kuberina_solver::admission_ir::{register_arrival, ArrivalRegistry};
/// let mut seen = ArrivalRegistry::new();
/// assert!(register_arrival(&mut seen, "ai/trainer", Some(3)).is_ok());
/// assert!(register_arrival(&mut seen, "ai/inference", Some(3)).is_err());
/// ```
pub fn register_arrival(
    seen: &mut ArrivalRegistry,
    declaration: &str,
    arrival: Option<u64>,
) -> Result<(), String> {
    let Some(position) = arrival else {
        return Ok(());
    };
    if let Some(first) = seen.get(&position) {
        return Err(format!(
            "{declaration} and {first} both have arrival: {position}; expected a distinct arrival per declaration"
        ));
    }
    seen.insert(position, declaration.to_string());
    Ok(())
}

/// Pod indices in arrival order, for in-order admission.
///
/// Replicas of one declaration are consecutive in `pods`, in replica order, so a stable
/// sort by `arrival` keeps them in the order IR v1 §3.5 states.
///
/// ```ignore
/// let order = arrival_order(&pods)?; // error if any Pod lacks `arrival`
/// ```
pub fn arrival_order(pods: &[Pod]) -> Result<Vec<usize>, String> {
    let mut keyed = Vec::with_capacity(pods.len());
    for (pod_idx, pod) in pods.iter().enumerate() {
        let Some(arrival) = pod.arrival else {
            return Err(format!(
                "pod {} has no arrival; --admission in-order needs arrival on every declaration (IR v1 §3.5)",
                pod.identity()
            ));
        };
        keyed.push((arrival, pod_idx));
    }
    keyed.sort();
    Ok(keyed.into_iter().map(|(_, pod_idx)| pod_idx).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ResourceVector;

    fn pod(name: &str, arrival: Option<u64>) -> Pod {
        Pod {
            name: name.into(),
            namespace: "ns".into(),
            requests: ResourceVector::zero(),
            tolerations: vec![],
            node_selector: HashMap::new(),
            affinity_targets: vec![],
            anti_affinity_targets: vec![],
            group_name: String::new(),
            topology_spread: None,
            observed: None,
            gpu_share: None,
            arrival,
        }
    }

    #[test]
    fn duplicate_arrival_names_both_declarations_and_the_position() {
        let mut seen = ArrivalRegistry::new();
        register_arrival(&mut seen, "ai/trainer", Some(3)).unwrap();
        let err = register_arrival(&mut seen, "ai/inference", Some(3)).unwrap_err();
        assert!(
            err.contains("ai/trainer") && err.contains("ai/inference") && err.contains('3'),
            "{err}"
        );
    }

    #[test]
    fn absent_arrival_is_not_registered() {
        let mut seen = ArrivalRegistry::new();
        register_arrival(&mut seen, "ai/a", None).unwrap();
        register_arrival(&mut seen, "ai/b", None).unwrap();
        assert!(seen.is_empty());
    }

    #[test]
    fn order_follows_arrival_and_keeps_replicas_in_replica_order() {
        let pods = [
            pod("late-0000", Some(9)),
            pod("late-0001", Some(9)),
            pod("early-0", Some(2)),
        ];
        assert_eq!(arrival_order(&pods), Ok(vec![2, 0, 1]));
    }

    #[test]
    fn missing_arrival_names_the_pod() {
        let pods = [pod("a-0", Some(1)), pod("b-0", None)];
        let err = arrival_order(&pods).unwrap_err();
        assert!(err.contains("ns/b-0") && err.contains("arrival"), "{err}");
    }
}
