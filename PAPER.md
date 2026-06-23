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

### 3.1. Google Autopilot

Google Autopilot là một giải pháp bị giới hạn về mặt thời gian (time-bounding). Nó sử dụng các cửa sổ trượt (moving windows) và thuật toán học máy dựa trên dữ liệu tiêu thụ tài nguyên trong quá khứ. Nếu nó set limit RAM quá thấp và một container bị chết (OOM), nó sẽ ghi nhận "chi phí" của sai lầm đó. Ở chu kỳ tính toán tiếp theo, nó sẽ tăng limit lên để bù đắp. Tuy nhiên, Autopilot giả định tài nguyên của cluster là vô hạn: nó không quan tâm máy chủ vật lý có đủ chỗ hay không. Thiếu thì cứ tăng limit, Borg sẽ tự động đi tìm (hoặc đẻ thêm) máy mới để chứa. Ràng buộc duy nhất của nó là làm sao để bám sát theo sự thay đổi của tải qua thời gian.

Kuberina thì ngược lại so với Autopilot, nó bị giới hạn bởi phần cứng vật lý mà toàn bộ cụm tài nguyên có (resource-bounding). Kuberina không giả định rằng mây sẽ tự đẻ ra máy. Nó nhận đầu vào là các giới hạn cứng: có 10 Node, tổng cộng 640GB RAM và 8 card A100 và thuật toán không được quyền vượt qua các ràng buộc này. Vì bị khóa chặt trong một hộp tài nguyên cố định, Kuberina không "đoán thời gian", mà nó chơi trò hình học không gian (Đóng gói đa chiều - MDBP). Nó thử xoay, lật, nhét các khối workloads vào các khe hở của Node sao cho vừa khít nhất.

| Đặc tính | Google Autopilot (Time-Bounding) | Kuberina (Resource-Bounding) |
| --- | --- | --- |
| **Vùng hoạt động** | Runtime | Pre-deployment |
| **Hệ quy chiếu** | Trục Thời Gian | Trục Không Gian |
| **Dữ liệu đầu vào** | Metrics lịch sử (CPU/RAM tiêu thụ thực) | Yêu cầu tài nguyên tĩnh (declarative Requests/Limits) |
| **Cách xử lý sai lầm** | Sửa sai ở chu kỳ thời gian tiếp theo (Learn from failure) | Cấm sai lầm bằng cách cắt tỉa nhánh (Pruning) thuật toán trước khi deploy. |
| **Hàm mục tiêu** | Bám sát tải thực tế, tự động nới lỏng/siết chặt. | Nhét được số lượng Pod tối đa vào một dung lượng Node cố định. |


Điều này không có nghĩa là Kuberina không thể học hỏi từ Autopilot. Trên thực tế, Autopilot và Kuberina là hai mặt bổ sung cho nhau. Kuberina giải quyết bài toán theo trục không gian (giả lập mọi khả năng ở hiện tại trước khi triển khai). Nó vẽ ra một cái khung hoàn hảo, nhưng cứng nhắc. Nếu đời thực đột nhiên chệch nhịp, Kuberina's `blueprint` tĩnh sẽ gặp rủi ro. Autopilot giải quyết bài toán theo trục thời gian (sửa sai bằng cách học từ quá khứ). Nó cực kỳ linh hoạt, nhưng vì nó "mù" về tổng thể giới hạn vật lý và có thể vô tình nới rộng giới hạn (Limit) của một số Pod đến mức làm vỡ cụm máy chủ. Khi kết hợp lại: Kuberina sẽ đóng vai trò xây dựng hai bờ đê vật lý (Resource-bounding), Autopilot sẽ là dòng nước chảy bên trong con đê đó (Time-bounding). 

