# Kuberina Benchmarks

Testdata generation and formal verification for the MSC Irina benchmark.

| Script | Purpose |
|---|---|
| `gen_irina_testdata.py` | Generates the MSC Irina-scale testbed: 186 nodes, 2,714 pods, 5,128 constraints, 8-dimensional resource vectors |
| `mathematical_proof.py` | Proves feasibility, quality (α against the LP lower bound), and significance (Monte Carlo against random placement) |
| `patch_math_proof.py` | Spent one-shot codemod from the 3D → 8D migration. Retained for provenance; not part of any pipeline |

## Usage

```bash
# Generate the testbed into solver/testdata/
make bench-generate-testdata

# Generate into research/testdata/ for the Python reference implementation
make research-generate-testdata

# Run the formal proof against solver/testdata + solver/kuberina_solution.yaml
make bench-proof
```

## Output paths

`gen_irina_testdata.py` writes to `testdata/` **relative to the current working
directory**, which is why the two generation targets differ only by the directory
they `cd` into. `mathematical_proof.py` resolves its inputs relative to the
repository root regardless of where it is invoked from.
