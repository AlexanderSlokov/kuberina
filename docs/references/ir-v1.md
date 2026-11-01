# Kuberina IR v1 — specification

**Status:** normative. This document defines Kuberina IR v1. Where an implementation
and this document disagree, the implementation has a bug.
**Governing decision:** [ADR-0002](../explanation/adr/0002-kuberina-ir-v1-stability-contract.md).
**Implementations:** `solver/src/parser.rs` (Rust, authoritative reader),
`kuberina-forge` (Go, writer and linker), `research/src/kuberina/parser.py` (Python
reference), `bench/gen_irina_testdata.py` (generator), `inspector/inspector.py`
(independent validator).

IR v1 has three document kinds: **Infrastructure**, **Workloads**, and **Blueprint**.
The first two are inputs to the solver; the third is its output and the forge's input.

---

## 1. Rules that apply to every document

**Header.** Every document may carry `apiVersion: kuberina.io/v1` and a `kind`. Both
are optional on input, and their absence means v1. A present `apiVersion` naming a
version the reader does not implement is an **error**, as is a `kind` that does not
match the document being read. Every document Kuberina writes carries both.

**Unknown fields are errors.** Every mapping in this specification is closed. A key not
listed here fails the parse, and the error names the offending key. This is not
strictness for its own sake: a dropped key once left the solver planning in four of its
eight dimensions while reporting eight, and no test caught it.

**Additive evolution.** Within v1, new *optional* fields may appear; their absence
always reproduces the behavior described here. Meanings, units, defaults and
requiredness do not change inside v1 — that is what v2 is for.

**Units are normative.**

| Quantity | Unit | Notes |
|---|---|---|
| `cpu` | cores | `2500m` is accepted and means 2.5 |
| `ram`, `storage` | GiB | `512Mi` → 0.5, `2Gi` → 2.0 |
| `gpu` | whole devices | fractional values are accepted but not meaningful to the packer |
| `disk.read`, `disk.write` | MB/s | `5G` → 5000 |
| `network.in`, `network.out` | MB/s | same scale as disk |

Anywhere a number is accepted, a Kubernetes quantity string is accepted too, and both
parse through one shared implementation (`solver/src/quantity.rs`).

**The resource block.** Wherever `resources`, `requests`, `allocatable`, or an
`observed` percentile appears, it has this shape and no other:

```yaml
cpu: 4.0
ram: 16Gi
gpu: 1
storage: 100Gi
disk:
  read: 500
  write: 500
network:
  in: 1000
  out: 1000
```

Flat spellings such as `disk_read` or `net_in` are **not** part of IR v1 and are
rejected. Omitted dimensions default to `0` for demand (pods, DaemonSets, observed
percentiles) and to *unconstrained* for node `allocatable`, so a homelab file may omit
throughput entirely and mean "not a limit here".

---

## 2. Infrastructure document

The cluster the plan is made against.

```yaml
apiVersion: kuberina.io/v1
kind: Infrastructure

nodes:
  - name: gpu-008                  # required, unique
    zone: az-1                     # optional, default ""
    rack: r14                      # optional, default ""
    labels:                        # optional, default {}
      accelerator: h100
    taints:                        # optional, default []
      - key: dedicated
        value: ml
        effect: NoSchedule
    allocatable:                   # optional; omitted dimensions are unconstrained
      cpu: 64
      ram: 256Gi
      gpu: 8
      storage: 2Ti
      disk: { read: 2000, write: 2000 }
      network: { in: 10000, out: 10000 }

daemonsets:
  - name: node-exporter            # required
    resources:                     # optional, default all-zero
      cpu: 0.1
      ram: 128Mi
    nodeSelector: {}               # optional, default {}
    tolerations: []                # optional, list of taint keys
```

