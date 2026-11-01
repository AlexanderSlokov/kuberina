# Session record — MSC Irina with affinity in force — 2026-11-01

**Commit measured:** `096177e`, "follow Kubernetes affinity semantics and regenerate MSC
Irina as IR v1 (#26)".
**Compared against:** `24ffc52`, recorded in `2026-09-06-ga-early-stop-threshold.md`.
**Host:** Linux 7.0.0-38-generic x86_64, 8 vCPU, 13 GiB RAM.
**Raw output:** `docs/plans/benchmarks/v0.3.0-affinity/`.

The two commits do not solve the same problem. At `24ffc52`, the solver received no
affinity or anti-affinity rule (see `2026-11-01-affinity-constraints-unread.md`). At
`096177e`, it receives the declaration-level rules below. The comparison shows what the
rules cost and what the solver achieves against them. It is not a before-and-after of
one instance.

## Instance

The infrastructure file is byte-identical between the two commits: 620 nodes and 4
DaemonSets. The workloads keep the same 2,714 Pods in 49 services, with these rules:

| Rule | Declaration-level rules | Pod-level size |
|---|---|---|
| Spread services repel their own replicas | 14 | 35,232 pairs |
| Cross-tier anti-affinity | 5 | 11,440 pairs |
| Affinity chain links (each Pod wants one Pod of the next service) | 10 | 674 entries |

At `24ffc52`, the generator's spread filter read the disk-read column instead of the
spread flag, so it marked all 49 services. The 14 above are the services that carry the
flag.

## Commands

```shell
make solver-irina              # full packing
make inspector-run             # equivalent run with --output in a scratch directory
make bench-proof
make solver-irina-headroom-20
make bench-proof-headroom-20
```

## Full packing

| | `24ffc52` (no rules) | `096177e` (rules in force) |
|---|---|---|
| Generations | 200 / 1,000 | 330 / 1,000 |
| Wall clock | 185.23 s | **864.67 s** |
| Peak RSS | 164,884 KiB | 171,652 KiB |
| FFD seed fitness | 1,582,568.1260 | 1,587,653.1260 |
| Final fitness | 1,582,556.1257 | 1,587,391.1255 |
| Active nodes | 540 / 620 | 540 / 620 |
| Fragmentation | 1,577,120.00 | 1,577,120.00 |
| Affinity violations, FFD seed | 0 | 1,017 |
| Affinity violations, final (solver) | 0 | **964** |
| Affinity violations, final (inspector) | not checked | **964** |
| Topology spread penalty (solver) | — | 17.00 |
| Capacity, selector, gang penalty | 0 | 0 |
| Verdict | FEASIBLE | FEASIBLE |
| Approximation ratio | α = 1.5297 (LB 353) | α = 1.5297 (LB 353) |

The run stopped with `Early stop at generation 329 (gain under 0.0100% for 200 gens)`.

## 20% headroom

| | `24ffc52` (no rules) | `096177e` (rules in force) |
|---|---|---|
| Generations | 200 / 1,000 | 429 / 1,000 |
| Wall clock | 184.23 s | **1,097.14 s** |
| Peak RSS | — | 172,876 KiB |
| FFD seed fitness | 2,305,971.1293 | 2,311,466.1293 |
| Final fitness | 2,305,938.1245 | 2,310,768.1128 |
| Active nodes | 615 / 620 | 615 / 620 |
| Affinity violations, FFD seed | 0 | 1,099 |
| Affinity violations, final (solver) | 0 | **960** |
| Affinity violations, final (inspector) | not checked | **960** |
| Topology spread penalty (solver) | — | 22.00 |
| Verdict | FEASIBLE | FEASIBLE |
| Approximation ratio | α = 1.3946 (LB 441) | α = 1.3946 (LB 441) |

The inspector's breakdown of the 960 violations in this solution: 602 of the 674
affinity entries are unmet, and 358 anti-affinity pairs share a node, of which 333 are
replicas of one spread service and 25 are cross-tier pairs.

In both configurations, the proof script reports feasibility proven and p < 1.0e-04
against 10,000 selector-aware random trials. Its topology spread figure (1,700 and
2,200) is the script's own measure, which groups by declaration. The solver's figure
(17 and 22) still groups by name and ignores Kubernetes zone keys (backlog item K-2).

## Observations

1. **The rules cost 4.7× and 6.0× in wall clock.** Seconds per generation rose from
   0.93 to 2.62 (full packing) and from 0.92 to 2.56 (headroom).
2. **The GA removes few violations.** It removes 53 of 1,017 (5.2%) in full packing and
   139 of 1,099 (12.6%) with headroom. Active nodes and fragmentation do not move.
3. **The objective gives the rules almost no weight.** With the default weights,
   affinity contributes 5 × 964 = 4,820 to a fitness of 1,587,391 (0.30%), while
   fragmentation contributes 1,577,120 (99.35%). A move that removes one violation is
   worth less than a move that frees 5 units of fragmentation.

## Fitness cost per term

Measured with a temporary example, `solver/examples/fitness_timing.rs` (not committed),
on the FFD seed of the full-packing instance, release build, 50 calls per term:

```shell
cargo run --release --example fitness_timing
```

| Term | ms per call |
|---|---|
| `compute_fitness`, total | 12.313 |
| `compute_affinity_violations` | 8.959 |
| `compute_topology_spread_penalty` | 3.265 |
| `compute_fragmentation` | 0.005 |
| `compute_utilization_variance` | 0.005 |
| `count_active_nodes` | 0.074 |

At 128 evaluations per generation, fitness accounts for about 1.58 s of the 2.62 s per
generation. The affinity term resolves targets by string on every call: it rebuilds a
map of 2,714 identities and hashes about 82,000 target strings. The topology spread term
scans every Pod for every spread Pod.
