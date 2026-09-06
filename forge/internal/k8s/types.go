// Package k8s decodes Kubernetes manifests into the subset Kuberina cares about.
//
// Unlike the IR reader, this decoder is deliberately lenient about unknown fields.
// It is reading foreign documents written for a different consumer, and a manifest
// carrying `spec.strategy` or a dozen annotations is not an error — it is Tuesday.
// Only fields listed here are read; everything else is ignored by design.
package k8s

import (
	"fmt"

	"gopkg.in/yaml.v3"
)

// Quantity keeps a resource value exactly as written, so `1`, `1.0` and `1000m`
// survive decoding intact and are parsed once, later, by the quantity package.
type Quantity string

func (q *Quantity) UnmarshalYAML(node *yaml.Node) error {
	if node.Kind != yaml.ScalarNode {
		return fmt.Errorf("expected a scalar resource quantity, got %v at line %d",
			node.Tag, node.Line)
	}
	*q = Quantity(node.Value)
	return nil
}

// Object is any Kubernetes document. Kind decides which fields are meaningful.
type Object struct {
	APIVersion string     `yaml:"apiVersion"`
	Kind       string     `yaml:"kind"`
	Metadata   ObjectMeta `yaml:"metadata"`
	Spec       Spec       `yaml:"spec"`
	Status     Status     `yaml:"status"`
}

type ObjectMeta struct {
	Name      string            `yaml:"name"`
	Namespace string            `yaml:"namespace"`
	Labels    map[string]string `yaml:"labels"`
}

// Spec covers both controller specs (Deployment, StatefulSet, DaemonSet, Job) and
// the bare PodSpec, which is what `spec` holds on a Pod.
type Spec struct {
	Replicas *int         `yaml:"replicas"`
	Template *PodTemplate `yaml:"template"`
	Taints   []Taint      `yaml:"taints"`

	PodSpec `yaml:",inline"`
}

type PodTemplate struct {
	Metadata ObjectMeta `yaml:"metadata"`
	Spec     PodSpec    `yaml:"spec"`
}

type PodSpec struct {
	Containers                []Container                `yaml:"containers"`
	InitContainers            []Container                `yaml:"initContainers"`
	NodeSelector              map[string]string          `yaml:"nodeSelector"`
	NodeName                  string                     `yaml:"nodeName"`
	Tolerations               []Toleration               `yaml:"tolerations"`
	TopologySpreadConstraints []TopologySpreadConstraint `yaml:"topologySpreadConstraints"`
	Affinity                  *Affinity                  `yaml:"affinity"`
}

type Container struct {
	Name      string             `yaml:"name"`
	Resources ContainerResources `yaml:"resources"`
}

type ContainerResources struct {
	Requests map[string]Quantity `yaml:"requests"`
	Limits   map[string]Quantity `yaml:"limits"`
}

type Toleration struct {
	Key      string `yaml:"key"`
	Operator string `yaml:"operator"`
	Value    string `yaml:"value"`
	Effect   string `yaml:"effect"`
}

type Taint struct {
	Key    string `yaml:"key"`
	Value  string `yaml:"value"`
	Effect string `yaml:"effect"`
}

type TopologySpreadConstraint struct {
	MaxSkew     int    `yaml:"maxSkew"`
	TopologyKey string `yaml:"topologyKey"`
}

type Affinity struct {
	PodAffinity     *PodAffinityTerms `yaml:"podAffinity"`
	PodAntiAffinity *PodAffinityTerms `yaml:"podAntiAffinity"`
}

type PodAffinityTerms struct {
	Required []PodAffinityTerm `yaml:"requiredDuringSchedulingIgnoredDuringExecution"`
	// Preferred terms nest the selector one level deeper, under `podAffinityTerm`.
	Preferred []WeightedPodAffinityTerm `yaml:"preferredDuringSchedulingIgnoredDuringExecution"`
}

type WeightedPodAffinityTerm struct {
	Weight int             `yaml:"weight"`
	Term   PodAffinityTerm `yaml:"podAffinityTerm"`
}

type PodAffinityTerm struct {
	LabelSelector *LabelSelector `yaml:"labelSelector"`
	TopologyKey   string         `yaml:"topologyKey"`
}

type LabelSelector struct {
	MatchLabels map[string]string `yaml:"matchLabels"`
	// MatchExpressions is read so it can be reported as unsupported rather than
	// silently ignored — a selector Kuberina cannot resolve must not look resolved.
	MatchExpressions []map[string]any `yaml:"matchExpressions"`
}

// Status carries the only field Kuberina reads from a live Node dump.
type Status struct {
	Allocatable map[string]Quantity `yaml:"allocatable"`
	Capacity    map[string]Quantity `yaml:"capacity"`
}

// Namespace returns the namespace to file this object under.
func (o *Object) Namespace() string {
	if o.Metadata.Namespace == "" {
		return "default"
	}
	return o.Metadata.Namespace
}

// PodTemplateSpec returns the pod spec a controller schedules, or the object's own
// spec when it is a bare Pod.
func (o *Object) PodTemplateSpec() PodSpec {
	if o.Spec.Template != nil {
		return o.Spec.Template.Spec
	}
	return o.Spec.PodSpec
}

// PodLabels returns the labels the scheduled pods will carry.
func (o *Object) PodLabels() map[string]string {
	if o.Spec.Template != nil && len(o.Spec.Template.Metadata.Labels) > 0 {
		return o.Spec.Template.Metadata.Labels
	}
	return o.Metadata.Labels
}

// ReplicaCount defaults to 1, matching Kubernetes.
func (o *Object) ReplicaCount() int {
	if o.Spec.Replicas == nil {
		return 1
	}
	return *o.Spec.Replicas
}