| Field | Type | Required | Default |
|---|---|---|---|
| `nodes[].name` | string | yes | — |
| `nodes[].zone`, `.rack` | string | no | `""` |
| `nodes[].labels` | map[string]string | no | `{}` |
| `nodes[].taints[].key` | string | yes within a taint | — |
| `nodes[].taints[].value`, `.effect` | string | no | `""` |
| `nodes[].allocatable` | resource block | no | unconstrained per dimension |
| `daemonsets[].name` | string | yes | — |
| `daemonsets[].resources` | resource block | no | zero |
| `daemonsets[].nodeSelector` | map[string]string | no | `{}` |
| `daemonsets[].tolerations` | list[string] | no | `[]` |

**Semantics.** DaemonSet resources are subtracted from every node's allocatable before
planning begins — Phase 0, the "ballast water" deduction. Taints and tolerations are
matched on `key` only; `value` and `effect` are accepted, preserved in the file, and
not yet interpreted.

---

## 3. Workloads document

What is to be placed.

```yaml
apiVersion: kuberina.io/v1
kind: Workloads

namespaces:
  ai:
    - name: trainer                # required, unique within its namespace
      replicas: 4                  # optional, default 1
      requests:                    # optional, default all-zero
        cpu: 8
        gpu: 1
        ram: 64Gi
      nodeSelector:                # optional — hard constraint
        accelerator: h100
      tolerations:                 # optional
        - key: dedicated
          operator: Equal
          value: ml
          effect: NoSchedule
      affinity: ["ai/parameter-server"]      # optional, soft
      antiAffinity: ["ai/trainer"]           # optional, soft
      topologySpread:                        # optional, soft
        maxSkew: 1
        topologyKey: zone
      gang: training-job                     # optional shorthand, see §3.2
      observed: { ... }                      # optional, see §3.3

groups:                                      # optional, see §3.2
  - name: training-job
    members: ["ai/trainer"]
    min_members: 3
    colocate: true
    nodeSelector:
      accelerator: h100
```

### 3.1 Replica unrolling

A declaration with `replicas: n` becomes `n` independent pods. Names are
`<name>-0000`, `<name>-0001`, … when `n > 1`, and `<name>-0` when `n == 1`. The pod
identity used everywhere else — blueprint keys, affinity targets, inspector output — is
`namespace/podname` after unrolling.

Affinity and anti-affinity targets are matched against these unrolled identities.

### 3.2 Gangs

A gang is placed all-or-nothing, the scheduling equivalent of a shipping line's block
booking. There are two ways to declare one, and they interoperate.

**Inline shorthand.** `gang: <name>` on a declaration enrols all of its replicas.
Every pod carrying the same name forms one group, `min_members` equals the full
membership, and `colocate` is false. This is unchanged from earlier releases.

**Explicit block.** A `groups:` entry states the same thing and can say more:

| Field | Type | Required | Default |
|---|---|---|---|
| `name` | string | yes | — |
| `members` | list[string], each `namespace/name` | yes | — |
| `min_members` | integer | no | number of resolved members |
| `colocate` | boolean | no | `false` |
| `nodeSelector` | map[string]string | no | `{}` |

`members` names *declarations*, not unrolled replicas: listing `ai/trainer` enrols all
four replicas. A member that matches no declaration is an error naming the offending
value. `min_members` must lie in `1..=len(members)`; anything else is an error stating
the admissible range. A pod may belong to at most one group, and a pod claimed by both
an inline `gang:` and a `groups:` entry is an error rather than a silent winner.

When a name appears in both forms, the explicit block wins — otherwise auto-grouping
would rebuild the gang with the shorthand's defaults and discard `min_members` and
`colocate`.

