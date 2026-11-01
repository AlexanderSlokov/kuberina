//! Read and validate the per-device GPU fields of IR v1 (§2 `gpuDevices`, §3.4
//! `gpuMilli`), backlog X-3.
//!
//! The parser calls these after it has read a node's `allocatable` or a declaration's
//! `requests`, so each rule is checked once, with the offending name and value in the
//! error.

use crate::gpu_devices::MILLI_PER_DEVICE;
use crate::model::{GpuShare, Node, Pod, ResourceVector};

/// Validate a node's `gpuDevices` and reconcile it with `allocatable.gpu`.
///
/// `gpu_declared` says whether the document stated `allocatable.gpu`. When it did not,
/// the node's GPU capacity becomes the device count.
///
/// ```
/// # use kuberina_solver::gpu_ir::node_gpu_devices;
/// # use kuberina_solver::model::ResourceVector;
/// let mut allocatable = ResourceVector::new(64.0, 256.0, f64::MAX);
/// assert_eq!(node_gpu_devices("gpu-a", Some(8), false, &mut allocatable), Ok(Some(8)));
/// assert_eq!(allocatable.gpu, 8.0);
/// ```
pub fn node_gpu_devices(
    node: &str,
    gpu_devices: Option<u32>,
    gpu_declared: bool,
    allocatable: &mut ResourceVector,
) -> Result<Option<u32>, String> {
    let Some(count) = gpu_devices else {
        return Ok(None);
    };
    if count == 0 {
        return Err(format!(
            "node {node:?} has gpuDevices: 0; expected at least 1"
        ));
    }
    if !gpu_declared {
        allocatable.gpu = f64::from(count);
    } else if allocatable.gpu != f64::from(count) {
        return Err(format!(
            "node {node:?} has gpuDevices: {count} but allocatable.gpu: {}; expected them equal, or allocatable.gpu omitted",
            allocatable.gpu
        ));
    }
    Ok(Some(count))
}

/// Validate a declaration's `gpuMilli` and turn its `requests.gpu` into the aggregate.
///
/// With `gpuMilli: m`, `requests.gpu` counts devices and must be a whole number of at
/// least 1. On return, `requests.gpu` holds `devices × m / 1000`.
///
/// ```
/// # use kuberina_solver::gpu_ir::pod_gpu_share;
/// # use kuberina_solver::model::{GpuShare, ResourceVector};
/// let mut requests = ResourceVector::new(4.0, 8.0, 1.0);
/// let share = pod_gpu_share("ai/inference", Some(500), &mut requests).unwrap();
/// assert_eq!(share, Some(GpuShare { devices: 1, milli: 500 }));
/// assert_eq!(requests.gpu, 0.5);
/// ```
pub fn pod_gpu_share(
    declaration: &str,
    gpu_milli: Option<u32>,
    requests: &mut ResourceVector,
) -> Result<Option<GpuShare>, String> {
    let Some(milli) = gpu_milli else {
        return Ok(None);
    };
    if !(1..=MILLI_PER_DEVICE).contains(&milli) {
        return Err(format!(
            "{declaration} has gpuMilli: {milli}; expected an integer in 1..=1000"
        ));
    }
    let devices = requests.gpu;
    if devices < 1.0 || devices.fract() != 0.0 {
        return Err(format!(
            "{declaration} has gpuMilli with requests.gpu: {devices}; expected a whole number of devices, at least 1"
        ));
    }
    let share = GpuShare {
        devices: devices as u32,
        milli,
    };
    requests.gpu = share.total_milli() as f64 / f64::from(MILLI_PER_DEVICE);
    Ok(Some(share))
}

