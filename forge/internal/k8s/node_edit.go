package k8s

import (
	"fmt"

	"gopkg.in/yaml.v3"
)

// Mapping returns the mapping node inside a decoded document.
func Mapping(doc *yaml.Node) (*yaml.Node, error) {
	node := doc
	if node.Kind == yaml.DocumentNode {
		if len(node.Content) == 0 {
			return nil, fmt.Errorf("empty document at line %d", node.Line)
		}
		node = node.Content[0]
	}
	if node.Kind != yaml.MappingNode {
		return nil, fmt.Errorf("expected a mapping at line %d, got %v", node.Line, node.Kind)
	}
	return node, nil
}

// Get returns the value node for key, or nil when the key is absent.
func Get(mapping *yaml.Node, key string) *yaml.Node {
	for i := 0; i+1 < len(mapping.Content); i += 2 {
		if mapping.Content[i].Value == key {
			return mapping.Content[i+1]
		}
	}
	return nil
}

// GetOrCreateMapping returns the mapping under key, creating an empty one if needed.
func GetOrCreateMapping(mapping *yaml.Node, key string) (*yaml.Node, error) {
	if existing := Get(mapping, key); existing != nil {
		if existing.Kind != yaml.MappingNode {
			return nil, fmt.Errorf("expected %q to be a mapping at line %d", key, existing.Line)
		}
		return existing, nil
	}
	created := &yaml.Node{Kind: yaml.MappingNode, Tag: "!!map"}
	Set(mapping, key, created)
	return created, nil
}

// Set writes key, replacing an existing entry in place so field order is stable.
func Set(mapping *yaml.Node, key string, value *yaml.Node) {
	for i := 0; i+1 < len(mapping.Content); i += 2 {
		if mapping.Content[i].Value == key {
			mapping.Content[i+1] = value
			return
		}
	}
	mapping.Content = append(mapping.Content,
		&yaml.Node{Kind: yaml.ScalarNode, Tag: "!!str", Value: key}, value)
}

// NodeFrom marshals a Go value into a node tree that can be spliced into a document.
func NodeFrom(value any) (*yaml.Node, error) {
	var node yaml.Node
	if err := node.Encode(value); err != nil {
		return nil, fmt.Errorf("building YAML node: %w", err)
	}
	return &node, nil
}

// PodSpecNode returns the pod spec a scheduler acts on: `spec` for a bare Pod,
// `spec.template.spec` for anything with a pod template.
func PodSpecNode(doc *yaml.Node) (*yaml.Node, error) {
	root, err := Mapping(doc)
	if err != nil {
		return nil, err
	}
	spec := Get(root, "spec")
	if spec == nil {
		return nil, fmt.Errorf("object has no spec at line %d", root.Line)
	}
	template := Get(spec, "template")
	if template == nil {
		return spec, nil
	}
	podSpec := Get(template, "spec")
	if podSpec == nil {
		return nil, fmt.Errorf("spec.template has no spec at line %d", template.Line)
	}
	return podSpec, nil
}
