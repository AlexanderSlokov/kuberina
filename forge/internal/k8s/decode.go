package k8s

import (
	"fmt"
	"io"
	"io/fs"
	"path/filepath"
	"sort"
	"strings"
)

// Source is one place manifests come from: a file, a directory, or "-" for stdin.
//
// Stdin is what puts the whole Helm and Kustomize ecosystem in reach without linking
// either of them (ADR-0001): `helm template ./chart | kuberina-forge in -`.
type Source string

// Load reads manifests and returns only the typed view, for callers that do not
// need to write the documents back out.
func Load(sources []Source, stdin io.Reader) ([]Object, error) {
	docs, err := LoadDocuments(sources, stdin)
	if err != nil {
		return nil, err
	}
	return Objects(docs), nil
}

// yamlFilesIn lists the YAML files under dir, in sorted path order.
//
// WHY sorted: the IR and the linked manifests are reviewed artifacts under version
// control. A diff that churns because a directory walk changed order is a diff
// nobody reads.
func yamlFilesIn(dir string) ([]string, error) {
	var paths []string
	err := filepath.WalkDir(dir, func(path string, d fs.DirEntry, err error) error {
		if err != nil {
			return err
		}
		if !d.IsDir() && isYAML(path) {
			paths = append(paths, path)
		}
		return nil
	})
	if err != nil {
		return nil, fmt.Errorf("walking %s: %w", dir, err)
	}
	sort.Strings(paths)
	return paths, nil
}

func isYAML(path string) bool {
	ext := strings.ToLower(filepath.Ext(path))
	return ext == ".yaml" || ext == ".yml"
}
