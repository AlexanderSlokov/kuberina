# Kuberina — Software Design Document

## 1. Language & Technology Stack

- POC code for formulas and algorithms in small simulations: Python
- Main engine and CLI tool implementations: Golang
- All 2 language will be chosen as latest version available. 

<!-- [Q-Claude] Câu hỏi về tech stack:
  1. Go version target? Go 1.21+ (có slices, slog) hay cần backward compatible?
  2. Python POC dùng library gì? NumPy, SciPy, DEAP (GA library), OR-Tools?
  3. Output YAML: dùng library nào để serialize? `sigs.k8s.io/yaml` hay `gopkg.in/yaml.v3`?
  4. Có dùng K8s client-go để đọc cluster state (kubectl get nodes) không, hay chỉ đọc static YAML file?
-->

## 2. System Architecture

<!-- [Q-Claude] Cần một high-level architecture diagram ở đây. Gợi ý cấu trúc:

  ┌──────────────┐     ┌──────────────┐
  │ cluster-     │     │ workload     │
  │ topology.yaml│     │ manifests/   │
  └──────┬───────┘     └──────┬───────┘
         │                    │
         ▼                    ▼
  ┌──────────────────────────────────┐
  │         Input Parser             │
  └──────────────┬───────────────────┘
                 │
                 ▼
  ┌──────────────────────────────────┐
  │   FFD Initializer (Phase 1)      │
  └──────────────┬───────────────────┘
                 │
                 ▼
  ┌──────────────────────────────────┐
  │   GA Optimizer (Phase 2)         │
  │   ┌────────────────────────┐     │
  │   │ CSP Solver (Phase 3)   │     │
  │   └────────────────────────┘     │
  └──────────────┬───────────────────┘
                 │
                 ▼
  ┌──────────────────────────────────┐
  │      Output Generator            │
  │  (YAML + injected constraints)   │
  └──────────────────────────────────┘

  Câu hỏi:
  1. CSP Solver chạy bên trong GA loop hay là module tách riêng?
  2. Input Parser có validate schema không? Dùng CRD-style validation hay custom?
  3. Có cần một "Diff Engine" để so sánh blueprint mới vs blueprint cũ (cho incremental re-plan)?
-->

## 3. Project Structure

<!-- [Q-Claude] Gợi ý cấu trúc thư mục Go project:

  kuberina/
  ├── cmd/
  │   └── kuberina/          # CLI entrypoint (cobra)
  │       └── main.go
  ├── internal/
  │   ├── parser/            # Input YAML parser
  │   ├── model/             # Data structures (Node, Pod, Blueprint)
  │   ├── ffd/               # First-Fit Decreasing algorithm
  │   ├── ga/                # Genetic Algorithm engine
  │   ├── csp/               # CSP Solver + Forward Checking
  │   ├── fitness/           # Fitness function calculations
  │   └── output/            # YAML output generator
  ├── research/              # Python POC scripts
  ├── testdata/              # Sample cluster topologies & workloads
  ├── PAPER.md
  ├── DESIGN.md
  └── README.md

  Câu hỏi:
  1. Bạn thích cấu trúc này không, hay có layout khác trong đầu?
  2. CLI framework: Cobra
  3. Có cần plugin system không (ví dụ: cho phép user define custom fitness function)?
-->
```markdown
  kuberina/
  ├── cmd/
  │   └── kuberina/          # CLI entrypoint (cobra)
  │       └── main.go
  ├── internal/
  │   ├── parser/            # Input YAML parser
  │   ├── model/             # Data structures (Node, Pod, Blueprint)
  │   ├── ffd/               # First-Fit Decreasing algorithm
  │   ├── ga/                # Genetic Algorithm engine
  │   ├── csp/               # CSP Solver + Forward Checking
  │   ├── fitness/           # Fitness function calculations
  │   └── output/            # YAML output generator
  ├── research/              # Python POC scripts
  ├── testdata/              # Sample cluster topologies & workloads
  ├── PAPER.md
  ├── DESIGN.md
  └── README.md
```
- CLI framework: Cobra

## 4. Data Model

