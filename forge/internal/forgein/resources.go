package forgein

import (
	"fmt"
	"strings"

	"github.com/AlexanderSlokov/kuberina/forge/internal/ir"
	"github.com/AlexanderSlokov/kuberina/forge/internal/k8s"
	"github.com/AlexanderSlokov/kuberina/forge/internal/quantity"
)

// Well-known Kubernetes labels Kuberina reads structurally.
const (
	labelZone     = "topology.kubernetes.io/zone"
	labelRack     = "topology.kubernetes.io/rack"
	labelRackAlt  = "kuberina.io/rack"
	labelHostname = "kubernetes.io/hostname"
)

// Kubernetes resource names Kuberina maps onto its eight dimensions.
//
// Disk and network throughput have no Kubernetes equivalent — the API has no
// concept of reserved IOPS or bandwidth — so those two pairs of dimensions are left
// unset here and must be supplied by the operator if they bind. On the MSC Irina
// benchmark disk write is the binding dimension, which is precisely why the gap is
// documented rather than quietly zeroed.
const (
	resCPU              = "cpu"
	resMemory           = "memory"
	resEphemeralStorage = "ephemeral-storage"
	gpuSuffix           = "/gpu"
)

// resourcesFrom maps a Kubernetes quantity map onto the IR resource block.
func resourcesFrom(quantities map[string]k8s.Quantity, owner string) (ir.Resources, error) {
	var out ir.Resources
	for name, raw := range quantities {
		if err := assignDimension(&out, name, string(raw)); err != nil {
			return ir.Resources{}, fmt.Errorf("%s: %w", owner, err)
		}
	}
	return out, nil
}

// assignDimension writes one Kubernetes resource into the dimension it belongs to.
func assignDimension(out *ir.Resources, name, raw string) error {
	switch {
	case name == resCPU:
		return assign(&out.CPU, quantity.CPU, raw)
	case name == resMemory:
		return assign(&out.RAM, quantity.Memory, raw)
	case name == resEphemeralStorage:
		return assign(&out.Storage, quantity.Memory, raw)
	case isGPU(name):
		return assign(&out.GPU, quantity.CPU, raw)
	}
	return nil // pods, hugepages, extended resources Kuberina does not model
}

func assign(dst *float64, parse func(string) (float64, error), raw string) error {
	value, err := parse(raw)
	if err != nil {
		return err
	}
	*dst += value
	return nil
}

// isGPU recognizes vendor GPU resources: nvidia.com/gpu, amd.com/gpu, and kin.
func isGPU(name string) bool {
	return strings.HasSuffix(name, gpuSuffix)
}

// rackOf reads a rack label, preferring the upstream key over Kuberina's own.
func rackOf(labels map[string]string) string {
	if rack := labels[labelRack]; rack != "" {
		return rack
	}
	return labels[labelRackAlt]
}
