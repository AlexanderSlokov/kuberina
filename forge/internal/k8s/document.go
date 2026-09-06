package k8s

import (
	"errors"
	"fmt"
	"io"
	"os"

	"gopkg.in/yaml.v3"
)

// Document is one manifest, kept both as the typed subset Kuberina reads and as the
// original node tree.
//
// WHY both: forge-in only needs the typed view, but forge-out has to write the
// manifest back out. Re-marshalling the typed view would delete every field
// Kuberina does not model — images, env, ports, probes — so the linker edits the
// node tree instead and the original survives, comments included.
type Document struct {
	Source string
	Index  int
	Node   *yaml.Node
	Object Object
}

// LoadDocuments reads every manifest named by the sources, in deterministic order.
func LoadDocuments(sources []Source, stdin io.Reader) ([]Document, error) {
	var docs []Document
	for _, src := range sources {
		batch, err := documentsFrom(src, stdin)
		if err != nil {
			return nil, err
		}
		docs = append(docs, batch...)
	}
	if len(docs) == 0 {
		return nil, errors.New("no Kubernetes objects found in the given sources")
	}
	return docs, nil
}

// Objects projects documents down to the typed view forge-in works with.
func Objects(docs []Document) []Object {
	out := make([]Object, 0, len(docs))
	for _, d := range docs {
		out = append(out, d.Object)
	}
	return out
}

func documentsFrom(src Source, stdin io.Reader) ([]Document, error) {
	if src == "-" {
		return decodeDocuments(stdin, "<stdin>")
	}
	info, err := os.Stat(string(src))
	if err != nil {
		return nil, fmt.Errorf("cannot read %s: %w", src, err)
	}
	if info.IsDir() {
		return documentsFromDir(string(src))
	}
	return documentsFromFile(string(src))
}

func documentsFromDir(dir string) ([]Document, error) {
	paths, err := yamlFilesIn(dir)
	if err != nil {
		return nil, err
	}
	var docs []Document
	for _, p := range paths {
		batch, err := documentsFromFile(p)
		if err != nil {
			return nil, err
		}
		docs = append(docs, batch...)
	}
	return docs, nil
}

func documentsFromFile(path string) ([]Document, error) {
	f, err := os.Open(path)
	if err != nil {
		return nil, fmt.Errorf("cannot read %s: %w", path, err)
	}
	defer f.Close()
	return decodeDocuments(f, path)
}

// decodeDocuments reads a multi-document stream, keeping node and typed view aligned.
func decodeDocuments(r io.Reader, source string) ([]Document, error) {
	dec := yaml.NewDecoder(r)
	var docs []Document
	for i := 0; ; i++ {
		var node yaml.Node
		err := dec.Decode(&node)
		if errors.Is(err, io.EOF) {
			return docs, nil
		}
		if err != nil {
			return nil, fmt.Errorf("%s: document %d: %w", source, i, err)
		}
		doc, ok, err := documentFrom(&node, source, i)
		if err != nil {
			return nil, err
		}
		if ok {
			docs = append(docs, doc)
		}
	}
}

func documentFrom(node *yaml.Node, source string, index int) (Document, bool, error) {
	var obj Object
	if err := node.Decode(&obj); err != nil {
		return Document{}, false, fmt.Errorf("%s: document %d: %w", source, index, err)
	}
	if obj.Kind == "" {
		return Document{}, false, nil // an empty document or a stray separator
	}
	return Document{Source: source, Index: index, Node: node, Object: obj}, true, nil
}
