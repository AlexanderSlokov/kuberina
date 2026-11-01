# Plan — Kuberina on Alibaba `cluster-trace-gpu-v2023`

**Issue:** [#24](https://github.com/AlexanderSlokov/kuberina/issues/24) ·
**ROADMAP:** Phase 2 (v0.4.0), *External Validation on Production Traces* ·
**Written at:** commit `7e5609f`, 2026-10-30 ·
**Survey this plan rests on:** `docs/references/sessions/2026-10-30-alibaba-gpu-trace-profile.md`

This plan describes how to run Kuberina on Alibaba's production-scale cluster data and
how to report what its results mean next to the published results of the Alibaba team.
Every figure that reaches `PAPER.md` first appears in a session record under
`docs/references/sessions/`, together with the commit and command that produced it.

## Progress

Updated 2026-11-01. Each finished stage has a session record:

| Stage | Status | Record |
|---|---|---|
| 1. Calibrate the referee | Done | `docs/references/sessions/2026-10-31-fgd-simulator-calibration.md` |
| 2. Trace adapter | Done | `docs/references/sessions/2026-11-01-alibaba-trace-adapter.md` |
| 3. Per-device GPU model (G1) | Done | `docs/references/sessions/2026-11-01-per-device-gpu.md` |
| 4. Admission (G2, G3) | Done, with provisional weights | `docs/references/sessions/2026-11-01-admission.md` |
| S-2. Objective weights on this instance | Next; decisions needed from the maintainer | — |
| 5. Scale check | Waits on S-2 | — |
| 6. Experiments E1–E5 | Waits on stage 5 | — |
| 7. Paper section | Waits on stage 6 | — |

Kuberina has not yet run on the trace. By *Purpose and stance*, it does not run there
until S-2 has fixed the weights.

## Purpose and stance

Every Kuberina result so far comes from MSC Irina, a cluster that Kuberina's own
generator built. The Alibaba trace supplies a problem that nobody here designed: the
node inventory, the Pods and their requests, and the constraints of a production GPU
cluster. The purpose of this plan is to run Kuberina on that problem and to state what
its result means beside the figures that FGD and its baselines reached on the same data.

The comparison is an observation. Its rules:

- **Kuberina's results are reported as they come.** A result below FGD's is reported in
  the same form and with the same prominence as a result above it.
- **Capabilities the data requires are in scope.** The trace contains fractional GPU
  requests and more demand than capacity, so Kuberina gains a per-device GPU model
  (Stage 3) and admission (Stage 4). Without them, Kuberina cannot represent the
  problem.
- **Tuning against the comparison is out of scope.** Fitness weights and GA settings
  come from measurements of the problem itself, such as the ranges of the objective
  terms, and are recorded with their derivation. They are fixed before Kuberina's
  first run on the trace, and the protocol in §4 is fixed with them.
- **The meaning of each figure is stated with the figure.** Kuberina plans offline and
  FGD schedules online, so each comparison in §4 names the advantage each side has.

---

## 1. What the paper should be able to say

When the plan is complete, PAPER.md gains one evaluation section with evidence:

1. **The referee is calibrated.** We ran the FGD authors' simulator ourselves and
   reproduced their published figure for FGD on the default trace, within a stated
   tolerance.
2. **How Kuberina compares under arrival order.** Pods arrive in the same seeded order
   the simulator uses, and Kuberina may choose only *where* each one goes. We report how
   far each policy gets before it first rejects a pod.
3. **What re-planning is worth.** We report Kuberina's GPU allocation at 130% of
   capacity under in-order admission, next to FGD's 95.39%. Kuberina may move pods it
   has already placed and FGD may not, so the label on the figure says so.
4. **The offline ceiling.** We report the best allocation Kuberina reaches when it may
   choose freely which pods to admit. The paper presents this figure as an upper
   bound.
5. **The cost.** We report wall clock per run against the simulator's per-decision
   latency. If a solve takes minutes, the paper says that Kuberina cannot stand in for
   `kube-scheduler`.

If a result goes against Kuberina, it is reported the same way as one in its favor.

---

## 2. Facts the plan is built on

From the trace profile and from reading the simulator's source (not yet from running
it):

