# Kuberina Benchmarks

Testdata generation and formal verification for the MSC Irina benchmark, and the
adapter that converts the Alibaba GPU trace into Kuberina IR.

| Script | Purpose |
|---|---|
| `gen_irina_testdata.py` | Generates the MSC Irina-scale testbed: 186 nodes, 2,714 pods, 5,128 constraints, 8-dimensional resource vectors |
| `mathematical_proof.py` | Proves feasibility, quality (α against the LP lower bound), and significance (Monte Carlo against random placement) |
| `alibaba/` | Converts Alibaba `cluster-trace-gpu-v2023` and FGD simulator logs into Kuberina IR (issue #24, X-2) |
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

## Alibaba trace adapter

The `alibaba/` package turns one run of the FGD scheduler simulator into a Kuberina
planning instance. It reads the trace CSV files and one simulator log per policy, and
writes three files:

| File | Content |
|---|---|
| `infra.yaml` | One node per row of `openb_node_list_gpu_node.csv`, with the GPU model as the `gpu-model` label and one GPU device per GPU (`gpuDevices`) |
| `workloads.yaml` | One declaration per Pod the simulator offered, in arrival order, in namespace `paib-gpu`. A Pod that needs part of a GPU states it with `gpuMilli` |
| `sequence.csv` | One row per arrival: IR identity, trace row, requests, and the node each policy chose (empty when rejected) |

To build the seed-42 instance from the committed simulator logs, run these commands from
the repository root. The first one needs `curl` and network access.

```shell
make bench-alibaba-fetch
make bench-alibaba-ir
```

The output is similar to this:

```console
nodes: 1213
pods: 10866
GPU demand written and logged: 8075080 milli-GPU
wrote alibaba/out/seed42/infra.yaml, alibaba/out/seed42/workloads.yaml, alibaba/out/seed42/sequence.csv
```

The downloads go to `bench/alibaba/data/` and the output to `bench/alibaba/out/`. Git
ignores both directories. The adapter writes nothing and exits with status 1 when the
logs offered different Pods or when the trace demand differs from a logged
`Arrived GPU Milli` value. The measured run is recorded in
[the trace adapter session record](../docs/references/sessions/2026-11-01-alibaba-trace-adapter.md).
