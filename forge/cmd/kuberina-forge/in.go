package main

import (
	"errors"
	"fmt"
	"os"

	"github.com/AlexanderSlokov/kuberina/forge/internal/forgein"
	"github.com/AlexanderSlokov/kuberina/forge/internal/ir"
	"github.com/AlexanderSlokov/kuberina/forge/internal/k8s"
)

func runIn(args []string) error {
	fs := flagSet("in")
	outInfra := fs.String("out-infra", "", "write the Infrastructure document here")
	outWorkloads := fs.String("out-workloads", "", "write the Workloads document here")
	if err := fs.Parse(args); err != nil {
		return err
	}
	if *outInfra == "" && *outWorkloads == "" {
		return errors.New("nothing to do: pass --out-infra, --out-workloads, or both")
	}
	sources, err := sourcesFrom(fs.Args())
	if err != nil {
		return err
	}

	result, err := convert(sources)
	if err != nil {
		return err
	}
	reportWarnings(result.Warnings)
	return writeDocuments(result, *outInfra, *outWorkloads)
}

func convert(sources []k8s.Source) (*forgein.Result, error) {
	objects, err := k8s.Load(sources, os.Stdin)
	if err != nil {
		return nil, err
	}
	return forgein.Convert(objects)
}

// writeDocuments writes the requested halves of the IR and says what landed where.
func writeDocuments(result *forgein.Result, infraPath, workloadsPath string) error {
	if infraPath != "" {
		if err := ir.Write(infraPath, result.Infra); err != nil {
			return err
		}
		fmt.Fprintf(os.Stderr, "wrote %s: %d nodes, %d daemonsets\n",
			infraPath, len(result.Infra.Nodes), len(result.Infra.DaemonSets))
	}
	if workloadsPath != "" {
		if err := ir.Write(workloadsPath, result.Workloads); err != nil {
			return err
		}
		fmt.Fprintf(os.Stderr, "wrote %s: %d declarations across %d namespaces\n",
			workloadsPath, countPods(result), len(result.Workloads.Namespaces))
	}
	return nil
}

func countPods(result *forgein.Result) int {
	total := 0
	for _, pods := range result.Workloads.Namespaces {
		total += len(pods)
	}
	return total
}

// reportWarnings prints what was read but could not be fully represented.
//
// WHY on stderr and never silently: a constraint the operator wrote and Kuberina
// dropped without saying so would make the plan wrong in a way nothing detects.
func reportWarnings(warnings []string) {
	if len(warnings) == 0 {
		return
	}
	fmt.Fprintf(os.Stderr, "%d input(s) could not be fully represented in IR v1:\n", len(warnings))
	for _, w := range warnings {
		fmt.Fprintf(os.Stderr, "  - %s\n", w)
	}
}

func sourcesFrom(args []string) ([]k8s.Source, error) {
	if len(args) == 0 {
		return nil, errors.New("no sources given; pass a file, a directory, or \"-\" for stdin")
	}
	sources := make([]k8s.Source, 0, len(args))
	for _, a := range args {
		sources = append(sources, k8s.Source(a))
	}
	return sources, nil
}
