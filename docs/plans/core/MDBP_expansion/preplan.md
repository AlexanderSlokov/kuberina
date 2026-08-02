# Kuberina: Proposal for v0.2.0 (The MDBP Expansion & Kuberina IR)

- **Quyết định:** Đã được chấp thuận (Accepted) cho Kuberina v0.2.0.
- **Mục tiêu:** Nâng cấp thuật toán Multi-Dimensional Bin Packing từ 3 chiều (CPU, RAM, GPU) lên thành ma trận 8 chiều (bổ sung thêm Storage, Disk I/O R/W, Network I/O In/Out) để giải quyết triệt để bài toán "Noisy Neighbor".

---

## 1. Tổng quan về kiến trúc mới của Kuberina

### 1.1. Triết lý: Compiler Pipeline (rustc → LLVM IR → Binary)

Lấy cảm hứng từ kiến trúc compiler của Rust: `rustc` phân tích source code rồi dịch ra **LLVM IR** — một biểu diễn trung gian chuẩn hóa. Sau đó, LLVM backend (hoàn toàn tách biệt) biên dịch IR thành machine code cho target platform.

Kuberina áp dụng cùng triết lý:

```
┌─────────────────────────────────────┐
│         kuberina-forge (Go)         │  ← "Frontend compiler"
│  Đọc K8s manifests, Helm, Kustomize│
│  Kết nối API hạ tầng (cloud, bare) │
│  Sinh file Kuberina IR:            │
│    _infra.yaml + _workloads.yaml   │
└──────────────┬──────────────────────┘
               │  Kuberina IR (YAML)
               ▼
┌─────────────────────────────────────┐
│       kuberina-solver (Rust)        │  ← "Backend compiler"
│  Nhận IR, giải MDBP, sinh blueprint│
│  Không biết gì về K8s API          │
│  Chỉ biết: nodes, pods, resources  │
└──────────────┬──────────────────────┘
               │  blueprint.yaml
               ▼
┌─────────────────────────────────────┐
│       kuberina-forge (Go)           │  ← "Linker"
│  Đọc blueprint → sinh output cho   │
│  target system (Helm, Kustomize,   │
│  ArgoCD, Terraform, Nomad,...)     │
└─────────────────────────────────────┘
```

### 1.2. Tại sao tên `kuberina-forge`?

Đánh giá các tên ứng cử:

| Tên             | Ý nghĩa                                           | Đánh giá       |
|:----------------|:---------------------------------------------------|:----------------|
| `operator`      | Xung đột ngữ nghĩa với K8s Operator (CRD controller) | ❌ Gây nhầm lẫn |
| `helper`        | Quá mơ hồ, không mô tả được vai trò cốt lõi       | ❌ Thiếu identity |
| `compiler`      | Chính xác về mặt kỹ thuật nhưng quá generic        | ⚠️ Chấp nhận được |
| `emitter`       | Mô tả hành vi "emit IR" nhưng bỏ qua chiều ngược  | ⚠️ Một chiều     |
| **`forge`**     | Lò rèn — nơi nguyên liệu thô (manifests) được **nung** thành sản phẩm chuẩn hóa (IR), và cũng là nơi blueprint được **đúc** thành output cuối cùng | ✅ **Đề xuất**  |

**`kuberina-forge`** — binary Go riêng biệt, làm hai nhiệm vụ:
- **Forge-in (emit):** Đọc K8s manifests / Helm / Kustomize / cloud API → sinh Kuberina IR
- **Forge-out (render):** Đọc `blueprint.yaml` → sinh output cho target platform

### 1.3. Lợi ích kiến trúc Compiler Pipeline

1. **Decoupling triệt để:** Rust solver không bao giờ import `client-go`, `helm`, hay bất kỳ SDK hạ tầng nào. Nó chỉ đọc IR — giống LLVM chỉ đọc IR, không biết ngôn ngữ nguồn.

2. **Integration flexibility vô hạn:** `kuberina-forge` là Go binary, có thể cắm mọi package:
   - `client-go` — đọc cluster state trực tiếp
   - `helm.sh/helm/v3` — parse Helm charts
   - `sigs.k8s.io/kustomize` — parse Kustomize overlays
   - Cloud SDKs (AWS, GCP, Azure) — đọc node pools từ cloud API
   - Terraform state — đọc infra đã provision

