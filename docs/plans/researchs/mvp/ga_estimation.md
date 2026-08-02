# GA Computational Estimation for Kuberina

*Estimated by Claude — June 22, 2026*

---

## 1. Reference: Typical AI Compute Production K8s Cluster (Medium)

Một cluster AI compute production "bình thường nhất" trông thế này:

| Loại Node | Số lượng | Specs/Node | Tổng |
|---|---|---|---|
| GPU Nodes (A100/H100) | 30 | 8 GPU, 96 cores, 768GB RAM | 240 GPU |
| CPU Compute | 50 | 32 cores, 128GB RAM | 1600 cores |
| Memory-Optimized (DB/Cache) | 15 | 16 cores, 512GB RAM | 7.5TB RAM |
| Infra/Management | 5 | 8 cores, 32GB RAM | — |
| **Total Nodes** | **100** | | |

Workload profile (sau khi trừ DaemonSets — chúng là "Ballast Water", pre-deducted):

| Loại Workload | Số Pods | Đặc điểm |
|---|---|---|
| AI Training Jobs (gang-scheduled) | 80 | GPU-bound, 4-8 GPU/pod, all-or-nothing |
| AI Inference Services | 60 | GPU-bound, 1-2 GPU/pod |
| Web/API Services | 150 | CPU-bound, 0.5-4 cores |
| Databases, Caches | 80 | Memory-bound, high-memory requests |
| Monitoring, Logging | 30 | Lightweight |
| **Total Pods (to schedule)** | **~400-500** | |

> [!NOTE]
> DaemonSets (~10 loại x 100 nodes = ~1000 pods) được trừ hao khỏi capacity trước khi GA chạy. Chúng là "Fixed Variables trong ILP" — không nằm trong search space.

---

## 2. Chromosome Encoding & Search Space

### Encoding
Mỗi individual (Blueprint) = 1 array int, trong đó `chromosome[i] = node_id` mà Pod i được assign vào.

```
// Go representation
type Blueprint struct {
    Assignment []int    // len = N (số pods), value = node index [0, M)
    Fitness    float64
    // cached per-node resource usage for fast fitness eval
    NodeLoad   []ResourceVector  // len = M
}
```

### Kích thước

| Metric | Medium Cluster (N=500, M=100) |
|---|---|
| Chromosome array | 500 × 8 bytes (int64) = **4 KB** |
| NodeLoad cache | 100 × 4 dims × 8 bytes = **3.2 KB** |
| Metadata (fitness, flags) | ~1 KB |
| **Total per individual** | **~8-10 KB** |

### Search Space

$$|S| = M^N = 100^{500} = 10^{1000}$$

Con số vô nghĩa lớn — lớn hơn số nguyên tử trong vũ trụ ($10^{80}$) khoảng $10^{920}$ lần. Đây là lý do brute-force không khả thi và GA + FFD warm-start là cần thiết.

---

## 3. Population Size & Memory

### Sizing logic

| Cluster Size | Pods (N) | Nodes (M) | Population | Lý do |
|---|---|---|---|---|
| Small | 50 | 10 | 128 | Search space nhỏ, converge nhanh |
| **Medium (ref)** | **500** | **100** | **512** | **Cân bằng diversity vs compute** |
| Large | 2000 | 500 | 1024 | Cần diversity cao hơn cho space lớn |
| XL | 5000 | 1000 | 1024 | Cap ở 1024, tăng generations thay thế |

> [!IMPORTANT]
> Population 1024 là cap hợp lý. Với N > 2000, tăng population tiếp cho diminishing returns — tốt hơn là tăng số generations hoặc dùng Island Model GA (chia population thành sub-populations chạy song song, merge định kỳ).

### RAM cho Population (double-buffered: parent + offspring)