| Fact | Value | Source |
|---|---|---|
| Nodes the simulator uses | 1,213 GPU nodes, 6,212 GPUs | `data/prepare_input.sh` copies `openb_node_list_gpu_node.yaml` only. The 310 CPU-only nodes are **not** part of the benchmark cluster. |
| Default pod list | 8,152 pods, total GPU demand 98.0% of capacity | trace profile |
| Fractional-GPU pods | 3,078 of 7,064 GPU pods | trace profile |
| Multi-GPU pods | 75, each needing whole devices on one node | trace profile |
| GPU-type constraints in `default` | none | trace profile |
| How the pod set reaches 130% | `tuneUpPods` (`pkg/simulator/simulator.go:1121`) draws pods with replacement from the original list, seeded by `-tuneseed`, until GPU demand would exceed 1.3× capacity. With `--shuffle-pod=true` the order is then shuffled. | source |
| Order of scheduling | One pod at a time, logged as `[i] attempt to create pod(<name>)`, with failures logged as `failed to schedule pod` | `SchedulePods`, `simulator.go:312` |
| Published KPI | GPU allocation ratio against arrived GPU demand, sampled at 0–130% | `experiments/analysis/expected_results/analysis_allo_discrete.csv` |
| Published FGD at 130%, default, seeds 42–51 | 95.39% (Best-Fit 93.08%, Random 87.47%) | same file, averaged |

The simulator draws its random numbers with Go's `math/rand`. Reimplementing that
generator to rebuild the pod sequence is fragile, so the plan **reads the sequence
from the simulator's own log** (§4, Stage 2). The log names every pod in the order
it was offered, and a cloned pod's name maps back to its source row.

---

## 3. What Kuberina lacks today

These three gaps are why Stages 3–5 exist. Each is stated as the failure it would
cause.

**G1 — GPU is one number per node.** `ResourceVector::fits` (`solver/src/model.rs:99`)
compares aggregate GPU. A 0.5-GPU pod would be accepted onto a node whose eight devices
are each 60% used. That placement is impossible on the real node, and the inspector
would pass it, because it sums GPU the same way (`inspector/inspector.py:138`).

**G2 — Every pod gets a node.** `Blueprint::assignment` is `Vec<usize>` with one node
per pod, and `ffd_warmstart` falls back to an overflowing node when nothing fits
(`solver/src/phase1_ffd.rs:172`). At 130% demand, every run ends with
`feasible: false`.

**G3 — The objective does not value admission.** The fitness function
(`solver/src/fitness.rs`) minimizes active nodes, fragmentation, affinity violations,
variance and spread. Once G2 is fixed, the optimizer could "improve" its score by
leaving GPU pods unplaced, because an empty node is cheaper. It needs a term that
rewards placed GPU demand, weighted so it dominates the node-count term.

---

## 4. Stages

Stages are ordered by dependency. Each one ends in a condition that can be checked. A
stage that produces measurements ends with a session record.

### Stage 1 — Calibrate the referee

Build the simulator (`go mod vendor && make`) at a recorded upstream commit, or use the
authors' image `qzweng/kubernetes-scheduler-simulator:atc23`. Run three cells from
`expected_run_scripts_0511.sh`: `default` × {FGD, Best-Fit, Random} × seed 42.

- Record the upstream SHAs of both `clusterdata` and the simulator. The trace-profile
  session record still lacks them.
- Check from the logs whether any pod in `default` is handled as a *deletion*
  (`SchedulePods` deletes pods carrying a deletion-time annotation). If deletions occur,
  the benchmark is not pure arrival, and §5 must model departures.
- Check that the logged pod sequence is identical across policies for the same seed. If
  it is not, every policy saw a different workload, and the comparison has to be made per
  policy.

**Done when:** our seed-42 FGD allocation at 130% lies within ±0.5 percentage points of
the published seed-42 value, and the session record states the observed difference.
If it lies outside that band, stop and find out why before continuing. A referee we
cannot reproduce cannot grade anyone.

**Cost:** roughly 10 minutes per cell on 2 vCPU, per the simulator's README.

### Stage 2 — Trace adapter (`bench/alibaba/`)

A converter that takes the trace CSVs plus one simulator log and writes Kuberina IR:

- `infra.yaml`: one node per row of `openb_node_list_gpu_node.csv`. `cpu_milli` → cores,
  `memory_mib` → GiB, the GPU model as a label, devices per G1/Stage 3. Throughput and
  storage are omitted, which IR v1 reads as unconstrained.