<!-- [Q-Claude] Cần định nghĩa rõ các struct chính:

  1. Node struct: Name, Labels, Taints, Allocatable (CPU, RAM, GPU), Zone/Rack topology
  2. Pod struct: Name, Namespace, Resource Requests, Resource Limits, Affinities, Tolerations, QoS class, Priority
  3. Blueprint struct: Map[Pod]Node assignment, Fitness score, Generation number
  4. ClusterTopology struct: []Node + metadata

  Câu hỏi:
  3. DaemonSet pods: pre-process (trừ hao capacity trước) hay model như Pod bình thường với constraint đặc biệt?
-->

1. `Node` struct: Name, Labels, Taints, Allocatable (CPU, RAM, GPU), Zone/Rack topology
2. `Pod` struct: Name, Namespace, Resource Requests, Resource Limits, Affinities, Tolerations, QoS class, Priority
3. `Blueprint` struct: Map[Pod]Node assignment, Fitness score, Generation number
4. `ClusterTopology` struct: []Node + metadata

### Resource model: 

Dùng đơn vị chuẩn K8s `resource.Quantity` — CPU ghi `500m` (milliCPU) hoặc `4` (cores), RAM/Disk ghi `8Gi` / `512Mi`, GPU ghi `nvidia.com/gpu: 1`. 

Internally parse bằng `k8s.io/apimachinery/pkg/api/resource.Quantity`. 

Cả input YAML và output YAML đều dùng notation này — người đọc blueprint thấy `memory: 8Gi` thay vì `memory: 8589934592`.

### Pod group (Gang)

Gang Scheduling trong K8s: một nhóm Pod **bắt buộc phải được schedule cùng lúc**, hoặc không Pod nào được schedule hết (all-or-nothing).

Use case kinh điển: Distributed AI Training — cần 8 Pod, mỗi Pod chiếm 1 GPU, tất cả phải chạy đồng thời trên các Node có `NVLink/InfiniBand` để communicate trong quá trình training. Nếu chỉ xếp được 7/8 Pod → Pod thứ 8 pending → cả 7 Pod kia ngồi chờ vô ích → resource deadlock.

#### Analogy hàng hải: Block Booking, không phải OOG

> **Tại sao không dùng OOG (Out of Gauge)?** OOG là 1 kiện hàng siêu trọng (turbine gió, máy biến thế) — một vật thể vật lý duy nhất chiếm nhiều slots liền kề. Nếu map gang scheduling sang OOG, ta sẽ gom 8 Pod thành 1 macro-block không chia được → **sai**, vì mỗi Pod trong gang vẫn là thực thể riêng biệt:
> - Mỗi Pod tiêu hao resource **riêng** trên Node nó đậu (CPU, RAM, GPU)
> - Mỗi Pod vẫn gây noisy neighbor với các Pod khác trên cùng Node
> - Mỗi Pod có thể có resource request khác nhau (worker 80GB VRAM vs parameter server 16GB VRAM)
> - Mỗi Pod vẫn phải thỏa mãn affinity/anti-affinity cá nhân
>
> **Analogy đúng: Block Booking / Slot Charter.** Một shipper đặt trước 20 container trên chuyến tàu. Hãng tàu phải xếp được **tất cả 20**, hoặc **từ chối cả booking**. Nhưng mỗi container trong lô vẫn có trọng lượng riêng, vẫn có thể là reefer/hazmat, vẫn ảnh hưởng trọng tâm bay nó đậu. Constraint "all-or-nothing" nằm ở **tầng thương mại** (booking), không ở tầng vật lý (kích thước).

#### Hệ quả cho Chromosome Encoding: Pod-level, không phải Group-level

Vì gang pods **không phải monolithic block**, mỗi pod vẫn cần là một biến quyết định riêng biệt trong chromosome. Trong CSP, đây gọi là **coupled variables** — nhiều biến có ràng buộc liên kết, nhưng mỗi biến vẫn có domain riêng.

