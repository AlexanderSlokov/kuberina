## Kuberina demands it!

- No GC.
- Memory Contiguity.
- Parallel Computing. Seriously!
- Rust newest features.
- No unsafe if "Kuberina not demands it".

## Features có thì càng tốt:

### 1. Log rõ ràng hơn theo từng step thế hệ khi chạy. Có thanh trạng thái thì càng tốt. Like this, like Docker, ví dụ thế, for better UX:

```bash
d66d6a6a3687: Downloading [===========================>                       ]  16.78MB/30.45MB
```

## Maybe this is roadmap?

1. Giữ nguyên bộ Python code research, cùng thân quyến của nó: 
	- Script Python mà Claude vừa viết để làm Testcase Generator (chuyên đẻ ra YAML giả lập Datacenter). 
    - Bộ testsuite và CLI convention phải được bảo toàn, và sẽ cần để port sang Rust.

2. CPU Cache (Memory Contiguity): Bài toán Đóng gói (Bin Packing) cần duyệt qua mảng dữ liệu cực kỳ nhiều lần. Thiết kế các mảng (Array/Vector) nằm liền kề nhau trên vùng bộ nhớ vật lý (khuyến khích dùng ArenaAllocator). Khi nhân CPU nạp mảng dữ liệu đó lên L1/L2/L3 Cache thì sẽ duyệt qua nhanh hơn nhiều.

3. Dùng thư viện `rayon`. Sửa đúng một dòng code từ `.iter()` (chạy tuần tự) thành `.par_iter()` (chạy song song). `rayon` sẽ tự động xé nhỏ 1.024 cá thể đó ra, nhét đầy vào tất cả các nhân CPU đang có của máy tính, và vắt kiệt phần cứng mà trình biên dịch vẫn đảm bảo 100% không bao giờ xảy ra lỗi xung đột bộ nhớ.

4. **`kuberina-operator`**: Chơi với K8s API thì không ai làm mượt, chuẩn, và được support tốt bằng Go. `kuberina-operator` là một K8s Controller chạy ngầm trong cụm. Nó dùng thư viện `client-go` chuẩn chỉ để lắng nghe sự thay đổi của K8s (watch Pods, Nodes). Khi cần quy hoạch, nó thu thập toàn bộ trạng thái cụm, nén thành một gói tin Protobuf (`TopologyRequest`).

5. **`kuberina-solver`**: Đây là một service độc lập, lắng nghe qua cổng gRPC. Nó không cần biết K8s là gì, không cần import thư viện K8s. Nó nhận gói `TopologyRequest` từ gRPC, bung vào bộ nhớ đặc ruột, gọi `rayon` để kích hoạt đa luồng bạo lực vắt kiệt 100% các nhân CPU. Giải xong trong 10 giây, nó đóng gói kết quả thành `BlueprintResponse` và ném trả lại qua gRPC.

