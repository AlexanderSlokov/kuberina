# Kuberina v0.2.0: 8D MDBP Expansion & IR Format Upgrade

Nâng cấp Rust solver từ 3D (CPU, RAM, GPU) lên 8D (+ Storage, Disk R/W, Network I/O) và chuyển sang Kuberina IR v0.2.0 format với `namespaces:` grouping, `replicas:` unrolling, K8s unit parsing, và `topologySpread` constraint. Scope giới hạn trong Rust solver + testdata — `kuberina-forge` (Go) deferred to v0.3.0.

> [!IMPORTANT]
> **Scope v0.2.0 — chỉ Rust solver:**
> - ✅ 8D `ResourceVector`, IR v0.2.0 parser, K8s quantity parser, replica unrolling, topologySpread
> - ❌ `kuberina-forge` Go binary (v0.3.0)
> - ❌ PAPER.md rewrite (separate task after solver verified)

## Open Questions

> [!WARNING]
> **TopologySpread: Hard hay Soft?**
>
> Preplan đề xuất **Soft Penalty** ($w_5 \cdot f_{\text{spread}}$) cho v0.2.0.
> Plan này follow đề xuất đó. Nếu cần Hard constraint (`topologySpread.strict: true`), sẽ thêm trong v0.2.1.

> [!IMPORTANT]
> **Node default cho dimensions mới**
>
> Preplan đề xuất: Pod default = 0 (không dùng), Node default = `f64::MAX` (unconstrained).
> Plan follow đề xuất này. Homelab users có thể bỏ qua `disk`/`network` mà không bị hỏng.

## Proposed Changes

Sắp xếp theo dependency order — từ foundation data types lên tới CLI layer.

---

### Component 1: Data Model (8D Resource Vector)

Foundation layer — mọi module khác phụ thuộc vào đây.

#### [MODIFY] [model.rs](file:///home/stella/workspace/kuberina/solver/src/model.rs)

**`ResourceVector` struct (L24-L32):** Mở rộng từ 3 → 8 fields:
```rust
pub struct ResourceVector {
    pub cpu: f64,        // cores
    pub ram: f64,        // GiB
    pub gpu: f64,        // units
    pub storage: f64,    // GiB
    pub disk_read: f64,  // MB/s
    pub disk_write: f64, // MB/s
    pub net_in: f64,     // MB/s
    pub net_out: f64,    // MB/s
}
```
- Update `new()` → `new(cpu, ram, gpu)` giữ nguyên (backward compat cho tests), thêm `new_8d(...)` cho full 8D construction.
- Update `zero()`, `fits()`, `subtract()`, `is_zero()`, `Add impl`, `Default impl` to operate on all 8 fields.
- Size: 8×f64 = 64 bytes. Vẫn `Copy`, vẫn cache-friendly.

**`FfdWeights` struct (L240-L254):** Mở rộng từ 3 → 8 weights:
```rust
pub struct FfdWeights {
    pub alpha: f64,      // CPU
    pub beta: f64,       // RAM
    pub gamma: f64,      // GPU
    pub delta: f64,      // Storage
    pub epsilon_r: f64,  // Disk Read
    pub epsilon_w: f64,  // Disk Write
    pub zeta_in: f64,    // Net In
    pub zeta_out: f64,   // Net Out
}
```
- Default: `delta=0.1, epsilon_r=0.01, epsilon_w=0.01, zeta_in=0.01, zeta_out=0.01` — low weight vì I/O thường không phải bottleneck trừ khi user explicit.

**`Node` struct (L83-L94):** Thêm `rack: String` field (default empty).

**`Pod` struct (L100-L119):** Thêm:
- `replicas: usize` (serde default = 1, used by parser for unrolling, NOT stored in final Pod)
- `topology_spread: Option<TopologySpread>` (new struct)

**[NEW] `TopologySpread` struct:**
```rust
pub struct TopologySpread {
    pub max_skew: usize,
    pub topology_key: String, // "zone" or "rack"
}
```

**`Scorecard` struct (L166-L174):** Thêm `topology_spread_penalty: f64`.

**`FitnessWeights` struct (L217-L233):** Thêm `topology_spread: f64` (default = `3.0`).

**Tests (L256-L300):** Update `ResourceVector::new()` calls, thêm test cho 8D arithmetic.

---

### Component 2: K8s Quantity Parser

#### [NEW] [quantity.rs](file:///home/stella/workspace/kuberina/solver/src/quantity.rs)

