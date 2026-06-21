# Kuberina

Kuberina is an algorithmic architect CLI tool that uses maritime stowage optimization mathematics to generate pre-deployment scheduling blueprint for heterogeneous Kubernetes clusters.

Inspired by how supercomputers solve the stowage planning problem for *MSC Irina*, Kuberina applies combinatorial optimization mathematics to pack Pods tightly into a non-standard K8s infrastructure (CPU, RAM, GPU mixed) before you actually run them.

---

## 💡 Why Kuberina?

The default scheduler of K8s (`kube-scheduler`) is designed to make dynamic decisions in milliseconds. It works by "seeing an empty spot and putting things in it," leading to severe resource fragmentation on expensive heterogeneous clusters.

**Kuberina takes a different approach:**
Instead of racing against time, Kuberina is a **static planning** tool. It compute in minutes to fully solve the Constraint Satisfaction Problem (CSP) and Multi-dimensional Bin Packing, thereby generating a perfect blueprint.

## 🚀 Core Features

* **Maritime-Inspired CSP Engine:** Apply the stowage planning model to cloud computing. Minimize fragmentation and resource waste.
* **Heterogeneous-First:** Understand the differences between regular Nodes, GPU Nodes (Nvidia A100, T4), and memory-optimized Nodes. "Cold cargo" (AI Compute) will always be placed in the correct "socket".
* **Zero-Touch K8s Interference:** Kuberina runs completely independently (Offline/CLI). Does not interfere with the running cluster, does not slow down `kube-apiserver`.
* **Auto-Inject Constraints:** receives pure configuration as input, automatically injects `NodeSelector`, `PodAffinity`, `PodAntiAffinity`, and `Tolerations` rules into the output YAML file.

## ⚙️ How it works

Kuberina receives two inputs:

1. **Infrastructure Map:** Physical/virtual topology of the K8s cluster (Node capacity, Taints, Labels).
2. **Workload Manifests:** List of deployments, AI jobs, services (Resource requirements).

```bash
# Install Kuberina
go install github.com/yourusername/kuberina@latest

# Run the stowage simulation
kuberina plan \
  --infra ./cluster-topology.yaml \
  --workloads ./apps-to-deploy/ \
  --output ./optimized-blueprint/

```

Kuberina runs millions of "stowage" scenarios in memory and returns the `./optimized-blueprint/` directory contains YAML files which includes the most optimal scheduling strategy. You just need to `kubectl apply -f ./optimized-blueprint/`.

# RESEARCH PAPERS

Please check folder `research` for detailed research papers and resources. The below sections are the highlights of the research papers.

## 🧠 Mathematical Foundation (Under the Hood)

This project is a practical demonstration of solving NP-Hard problems in Operations Research. The core of Kuberina uses a heuristic variation of **Integer Linear Programming (ILP)** to minimize the number of Nodes used ($y_j$) while respecting the 4D spatial constraints of containers.

### 1. Initializing algorithm: Vector Packing First-Fit Decreasing (FFD)

* **Original maritime:** Stack the heaviest, largest containers (40ft, heavy and oversized goods) first (at the bottom). Only then insert the small containers (20ft) into the remaining gaps.
* **Applied to Kuberina:** This is a Greedy algorithm used to create a draft (Draft Blueprint) in an instant.
* **How it works:** Kuberina will calculate the "synthetic volume" of a Pod based on the weights of the resources:

$$V_i = \alpha \cdot \text{CPU}_i + \beta \cdot \text{RAM}_i + \gamma \cdot \text{GPU}_i$$

The algorithm will sort the list of Pods in descending order of $V_i$. The Pod that consumes the most GPU/RAM will be prioritized for loading onto the Cluster first. The algorithm scans through the list of Nodes, and the Node that has just enough space (First-Fit) is inserted.

### 2. Core Engine: Genetic Algorithm (GA)