3. **Non-K8s MDBP:** Vì solver chỉ biết IR (nodes + pods + resources), hoàn toàn có thể viết forge adapter cho:
   - **HashiCorp Nomad** — job scheduling trên Nomad clusters
   - **Docker Swarm** — service placement
   - **Bare-metal HPC** — job scheduling trên Slurm/PBS clusters
   - **Thậm chí maritime thật** — quay về bài toán stowage gốc

4. **Testability:** Solver test hoàn toàn bằng static IR files, không cần mock K8s API.

### 1.4. Các script Python hỗ trợ (giữ nguyên)

Scripts Python hiện tại (`research/`) cho visualization, validation, benchmark generation — giữ nguyên, không cần đóng gói thành binary.

---

## 2. Đề xuất kiến trúc Rust core: Hexagonal Architecture (Ports & Adapters)

### 2.1. Tại sao Hexagonal thay vì Layered?

So sánh hai lựa chọn cho Rust solver:

| Tiêu chí                    | Layered (Onion)                        | Hexagonal (Ports & Adapters)           |
|:------------------------------|:---------------------------------------|:---------------------------------------|
| **Dependency direction**      | Từ trên xuống, layer phụ thuộc layer dưới | Domain core không phụ thuộc gì cả      |
| **I/O isolation**             | Parser/output nằm trong các layer riêng nhưng domain vẫn "biết" layer nào gọi nó | Domain chỉ biết `trait Port`, adapter implement trait |
| **Testability**               | Tốt, nhưng phải mock cả layer         | Xuất sắc — swap adapter bằng in-memory fake |
| **Phù hợp Rust trait system** | Trung bình — Rust không có inheritance | Hoàn hảo — Rust traits = ports tự nhiên |
| **Future extensibility**      | Thêm layer mới = refactor dependency chain | Thêm adapter mới = implement trait, zero chạm core |

**Đề xuất: Hexagonal Architecture** — lý do chính:

1. **Rust traits là ports tự nhiên.** `trait InfraPort { fn load_nodes(&self) -> Vec<Node>; }` — bất kỳ adapter nào (YAML file, stdin, in-memory test) implement trait này đều plug vào solver mà domain code không thay đổi.

2. **Coherent với compiler pipeline.** Solver core (hexagon center) chỉ biết domain types (`Node`, `Pod`, `ResourceVector`) và algorithms (FFD, GA, CSP). Mọi I/O đều đi qua ports — chính xác như LLVM core không biết file system.

3. **8-dimensional expansion dễ dàng.** Khi mở rộng `ResourceVector` từ 3 → 8 chiều, chỉ domain core thay đổi. Adapters (parser, output) thay đổi independently.

### 2.2. Cấu trúc thư mục đề xuất

```
solver/src/
├── domain/                    # 🎯 Hexagon Core — pure logic, zero I/O
│   ├── model.rs               # ResourceVector (8D), Node, Pod, Blueprint,...
│   ├── fitness.rs             # Objective function F(s) = w₁·f_nodes + ...
│   ├── csp.rs                 # Hard constraint enforcement + Forward Checking
│   ├── phase0.rs              # DaemonSet pre-deduction (ballast water)
│   ├── phase1_ffd.rs          # FFD warm-start (vector packing)
│   └── phase2_ga.rs           # Genetic Algorithm (selection, crossover, mutation)
│
├── ports/                     # 🔌 Port traits — domain-owned interfaces
│   ├── input_port.rs          # trait InfraSource, trait WorkloadSource
│   └── output_port.rs         # trait BlueprintSink
│
├── adapters/                  # 🔧 Adapter implementations — I/O lives here
│   ├── yaml_reader.rs         # Reads Kuberina IR v0.2.0 YAML files
│   ├── yaml_writer.rs         # Writes blueprint.yaml output
│   └── quantity_parser.rs     # K8s unit parser: "512Mi" → 0.5 GiB
│
├── lib.rs                     # Public API surface
└── main.rs                    # CLI entrypoint, wiring adapters → ports
```

### 2.3. Ví dụ Port trait

