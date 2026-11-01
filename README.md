# Kuberina

[![DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.21582492.svg)](https://doi.org/10.5281/zenodo.21582492)

Kuberina is a command-line tool that plans where Pods run in a heterogeneous Kubernetes
cluster before you deploy them. It reads your node inventory and workload manifests,
searches for a placement that fits the Pods onto as few nodes as possible while it
respects their constraints, and writes the result to a file that your team can review.

To try it on a three-node sample cluster, follow
[Plan your first cluster](docs/tutorials/plan-your-first-cluster.md).

The method comes from maritime stowage planning: the problem of loading cargo onto a
container ship such as the *MSC Irina*. Kuberina treats nodes as holds and Pods as
cargo, and solves placement as a constrained, multi-dimensional bin packing problem.

## Why you need Kuberina

The default Kubernetes scheduler, `kube-scheduler`, places each Pod when it arrives,
within milliseconds, and without knowledge of the Pods that arrive after it. On a cluster that mixes CPU, memory, and GPU
nodes, these one-at-a-time decisions fragment capacity: free resources end up spread
across many nodes in amounts too small for the next workload to use.

These decisions also leave no record. For example, if one node runs at 98% CPU and
another at 15%, no document explains how the cluster reached that state.

Kuberina plans the placement of all Pods together, ahead of deployment, and writes the
plan to a file. You can read the plan, discuss it, change the inputs, and plan again
before anything reaches the cluster.

## What Kuberina provides

- **Placement across eight resource dimensions.** Kuberina packs Pods against CPU,
  memory, GPU, storage, disk read and write throughput, and network ingress and egress.
- **Constraint handling.** A _hard constraint_ is a rule that a plan must satisfy, and
  a _soft constraint_ is a preference that Kuberina scores. Node selectors and taints
  are hard constraints. Pod affinity, anti-affinity, and topology spread are soft
  constraints. Pod groups carry a minimum member count, and Kuberina penalizes any plan that
  places fewer members than that minimum.
- **DaemonSet overhead.** Kuberina subtracts DaemonSet requests from the allocatable
  capacity of every node before it plans.
- **Capacity headroom.** The `--headroom` flag reserves a percentage of each node's
  capacity.
- **Shared GPUs.** A node can declare its GPU devices with `gpuDevices`, and a workload
  can state the share of each device it needs with `gpuMilli`. Kuberina then checks
  that every Pod fits on real devices, in addition to the node's GPU total. For example,
  a Pod that needs half a GPU does not fit a node whose eight GPUs are each 60% used.
- **Admission for benchmarks.** With `--admission in-order` or `--admission free`, the
  solver may leave Pods unplaced when they ask for more than the cluster holds, and it
  lists them under `unplaced:` in the blueprint. This mode serves comparisons with
  scheduler traces, and `kuberina-forge out` refuses such a partial plan.
- **A reviewable blueprint.** A _blueprint_ is the YAML file that maps each Pod to a
  node.
- **Manifest output.** `kuberina-forge out` writes your manifests back out with a
  required `nodeAffinity` that pins each workload to its planned nodes.
- **Independent validation.** The inspector checks capacity, GPU devices, node
  selectors, affinity rules, and the admission rules in a blueprint with its own code,
  and renders a heatmap of node utilization.

## How Kuberina works

Kuberina runs as three steps. Each step reads files and writes files, and none of them
connects to your cluster.

1. The `kuberina-forge in` command converts Kubernetes manifests and node definitions
   into _Kuberina IR_ (intermediate representation): two YAML files, one that describes
   the nodes and one that describes the workloads. The format is specified in
   [Kuberina IR v1](docs/references/ir-v1.md).
1. The `kuberina-solver plan` command computes the placement in three phases. Phase 0
   subtracts DaemonSet overhead. Phase 1 builds a first placement with first-fit
   decreasing (FFD), which places the largest Pods first. Phase 2 improves that
   placement with a _genetic algorithm_: it keeps a population of candidate plans,
   combines and mutates the best ones, and repairs any plan that breaks a hard
   constraint. The solver writes the blueprint to `kuberina_solution.yaml` in the
   current directory.
1. The `kuberina-forge out` command applies the blueprint to your original manifests.

## Before you begin

Install the tools for the components you use:

| Tool | Needed for |
|---|---|
| Rust toolchain | `solver-*` targets |
| Go 1.26 or later | `forge-*` targets |
| [uv](https://docs.astral.sh/uv/) | `inspector-*`, `bench-*`, and `research-*` targets |

The `inspector-*` and `bench-*` targets install PyYAML when they run, through
`uv run --with pyyaml`. The `research-*` targets use the locked environment in
`research/pyproject.toml`. The `full-pipeline` target needs both Rust and uv.

Build the two binaries:

```shell
make forge-build     # needs Go; writes bin/kuberina-forge
make solver-build    # needs Rust; writes solver/target/debug/kuberina-solver
```

## Plan a deployment

1. Export your nodes and convert them, together with your manifests, into Kuberina IR:

    ```shell
    kubectl get nodes -o yaml > nodes.yaml
    kuberina-forge in --out-infra infra.yaml --out-workloads workloads.yaml \
        ./k8s nodes.yaml
    ```

    Place flags before the sources. `kuberina-forge` stops reading flags at the first
    source argument.

    If you render manifests with Helm or Kustomize, pipe the rendered output to
    `kuberina-forge in` and pass `-` as the source:

    ```shell
    helm template ./chart | kuberina-forge in --out-workloads workloads.yaml -
    ```

1. Compute a plan:

    ```shell
    kuberina-solver plan --infra infra.yaml --workloads workloads.yaml
    ```

    The output ends with lines similar to this:

    ```console
      (Full stowage plan exported to kuberina_solution.yaml)
      ✅ FEASIBLE — every hard constraint holds.
    ```

    The solver reports one of three verdicts. FEASIBLE means the plan satisfies every
    hard constraint. RESERVE NOT MET means the plan fits but uses some of the capacity
    that `--headroom` asked it to keep free. INFEASIBLE — DO NOT APPLY means the cluster
    cannot hold the workloads as declared, and the lines that follow list each resource
    that overflows.

1. Review `kuberina_solution.yaml` with your team. To change the plan, edit
   `infra.yaml` or `workloads.yaml` and run the solver again.

1. Apply the agreed blueprint to your manifests, and deploy the result:

    ```shell
    kuberina-forge out --blueprint kuberina_solution.yaml --out ./planned ./k8s
    kubectl apply -f ./planned
    ```

    The output manifests match your input manifests, with one addition: a required
    `nodeAffinity` on each workload. `kuberina-forge out` refuses a blueprint that the
    solver marked infeasible, and one that leaves Pods unplaced.

## Run the benchmark

The repository includes *MSC Irina*, a generated cluster of 620 nodes and 2,714 Pods.

To plan the benchmark and view the result:

```shell
make solver-irina     # needs Rust
make inspector-run    # needs uv
```

`make inspector-run` writes `kuberina_dashboard.html`. Open it in a browser to see the
utilization heatmap.

At commit `096177e`, one plan of this cluster took 865 seconds and used 540 of the 620
nodes, with 964 of its soft affinity and anti-affinity rules unmet. The session record
[2026-11-01-irina-with-affinity](docs/references/sessions/2026-11-01-irina-with-affinity.md)
lists the commands and the full results.

## What Kuberina is not

Kuberina:

- Does not replace `kube-scheduler`. The cluster scheduler still binds every Pod, within
  the `nodeAffinity` that Kuberina adds.
- Does not connect to, watch, or modify a running cluster.
- Does not scale workloads.
- Does not re-plan by itself when usage changes. To update a plan, you run the solver
  again.

## Documentation

| Document | Audience | Contents |
|---|---|---|
| [PAPER.md](docs/references/PAPER.md) | Researchers, reviewers | Mathematical foundation, problem formulation, and experimental results |
| [DESIGN.md](docs/DESIGN.md) | Contributors, maintainers | Architecture, data model, CLI design, and testing strategy |
| [Kuberina IR v1](docs/references/ir-v1.md) | Tool authors, operators | Normative specification of the input and output documents |
| [Architecture decisions](docs/explanation/adr/README.md) | Contributors | Recorded design decisions and their reasons |
| [ROADMAP.md](ROADMAP.md) | Everyone | Planned milestones |
| [CHANGELOG.md](CHANGELOG.md) | Operators | Changes in each release |

## Repository layout

| Path | Component | Language | Description |
|---|---|---|---|
| `forge/` | `kuberina-forge` | Go | Converts manifests to Kuberina IR (`in`) and applies a blueprint to manifests as node affinity (`out`). Runs as a CLI; see [ADR-0001](docs/explanation/adr/0001-go-component-is-a-cli.md). |
| `solver/` | `kuberina-solver` | Rust | The optimization engine: FFD warm start, genetic algorithm, and constraint repair over an 8-dimensional model. |
| `inspector/` | Inspector | Python | Constraint validator and heatmap dashboard. It shares no code with the solver, so its result is an independent check. |
| `bench/` | Benchmarks | Python | Generates the MSC Irina test data and runs the feasibility, quality, and significance proofs. |
| `research/` | Reference implementation | Python | The first implementation of the three-phase pipeline, used to validate the model before the Rust port. |

## Licensing roadmap

Kuberina employs a 3-phase licensing strategy to ensure sustainable development, protect intellectual property and deliver a fully permissive open-source project to the community. 

Please read the following licensing transitions carefully. By using, modifying, or distributing this software, you acknowledge and agree to this roadmap. 

> **Important Note for Contributors:** To ensure the project retains the legal ability to execute these planned license transitions, all community contributions (Pull Requests) require agreeing to our Contributor License Agreement (CLA). This grants the project author the right to re-license your contributions in accordance with the roadmap below.

### Phase 1: Kuberina (AGPLv3)

**Applies to:** All current versions starting from `0.1.0`, and all subsequent releases, up to (but not including) version `12.0.0`, unless Phase 2 is triggered first.  
**License:** **GNU Affero General Public License v3.0 (AGPLv3)**.  
**Implication:** Any modifications or deployments of Kuberina provided as a network service must make their source code available to users under the same AGPLv3 license.  

### Phase 2: Naughtian Kuberina (SSPLv1)

**Trigger Condition:** The formal legal incorporation of the author's corporate entity (e.g., "Naughtian Lab" or "Naughtian Corp").  
**Action:** Upon this event, the intellectual property and repository ownership of Kuberina will be transferred to said corporate entity.   
**License:** From the exact date of transfer forward, all new versions and commits of Kuberina will be licensed under the **Server Side Public License v1 (SSPLv1)**.  
*Note: Previously released AGPLv3 versions will permanently remain AGPLv3. Re-licensing applies strictly to new releases following the corporate transfer.*  

### Phase 3: Liberated Kuberina (Apache License 2.0)

The project is committed to an eventual transition to a fully permissive model. The repository and its subsequent releases will transition to the **Apache License 2.0** upon the occurrence of *either* of the following strategic milestones:

1. **CNCF Donation Initiative:** Prior to, and as a formal step for, donating Kuberina to the Cloud Native Computing Foundation (CNCF), the project will be re-licensed to Apache 2.0 to comply with CNCF's intellectual property policies.
2. **Reaching Version 12.0.0:** Kuberina officially reaches the `12.0.0` milestone.

*(To guarantee Kuberina's permanent availability to the open-source community regardless of versioning speed, any given release of Kuberina will automatically transition to the Apache License 2.0 exactly 48 months after its initial release date).*

**LTS Roadmap:** Upon the release of `12.0.0`, the project will establish a dedicated Long-Term Support branch (`12.0.0-lts`). It is the explicit intent and roadmap of the project's governing entity to provide a minimum **5-year window of maintenance and security patching** for this specific LTS release under the Apache 2.0 license.
