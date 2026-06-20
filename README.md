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

## 🧠 Mathematical Foundation (Under the Hood)

This project is a practical demonstration of solving NP-Hard problems in Operations Research. The core of Kuberina uses a heuristic variation of **Integer Linear Programming (ILP)** to minimize the number of Nodes used ($y_j$) while respecting the 4D spatial constraints of containers.

Kuberina uses a Hybrid Pipeline: go from rough to refinement, combining the speed of experience (Heuristic) and the accuracy of mathematics (Optimization).

### 1. Initializing algorithm: Vector Packing First-Fit Decreasing (FFD)

* **Original maritime:** Stack the heaviest, largest containers (40ft, heavy and oversized goods) first (at the bottom). Only then insert the small containers (20ft) into the remaining gaps.
* **Applied to Kuberina:** This is a Greedy algorithm used to create a draft (Draft Blueprint) in an instant.
* **How it works:** Kuberina will calculate the "synthetic volume" of a Pod based on the weights of the resources:

$$V_i = \alpha \cdot \text{CPU}_i + \beta \cdot \text{RAM}_i + \gamma \cdot \text{GPU}_i$$

The algorithm will sort the list of Pods in descending order of $V_i$. The Pod that consumes the most GPU/RAM will be prioritized for loading onto the Cluster first. The algorithm scans through the list of Nodes, and the Node that has just enough space (First-Fit) is inserted.

### 2. Core Engine: Genetic Algorithm (GA)

* **Original maritime:** After having a draft, the system realizes that some containers are misplaced (for example, refrigerated containers are placed far from power outlets). It will create thousands of "mutations" (randomly swapping containers) and select better packing arrangements through each generation.
* **Applied to Kuberina:** This is the heart of Kuberina (the step you will code in Go). FFD above packs very quickly but is often stuck locally. GA will "evolve" that diagram.
* **How it works:**
*   Kuberina creates 100 different Blueprint versions (Population).
*   It calculates the "Risk Score" (Fitness Score) for each version. For example, violating `PodAntiAffinity` (placing 2 DBs on the same Node) is deducted 1000 points; wasting too much free CPU on a Node is deducted 10 points.
*   It performs **Crossover**: Takes half of the arrangement from Blueprint A and combines it with half of the arrangement from Blueprint B.
*   It performs **Mutation**: Randomly picks a Pod from Node 1 and throws it to Node 3.
*   Run this loop for 5000 generations (takes only a few seconds with Go), the final surviving Blueprint is the most optimal one.

### 3. The Censor: Constraint Satisfaction Problem (CSP) Solver với Forward Checking

* **Original maritime:** Strict rules: Flammable goods are FORBIDDEN from being placed next to food. If violated, the layout is immediately rejected without further calculation.
* **Applied to Kuberina:** K8s has "hard constraints" that the algorithm must not violate.
* **How it works:** Before GA scores or FFD inserts a Pod into a Node, the CSP Solver will cross-check:
* Node has Taint `NoSchedule`? Pod has corresponding Toleration?
* Resource constraints: $\sum \text{CPU}_{\text{req}} \le \text{CPU}_{\text{allocatable}}$
* **Forward Checking** Technique: If placing Pod A on Node 1 makes Pod B (which must go with Pod A) have no room left on Node 1, the algorithm will immediately discard the move of placing Pod A to save time.

---

### Algorithm Mapping Table

| MSC Irina Ship Constraints | Kuberina (Kubernetes Constraints) | Solution Technique (Mathematics) |
| --- | --- | --- |
| **Container Size (20ft, 40ft)** | CPU, RAM, GPU Requests/Limits | MDBP (Multi-Dimensional Bin Packing) |
| **Cảng dỡ hàng (Destination)** | Pod Affinity / Network Latency | Đưa vào hàm Fitness (Cộng điểm nếu ở gần) |
| **(Hazmat)** | Pod Anti-Affinity (Cách ly Noisy Neighbors) | CSP (Hard Constraints - Loại bỏ nước đi) |
| **Container Lạnh (Reefer Plugs)** | Node Selectors / Taints (GPU Nodes) | CSP (Hard Constraints) |
| **Cân bằng trọng tâm (Stability)** | Cân bằng tải (Resource Utilization) | Hàm mục tiêu tối thiểu hóa phương sai tải: $\text{Min} \sum (U_j - \bar{U})^2$ |