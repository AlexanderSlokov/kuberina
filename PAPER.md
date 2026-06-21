# Kuberina: Maritime Stowage-Inspired Combinatorial Optimization for Pre-deployment Scheduling in Heterogeneous Kubernetes Clusters

> **Authors:** _[Tên tác giả]_
>
> **Affiliation:** _[Tên tổ chức / trường]_
>
> **Date:** _[Ngày]_

---

## Abstract

<!-- [Q-Claude] Abstract thường gồm 4-5 câu:
  1. Bối cảnh vấn đề (K8s scheduler fragmentation trên heterogeneous cluster)
  2. Gap trong các giải pháp hiện tại
  3. Đề xuất của bạn (Kuberina - offline static planner dùng FFD + GA + CSP)
  4. Kết quả chính (metrics từ phần Evaluation Goals)
  5. Ý nghĩa / đóng góp
  Nên viết bằng tiếng Anh nếu nhắm đến venue quốc tế. Độ dài khuyến nghị: 150-250 từ.
-->

## 1. Introduction

<!-- [Q-Claude] Phần này nên trả lời:
  - Tại sao kube-scheduler mặc định không đủ tốt cho heterogeneous cluster? Có số liệu cụ thể không (ví dụ: Datadog report 30-40% CPU utilization)?
  - "Static planning" khác gì "dynamic scheduling"? Khi nào static planning có lợi thế hơn?
  - Motivation chính cho analogy hàng hải: bạn tìm thấy mối liên hệ này từ đâu? Từ kinh nghiệm thực tế hay từ literature review?

  Contribution statement — gợi ý 4 contributions:
  1. (Algorithmic) Một hybrid pipeline FFD + GA + CSP Forward Checking cho K8s scheduling offline, lấy cảm hứng từ maritime stowage planning.
  2. (Practical) Một CLI tool tạo ra pre-deployment blueprint có thể kubectl apply trực tiếp, không cần can thiệp vào cluster đang chạy.
  3. (Methodological) Chứng minh rằng analogy giữa container stowage planning và K8s pod scheduling là valid và productive — mỗi constraint trong maritime domain đều có mapping 1:1 sang K8s domain.
  4. (Process) Đề xuất rằng giá trị của offline scheduling optimization không chỉ nằm ở solution quality, mà ở việc tạo ra một **auditable, iterable artifact** (blueprint) cho phép collaborative infrastructure decision-making — tương tự cách Git biến code deployment thành code review, và Terraform biến infrastructure provisioning thành reviewable plan.
     Một bản thiết kế được tính toán bởi combinatorial optimization qua hàng nghìn thế hệ tiến hóa, có cơ sở toán học để bảo vệ mọi quyết định placement, thay thế cho các quyết định scheduling dựa trên trực giác cá nhân (architect intuition) vốn không thể audit, không thể reproduce, và không thể challenge.
-->

## 2. Related Work

<!-- [Q-Claude] Section này hiện đang trống hoàn toàn. Đây là phần bắt buộc cho một bài báo.
  Gợi ý các hướng cần survey:
  1. Kubernetes scheduling optimizers: Descheduler, Volcano, kube-batch, Trimaran — chúng giải quyết vấn đề gì và thiếu gì?
  2. Bin Packing trong cloud: các paper về VM placement, container packing (Google Borg, Tetris scheduler của Microsoft)
  3. Genetic Algorithm cho resource scheduling: đã có ai dùng GA cho K8s scheduling chưa?
  4. Maritime stowage planning literature: các paper gốc về stowage optimization mà bạn lấy cảm hứng (ví dụ: Avriel et al., Pacino et al.)
  5. CSP/ILP trong scheduling: OR-Tools, CP-SAT solver applications

  Câu hỏi quan trọng: Bạn đã có danh sách references chưa? Nếu chưa, tôi có thể giúp tìm các paper liên quan.
-->

## 3. Problem Formulation

### 3.1. Analogy: Maritime Stowage Planning and Kubernetes Scheduling

Bảng dưới đây thể hiện sự tương đồng 1:1 giữa các quy tắc bốc xếp (Stowage Planning) trên siêu tàu viễn dương (như MSC Irina) và các quy tắc lập lịch tài nguyên trong cụm máy chủ Kubernetes, đặc biệt là cho AI Compute.

