package forgeout

import (
	"strings"
	"testing"

	"github.com/AlexanderSlokov/kuberina/forge/internal/ir"
	"github.com/AlexanderSlokov/kuberina/forge/internal/k8s"
)

const manifests = "../../testdata/manifests"

func loadDocs(t *testing.T) []k8s.Document {
	t.Helper()
	docs, err := k8s.LoadDocuments([]k8s.Source{manifests}, nil)
	if err != nil {
		t.Fatalf("loading manifests: %v", err)
	}
	return docs
}

func blueprint(solution map[string]string) *ir.Blueprint {
	return &ir.Blueprint{
		APIVersion: ir.APIVersion,
		Kind:       ir.KindBlueprint,
		Metadata:   ir.BlueprintMetadata{Feasible: true, Pods: len(solution)},
		Solution:   solution,
	}
}

func fullSolution() map[string]string {
	return map[string]string{
		"ai/trainer-0000":       "gpu-a",
		"ai/trainer-0001":       "gpu-b",
		"ai/parameter-server-0": "gpu-a",
	}
}

func TestLinkGroupsReplicasByDeclaration(t *testing.T) {
	placements, err := Link(blueprint(fullSolution()), k8s.Objects(loadDocs(t)))
	if err != nil {
		t.Fatalf("Link: %v", err)
	}
	trainer, ok := placements["ai/trainer"]
	if !ok {
		t.Fatal("trainer was not linked")
	}
	if trainer.Replicas != 2 {
		t.Errorf("replicas = %d, want 2", trainer.Replicas)
	}
	if strings.Join(trainer.Nodes, ",") != "gpu-a,gpu-b" {
		t.Errorf("nodes = %v, want the sorted set [gpu-a gpu-b]", trainer.Nodes)
	}
	if got := placements["ai/parameter-server"].Nodes; len(got) != 1 || got[0] != "gpu-a" {
		t.Errorf("single-replica placement must be exact, got %v", got)
	}
}

func TestLinkRejectsAPodTheManifestsDoNotContain(t *testing.T) {
	solution := fullSolution()
	solution["ai/ghost-0"] = "gpu-a"
	_, err := Link(blueprint(solution), k8s.Objects(loadDocs(t)))
	if err == nil {
		t.Fatal("expected a refusal: the blueprint places a workload that is not here")
	}
	if !strings.Contains(err.Error(), "ai/ghost-0") {
		t.Errorf("error must name the offending entry, got %q", err)
	}
}

func TestLinkRejectsAWorkloadTheBlueprintForgot(t *testing.T) {
	solution := fullSolution()
	delete(solution, "ai/parameter-server-0")
	_, err := Link(blueprint(solution), k8s.Objects(loadDocs(t)))
	if err == nil {
		t.Fatal("expected a refusal: a workload in the manifests was never placed")
	}
	if !strings.Contains(err.Error(), "ai/parameter-server") {
		t.Errorf("error must name what was missed, got %q", err)
	}
}

func TestRenderPinsPlacedWorkloadsAndPreservesEverythingElse(t *testing.T) {
	docs := loadDocs(t)
	placements, err := Link(blueprint(fullSolution()), k8s.Objects(docs))
	if err != nil {
		t.Fatalf("Link: %v", err)
	}
	rendered, err := Render(docs, placements)
	if err != nil {
		t.Fatalf("Render: %v", err)
	}

	all := concat(rendered)
	for _, want := range []string{
		"kubernetes.io/hostname",        // the pin
		"ghcr.io/example/trainer:1.4.2", // an image we never modelled
		"EPOCHS",                        // env we never modelled
		"whenUnsatisfiable",             // a field of a constraint we only partly model
		"kind: Service",                 // an object we ignore entirely
		"podAffinity",                   // the operator's own affinity rule
	} {
		if !strings.Contains(all, want) {
			t.Errorf("rendered output lost %q — the linker must not rewrite what it does not model", want)
		}
	}
}

func TestRenderLeavesUnplacedDocumentsAlone(t *testing.T) {
	docs := loadDocs(t)
	placements, err := Link(blueprint(fullSolution()), k8s.Objects(docs))
	if err != nil {
		t.Fatalf("Link: %v", err)
	}
	rendered, err := Render(docs, placements)
	if err != nil {
		t.Fatalf("Render: %v", err)
	}
	// The node dump carries no pod spec; touching it would be a bug.
	for _, r := range rendered {
		if r.RelPath == "cluster-nodes.yaml" && strings.Contains(string(r.Content), "nodeAffinity") {
			t.Error("forge-out pinned something in the Node dump")
		}
	}
}

func concat(rendered []Rendered) string {
	var b strings.Builder
	for _, r := range rendered {
		b.Write(r.Content)
	}
	return b.String()
}