| Tier | Per Individual | Population | Double-buffered | 
|---|---|---|---|
| Small | ~1 KB | 128 KB | 256 KB |
| **Medium** | **~10 KB** | **5 MB** | **10 MB** |
| Large | ~25 KB | 25 MB | 50 MB |
| XL | ~60 KB | 60 MB | 120 MB |

---

## 4. Fitness Evaluation Cost (Per Individual)

Fitness function cần tính:

| Operation | Complexity | Medium (N=500, M=100) |
|---|---|---|
| Resource sum per node | O(N) | 500 ops |
| Capacity violation check | O(M × D) | 400 ops |
| Affinity/Anti-affinity check | O(N × k), k=avg constraints ≈ 5 | 2,500 ops |
| Utilization variance | O(M) | 100 ops |
| Gang scheduling validation | O(G), G=gang pods | ~80 ops |
| **Total per individual** | | **~3,500 ops** |

### Per Generation

| Step | Complexity | Medium Cluster |
|---|---|---|
| Selection (tournament, size=3) | O(Pop × 3) | 1,536 comparisons |
| Crossover + Repair | O(Pop/2 × N) | 128,000 ops |
| Mutation + CSP check | O(Pop × rate × M), rate=5% | 2,560 ops |
| Fitness eval (all individuals) | O(Pop × 3,500) | 1,792,000 ops |
| **Total per generation** | | **~2M ops** |

---

## 5. Convergence Estimation

### Tại sao FFD warm-start giúp rất nhiều

Không có FFD (random init):
- Population bắt đầu từ random assignments → phần lớn vi phạm capacity constraints
- Cần ~500-1000 generations chỉ để "sửa" violations
- Total: 3000-8000 generations

**Có FFD warm-start:**
- Population bắt đầu từ **feasible solutions** (FFD đã pack hợp lệ)
- GA chỉ cần optimize soft constraints (affinity, balance, fragmentation)
- Convergence nhanh hơn 3-5x

### Estimated Generations to Near-Optimal

| Tier | Không FFD | Có FFD | Early Stop (no improvement for N gens) |
|---|---|---|---|
| Small | 500-1500 | **200-500** | 50 gens |
| **Medium** | 2000-5000 | **500-2000** | 100 gens |
| Large | 5000-15000 | **1000-5000** | 150 gens |
| XL | 10000-30000 | **2000-10000** | 200 gens |

> [!TIP]
> Early stopping là chìa khóa. Nếu best fitness không cải thiện sau 100-200 generations liên tiếp, coi như đã converge. Trong thực tế, phần lớn improvement xảy ra trong 30% generations đầu tiên.

---

## 6. Wall-Clock Time với Go (Goroutines)

### Assumptions
- Machine: 8-core CPU (typical dev/CI machine)
- Go throughput: ~50-100M ops/sec per core (fitness eval có memory access, cache miss)
- Fitness evaluation: **embarrassingly parallel** — mỗi individual evaluate độc lập → chia đều cho goroutines
- Crossover/Mutation: có thể parallel nhưng overhead sync cao → chạy sequential hoặc batch

### Time per Generation

| Tier | Ops/Gen | 1 Core | 8 Cores (goroutines) |
|---|---|---|---|
| Small | ~200K | 2-4 ms | **< 1 ms** |
| **Medium** | **~2M** | 20-40 ms | **3-5 ms** |
| Large | ~15M | 150-300 ms | **20-40 ms** |
| XL | ~50M | 500ms-1s | **70-130 ms** |

### Total Time to Convergence (with FFD + Early Stop)

| Tier | Pods | Nodes | Generations | Wall Time (8 cores) |
|---|---|---|---|---|
| Small | 50 | 10 | 200-500 | **< 0.5 sec** |
| **Medium** | **500** | **100** | **500-2000** | **2-10 sec** |
| Large | 2000 | 500 | 1000-5000 | **20 sec - 3.5 min** |
| XL | 5000 | 1000 | 2000-10000 | **2.5 - 22 min** |