| Ngữ Cảnh Hàng Hải (Maritime Stowage) | Ý Nghĩa Thực Tế Trên Tàu | Ngữ Cảnh Kubernetes (Kuberina Context) | Kỹ Thuật Giải Quyết Toán Học (Heuristic/CSP) |
| :---- | :---- | :---- | :---- |
| **Container Dimensions (20ft, 40ft, 45ft)** | Khối lượng và kích thước không gian vật lý của thùng hàng. | **Resource Requests/Limits** (CPU, RAM, Disk, GPU utilization, VRAM requirement). | **Multi-Dimensional Bin Packing (MDBP)**. Mỗi chiều tài nguyên là một dimension của Bin. |
| **Reefer Containers (Hàng lạnh)** | Cần cắm vào các khe (slots) có nguồn điện 3 pha và thiết bị giám sát nhiệt. | **AI Compute / GPU Workloads**. Yêu cầu phần cứng chuyên dụng (Nvidia A100, TPU). | **CSP (Hard Constraints)**. Lọc bằng NodeSelector & NodeAffinity. |
| **Hazmat Segregation (Hàng nguy hiểm)** | Hóa chất dễ cháy KHÔNG ĐƯỢC để cạnh hàng thực phẩm hoặc khu sinh hoạt. | **Pod Anti-Affinity / Taints & Tolerations**. Cách ly Noisy Neighbors hoặc cô lập môi trường bảo mật. | **Conflict Graph / CSP**. Đỉnh là Pod, Cạnh là xung đột. Giải thuật Graph Coloring để tránh đụng độ. |
| **Destination Port (LIFO Rotation)** | Hàng dỡ cảng đầu tiên phải nằm trên cùng, không bị đè bởi hàng cảng cuối. | **Pod Affinity / Network Topology**. Gom các service giao tiếp nhiều vào cùng một Node/Zone để giảm độ trễ mạng. | **Fitness Function (Soft Constraints)**. Cộng điểm thưởng vào ma trận khi 2 Pod đồ thị liên kết nằm gần nhau. |
| **Vessel Trim & Stability (Trọng tâm)** | Phân bổ đều trọng lượng mạn trái/phải, trước/sau để tàu không lật. | **Resource Utilization Balancing**. Tránh tình trạng Node 1 chạy 100% CPU, Node 2 chỉ chạy 10%. Tất cả Node đạt trạng thái compute tương đồng nhất có thể để không tạo ra điểm hội tụ nhiệt / sụt áp trên hạ tầng phần cứng. | **Variance Minimization**. |
| **Out of Gauge (OOG) Cargo** | Hàng siêu trọng chiếm nhiều slots cùng lúc. | **Gang Scheduling / Distributed AI Training**. Hàng chục Pod AI phải cùng được schedule một lúc trên các Node có NVLink. | **All-or-Nothing Constraint / Block Packing**. Buộc thuật toán nhóm các Pod thành một khối (Macro-block) không thể chia cắt. |
| **Lashing & Securing (Chằng buộc)** | Các loại cáp thép gia cố thêm để giữ chặt các container trên boong khi có bão. | **QoS Classes (Guaranteed vs Burstable)**. Pod "Guaranteed" được khóa chặt (Pin) tài nguyên, không bị evict. | **Knapsack Problem với Strict Bounds**. Các Pod Guaranteed được ưu tiên phân bổ (pre-allocate) trước. |
| **Hatch Covers (Nắp hầm hàng)** | Chia tách khoang dưới boong (Under deck) và trên boong (On deck). | **Topology Spread Constraints / Availability Zones (AZ)**. Phân tán rủi ro, không để tất cả DB Replicas vào chung 1 Rack/Zone. | **Distribution Constraints**. Giới hạn Min-Max sức chứa cho từng phân vùng logic. |
| **Ballast Water (Nước dằn tàu)** | Bơm nước vào các khoang ngầm để duy trì độ chìm ổn định cơ bản. | **DaemonSets / Core System Pods** (CSI, CNI, kube-proxy). Luôn phải chạy nền trên mọi Node. | **Fixed Variables trong ILP**. Trừ hao tài nguyên cố định trước khi giải bài toán cho các Pod động. |
| **Restows (Đảo hàng / Chuyển mạn)** | Phải cẩu container A xuống bến để lấy được container B nằm bên dưới. Tốn kém chi phí. | **Pod Preemption & Eviction**. Khử các Pod ưu tiên thấp để lấy chỗ trống nhét Pod AI khẩn cấp vào. | **Penalty Function (Hàm phạt)**. Thuật toán trừ điểm nặng nếu sơ đồ dẫn đến tỷ lệ phải Evict cao khi có traffic spike. |
| **BAPLIE / Stowage Plan** | Bản thiết kế sơ đồ bốc xếp cuối cùng gửi cho Cảng biển trước khi tàu cập bến. | **The Pre-deployment Blueprint**. Bộ file YAML/Helm đã được tiêm các rules xuất ra từ Kuberina. | **The Final State Matrix**. Ma trận kết quả đầu ra của thuật toán sau N thế hệ tiến hóa. |

