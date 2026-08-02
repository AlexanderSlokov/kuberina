# Kuberina Roadmap

Tài liệu này vạch ra lộ trình phát triển của Kuberina.

## v0.2.0: The MDBP Expansion & Kuberina IR 

Tập trung vào việc mở rộng không gian bài toán và chuẩn hóa input/output (IR) của lõi solver.

- **8-Dimensional MDBP Matrix:** Nâng cấp thuật toán Multi-Dimensional Bin Packing từ 3 chiều lên 8 chiều (CPU, RAM, GPU, Storage, Disk Read, Disk Write, Network In, Network Out) nhằm triệt để giải quyết vấn đề "Noisy Neighbor".
- **Kuberina IR v0.2.0:** Định dạng biểu diễn trung gian chuẩn hóa dạng YAML (`_infra.yaml`, `_workloads.yaml`) giao tiếp trực tiếp với Rust core engine.
- **Advanced K8s Features:** 
  - Hỗ trợ unroll `replicas` tự động để giảm kích thước manifest.
  - Phân tích và convert các đơn vị chuẩn K8s (m, Mi, Gi, k, M, G).
  - Tích hợp `topologySpread` constraint (maxSkew) dưới dạng Soft Penalty.
  - Đơn giản hóa Gang scheduling.

## v0.3.0: Kuberina Forge & Hexagonal Architecture

Biến Kuberina thành một compiler pipeline thực thụ (tương tự kiến trúc LLVM).

- **`kuberina-forge` (Go CLI):** Phát triển một khối độc lập đóng vai trò "Lò rèn".
  - *Forge-in (Frontend):* Parse K8s manifests, Helm charts, Kustomize, API cloud (AWS, GCP) -> Emit ra Kuberina IR.
  - *Forge-out (Backend/Linker):* Đọc `blueprint.yaml` (kết quả từ solver) -> Render ra file manifest hoặc gọi API trực tiếp áp dụng lên target system.
- **Hexagonal Architecture (Ports & Adapters):** Refactor `kuberina-solver` (Rust) để tách biệt hoàn toàn Domain Core (FFD, GA, CSP) khỏi I/O (Parser, Writer), biến các module đọc/ghi thành Adapter.

## v0.4.0+: gRPC, Distributed Solving & Agnostic Target

Tách rời việc tính toán nặng ra khỏi môi trường vận hành đắt đỏ, và hỗ trợ các nền tảng ngoài K8s.

- **gRPC & Protobuf:** Giao tiếp giữa `kuberina-forge` và `kuberina-solver` chuyển từ static YAML files sang chuẩn Protobuf/gRPC, tạo nền tảng cho distributed architecture.
- **Tính Đa dụng (Agnostic):** Vì Solver chỉ nhận dữ liệu IR, Kuberina không còn bị trói buộc vào K8s. Có thể giải quyết bài toán quy hoạch cho: máy ảo Proxmox, HashiCorp Nomad, cấp phát tài nguyên HPC (Slurm), tối ưu hóa kho bãi IoT, hoặc trở về bài toán Maritime Stowage gốc.
- **Bảo mật & Tính toán Phân tán (Cloud-to-Edge):** 
  - Các cụm K8s chạy ở Cloud (như AWS, GCP - tốn chi phí compute lớn). Kuberina agent (`kuberina-forge`) trên Cloud chỉ đóng vai trò lấy dữ liệu.
  - Agent gọi gRPC truyền dữ liệu về một máy vật lý tại local (ví dụ: máy tính ThinkCentre ở nhà). Lõi `kuberina-solver` (Rust) tính toán tối ưu cục bộ, sau đó gửi `blueprint` hoàn chỉnh ngược lên Cloud.
  - **Lợi ích:** Tiết kiệm toàn bộ chi phí chạy thuật toán tối ưu (GA) trên Cloud, đồng thời bảo mật mô hình tính toán.