Module mới để parse K8s resource notation thành f64. Tách riêng vì logic phức tạp, dễ test.

```rust
/// Parse K8s quantity strings into f64 values.
///
/// Supported suffixes:
///   - CPU: "300m" → 0.3, "4" → 4.0
///   - Memory: "512Mi" → 0.5 GiB, "8Gi" → 8.0, "2Ti" → 2048.0
///   - Throughput: "500M" → 500.0 MB/s, "10G" → 10000.0 MB/s
///
/// ```
/// # use kuberina_solver::quantity::parse_quantity;
/// assert!((parse_quantity("512Mi").unwrap() - 0.5).abs() < 1e-9);
/// assert!((parse_quantity("300m").unwrap() - 0.3).abs() < 1e-9);
/// assert!((parse_quantity("10G").unwrap() - 10000.0).abs() < 1e-9);
/// ```
pub fn parse_quantity(val: &str) -> Result<f64, String>
```

Suffix table:
| Suffix | Multiplier | Target unit | Use case |
|:-------|:-----------|:------------|:---------|
| `m` | ÷ 1000 | cores | CPU millicores |
| `Ki` | × (1/1048576) | GiB | Memory |
| `Mi` | × (1/1024) | GiB | Memory |
| `Gi` | × 1 | GiB | Memory |
| `Ti` | × 1024 | GiB | Memory |
| `k` | × 0.001 | MB/s | Throughput (decimal) |
| `M` | × 1 | MB/s | Throughput |
| `G` | × 1000 | MB/s | Throughput |

> [!NOTE]
> **Context-dependent parsing:** Cùng `"500M"` nhưng ý nghĩa khác nhau khi parse cho `ram` vs `disk.read`. Parser sẽ nhận thêm `context: QuantityContext` enum (`Memory`, `Throughput`, `Cpu`) để chọn đúng multiplier table. Suffix `M` trong context Memory = MiB → GiB, trong context Throughput = MB/s.

Tests: ≥15 cases covering edge cases (bare number, milliCPU, binary vs decimal, invalid suffix error message).

---

### Component 3: IR v0.2.0 Parser

#### [MODIFY] [parser.rs](file:///home/stella/workspace/kuberina/solver/src/parser.rs)

Refactor hoàn toàn parser để đọc IR v0.2.0 format.

**Raw deserialization structs (new):**
```rust
/// Raw YAML shape for IR v0.2.0 workloads.
/// Uses String for resources to support K8s notation like "512Mi".
struct RawWorkloadFile {
    namespaces: HashMap<String, Vec<RawPod>>,
}

struct RawPod {
    name: String,
    replicas: Option<usize>,      // default 1
    gang: Option<String>,
    requests: RawResourceRequests,
    node_selector: Option<HashMap<String, String>>,
    tolerations: Option<Vec<RawToleration>>,
    affinity: Option<Vec<String>>,
    anti_affinity: Option<Vec<String>>,
    topology_spread: Option<RawTopologySpread>,
}

struct RawResourceRequests {
    cpu: Option<ResourceValue>,   // f64 or String
    ram: Option<ResourceValue>,
    gpu: Option<ResourceValue>,
    storage: Option<ResourceValue>,
    disk: Option<RawDiskIO>,
    network: Option<RawNetworkIO>,
}

struct RawDiskIO { read: Option<ResourceValue>, write: Option<ResourceValue> }
struct RawNetworkIO { in_: Option<ResourceValue>, out: Option<ResourceValue> }
```

**`ResourceValue`**: Custom deserializer accepting both `f64` (backward compat: `cpu: 0.3`) and `String` (new: `ram: "512Mi"`). Uses `quantity.rs` for string parsing.

**Key transformations:**
1. **Namespace flattening:** `namespaces.monitoring[0]` → `Pod { namespace: "monitoring", ... }`
2. **Replica unrolling:** `name: grafana, replicas: 3` → `grafana-0000`, `grafana-0001`, `grafana-0002` (each independent Pod in the search space)
3. **Gang auto-grouping:** Scan all pods for unique `gang` values → auto-generate `PodGroup` list. `min_members = count of pods in that gang group`.
4. **Taint format:** `RawToleration { key, operator, value, effect }` → flatten to simple `String` key for CSP matching (matching existing taint model).

**Raw infra structs (update):**
```rust
struct RawInfraFile {
    nodes: Vec<RawNode>,
    daemonsets: Option<Vec<RawDaemonSet>>,
}