/// Refuse pods that cannot be checked per device when any node declares devices.
///
/// A pod with a fractional `requests.gpu` and no `gpuMilli` has no device shape
/// (IR v1 §3.4).
///
/// ```ignore
/// check_device_inputs(&nodes, &pods)?;
/// ```
pub fn check_device_inputs(nodes: &[Node], pods: &[Pod]) -> Result<(), String> {
    if nodes.iter().all(|n| n.gpu_devices.is_none()) {
        return Ok(());
    }
    let shapeless = pods
        .iter()
        .find(|p| p.gpu_share.is_none() && p.requests.gpu.fract() != 0.0);
    match shapeless {
        None => Ok(()),
        Some(pod) => Err(format!(
            "pod {} requests gpu: {} without gpuMilli, while a node declares gpuDevices; \
             expected a whole number of devices, or gpuMilli with gpu: 1 (IR v1 §3.4)",
            pod.identity(),
            pod.requests.gpu
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn node(devices: Option<u32>) -> Node {
        Node {
            name: "gpu-node-01".into(),
            allocatable: ResourceVector::new(64.0, 256.0, 8.0),
            labels: HashMap::new(),
            taints: vec![],
            zone: String::new(),
            rack: String::new(),
            gpu_devices: devices,
        }
    }

    fn pod(gpu: f64, gpu_share: Option<GpuShare>) -> Pod {
        Pod {
            name: "inference-0".into(),
            namespace: "ai".into(),
            requests: ResourceVector::new(1.0, 1.0, gpu),
            tolerations: vec![],
            node_selector: HashMap::new(),
            affinity_targets: vec![],
            anti_affinity_targets: vec![],
            group_name: String::new(),
            topology_spread: None,
            observed: None,
            gpu_share,
        }
    }

    #[test]
    fn absent_gpu_devices_leaves_the_node_unchanged() {
        let mut allocatable = ResourceVector::new(64.0, 256.0, 8.0);
        assert_eq!(
            node_gpu_devices("gpu-a", None, true, &mut allocatable),
            Ok(None)
        );
        assert_eq!(allocatable.gpu, 8.0);
    }

    #[test]
    fn matching_allocatable_gpu_is_accepted() {
        let mut allocatable = ResourceVector::new(64.0, 256.0, 8.0);
        assert_eq!(
            node_gpu_devices("gpu-a", Some(8), true, &mut allocatable),
            Ok(Some(8))
        );
    }

    #[test]
    fn allocatable_gpu_that_differs_from_the_device_count_names_node_and_values() {
        let mut allocatable = ResourceVector::new(64.0, 256.0, 4.0);
        let err = node_gpu_devices("gpu-a", Some(8), true, &mut allocatable).unwrap_err();
        assert!(
            err.contains("gpu-a") && err.contains('8') && err.contains('4'),
            "{err}"
        );
    }

    #[test]
    fn zero_devices_is_an_error_naming_the_node() {
        let mut allocatable = ResourceVector::new(64.0, 256.0, f64::MAX);
        let err = node_gpu_devices("gpu-a", Some(0), false, &mut allocatable).unwrap_err();
        assert!(err.contains("gpu-a") && err.contains("at least 1"), "{err}");
    }

    #[test]
    fn absent_gpu_milli_leaves_requests_unchanged() {
        let mut requests = ResourceVector::new(4.0, 8.0, 2.0);
        assert_eq!(pod_gpu_share("ai/trainer", None, &mut requests), Ok(None));
        assert_eq!(requests.gpu, 2.0);
    }

    #[test]
    fn multi_device_share_sets_the_aggregate_to_devices_times_milli() {
        let mut requests = ResourceVector::new(4.0, 8.0, 2.0);
        let share = pod_gpu_share("ai/trainer", Some(250), &mut requests).unwrap();
        assert_eq!(
            share,
            Some(GpuShare {
                devices: 2,
                milli: 250
            })
        );
        assert_eq!(requests.gpu, 0.5);
    }

    #[test]
    fn gpu_milli_outside_one_to_a_thousand_names_the_value_and_range() {
        for bad in [0, 1001] {
            let mut requests = ResourceVector::new(4.0, 8.0, 1.0);
            let err = pod_gpu_share("ai/inference", Some(bad), &mut requests).unwrap_err();
            assert!(
                err.contains("ai/inference")
                    && err.contains(&bad.to_string())
                    && err.contains("1..=1000"),
                "{err}"
            );
        }
    }

    #[test]
    fn gpu_milli_with_a_fractional_or_zero_device_count_names_the_value() {
        for bad in [0.0, 0.5] {
            let mut requests = ResourceVector::new(4.0, 8.0, bad);
            let err = pod_gpu_share("ai/inference", Some(500), &mut requests).unwrap_err();
            assert!(
                err.contains("ai/inference") && err.contains("whole number"),
                "{err}"
            );
        }
    }

    #[test]
    fn fractional_gpu_without_gpu_milli_is_refused_when_a_node_has_devices() {
        let err = check_device_inputs(&[node(Some(8))], &[pod(0.5, None)]).unwrap_err();
        assert!(
            err.contains("ai/inference-0") && err.contains("0.5") && err.contains("gpuMilli"),
            "{err}"
        );
    }

    #[test]
    fn fractional_gpu_without_gpu_milli_is_accepted_without_device_nodes() {
        assert_eq!(
            check_device_inputs(&[node(None)], &[pod(0.5, None)]),
            Ok(())
        );
    }

    #[test]
    fn shared_and_whole_gpu_pods_are_accepted_with_device_nodes() {
        let pods = [
            pod(
                0.5,
                Some(GpuShare {
                    devices: 1,
                    milli: 500,
                }),
            ),
            pod(2.0, None),
        ];
        assert_eq!(check_device_inputs(&[node(Some(8))], &pods), Ok(()));
    }
}