Chúng tôi gọi hiện tượng nảy sinh từ sự kết hợp giữa hai phần mềm này là "resource canal". The core of Autopilot's algorithm is an `Arg Min` function (tìm giá trị nhỏ nhất) của hàm chi phí: Chi phí Overrun (cấp thiếu tài nguyên dẫn đến OOM/chậm) và chi phí Underrun (cấp thừa tài nguyên dẫn đến lãng phí máy chủ). Khi Autopilot chạy trong "resource canal" do Kuberina tạo ra, hàm chi phí này không còn là phép thử-sai mù quáng nữa. Kuberina đã chặn đứng cực trị của Underrun và Overrun ngay từ đầu, khiến thuật toán học tăng cường (RL) của Autopilot hội tụ (converge) nhanh gấp hàng chục lần.

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
| **Block Booking / Slot Charter (Đặt chỗ theo lô)** | Một khách hàng lớn (shipper) đặt trước N slots container trên một chuyến tàu. Hãng tàu phải xếp được **tất cả N container**, hoặc **từ chối cả booking**. Tuy nhiên, mỗi container trong lô vẫn là thực thể riêng biệt — vẫn có trọng lượng riêng, vẫn có thể là reefer/hazmat, vẫn ảnh hưởng trọng tâm bay nó đậu, vẫn phải tuân thủ mọi ràng buộc cá thể. Constraint "all-or-nothing" tồn tại ở **tầng thương mại** (booking), không ở tầng vật lý (kích thước). | **Gang Scheduling / Distributed AI Training**. Một nhóm Pod AI (ví dụ 8 workers) phải cùng được schedule, hoặc không Pod nào được schedule. Nhưng mỗi Pod vẫn là thực thể độc lập — tiêu hao resource riêng trên Node nó đậu, vẫn gây noisy neighbor, vẫn phải thỏa mãn affinity/anti-affinity cá nhân. Constraint "all-or-nothing" tồn tại ở **tầng workload** (training job), không ở tầng resource (kích thước Pod). | **Coupled Variables trong CSP**. Mỗi Pod trong gang là một biến quyết định riêng biệt (pod_i → node_j), nhưng các biến này bị **ràng buộc liên kết**: tất cả phải feasible đồng thời, hoặc toàn bộ assignment bị reject. GA encode ở pod-level, enforce gang constraint qua Forward Checking + hard penalty trong fitness. |
| **Lashing & Securing (Chằng buộc)** | Các loại cáp thép gia cố thêm để giữ chặt các container trên boong khi có bão. | **QoS Classes (Guaranteed vs Burstable)**. Pod "Guaranteed" được khóa chặt (Pin) tài nguyên, không bị evict. | **Knapsack Problem với Strict Bounds**. Các Pod Guaranteed được ưu tiên phân bổ (pre-allocate) trước. |
| **Hatch Covers (Nắp hầm hàng)** | Chia tách khoang dưới boong (Under deck) và trên boong (On deck). | **Topology Spread Constraints / Availability Zones (AZ)**. Phân tán rủi ro, không để tất cả DB Replicas vào chung 1 Rack/Zone. | **Distribution Constraints**. Giới hạn Min-Max sức chứa cho từng phân vùng logic. |
| **Ballast Water (Nước dằn tàu)** | Bơm nước vào các khoang ngầm để duy trì độ chìm ổn định cơ bản. | **DaemonSets / Core System Pods** (CSI, CNI, kube-proxy). Luôn phải chạy nền trên mọi Node. | **Fixed Variables trong ILP**. Trừ hao tài nguyên cố định trước khi giải bài toán cho các Pod động. |
| **Restows (Đảo hàng / Chuyển mạn)** | Phải cẩu container A xuống bến để lấy được container B nằm bên dưới. Tốn kém chi phí. | **Pod Preemption & Eviction**. Khử các Pod ưu tiên thấp để lấy chỗ trống nhét Pod AI khẩn cấp vào. | **Penalty Function (Hàm phạt)**. Thuật toán trừ điểm nặng nếu sơ đồ dẫn đến tỷ lệ phải Evict cao khi có traffic spike. |
| **BAPLIE / Stowage Plan** | Bản thiết kế sơ đồ bốc xếp cuối cùng gửi cho Cảng biển trước khi tàu cập bến. | **The Pre-deployment Blueprint**. Bộ file YAML/Helm đã được tiêm các rules xuất ra từ Kuberina. | **The Final State Matrix**. Ma trận kết quả đầu ra của thuật toán sau N thế hệ tiến hóa. |

*Ghi chú: Sự chặt chẽ của các ràng buộc logic (Hard Constraints - CSP) sẽ được tính toán trước để cắt tỉa không gian tìm kiếm, sau đó các ràng buộc mềm (Soft Constraints - Fitness) sẽ được tối ưu hóa bằng thuật toán Heuristic/Di truyền để tìm ra sơ đồ lấp đầy tối đa.*

### 3.2. Formal Definition

#### Notation

| Symbol | Definition |
|---|---|
| $\mathcal{N} = \{n_1, \ldots, n_m\}$ | Set of Nodes in the cluster |
| $\mathcal{P} = \{p_1, \ldots, p_k\}$ | Set of Pods to schedule (excluding DaemonSet pods) |
| $\mathcal{R} = \{\text{CPU}, \text{RAM}, \text{GPU}, \ldots\}$ | Set of resource dimensions |
| $\mathcal{G} = \{G_1, \ldots, G_q\}$ | Set of Pod Groups (gangs) |
| $\mathcal{D} = \{d_1, \ldots, d_h\}$ | Set of DaemonSets |
| $x_{ij} \in \{0, 1\}$ | Decision variable: 1 if pod $p_i$ is assigned to node $n_j$ |
| $y_j \in \{0, 1\}$ | 1 if node $n_j$ has at least one pod assigned |
| $\text{req}_i^r$ | Resource request of pod $p_i$ for resource $r \in \mathcal{R}$ |
| $C_j^r$ | Allocatable capacity of node $n_j$ for resource $r$, **after DaemonSet pre-deduction** |
| $U_j^r$ | Utilization of node $n_j$ for resource $r$: $U_j^r = \sum_{i} x_{ij} \cdot \text{req}_i^r / C_j^r$ |