struct RawNode {
    name: String,
    zone: Option<String>,
    rack: Option<String>,         // NEW
    labels: Option<HashMap<String, String>>,
    taints: Option<Vec<RawTaint>>,
    allocatable: RawResourceRequests, // reuse same struct, K8s units
}
```

**Node resource defaults:** Khi dimension bị bỏ qua (e.g., homelab không khai báo `disk`), default = `f64::MAX` (unconstrained). Bảo đảm homelab testdata v0.1.x vẫn hoạt động qua forge/manual conversion.

**Tests:**
- Parse IR v0.2.0 sample files
- Verify replica unrolling: 18 replicas → 18 pods with correct names
- Verify K8s quantity round-trip: `"512Mi"` → `0.5` GiB
- Verify gang auto-grouping from inline `gang:` field
- Verify namespace flattening

---

### Component 4: CSP Constraint Enforcement (8D)

#### [MODIFY] [csp.rs](file:///home/stella/workspace/kuberina/solver/src/csp.rs)

**`compute_capacity_overflow()` (L43-L61):** Mở rộng từ 3D → 8D overflow check:
```rust
overflow += (load.storage - cap.storage).max(0.0);
overflow += (load.disk_read - cap.disk_read).max(0.0);
overflow += (load.disk_write - cap.disk_write).max(0.0);
overflow += (load.net_in - cap.net_in).max(0.0);
overflow += (load.net_out - cap.net_out).max(0.0);
```

WHY: Đây chính là nơi Noisy Neighbor bị detect — 10 DB pods nhồi trên 1 node sẽ trigger massive `disk_read` overflow penalty, buộc GA di chuyển chúng.

**Tests:** Update helper `node()` constructors, thêm test cho Disk/Network overflow detection.

---

### Component 5: Fitness Function (8D + TopologySpread)

#### [MODIFY] [fitness.rs](file:///home/stella/workspace/kuberina/solver/src/fitness.rs)

**`compute_fragmentation()` (L66-L79):** Thêm 5 dimensions vào waste calculation:
```rust
total_waste += (cap.storage - load.storage).max(0.0);
total_waste += (cap.disk_read - load.disk_read).max(0.0);
// ... etc
```

> [!WARNING]
> **Fragmentation với `f64::MAX` nodes:** Nếu node có `disk_read = f64::MAX` (unconstrained) và load = 100, thì waste = `f64::MAX` → phá fitness. Cần guard: `if cap.disk_read < f64::MAX { total_waste += ... }`. Skip unconstrained dimensions từ waste calc.

**[NEW] `compute_topology_spread_penalty()`:**
```rust
/// f_spread: penalize uneven distribution across topology zones/racks.
///
/// For each pod with topologySpread, compute actual skew vs maxSkew.
/// Penalty = Σ max(0, actual_skew - max_skew) per topology group.
pub fn compute_topology_spread_penalty(
    assignment: &[usize],
    pods: &[Pod],
    nodes: &[Node],
) -> f64
```

Logic:
1. Group pods by their `topology_spread.topology_key` value
2. For each group, count pods per zone/rack
3. Compute `skew = max_count - min_count`
4. Penalty += `max(0, skew - max_skew)` per group

**`compute_fitness()` (L17-L45):** Thêm `w5 * f_spread` vào weighted sum.

**`compute_utilization_variance()` (L116-L133):** Hiện chỉ tính CPU variance. Giữ nguyên — variance trên 8 dimensions phức tạp và không rõ benefit. CPU variance đủ đại diện cho "vessel stability".

**Tests:** Update constructors, thêm test cho topology spread penalty.

---

### Component 6: Phase 0 — DaemonSet Pre-deduction (8D)

#### [MODIFY] [phase0.rs](file:///home/stella/workspace/kuberina/solver/src/phase0.rs)

**`pre_deduct_daemonsets()` (L69-L87):** Không cần thay đổi logic — `ResourceVector::subtract()` đã tự động handle 8D nhờ Component 1 update. Chỉ cần update `Node` struct clone to include `rack`.

**Tests:** Update helper `make_node()` to include `rack` field.

---

### Component 7: Phase 1 — FFD Warm-Start (8D)

#### [MODIFY] [phase1_ffd.rs](file:///home/stella/workspace/kuberina/solver/src/phase1_ffd.rs)

**`synthetic_volume()` (L31-L34):** Mở rộng formula:
```rust
pub fn synthetic_volume(pod: &Pod, weights: &FfdWeights) -> f64 {
    let r = &pod.requests;
    weights.alpha * r.cpu
        + weights.beta * r.ram
        + weights.gamma * r.gpu
        + weights.delta * r.storage
        + weights.epsilon_r * r.disk_read
        + weights.epsilon_w * r.disk_write
        + weights.zeta_in * r.net_in
        + weights.zeta_out * r.net_out
}
```

**Tests:** Update `synthetic_volume_calculation` test, thêm test cho 8D volume.

---

### Component 8: Phase 2 — GA Optimizer

#### [MODIFY] [phase2_ga.rs](file:///home/stella/workspace/kuberina/solver/src/phase2_ga.rs)

**Không thay đổi logic.** GA operates on `assignment: Vec<usize>` — dimension-agnostic. Fitness evaluation đã handle 8D qua Component 5. Gang repair, crossover, mutation — tất cả independent of resource dimensions.

**Tests:** Update helper `pod()` và `node()` constructors cho `rack` field.

---

### Component 9: CLI + Output (8D display)

#### [MODIFY] [main.rs](file:///home/stella/workspace/kuberina/solver/src/main.rs)

**`run_plan()` Pareto scaling (L73-L82):** Mở rộng scaling cho 8D:
```rust
node.allocatable.storage *= factor;
node.allocatable.disk_read *= factor;
// ... etc — but only if != f64::MAX (unconstrained)
```

**`print_phase0_summary()` (L136-L150):** Thêm display cho Storage, Disk, Network overhead.

**`print_all_nodes()` (L191-L223):** Thêm Disk I/O và Network utilization display.

**`print_blueprint()` YAML export (L178-L188):** Include namespace in exported YAML key: `namespace/pod: node`.

---

### Component 10: Module Registration

#### [MODIFY] [lib.rs](file:///home/stella/workspace/kuberina/solver/src/lib.rs)

Thêm `pub mod quantity;` cho K8s quantity parser module.

---

### Component 11: Testdata IR v0.2.0

#### [MODIFY] [homelab_infra.yaml](file:///home/stella/workspace/kuberina/solver/testdata/homelab_infra.yaml)

Convert sang IR v0.2.0 format:
- Resource values → K8s units (`ram: 16.0` → `ram: "16Gi"`)
- Add `rack` field (default `rack-home`)
- Add `disk` and `network` allocatable (sensible defaults for ThinkCentre M720q: SATA SSD ~500MB/s, 1Gbps NIC)
- DaemonSet resources → 8D with K8s units
- Taint format → structured `{ key, value, effect }`

#### [MODIFY] [homelab_workloads.yaml](file:///home/stella/workspace/kuberina/solver/testdata/homelab_workloads.yaml)

Convert sang IR v0.2.0 format:
- Top-level `pods:` → `namespaces:` grouped dict
- Resource values → K8s units
- Add `disk` and `network` requests (sensible defaults per service: jellyfin high disk read, postgres high disk write, etc.)
- Remove `groups: []` → gangs via inline `gang:` field (none for homelab)

---

### Component 12: Cargo.toml Version Bump

#### [MODIFY] [Cargo.toml](file:///home/stella/workspace/kuberina/solver/Cargo.toml)

- Version: `"0.1.0"` → `"0.2.0"`

---

## Verification Plan

### Automated Tests

```bash
cd solver && cargo test
```

Expected:
1. `quantity::tests` — ≥15 cases for K8s unit parsing
2. `parser::tests` — IR v0.2.0 parsing, replica unrolling, gang auto-grouping
3. `model::tests` — 8D `ResourceVector` arithmetic
4. `csp::tests` — 8D capacity overflow (Noisy Neighbor detection)
5. `fitness::tests` — 8D fragmentation, topology spread penalty
6. `phase1_ffd::tests` — 8D synthetic volume
7. `phase2_ga::tests` — GA convergence (dimension-agnostic, should pass unchanged)
8. Existing tests updated to compile with new struct fields

### Integration Test

```bash
cd solver && cargo run --release -- plan --infra testdata/homelab_infra.yaml --workloads testdata/homelab_workloads.yaml
```

Expected:
- Fitness finite, zero hard constraint violations
- 8D resource utilization displayed in output
- `kuberina_solution.yaml` exported with namespace/pod keys

### Build Verification

```bash
cd solver && cargo clippy -- -D warnings && cargo fmt --check
```
