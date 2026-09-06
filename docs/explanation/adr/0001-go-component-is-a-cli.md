# ADR-0001 — The Go component is a CLI, not an operator

**Status:** Accepted — 2026-09-06
**Deciders:** Dinh Tan Dung
**Affects:** `kuberina-forge`, ROADMAP Phase 1 and Phase 2, issues #7 and #14

## Context

The repository has carried two incompatible designs for its Go component, and both are
written down.

`docs/plans/core/RIIR/preplan.md` §4 describes **`kuberina-operator`**: a Kubernetes
controller running inside the cluster, using `client-go` to watch Pods and Nodes,
collecting cluster state when a plan is needed and shipping it to the solver as a
Protobuf `TopologyRequest` over gRPC.

`ROADMAP.md` Phase 1 describes **`kuberina-forge`**: a CLI frontend and linker —
*forge-in* ingesting manifests, Helm charts, Kustomize output and cloud API state into
Kuberina IR, *forge-out* rendering the solver's blueprint back into applyable manifests.

These are not two names for one thing. A controller watches and reacts; a CLI is
invoked. And ROADMAP Phase 3 already states a principle the controller design cannot
satisfy:

> Kuberina must not poll a metrics endpoint or re-plan itself: the blueprint's value
> rests on being a reviewed artifact, and a planner that silently regenerates from live
> metrics gives that up.

A control loop whose whole purpose is to notice drift and act on it is, structurally, a
planner that regenerates itself. The two documents were describing different products.

Meanwhile `main.go` is seven lines that print a banner, `go.mod` declares no
dependencies, and issues [#7](https://github.com/AlexanderSlokov/kuberina/issues/7) and
[#14](https://github.com/AlexanderSlokov/kuberina/issues/14) are open because the README
presents `kuberina plan` as something an operator can run today. The ambiguity has
already produced documentation that overstates the product.

## Decision

**`kuberina-forge` is a command-line program. It has no control loop.**

Concretely, for as long as this record stands:

1. Forge runs when a human or a CI job runs it, and exits. It does not watch, reconcile,
   subscribe, or hold a lease.
2. Forge does not write to a live cluster. Its output is files — Kuberina IR going in,
   applyable manifests coming out. Applying them is `kubectl`'s job, under whatever
   review process the operator already has.
3. Forge does not decide when to re-plan. Drift detection, scheduling policy, and
   anything that turns an observation into an action are out of scope.
4. Input is read from files, directories, or stdin. Cluster state enters the same way
   any other input does — as a file the operator produced, for example with
   `kubectl get nodes -o yaml`.

**The operator concern moves to the Kuberina Integration Platform (KIP), a separate
closed-source repository.** Nothing in this repository will depend on KIP, and KIP is
free to build the controller, the gRPC transport and the drift triggers that this
decision keeps out of the open-source core. The seam between them is the IR: KIP
produces Kuberina IR and consumes blueprints, exactly as a human with `kubectl` does.

## Consequences

**What this makes easy.**

- Forge is hermetic and deterministic: same inputs, same bytes out. It can be tested
  with golden files and no cluster, which is the only kind of test this repository can
  actually run in CI today.
- The dependency footprint stays small. No `client-go`, no `apimachinery`, no
  `controller-runtime`, and therefore no obligation to track their release cadence or
  their Kubernetes version skew policy.
- The reviewed-artifact thesis in PAPER §1 and §8.4 survives contact with the tooling
  instead of being contradicted by it.
- The Helm question answers itself. Helm leads Kubernetes packaging — 81% usage and 75%
  naming it their package manager in the CNCF 2025 survey — but Helm, Kustomize, Argo CD
  and Flux all *end* at rendered manifests. Reading manifests from stdin serves every one
  of them: `helm template ./chart | kuberina-forge in -`. Native chart rendering can be
  added later without changing this decision.

**What this makes hard, and we accept.**

- An operator who wants current cluster state must produce it themselves. This is one
  extra command, and it is the command that makes the input reviewable.
- Continuous placement — noticing at 03:00 that the cluster has drifted and acting — is
  not something Kuberina will do. That is the point of the decision, not an oversight.
- If KIP later needs behavior the CLI does not expose, the fix is to widen the CLI's
  documented interface, not to grow a daemon inside this repository.

**What it forecloses.** Any future work that begins "the forge should watch…" is
out of scope by this record. Overturning it takes a new ADR that supersedes this one and
that says what happens to the reviewed-artifact principle.

## Alternatives considered

**Build the controller as described in preplan.md §4.** Rejected. It contradicts
ROADMAP Phase 3's stated principle, it puts `client-go` and its version-skew obligations
into the core repository, and it makes every test require a cluster or an envtest
harness. The idea is not wrong — it is simply a different product, and it now has a
home in KIP.

**A CLI that also reads the cluster read-only via `client-go` (`--from-cluster`).**
Rejected for v1, and it remains available later. Reading is not watching, so this would
not violate the principle. But it pulls the entire `k8s.io` dependency tree in for
something `kubectl get -o yaml` already does, and it makes forge's behavior depend on
ambient kubeconfig state — which is exactly what makes a planning input hard to review.

**Render Helm and Kustomize natively inside forge.** Rejected for v1. `helm.sh/helm/v3`
and `sigs.k8s.io/kustomize` are heavy dependencies that duplicate tools the operator
already has installed and already trusts. A pipe is not a missing feature.

## References

- `ROADMAP.md` Phase 1 (`kuberina-forge`), Phase 2 (gRPC, cloud-to-edge), Phase 3
  (Kalena interoperability, the reviewed-artifact principle)
- `docs/plans/core/RIIR/preplan.md` §4–§5 (the superseded `kuberina-operator` sketch)
- `docs/DESIGN.md` §1 (language split, and why Go was chosen for this component)
- Issues [#7](https://github.com/AlexanderSlokov/kuberina/issues/7),
  [#14](https://github.com/AlexanderSlokov/kuberina/issues/14)
- CNCF, "Kubernetes Established as the De Facto 'Operating System' for AI as Production
  Use Hits 82% in 2025 CNCF Annual Cloud Native Survey," January 2026 — Helm usage and
  GitOps adoption figures cited above.