```go
// PodGroup represents a gang-scheduled unit — all pods must be placed, or none.
// Maritime analogy: Block Booking — a batch of individual containers bound by
// a commercial "all-or-nothing" constraint, NOT a single monolithic OOG cargo.
// Each pod still individually affects the node it lands on.
type PodGroup struct {
    Name        string
    Pods        []int          // indices into the global Pod slice
    MinMembers  int            // all-or-nothing: MinMembers == len(Pods)

    // Constraint: which nodes are eligible for this group?
    // e.g., all pods need GPU nodes with NVLink topology
    NodeSelector map[string]string

    // Scheduling mode
    Colocate     bool          // true = prefer same node/rack (NVLink locality)
}
```

Chromosome encoding giữ nguyên pod-level — mỗi gang pod là 1 phần tử riêng trong `Assignment[]`:

```go
type Blueprint struct {
    Assignment []int           // len = ALL pods (gang + non-gang), value = node index
    Fitness    float64
    NodeLoad   []ResourceVector // cached per-node resource usage for fast fitness eval
}
// Gang constraint enforce ở tầng trên (CSP + fitness), không ở tầng encoding.
```

#### GA Operators: Gang-aware Mutation & Crossover

**Mutation** — move từng pod riêng, rollback nếu phá gang:

```go
func mutate(b *Blueprint, groups []PodGroup, rate float64) {
    for i, nodeID := range b.Assignment {
        if rand.Float64() > rate { continue }

        if g := findGroup(i, groups); g != nil {
            // Pod thuộc gang → thử move, kiểm tra gang feasibility
            newNode := randomEligibleNode(i)
            b.Assignment[i] = newNode
            if !gangStillFeasible(b, g) {
                b.Assignment[i] = nodeID  // rollback
            }
        } else {
            // Pod thường → mutation bình thường
            b.Assignment[i] = randomEligibleNode(i)
        }
    }
}
```

**Crossover** — uniform crossover + gang repair:

```go
func crossover(p1, p2 *Blueprint, groups []PodGroup) *Blueprint {
    child := uniformCrossover(p1, p2)

    // Crossover có thể cắt ngang 1 gang: lấy pod 1-4 từ p1, pod 5-8 từ p2
    // → kết hợp có thể vỡ capacity trên 1 node nào đó.
    // Repair: fallback lấy toàn bộ gang assignment từ parent tốt hơn.
    for _, g := range groups {
        if !gangStillFeasible(child, &g) {
            source := betterParent(p1, p2)
            for _, podIdx := range g.Pods {
                child.Assignment[podIdx] = source.Assignment[podIdx]
            }
        }
    }
    return child
}
```

**Fitness** — gang vi phạm = hard constraint, giết solution:

```go
func gangPenalty(b *Blueprint, groups []PodGroup) float64 {
    for _, g := range groups {
        placed := 0
        for _, podIdx := range g.Pods {
            if nodeHasCapacityFor(b.Assignment[podIdx], podIdx, b.NodeLoad) {
                placed++
            }
        }
        if placed < g.MinMembers {
            return -math.MaxFloat64  // kill this solution
        }
    }
    return 0
}
```

#### CSP Forward Checking for Pod Group

Trước khi GA mutation di chuyển bất kỳ pod nào trong gang, check cả gang trước — đúng tinh thần hàng hải: *"trước khi nhận booking 20 container, kiểm tra tàu còn đủ slots cho tất cả 20 cái không."*

```go
func canPlaceGang(group PodGroup, candidateNodes []int, nodeCapacity []ResourceVector) bool {
    // 1. Check: đủ nodes eligible cho tất cả pods trong gang?
    eligible := filterBySelector(candidateNodes, group.NodeSelector)
    if len(eligible) < len(group.Pods) {
        return false  // prune immediately — không đủ nodes
    }

    // 2. Check: tổng remaining capacity >= tổng resource demand?
    totalDemand := sumResources(group.Pods)
    totalAvail := sumRemainingCapacity(eligible)
    if !totalAvail.FitsAll(totalDemand) {
        return false  // prune immediately — không đủ tổng resource
    }

    // 3. If colocate=true: có node/rack nào chứa được cả gang không?
    if group.Colocate {
        for _, n := range eligible {
            if nodeCapacity[n].FitsAll(totalDemand) {
                return true
            }
        }
        return false  // không rack nào đủ chỗ cho cả nhóm
    }

    return true
}
```


### DaemonSet pods

DaemonSets **pre-process: trừ hao capacity trước khi GA/FFD chạy.**

