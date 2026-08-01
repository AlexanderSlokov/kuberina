# Kuberina

[![DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.21582492.svg)](https://doi.org/10.5281/zenodo.21582492)

Kuberina is an algorithmic architect CLI tool that uses maritime stowage optimization mathematics to generate pre-deployment scheduling blueprint for heterogeneous Kubernetes clusters.

Inspired by how supercomputers solve the stowage planning problem for *MSC Irina*, Kuberina applies combinatorial optimization mathematics to pack Pods tightly into a non-standard K8s infrastructure (CPU, RAM, GPU mixed) before you actually run them.

---

# Important: licensing roadmap

Kuberina employs a 3-phases licensing strategy to ensure sustainable development, protect intellectual property against cloud-provider exploitation and deliver a fully permissive open-source project to the community. 

Please read the following licensing transitions carefully. By using, contributing to, or distributing this software, you acknowledge and agree to this roadmap:

### Phase 1: Current State - Kuberina (AGPLv3)

**Applies to:** All current versions starting from `0.1.0`, and all subsequent releases, up to (but not including) version `12.0.0`, unless Phase 2 is triggered first.  
**License:** **GNU Affero General Public License v3.0 (AGPLv3)**.  
**Implication:** Any modifications or deployments of Kuberina provided as a network service must make their source code available to users under the same AGPLv3 license.  

### Phase 2: Naughtian Kuberina  (SSPLv1)

**Trigger Condition:** The formal legal incorporation of the author's corporate entity (e.g., "Naughtian Lab" or "Naughtian Corp").  
**Action:** Upon this event, the intellectual property and repository ownership of Kuberina will be transferred to said corporate entity.   
**License:** From the exact date of transfer forward, all new versions and commits of Kuberina will be licensed under the **Server Side Public License v1 (SSPLv1)**.  
*Note: Previously released AGPLv3 versions will remain AGPLv3. Re-licensing applies strictly to new releases following the corporate transfer.*  

### Phase 3: Liberated Kuberina (Apache License 2.0)

The repository and its subsequent releases will permanently transition to the permissive **Apache License 2.0** immediately upon the occurrence of *either* of the following two milestone events:

1. **CNCF Acceptance:** Kuberina is officially accepted and donated as a hosted project within the Cloud Native Computing Foundation (CNCF).
2. **Reaching Version 12.0.0:** Kuberina officially releases version `12.0.0`. 
  
**LTS Guarantee:** Upon the release of `12.0.0`, a dedicated Long-Term Support branch (`12.0.0-lts`) will be established. The corporate entity established at **Phase 2** will guarantees a minimum **5-year Service Level Agreement (SLA)** for maintenance and security patching on this specific LTS release under the Apache 2.0 license.


## 💡 Why Kuberina?

The default scheduler of K8s (`kube-scheduler`) is designed to make dynamic decisions in milliseconds. It works by "seeing an empty spot and putting things in it," leading to severe resource fragmentation on expensive heterogeneous clusters.

**Kuberina takes a different approach:**
Instead of racing against time, Kuberina is a **static planning** tool. It computes in seconds to solve the Constraint Satisfaction Problem (CSP) and Multi-dimensional Bin Packing, thereby generating an optimal blueprint.

## 🔁 Kuberina as an Infrastructure Decision Protocol

**The real problem isn't optimization — it's that scheduling decisions are invisible.**

Today, `kube-scheduler` makes placement decisions inside a black box. No one reviews them. No one debates them. When Node 7 hits 98% CPU while Node 12 sits at 15%, nobody can explain why — because the decision was never written down.

Kuberina changes this by producing a **reviewable blueprint** — a concrete YAML artifact that your team can open, inspect, challenge, and iterate on:

```
# A typical Kuberina workflow:
kuberina plan → blueprint.yaml          # 10 seconds

# Team review:
"Move Loki to Node 4, it's stressing frontend disk I/O."
"Rejected — Node 4 has Redis, kernel tuning conflict. Add a rule instead."

kuberina plan → blueprint-v2.yaml       # 10 seconds
# Repeat until consensus.

kubectl apply -f blueprint-final.yaml   # Peer-reviewed. Mathematically grounded.
```

This is the same paradigm shift that **Git** brought to code (reviewable diffs instead of FTP uploads) and **Terraform** brought to infrastructure (`terraform plan` instead of clicking in the AWS console). Kuberina brings it to **Kubernetes scheduling**: every pod placement is computed by combinatorial optimization, written down, and open to debate.

A blueprint backed by 2,000 generations of evolutionary optimization across millions of stowage scenarios is infinitely more defensible than a whiteboard drawing from an architect whose reasoning is "10 years of experience" and "trust me."

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
# Build the Rust solver engine
make solver-build

# Run the stowage simulation (MSC Irina scale datacenter)
make solver-irina

# Run the independent validator & heatmap dashboard
make solver-inspect
```

Kuberina runs millions of "stowage" scenarios in memory using an evolutionary algorithm (GA). It outputs a stowage plan to the console and generates `kuberina_solution.yaml`. 
You can then open `kuberina_dashboard.html` in your browser to interactively view the cluster heatmap!

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



<!-- [Q-Claude] Chọn license nào? MIT, Apache 2.0, hay GPL? 
  Nếu muốn cộng đồng thoải mái contribute: MIT hoặc Apache 2.0.
  Nếu muốn enforce open-source derivatives: GPL.
-->