- `workloads.yaml`: one declaration per pod, in log order, with an explicit arrival
  index. Each `-tuned-<i>` clone resolves back to its source row.
- `sequence.csv`: arrival index, pod name, GPU milli, and for each online policy whether
  it placed that pod. This is what Stage 6 compares against.

**Done when:** the sum of GPU demand over `workloads.yaml` matches the simulator's
logged arrived GPU milli exactly. The adapter has tests built on a fixture log of a
dozen lines, with no network and no simulator run.

### Stage 3 — Per-device GPU model (closes G1)

- **IR:** add an optional `gpuDevices` field on a node (device count, milli per device)
  and an optional `gpuMilli` on a request. Both are additive under ADR-0002. When they are
  absent, the scalar `gpu` behaves exactly as today. Specify the fields in
  `docs/references/ir-v1.md` before writing code.
- **Solver:** keep the genome as node-per-pod. Device feasibility becomes a node-local
  decode: given the pods assigned to a node, pack fractional requests onto devices
  best-fit, and give multi-GPU requests whole free devices. A node is feasible if and
  only if that decode succeeds. GA operators do not change. Only `fits`, the load
  bookkeeping, and the overflow and fragmentation terms do.
- **Inspector:** check device feasibility with its own implementation, so that its
  verdict is an independent check on the solver.
- **Forge:** the device binding is for the planner's own feasibility check only.
  `kuberina-forge out` keeps pinning pods to nodes. On a real cluster, the GPU-sharing
  device plugin chooses the device.

**Done when:** a fixture with eight devices each 60% used rejects a 0.5-GPU pod in the
solver *and* in the inspector, and the MSC Irina benchmark reproduces its committed FFD
seed fitness to the digit.

### Stage 4 — Admission (closes G2 and G3)

- **Model:** an assignment value meaning *unplaced*. With admission on, FFD marks a pod
  that fits nowhere as unplaced.
