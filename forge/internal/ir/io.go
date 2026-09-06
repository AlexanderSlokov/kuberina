package ir

import (
	"bytes"
	"fmt"
	"os"

	"gopkg.in/yaml.v3"
)

// Write marshals a document to path, creating or truncating it.
func Write(path string, doc any) error {
	body, err := yaml.Marshal(doc)
	if err != nil {
		return fmt.Errorf("serializing %s: %w", path, err)
	}
	if err := os.WriteFile(path, body, 0o644); err != nil {
		return fmt.Errorf("writing %s: %w", path, err)
	}
	return nil
}

// LoadBlueprint reads a blueprint and refuses anything this binary cannot vouch for.
//
// A blueprint drives changes to a live cluster, so every reason to stop is checked
// before the caller can act on it: unknown schema version, wrong kind, a plan the
// solver itself marked infeasible, or a pod count that disagrees with the entries.
func LoadBlueprint(path string) (*Blueprint, error) {
	body, err := os.ReadFile(path)
	if err != nil {
		return nil, fmt.Errorf("reading blueprint: %w", err)
	}
	var bp Blueprint
	dec := yaml.NewDecoder(bytes.NewReader(body))
	dec.KnownFields(true)
	if err := dec.Decode(&bp); err != nil {
		return nil, fmt.Errorf("parsing blueprint %s: %w", path, err)
	}
	if err := bp.validate(path); err != nil {
		return nil, err
	}
	return &bp, nil
}

func (b *Blueprint) validate(path string) error {
	if b.APIVersion != "" && b.APIVersion != APIVersion {
		return fmt.Errorf(
			"%s: unsupported apiVersion %q, this binary implements %q",
			path, b.APIVersion, APIVersion)
	}
	if b.Kind != "" && b.Kind != KindBlueprint {
		return fmt.Errorf("%s: expected kind %q, found %q", path, KindBlueprint, b.Kind)
	}
	if len(b.Solution) == 0 {
		return fmt.Errorf("%s: blueprint carries no placements", path)
	}
	return b.checkIntegrity(path)
}

// checkIntegrity rejects a blueprint whose own header disagrees with its body.
func (b *Blueprint) checkIntegrity(path string) error {
	if !b.Metadata.Feasible && b.Kind != "" {
		return fmt.Errorf(
			"%s: solver marked this plan infeasible; it was written for inspection and must not be applied",
			path)
	}
	if b.Metadata.Pods != 0 && b.Metadata.Pods != len(b.Solution) {
		return fmt.Errorf(
			"%s: metadata.pods is %d but solution has %d entries; the file is truncated or edited",
			path, b.Metadata.Pods, len(b.Solution))
	}
	return nil
}
