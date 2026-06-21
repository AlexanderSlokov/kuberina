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
  1. Resource model: dùng resource.Quantity của K8s (milliCPU, bytes) hay đơn vị đơn giản hơn?
  2. Pod group (Gang): model thế nào? Một wrapper struct chứa []Pod + constraint "all-or-nothing"?
  3. DaemonSet pods: pre-process (trừ hao capacity trước) hay model như Pod bình thường với constraint đặc biệt?
-->

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
