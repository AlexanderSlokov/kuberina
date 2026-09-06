package forgein

import (
	"fmt"
	"sort"

	"github.com/AlexanderSlokov/kuberina/forge/internal/k8s"
)

// declaration is one workload and the labels its pods will carry, which is what a
// podAffinity selector actually selects on.
type declaration struct {
	identity string // namespace/name, the identity the IR uses everywhere
	labels   map[string]string
}

// collectDeclarations indexes every workload so affinity selectors can be resolved
// against real targets rather than left as opaque label queries.
func collectDeclarations(objects []k8s.Object) []declaration {
	var out []declaration
	for i := range objects {
		obj := &objects[i]
		if !workloadKinds[obj.Kind] {
			continue
		}
		out = append(out, declaration{
			identity: fmt.Sprintf("%s/%s", obj.Namespace(), obj.Metadata.Name),
			labels:   obj.PodLabels(),
		})
	}
	sort.Slice(out, func(i, j int) bool { return out[i].identity < out[j].identity })
	return out
}

// resolveAffinity turns podAffinity and podAntiAffinity into IR target lists.
//
// Kubernetes expresses affinity as a label query; the IR names the pods directly,
// because the solver evaluates affinity as a soft term over identities it already
// has. Resolving the query here — once, against the same manifest set — is what
// makes the two representations line up.
//
// Returns the affinity targets, the anti-affinity targets, and warnings for any
// term that could not be resolved.
func resolveAffinity(
	obj *k8s.Object,
	spec k8s.PodSpec,
	declarations []declaration,
) (affinity, anti []string, warnings []string) {
	if spec.Affinity == nil {
		return nil, nil, nil
	}
	self := fmt.Sprintf("%s/%s", obj.Namespace(), obj.Metadata.Name)
	affinity, w1 := resolveTerms(spec.Affinity.PodAffinity, self, declarations, "podAffinity")
	anti, w2 := resolveTerms(spec.Affinity.PodAntiAffinity, self, declarations, "podAntiAffinity")
	return affinity, anti, append(w1, w2...)
}

func resolveTerms(
	terms *k8s.PodAffinityTerms,
	self string,
	declarations []declaration,
	field string,
) (targets []string, warnings []string) {
	if terms == nil {
		return nil, nil
	}
	all := terms.Required
	for _, weighted := range terms.Preferred {
		all = append(all, weighted.Term)
	}
	for _, term := range all {
		matched, warning := matchTerm(term, self, declarations, field)
		targets = append(targets, matched...)
		if warning != "" {
			warnings = append(warnings, warning)
		}
	}
	return dedupe(targets), warnings
}

// matchTerm resolves one term, or explains why it could not be resolved.
func matchTerm(
	term k8s.PodAffinityTerm,
	self string,
	declarations []declaration,
	field string,
) (targets []string, warning string) {
	if term.LabelSelector == nil {
		return nil, fmt.Sprintf("%s: %s term has no labelSelector and was skipped", self, field)
	}
	if len(term.LabelSelector.MatchExpressions) > 0 {
		return nil, fmt.Sprintf(
			"%s: %s uses matchExpressions, which Kuberina IR v1 cannot express; only matchLabels was resolved",
			self, field)
	}
	if len(term.LabelSelector.MatchLabels) == 0 {
		return nil, fmt.Sprintf("%s: %s selector is empty and was skipped", self, field)
	}
	for _, d := range declarations {
		if d.identity != self && matchesLabels(d.labels, term.LabelSelector.MatchLabels) {
			targets = append(targets, d.identity)
		}
	}
	if len(targets) == 0 {
		return nil, fmt.Sprintf("%s: %s selector matched no workload in this input", self, field)
	}
	return targets, ""
}

// matchesLabels reports whether every selector key matches, as Kubernetes requires.
func matchesLabels(labels, selector map[string]string) bool {
	for k, v := range selector {
		if labels[k] != v {
			return false
		}
	}
	return true
}

func dedupe(values []string) []string {
	if len(values) == 0 {
		return nil
	}
	seen := make(map[string]bool, len(values))
	var out []string
	for _, v := range values {
		if !seen[v] {
			seen[v] = true
			out = append(out, v)
		}
	}
	sort.Strings(out)
	return out
}
