// Package forgein converts Kubernetes manifests into Kuberina IR v1.
//
// This is the "forge-in" half of ROADMAP Phase 1: the frontend of a compiler
// pipeline whose middle-end is the Rust solver. What it emits is defined by
// docs/references/ir-v1.md.
package forgein

import (
	"fmt"
	"sort"

	"github.com/AlexanderSlokov/kuberina/forge/internal/ir"
	"github.com/AlexanderSlokov/kuberina/forge/internal/k8s"
)

// Kinds that describe capacity rather than demand.
const kindNode = "Node"

// Kinds whose pods are scheduled once per node, and so are pre-deducted rather
// than placed. See ir-v1.md §2.
const kindDaemonSet = "DaemonSet"

// workloadKinds produce pods the solver places.
var workloadKinds = map[string]bool{
	"Pod":         true,
	"Deployment":  true,
	"StatefulSet": true,
	"ReplicaSet":  true,
	"Job":         true,
}

// Result is what one conversion produced, including what it could not represent.
type Result struct {
	Infra     *ir.Infrastructure
	Workloads *ir.Workloads
	// Warnings names inputs that were read but could not be fully represented.
	// They are reported rather than dropped: a constraint the operator wrote and
	// Kuberina silently ignored is worse than one it declines out loud.
	Warnings []string
}

// Convert turns decoded manifests into an infrastructure and a workloads document.
func Convert(objects []k8s.Object) (*Result, error) {
	res := &Result{Infra: ir.NewInfrastructure(), Workloads: ir.NewWorkloads()}

	declarations := collectDeclarations(objects)
	for i := range objects {
		if err := res.add(&objects[i], declarations); err != nil {
			return nil, err
		}
	}
	res.sortForDeterminism()
	return res, nil
}

// add routes one object to the part of the IR it belongs in.
func (r *Result) add(obj *k8s.Object, declarations []declaration) error {
	switch {
	case obj.Kind == kindNode:
		node, err := convertNode(obj)
		if err != nil {
			return err
		}
		r.Infra.Nodes = append(r.Infra.Nodes, node)
	case obj.Kind == kindDaemonSet:
		ds, err := convertDaemonSet(obj)
		if err != nil {
			return err
		}
		r.Infra.DaemonSets = append(r.Infra.DaemonSets, ds)
	case workloadKinds[obj.Kind]:
		return r.addWorkload(obj, declarations)
	}
	return nil // an object Kuberina does not model: Service, ConfigMap, and friends
}

func (r *Result) addWorkload(obj *k8s.Object, declarations []declaration) error {
	pod, warnings, err := convertWorkload(obj, declarations)
	if err != nil {
		return err
	}
	ns := obj.Namespace()
	r.Workloads.Namespaces[ns] = append(r.Workloads.Namespaces[ns], pod)
	r.Warnings = append(r.Warnings, warnings...)
	return nil
}

// sortForDeterminism makes the emitted IR byte-identical for identical input.
//
// WHY it matters: the IR is a reviewed artifact under version control. A diff that
// churns because a directory walk returned files in a different order is a diff
// nobody reads.
func (r *Result) sortForDeterminism() {
	sort.Slice(r.Infra.Nodes, func(i, j int) bool {
		return r.Infra.Nodes[i].Name < r.Infra.Nodes[j].Name
	})
	sort.Slice(r.Infra.DaemonSets, func(i, j int) bool {
		return r.Infra.DaemonSets[i].Name < r.Infra.DaemonSets[j].Name
	})
	for ns := range r.Workloads.Namespaces {
		pods := r.Workloads.Namespaces[ns]
		sort.Slice(pods, func(i, j int) bool { return pods[i].Name < pods[j].Name })
	}
	sort.Strings(r.Warnings)
}