> [!NOTE]
> Với medium cluster (500 pods, 100 nodes) — use case phổ biến nhất — Kuberina chạy xong **trong dưới 10 giây**. Nhanh hơn `helm install` nhiều.

---

## 7. Tổng Resource Kuberina Tiêu Thụ

### RAM Breakdown (Medium Cluster, Peak)

| Component | RAM |
|---|---|
| Go runtime (GC, scheduler, stacks) | 20-30 MB |
| Pod data structures (500 pods × ~200B) | ~0.1 MB |
| Node data structures (100 nodes × ~500B) | ~0.05 MB |
| Population (512 individuals, double-buffered) | 10 MB |
| Fitness eval scratch space | 5 MB |
| Goroutine stacks (8 × 8KB initial, grow to ~64KB) | ~0.5 MB |
| Input/Output YAML buffers | 5-10 MB |
| **Total Peak RAM** | **~50-60 MB** |

### Full Table

| Tier | Peak RAM | CPU Cores (saturated) | Total Time | 
|---|---|---|---|
| Small | **~30 MB** | 4-8 | < 0.5 sec |
| **Medium** | **~50-60 MB** | 8 | **2-10 sec** |
| Large | **~150-250 MB** | 8-16 | 20s - 3.5 min |
| XL | **~300-500 MB** | 8-16 | 2.5 - 22 min |

### CPU Profile (Medium Cluster)

```
Estimated CPU time breakdown:
┌─────────────────────────────────┬───────────┐
│ Phase                           │ % of Time │
├─────────────────────────────────┼───────────┤
│ Input parsing (YAML)            │    2-5%   │
│ FFD Initialization              │    1-3%   │
│ GA: Fitness Evaluation          │   60-70%  │
│ GA: Crossover + Repair          │   15-20%  │
│ GA: Mutation + CSP Check        │    5-8%   │
│ GA: Selection                   │    1-2%   │
│ Output generation (YAML write)  │    3-5%   │
└─────────────────────────────────┴───────────┘
```

---

## 8. So sánh Context: Kuberina vs các process khác trên cùng machine

| Process | RAM | CPU | Time |
|---|---|---|---|
| `kubectl apply -f` (500 resources) | ~50 MB | 1 core | 10-30 sec |
| `helm install` (large chart) | ~100 MB | 1 core | 15-60 sec |
| `terraform plan` (100 resources) | ~200 MB | 1 core | 30-120 sec |
| **Kuberina plan (500 pods)** | **~50-60 MB** | **8 cores** | **2-10 sec** |
| `docker build` (medium image) | ~500 MB | multi-core | 30-300 sec |

> [!TIP]
> Kuberina nhẹ hơn `terraform plan` và nhanh hơn `helm install`. Đây là selling point cho README: "Runs in seconds, lighter than your build pipeline."

---

## 9. Caveats & Assumptions

1. **Fitness function complexity**: Ước lượng O(N×k) cho affinity check giả định k nhỏ (~5 constraints/pod). Nếu cluster có rất nhiều inter-pod affinity rules (k > 20), thời gian sẽ tăng tuyến tính.

2. **Go GC pressure**: Nếu tạo Blueprint mới mỗi generation (không reuse), GC pause có thể thêm 5-10% overhead. Nên dùng object pooling (`sync.Pool`) cho Blueprint structs.

3. **Memory estimate chưa tính**: profiling data, logging buffers, metrics collection. Thêm ~10-20 MB trong production build.

4. **"Near-optimal"**: GA không guarantee global optimum. "Convergence" ở đây nghĩa là fitness score không cải thiện đáng kể sau N generations liên tiếp — đây là **local optimum** hoặc **near-optimal**. Để validate quality, cần so sánh với ILP exact solver trên small instances.

5. **Island Model GA**: Cho XL clusters (5000+ pods), nên chia population thành 4-8 islands chạy trên separate goroutine pools, merge best individuals mỗi 50-100 generations. Giảm wall time thêm 2-3x nhưng tăng code complexity.
