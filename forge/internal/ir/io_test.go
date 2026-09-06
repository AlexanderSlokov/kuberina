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
