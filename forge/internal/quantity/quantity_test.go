package quantity

import (
	"math"
	"strings"
	"testing"
)

func TestCPU(t *testing.T) {
	cases := map[string]float64{
		"4":     4,
		"2500m": 2.5,
		"100m":  0.1,
		"0.5":   0.5,
	}
	for in, want := range cases {
		got, err := CPU(in)
		if err != nil {
			t.Fatalf("CPU(%q): %v", in, err)
		}
		if math.Abs(got-want) > 1e-9 {
			t.Errorf("CPU(%q) = %v, want %v", in, got, want)
		}
	}
}

func TestMemoryToGiB(t *testing.T) {
	cases := map[string]float64{
		"512Mi":      0.5,
		"2Gi":        2,
		"1Ti":        1024,
		"1073741824": 1, // bare bytes
	}
	for in, want := range cases {
		got, err := Memory(in)
		if err != nil {
			t.Fatalf("Memory(%q): %v", in, err)
		}
		if math.Abs(got-want) > 1e-9 {
			t.Errorf("Memory(%q) = %v, want %v", in, got, want)
		}
	}
}

func TestThroughputToMBps(t *testing.T) {
	got, err := Throughput("5G")
	if err != nil {
		t.Fatalf("Throughput: %v", err)
	}
	if math.Abs(got-5000) > 1e-9 {
		t.Errorf("Throughput(5G) = %v, want 5000", got)
	}
}

func TestRejectsGarbageAndNamesTheValue(t *testing.T) {
	if _, err := CPU("banana"); err == nil {
		t.Fatal("expected an error for a non-numeric CPU quantity")
	} else if !strings.Contains(err.Error(), "banana") {
		t.Errorf("error must name the offending value, got %q", err)
	}
	if _, err := Memory("12Xi"); err == nil {
		t.Fatal("expected an error for an unknown suffix")
	}
}