- **Objective:** a term for unplaced GPU demand, weighted so that placing one more GPU
  pod always outweighs removing one active node. Derive the weight from the two terms'
  ranges on this instance and record the derivation, as was done for
  `min_relative_improvement` (#20).
- **Blueprint:** an additive `unplaced:` list and `metadata.unplaced` count. A blueprint
  that declines pods but breaches no capacity is `feasible: true`.
- **Admission mode:** `--admission in-order` admits pods in arrival index and may move
  any pod already admitted. `--admission free` may choose any subset. Without the flag,
  behavior is unchanged.

**Done when:** a fixture at 130% demand yields a feasible blueprint with a non-empty
`unplaced` list. Under `in-order`, no pod is unplaced while a later-arriving pod is
placed in its stead.

### Stage 5 — Scale check

MSC Irina (2,714 pods, 620 nodes) takes 185 s at `24ffc52`. The tuned default set is
about 10,800 pods on 1,213 nodes. Run one solve of the full tuned set for seed 42 and
record wall clock and memory before committing to the experiment grid.

**Done when:** a session record gives a measured cost per solve. §6 is then re-budgeted
from that number.

### Stage 6 — Experiments

All runs use `default` with seeds 42–51, the published seed set. Nothing is reported for
fewer than ten seeds without saying so.

| ID | Question | Kuberina mode | Compared against | Metric |
|---|---|---|---|---|
| **E1** (headline) | How far does each policy get before it first rejects a pod? | `in-order` on arrival prefixes. Binary-search the longest prefix placed in full. | First `failed to schedule` line in each policy's log | Arrived GPU % at first rejection, mean ± stdev over seeds |
| **E2** | What is moving placed pods worth? | `in-order` on the full 130% set | Published 130% allocation for each policy | GPU allocation %, plus pods rejected |
| **E3** | What is the offline ceiling? | `free` on the full 130% set | — (upper bound) | GPU allocation %, and the trivial bound min(100%, demand) |
| **E4** | What does it cost? | E2 runs | Simulator wall clock per seed | Seconds per run, and seconds per placed pod |
| **E5** (referee cross-check) | Does the simulator agree with our inspector? | Kuberina's E2 placements, loaded back into the simulator if it accepts pre-bound pods | Inspector verdict | Allocation and fragmentation as *the simulator* computes them |

E1 is chosen as the headline because it is the only metric where neither side has an
information advantage over the other: both see the same pods in the same order, and the
number is the point where they first fail. E1 relies on prefix packability being
monotone. It is monotone for an exact packer, but a heuristic may break it, so the
binary search records every probe and reports any non-monotone pair it hits.

E5 is optional and depends on whether the simulator honors `spec.nodeName` on input. If
it does, it is the strongest result in this plan, because Kuberina's output is then
scored by the other side's code.

**Done when:** one session record per experiment, with raw console output under
`docs/plans/benchmarks/alibaba-gpu-v2023/`.

### Stage 7 — Paper section

Written only from the Stage 1–6 session records, after they are merged. It reports E1 as
the comparison, E2 and E3 with their asymmetries stated in the same sentence as the
number, and E4 without softening. Per `AGENTS.md`, if a claim drafted here disagrees
with a record, the claim is wrong.

---

## 5. Threats to validity

These are stated now so the paper does not have to discover them later:

- **Offline against online.** E2 and E3 measure a different problem from the one FGD
  solves. Only E1 is like for like, and even E1 lets Kuberina re-plan the prefix from
  scratch.
- **Departures.** Resolved on 2026-10-31: `data/pod_csv_to_yaml.py` writes no
  deletion-time annotation (lines 119–120 are commented out), so the benchmark contains
  arrivals only and E1 can use arrival prefixes.
- **Tuned clones.** At 130%, about a quarter of the pods are duplicates of real ones. A
  packer that benefits from identical items gains from that. The gain comes from how the
  benchmark is built, and production clusters may show less of it.
- **Dimensions not modeled.** The simulator models CPU, memory and GPU. Kuberina's
  storage, disk and network dimensions are unconstrained here, so none of Kuberina's
  8-dimensional machinery is exercised. The paper cites this benchmark for CPU, memory
  and GPU only.
- **Heuristic.** Kuberina's GA is stochastic. Each seed is a pair: the trace seed and the
  solver seed. Both are recorded, and the solver seed is held fixed across trace seeds.

---

## 6. Budget (estimate, to be replaced by Stage 5)

| Item | Estimate |
|---|---|
| Stage 1, 3 cells | ~30 min on 2 vCPU |
| Stage 1, all 6 policies × 10 seeds, if the published values do not reproduce | ~10 h at the README's rate, parallelizable |
| One Kuberina solve, ~10,800 pods | unknown. Scaling Irina's 185 s by pod count gives roughly 12 min, but this is not a measurement. |
| E1 binary search, per seed | ~14 probes × one solve. Probes can be screened with FFD alone and confirmed by GA at the boundary. |
| E1 + E2 + E3, ten seeds | dominated by E1. Plan for a day of machine time until Stage 5 says otherwise. |

---

## 7. Beyond the default trace

The order is fixed by issue #24, and none of it starts before Stage 7:

1. The `gpuspec*` variants. GPU-type constraints are an OR over models
   (`V100M16|V100M32`), and IR v1's `nodeSelector` is equality-only. Supporting them
   needs a selector extension, which is a separate ROADMAP item.
2. `cluster-trace-gpu-v2026`, after its schema is read. Pod-level records mean a replay.
   Aggregate fact tables mean a synthetic generator calibrated to them.
3. `cluster-trace-v2018` container metadata, as the first real input for the `observed`
   block (K-1).

## 8. Decisions

Settled by the maintainer on 2026-10-31:

1. **E1 is the headline comparison.**
2. **The external simulator is built and run on the maintainer's workstation**, the
   same machine that produces Kuberina's own benchmark records.
3. **Stages 3 and 4 belong to Phase 2 (v0.4.0).** The IR fields they add are additive
   under ADR-0002 and land after v0.3.0 fixes the v1 surface.

Settled by the maintainer on 2026-11-01, for Stage 4:

4. **In-order admission means skip and continue.** Pod *i* is admitted if, and only if,
   the Pods admitted before it plus *i* can be packed. Kuberina may move Pods it has
   already admitted; a rejected Pod stays rejected. This is FGD's rule plus
   re-planning, which is what E2 measures.
5. **Arrival order is an IR field,** an optional `arrival` on a declaration.
6. **`kuberina-forge out` refuses a blueprint with unplaced Pods.** Admission serves the
   benchmark; applying a partial plan to a cluster is out of scope.