*Ghi chú: Sự chặt chẽ của các ràng buộc logic (Hard Constraints - CSP) sẽ được tính toán trước để cắt tỉa không gian tìm kiếm, sau đó các ràng buộc mềm (Soft Constraints - Fitness) sẽ được tối ưu hóa bằng thuật toán Heuristic/Di truyền để tìm ra sơ đồ lấp đầy tối đa.*

### 3.2. Formal Definition

<!-- [Q-Claude] Bảng analogy ở trên rất trực quan, nhưng một bài báo cần phần formal definition chặt chẽ hơn:
  - Định nghĩa tập hợp: N = {n_1, ..., n_m} (Nodes), P = {p_1, ..., p_k} (Pods), R = {CPU, RAM, GPU, ...} (Resource dimensions)
  - Objective function: minimize gì? Bạn viết "minimize number of Nodes y_j" nhưng fitness function lại có nhiều thành phần (fragmentation, affinity, balance). Vậy đây là single-objective hay multi-objective optimization?
  - Hard constraints vs soft constraints: liệt kê formal dạng bất đẳng thức
  - Bảng notation cho tất cả ký hiệu toán học dùng trong paper

  Câu hỏi: Bạn có muốn formulate bài toán dưới dạng ILP chuẩn (với decision variables x_ij, y_j) không, hay chỉ describe GA-based heuristic?
-->

## 4. Proposed Method

### 4.1. System Overview

<!-- [Q-Claude] Nên có một architecture diagram (figure) ở đây cho thấy pipeline: Input -> FFD -> GA+CSP -> Output.
  Câu hỏi: Tool nhận input dạng gì chính xác? Raw YAML, Helm chart, hay một schema riêng (cluster-topology.yaml)?
-->

### 4.2. Phase 1: Initialization via Vector Packing First-Fit Decreasing (FFD)

* **Original maritime:** Stack the heaviest, largest containers (40ft, heavy and oversized goods) first (at the bottom). Only then insert the small containers (20ft) into the remaining gaps.
* **Applied to Kuberina:** This is a Greedy algorithm used to create a draft (Draft Blueprint) in an instant.
* **How it works:** Kuberina will calculate the "synthetic volume" of a Pod based on the weights of the resources:

$$V_i = \alpha \cdot \text{CPU}_i + \beta \cdot \text{RAM}_i + \gamma \cdot \text{GPU}_i$$

The algorithm will sort the list of Pods in descending order of $V_i$. The Pod that consumes the most GPU/RAM will be prioritized for loading onto the Cluster first. The algorithm scans through the list of Nodes, and the Node that has just enough space (First-Fit) is inserted.

<!-- [Q-Claude] Câu hỏi về FFD:
  1. Các hệ số alpha, beta, gamma được xác định như thế nào? Cố định hay tunable? Giá trị mặc định là bao nhiêu?
  2. "First-Fit" hay "Best-Fit"? Bạn viết First-Fit nhưng có cân nhắc Best-Fit Decreasing (BFD) không? FFD có approximation ratio 11/9 * OPT + 6/9 cho bin packing — bạn có muốn cite bound này không?
  3. Khi Node list cũng heterogeneous (khác capacity), thứ tự duyệt Node có ảnh hưởng không? Node lớn trước hay nhỏ trước?
  4. Nếu có thêm resource dimensions (ví dụ: network bandwidth, ephemeral storage), công thức V_i có scale được không?
-->