// convertNode maps a Node object onto IR capacity.
//
// Allocatable is preferred over capacity: it is what the kubelet will actually hand
// out, which is the number the packing model needs.
func convertNode(obj *k8s.Object) (ir.Node, error) {
	quantities := obj.Status.Allocatable
	if len(quantities) == 0 {
		quantities = obj.Status.Capacity
	}
	resources, err := resourcesFrom(quantities, obj.Metadata.Name)
	if err != nil {
		return ir.Node{}, err
	}
	return ir.Node{
		Name:        obj.Metadata.Name,
		Zone:        obj.Metadata.Labels[labelZone],
		Rack:        rackOf(obj.Metadata.Labels),
		Labels:      obj.Metadata.Labels,
		Taints:      convertTaints(obj.Spec.Taints),
		Allocatable: resources,
	}, nil
}

func convertTaints(taints []k8s.Taint) []ir.Taint {
	out := make([]ir.Taint, 0, len(taints))
	for _, t := range taints {
		out = append(out, ir.Taint{Key: t.Key, Value: t.Value, Effect: t.Effect})
	}
	if len(out) == 0 {
		return nil
	}
	return out
}

// convertDaemonSet sums one replica's demand: a DaemonSet lands on every node.
func convertDaemonSet(obj *k8s.Object) (ir.DaemonSet, error) {
	spec := obj.PodTemplateSpec()
	resources, err := sumContainers(spec, obj.Metadata.Name)
	if err != nil {
		return ir.DaemonSet{}, err
	}
	return ir.DaemonSet{
		Name:         obj.Metadata.Name,
		Resources:    resources,
		NodeSelector: spec.NodeSelector,
		Tolerations:  tolerationKeys(spec.Tolerations),
	}, nil
}

// tolerationKeys flattens tolerations to keys, which is what the infra IR carries.
func tolerationKeys(tolerations []k8s.Toleration) []string {
	out := make([]string, 0, len(tolerations))
	for _, t := range tolerations {
		if t.Key != "" {
			out = append(out, t.Key)
		}
	}
	if len(out) == 0 {
		return nil
	}
	return out
}

// convertWorkload maps one controller or bare Pod onto an IR pod declaration.
func convertWorkload(obj *k8s.Object, declarations []declaration) (ir.Pod, []string, error) {
	spec := obj.PodTemplateSpec()
	requests, err := sumContainers(spec, obj.Metadata.Name)
	if err != nil {
		return ir.Pod{}, nil, err
	}
	affinity, anti, warnings := resolveAffinity(obj, spec, declarations)

	return ir.Pod{
		Name:           obj.Metadata.Name,
		Replicas:       obj.ReplicaCount(),
		Requests:       requests,
		NodeSelector:   spec.NodeSelector,
		Tolerations:    convertTolerations(spec.Tolerations),
		Affinity:       affinity,
		AntiAffinity:   anti,
		TopologySpread: convertSpread(spec.TopologySpreadConstraints),
	}, warnings, nil
}

func convertTolerations(tolerations []k8s.Toleration) []ir.Toleration {
	out := make([]ir.Toleration, 0, len(tolerations))
	for _, t := range tolerations {
		out = append(out, ir.Toleration{
			Key: t.Key, Operator: t.Operator, Value: t.Value, Effect: t.Effect,
		})
	}
	if len(out) == 0 {
		return nil
	}
	return out
}

// convertSpread keeps the first constraint: IR v1 models one spread rule per pod.
func convertSpread(constraints []k8s.TopologySpreadConstraint) *ir.TopologySpread {
	if len(constraints) == 0 {
		return nil
	}
	c := constraints[0]
	return &ir.TopologySpread{MaxSkew: c.MaxSkew, TopologyKey: c.TopologyKey}
}

// sumContainers adds up requests across containers, as the kubelet does.
//
// Init containers are excluded: they do not run concurrently with the main
// containers, so counting them would inflate steady-state demand.
func sumContainers(spec k8s.PodSpec, owner string) (ir.Resources, error) {
	var total ir.Resources
	for _, c := range spec.Containers {
		part, err := resourcesFrom(c.Resources.Requests, fmt.Sprintf("%s/%s", owner, c.Name))
		if err != nil {
			return ir.Resources{}, err
		}
		total = addResources(total, part)
	}
	return total, nil
}

func addResources(a, b ir.Resources) ir.Resources {
	a.CPU += b.CPU
	a.RAM += b.RAM
	a.GPU += b.GPU
	a.Storage += b.Storage
	return a
}
