# Kuberina: Proposal for v0.2.0 (The MDBP Expansion & Kuberina IR)

- **Quyết định:** Đã được chấp thuận (Accepted) cho Kuberina v0.2.0.
- **Mục tiêu:** Nâng cấp thuật toán Multi-Dimensional Bin Packing từ 3 chiều (CPU, RAM, GPU) lên thành ma trận 7 chiều (bổ sung thêm Storage, Disk I/O R/W, Network I/O In/Out) để giải quyết triệt để bài toán "Noisy Neighbor".

## Tổng quan về kiến trúc mới của Kuberina

1. **kuberina-operator:** Một module rời (tương lai) sẽ lo nhiệm vụ biên dịch Standard Kubernetes Manifests (Helm/Kustomize) thành Kuberina IR (`_infra.yaml` và `_workloads.yaml`) và file blueprint.yaml kết quả của Kuberina lại thành các file của các hệ thống khác (helm, kustomize,...)

2. **kuberina-solver:** Lõi engine Rust, không có gì thay đổi về mặt chức năng, nhận input là IR (`_infra.yaml` và `_workloads.yaml`), giải toán, hết.

3. **kuberina-helper:** các script Python hỗ trợ render file `blueprint.yaml` kết quả thành file html, validate kết quả....

Dưới đây là định dạng chính thức của **Kuberina IR (Intermediate Representation)** dùng làm ngôn ngữ giao tiếp trực tiếp với Rust core engine. Kuberina IR ưu tiên sự tinh gọn, các đơn vị tài nguyên chuẩn hóa, hỗ trợ nhân bản (replicas), và gang-scheduling.

## Định dạng YAML (Kuberina IR v0.2.0)

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