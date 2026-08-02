# kuberina-solver — Rust Port Walkthrough

## What was done

Straight 1:1 port of the Python research solver into a Rust crate at `solver/`.

### Files created (9 source + 1 integration test)

| File | Lines | Port source | Key decisions |
|---|---|---|---|
| [Cargo.toml](file:///home/stella/workspace/kuberina/solver/Cargo.toml) | 18 | — | Edition 2024, 4 deps: serde, rayon, rand, clap |
| [lib.rs](file:///home/stella/workspace/kuberina/solver/src/lib.rs) | 7 | — | Module wiring |
| [model.rs](file:///home/stella/workspace/kuberina/solver/src/model.rs) | 288 | [types.py](file:///home/stella/workspace/kuberina/research/src/kuberina/model/types.py) | `ResourceVector: Copy` + `impl Add`, all types derive `Deserialize` |
| [parser.rs](file:///home/stella/workspace/kuberina/solver/src/parser.rs) | 97 | [parser.py](file:///home/stella/workspace/kuberina/research/src/kuberina/parser.py) | serde_yaml auto-maps, simpler than Python |
| [phase0.rs](file:///home/stella/workspace/kuberina/solver/src/phase0.rs) | 116 | [phase0.py](file:///home/stella/workspace/kuberina/research/src/kuberina/phases/phase0.py) | Iterator chains |
| [csp.rs](file:///home/stella/workspace/kuberina/solver/src/csp.rs) | 219 | [csp.py](file:///home/stella/workspace/kuberina/research/src/kuberina/phases/csp.py) | All hard constraint checks |
| [fitness.rs](file:///home/stella/workspace/kuberina/solver/src/fitness.rs) | 216 | [fitness.py](file:///home/stella/workspace/kuberina/research/src/kuberina/fitness.py) | Sample variance (N-1) matching Python `statistics.variance()` |
| [phase1_ffd.rs](file:///home/stella/workspace/kuberina/solver/src/phase1_ffd.rs) | 136 | [phase1_ffd.py](file:///home/stella/workspace/kuberina/research/src/kuberina/phases/phase1_ffd.py) | FFD greedy packing |
| [phase2_ga.rs](file:///home/stella/workspace/kuberina/solver/src/phase2_ga.rs) | 275 | [phase2_ga.py](file:///home/stella/workspace/kuberina/research/src/kuberina/phases/phase2_ga.py) | **`rayon::par_iter_mut()`** cho parallel fitness eval |
| [main.rs](file:///home/stella/workspace/kuberina/solver/src/main.rs) | 211 | [\_\_main\_\_.py](file:///home/stella/workspace/kuberina/research/src/kuberina/__main__.py) | clap derive, progress bar, auto-scale GA config |
| [integration.rs](file:///home/stella/workspace/kuberina/solver/tests/integration.rs) | 82 | [test_integration.py](file:///home/stella/workspace/kuberina/research/tests/test_integration.py) | Full pipeline homelab scenario |

**Total: ~1,647 lines Rust** (vs ~700 lines Python — includes tests inline + doc-tests).

### Design decisions

1. **`ResourceVector: Copy`** — 24 bytes on stack, no heap allocation. This is the cache contiguity play from preplan.md §2.
2. **`impl Add for ResourceVector`** — use `+` operator instead of `.add()` method. Clippy-clean.
3. **`rayon` only in `evaluate_all()`** — one line change from `.iter_mut()` to `.par_iter_mut()`. Rest stays sequential (needs `&mut` exclusive access).
4. **`gen` → `generation`** — `gen` is a reserved keyword in Rust edition 2024.
5. **Testdata copied** into `solver/testdata/` — `cargo test` is self-contained.

## Test results

```
48/48 passed:
  37 unit tests (inline #[cfg(test)])
   3 integration tests (tests/integration.rs)
   8 doc-tests
cargo clippy: 0 warnings
```

## Verification commands

```bash
# Root Makefile
make solver-build     # Build
make solver-test      # Run all tests
make solver-clippy    # Lint
make solver-homelab   # Run on 3-node homelab (compare with Python)
make solver-irina     # Run on 150-node MSC Irina stress test (release mode)

# Compare with Python
make research-homelab
```

> [!NOTE]
> Fitness values between Python and Rust won't match bit-for-bit (different RNG implementations), but both should:
> - Place USB-dongle pods on thinkcentre-beta
> - Have zero capacity overflow
> - Produce fitness in the same order of magnitude
