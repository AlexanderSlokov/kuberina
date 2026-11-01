package ir

import (
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func writeBlueprint(t *testing.T, body string) string {
	t.Helper()
	path := filepath.Join(t.TempDir(), "blueprint.yaml")
	if err := os.WriteFile(path, []byte(body), 0o644); err != nil {
		t.Fatalf("writing fixture: %v", err)
	}
	return path
}

const feasible = `apiVersion: kuberina.io/v1
kind: Blueprint
metadata:
  feasible: true
  pods: 2
  activeNodes: 1
solution:
  ai/trainer-0000: gpu-a
  ai/trainer-0001: gpu-a
`

func TestLoadBlueprintAcceptsAFeasiblePlan(t *testing.T) {
	bp, err := LoadBlueprint(writeBlueprint(t, feasible))
	if err != nil {
		t.Fatalf("LoadBlueprint: %v", err)
	}
	if len(bp.Solution) != 2 || bp.Solution["ai/trainer-0000"] != "gpu-a" {
		t.Errorf("solution not read: %+v", bp.Solution)
	}
}

func TestLoadBlueprintRefusesAnInfeasiblePlan(t *testing.T) {
	body := strings.Replace(feasible, "feasible: true", "feasible: false", 1)
	_, err := LoadBlueprint(writeBlueprint(t, body))
	if err == nil {
		t.Fatal("an infeasible plan must never be linked")
	}
	if !strings.Contains(err.Error(), "infeasible") {
		t.Errorf("error must say why, got %q", err)
	}
}

func TestLoadBlueprintRefusesAnUnknownApiVersion(t *testing.T) {
	body := strings.Replace(feasible, APIVersion, "kuberina.io/v2", 1)
	_, err := LoadBlueprint(writeBlueprint(t, body))
	if err == nil {
		t.Fatal("a future schema must be refused, not guessed at")
	}
	if !strings.Contains(err.Error(), "kuberina.io/v2") {
		t.Errorf("error must name the offending version, got %q", err)
	}
}

func TestLoadBlueprintDetectsATruncatedFile(t *testing.T) {
	body := strings.Replace(feasible, "  ai/trainer-0001: gpu-a\n", "", 1)
	_, err := LoadBlueprint(writeBlueprint(t, body))
	if err == nil {
		t.Fatal("metadata.pods disagreeing with the body must be caught")
	}
	if !strings.Contains(err.Error(), "truncated") {
		t.Errorf("error must explain the mismatch, got %q", err)
	}
}

func TestLoadBlueprintRejectsUnknownFields(t *testing.T) {
	body := feasible + "surprise: yes\n"
	if _, err := LoadBlueprint(writeBlueprint(t, body)); err == nil {
		t.Fatal("IR documents are closed; an unknown key must fail the parse")
	}
}

// The per-device GPU fields (ir-v1.md §2 and §3.4, X-3) serialize under their IR
// names and disappear when unset, so documents without them stay byte-identical.
func TestGpuDeviceFieldsSerializeUnderTheirIrNames(t *testing.T) {
	path := filepath.Join(t.TempDir(), "doc.yaml")
	doc := struct {
		Node Node `yaml:"node"`
		Pod  Pod  `yaml:"pod"`
	}{
		Node: Node{Name: "gpu-node-01", GPUDevices: 8},
		Pod:  Pod{Name: "inference", Requests: Resources{GPU: 1}, GPUMilli: 500},
	}
	if err := Write(path, doc); err != nil {
		t.Fatalf("Write: %v", err)
	}
	body, _ := os.ReadFile(path)
	for _, want := range []string{"gpuDevices: 8", "gpuMilli: 500"} {
		if !strings.Contains(string(body), want) {
			t.Errorf("missing %q in:\n%s", want, body)
		}
	}
}

func TestUnsetGpuDeviceFieldsAreOmitted(t *testing.T) {
	path := filepath.Join(t.TempDir(), "doc.yaml")
	if err := Write(path, struct {
		Node Node
		Pod  Pod
	}{Node{Name: "n"}, Pod{Name: "p"}}); err != nil {
		t.Fatalf("Write: %v", err)
	}
	body, _ := os.ReadFile(path)
	if strings.Contains(string(body), "gpu") {
		t.Errorf("unset GPU fields must be omitted, got:\n%s", body)
	}
}

// A blueprint that declines Pods serves a benchmark (ir-v1.md §4.1, X-4). The forge
// must read it, so that the refusal names the reason, and must not link it.
func TestLoadBlueprintRefusesAPlanWithUnplacedPods(t *testing.T) {
	body := strings.Replace(feasible, "  activeNodes: 1\n",
		"  activeNodes: 1\n  admission: in-order\n  unplaced: 1\n", 1) + "unplaced:\n  - ai/trainer-0002\n"
	_, err := LoadBlueprint(writeBlueprint(t, body))
	if err == nil {
		t.Fatal("a plan with unplaced Pods must never be linked")
	}
	if !strings.Contains(err.Error(), "unplaced") || !strings.Contains(err.Error(), "1 Pod") {
		t.Errorf("error must say why and how many, got %q", err)
	}
}

func TestLoadBlueprintAcceptsAdmissionMetadataWithNothingUnplaced(t *testing.T) {
	body := strings.Replace(feasible, "  activeNodes: 1\n",
		"  activeNodes: 1\n  admission: free\n  unplaced: 0\n", 1) + "unplaced: []\n"
	if _, err := LoadBlueprint(writeBlueprint(t, body)); err != nil {
		t.Fatalf("a plan that declined nothing is a full plan: %v", err)
	}
}
