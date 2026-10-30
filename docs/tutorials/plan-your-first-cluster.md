# Plan Your First Cluster

This tutorial takes you from a set of Kubernetes manifests to a placement plan and back
to manifests that are ready to deploy. You work with a sample cluster of three nodes and
three Pods, so every step finishes in under a second and you can read every line of the
output. No running cluster is needed.

The outputs in this tutorial were recorded at commit `c15a04c`.


## Objectives

- Build the `kuberina-forge` and `kuberina-solver` tools.
- Convert Kubernetes manifests into Kuberina IR.
- Compute a placement plan and read it.
- Write manifests that pin each workload to its planned node.


## Before you begin

You need the following tools:

- Git.
- A Rust toolchain, to build `kuberina-solver`. See
  [Install Rust](https://www.rust-lang.org/tools/install).
- Go 1.26 or later, to build `kuberina-forge`. See [Download and install
  Go](https://go.dev/doc/install).

You do not need `kubectl` or access to a cluster.


## Build the tools

1. Clone the repository and enter it:

    ```shell
    git clone https://github.com/AlexanderSlokov/kuberina.git
    cd kuberina
    ```

1. Build both tools:

    ```shell
    make forge-build
    make solver-build
    ```

    The first command writes `bin/kuberina-forge`. The second writes
    `solver/target/debug/kuberina-solver`.

1. Add both tools to your `PATH` for this terminal session:

    ```shell
    export PATH="$PWD/bin:$PWD/solver/target/debug:$PATH"
    ```

1. Create a working directory with a copy of the sample manifests, and move into it:

    ```shell
    mkdir -p /tmp/kuberina-tutorial
    cp -r forge/testdata/manifests /tmp/kuberina-tutorial/k8s
    cd /tmp/kuberina-tutorial
    ```


## Look at the sample cluster

The `k8s` directory holds two files. The `cluster-nodes.yaml` file describes three
nodes, in the same format that `kubectl get nodes -o yaml` prints:

| Node | Zone | CPU | Memory | GPUs | Notes |
|---|---|---|---|---|---|
| `gpu-a` | az-1 | 64 | 256 GiB | 8 | Label `accelerator: h100`. Taint `dedicated=ml:NoSchedule`. |
| `gpu-b` | az-2 | 64 | 256 GiB | 8 | Label `accelerator: h100`. |
| `std-a` | az-1 | 16 | 64 GiB | 0 | |

A _taint_ marks a node so that only Pods with a matching _toleration_ can run on it. Here,
`gpu-a` accepts only Pods that tolerate the `dedicated` taint.

The `workloads.yaml` file describes the workloads:

- A Deployment named `trainer` with 2 replicas. Each replica asks for 1 GPU and selects
  nodes labeled `accelerator: h100`. It tolerates the `dedicated` taint.
- A Deployment named `parameter-server` with 1 replica. Its Pod affinity requires it to
  run on the same node as a `trainer` Pod. It does not tolerate the `dedicated` taint.
- A DaemonSet named `node-exporter`, which runs one Pod on every node.

Before you go on, predict where the three Pods should go. The trainers need an H100
node, so they can use `gpu-a` or `gpu-b`. The parameter server cannot use `gpu-a`,
because of the taint, and its affinity puts it next to a trainer.


## Convert the manifests into Kuberina IR

_Kuberina IR_ (intermediate representation) is the input format of the solver: one YAML
file that describes the nodes and one that describes the workloads. The
`kuberina-forge in` command writes both.

1. Convert the manifests:

    ```shell
    kuberina-forge in --out-infra infra.yaml --out-workloads workloads.yaml ./k8s
    ```

    Place flags before the source directory. The tool stops reading flags at the first
    source argument.

    The output is similar to this:

    ```console
    wrote infra.yaml: 3 nodes, 1 daemonsets
    wrote workloads.yaml: 2 declarations across 1 namespaces
    ```

1. Open `workloads.yaml` and find the `trainer` entry:

    ```yaml
    - name: trainer
      replicas: 2
      requests:
        cpu: 8.25
        ram: 32.5
        gpu: 1
      nodeSelector:
        accelerator: h100
    ```

    The `trainer` Pod has two containers. The trainer container asks for 8 CPUs and
    32 GiB of memory, and the log shipper asks for 0.25 CPU and 0.5 GiB. The tool adds
    them up the same way the kubelet does, so the IR records one request of 8.25 CPUs and
    32.5 GiB per Pod. Memory is in GiB.


## Compute a plan

1. Run the solver:

    ```shell
    kuberina-solver plan --infra infra.yaml --workloads workloads.yaml
    ```

    While the genetic algorithm runs, the solver prints a progress line every ten
    generations. When it finishes, the output ends with lines similar to this:

    ```console
    ═══ Phase 0: DaemonSet Pre-deduction (Ballast Water) ═══
      gpu-a: 64.0 → 63.90 CPU, 256.0 → 255.875 GiB RAM (-0.10 CPU, -0.125 GiB overhead)
      gpu-b: 64.0 → 63.90 CPU, 256.0 → 255.875 GiB RAM (-0.10 CPU, -0.125 GiB overhead)
      std-a: 16.0 → 15.90 CPU, 64.0 → 63.875 GiB RAM (-0.10 CPU, -0.125 GiB overhead)

    ═══ Final Blueprint (Stowage Plan) ═══
      ...
      gpu-a:
        CPU: 0.00/63.90 (0%)
        RAM: 0.000/255.875 GiB (0%)

      gpu-b:
        - parameter-server-0
        - trainer-0000
        - trainer-0001
        CPU: 20.50/63.90 (32%)
        RAM: 81.000/255.875 GiB (32%)

      std-a:
        CPU: 0.00/15.90 (0%)
        RAM: 0.000/63.875 GiB (0%)

      (Full stowage plan exported to kuberina_solution.yaml)
      ✅ FEASIBLE — every hard constraint holds.
    ```

1. Read the Phase 0 lines. The `node-exporter` DaemonSet runs on every node and asks
   for 0.1 CPU and 0.125 GiB. The solver subtracts that from each node before it places
   anything, so `gpu-a` offers 63.90 CPUs instead of 64.

1. Read the placement. All three Pods are on `gpu-b`, and `gpu-a` and `std-a` stay
   empty. This matches the prediction from earlier:

    - The trainers need an H100 node: `gpu-a` or `gpu-b`.
    - The parameter server cannot use `gpu-a`, and its affinity puts it next to a
      trainer.
    - The solver prefers plans that use fewer nodes, and one node holds all three Pods
      at 32% CPU.

1. Read the last line. FEASIBLE means that the plan satisfies every _hard constraint_:
   no node is over capacity, and every node selector and taint is respected.

> [!CAUTION]
> The `trainer` Deployment also asks Kubernetes to spread its replicas across zones,
> with at most one replica of difference between zones. This plan puts both replicas in
> zone az-2, and the solver reports no penalty for it. At commit `c15a04c`, the solver
> ignores topology spread rules that use the standard Kubernetes key
> `topology.kubernetes.io/zone`. The fix is tracked as K-2 in
> [BACKLOG.md](../../BACKLOG.md).


## Read the blueprint

A _blueprint_ is the file that records the plan. Print it:

```shell
cat kuberina_solution.yaml
```

The output is similar to this:

```yaml
apiVersion: kuberina.io/v1
kind: Blueprint
metadata:
  feasible: true
  pods: 3
  activeNodes: 1
solution:
  ai/parameter-server-0: gpu-b
  ai/trainer-0000: gpu-b
  ai/trainer-0001: gpu-b
```

Each line under `solution` maps one Pod to one node, and each Pod is named
`<namespace>/<workload>-<index>`. The `metadata.feasible` field is the verdict from the
last step. A team reviews this file before anything is deployed. To change the plan,
edit `infra.yaml` or `workloads.yaml` and run the solver again.


## Write the planned manifests

1. Apply the blueprint to the original manifests:

    ```shell
    kuberina-forge out --blueprint kuberina_solution.yaml --out ./planned ./k8s
    ```

    The output is similar to this:

    ```console
    wrote 2 manifest file(s) to ./planned
      ai/parameter-server → 1 replica(s) on gpu-b
      ai/trainer → 2 replica(s) on gpu-b

    Each workload is pinned with a required nodeAffinity on kubernetes.io/hostname.
    Review, then apply with kubectl.
    ```

1. Open `planned/workloads.yaml` and find the `trainer` Deployment. The tool added one
   block to its Pod template:

    ```yaml
    affinity:
      nodeAffinity:
        requiredDuringSchedulingIgnoredDuringExecution:
          nodeSelectorTerms:
            - matchExpressions:
                - key: kubernetes.io/hostname
                  operator: In
                  values:
                    - gpu-b
    ```

    This _node affinity_ tells `kube-scheduler` that the Pods may run only on the node
    named `gpu-b`. Everything else in the file matches the original manifest.

On a real cluster, you would now run `kubectl apply -f ./planned`.


## Clean up

1. Leave the working directory and delete it:

    ```shell
    cd -
    rm -rf /tmp/kuberina-tutorial
    ```


## What's next

- Plan your own cluster with the steps in [Plan a deployment](../../README.md#plan-a-deployment).
- Read the input and output formats in [Kuberina IR v1](../references/ir-v1.md).
- Learn why `kuberina-forge` runs as a command-line tool in
  [ADR-0001](../explanation/adr/0001-go-component-is-a-cli.md).
- Read the mathematical model in [PAPER.md](../references/PAPER.md).