#### Analogy hàng hải: Hệ thống nội tại của tàu

DaemonSet là **hệ thống vận hành của chính con tàu** — tồn tại trước khi bất kỳ container nào được bốc lên:

| Hệ thống tàu | DaemonSet tương ứng |
|---|---|
| Hệ thống bơm nước dằn (Ballast) — chống lật | `kube-proxy` — networking cơ bản trên mọi node |
| Hệ thống quan trắc (sensors, gauges) | `node-exporter`, `datadog-agent` — monitoring |
| Hệ thống liên lạc (radio, AIS) | `calico-node`, `cilium` — CNI networking |
| Hệ thống làm mát turbine | `csi-node-driver` — storage driver |
| Hệ thống cảnh báo va chạm | `falco`, `kube-audit` — security monitoring |

Không ai "xếp" những hệ thống này vào tàu — chúng **là** tàu. Stowage planner nhìn vào tàu và thấy: "tàu này capacity 24,000 TEU, nhưng trừ hệ thống nội tại (bơm, ống dẫn, lối đi) → capacity thực cho hàng hóa là 23,200 TEU." Rồi mới bắt đầu planning.

#### Cách xử lý trong Kuberina

DaemonSets **không nằm trong search space** — chúng là **fixed variables**, không phải decision variables:

```go
// Phase 0: Pre-deduct DaemonSet resource consumption from node capacity.
// This runs BEFORE FFD and GA — the optimizer never sees DaemonSet pods.
func preDeductDaemonSets(nodes []Node, daemonSets []DaemonSet) {
    for i := range nodes {
        for _, ds := range daemonSets {
            if ds.ShouldRunOn(&nodes[i]) {  // check nodeSelector, tolerations
                nodes[i].Allocatable.CPU -= ds.Resources.CPU
                nodes[i].Allocatable.RAM -= ds.Resources.RAM
                // GPU DaemonSets rất hiếm, nhưng handle cho đúng
                nodes[i].Allocatable.GPU -= ds.Resources.GPU
            }
        }
    }
    // Sau bước này, nodes[i].Allocatable = capacity THỰC cho workload pods.
    // FFD và GA chỉ thấy capacity đã trừ hao.
}
```

```go
// DaemonSet represents a system-level workload that runs on every eligible node.
// Maritime analogy: Ship's own systems (ballast pumps, comms, sensors).
// NOT cargo — pre-deducted from capacity, not part of the optimization problem.
type DaemonSet struct {
    Name         string
    Resources    ResourceVector         // resource consumed per node
    NodeSelector map[string]string      // which nodes this DS runs on
    Tolerations  []Toleration           // DS thường tolerate mọi taint
}
```

#### Tại sao không model DaemonSet như Pod bình thường?

Nếu cho DaemonSet pods vào GA search space:
- **Lãng phí compute**: GA sẽ thử "move" kube-proxy từ Node 1 sang Node 3 → vô nghĩa, kube-proxy phải chạy trên MỌI node
- **Search space phình to vô ích**: 10 loại DaemonSet × 100 nodes = 1000 pods thêm vào chromosome → tăng từ 500 lên 1500 biến, mà kết quả bắt buộc chỉ có 1 cách duy nhất
- **Violates reality**: DaemonSet không phải "quyết định scheduling" — chúng là tiền đề (precondition)

## 5. Input Schema

<!-- [Q-Claude] Cần define rõ format của 2 input files:

  ### 5.1. cluster-topology.yaml
  - Schema chính xác? Tự define hay reuse K8s Node spec?
  - Ví dụ: có cần field custom nào ngoài K8s standard (e.g., rack_id, power_zone)?

  ### 5.2. Workload Manifests
  - Chấp nhận format gì? Raw K8s Deployment YAML? Helm values? Kustomize?
  - Có cần đọc từ live cluster (kubectl get deployments) không?
  - Làm sao biết Pod nào là Gang (AI training batch)?

  Câu hỏi quan trọng: User cung cấp resource requests, resource limits, hay cả hai? Kuberina dùng requests hay limits để tính toán bin packing?
-->

## 6. Output Schema