**Semantics.** `min_members` is the smallest admissible gang: the fitness function
penalizes a placement that admits fewer members than this. `colocate: true` requires
every member on one node. Both were reachable only from unit tests before IR v1
(issue #9).

### 3.3 Observed usage

Optional, additive, and **not read by the packing model** in this release. It exists so
that a producer of usage telemetry — Kalena, a Prometheus export, VPA recommendations,
a `kubectl top` snapshot — can record steady state on the same terms as every other
input: a file the operator reviews, never an endpoint the planner polls.

```yaml
observed:
  window: 720h                     # retention the statistics summarize, verbatim
  p50:  { cpu: 0.9, ram: 3Gi }
  p95:  { cpu: 2.4, ram: 5Gi }
  p99:  { cpu: 3.1, ram: 6Gi }
  peak: { cpu: 4.6, ram: 7Gi }
  exceeded_request_fraction: 0.004
```

| Field | Type | Required | Default |
|---|---|---|---|
| `window` | string | no | `""` |
| `p50`, `p95`, `p99`, `peak` | resource block | no | absent |
| `exceeded_request_fraction` | float in `[0, 1]` | no | `0.0` |

Each percentile is a full resource block and parses like any other. An absent
percentile stays absent rather than becoming zero — the two mean different things.
`exceeded_request_fraction` is kept because the percentiles lose it: a workload whose
p99 sits below its request but which occasionally exceeds it entirely is mis-sized, not
over-provisioned.

Selecting what to pack against (`--pack-against p95`, as proposed in issue #13) is a
separate decision and is not part of IR v1.

---

## 4. Blueprint document

The solver's output, and `kuberina-forge out`'s input.

```yaml
apiVersion: kuberina.io/v1
kind: Blueprint
metadata:
  feasible: true
  pods: 2714
  activeNodes: 540
solution:
  ai/trainer-0000: gpu-008
  ai/trainer-0001: gpu-008
  edge/api-gateway-0: std-114
```

| Field | Type | Meaning |
|---|---|---|
| `metadata.feasible` | boolean | false when the plan breaches real capacity; such a file is written for inspection and must not be applied |
| `metadata.pods` | integer | number of entries in `solution`, for a cheap integrity check |
| `metadata.activeNodes` | integer | distinct nodes used |
| `solution` | map[string]string | `namespace/podname` → node name, one entry per unrolled pod |

An infeasible blueprint additionally carries a comment banner at the top of the file.
Consumers must treat `metadata.feasible: false` as refusal to apply, not as a warning.

---

## 5. Compatibility

**Forward.** A v1 reader rejects a document using a field added in a later v1.x
release. Upgrade the reader before the producer.

**Backward.** Files written before IR v1 was declared parse unchanged: they simply omit
the header. The only breaking correction is that a document containing an unrecognized
key now fails instead of silently losing that key.

**Toward v2.** A change that alters the meaning, unit, default or requiredness of any
field above, or the shape of any document, requires `apiVersion: kuberina.io/v2` and a
successor to this specification. When that happens, the solver reads v1 for at least one
further minor release and `kuberina-forge` gains a conversion path.

---

## 6. Worked example

A minimal but complete pair, exercising gangs and observed usage.

```yaml
# infra.yaml
apiVersion: kuberina.io/v1
kind: Infrastructure
nodes:
  - name: gpu-a
    zone: az-1
    labels: { accelerator: h100 }
    allocatable: { cpu: 64, ram: 256Gi, gpu: 8 }
  - name: gpu-b
    zone: az-2
    labels: { accelerator: h100 }
    allocatable: { cpu: 64, ram: 256Gi, gpu: 8 }
daemonsets:
  - name: node-exporter
    resources: { cpu: 0.1, ram: 128Mi }
```

```yaml
# workloads.yaml
apiVersion: kuberina.io/v1
kind: Workloads
namespaces:
  ai:
    - name: trainer
      replicas: 4
      requests: { cpu: 8, gpu: 1, ram: 32Gi }
      nodeSelector: { accelerator: h100 }
      observed:
        window: 720h
        p95: { cpu: 6.1, ram: 28Gi }
        exceeded_request_fraction: 0.0
groups:
  - name: training-job
    members: ["ai/trainer"]
    min_members: 3
    colocate: true
```

This declares four pods that must land together on one H100 node, will still be
admitted if only three fit, and carries a month of observed usage that this release
records but does not act on.
