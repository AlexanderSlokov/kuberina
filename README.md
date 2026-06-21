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

<!-- [Q-Claude] Gợi ý bổ sung cho Core Features:
  1. Có nên thêm feature "Blueprint Report" (visualize kết quả trước khi apply) không?
  2. "Auto-Inject Constraints" — có support Helm chart output hay chỉ raw YAML?
  3. Nên thêm một section "Non-Goals" (những gì Kuberina KHÔNG làm) để đặt kỳ vọng đúng cho user. Ví dụ: Kuberina không phải runtime scheduler, không replace kube-scheduler, không handle autoscaling.
-->

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

<!-- [Q-Claude] Gợi ý bổ sung cho README:
  1. Thêm section "Quick Start" với một ví dụ end-to-end hoàn chỉnh (sample input -> command -> sample output)?
  2. Thêm section "Installation" chi tiết hơn (prerequisites: Go version, OS support)?
  3. "github.com/yourusername/kuberina" — update thành Go module path thật?
  4. Thêm ví dụ output: trước/sau khi Kuberina inject constraints vào YAML?
  5. Badge row (CI status, Go version, license) ở đầu README?
-->

## 📚 Documentation

| Document | Audience | Description |
|---|---|---|
| [PAPER.md](./PAPER.md) | Researchers, reviewers | Full research paper: mathematical foundation, problem formulation, experimental results |
| [DESIGN.md](./DESIGN.md) | Contributors, maintainers | Software design document: architecture, data model, CLI design, testing strategy |
| `research/` | Algorithm developers | Python POC code for formulas and algorithms in small simulations |

## 📄 License

<!-- [Q-Claude] Chọn license nào? MIT, Apache 2.0, hay GPL? 
  Nếu muốn cộng đồng thoải mái contribute: MIT hoặc Apache 2.0.
  Nếu muốn enforce open-source derivatives: GPL.
-->
