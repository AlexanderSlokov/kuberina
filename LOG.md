# Research Log: The "MSC Irina" Infeasible Plateau Anomaly

## Vấn đề (The Problem)

Trong bài stress-test "MSC Irina Scale" (150 nodes, 2714 pods, 90% CPU fill rate, 95% GPU fill rate), thuật toán Genetic Algorithm (GA) của Kuberina bị rơi vào một vòng lặp vô cực (infinite loop of infeasible space), với `seed fitness = inf` và không có bất kỳ sự cải thiện nào qua các thế hệ.

**Nguyên nhân cốt lõi (Root Cause):**
1. **Lỗi ưu tiên của FFD (Fragmentation Trap):**
   - Hàm Warm-start `ffd_warmstart` xếp Pod theo `Synthetic Volume` ($V = CPU + RAM + GPU$).
   - Các Pod như `postgres-primary` (6C/96G) nặng hơn `training-worker` (12C/48G/4GPU) nên được xếp trước.
   - Cả 2 Node Memory và Node GPU đều có label `disk=nvme` mà `postgres` yêu cầu. Do xếp trước, `postgres` nhồi chật cứng các Node GPU.
2. **GPU Pod bị "vô gia cư":**
   - Khi tới lượt xếp `training-worker` (bắt buộc phải vào Node GPU do `gpu=nvidia-a100`), các Node GPU đã không còn đủ CPU/RAM.
   - FFD bắt buộc ném `training-worker` vào Node 0 như một giải pháp Fallback.
3. **Mù lòa Đạo hàm (Zero-Gradient / Infeasible Plateau):**
   - Ném vào Node 0 gây ra hai vi phạm cứng (Hard Constraint): NodeSelector và Capacity.
   - Hàm `compute_fitness` kiểm tra Hard Constraints qua `check_capacity_all_nodes()` và trả về `math.inf` nếu thất bại.
   - Khi toàn bộ 1024 cá thể trong quần thể bị phạt `math.inf`, phương pháp Tournament Selection của GA hoàn toàn mất khả năng phân biệt cá thể "tệ" và "đỡ tệ". Đạo hàm bằng 0, không có hướng tiến hóa (No Evolutionary Gradient).

## Đề xuất Giải pháp (Proposed Solution)

**Chuyển đổi Hard Constraint Penalty từ Binary/Infinity sang Soft Gradient Penalty.**

1. Sửa hàm kiểm tra Capacity để tính toán cụ thể mức độ tràn tài nguyên:
   - Thay vì trả về `True/False`, hàm sẽ tính tổng lượng tài nguyên tràn (Overflow Volume) trên toàn Cluster.
   - Công thức: `Overflow = Σ max(0, Node_Load - Node_Cap)` cho mọi tài nguyên (CPU, RAM, GPU).

2. Sửa hàm kiểm tra NodeSelector (bổ sung penalty nếu chưa có):
   - Đếm số lượng Pod bị xếp sai NodeSelector (chẳng hạn `training-worker` rơi vào Node 0).

3. Cập nhật `compute_fitness`:
   - Gỡ bỏ `math.inf`.
   - Tính điểm phạt vi phạm cứng khổng lồ nhưng có độ dốc (ví dụ: `Base_Penalty (1,000,000) + Overflow_Amount * 10,000`).
   - Nhờ đó, nếu GA tạo ra một đột biến (Mutation) làm giảm mức độ tràn 1GB RAM, Fitness sẽ giảm từ 1,050,000 xuống 1,040,000. Cá thể này sẽ thắng Tournament Selection và gen của nó được bảo tồn.

Sự chuyển đổi sang Soft Constraint với Gradient dốc đứng này sẽ cho phép GA tự động học cách di chuyển `postgres` ra khỏi Node GPU và dọn chỗ cho `training-worker`, tự động sửa chữa những sai lầm ban đầu của FFD.