```rust
/// Port for reading infrastructure data.
/// Domain-owned: domain defines WHAT it needs, adapters decide HOW to get it.
pub trait InfraSource {
    fn load_nodes(&self) -> Result<Vec<Node>, SolverError>;
    fn load_daemonsets(&self) -> Result<Vec<DaemonSet>, SolverError>;
}

/// Port for reading workload data.
pub trait WorkloadSource {
    fn load_pods(&self) -> Result<Vec<Pod>, SolverError>;
    fn load_groups(&self) -> Result<Vec<PodGroup>, SolverError>;
}

/// Port for writing solution output.
pub trait BlueprintSink {
    fn write(&self, blueprint: &Blueprint, nodes: &[Node], pods: &[Pod]) -> Result<(), SolverError>;
}
```

### 2.4. Lưu ý: Refactor vs Rewrite

Cấu trúc hiện tại (`solver/src/*.rs` flat) vẫn đúng logic, chỉ thiếu tổ chức. Đề xuất:
- **Phase 1 (v0.2.0):** Tập trung vào 8D expansion + IR format mới. Giữ flat structure, nhưng tách `quantity_parser.rs` riêng.
- **Phase 2 (v0.3.0):** Refactor sang hexagonal khi codebase đủ lớn để cần nó. Không refactor sớm — premature architecture là kẻ thù.

---

## 3. Kuberina IR v0.2.0 Specification

Dưới đây là định dạng chính thức của **Kuberina IR (Intermediate Representation)** dùng làm ngôn ngữ giao tiếp trực tiếp với Rust core engine. Kuberina IR ưu tiên sự tinh gọn, các đơn vị tài nguyên chuẩn hóa, hỗ trợ nhân bản (replicas), và gang-scheduling.

### 3.1. Workloads IR (`_workloads.yaml`)

```yaml
namespaces:
  monitoring:
    - name: grafana
      replicas: 1 # Tự động tạo grafana-0
      requests:
        cpu: 0.3
        ram: "512Mi"    # Sử dụng chuẩn đơn vị K8s (Mi, Gi)
        storage: "3Gi"
        disk:
          read: "15M"   # Tốc độ IOPS/Throughput (M = Megabytes/s)
          write: "5M"
        network:
          in: "4M"      # Tốc độ mạng
          out: "7M"
      affinity:
        - prometheus
      antiAffinity:
        - kibana

  ai:
    - name: llm-inference
      replicas: 18      # Tự động bung ra llm-inference-0000 -> 0017, tiết kiệm dung lượng file
      gang: "llm-training-job" # Gang-scheduling: Bắt buộc phải schedule đủ 18 pods cùng lúc
      requests:
        cpu: 6.0
        ram: "24Gi"
        gpu: 1
        storage: "50Gi"
        disk:
          read: "500M"  # Cần đọc model nặng liên tục -> cần node có Disk R cao
          write: "10M"
        network:
          in: "100M"    # Đồng bộ gradient giữa các máy
          out: "100M"
      nodeSelector:
        accelerator: nvidia-a100
      tolerations:
        - key: "gpu-node"
          operator: "Exists"
      topologySpread: 
        maxSkew: 1
        topologyKey: "zone" # Không cho phép nhồi quá nhiều replicas vào cùng 1 zone
      antiAffinity:
        - llm-inference     # Chống xếp 2 pod inference vào cùng 1 node
```

### 3.2. Infrastructure IR (`_infra.yaml`)

```yaml
nodes:
  - name: dgx-01
    zone: us-east-1a
    rack: rack-A
    labels:
      node-role: worker
      accelerator: nvidia-a100
    taints:
      - key: "gpu-node"
        value: "true"
        effect: "NoSchedule"
    allocatable:
      cpu: 64.0
      ram: "256Gi"
      gpu: 8
      storage: "2Ti"
      disk:
        read: "5G"  # Trần băng thông ổ cứng (NVMe PCIe Gen 4)
        write: "5G"
      network:
        in: "10G"   # Giới hạn Network Interface Card (10Gbps)
        out: "10G"
```

---

## 4. Gap Analysis: IR v0.2.0 vs Testdata hiện tại (v0.1.x)

So sánh Kuberina IR v0.2.0 spec ở trên với hai file testdata hiện tại mà solver đang parse được:

