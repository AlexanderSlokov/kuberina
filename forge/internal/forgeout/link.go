// Package forgeout renders a solver blueprint back onto Kubernetes manifests.
//
// This is the "forge-out" half of ROADMAP Phase 1 — the linker. It reads the
// blueprint the solver emitted, checks it against the manifests it was planned
// from, and writes manifests that constrain the scheduler to the planned nodes.
//
// # Why node affinity rather than nodeName
//
// A blueprint places pods; a Deployment declares replicas. Kubernetes gives no way
// to say "replica 0 on node A, replica 1 on node B" — `nodeName` exists on a Pod,
// not on a pod template. Rewriting a Deployment into individual pinned Pods would
// produce something applyable but unmanageable: no rollouts, no rescheduling on
// node failure, no controller.
//
// So forge-out emits the honest translation: a required nodeAffinity restricting
// each workload to the set of nodes its replicas were planned onto. For a
// single-replica workload that set has one element and the placement is exact. For a
// larger one, kube-scheduler chooses within the planned set — the plan's capacity
// decision is enforced, and the per-replica assignment is left to the component that
// owns it at runtime. What Kuberina decided is preserved; what it cannot enforce is
// not pretended.
package forgeout

import (
	"fmt"
	"sort"
	"strings"

	"github.com/AlexanderSlokov/kuberina/forge/internal/ir"
	"github.com/AlexanderSlokov/kuberina/forge/internal/k8s"
)

// hostnameLabel is the node label every Kubernetes node carries and that
// nodeAffinity matches on.
const hostnameLabel = "kubernetes.io/hostname"

// Placement is what the blueprint decided for one workload declaration.
type Placement struct {
	// Identity is namespace/name of the declaration, as the IR names it.
	Identity string
	// Nodes are the distinct nodes its replicas were planned onto, sorted.
	Nodes []string
	// Replicas is how many pods the blueprint placed for this declaration.
	Replicas int
}

// Link matches blueprint entries to workload declarations.
//
// Every entry must belong to a declaration present in the manifests, and every
// workload declaration must have been placed. Either mismatch means the blueprint
// and the manifests are not the same plan, which is the one thing a linker must
// never paper over.
func Link(bp *ir.Blueprint, objects []k8s.Object) (map[string]Placement, error) {
	known := workloadIdentities(objects)
	placements := map[string]Placement{}

	for podKey, node := range bp.Solution {
		identity, err := declarationOf(podKey, known)
		if err != nil {
			return nil, err
		}
		p := placements[identity]
		p.Identity = identity
		p.Replicas++
		p.Nodes = append(p.Nodes, node)
		placements[identity] = p
	}
	if err := checkAllPlaced(known, placements); err != nil {
		return nil, err
	}
	return normalize(placements), nil
}

// workloadIdentities indexes the declarations the manifests contain.
func workloadIdentities(objects []k8s.Object) map[string]bool {
	known := map[string]bool{}
	for i := range objects {
		obj := &objects[i]
		if isWorkload(obj.Kind) {
			known[fmt.Sprintf("%s/%s", obj.Namespace(), obj.Metadata.Name)] = true
		}
	}
	return known
}

// declarationOf strips the replica suffix a blueprint key carries.
//
// The solver unrolls `replicas: 4` into `trainer-0000`..`trainer-0003`, and a single
// replica into `trainer-0`, so the declaration is the key with its last `-NNNN`
// segment removed. Resolving against the known set rather than by pattern alone
// keeps a workload legitimately named `web-0` from being mistaken for a suffix.
func declarationOf(podKey string, known map[string]bool) (string, error) {
	if known[podKey] {
		return podKey, nil
	}
	if idx := strings.LastIndex(podKey, "-"); idx > 0 {
		candidate := podKey[:idx]
		if known[candidate] {
			return candidate, nil
		}
	}
	return "", fmt.Errorf(
		"blueprint places %q, which matches no workload in the given manifests; "+
			"the blueprint and the manifests are not the same plan", podKey)
}

func checkAllPlaced(known map[string]bool, placements map[string]Placement) error {
	var missing []string
	for identity := range known {
		if _, ok := placements[identity]; !ok {
			missing = append(missing, identity)
		}
	}
	if len(missing) == 0 {
		return nil
	}
	sort.Strings(missing)
	return fmt.Errorf(
		"the blueprint places nothing for %s; it was planned from a different manifest set",
		strings.Join(missing, ", "))
}

// normalize sorts and de-duplicates the node set of every placement.
func normalize(placements map[string]Placement) map[string]Placement {
	for identity, p := range placements {
		p.Nodes = uniqueSorted(p.Nodes)
		placements[identity] = p
	}
	return placements
}

func uniqueSorted(values []string) []string {
	seen := map[string]bool{}
	out := make([]string, 0, len(values))
	for _, v := range values {
		if !seen[v] {
			seen[v] = true
			out = append(out, v)
		}
	}
	sort.Strings(out)
	return out
}

func isWorkload(kind string) bool {
	switch kind {
	case "Pod", "Deployment", "StatefulSet", "ReplicaSet", "Job":
		return true
	}
	return false
}