#### Phase 0: DaemonSet Pre-deduction (Fixed Variables)

DaemonSets are not decision variables — they are the ship's own systems (ballast, monitoring, comms), pre-deducted before optimization begins:

$$C_j^r = C_{j,\text{raw}}^r - \sum_{d \in \mathcal{D}} \mathbb{1}[\text{eligible}(d, n_j)] \cdot \text{res}_d^r$$

where $\mathbb{1}[\text{eligible}(d, n_j)]$ is 1 if DaemonSet $d$ runs on node $n_j$ (based on nodeSelector and tolerations). After this step, $\mathcal{P}$ and $C_j^r$ are the only inputs to the optimizer.

#### Decision Variables (Chromosome Encoding)

Each solution (Blueprint) is encoded as a pod-level assignment vector:

$$\mathbf{s} = [x_1, x_2, \ldots, x_k] \quad \text{where } x_i \in \{1, \ldots, m\} \text{ is the node index for pod } p_i$$

Gang pods are **not** aggregated into macro-blocks. Each pod in a gang remains an individual decision variable (coupled variable in CSP), because each pod independently consumes resources on its assigned node.

#### Objective Function (Single-objective, Weighted Sum)

$$\min F(\mathbf{s}) = w_1 \cdot f_{\text{nodes}}(\mathbf{s}) + w_2 \cdot f_{\text{frag}}(\mathbf{s}) + w_3 \cdot f_{\text{affinity}}(\mathbf{s}) + w_4 \cdot f_{\text{var}}(\mathbf{s}) + \Phi(\mathbf{s})$$

where:

| Component | Formula | Maritime Analogy |
|---|---|---|
| $f_{\text{nodes}}$ | $\sum_{j=1}^{m} y_j$ (number of active nodes) | Minimize number of bays used |
| $f_{\text{frag}}$ | $\sum_{j: y_j=1} \sum_{r} \max(0, C_j^r - \sum_i x_{ij} \cdot \text{req}_i^r)$ (wasted capacity) | Minimize empty slots in used bays |
| $f_{\text{affinity}}$ | Number of soft affinity/anti-affinity rule violations | Destination port grouping violations |
| $f_{\text{var}}$ | $\text{Var}(\{U_j^r : y_j = 1\})$ (utilization variance across active nodes) | Vessel trim & stability |
| $\Phi(\mathbf{s})$ | Hard constraint penalty: $-\infty$ if any hard constraint violated | Immediate rejection of illegal stowage |

#### Hard Constraints (CSP — must not violate)

1. **Capacity**: No node exceeds allocatable resources on any dimension.

$$\forall j, \forall r \in \mathcal{R}: \quad \sum_{i=1}^{k} x_{ij} \cdot \text{req}_i^r \le C_j^r$$

2. **Assignment**: Every pod is assigned to exactly one node.

$$\forall i: \quad \sum_{j=1}^{m} x_{ij} = 1$$

3. **Taint/Toleration**: Pod can only be placed on a tainted node if it has the matching toleration.

$$\forall i, j: \quad x_{ij} = 1 \implies \text{Taints}(n_j) \subseteq \text{Tolerations}(p_i)$$

4. **NodeSelector / NodeAffinity (required)**: Pod can only be placed on nodes matching its selector.

$$\forall i, j: \quad x_{ij} = 1 \implies \text{Labels}(n_j) \supseteq \text{Selector}(p_i)$$

5. **Gang All-or-Nothing (Block Booking)**: For each pod group $G_q = \{p_{q_1}, \ldots, p_{q_t}\}$, either all pods are feasibly placed, or none.

$$\forall G_q \in \mathcal{G}: \quad \sum_{i \in G_q} \mathbb{1}[\text{feasible}(p_i)] = |G_q| \quad \text{or} \quad 0$$

This is a coupled constraint — each $x_{q_l, j}$ is a separate decision variable, but the group constraint binds them. (Maritime analogy: Block Booking, not OOG — individual containers with a commercial all-or-nothing commitment.)

#### Soft Constraints (Fitness — optimize but don't reject)

1. **Pod Affinity (preferred)**: Reward co-locating communicating pods on same node/zone.
2. **Pod Anti-Affinity (preferred)**: Penalize co-locating conflicting pods.
3. **Topology Spread**: Penalize uneven distribution across zones/racks.
4. **Utilization Balance**: Minimize variance of utilization across active nodes (vessel stability).

#### Complexity