### 4.1. Infra: `homelab_infra.yaml` (v0.1.x) → IR v0.2.0

| Thay đổi | v0.1.x (hiện tại) | v0.2.0 (mới) | Đánh giá |
|:---------|:-------------------|:-------------|:---------|
| **Resource dimensions** | `cpu`, `ram`, `gpu` (f64 raw) | + `storage`, `disk.read`, `disk.write`, `network.in`, `network.out` | ✅ Hợp lý — giải Noisy Neighbor |
| **Resource units** | Bare floats (`cpu: 4.0`, `ram: 16.0`) | K8s strings (`ram: "256Gi"`, `disk.read: "5G"`) | ✅ Hợp lý — nhất quán với K8s |
| **Nested resources** | Flat (`cpu`, `ram`, `gpu`) | Nested (`disk.read`, `disk.write`, `network.in`, `network.out`) | ✅ Hợp lý — nhóm logic, dễ đọc |
| **Rack topology** | Không có | `rack: rack-A` | ✅ Hợp lý — cần cho topologySpread |
| **Taint format** | Simple strings (`taints: []`) | Structured (`key`, `value`, `effect`) | ✅ Hợp lý — match K8s taint model |
| **DaemonSet resources** | 3D raw floats | Cần mở rộng thành 8D + K8s units | ⚠️ Cần bổ sung trong spec |

### 4.2. Workloads: `homelab_workloads.yaml` (v0.1.x) → IR v0.2.0

| Thay đổi | v0.1.x (hiện tại) | v0.2.0 (mới) | Đánh giá |
|:---------|:-------------------|:-------------|:---------|
| **Top-level key** | `pods:` (flat list) | `namespaces:` (grouped by namespace) | ✅ Hợp lý — tự nhiên hơn, giảm boilerplate |
| **Namespace** | Per-pod field (`namespace: dns`) | Namespace là key của dict | ✅ Hợp lý — DRY, giống Kustomize |
| **Replicas** | Không có (mỗi pod viết riêng) | `replicas: N` → unroll thành N pods | ✅ Hợp lý — tiết kiệm cho AI workloads |
| **Gang scheduling** | `group:` field + `groups:` top-level | `gang: "group-name"` inline per-pod | ✅ Đơn giản hơn — không cần `groups:` section |
| **Resource dimensions** | `cpu`, `ram` (f64 raw) | + `storage`, `disk.*`, `network.*` (K8s units) | ✅ Hợp lý |
| **topologySpread** | Không có | `topologySpread: {maxSkew, topologyKey}` | ✅ Hợp lý — K8s native feature |
| **Toleration format** | Simple strings | Structured (`key`, `operator`) | ✅ Hợp lý — match K8s API |

### 4.3. Các điểm cần bổ sung / làm rõ trong IR v0.2.0

1. **DaemonSets trong IR v0.2.0:** Spec hiện tại chưa cho ví dụ DaemonSet với 8D resources. Cần bổ sung ví dụ DaemonSet section trong `_infra.yaml` IR spec.

2. **Groups section bị loại bỏ:** v0.1.x có `groups: []` top-level. v0.2.0 dùng `gang:` inline. Cần xác nhận: `PodGroup` struct có được tự động sinh từ `gang:` field khi parse không? (Đề xuất: có — parser scan tất cả `gang:` values, deduplicate, sinh `PodGroup` automatically.)

3. **Backward compatibility:** Solver v0.2.0 có cần đọc cả format v0.1.x (flat `pods:` + bare floats)? Hay bắt buộc dùng `kuberina-forge` để chuyển đổi?
   - **Đề xuất:** Bỏ backward compat trong solver. Forge lo chuyển đổi. Solver chỉ đọc IR v0.2.0.

4. **`rack` field trên Node:** v0.1.x không có. v0.2.0 thêm `rack`. Cần cho `topologySpread` với `topologyKey: "rack"`. Hợp lý.

5. **Default values cho dimensions mới:** Nếu một pod không khai báo `disk` hoặc `network`, default = 0 (giống `gpu` hiện tại). Nếu một node không khai báo, default = `+∞`? Hay bắt buộc khai báo?
   - **Đề xuất:** Pod default = 0 (không dùng resource đó). Node default = `f64::MAX` (không giới hạn — unconstrained dimension). Điều này cho phép homelab users bỏ qua `disk`/`network` nếu không quan tâm.