<!-- [Q-Claude]
  Output gồm những gì?
  1. Modified YAML files với injected NodeSelector, Affinity, Toleration?
  2. Summary report (text/JSON) cho biết: bao nhiêu Node dùng, utilization %, nào Pod đi đâu?
  3. Diff file cho biết Kuberina đã thay đổi gì so với input?
  4. Visualization (optional): ASCII art / HTML chart cho blueprint?

  Câu hỏi: Output có idempotent không? Chạy kuberina plan 2 lần liên tiếp trên cùng input, kết quả có giống nhau không? (GA có random seed)
-->

## 7. Algorithm Configuration & Tuning

<!-- [Q-Claude] Cần document tất cả hyperparameters và default values:

  ### FFD Parameters
  - alpha, beta, gamma (resource weights cho synthetic volume)
  - Node sorting order

  ### GA Parameters  
  - Population size (128/256/512/1024 — decision logic?)
  - Max generations
  - Mutation rate
  - Crossover rate
  - Elitism percentage
  - Early stopping threshold
  - Random seed (for reproducibility)

  ### Fitness Weights
  - w_node_count
  - w_fragmentation
  - w_affinity_violation
  - w_utilization_variance
  - Penalty cho hard constraint violation

  Câu hỏi: User có thể override các giá trị này qua CLI flags hoặc config file không? Hay chỉ có developer mới tune được?
-->

### FFD Parameters
- alpha, beta, gamma (resource weights cho synthetic volume)
- Node sorting order

### GA Parameters  
- Population size (128/256/512/1024 — decision logic?)
- Max generations
- Mutation rate
- Crossover rate
- Elitism percentage
- Early stopping threshold
- Random seed (for reproducibility)

### Fitness Weights
- w_node_count
- w_fragmentation
- w_affinity_violation
- w_utilization_variance
- Penalty cho hard constraint violation

## 8. CLI Interface Design

<!-- [Q-Claude] Gợi ý CLI commands:

  kuberina plan     --infra <file> --workloads <dir> --output <dir>   # Main command
  kuberina validate --infra <file> --workloads <dir>                  # Chỉ validate input, không solve
  kuberina diff     --old <dir> --new <dir>                           # So sánh 2 blueprints
  kuberina report   --blueprint <dir>                                 # Generate report từ blueprint đã tạo
  kuberina version                                                    # Version info

  Câu hỏi:
  1. Có cần --dry-run flag không?
  2. Output format: chỉ YAML hay cần JSON, TOML?
  3. Verbosity levels: --verbose, --debug, --quiet?
  4. Progress bar cho GA computation? (Có thể mất vài phút cho cluster lớn)
-->

## 9. Testing Strategy

<!-- [Q-Claude]
  1. Unit tests: Từng module (FFD, GA, CSP) test riêng
  2. Integration tests: Full pipeline input -> output
  3. Benchmark tests: Measure thời gian chạy với dataset sizes khác nhau
  4. Golden tests: So sánh output với expected output đã biết
  5. Fuzz tests: Random input generation để tìm edge cases

  Câu hỏi:
  1. Testdata lấy từ đâu? Synthetic generation script hay copy từ real cluster?
  2. CI/CD: GitHub Actions? Chạy test khi nào?
  3. GA test: vì GA non-deterministic, test assertion thế nào? (seed-based reproducibility?)
-->

## 10. Performance & Scalability Targets

<!-- [Q-Claude]
  Cần define rõ:
  1. Max cluster size support: 100 Nodes? 1000? 10000?
  2. Max workload count: 500 Pods? 5000? 50000?
  3. Time budget: "vài phút" cụ thể là max bao nhiêu phút cho cluster size nào?
  4. Memory budget: GA population 1024 x Blueprint size — estimate RAM usage?
  5. Có cần streaming output (xuất partial results trong khi GA vẫn chạy)?
-->

## 11. Future Considerations

<!-- [Q-Claude]
  1. Web UI dashboard: visualize blueprint trước khi apply?
  2. Kubernetes Operator: tự động re-plan khi cluster state thay đổi?
  3. Multi-cluster support: schedule across multiple clusters?
  4. Cost-aware optimization: integrate cloud pricing (spot vs on-demand)?
  5. Historical learning: dùng past scheduling data để warm-start GA?
-->