* **Original maritime:** After having a draft, the system realizes that some containers are misplaced (for example, refrigerated containers are placed far from power outlets). It will create thousands of "mutations" (randomly swapping containers) and select better packing arrangements through each generation.
* **Applied to Kuberina:**
*   Kuberina creates a set of different Blueprint versions (Population). Generally, 128 or 256 versions for small cluster and 512 or 1024 for big cluster.
*   It calculates the "Risk Score" (Fitness Score) for each version. For example, violating `PodAntiAffinity` (placing 2 DBs on the same Node) is deducted a large cetain amount of points; wasting too much free CPU on a Node is deducted a small cetain amount of points.
*   Performs **Crossover**: Takes half of the arrangement from Blueprint A and combines it with half of the arrangement from Blueprint B.
*   Performs **Mutation**: Randomly picks a Pod from Node 1 and throws it to Node 3.
*   Run this loop for aleast thousands of generations (takes only a few seconds with multiple Go routines), the final surviving Blueprint is the most optimal one.

### 3. The Censor: Constraint Satisfaction Problem (CSP) Solver with Forward Checking

* **Original maritime:** For instance, Flammable goods are FORBIDDEN from being placed next to food. If violated, the layout is immediately rejected without further calculation.
* **Applied to Kuberina:** K8s has "hard constraints" that the algorithm must not violate.
* **How it works:** Before GA scores or FFD inserts a Pod into a Node, the CSP Solver will cross-check:
* Node has Taint `NoSchedule`? Pod has corresponding Toleration?
* Resource constraints: $\sum \text{CPU}_{\text{req}} \le \text{CPU}_{\text{allocatable}}$
* **Forward Checking** Technique: If placing Pod A on Node 1 makes Pod B (which must go with Pod A) have no room left on Node 1, the algorithm will immediately discard the move of placing Pod A to save time.

---

## Problem Formulation

### **🚢 Bảng Ánh Xạ Kiến Trúc Thuật Toán: Logistics Hàng Hải vs. Kubernetes Scheduling**

Bảng dưới đây thể hiện sự tương đồng 1:1 giữa các quy tắc bốc xếp (Stowage Planning) trên siêu tàu viễn dương (như MSC Irina) và các quy tắc lập lịch tài nguyên trong cụm máy chủ Kubernetes, đặc biệt là cho AI Compute.