The problem is a Multi-Dimensional Bin Packing Problem (MDBP), known to be **NP-hard** (Garey & Johnson, 1979). The search space is:

$$|\mathcal{S}| = m^k$$

For a medium cluster ($m = 100, k = 500$): $|\mathcal{S}| = 10^{1000}$ — brute-force is infeasible. This motivates the hybrid FFD (warm-start) + GA (heuristic optimization) + CSP (constraint enforcement) approach.

## 4. Proposed Method

### 4.1. System Overview

<!-- [Q-Claude] Nên có một architecture diagram (figure) ở đây cho thấy pipeline: Input -> FFD -> GA+CSP -> Output.
  Câu hỏi: Tool nhận input dạng gì chính xác? Raw YAML, Helm chart, hay một schema riêng (cluster-topology.yaml)?
-->

### 4.2. Phase 1: Initialization via Vector Packing First-Fit Decreasing (FFD)

**Motivation**: A purely random initialization for the Genetic Algorithm in a highly constrained space (such as heterogeneous Kubernetes scheduling) results in an initial population composed almost entirely of infeasible solutions (e.g., violating capacity constraints). Correcting these violations takes the GA an exorbitant number of generations.

**The FFD Warm-Start**: We apply a greedy First-Fit Decreasing algorithm to generate a set of *feasible* initial blueprints, accelerating GA convergence by 3-5x.
1. **Synthetic Volume Calculation**: We calculate a scalar weight $V_i$ for each pod based on normalized resource scarcity:
   $$V_i = \alpha \cdot \text{CPU}_i + \beta \cdot \text{RAM}_i + \gamma \cdot \text{GPU}_i$$
   where $\alpha, \beta, \gamma$ are tunable parameters reflecting the relative cost or scarcity of resources in the specific cluster.
2. **Decreasing Sort**: Pods are sorted in descending order of $V_i$. (Maritime analogy: stow the heaviest and largest containers first).
3. **First-Fit Placement**: The algorithm iterates through the sorted pods and places each pod into the first node that has sufficient residual capacity.

This fast $O(k \log k + k \cdot m)$ heuristic produces the seed population for the GA.

### 4.3. Phase 2: Optimization via Genetic Algorithm (GA)

The GA optimizes the soft constraints (affinity, resource balancing, fragmentation) taking the FFD output as its starting point.

1. **Population & Parallelism**: The population size is scaled based on the problem size (e.g., $|Pop| = 512$ for a medium cluster of 100 nodes and 500 pods). Because fitness evaluation for each individual is completely independent, we implement an embarrassingly parallel evaluation model using Go routines, achieving evaluation times of under 10 milliseconds per generation on an 8-core CPU.
2. **Selection**: We use Tournament Selection with a tournament size $k_{tour}=3$ to maintain high selection pressure while preserving diversity.
3. **Crossover with Gang Repair**: We apply Uniform Crossover. However, standard crossover can break the feasibility of Gang Scheduling (Block Booking). If a crossover operation splits a gang (e.g., pods 1-4 inherit from parent A, pods 5-8 inherit from parent B) and violates the node's capacity, a **Repair Mechanism** is triggered: the algorithm rolls back the entire gang's assignment to match the parent that yielded a feasible placement for that gang.
4. **Mutation with Forward Checking**: We apply a random reset mutation with rate $p_m \approx 0.05$. Crucially, mutation is deeply integrated with the CSP Solver. Before a pod is moved to a new node, the solver performs a forward capacity check. If the mutation violates hard constraints (or breaks the gang's all-or-nothing constraint), the mutation is rejected (rolled back).
5. **Termination**: The GA employs an early stopping criterion. If the best fitness score in the population does not improve for $N_{stop}$ consecutive generations (e.g., 100 generations), the algorithm assumes it has converged to a near-optimal local minimum and halts.

### 4.4. Phase 3: Constraint Enforcement via CSP Solver with Forward Checking

Unlike traditional pipelines where the solver is a separate sequential step, Kuberina tightly integrates the CSP solver *into* the FFD and GA operators (Mutation and Repair).

* **Hard Constraint Filtering**: Every placement decision (FFD insertion or GA mutation) is pre-screened by the CSP solver against Taints, Tolerations, NodeSelectors, and exact Resource capacities. If an assignment is invalid, it is pruned immediately, saving the computational cost of full fitness evaluation.
* **Forward Checking for Block Booking**: When evaluating a placement for a pod belonging to a gang $G_q$, the CSP solver employs Forward Checking. It does not merely check if the target node has room for the *single* pod; it verifies if the target node (or set of eligible nodes) possesses enough total residual capacity to accommodate the *entire* group $G_q$. If the collective requirement cannot be met, the branch is discarded instantly. This prevents the optimizer from wandering into deep infeasible regions of the search space.

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