---

## 5. Phân tích tác động đến PAPER.md

Expansion từ 3D → 8D + IR format mới + compiler pipeline architecture **sẽ trigger rewrite** một số phần trong `PAPER.md`. Dưới đây là phân tích từng section:

### 5.1. Sections PHẢI chỉnh lý

| Section | Nội dung cần thay đổi | Mức độ |
|:--------|:----------------------|:-------|
| **Abstract** | Cập nhật "3 dimensions (CPU, RAM, GPU)" → "8 dimensions". Cập nhật benchmark results nếu re-run. | 🔴 Major |
| **§3.1.5 Constraint Mapping Table** | Row "Container Dimensions" hiện ghi `(CPU, RAM, GPU, VRAM)`. Cần mở rộng thành `(CPU, RAM, GPU, Storage, Disk R/W, Network I/O)`. | 🟡 Medium |
| **§3.2 Formal Definition — Notation** | $\mathcal{R} = \{\text{CPU}, \text{RAM}, \text{GPU}, \ldots\}$ — cần explicit hóa 8 chiều. | 🟡 Medium |
| **§3.2 — Phase 0 DaemonSet formula** | Formula $C_j^r$ vẫn đúng (generic trên $r$), nhưng prose cần nhắc 8 dimensions. | 🟢 Minor |
| **§4.1 System Overview** | Hiện mô tả CLI offline engine. Cần bổ sung: (1) compiler pipeline architecture, (2) `kuberina-forge` vs `kuberina-solver` separation, (3) IR concept. | 🔴 Major |
| **§4.2 Phase 1 FFD — Synthetic Volume** | Formula $V_i = \alpha \cdot \text{CPU}_i + \beta \cdot \text{RAM}_i + \gamma \cdot \text{GPU}_i$. Cần mở rộng: $V_i = \alpha \cdot \text{CPU}_i + \beta \cdot \text{RAM}_i + \gamma \cdot \text{GPU}_i + \delta \cdot \text{Storage}_i + \epsilon_r \cdot \text{DiskR}_i + \epsilon_w \cdot \text{DiskW}_i + \zeta_{in} \cdot \text{NetIn}_i + \zeta_{out} \cdot \text{NetOut}_i$ | 🔴 Major — công thức thay đổi |
| **§4.3 Phase 2 GA** | Prose nhắc "three dimensions" ở vài chỗ. Cần generalize. | 🟡 Medium |
| **§4.4 Phase 3 CSP** | Capacity checking prose cần nhắc 8D. Formula vẫn generic. | 🟢 Minor |
| **§6.1 Testbed Description** | Bảng metrics hiện chỉ có CPU/RAM/GPU. Cần bổ sung Storage, Disk, Network columns. | 🔴 Major — bảng benchmark |
| **§6.4 Metrics** | Fragmentation formula cần explicit 8D. | 🟡 Medium |
| **§7 Results** | **Toàn bộ bảng kết quả phải re-run** với 8D benchmark. Nếu chưa re-run, phải ghi rõ "results are from v0.1.x 3D benchmark". | 🔴 Major |
| **§7.3 Mathematical Verification** | LP bounds cần tính trên 8 dimensions thay vì 3. | 🔴 Major |

### 5.2. Sections KHÔNG cần chỉnh lý

| Section | Lý do giữ nguyên |
|:--------|:-----------------|
| **§1 Introduction** | Vấn đề (fragmentation, dynamic vs offline) không thay đổi. |
| **§2 Related Work** | Survey các hệ thống khác — independent. |
| **§3.1.1–3.1.4** | Maritime analogies (mega-vessel scale, stability, cargo classification, hierarchical decomposition) — independent. |
| **§3.2 — Hard/Soft Constraints** | Formulas đã generic trên $r \in \mathcal{R}$, chỉ cần mở rộng $\mathcal{R}$. |
| **§4.5 Gang Scheduling** | Block Booking model không thay đổi. |
| **§5 Security & Auditability** | Independent. |
| **§8 Discussion** | Có thể cần minor updates nhưng không structural. |
| **§9 Conclusion** | Sẽ cần cập nhật số liệu nếu re-run benchmark. |