| Ngữ Cảnh Hàng Hải (Maritime Stowage) | Ý Nghĩa Thực Tế Trên Tàu | Ngữ Cảnh Kubernetes (Kuberina Context) | Kỹ Thuật Giải Quyết Toán Học (Heuristic/CSP) |
| :---- | :---- | :---- | :---- |
| **Container Dimensions (20ft, 40ft, 45ft)** | Khối lượng và kích thước không gian vật lý của thùng hàng. | **Resource Requests/Limits** (CPU, RAM, Disk, GPU utilization, VRAM requirement). | **Multi-Dimensional Bin Packing (MDBP)**. Mỗi chiều tài nguyên là một dimension của Bin. |
| **Reefer Containers (Hàng lạnh)** | Cần cắm vào các khe (slots) có nguồn điện 3 pha và thiết bị giám sát nhiệt. | **AI Compute / GPU Workloads**. Yêu cầu phần cứng chuyên dụng (Nvidia A100, TPU). | **CSP (Hard Constraints)**. Lọc bằng NodeSelector & NodeAffinity. |
| **Hazmat Segregation (Hàng nguy hiểm)** | Hóa chất dễ cháy KHÔNG ĐƯỢC để cạnh hàng thực phẩm hoặc khu sinh hoạt. | **Pod Anti-Affinity / Taints & Tolerations**. Cách ly Noisy Neighbors hoặc cô lập môi trường bảo mật. | **Conflict Graph / CSP**. Đỉnh là Pod, Cạnh là xung đột. Giải thuật Graph Coloring để tránh đụng độ. |
| **Destination Port (LIFO Rotation)** | Hàng dỡ cảng đầu tiên phải nằm trên cùng, không bị đè bởi hàng cảng cuối. | **Pod Affinity / Network Topology**. Gom các service giao tiếp nhiều vào cùng một Node/Zone để giảm độ trễ mạng. | **Đưa vào hàm Fitness (Soft Constraints)**. Cộng điểm thưởng (![][image1]) vào ma trận khi 2 Pod đồ thị liên kết nằm gần nhau. |
| **Vessel Trim & Stability (Trọng tâm)** | Phân bổ đều trọng lượng mạn trái/phải, trước/sau để tàu không lật. | **Resource Utilization Balancing**. Tránh tình trạng Node 1 chạy 100% CPU, Node 2 chỉ chạy 10%. Tất cả Node đạt trạng thái compute tương đồng nhất có thể để không tạo ra điểm hội tụ nhiệt / sụt áp trên hạ tầng phần cứng. | **Hàm mục tiêu tối thiểu hóa phương sai (Variance Minimization)**: ![][image2]. |
| **Out of Gauge (OOG) Cargo** | Hàng siêu trọng chiếm nhiều slots cùng lúc. | **Gang Scheduling / Distributed AI Training**. Hàng chục Pod AI phải cùng được schedule một lúc trên các Node có NVLink. | **All-or-Nothing Constraint / Block Packing**. Buộc thuật toán nhóm các Pod thành một khối (Macro-block) không thể chia cắt. |
| **Lashing & Securing (Chằng buộc)** | Các loại cáp thép gia cố thêm để giữ chặt các container trên boong khi có bão. | **QoS Classes (Guaranteed vs Burstable)**. Pod "Guaranteed" được khóa chặt (Pin) tài nguyên, không bị evict. | **Knapsack Problem với Strict Bounds**. Các Pod Guaranteed được ưu tiên phân bổ (pre-allocate) trước. |
| **Hatch Covers (Nắp hầm hàng)** | Chia tách khoang dưới boong (Under deck) và trên boong (On deck). | **Topology Spread Constraints / Availability Zones (AZ)**. Phân tán rủi ro, không để tất cả DB Replicas vào chung 1 Rack/Zone. | **Distribution Constraints**. Giới hạn Min-Max sức chứa cho từng phân vùng logic: ![][image3]. |
| **Ballast Water (Nước dằn tàu)** | Bơm nước vào các khoang ngầm để duy trì độ chìm ổn định cơ bản. | **DaemonSets / Core System Pods** (CSI, CNI, kube-proxy). Luôn phải chạy nền trên mọi Node. | **Fixed Variables trong ILP**. Trừ hao tài nguyên cố định trước khi giải bài toán cho các Pod động. |
| **Restows (Đảo hàng / Chuyển mạn)** | Phải cẩu container A xuống bến để lấy được container B nằm bên dưới. Tốn kém chi phí. | **Pod Preemption & Eviction**. Khử các Pod ưu tiên thấp để lấy chỗ trống nhét Pod AI khẩn cấp vào. | **Penalty Function (Hàm phạt)**. Thuật toán trừ điểm nặng nếu sơ đồ dẫn đến tỷ lệ phải Evict cao khi có traffic spike. |
| **BAPLIE / Stowage Plan** | Bản thiết kế sơ đồ bốc xếp cuối cùng gửi cho Cảng biển trước khi tàu cập bến. | **The Pre-deployment Blueprint**. Bộ file YAML/Helm đã được tiêm các rules xuất ra từ Kuberina. | **The Final State Matrix**. Ma trận kết quả đầu ra của thuật toán sau ![][image4] thế hệ tiến hóa. |

*Ghi chú: Sự chặt chẽ của các ràng buộc logic (Hard Constraints \- CSP) sẽ được tính toán trước để cắt tỉa không gian tìm kiếm, sau đó các ràng buộc mềm (Soft Constraints \- Fitness) sẽ được tối ưu hóa bằng thuật toán Heuristic/Di truyền để tìm ra sơ đồ lấp đầy tối đa.*