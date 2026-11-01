package forgein

import (
	"math"
	"strings"
	"testing"

	"github.com/AlexanderSlokov/kuberina/forge/internal/ir"
	"github.com/AlexanderSlokov/kuberina/forge/internal/k8s"
)

const manifests = "../../testdata/manifests"

func convertTestdata(t *testing.T) *Result {
	t.Helper()
	objects, err := k8s.Load([]k8s.Source{manifests}, nil)
	if err != nil {
		t.Fatalf("loading manifests: %v", err)
	}
	result, err := Convert(objects)
	if err != nil {
		t.Fatalf("converting: %v", err)
	}
	return result
}

func TestNodesCarryCapacityInIRUnits(t *testing.T) {
	res := convertTestdata(t)
	if len(res.Infra.Nodes) != 3 {
		t.Fatalf("got %d nodes, want 3", len(res.Infra.Nodes))
	}
	gpuA := res.Infra.Nodes[0]
	if gpuA.Name != "gpu-a" {
		t.Fatalf("nodes must be sorted by name, got %q first", gpuA.Name)
	}
	assertClose(t, "gpu-a cpu", gpuA.Allocatable.CPU, 64)
	assertClose(t, "gpu-a ram (256Gi → GiB)", gpuA.Allocatable.RAM, 256)
	assertClose(t, "gpu-a gpu (nvidia.com/gpu)", gpuA.Allocatable.GPU, 8)
	assertClose(t, "gpu-a storage (2Ti → GiB)", gpuA.Allocatable.Storage, 2048)
	if gpuA.Zone != "az-1" {
		t.Errorf("zone must come from the well-known label, got %q", gpuA.Zone)
	}
	if len(gpuA.Taints) != 1 || gpuA.Taints[0].Key != "dedicated" {
		t.Errorf("taints were dropped: %+v", gpuA.Taints)
	}

	// 16000m must resolve to 16 cores, not 16000.
	assertClose(t, "std-a cpu (16000m)", res.Infra.Nodes[2].Allocatable.CPU, 16)
}

func TestDaemonSetsGoToInfrastructureNotWorkloads(t *testing.T) {
	res := convertTestdata(t)
	if len(res.Infra.DaemonSets) != 1 {
		t.Fatalf("got %d daemonsets, want 1", len(res.Infra.DaemonSets))
	}
	ds := res.Infra.DaemonSets[0]
	assertClose(t, "node-exporter cpu (100m)", ds.Resources.CPU, 0.1)
	assertClose(t, "node-exporter ram (128Mi)", ds.Resources.RAM, 0.125)

	for _, pods := range res.Workloads.Namespaces {
		for _, p := range pods {
			if p.Name == "node-exporter" {
				t.Fatal("a DaemonSet must be pre-deducted, not placed")
			}
		}
	}
}

func TestContainerRequestsAreSummed(t *testing.T) {
	trainer := findPod(t, convertTestdata(t), "ai", "trainer")
	// 8 + 250m CPU, 32Gi + 512Mi RAM, across two containers.
	assertClose(t, "trainer cpu", trainer.Requests.CPU, 8.25)
	assertClose(t, "trainer ram", trainer.Requests.RAM, 32.5)
	assertClose(t, "trainer gpu", trainer.Requests.GPU, 1)
	if trainer.Replicas != 2 {
		t.Errorf("replicas = %d, want 2", trainer.Replicas)
	}
}

func TestSchedulingConstraintsSurvive(t *testing.T) {
	trainer := findPod(t, convertTestdata(t), "ai", "trainer")
	if trainer.NodeSelector["accelerator"] != "h100" {
		t.Errorf("nodeSelector lost: %+v", trainer.NodeSelector)
	}
	if len(trainer.Tolerations) != 1 || trainer.Tolerations[0].Effect != "NoSchedule" {
		t.Errorf("tolerations lost: %+v", trainer.Tolerations)
	}
	if trainer.TopologySpread == nil || trainer.TopologySpread.MaxSkew != 1 {
		t.Errorf("topologySpread lost: %+v", trainer.TopologySpread)
	}
}

func TestPodAffinityResolvesToIRIdentities(t *testing.T) {
	ps := findPod(t, convertTestdata(t), "ai", "parameter-server")
	if len(ps.Affinity) != 1 || ps.Affinity[0] != "ai/trainer" {
		t.Fatalf("podAffinity must resolve to namespace/name, got %v", ps.Affinity)
	}
}

func TestUnmodelledKindsAreIgnored(t *testing.T) {
	res := convertTestdata(t)
	if got := len(res.Workloads.Namespaces["ai"]); got != 2 {
		t.Fatalf("ai namespace has %d declarations, want 2 (the Service is not one)", got)
	}
}

func TestMatchExpressionsAreWarnedAboutNotSilentlyDropped(t *testing.T) {
	yaml := `
apiVersion: apps/v1
kind: Deployment
metadata: { name: web, namespace: shop }
spec:
  template:
    spec:
      affinity:
        podAntiAffinity:
          requiredDuringSchedulingIgnoredDuringExecution:
            - labelSelector:
                matchExpressions:
                  - { key: app, operator: In, values: [web] }
              topologyKey: kubernetes.io/hostname
      containers:
        - name: web
          resources: { requests: { cpu: "1" } }
`
	objects, err := k8s.Load([]k8s.Source{"-"}, strings.NewReader(yaml))
	if err != nil {
		t.Fatalf("loading: %v", err)
	}
	res, err := Convert(objects)
	if err != nil {
		t.Fatalf("converting: %v", err)
	}
	if len(res.Warnings) != 1 || !strings.Contains(res.Warnings[0], "matchExpressions") {
		t.Fatalf("expected a warning naming matchExpressions, got %v", res.Warnings)
	}
}

func TestConversionIsDeterministic(t *testing.T) {
	first, second := convertTestdata(t), convertTestdata(t)
	if first.Infra.Nodes[0].Name != second.Infra.Nodes[0].Name {
		t.Fatal("node order is not stable across runs")
	}
	for ns, pods := range first.Workloads.Namespaces {
		for i, p := range pods {
			if second.Workloads.Namespaces[ns][i].Name != p.Name {
				t.Fatalf("pod order in %s is not stable across runs", ns)
			}
		}
	}
}

func findPod(t *testing.T, res *Result, namespace, name string) ir.Pod {
	t.Helper()
	for _, p := range res.Workloads.Namespaces[namespace] {
		if p.Name == name {
			return p
		}
	}
	t.Fatalf("no pod %s/%s in the converted IR", namespace, name)
	return ir.Pod{}
}

func assertClose(t *testing.T, what string, got, want float64) {
	t.Helper()
	if math.Abs(got-want) > 1e-9 {
		t.Errorf("%s = %v, want %v", what, got, want)
	}
}