### 5.3. Sections CẦN BỔ SUNG MỚI

| Section mới | Nội dung |
|:------------|:---------|
| **§4.1.1 Kuberina IR** | Mô tả formal IR specification, giải thích tại sao cần intermediate representation (compiler analogy). |
| **§4.1.2 Compiler Pipeline** | Mô tả `forge` → IR → `solver` → blueprint → `forge` pipeline. Non-K8s extensibility. |
| **§4.4.x TopologySpread** | Formalize topologySpread constraint as soft/hard penalty. MaxSkew formula. |
| **§8.x Noisy Neighbor** | Discussion section mới: how 8D expansion addresses Noisy Neighbor problem specifically (Disk I/O contention, network bandwidth saturation). |

### 5.4. Công thức toán cần thay đổi

1. **Synthetic Volume (§4.2):**
   - Hiện tại: $V_i = \alpha \cdot \text{CPU}_i + \beta \cdot \text{RAM}_i + \gamma \cdot \text{GPU}_i$
   - Mới: $V_i = \sum_{r \in \mathcal{R}} w_r \cdot \text{req}_i^r$ (generalized, 8 weights)

2. **Fragmentation (§3.2, §6.4):**
   - Hiện tại (implicit 3D): $f_{\text{frag}} = \sum_{j: y_j=1} \sum_{r \in \{C,R,G\}} \max(0, C_j^r - \sum_i x_{ij} \cdot \text{req}_i^r)$
   - Mới: $r \in \mathcal{R}$ với $|\mathcal{R}| = 8$

3. **LP Lower Bounds (§7.3):**
   - Cần tính $L^r$ cho 8 dimensions thay vì 3. $L = \max_r L^r$ vẫn đúng.

4. **TopologySpread penalty (MỚI):**
   - $f_{\text{spread}}(s) = \sum_{t \in \mathcal{T}} \max(0, \text{skew}_t(s) - \text{maxSkew}_t)$
   - Trong đó $\text{skew}_t = \max_z \text{count}(z) - \min_z \text{count}(z)$ trên topology key $t$.

5. **Objective function (§3.2):**
   - Thêm term: $F(s) = w_1 f_{\text{nodes}} + w_2 f_{\text{frag}} + w_3 f_{\text{affinity}} + w_4 f_{\text{var}} + w_5 f_{\text{spread}} + \Phi(s)$

6. **FfdWeights struct:**
   - Hiện tại: $\alpha, \beta, \gamma$ (3 weights)
   - Mới: $w_r$ for each $r \in \mathcal{R}$ (8 weights). Tên: `alpha` (CPU), `beta` (RAM), `gamma` (GPU), `delta` (Storage), `epsilon_r` (Disk Read), `epsilon_w` (Disk Write), `zeta_in` (Net In), `zeta_out` (Net Out).

---

## 6. Open Questions

> [!WARNING]
> **TopologySpread: Hard hay Soft Constraint?**
>
> `topologySpread` trong K8s có `whenUnsatisfiable: DoNotSchedule | ScheduleAnyway`.
> Đề xuất cho v0.2.0: implement as **Soft Penalty** ($w_5 \cdot f_{\text{spread}}$) — prevent GA plateau.
> Nếu user cần hard, thêm flag `topologySpread.strict: true` → route vào $\Phi(s)$.

> [!IMPORTANT]
> **Backward Compatibility Policy**
>
> Solver v0.2.0 sẽ chỉ đọc IR v0.2.0 format. `kuberina-forge` chịu trách nhiệm chuyển đổi
> từ mọi format nguồn (K8s manifests, Helm, v0.1.x testdata,...) sang IR v0.2.0.
> Điều này có nghĩa testdata hiện tại (`homelab_*.yaml`) sẽ phải cập nhật sang format mới.

> [!NOTE]
> **Thứ tự triển khai**
>
> 1. Mở rộng Rust solver (8D + IR parsing) — ưu tiên cao nhất
> 2. Cập nhật testdata sang IR v0.2.0 format
> 3. Cập nhật PAPER.md formulas + benchmarks
> 4. `kuberina-forge` (Go binary) — ưu tiên thấp hơn, có thể v0.3.0