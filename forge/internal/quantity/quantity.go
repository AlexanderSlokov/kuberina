// Package quantity parses Kubernetes resource quantities into the units Kuberina IR
// uses: CPU in cores, memory and storage in GiB, throughput in MB/s.
//
// It deliberately mirrors solver/src/quantity.rs rather than importing
// k8s.io/apimachinery. ADR-0001 keeps the k8s.io dependency tree out of this
// repository; the arithmetic here is thirty lines and the two implementations are
// checked against the same table in docs/references/ir-v1.md §1.
package quantity

import (
	"fmt"
	"strconv"
	"strings"
)

const bytesPerGiB = 1024 * 1024 * 1024

// binarySuffixes are the IEC suffixes Kubernetes accepts, in bytes.
var binarySuffixes = map[string]float64{
	"Ki": 1 << 10,
	"Mi": 1 << 20,
	"Gi": 1 << 30,
	"Ti": 1 << 40,
	"Pi": 1 << 50,
	"Ei": 1 << 60,
}

// decimalSuffixes are the SI suffixes Kubernetes accepts, in base units.
var decimalSuffixes = map[string]float64{
	"k": 1e3, "K": 1e3,
	"M": 1e6,
	"G": 1e9,
	"T": 1e12,
	"P": 1e15,
	"E": 1e18,
}

// CPU parses a CPU quantity into cores. "2500m" is 2.5, "4" is 4.
func CPU(s string) (float64, error) {
	s = strings.TrimSpace(s)
	if after, ok := strings.CutSuffix(s, "m"); ok {
		milli, err := strconv.ParseFloat(after, 64)
		if err != nil {
			return 0, fmt.Errorf("cpu quantity %q: %w", s, err)
		}
		return milli / 1000, nil
	}
	return plain(s, "cpu")
}

// Memory parses a memory or storage quantity into GiB. "512Mi" is 0.5.
func Memory(s string) (float64, error) {
	bytes, err := bytesOf(s)
	if err != nil {
		return 0, fmt.Errorf("memory quantity %q: %w", s, err)
	}
	return bytes / bytesPerGiB, nil
}

// Throughput parses a throughput quantity into MB/s. "5G" is 5000.
func Throughput(s string) (float64, error) {
	base, err := bytesOf(s)
	if err != nil {
		return 0, fmt.Errorf("throughput quantity %q: %w", s, err)
	}
	return base / 1e6, nil
}

// bytesOf resolves any suffixed quantity to base units, defaulting to bare bytes.
func bytesOf(s string) (float64, error) {
	s = strings.TrimSpace(s)
	if len(s) > 2 {
		if mult, ok := binarySuffixes[s[len(s)-2:]]; ok {
			return scaled(s[:len(s)-2], mult)
		}
	}
	if len(s) > 1 {
		if mult, ok := decimalSuffixes[s[len(s)-1:]]; ok {
			return scaled(s[:len(s)-1], mult)
		}
	}
	return plain(s, "quantity")
}

func scaled(digits string, mult float64) (float64, error) {
	n, err := strconv.ParseFloat(strings.TrimSpace(digits), 64)
	if err != nil {
		return 0, fmt.Errorf("expected a number before the suffix, got %q", digits)
	}
	return n * mult, nil
}

func plain(s, field string) (float64, error) {
	n, err := strconv.ParseFloat(s, 64)
	if err != nil {
		return 0, fmt.Errorf("%s: expected a number or a Kubernetes quantity, got %q", field, s)
	}
	return n, nil
}
