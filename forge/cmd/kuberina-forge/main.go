// Command kuberina-forge is the frontend and linker of the Kuberina pipeline.
//
//	kuberina-forge in  --out-infra infra.yaml --out-workloads workloads.yaml ./k8s
//	helm template ./chart | kuberina-forge in --out-workloads workloads.yaml -
//	kuberina-forge out --blueprint kuberina_solution.yaml --out ./planned ./k8s
//
// It is a command, not a controller: it runs, writes files, and exits. See
// docs/explanation/adr/0001-go-component-is-a-cli.md.
package main

import (
	"flag"
	"fmt"
	"os"
)

const usage = `kuberina-forge — Kubernetes manifests in, planned manifests out.

Usage:
  kuberina-forge in  [flags] <source>...   convert manifests to Kuberina IR
  kuberina-forge out [flags] <source>...   apply a blueprint back to manifests

A source is a file, a directory, or "-" for stdin.

Run "kuberina-forge in -h" or "kuberina-forge out -h" for the flags of each.
`

func main() {
	if len(os.Args) < 2 {
		fmt.Fprint(os.Stderr, usage)
		os.Exit(2)
	}
	if err := run(os.Args[1], os.Args[2:]); err != nil {
		fmt.Fprintf(os.Stderr, "kuberina-forge: %v\n", err)
		os.Exit(1)
	}
}

func run(command string, args []string) error {
	switch command {
	case "in":
		return runIn(args)
	case "out":
		return runOut(args)
	case "-h", "--help", "help":
		fmt.Print(usage)
		return nil
	}
	return fmt.Errorf("unknown command %q; expected \"in\" or \"out\"", command)
}

// flagSet builds a subcommand flag set that prints the shared usage on error.
func flagSet(name string) *flag.FlagSet {
	fs := flag.NewFlagSet(name, flag.ExitOnError)
	fs.Usage = func() {
		fmt.Fprintf(os.Stderr, "Usage: kuberina-forge %s [flags] <source>...\n\nFlags:\n", name)
		fs.PrintDefaults()
	}
	return fs
}
