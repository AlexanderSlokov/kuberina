// Package ir holds the Kuberina IR v1 document types.
//
// The normative schema is docs/references/ir-v1.md. These structs are the Go half of
// it; solver/src/parser.rs is the Rust half. A field added here without a
// corresponding entry in the specification is a bug in this package.
package ir

const (
	// APIVersion every document Kuberina writes carries.
	APIVersion = "kuberina.io/v1"

	KindInfrastructure = "Infrastructure"
	KindWorkloads      = "Workloads"
	KindBlueprint      = "Blueprint"
)

// Resources is the eight-dimensional block used for capacity and for demand.
//
// Units are normative: CPU in cores, RAM and storage in GiB, GPU in whole devices,
// disk and network throughput in MB/s.
type Resources struct {
	CPU     float64    `yaml:"cpu,omitempty"`
	RAM     float64    `yaml:"ram,omitempty"`
	GPU     float64    `yaml:"gpu,omitempty"`
	Storage float64    `yaml:"storage,omitempty"`
	Disk    *DiskIO    `yaml:"disk,omitempty"`
	Network *NetworkIO `yaml:"network,omitempty"`
}

// Empty reports whether nothing was declared, so the caller can omit the block.
func (r Resources) Empty() bool {
	return r.CPU == 0 && r.RAM == 0 && r.GPU == 0 && r.Storage == 0 &&
		r.Disk == nil && r.Network == nil
}

type DiskIO struct {
	Read  float64 `yaml:"read,omitempty"`
	Write float64 `yaml:"write,omitempty"`
}

type NetworkIO struct {
	In  float64 `yaml:"in,omitempty"`
	Out float64 `yaml:"out,omitempty"`
}

// Infrastructure is the cluster a plan is made against.
type Infrastructure struct {
	APIVersion string      `yaml:"apiVersion"`
	Kind       string      `yaml:"kind"`
	Nodes      []Node      `yaml:"nodes"`
	DaemonSets []DaemonSet `yaml:"daemonsets,omitempty"`
}

type Node struct {
	Name        string            `yaml:"name"`
	Zone        string            `yaml:"zone,omitempty"`
	Rack        string            `yaml:"rack,omitempty"`
	Labels      map[string]string `yaml:"labels,omitempty"`
	Taints      []Taint           `yaml:"taints,omitempty"`
	Allocatable Resources         `yaml:"allocatable"`
}

type Taint struct {
	Key    string `yaml:"key"`
	Value  string `yaml:"value,omitempty"`
	Effect string `yaml:"effect,omitempty"`
}

type DaemonSet struct {
	Name         string            `yaml:"name"`
	Resources    Resources         `yaml:"resources,omitempty"`
	NodeSelector map[string]string `yaml:"nodeSelector,omitempty"`
	Tolerations  []string          `yaml:"tolerations,omitempty"`
}

// Workloads is what is to be placed, grouped by namespace.
type Workloads struct {
	APIVersion string           `yaml:"apiVersion"`
	Kind       string           `yaml:"kind"`
	Namespaces map[string][]Pod `yaml:"namespaces"`
	Groups     []Group          `yaml:"groups,omitempty"`
}

// Pod is one declaration; `Replicas` of them are unrolled by the solver.
type Pod struct {
	Name           string            `yaml:"name"`
	Replicas       int               `yaml:"replicas,omitempty"`
	Requests       Resources         `yaml:"requests,omitempty"`
	NodeSelector   map[string]string `yaml:"nodeSelector,omitempty"`
	Tolerations    []Toleration      `yaml:"tolerations,omitempty"`
	Affinity       []string          `yaml:"affinity,omitempty"`
	AntiAffinity   []string          `yaml:"antiAffinity,omitempty"`
	TopologySpread *TopologySpread   `yaml:"topologySpread,omitempty"`
	Gang           string            `yaml:"gang,omitempty"`
	Observed       *Observed         `yaml:"observed,omitempty"`
}

type Toleration struct {
	Key      string `yaml:"key"`
	Operator string `yaml:"operator,omitempty"`
	Value    string `yaml:"value,omitempty"`
	Effect   string `yaml:"effect,omitempty"`
}

type TopologySpread struct {
	MaxSkew     int    `yaml:"maxSkew"`
	TopologyKey string `yaml:"topologyKey"`
}

// Group is an explicitly declared gang. See ir-v1.md §3.2.
type Group struct {
	Name         string            `yaml:"name"`
	Members      []string          `yaml:"members"`
	MinMembers   int               `yaml:"min_members,omitempty"`
	Colocate     bool              `yaml:"colocate,omitempty"`
	NodeSelector map[string]string `yaml:"nodeSelector,omitempty"`
}

// Observed is optional steady-state usage. Recorded, not yet packed against.
type Observed struct {
	Window                  string     `yaml:"window,omitempty"`
	P50                     *Resources `yaml:"p50,omitempty"`
	P95                     *Resources `yaml:"p95,omitempty"`
	P99                     *Resources `yaml:"p99,omitempty"`
	Peak                    *Resources `yaml:"peak,omitempty"`
	ExceededRequestFraction float64    `yaml:"exceeded_request_fraction,omitempty"`
}

// Blueprint is the solver's output and the forge's linking input.
type Blueprint struct {
	APIVersion string            `yaml:"apiVersion"`
	Kind       string            `yaml:"kind"`
	Metadata   BlueprintMetadata `yaml:"metadata"`
	Solution   map[string]string `yaml:"solution"`
}

type BlueprintMetadata struct {
	Feasible    bool `yaml:"feasible"`
	Pods        int  `yaml:"pods"`
	ActiveNodes int  `yaml:"activeNodes"`
}

// NewInfrastructure returns an Infrastructure with its header already set.
func NewInfrastructure() *Infrastructure {
	return &Infrastructure{APIVersion: APIVersion, Kind: KindInfrastructure}
}

// NewWorkloads returns a Workloads document with its header already set.
func NewWorkloads() *Workloads {
	return &Workloads{
		APIVersion: APIVersion,
		Kind:       KindWorkloads,
		Namespaces: map[string][]Pod{},
	}
}