### 4.3. Phase 2: Optimization via Genetic Algorithm (GA)

* **Original maritime:** After having a draft, the system realizes that some containers are misplaced (for example, refrigerated containers are placed far from power outlets). It will create thousands of "mutations" (randomly swapping containers) and select better packing arrangements through each generation.
* **Applied to Kuberina:**
*   Kuberina creates a set of different Blueprint versions (Population). Generally, 128 or 256 versions for small cluster and 512 or 1024 for big cluster.
*   It calculates the "Risk Score" (Fitness Score) for each version. For example, violating `PodAntiAffinity` (placing 2 DBs on the same Node) is deducted a large certain amount of points; wasting too much free CPU on a Node is deducted a small certain amount of points.
*   Performs **Crossover**: Takes half of the arrangement from Blueprint A and combines it with half of the arrangement from Blueprint B.
*   Performs **Mutation**: Randomly picks a Pod from Node 1 and throws it to Node 3.
*   Run this loop for at least thousands of generations (takes only a few seconds with multiple Go routines), the final surviving Blueprint is the most optimal one.

<!-- [Q-Claude] Câu hỏi về GA — đây là phần core nên cần chi tiết nhất:
  1. **Chromosome encoding**: Mỗi individual (Blueprint) được encode như thế nào? Array of integers [pod_i -> node_j]? Hay permutation-based?
  2. **Selection method**: Tournament selection? Roulette wheel? Elitism rate bao nhiêu %?
  3. **Crossover operator**: "Takes half" — cụ thể là gì? One-point crossover, two-point, hay uniform? Vì đây là assignment problem (không phải TSP), crossover thông thường có thể tạo ra infeasible solutions (violate capacity). Bạn xử lý repair mechanism thế nào?
  4. **Mutation rate**: Cố định hay adaptive? Giá trị cụ thể?
  5. **Fitness function**: Cần viết formal. Hiện tại mô tả là "deducted points" nhưng cần công thức cụ thể, ví dụ:
     F(s) = w1 * NodeCount(s) + w2 * Fragmentation(s) + w3 * AffinityViolation(s) + w4 * UtilVariance(s)
     Các trọng số w1..w4 được tune thế nào?
  6. **Termination criteria**: "At least thousands of generations" — có early stopping condition không? (ví dụ: convergence detection khi fitness không cải thiện sau N generations)
  7. **Parallelism**: "Multiple Go routines" — island model GA hay chỉ parallel fitness evaluation?
-->

### 4.4. Phase 3: Constraint Enforcement via CSP Solver with Forward Checking

* **Original maritime:** For instance, Flammable goods are FORBIDDEN from being placed next to food. If violated, the layout is immediately rejected without further calculation.
* **Applied to Kuberina:** K8s has "hard constraints" that the algorithm must not violate.
* **How it works:** Before GA scores or FFD inserts a Pod into a Node, the CSP Solver will cross-check:
  * Node has Taint `NoSchedule`? Pod has corresponding Toleration?
  * Resource constraints: $\sum \text{CPU}_{\text{req}} \le \text{CPU}_{\text{allocatable}}$
* **Forward Checking** Technique: If placing Pod A on Node 1 makes Pod B (which must go with Pod A) have no room left on Node 1, the algorithm will immediately discard the move of placing Pod A to save time.

<!-- [Q-Claude] Câu hỏi về CSP:
  1. CSP Solver chạy ở đâu trong pipeline? Trước GA (filter), trong GA (repair), hay cả hai? Diagram hiện tại nói "Before GA scores or FFD inserts" — nghĩa là nó là pre-check cho mọi operation?
  2. Forward Checking: bạn implement full Arc Consistency (AC-3) hay chỉ Forward Checking đơn giản? Sự khác biệt này ảnh hưởng đến pruning efficiency.
  3. Danh sách đầy đủ hard constraints: ngoài Taint/Toleration và resource capacity, còn gì nữa? NodeSelector, NodeAffinity (required), PodAffinity (required)?
  4. Khi GA mutation tạo ra một solution vi phạm hard constraint, bạn: (a) reject mutation, (b) repair solution, hay (c) penalize nặng trong fitness?
-->

## 5. Experimental Setup

