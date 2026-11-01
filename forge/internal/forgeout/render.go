package forgeout

import (
	"bytes"
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"

	"gopkg.in/yaml.v3"

	"github.com/AlexanderSlokov/kuberina/forge/internal/k8s"
)

// nodeAffinity is the fragment forge-out injects. Its shape is Kubernetes', not
// Kuberina's, because kube-scheduler is the consumer.
type nodeAffinity struct {
	Required nodeSelector `yaml:"requiredDuringSchedulingIgnoredDuringExecution"`
}

type nodeSelector struct {
	Terms []nodeSelectorTerm `yaml:"nodeSelectorTerms"`
}

type nodeSelectorTerm struct {
	MatchExpressions []matchExpression `yaml:"matchExpressions"`
}

type matchExpression struct {
	Key      string   `yaml:"key"`
	Operator string   `yaml:"operator"`
	Values   []string `yaml:"values"`
}

// Rendered is one linked manifest stream, ready to be written or printed.
type Rendered struct {
	// RelPath is where the stream should be written, relative to the output root.
	RelPath string
	Content []byte
}

// Render rewrites every document, pinning placed workloads to their planned nodes.
//
// Documents Kuberina did not place — Services, ConfigMaps, and anything else — pass
// through untouched, so the output is a drop-in replacement for the input.
func Render(docs []k8s.Document, placements map[string]Placement) ([]Rendered, error) {
	byFile := map[string][]*yaml.Node{}
	var order []string

	for i := range docs {
		doc := &docs[i]
		if err := pinDocument(doc, placements); err != nil {
			return nil, err
		}
		if _, seen := byFile[doc.Source]; !seen {
			order = append(order, doc.Source)
		}
		byFile[doc.Source] = append(byFile[doc.Source], doc.Node)
	}
	return encodeStreams(order, byFile)
}

// pinDocument injects node affinity into one document, when it was placed.
func pinDocument(doc *k8s.Document, placements map[string]Placement) error {
	identity := fmt.Sprintf("%s/%s", doc.Object.Namespace(), doc.Object.Metadata.Name)
	placement, ok := placements[identity]
	if !ok || !isWorkload(doc.Object.Kind) {
		return nil
	}
	podSpec, err := k8s.PodSpecNode(doc.Node)
	if err != nil {
		return fmt.Errorf("%s: %w", identity, err)
	}
	return pinPodSpec(podSpec, placement)
}

// pinPodSpec sets spec.affinity.nodeAffinity, leaving pod affinity rules alone.
func pinPodSpec(podSpec *yaml.Node, placement Placement) error {
	affinity, err := k8s.GetOrCreateMapping(podSpec, "affinity")
	if err != nil {
		return err
	}
	fragment, err := k8s.NodeFrom(affinityFor(placement))
	if err != nil {
		return err
	}
	k8s.Set(affinity, "nodeAffinity", fragment)
	return nil
}

func affinityFor(placement Placement) nodeAffinity {
	return nodeAffinity{
		Required: nodeSelector{
			Terms: []nodeSelectorTerm{{
				MatchExpressions: []matchExpression{{
					Key:      hostnameLabel,
					Operator: "In",
					Values:   placement.Nodes,
				}},
			}},
		},
	}
}

// encodeStreams re-serializes each source file as one multi-document stream.
func encodeStreams(order []string, byFile map[string][]*yaml.Node) ([]Rendered, error) {
	out := make([]Rendered, 0, len(order))
	for _, source := range order {
		content, err := encodeStream(byFile[source])
		if err != nil {
			return nil, fmt.Errorf("%s: %w", source, err)
		}
		out = append(out, Rendered{RelPath: outputName(source), Content: content})
	}
	sort.Slice(out, func(i, j int) bool { return out[i].RelPath < out[j].RelPath })
	return out, nil
}

func encodeStream(nodes []*yaml.Node) ([]byte, error) {
	var buf bytes.Buffer
	enc := yaml.NewEncoder(&buf)
	enc.SetIndent(2)
	for _, n := range nodes {
		if err := enc.Encode(n); err != nil {
			return nil, err
		}
	}
	if err := enc.Close(); err != nil {
		return nil, err
	}
	return buf.Bytes(), nil
}

// outputName keeps the input's file name, flattening directories into one level.
func outputName(source string) string {
	if source == "<stdin>" {
		return "stdin.yaml"
	}
	return filepath.Base(source)
}

// WriteAll writes rendered streams under dir, creating it if necessary.
func WriteAll(dir string, rendered []Rendered) error {
	if err := os.MkdirAll(dir, 0o755); err != nil {
		return fmt.Errorf("creating %s: %w", dir, err)
	}
	for _, r := range rendered {
		path := filepath.Join(dir, r.RelPath)
		if err := os.WriteFile(path, r.Content, 0o644); err != nil {
			return fmt.Errorf("writing %s: %w", path, err)
		}
	}
	return nil
}

// Summarize describes what was pinned, for the operator reading the console.
func Summarize(placements map[string]Placement) string {
	identities := make([]string, 0, len(placements))
	for id := range placements {
		identities = append(identities, id)
	}
	sort.Strings(identities)

	var b strings.Builder
	for _, id := range identities {
		p := placements[id]
		fmt.Fprintf(&b, "  %s → %d replica(s) on %s\n",
			id, p.Replicas, strings.Join(p.Nodes, ", "))
	}
	return b.String()
}
