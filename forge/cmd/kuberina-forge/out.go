package main

import (
	"errors"
	"fmt"
	"os"

	"github.com/AlexanderSlokov/kuberina/forge/internal/forgeout"
	"github.com/AlexanderSlokov/kuberina/forge/internal/ir"
	"github.com/AlexanderSlokov/kuberina/forge/internal/k8s"
)

func runOut(args []string) error {
	fs := flagSet("out")
	blueprint := fs.String("blueprint", "", "the solver's blueprint (required)")
	outDir := fs.String("out", "", "directory to write linked manifests into (required)")
	if err := fs.Parse(args); err != nil {
		return err
	}
	if *blueprint == "" || *outDir == "" {
		return errors.New("both --blueprint and --out are required")
	}
	sources, err := sourcesFrom(fs.Args())
	if err != nil {
		return err
	}
	return link(*blueprint, *outDir, sources)
}

// link is the whole of forge-out: read, check, pin, write.
func link(blueprintPath, outDir string, sources []k8s.Source) error {
	bp, err := ir.LoadBlueprint(blueprintPath)
	if err != nil {
		return err
	}
	docs, err := k8s.LoadDocuments(sources, os.Stdin)
	if err != nil {
		return err
	}
	placements, err := forgeout.Link(bp, k8s.Objects(docs))
	if err != nil {
		return err
	}
	rendered, err := forgeout.Render(docs, placements)
	if err != nil {
		return err
	}
	if err := forgeout.WriteAll(outDir, rendered); err != nil {
		return err
	}
	report(outDir, rendered, placements)
	return nil
}

func report(outDir string, rendered []forgeout.Rendered, placements map[string]forgeout.Placement) {
	fmt.Fprintf(os.Stderr, "wrote %d manifest file(s) to %s\n", len(rendered), outDir)
	fmt.Fprint(os.Stderr, forgeout.Summarize(placements))
	fmt.Fprint(os.Stderr,
		"\nEach workload is pinned with a required nodeAffinity on kubernetes.io/hostname.\n"+
			"Review, then apply with kubectl.\n")
}