<!-- [Q-Claude] Phần này hoàn toàn trống. Để bài báo có sức thuyết phục, cần:
  1. **Testbed description**: Cluster config dùng để test (bao nhiêu Node, specs, loại GPU). Dùng cluster thật hay simulated?
  2. **Workload profiles**: Bao nhiêu Pod, mix giữa CPU-intensive / GPU-intensive / memory-intensive? Lấy từ đâu (synthetic hay real-world trace như Google Cluster Trace, Alibaba Cluster Trace)?
  3. **Baselines**: So sánh Kuberina với gì? Tối thiểu nên có:
     - kube-scheduler mặc định (LeastAllocated / MostAllocated)
     - Random placement
     - Pure FFD (không GA)
     - Có thể thêm: Volcano, kube-batch
  4. **Metrics**: Cụ thể hóa cách đo từng metric (Node count, fragmentation index, utilization %, scheduling success rate)
  5. **Parameter settings**: Population size, generation count, mutation rate, crossover rate, alpha/beta/gamma
  6. **Statistical significance**: Chạy bao nhiêu lần? Report mean +/- std?
-->

## 6. Results and Analysis

<!-- [Q-Claude] Trình bày kết quả cho từng evaluation goal:
  - Bảng so sánh (Table) giữa Kuberina vs baselines
  - Biểu đồ convergence của GA qua các generation
  - Biểu đồ phân bố resource utilization trước/sau optimization
  - Case study cho Gang Scheduling scenario
  Câu hỏi: Bạn đã có kết quả POC từ Python code trong folder research/ chưa? Nếu có, tôi có thể giúp format thành bảng/biểu đồ.
-->

### 6.1. Node Reduction (Infrastructure Cost)

<!-- Target: 10% - 15% reduction -->

### 6.2. Resource Fragmentation

<!-- Target: 30% - 40% reduction -->

### 6.3. Resource Utilization

<!-- Target: 75% - 85% (up from industry average of 30-40%) -->

### 6.4. Scheduling Success Rate for AI Compute

<!-- Target: 100% for Gang Scheduling scenarios -->

### 6.5. Computational Performance

<!-- [Q-Claude] Thêm section này: thời gian chạy của Kuberina scale thế nào khi cluster size tăng? 
  Ví dụ: 50 Pods/10 Nodes vs 500 Pods/100 Nodes vs 5000 Pods/1000 Nodes.
  Đây là câu hỏi reviewer sẽ hỏi đầu tiên.
-->

## 7. Discussion

<!-- [Q-Claude] Nên thảo luận:
  1. **Limitations**: Kuberina là static planner — khi workload thay đổi runtime (autoscaling, crash), blueprint cũ có bị stale không? Cần re-plan frequency thế nào?
  2. **Scalability**: GA với 5000+ Pods có chạy được trong thời gian chấp nhận được không?
  3. **Sensitivity analysis**: Kết quả nhạy cảm thế nào với alpha/beta/gamma và các hyperparameters của GA?
  4. **Practical deployment**: Tích hợp vào CI/CD pipeline thực tế thế nào? Ai trigger kuberina plan? Manual hay automated?
  5. **Threats to validity**: Simulated cluster vs real cluster, synthetic workload vs real workload
-->

## 8. Conclusion and Future Work

<!-- [Q-Claude]
  - Conclusion: tóm tắt lại contributions và kết quả chính (2-3 paragraphs)
  - Future work gợi ý:
    1. Online/incremental re-planning (không cần re-solve toàn bộ)
    2. Multi-objective optimization (Pareto front thay vì weighted sum)
    3. Integration với Kubernetes Scheduler Extender để auto-apply blueprint
    4. Support cho multi-cluster / federation scheduling
    5. Reinforcement Learning thay thế hoặc bổ sung cho GA
-->

## References

<!-- [Q-Claude] Cần ít nhất các nhóm references sau:
  1. Kubernetes scheduling: chính thức docs + các paper cải tiến scheduler
  2. Bin Packing: Coffman et al., Garey & Johnson (NP-hardness proof)
  3. Genetic Algorithm: Holland (1975), Goldberg (1989), hoặc modern survey
  4. Maritime stowage: Avriel et al., Pacino et al., Delgado et al.
  5. Cloud resource management: Google Borg paper, Microsoft Tetris, Alibaba Sigma
  6. Datadog reports (cho số liệu utilization 30-40%)
-->
