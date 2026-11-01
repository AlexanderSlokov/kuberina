//! Resolve affinity and anti-affinity targets to unrolled pod identities.
//!
//! IR v1 §3.1.1: a target names a declaration (`ai/trainer`, every replica) or one
//! unrolled pod (`ai/trainer-0003`). Before issue #26 the fitness function looked
//! targets up by bare pod name, so no target in either form matched and affinity
//! had no effect on any plan.

use std::collections::HashMap;

use crate::model::Pod;

/// Rewrite every pod's targets into the identities of the pods they name.
///
/// Each `affinity` entry keeps its own group of identities, because Kubernetes
/// satisfies it with any one of them; `antiAffinity` targets become one flat list.
///
/// `declarations` maps `namespace/name` to the indices of that declaration's
/// replicas in `pods`. Returns an error naming the first target that resolves to
/// nothing.
///
/// ```ignore
/// resolve_affinity_targets(&mut pods, &declarations)?;
/// // pods[i].affinity_targets now holds identities such as "ai/trainer-0000".
/// ```
pub fn resolve_affinity_targets(
    pods: &mut [Pod],
    declarations: &HashMap<String, Vec<usize>>,
) -> Result<(), String> {
    let identities: Vec<String> = pods.iter().map(Pod::identity).collect();
    let resolver = TargetResolver {
        identities: &identities,
        index_of: identities
            .iter()
            .enumerate()
            .map(|(i, id)| (id.as_str(), i))
            .collect(),
        declarations,
    };
    for (owner, pod) in pods.iter_mut().enumerate() {
        pod.affinity_targets = pod
            .affinity_targets
            .iter()
            .map(|entry| resolver.resolve_list(owner, entry))
            .collect::<Result<_, _>>()?;
        pod.anti_affinity_targets = resolver.resolve_list(owner, &pod.anti_affinity_targets)?;
    }
    Ok(())
}

/// Lookup tables shared by every target in one workloads document.
struct TargetResolver<'a> {
    identities: &'a [String],
    index_of: HashMap<&'a str, usize>,
    declarations: &'a HashMap<String, Vec<usize>>,
}

impl TargetResolver<'_> {
    /// Resolve one pod's target list into identities, leaving out the pod itself.
    fn resolve_list(&self, owner: usize, targets: &[String]) -> Result<Vec<String>, String> {
        let mut resolved = Vec::new();
        for target in targets {
            let named = self.resolve_one(owner, target)?;
            let others = named.into_iter().filter(|&j| j != owner);
            resolved.extend(others.map(|j| self.identities[j].clone()));
        }
        Ok(resolved)
    }

    /// Indices named by one target: a declaration first, then an unrolled identity.
    fn resolve_one(&self, owner: usize, target: &str) -> Result<Vec<usize>, String> {
        if let Some(replicas) = self.declarations.get(target) {
            return Ok(replicas.clone());
        }
        if let Some(&index) = self.index_of.get(target) {
            return Ok(vec![index]);
        }
        Err(format!(
            "pod {} has affinity target {target:?}, which matches no declaration and no pod; \
             expected namespace/name for a declaration or namespace/name-NNNN for one replica",
            self.identities[owner]
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ResourceVector;

    fn pod(namespace: &str, name: &str) -> Pod {
        Pod {
            name: name.into(),
            namespace: namespace.into(),
            requests: ResourceVector::zero(),
            tolerations: vec![],
            node_selector: HashMap::new(),
            affinity_targets: vec![],
            anti_affinity_targets: vec![],
            group_name: String::new(),
            topology_spread: None,
            observed: None,
            gpu_share: None,
        }
    }

    /// Two trainer replicas and one parameter server, all in namespace `ai`.
    fn training_job() -> (Vec<Pod>, HashMap<String, Vec<usize>>) {
        let pods = vec![
            pod("ai", "trainer-0000"),
            pod("ai", "trainer-0001"),
            pod("ai", "parameter-server-0"),
        ];
        let declarations = HashMap::from([
            ("ai/trainer".to_string(), vec![0, 1]),
            ("ai/parameter-server".to_string(), vec![2]),
        ]);
        (pods, declarations)
    }

    #[test]
    fn declaration_target_expands_to_every_replica() {
        let (mut pods, declarations) = training_job();
        pods[2].affinity_targets = vec![vec!["ai/trainer".into()]];
        resolve_affinity_targets(&mut pods, &declarations).unwrap();
        assert_eq!(
            pods[2].affinity_targets,
            vec![vec!["ai/trainer-0000", "ai/trainer-0001"]]
        );
    }

    #[test]
    fn unrolled_identity_target_names_one_pod() {
        let (mut pods, declarations) = training_job();
        pods[2].anti_affinity_targets = vec!["ai/trainer-0001".into()];
        resolve_affinity_targets(&mut pods, &declarations).unwrap();
        assert_eq!(pods[2].anti_affinity_targets, vec!["ai/trainer-0001"]);
    }

    #[test]
    fn target_without_namespace_is_an_error_naming_the_value() {
        let (mut pods, declarations) = training_job();
        pods[2].affinity_targets = vec![vec!["trainer".into()]];
        let err = resolve_affinity_targets(&mut pods, &declarations).unwrap_err();
        assert!(err.contains("\"trainer\""), "{err}");
        assert!(err.contains("ai/parameter-server-0"), "{err}");
        assert!(err.contains("namespace/name"), "{err}");
    }

    #[test]
    fn own_declaration_resolves_to_the_other_replicas() {
        let (mut pods, declarations) = training_job();
        pods[0].anti_affinity_targets = vec!["ai/trainer".into()];
        resolve_affinity_targets(&mut pods, &declarations).unwrap();
        assert_eq!(pods[0].anti_affinity_targets, vec!["ai/trainer-0001"]);
    }
}
