# **Báo cáo Khoa học: Thuật toán Quy hoạch Tiền Triển khai trên Cụm Kubernetes Không Đồng Nhất – Ứng dụng Mô hình Tối ưu hóa Xếp dỡ Hàng hải**

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

## **Giới thiệu và Bối cảnh Đặt vấn đề**

Sự tiến hóa của hạ tầng điện toán đám mây nguyên bản (cloud-native) đã định vị Kubernetes như một tiêu chuẩn thực tế cho việc điều phối các hệ thống phân tán. Tuy nhiên, khi các cụm máy chủ (cluster) ngày càng trở nên không đồng nhất (heterogeneous) — tích hợp phần cứng chuyên dụng như thiết bị xử lý đồ họa (GPU), thiết bị xử lý tensor (TPU), và các node tối ưu hóa bộ nhớ — các cơ chế lập lịch mặc định bắt đầu bộc lộ những hạn chế sâu sắc. Bộ lập lịch mặc định kube-scheduler được thiết kế để tối ưu hóa độ trễ, đưa ra các quyết định xếp đặt động (dynamic runtime scheduling) tính bằng mili-giây dựa trên nguyên tắc "đến trước phục vụ trước" (first-come, first-served) khi phát hiện các khoảng trống tài nguyên.1 Mặc dù phương pháp tiếp cận này là đủ cho các vi dịch vụ (microservices) đồng nhất và phi trạng thái (stateless), nó hoàn toàn thất bại dưới độ phức tạp hình học của các cấu trúc phần cứng đa dạng và các khối lượng công việc trí tuệ nhân tạo (AI) tiên tiến. Hậu quả là sự phân mảnh tài nguyên nghiêm trọng, với các phân tích ngành liên tục cho thấy môi trường đám mây thường chỉ hoạt động ở mức 30% đến 40% công suất sử dụng đơn vị xử lý trung tâm (CPU).1  
Để giải quyết những hạn chế của việc lập lịch động trong thời gian thực, một sự dịch chuyển hệ tư tưởng sang quy hoạch tĩnh (static planning) ngoại tuyến (offline) là điều kiện tiên quyết. Sự dịch chuyển này rút ra một phả hệ kiến trúc trực tiếp từ lĩnh vực hậu cần hàng hải, cụ thể là Bài toán Quy hoạch Kế hoạch Xếp dỡ Container (Container Stowage Planning Problem \- CSPP) được sử dụng bởi các siêu tàu viễn dương.1 Các siêu máy tính hiện đại tối ưu hóa vị trí của các container vật lý trên các siêu tàu từ rất lâu trước khi tàu cập cảng, đánh giá hàng triệu kịch bản không gian.1 Bằng cách chuyển đổi việc lập lịch Kubernetes từ một quá trình động, vô hình sang một công cụ giao diện dòng lệnh (CLI engine) tiền triển khai (pre-deployment) dựa trên nền tảng toán học, các tổ chức có thể giải quyết bài toán Đóng gói Thùng Đa chiều (Multi-Dimensional Bin Packing \- MDBP) ngoại tuyến.1 Phương pháp này tạo ra các bản thiết kế (blueprint) xếp đặt mang tính khai báo, tách biệt hoàn toàn toán học tối ưu hóa khỏi kube-apiserver đang hoạt động, qua đó loại bỏ sự phân mảnh tài nguyên mà không can thiệp vào trạng thái của cụm máy chủ.1

## **Giải phẫu Bài toán Quy hoạch Xếp dỡ Container Hàng hải (CSPP)**

Bài toán CSPP trong hàng hải là một thách thức tối ưu hóa tổ hợp NP-hard vô cùng phức tạp, trực tiếp quyết định hiệu quả hoạt động và sự an toàn của chuỗi cung ứng toàn cầu.2 Các tàu container vận chuyển hàng hóa tiêu chuẩn hóa giữa các cảng trên các tuyến đường vòng xích cố định. Một kế hoạch xếp dỡ chỉ định các container từ một bến bãi (terminal) vào các khe (slots) cụ thể trên tàu, tính toán dựa trên hình học vật lý của tàu, kích thước container và các giới hạn ổn định nghiêm ngặt.5 Mức độ phức tạp của bài toán tăng lên theo cấp số nhân khi dung lượng của tàu ngày càng lớn.

### **Quy mô của các Siêu Tàu Viễn Dương**

Các siêu tàu viễn dương hiện đại, chẳng hạn như lớp tàu MSC Irina, sở hữu sức chở vượt quá 24.300 đơn vị tương đương hai mươi feet (TEU).7 Tầm vóc của những cỗ máy này đòi hỏi các thuật toán lập kế hoạch phi thường. Cụ thể, một tàu lớp MSC Irina khi được xếp đầy tải có thể chứa 24.346 TEU, và nếu đặt các container này nối đuôi nhau, chúng sẽ trải dài 147,5 kilomet, tương đương thể tích của 322 hồ bơi tiêu chuẩn Olympic hoặc trọng lượng của 52 tháp Eiffel.8 Nếu không có một đơn vị đo lường tiêu chuẩn và một nền tảng toán học mạnh mẽ, việc hợp nhất các biến số này vào một kế hoạch xếp dỡ duy nhất sẽ là điều không tưởng.10

### **Động lực học Vật lý: Độ ổn định, Lực cắt và Momen uốn**

Kế hoạch xếp dỡ tàu bị chi phối chủ yếu bởi hai mục tiêu thường xuyên xung đột: đảm bảo tính ổn định của tàu và giảm thiểu số lượng các lần di dời container không cần thiết (relocations/restows).3 Tính ổn định của tàu được định nghĩa bởi các nguyên tắc thủy tĩnh học, đặc biệt là chiều cao khuynh tâm ban đầu (initial metacentric height \- GM), được tính toán sử dụng góc nghiêng (heel), mớn nước (draft) và độ chúi (trim) của tàu.3 Tổng trọng lượng và lực đẩy nổi (buoyancy) phải được phân bổ chính xác để ngăn chặn các momen uốn ngang (transverse bending moments/torsion) và lực cắt dọc (shear forces) gây nguy hiểm cho kết cấu.6 Nếu các container nặng được đặt quá cao, tàu sẽ mất ổn định; nếu trọng lượng tập trung quá nhiều ở mũi hoặc lái, kết cấu tàu sẽ chịu ứng suất nghiêm trọng.  
Để giải quyết sự mất cân bằng tải trọng không thể tránh khỏi, các kỹ sư sử dụng các két nước dằn (ballast tanks) phân bố dọc theo thân tàu. Nước được bơm vào hoặc xả ra để sửa đổi độ dịch chuyển (displacement) và trọng tâm dọc (longitudinal center of gravity \- LCG), qua đó thay đổi khuynh tâm, mớn nước và sức nổi của từng phần trên tàu.6 Đặc tính này thiết lập một trạng thái nền tảng cơ bản (baseline weight) trước khi bất kỳ tải trọng nào được thêm vào, một nguyên lý được ánh xạ trực tiếp sang điện toán đám mây.

### **Phân loại Hàng hóa và Quy định Cách ly**

Ngoài sự ổn định về mặt cấu trúc, hệ thống xếp dỡ (stowage engine) phải tính toán vị trí dựa trên các phân loại container đặc thù. Các container tiêu chuẩn phải chia sẻ không gian với container bệ phẳng (flat-rack containers \- loại 1B theo tiêu chuẩn ISO) được sử dụng cho thiết bị quá khổ hoặc phương tiện không có thành bên, và container bồn (tank containers \- loại 1T theo ISO) được sử dụng cho chất lỏng và khí có khả năng chịu áp suất.15 Hơn nữa, container lạnh (reefers) yêu cầu các khe cắm cụ thể được trang bị kết nối điện lưới, trong khi các vật liệu nguy hiểm (hazmat) phải được cách ly vật lý theo các quy định an toàn phân tách vô cùng nghiêm ngặt.1

### **Phân rã Phân cấp: Kế hoạch Master Bay và Kế hoạch Slot**

Vì tính chất không thể giải quyết bằng các phương pháp tính toán vét cạn thông thường (computationally intractable) khi tối ưu hóa đồng thời 24.000 đơn vị riêng biệt, các nhà nghiên cứu vận hành hàng hải thường phân rã bài toán CSPP thành hai giai đoạn phân cấp.3 Giai đoạn đầu tiên là Bài toán Kế hoạch Khoang Chính (Master Bay Plan Problem), phân phối các nhóm container ở cấp độ vĩ mô vào các phần cắt dọc (bays) cụ thể của tàu.12 Giai đoạn thứ hai là Bài toán Kế hoạch Khe (Slot Plan Problem), gán từng container riêng lẻ vào tọa độ lưới (grid coordinates) chính xác trong các khoang đó.12 Sự phân rã này mang tính biểu tượng sâu sắc trong kỹ thuật phần mềm, phản ánh chính xác cách các khối lượng công việc được gán cho các nhóm node (Node Pools) trước khi được lên lịch chi tiết vào các socket CPU cụ thể trên một node.

## **Chuyển đổi Cấu trúc: Từ Siêu Tàu Viễn Dương sang Tô-pô Kubernetes**

Sự liên kết cấu trúc giữa quy hoạch xếp dỡ hàng hải trên siêu tàu như MSC Irina và quy hoạch điện toán AI trên Kubernetes là sự đồng cấu (isomorphic) về mặt chức năng. Cả hai môi trường đều đại diện cho các hạ tầng có ranh giới vật lý cứng nhắc, trong đó hàng hóa đa chiều phải được đóng gói khít vào nhau trong khi thỏa mãn một tập hợp nghiêm ngặt các ràng buộc cứng (bắt buộc) và ràng buộc mềm (ưu tiên).1 Để xây dựng một CLI engine tiền triển khai cho Kubernetes, các kỹ thuật heuristic toán học của hậu cần hàng hải phải được ánh xạ chính xác sang các khái niệm trừu tượng nguyên bản của đám mây.  
Bảng dưới đây minh họa quá trình ánh xạ mang tính khái niệm và thuật toán theo tỷ lệ 1:1, cấu trúc nên nền tảng của bộ lập lịch tiền triển khai tĩnh:

| Bối cảnh Xếp dỡ Hàng hải (Vận tải Biển) | Bối cảnh Điều phối Kubernetes (Điện toán Đám mây) | Kỹ thuật Toán học / Thuật toán Tương ứng |
| :---- | :---- | :---- |
| **Kích thước Container** (20ft, 40ft, High-Cube) | **Yêu cầu & Giới hạn Tài nguyên** (CPU, RAM, GPU, VRAM) | Bài toán Đóng gói Thùng Đa chiều (MDBP) |
| **Container Lạnh (Reefer)** (Yêu cầu khe cắm điện) | **Khối lượng Điện toán AI chuyên dụng** (GPU Nvidia A100, T4) | Ràng buộc Cứng CSP (NodeSelector, NodeAffinity) |
| **Cách ly Hóa chất Nguy hiểm (Hazmat)** | **Chống Tương tác Pod (Pod Anti-Affinity) / Taints & Tolerations** | Đồ thị Xung đột (Conflict Graph) & Tô màu Đồ thị |
| **Gộp nhóm Cảng Đích (Vào sau Ra trước \- LIFO)** | **Tương tác Pod (Pod Affinity) / Tô-pô Mạng** (Đồng vị trí dịch vụ) | Hàm Độ thích nghi (Fitness Function) cho Ràng buộc Mềm |
| **Độ chúi & Cân bằng Tàu (Trọng lượng / Chiều cao GM)** | **Cân bằng Mức độ Sử dụng Tài nguyên** (Tải node đồng đều) | Cực tiểu hóa Phương sai (Variance Minimization) |
| **Đặt chỗ Khối (Block Booking / Slot Charter)** | **Lập lịch Nhóm (Gang Scheduling)** (Huấn luyện AI Phân tán) | Biến Ghép cặp trong CSP, Kiểm tra Chuyển tiếp (Forward Checking) |
| **Cáp chằng buộc & Cố định (An toàn bão tố)** | **Lớp Chất lượng Dịch vụ (QoS)** (Đảm bảo vs. Có thể Mở rộng) | Bài toán Cái túi (Knapsack) với Ranh giới Dung lượng Nghiêm ngặt |
| **Nắp hầm hàng (Phân tách Dưới hầm / Trên boong)** | **Ràng buộc Phân tán Tô-pô** (Vùng Sẵn sàng / Availability Zones) | Ràng buộc Phân phối (Dung lượng Tối thiểu/Tối đa mỗi Vùng) |
| **Nước Dằn (Ballast Water)** | **DaemonSets** (Pod hệ thống cốt lõi, CNI, CSI agents) | Biến Cố định trong Quy hoạch Tuyến tính Nguyên (ILP) |
| **Tái xếp dỡ tốn kém (Restows)** | **Thu hồi & Trục xuất Pod (Pod Preemption & Eviction)** | Hàm Phạt nặng (Heavy Penalty Functions) trong Đánh giá Fitness |
| **Kế hoạch BAPLIE / Sơ đồ Tàu Cuối cùng** | **Bản thiết kế Tiền Triển khai (Pre-deployment Blueprint)** | Ma trận Trạng thái Cuối cùng (Kết quả của Thuật toán Di truyền) |

Việc xử lý tô-pô mạng và vật lý của cụm Kubernetes như thân tàu và các tệp manifest như hàng hóa đã biến đổi bài toán lập lịch từ một vấn đề quản lý hàng đợi phản ứng (reactive queue management) thành một câu đố hình học và không gian mang tính tất định.

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

## **Nghịch lý của Lập lịch Động và Hiệu ứng "Kênh Tài nguyên" Autopilot**

Lỗ hổng cốt lõi của việc phụ thuộc hoàn toàn vào hệ thống lập lịch động của Kubernetes xuất phát từ những giới hạn về mặt thời gian của nó. Kube-scheduler đánh giá các pod một cách riêng lẻ khi chúng xuất hiện trong hàng đợi. Khi phát hiện một khe trống trên một node bất kỳ, nó sẽ liên kết pod ngay lập tức để tiết kiệm độ trễ.1 Dù thuật toán gán khớp đầu tiên (first-fit heuristic) này vô cùng đơn giản về mặt toán học, nó chắc chắn dẫn đến sự phân mảnh không gian theo thời gian. Các khối lượng công việc có mật độ cao được đưa vào hàng đợi muộn hơn thường xuyên thấy tài nguyên tổng hợp của cụm đã bị vỡ vụn trên nhiều node được lấp đầy một phần, khiến các khối lượng công việc đó không thể được lập lịch (un-schedulable) mặc dù dung lượng tổng cộng của cụm là hoàn toàn đủ.1  
Sự phân mảnh này tạo ra sự phụ thuộc vào các công cụ mở rộng cụm (cluster autoscaler) để liên tục tạo thêm hạ tầng mới, làm phình to chi phí đám mây một cách vô ích. Ngược lại, việc ứng dụng các siêu máy tính chạy mô phỏng quy hoạch xếp dỡ hàng hải đánh giá hàng triệu kịch bản tổng thể ngoại tuyến, cho phép tính toán cấu trúc bố trí tối ưu trước khi bất kỳ container nào thực sự được cần cẩu nhấc lên.1 Do đó, việc nghiên cứu về các trình biên dịch thiết kế tĩnh mang lại tiềm năng triệt tiêu sự lãng phí này.

### **Mô hình Tiêu thụ Autopilot của Google**

Các dịch vụ nền tảng điều khiển được quản lý toàn diện (managed control plane), chẳng hạn như Google Kubernetes Engine (GKE) Autopilot, cố gắng trừu tượng hóa sự phân mảnh này bằng cách loại bỏ hoàn toàn khái niệm "node" khỏi tầm nhìn vận hành của người dùng.23 Autopilot cung cấp hạ tầng động và chỉ tính phí dựa trên yêu cầu tài nguyên của từng pod (vCPU và bộ nhớ) thay vì tính phí 24/7 cho toàn bộ máy ảo Compute Engine.23 Hệ thống này sử dụng dữ liệu lịch sử, các thuật toán như Cửa sổ Trượt làm mịn theo hàm mũ (exponentially-smoothed Sliding Window) và Học Tăng cường (Reinforcement Learning) qua công cụ Vertical Pod Autoscaler (VPA) để tự động điều chỉnh quy mô tài nguyên theo chiều dọc và giải quyết các lỗi thiếu bộ nhớ (Out of Memory \- OOM).24  
Mặc dù Autopilot hoạt động xuất sắc đối với các môi trường có khối lượng công việc biến động mạnh (bursty), khối lượng công việc theo đợt trung bình, hoặc cho các nhóm phát triển không chuyên về quản trị hạ tầng 23, nó lại ngầm tạo ra một ảo giác về nguồn tài nguyên vô hạn. Thuật toán này mặc định rằng nhà cung cấp đám mây luôn có sẵn các máy chủ vật lý chờ sẵn.1 Tuy nhiên, trong các cụm trí tuệ nhân tạo chuyên sâu yêu cầu phần cứng giới hạn nghiêm ngặt — chẳng hạn như các nút chứa GPU Nvidia L40S, A100 hay T4 — ranh giới vật lý là tuyệt đối và không thể bị phần mềm điều khiển (control plane) trừu tượng hóa hay "sản sinh" ra tức thời khi thiếu hụt.22 Việc không thể kiểm soát node đồng nghĩa với việc các nhóm kỹ sư không thể đảm bảo sự hiện diện liên tục, sát sao của các pod AI gần khu vực bộ nhớ băng thông cao.

### **Hiệu ứng "Kênh Tài nguyên" (The Resource Canal Effect)**

Một kiến trúc tối ưu không tìm cách thay thế các công cụ mở rộng động như Autopilot mà bổ sung cho chúng thông qua một khuôn khổ được định nghĩa là hiệu ứng "Kênh Tài nguyên".1 Cụ thể, một công cụ quy hoạch tĩnh tiền triển khai sẽ thiết lập các ranh giới vật lý cứng (đóng vai trò như các "bờ kênh" cố định) sử dụng thuật toán đóng gói thùng đa chiều, chỉ định chính xác nơi các nguyên mẫu (archetypes) khối lượng công việc cụ thể cư trú.1 Các công cụ mở rộng động (dynamic scalers) sau đó sẽ hoạt động bên trong các ranh giới này, điều chỉnh mức tiêu thụ tài nguyên dựa trên lưu lượng thời gian thực (đóng vai trò như "mực nước").  
Sự cộng sinh này tăng tốc tốc độ hội tụ (convergence rate) của các thuật toán học tăng cường động lên đến hàng chục lần.1 Khi các hệ thống động không bị ép buộc phải liên tục kích hoạt quá trình khởi tạo node cơ sở — vì toàn bộ cấu trúc tô-pô ban đầu đã được đóng gói hoàn hảo — chúng sẽ tính toán các điều chỉnh theo chiều dọc chính xác hơn nhiều với độ trễ thấp hơn đáng kể.1

## **Thiết kế Kiến trúc của Động cơ CLI Blueprint Ngoại tuyến**

Để triển khai các thuật toán heuristic hàng hải vào môi trường đám mây, cơ chế lập lịch cần được phẫu thuật bóc tách khỏi cụm đang hoạt động. Thiết kế hệ thống yêu cầu một công cụ dòng lệnh (CLI tool) đóng vai trò kiến trúc sư thuật toán độc lập — một động cơ mô phỏng tạo ra các bản thiết kế lập lịch tiền triển khai tĩnh.1

### **Sự Không can thiệp Hoàn toàn vào Kubernetes (Zero-Touch Interference)**

Đặc tính cốt lõi xác định động cơ này là sự tách biệt tuyệt đối khỏi kube-apiserver. CLI hoạt động hoàn toàn ngoại tuyến, thường được tích hợp ngay bên trong một đường ống tích hợp và phân phối liên tục (CI/CD) hoặc được chạy nội bộ trên máy trạm của kỹ sư.1 Vì nó không chạy như một trình điều khiển (controller) hay tiến trình ngầm (daemon) bên trong cụm đang hoạt động, nó triệt tiêu mọi rủi ro tiêu tốn tài nguyên quý giá của cụm điều khiển, gây ngẽn cổ chai API (API throttling) hoặc kích hoạt các vòng lặp cạnh tranh vô hạn (race conditions) trong quá trình lập lịch.1

### **Dữ liệu Đầu vào và Cấu trúc Cấu hình Khai báo**
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

## **Đường ống Tối ưu hóa Lai Ba Giai đoạn (Three-Phase Hybrid Pipeline)**

### **Giai đoạn 0: Khấu trừ Khởi tạo DaemonSet (Sự Tương đồng của Nước Dằn Tàu)**

Trước khi quá trình tối ưu hóa bắt đầu, thuật toán phải thanh toán mọi chi phí tiêu hao (overhead) của hệ thống. Trong quy hoạch xếp dỡ hàng hải, các két nước dằn (ballast tanks) được làm đầy hoặc bơm cạn để thiết lập mớn nước tĩnh và trọng tâm dọc cơ sở trước khi quá trình chuyển hàng hóa lên tàu bắt đầu.6 Tương tự, trên các node Kubernetes, các tiến trình nền tảng bắt buộc (như plugin mạng CNI, các công cụ thu thập log, hoặc hệ thống lưu trữ daemon) luôn vận hành ngầm thông qua cấu trúc DaemonSet.1  
Động cơ tiến hành việc khấu trừ trước (pre-deduction) ở Giai đoạn 0 (Phase 0), sửa đổi trực tiếp dung lượng thô ban đầu ![][image12] của node ![][image13] trên toàn bộ các chiều không gian tài nguyên ![][image14]:  
![][image15]  
Hàm chỉ báo ![][image16] kích hoạt quá trình khấu trừ toán học. Quá trình này cung cấp sự đảm bảo tuyệt đối rằng không gian tối ưu hóa chỉ đại diện cho lượng tài nguyên thuần có khả năng cấp phát, tránh hiện tượng lỗi tràn viền ảo ảnh vào cuối tiến trình thuật toán.1

### **Hàm Mục tiêu (Objective Function)**

Khác biệt với bài toán đóng gói thùng thuần túy, vốn chỉ mang duy nhất một mục tiêu là cực tiểu hóa lượng thùng được sử dụng, mô hình kế thừa từ kỹ thuật xếp dỡ hàng hải này đánh giá độ thích nghi tổng thể dựa trên đa mục tiêu cạnh tranh bằng công thức tổng có trọng số (weighted sum formula).1 Hàm mục tiêu toàn cục (global objective function) được thiết lập như sau:  
![][image17]  
Các thành phần riêng lẻ của hàm tuyến tính này phản ánh tương quan trực tiếp 1:1 với các ưu tiên trong vận hành sản xuất:

* ![][image18]: Cực tiểu hóa tổng lượng node được kích hoạt. Chỉ số này đóng vai trò tương tự như việc giảm số lượng các tàu chuyên chở phải thuê mướn vật lý, trực tiếp cắt giảm chi phí tính toán đám mây.1  
* ![][image19]: Định lượng và trừng phạt dung lượng rỗng, không thể sử dụng (phân mảnh) trên các node đang hoạt động, tạo điều kiện thuận lợi cho sự bố trí khít khao.1  
* ![][image20]: Tính toán vi phạm ràng buộc mềm. Nếu hai pod mong muốn định vị sát nhau để giảm độ trễ mạng (tương tự như việc xếp các container hướng tới cùng một cảng đích LIFO cùng nhau), việc phân tách chúng phải chịu một hình phạt toán học.1  
* ![][image21]: Tính toán phương sai mức độ sử dụng tài nguyên giữa tất cả các node đang hoạt động. Đây chính là biểu tượng toán học trực tiếp tương đương với việc xác định chiều cao khuynh tâm (GM) và mức cân bằng để ngăn chìm tàu vật lý.1 Việc san lấp tải trọng đồng đều trên các node hoạt động ngăn chặn các rủi ro thắt cổ chai nhiệt năng và băng thông.  
* ![][image22]: Hàm trừng phạt tuyệt đối. Bất cứ khi nào xuất hiện vi phạm ràng buộc cứng — chẳng hạn như một khối lượng công việc đòi hỏi GPU A100 lại bị đặt vào một bộ xử lý CPU đa dụng (general-purpose) — hàm tuyến tính sẽ trả về giá trị ![][image23], loại bỏ lập tức cấu hình di truyền đó.1

## **Đường ống Tối ưu hóa Lai Ba Giai đoạn (Three-Phase Hybrid Pipeline)**

Việc giải phương trình phi tuyến ![][image24] đòi hỏi phải băng qua các cảnh quan cấu trúc tổ hợp khổng lồ. Thuật toán di truyền (Genetic Algorithm \- GA) cơ bản thường xuyên thất bại trong việc nhanh chóng tìm kiếm các giải pháp khả thi nếu được khởi tạo với sự bố trí hoàn toàn ngẫu nhiên. Do tính chất dày đặc của các ràng buộc cứng, đa số quần thể ban đầu mang giá trị vô hiệu.1 Để lách qua rào cản này, động cơ áp dụng đường ống tối ưu hóa lai ba giai đoạn, hòa trộn các giải thuật tham lam (greedy heuristics) cùng với sinh học tiến hóa.1

### **Giai đoạn 1: Vector Packing First-Fit Decreasing (FFD) Warm-Start**

Nhằm khởi tạo một nền tảng vững chắc, giải thuật sinh ra một quần thể hạt giống khả thi bằng kỹ thuật sửa đổi heuristic Gán khớp Đầu tiên Giảm dần (First-Fit Decreasing).1 Do pod sở hữu thuộc tính đa chiều (RAM không thể bù cho CPU), việc đơn thuần sắp xếp chúng theo "kích thước" là một bài toán phức tạp. Cỗ máy CLI tổng hợp một thước đo thể tích thống nhất ![][image25] cho mỗi pod thông qua một công thức véc-tơ tỷ lệ (scalar):  
![][image26]  
Các Pod được sắp xếp theo trình tự giảm dần dựa trên ![][image25] và tham lam đẩy vào trong không gian đa chiều thích hợp nhất trên node đầu tiên. Cơ chế khởi động ấm (warm-start) này đảm bảo rằng các bước tiến hóa sau đó khởi nguồn từ các nhiễm sắc thể (chromosomes) gốc có giá trị hợp lệ về mặt vật lý, từ đó đẩy tốc độ hội tụ toán học nhanh hơn khoảng 3 đến 5 lần so với ngẫu nhiên.1

### **Giai đoạn 2: Quá trình Tiến hóa Bằng Thuật toán Di truyền (GA Execution)**

Sau khi thiết lập quần thể nền tảng, Thuật toán Di truyền chiếm quyền kiểm soát toàn bộ. Thuật toán lợi dụng mô hình tính toán song song (parallelism qua Goroutines) nhằm bứt tốc xử lý hàng ngàn bản định giá độ thích nghi (fitness evaluations) mỗi giây với vòng đời sinh học.1

* **Lựa chọn (Selection):** Động cơ vận dụng Chọn lọc Giải đấu (Tournament Selection với ![][image27]), nơi một nhóm nhỏ cấu trúc blueprint ngẫu nhiên được trích xuất, và các mã gen khỏe mạnh nhất được đưa vào buồng sinh sản.1  
* **Cơ chế Lai ghép (Crossover Mechanism):** Một phép toán lai chéo đồng dạng (Uniform Crossover) dung hợp các tính trạng từ cặp phụ huynh blueprint. Tuy nhiên, vì các pod thường xuyên đan xen không gian lẫn nhau, phương pháp lai ghép nguyên thủy có thể vô tình sinh ra mã gen vi phạm khả năng chuyên chở tĩnh. Để xử lý rủi ro này, động cơ cài đặt "Cơ chế Sửa chữa Cụm/Nhóm" (Gang Repair mechanism) nhằm quay ngược (rollback) các nhiễm sắc thể cấu trúc bị đứt gãy về tình trạng an toàn thừa hưởng từ một nhiễm sắc thể phụ huynh hợp lệ.1  
* **Đột biến (Mutation):** Để thoát khỏi bẫy tối ưu cục bộ (local optima), các đột biến thiết lập lại ngẫu nhiên (random reset mutations) sẽ chỉnh sửa một tỷ lệ siêu nhỏ các vị trí thiết lập (![][image28]). Điểm then chốt ở đây là mọi hành vi đột biến đều chịu sự thẩm định (pre-screened) thông qua bộ giải Thỏa mãn Ràng buộc CSP. Các đột biến tự định tuyến vào sai vùng taints hoặc node bị loại bỏ thẳng tay, bảo vệ sức mạnh xử lý không bị hoang phí trên các nhánh tính toán ngõ cụt.1  
* **Điểm Cắt (Termination):** Trình diễn GA lặp lại theo quy tắc đệ quy cho đến khi thỏa mãn chuẩn mực dừng sớm — thông thường là ngưỡng ![][image29] thế hệ liên tiếp khi điểm ![][image24] xuất sắc nhất hoàn toàn chững lại, không tiến triển.1

### **Giai đoạn 3: Kiểm tra Chuyển tiếp (CSP Forward Checking)**

Được chèn đúc sâu bên trong lòng các vòng lặp FFD và GA là hệ thống giải quyết Ràng buộc rời rạc thực thi tính năng Kiểm tra Chuyển tiếp (Forward Checking).1 Thuật toán điện toán cổ điển thực hiện việc nhét khối lượng vào rồi mới phản ứng phân tích lỗi. Ngược lại, kỹ thuật Kiểm tra Chuyển tiếp tính toán trước phần dung lượng toàn cục trải dài các chiều không gian *trước* khi phê chuẩn bất cứ tác vụ ràng buộc nào. Nếu đưa Pod A vào Node 1 làm bay biến sạch khả năng đáp ứng điều kiện về sau cho Pod B, nhánh dự án đó tức khắc bị triệt tiêu từ trong trứng nước. Kiến trúc tầm nhìn xa này dập tắt nguy cơ rơi vào bế tắc (deadlock) trong các môi trường điện toán đám mây cấp cao.1

## **Giải quyết Bài toán Gang Scheduling AI thông qua Mô hình Đặt chỗ Khối (Block Booking)**

Một trong những bài toán phức tạp và hóc búa nhất của lĩnh vực điều phối điện toán hiện đại là quản trị cấu trúc cho việc huấn luyện máy học (ML) và trí tuệ nhân tạo (AI) phân tán.22 Hệ khối lượng công việc này hoàn toàn dựa vào nền tảng phần cứng gia tốc vật lý chuyên sâu và phải tuân theo một điều kiện vận hành khốc liệt mang tên Lập lịch Nhóm (Gang Scheduling).36

### **Vấn nạn "Tất cả hoặc Không gì cả" (All-or-Nothing)**

Lập lịch nhóm cưỡng chế một vòng đời mang nguyên tắc "Tất cả hoặc Không gì cả" (All-or-Nothing).36 Các mô hình ngôn ngữ lớn (LLMs) được huấn luyện qua phương thức song song dữ liệu (data parallelism) đòi hỏi sự góp mặt đồng thời của hàng chục pod rải rác. Giả sử một phiên làm việc (Job) khát 64 GPU cùng lúc, nhưng tại khoảnh khắc đó, cụm Kubernetes chỉ có thể gom góp được 60 GPU. Nếu hệ thống điền trước 60 GPU này, hậu quả sẽ thảm khốc hơn cả việc không làm gì cả. Tiến trình học sâu không thể nổ máy khi thiếu vắng 4 công nhân cuối cùng, khiến 60 GPU nghìn đô đắt đỏ kia bị treo ngâm hoàn toàn vô ích vô định kỳ.22 Trong cụm Kubernetes truyền thống dựa trên lập lịch động, tập quán vận hành này thường xuyên đẻ ra các bế tắc ngõ cụt khôn lường.22  
Nhiều dự án như trình lập lịch Volcano hay hệ sinh thái Kueue (sử dụng các tham số WorkloadPriorityClass, ClusterQueue) đã được thiết kế như những lớp áo khoác bên trên Kubernetes, được sinh ra để kìm nén những job này trong hàng chờ (queue) đến khi nào hệ sinh thái gom đủ tài nguyên.22 Mặc dù Kueue thừa nhận nghiêm ngặt cơ chế "Tất cả hoặc Không gì cả" và hỗ trợ thu hồi quyền ưu tiên, sự phân mảnh phần cứng rời rạc vẫn luôn là bức tường chắn cốt lõi.22 Giả sử một kiến trúc NVIDIA DGX Cloud gồm ngàn bộ L40S GPU xảy ra hiện tượng phân mảnh, các dự án huấn luyện đa nút sẽ tê liệt vĩnh viễn.22 Tuy về toán học, tổng GPU còn trống trên cụm đủ thỏa mãn yêu cầu, nhưng về vật lý chúng bị bắn văng rải rác (fragmented) tới mức mạng kết nối lõi băng thông cực lớn như NVLink không thể kết nối.22

### **Lời giải từ Hàng hải: Mô hình Block Booking và Gang Repair**

Thế giới quản trị kho bãi hàng hải đã khuất phục động lực học bế tắc này thông qua hai quan điểm cấu trúc: Đặt chỗ Khối (Block Booking) và Thuê trọn Khe (Slot Chartering). Hãng vận tải biển không xé lẻ hàng hóa khi cung cấp khoang cho đối tác hậu cần lớn.1 Nếu một nhà thầu đóng trọn kiện hàng 500-TEU kết nối mật thiết làm một lô, thuật toán xếp dỡ trên tàu Irina không bao giờ băm nhỏ chúng rải rác dọc thân tàu; chúng bị khóa cứng trong một cấu trúc hình học vĩ mô và không thể chia tách.  
Động cơ CLI blueprint ngoại tuyến hóa giải thành công các ngõ cụt trong lập lịch nhóm của Kubernetes bằng cách mô hình hóa công khai trực tiếp các khối công việc ![][image4] thành biến số ghép cặp (coupled variables) bên trong vòng lặp hệ thống CSP.1 Quá trình kiểm tra chuyển tiếp (forward-checking) sẽ gom chặt 64 chiếc GPU đó thành một khối bằng thuật toán. Và trong giai đoạn Giải thuật Di truyền (GA Phase), nếu phép lai ghép (crossover) vô ý bẻ gãy khối lượng đó văng qua các vách ngăn node không hợp lệ, cơ chế phòng thủ mang tên "Sửa chữa Nhóm" (Gang Repair) sẽ đảo ngược mã gen về trạng thái sạch sẽ.1  
Bởi vì toàn bộ quá trình đóng gói này thực hiện trước khi quá trình triển khai diễn ra ở không gian mây, nó cung cấp bảo chứng tuyệt đối rằng một khi chuỗi manifest cấu hình (blueprint) được triển khai, đúng chính xác 64 pod kia sẽ lập tức lấp đầy khít vào 64 khe không gian (slots) GPU hoàn hảo. Mục tiêu hướng tới tỷ lệ thành công 100% trong lập lịch điện toán AI này chặn đứng hoàn toàn hiện trạng phần cứng đắt tiền nằm chờ chết do sự mù lòa của cơ chế lập lịch theo thời gian thực.1

## **Posture Bảo mật và Kiểm toán Hạ tầng Tiền Triển khai (Pre-deployment Auditing)**

Bước chuyển đổi sang kiến trúc hoạch định tiền triển khai (offline planning) không chỉ tác động sâu sắc đến hiệu năng mà còn mang lại lợi ích hạ nguồn khổng lồ cho hệ sinh thái bảo mật (security posture) của các tổ chức, đặc biệt trong những hệ thống chịu sự giám sát tuân thủ nghiêm ngặt như tài chính công nghệ (FinTech) hoặc môi trường không gian mạng quân sự.35

### **Phân tích Nguy cơ Động (Dynamic Risk Analysis)**

Trong các môi trường dựa vào hệ thống lập lịch tự động, tốc độ khởi tạo và phá hủy thực thể (chẳng hạn như hàng trăm node hay 5.000 pod quay vòng mỗi phút) sinh ra một bề mặt tấn công cực lớn.35 Báo cáo Điều tra Xâm phạm Dữ liệu từ Verizon năm 2024 (2024 Verizon Data Breach Investigation Report) ghi nhận tới 15% tổn thất bảo mật có sự góp mặt từ các lỗ hổng hệ thống liên đới qua chuỗi cung ứng phần mềm và cấu hình sai lệch trong hệ thống điều phối.35  
Kubernetes yêu cầu cấu hình các hệ thống quản lý rủi ro khắt khe thông qua RBAC (Quyền truy cập dựa trên vai trò), sổ ghi chép hành động (audit logging) và các Admission Controllers (Bộ điều khiển đầu vào) tại runtime.40 Khi vị trí xếp dỡ được ủy thác hoàn toàn cho lập lịch động (dynamic scheduling), việc thẩm định sự an toàn — chẳng hạn như yêu cầu cách ly một pod xử lý thông tin nhạy cảm định danh (PII) không được phép tồn tại cùng node với ứng dụng web đối mặt với public internet — đặt toàn bộ gánh nặng lên Admission Controller.40 Nếu lớp điều khiển này lỗi nhịp, sụp đổ hay bị vượt mặt, phần mềm độc hại vẫn được đẩy ra hạ tầng thực tế.26

### **Tường minh và Định lượng Tiền Triển khai (Deterministic Validation)**

Ngược lại, thông qua kiến trúc mô phỏng bằng công cụ độc lập tĩnh, bản đồ tô-pô định vị cuối cùng (final cluster layout) trở nên mang tính tất định (deterministic) ở thời điểm sớm nhất.20 Khi mô phỏng kết thúc, nó giao nộp một báo cáo khai báo (declarative blueprint) vĩ mô bao trùm toàn cảnh rủi ro (container-centric view of risk).40  
Nhóm kỹ sư bảo mật hoàn toàn có thể khởi động hàng loạt các cơ chế kiểm tra tính an toàn trên kho chứa mã nguồn thông qua kết nối quét quét ảnh nhân nền (base image vulnerability scanning/static analysis), thanh tra trực tiếp mã YAML artifact.35 Việc này giúp phân tích độ cách ly namespace, kiểm tra các taints/tolerances, và giới hạn mạng được áp đặt toàn cục.26 Nếu blueprint xuất hiện các lỗi liên đới logic có nguy cơ bộc lộ điểm yếu, luồng CI/CD lập tức chặn đứng sự khởi hành. Hành động này đóng vai trò chốt chặn bất khả xâm phạm vật lý; mã lỗi và nguy cơ bị nghiền nát trước khi nó có bất kỳ khả năng tiếp xúc nào lên thiết bị đang vận hành.20

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

#### **Nguồn trích dẫn**

> 2. \[2510.02589\] A Benchmark Study of Deep Reinforcement Learning Algorithms for the Container Stowage Planning Problem \- arXiv, truy cập vào tháng 7 19, 2026, [https://arxiv.org/abs/2510.02589](https://arxiv.org/abs/2510.02589)  
> 3. Many-Objective Container Stowage Optimization Based on Improved NSGA-III \- MDPI, truy cập vào tháng 7 19, 2026, [https://www.mdpi.com/2077-1312/10/4/517](https://www.mdpi.com/2077-1312/10/4/517)  
> 4. Integrating container stowage plan and yard operations \- loadmaster.ai, truy cập vào tháng 7 19, 2026, [https://loadmaster.ai/integrating-stowage-and-yard-planning-in-port-operations/](https://loadmaster.ai/integrating-stowage-and-yard-planning-in-port-operations/)  
> 5. A Multi-stage Decomposition Heuristic for the Container Stowage Problem \- University of California, Berkeley, truy cập vào tháng 7 19, 2026, [https://kaminsky.ieor.berkeley.edu/Reprints/MSOM08.pdf](https://kaminsky.ieor.berkeley.edu/Reprints/MSOM08.pdf)  
> 6. An accurate model for seaworthy container vessel stowage planning with ballast tanks, truy cập vào tháng 7 19, 2026, [https://backend.orbit.dtu.dk/ws/files/54505528/An\_accurate\_model\_for\_seaworthy\_container\_vessel\_stowage\_planning\_with\_ballast\_tanks.pdf](https://backend.orbit.dtu.dk/ws/files/54505528/An_accurate_model_for_seaworthy_container_vessel_stowage_planning_with_ballast_tanks.pdf)  
> 7. How Many Containers Fit on a Cargo Ship? (2026 Guide) \- Ship4wd, truy cập vào tháng 7 19, 2026, [https://ship4wd.com/logistics-shipping/how-many-containers-fit-on-a-cargo-ship](https://ship4wd.com/logistics-shipping/how-many-containers-fit-on-a-cargo-ship)  
> 8. What is the largest container ship in the world? \- Shipping Containers, truy cập vào tháng 7 19, 2026, [https://shipping-containers.com.au/what-is-the-largest-container-ship-in-the-world/](https://shipping-containers.com.au/what-is-the-largest-container-ship-in-the-world/)  
> 9. The Growing Role of Mega-Ships in Internation Shipping \- IoSCM, truy cập vào tháng 7 19, 2026, [https://www.ioscm.com/blog/the-growing-role-of-mega-ships-in-international-shipping/](https://www.ioscm.com/blog/the-growing-role-of-mega-ships-in-international-shipping/)  
> 10. What Is A TEU? Calculating Cargo Ship Capacity (With Examples) \- CHS Container Group, truy cập vào tháng 7 19, 2026, [https://chs-containergroup.com/us/what-is-a-teu-shipping/](https://chs-containergroup.com/us/what-is-a-teu-shipping/)  
> 11. Container Vessel Stowage Planning System Using Genetic Algorithm \- Semantic Scholar, truy cập vào tháng 7 19, 2026, [https://www.semanticscholar.org/paper/Container-Vessel-Stowage-Planning-System-Using-Weiss-Cohen-Coelho/fd9db27375afbebddbb2f19db8612c362c3d7c9a](https://www.semanticscholar.org/paper/Container-Vessel-Stowage-Planning-System-Using-Weiss-Cohen-Coelho/fd9db27375afbebddbb2f19db8612c362c3d7c9a)  
> 12. An AIMMS-based decision-making model for optimizing the intelligent stowage of export containers in a single bay, truy cập vào tháng 7 19, 2026, [https://www.aimsciences.org/article/doi/10.3934/dcdss.2019076](https://www.aimsciences.org/article/doi/10.3934/dcdss.2019076)  
> 13. Fast Generation of Container Vessel Stowage Plans \- IT University of Copenhagen, truy cập vào tháng 7 19, 2026, [https://en.itu.dk/-/media/EN/Research/PhD-Programme/PhD-defences/2012/Dario-Pacino-Thesispdf.pdf](https://en.itu.dk/-/media/EN/Research/PhD-Programme/PhD-defences/2012/Dario-Pacino-Thesispdf.pdf)  
> 14. An Accurate Model for Seaworthy Container Vessel Stowage Planning with Ballast Tanks \- Sealytix, truy cập vào tháng 7 19, 2026, [https://www.sealytix.com/media/kdemcjjx/anaccuratemodelforseaworthycontainervesselstowageplanningwithballasttanks.pdf](https://www.sealytix.com/media/kdemcjjx/anaccuratemodelforseaworthycontainervesselstowageplanningwithballasttanks.pdf)  
> 15. Stowage plan for container ships \- Grokipedia, truy cập vào tháng 7 19, 2026, [https://grokipedia.com/page/Stowage\_plan\_for\_container\_ships](https://grokipedia.com/page/Stowage_plan_for_container_ships)  
> 16. Container-Ship Stowage Planning Problem \- Encyclopedia.pub, truy cập vào tháng 7 19, 2026, [https://encyclopedia.pub/entry/22494](https://encyclopedia.pub/entry/22494)  
> 17. Fast Generation of Container Vessel Stowage Plans using mixed integer programming for optimal master planning and constraint bas, truy cập vào tháng 7 19, 2026, [https://backend.orbit.dtu.dk/ws/portalfiles/portal/54505332/thesis.pdf](https://backend.orbit.dtu.dk/ws/portalfiles/portal/54505332/thesis.pdf)  
> 18. Matheuristics for Slot Planning of Container Vessel BaysI \- Sealytix, truy cập vào tháng 7 19, 2026, [https://www.sealytix.com/media/eqepwvxj/matheuristicsforslotplanningofcontainervesselbays.pdf](https://www.sealytix.com/media/eqepwvxj/matheuristicsforslotplanningofcontainervesselbays.pdf)  
> 19. Models and solution algorithms for container terminal ... \- DR-NTU, truy cập vào tháng 7 19, 2026, [https://dr.ntu.edu.sg/bitstreams/49ce426d-33c4-47a5-b5e3-baac2b502b56/download](https://dr.ntu.edu.sg/bitstreams/49ce426d-33c4-47a5-b5e3-baac2b502b56/download)  
> 20. SAGE \- A Tool for Optimal Deployments in Kubernetes Clusters \- arXiv, truy cập vào tháng 7 19, 2026, [https://arxiv.org/pdf/2307.06318](https://arxiv.org/pdf/2307.06318)  
> 21. Google Autopilot cluster: unschedulable pods \- Stack Overflow, truy cập vào tháng 7 19, 2026, [https://stackoverflow.com/questions/67031113/google-autopilot-cluster-unschedulable-pods](https://stackoverflow.com/questions/67031113/google-autopilot-cluster-unschedulable-pods)  
> 22. Practical Tips for Preventing GPU Fragmentation for Volcano Scheduler \- NVIDIA Developer, truy cập vào tháng 7 19, 2026, [https://developer.nvidia.com/blog/practical-tips-for-preventing-gpu-fragmentation-for-volcano-scheduler/](https://developer.nvidia.com/blog/practical-tips-for-preventing-gpu-fragmentation-for-volcano-scheduler/)  
> 23. GKE Autopilot vs Standard 2026: Which Mode Should You Pick? \- Aleksei Aleinikov, truy cập vào tháng 7 19, 2026, [https://www.alekseialeinikov.com/en/blog/topics/cloud/gke-autopilot-vs-standard-2026](https://www.alekseialeinikov.com/en/blog/topics/cloud/gke-autopilot-vs-standard-2026)  
> 24. Autopilot Became the Default Operation Mode for Google Kubernetes Engine \- InfoQ, truy cập vào tháng 7 19, 2026, [https://www.infoq.com/news/2023/04/autopilot-google-kubernetes/](https://www.infoq.com/news/2023/04/autopilot-google-kubernetes/)  
> 25. Auto-scaling Approaches for Cloud-native Applications: A Survey and Taxonomy \- arXiv, truy cập vào tháng 7 19, 2026, [https://arxiv.org/html/2507.17128v1](https://arxiv.org/html/2507.17128v1)  
> 26. Kubernetes and OpenStack Orchestration for Multi-Tenant Cloud Environments: Namespace Isolation and GPU Scheduling Strategies | Request PDF \- ResearchGate, truy cập vào tháng 7 19, 2026, [https://www.researchgate.net/publication/397048156\_Kubernetes\_and\_OpenStack\_Orchestration\_for\_Multi-Tenant\_Cloud\_Environments\_Namespace\_Isolation\_and\_GPU\_Scheduling\_Strategies](https://www.researchgate.net/publication/397048156_Kubernetes_and_OpenStack_Orchestration_for_Multi-Tenant_Cloud_Environments_Namespace_Isolation_and_GPU_Scheduling_Strategies)  
> 27. Software System for Container Vessel Stowage Planning ... \- CMAP, truy cập vào tháng 7 19, 2026, [http://www.cmap.polytechnique.fr/\~nikolaus.hansen/proceedings/2015/GECCO/companion/p1519.pdf](http://www.cmap.polytechnique.fr/~nikolaus.hansen/proceedings/2015/GECCO/companion/p1519.pdf)  
> 28. Collaborative Optimization of Vessel Stowage Planning and Yard Pickup in Automated Container Terminals \- MDPI, truy cập vào tháng 7 19, 2026, [https://www.mdpi.com/2227-7390/12/21/3387](https://www.mdpi.com/2227-7390/12/21/3387)  
> 29. Literature Survey on the Container Stowage Planning Problem \- arXiv, truy cập vào tháng 7 19, 2026, [https://arxiv.org/pdf/2307.07573](https://arxiv.org/pdf/2307.07573)  
> 30. Optimising Container Stowage: Minimising Relocations in Maritime Logistics \- IE University, truy cập vào tháng 7 19, 2026, [https://www.ie.edu/university/studies/projects/optimising-container-stowage-minimising-relocations-maritime-logistics/](https://www.ie.edu/university/studies/projects/optimising-container-stowage-minimising-relocations-maritime-logistics/)  
> 31. (PDF) Solving integrated problem of stowage planning with crane split by an improved genetic algorithm based on novel encoding mode \- ResearchGate, truy cập vào tháng 7 19, 2026, [https://www.researchgate.net/publication/363317418\_Solving\_integrated\_problem\_of\_stowage\_planning\_with\_crane\_split\_by\_an\_improved\_genetic\_algorithm\_based\_on\_novel\_encoding\_mode](https://www.researchgate.net/publication/363317418_Solving_integrated_problem_of_stowage_planning_with_crane_split_by_an_improved_genetic_algorithm_based_on_novel_encoding_mode)  
> 32. University of Ibadan Genetic Algorithm Based Space-Optimised Arrangement of Containers and Stability in Containerships, truy cập vào tháng 7 19, 2026, [https://journals.ui.edu.ng/index.php/uijslictr/article/download/1136/955/3313](https://journals.ui.edu.ng/index.php/uijslictr/article/download/1136/955/3313)  
> 33. A Genetic Algorithm for Solving a Container Storage Problem Using a Residence Time Strategy, truy cập vào tháng 7 19, 2026, [https://sic.ici.ro/documents/356/SIC1-2017-Art7.pdf](https://sic.ici.ro/documents/356/SIC1-2017-Art7.pdf)  
> 34. Solving the Integrated Multi-Port Stowage Planning and Container Relocation Problems with a Genetic Algorithm and Simulation \- MDPI, truy cập vào tháng 7 19, 2026, [https://www.mdpi.com/2076-3417/12/16/8191](https://www.mdpi.com/2076-3417/12/16/8191)  
> 35. Kubernetes Best Practices for Data Teams \- DataExpert.io, truy cập vào tháng 7 19, 2026, [https://www.dataexpert.io/blog/kubernetes-best-practices-data-teams](https://www.dataexpert.io/blog/kubernetes-best-practices-data-teams)  
> 36. Plugins | Volcano, truy cập vào tháng 7 19, 2026, [https://volcano.sh/docs/v1.8.2/scheduler/plugins/](https://volcano.sh/docs/v1.8.2/scheduler/plugins/)  
> 37. Spark on Kubernetes – Gang Scheduling with YuniKorn | Blog \- Cloudera, truy cập vào tháng 7 19, 2026, [https://www.cloudera.com/blog/technical/spark-on-kubernetes-gang-scheduling-with-yunikorn.html](https://www.cloudera.com/blog/technical/spark-on-kubernetes-gang-scheduling-with-yunikorn.html)  
> 38. Scheduling Group \- Pods \- Kubernetes, truy cập vào tháng 7 19, 2026, [https://kubernetes.io/docs/concepts/workloads/pods/scheduling-group/](https://kubernetes.io/docs/concepts/workloads/pods/scheduling-group/)  
> 39. Gang scheduling, Priority scheduling, and Autoscaling for KubeRay CRDs with Kueue, truy cập vào tháng 7 19, 2026, [https://docs.ray.io/en/latest/cluster/kubernetes/k8s-ecosystem/kueue.html](https://docs.ray.io/en/latest/cluster/kubernetes/k8s-ecosystem/kueue.html)  
> 40. Container Security in 2026: 7 Key Components, Risks & Defenses \- Checkmarx, truy cập vào tháng 7 19, 2026, [https://checkmarx.com/learn/container-security/container-security-in-2026-7-key-components-risks-defenses/](https://checkmarx.com/learn/container-security/container-security-in-2026-7-key-components-risks-defenses/)

[image1]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABQAAAAZCAYAAAAxFw7TAAABK0lEQVR4Xu2TvyuGURTHD1b5kQizyfwusijKaFNkNb2jEpZ3IIlBGJTpXcTsT8BgZUFhYZSBskh8bue9Ofc8ngfTW3o/9enp3G/d5zzPPVfkvzOKx/iCcy77MyX8qBk2HEzjlCW8wiYfGA5xAQewxWUJzfgs+uZhl0Xa8FZ+2CgSWn8V3XDDZZFJ3PSLeUzhFt7gncsiVRzzi3msinawLtrlSJLqZ97Xnr/iFHvl6xSrSar/dd+t5dKH56a+EB2JTrNWwRlTFzKLK6ZeFO3SDu0Jdpm6kCMcMnU/voseUBindjwzeSHdeC3ZYT4Q7XIaJ3AtjbOEtwaWRW+IJ8xl6PIS93A8jbM84i4+YY/LIjuiXb5hq8syxAs+7wNDBz7gtg++oyz6bxo0qBefako2Jv/E1xMAAAAASUVORK5CYII=>

[image2]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABAAAAAZCAYAAAA4/K6pAAABCUlEQVR4Xu2SMUtCURTHj4IQgl9AkETcpUClNVuiUWgR+gjSZFFb4Ky4tLW0SUtLFC066QfQwS0MN0GkIcghf5fztPuuPNK2wB/84N3zP/fcy+OKbLGJ4Tm2cYTfju/4itnFBpsj7GEFizjEMR5iCS+xLzroC491m5LATzyzahNsWWtDGBuiQ17soIr3GPLWcdGm22XHD0nRrOUv+zkRbSq7AeyKZlduYHMt2lRwA6jjm+gPD6SJM1ltOsUp5pz6CgN89r6jeIA1r763aAoiJXr9O3zCD3wQPX3H6gvEvAMzYB8zGPHHv9MRveqfMA/KnH7jButyITog7Qbr0sVHt7gJedH3vuXfMQeaNTPwcymTSgAAAABJRU5ErkJggg==>

[image3]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABEAAAAaCAYAAABRqrc5AAABNUlEQVR4Xu2TLUsEURRAr/gF24UtIloMLibxBxgNgkldEATLWgwWtQhicfEHGASDoiAaTCKsXSwb1GCSRVhEBINBUBA91/tm582D2SnGOXAY3r333fcxMyI57ejBBTzHW3zDH+cz1vEUF7HTpiQZxXs8wQo+4itO4Dyu4iV+izXdtGkx/djEFTfuxk+8alXEzIo1qYWJJVzHDjceESvcaVXEDIrlogVTmREr1GOELOMdFsJEiJ5Xm5S8mO5yDq9xyIunciZ2R124Jnb+Bh5jX1yWjl7qC+6KvbEnsV1d+EVZTIpN0qcyjB8uNh4VZfEvTfbxHXu9WFWsyZEXS0XvQz91vUCfIn459RuK2MBtb/zHtNiKU2ECDsRy+juUcQ+3xBZOcOjUVxsygA9ijW5wLJnOycnmF6qaQgFaoLmeAAAAAElFTkSuQmCC>

[image4]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAA0AAAAaCAYAAABsONZfAAABEUlEQVR4Xu3SvyuGYRTG8UsYGFAGFllMsvgxMLEgjFarlGRRUiYjA6uUZPEHSBmMFjGQMhiUv0AhCwPf49B93jPozcpVn+W6z/3e9/O8j/R3MoB1nOEZbzhAUxz6Ti22cYsVTGABD3jHUBn11OEIJ2hIa4fyH6pJvTZwg+bU9+AVo6lXn/z46bxApjCeS8smHlGfF37KPc5zGTKG2Vi0ya92HMuQXvn6fCw7v8qLWIbsyde7Y/mrTZYX+b/elfpJ+Ybr1H9mV754iUG0YE7+Rq1fLaMlHbiTDxg7eQdreEJ7Ga1MK0YwjEb5d2hfyHKYqYi9jCXMyB/YNmxhPw7lnKpczVxhUVV8If3y69mp/6kmH+BYORguQbj8AAAAAElFTkSuQmCC>

[image5]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABAAAAAaCAYAAAC+aNwHAAABI0lEQVR4Xu2TsUoDQRCGR8TCiIUIvkAI+ApaCbEQCy0sAiK2BoTESgmICnY2WtjYCKl9AYOVnVYKapMUiiCClY1FEDHfOLt35xBsbO+Dr9j5Z4c9dk8kJ8sobuAlvuB38A3vsIXbOBE3ZJnFe9zEJXzCd5zDNTzGR7GBOrz0syswiZ+4GtaD+IE3SUdaPxEb0sGhGOxiAwfCWqdr01lsyKA9D2L5tMsSFsUatnwQ2BHL130Q0dNow4yrR+piecUHkSZ+YcEHAc1fcdgHkWe88MWAXmEX91w9YUrseFUfBPbxCkd8EPn3gAOx7x/zAayIXeG4D7K08dzV9Ikf4jUWXfaLebHjH+GC2D2f4i0uS/rQ+lKT9OdR9Zr0/ZfFnnBOzp/0AHxiPJh8LMsHAAAAAElFTkSuQmCC>

[image6]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAIcAAAAaCAYAAACdH0+XAAADmElEQVR4Xu2aaahMYRjHH2shEpFQtogi8YEvQsiS7EtJci3ZQlIURfZCJPHBlsu1S1mTfKFsKUSRyJKQEFJSSvz/nnfuvPdpzpg5t3uuGe+vft2Z93ln5txz/udd5l6RQCAQCASKmoHwl3OUqQXyZ5ukz2fBw3Ass42BSnPZNhQiIRxVQwhHIJIQjkAkIRyBSGKHo6Poi1/Dr/AufAgH+J0SIoSjaogVjnrwFTwK28GGcKLo1meI1y8T9yS9TcrFdfqyrESFowTeFP3MMtgbHofH4PB0t6KhNtwIb8P7cCmcAk/DUtipvGduxApHP9ELxxNe12vfDDt4z5MiUzhK4HJYSzTMPF6Obk3hVdHQFBu74WD3mDfpd3hQNBS8mTe5Wq7ECkcT+EXSd/d7eAj28DslSKZwnIU13OPOose5QDQoHDkYcJ/2cKxpKyQ4zW/1ns8S/Z27w27wMGzj1de6OmtRxAoH4dqCaw07DfT0OyVEpnD4zBU9NgbA0gLuhFdEh95i4Qh8Zhs9WsPPsKYteMQKB+9IvnlzOBnugY9FL8A5r18m8l1zrNGXZeVv4eCJemIbDSukeMLB6/MW7rUFj+nwlG00xAoHL8Yn2Mxr43D9HF702pLChoMLs/VwvOia6CPc59VHw2nec5ItHJyWRthGD/7ui2ErW/DgSDvGNno0hkvczyi4noiaBjgC7oJdRRfevLFKvDqnHH/k3C96zLzJuXA949VSxAoHFzv8cL64L2wAB8GfcJ7XLylsOHgheHzz4Rz3eIOrMdCXRHdYPtnC8Ub0PaKmTJ5c1qPuRIbnh2ifTFMbSf2xiz8z0UW0/g7WMTWySnQB2kt0fcG+Q12tj+jC1OcpHCc6gkyCDyqW/xArHAwCv9M4IXoQ3+AtuEjSi8AkseFoJLro5K6EQeDx3oAn4WrRHYuF4ThgGx18H17cqOAPE70xVtqCg+eEOyXuGOqbWooZoudyqi04OIV/ED0Ojg4WjihchPMib4Gz4XXREXOhVFxbMKD8rPOiX0VEESsc/xo2HHFgOOzd5TNB9AJWN9tFdyaVgaMFd5ct4UvR0dRfIqQI4XAwHGW20YN3Y75fIlUFF2xDDErhTPf4hegWl1+eWf77cHAKYjCuwUfucVu/g+hCk0NwdTNSoqeufLgj6S8rOYLsgP3Lq2n++3DkAudsruqrG/4pgTuxpCiacHCBRcO/CVaeovo3wUAgEAgULL8BfJjRlKJg8ZoAAAAASUVORK5CYII=>

[image7]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAIoAAAAaCAYAAABo4cQnAAAERUlEQVR4Xu2aeciNeRTHj33JLhKG7NuQiJk/pmRISU1jSYhCiLJvDTORbWwztlIzGeaVki37zj92GltkjaGk0VCW8oeRxvfr3Mfz3OO5973Pvbrv+977+9SnOOfyPst5fr9znvuKOBwOR77TES6H++EUk3P4DIYbRa/RlyaX83wD/4d/wK9hmfj0R6rDWjaYZ5SCTeAs0Ws2JD6d28wTPenKNiF6YVrA0fAmnBqfzmtuwD02mMv8LFoopW0CbIWn4SbRz7hC8fkLHrbBXGaxaBEkg1uSK5R4zsAjNljcqALnwwLRgz0W8Hv/YynxC3xngwZXKJ/ClZbXu9jSEp6EPWwiTVbCNzZocIXyKbwHx20wHYbDc/CK6B7/leievwX28T8WibLwlHze0Yzj3iMbNORCoTSE6+B1OBA2g9vgIXgb9oO14Sq4XXRrmfHhX4azA16ywagMh7NFR81Kohf5suiBnBAtoHSYCIfZYJqweR0EX8NRJmfJhULZBZvDP+FLuEb03pC1og/LQVgvFhsqes5cwcNg/KnoRMgHOC32io6WpLXoD5wgemBcUbrFch47YV8TC2O9hE8nUeHI+wC+gl1NLgyvUKbZRAmBPZ03ofCBvQor+ukPhcIHhu9IPPgQJSsU0hM+h//CXiYXmXGiP7CpTQToD+vaYAgXJL5xDZPLairUFH1qHsPuJmcp6YXiUV/0PH4w8Ttwt4kVwL9NLEg70ZVpLvzC5NJiM7xrg2lyAFawwQzhasY+KhleoUy3iRIGt22eR5dAjL0KY+MDsaqiq+2vgZilAN4Xf+eIDPerRXAALA+fiW4ZHhxfR8T+zO9Yloo2T6kwBo60wQxh8/afDRq8QknU3HHZbW+DARqJfj+SrMh5Xp1sMACfYK7OyW5MYcdRAP+R+O2b94Lnxu3YxnjeneFPgZwHp56jNhiFb8Wv0LGxP/PtJ6kj+t6DFUsmwVbwXuzvhcELzf6ng01kwDL41gYN3Jp4Hnzdb2kjmnsCy5mcxz7Rz0y2iRgc9ZlPdh3YW/Az3KbDSOU4uM1yygvCiZRbT5DfRHs4wtf0YX0K36NkVCjVRBtWTjcsCjY8Z0VXDV5oTj4e3Nt+hCsCscKoITpmzxG/Q8+EJZL4hRtvLL/TYJ43gV4UvZAe7K3Y/XNVSjS280Hh/5Ho3Q+byBeiI2cieIPZcLa1iRiFHQe/1GQLwAc5CCdQTpNBuNKzCNi39DY5DxaK1yBnhWuie2bUhojvYvhewDaz/Co8Ct4r/GRLeipw3Awu30VFto7jvGSxULj/cVRrAGeaXLZgP8VCSfTrBanCRrs4kK3j4MrK39/JCtz7uIStFp31iwI2aiyUWjYRge9Et8KiJlvHwR7ooWgLkDdwmuDez6aO+3g6LJQM3k5+RrJxHBwo+NuAvGbsP/MK9kdsqG/B303O4cNGeANcABubnMPhcDgcDkeJ5z282OA1eg2YXwAAAABJRU5ErkJggg==>

[image8]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABAAAAAbCAYAAAB1NA+iAAABFElEQVR4Xu3SPUsDQRDG8VEElaBYaAS18UNoo9jYBKxMFLGyifa+oZUIdhYBv4BdtBMkLyTYWWhhZSVW1laKQlr9D7Mck7vrIlb3wA9uZ5a53eNEsmT5w+zgDnXMoIkHvGHN7UvNLKqYxA9esBh6W/jGVFin5gDLWBIbsOJ6+qy19bAewSumox0uJ+gg52pHYgMKYd2PbfRFO1x6HtBAK1ariQ0Yj9UTGcAnjl1tQuxElbAu4RKH0Q6XebE3XYkdT496jXsMYRhlsePrkET28IVTPOIJ5xgMff36Y2hjNdS6ciPJ+8ej/8I7RpH3DT3uB858MSW7uMAGFnxjTuz+RV9MySZuxa7ZFf2zdMAz9mO9LP+RXxbyMnGfIpnsAAAAAElFTkSuQmCC>

[image9]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABsAAAAaCAYAAABGiCfwAAABUElEQVR4Xu2Uuy8FURCHh0RNhIJIxCuRiEooaPwBEoVKLg0a0SlQaTQ0KmrXoxGReHU6CpUGDSKUOoVSwTfO7Obsqe7d7BZkv+RLdn+ze+fMuScr8kdZw2/sDwt50IYfWBsW8mAGj8MwL3ZwUdyES3iaLGfLM06Im3AS75Ll7OgUdzgusCOoZY5Oc4Ct+IbNZi6Ucc6uX7EdN+JqwAJe4hE24DIe4oO4ranDMdzHa3uu7/dNxy122bVOuIWjcdVjQNxJ0od137XBsNVaLLvBeayx/BH37LoqdNwRcSdJf7jk1aJmulKfe0nZLGIbP8VtWcS0uGaDXtZtmR6K1OjWnASZbu+7JD9Bq/iFTV5WFdFq9aD4vOCud69Nn/Dc7stYH1crRP98bdbjZb2WTXmZfoo0m8UhSS6kYlbwLMjGxZ3MxiBfxyvclBRTFRQU/CN+AKFYPZNQOwDxAAAAAElFTkSuQmCC>

[image10]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABIAAAAaCAYAAAC6nQw6AAAA5ElEQVR4Xu2RvQ4BQRRGL4maiEaiQyJRCgWvIFEo0VCJB6BS06io/VWiEXQ6Cg9AQ6LXKxV8s3N3c3fiARRzkpPsntm5+0dksfwhHXiAaxiBXbiCV7iHIViGS3ji67LOTkEOTmESfkhvLvJanNsZtmGA+w0u+NhjCEuwSnpTTay5g8aiKS70Y5DLBL5Iv4ZLg/SgvGgpbk3RfKjH3RhNvfITBkXrwzeMiebh3kV9dMkDzsW5GniHOz6fwbC3SvpDqkFp0TLc6qIluLVggfw3cejBrdEqpP9g1OgDeIQjMp7GYvkbvhiYK6Z1PF2yAAAAAElFTkSuQmCC>

[image11]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAsAAAAbCAYAAACqenW9AAAAzUlEQVR4XmNgGEEgB4hfAfF/IC5Ek8MK0hggim3QJbCBpUD8AYiZ0SXQAUjBSyBegS6BDVgwQJyQBOXbAvFqID4DxPYwRTBQywBRLAPEGUAcywDR+AmIs5HUgcEhIL4ExAVAHAoV8wDi2UDMD1MEAsIMEFNBGGQtKBhZkRUgA5IUhzFAFNoxQNx8D4gXoahAAlOA+B0DInxBoQByPwjEALELlA0GIInNSPxNQLyTAaIZFFEsMAkOIP4FxJEwASCwBuLDQHyMAeK0UTBYAQDwPibt6wkwywAAAABJRU5ErkJggg==>

[image12]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAaCAYAAAD1wA/qAAACZUlEQVR4Xu2XS4iOYRTH/265bpBbwsJlgyI1K4ZEyiWXBUpN02CjMUMW4zbkrlxyDQsWNAtbySVyWQzFQpGEQmnWWCAr/v/O8zZnTjOTxby979T3r18955zn+773+Z7znOe8QEUVVZSnppGL5AY5QW6S9R1mFKxJZB95TD6R5+QNqUvxk2Q+2Ukmkj9kcpq/Kc0pXM3kF3lGlpCByd+PHCQvU3wQmUDWkQdpTml0jfwlW2MgqT/5Su443wWy39mFqwG2CKVUd7pHtjtbKbfY2YVqNPlBPpIBIRZ1HHbIJaXbNzK0PVysTsN2w//TvVLvYAuZHQO9SapKWsRvWKp0p7WwSlVafYEtZnDwew2Bldm+MUDtJkeiswjpftBC1sRA0ghyiyyMgaR5ZE50FqFh5D5pI9UhNhdWcmcFf2mly66WPCEvSAvsTqmB3eKdaTppgn2ms7TU/fIBtpvXydXkVyqqjB8gjcmnlFXrk13IZ9J4M1kO6ybUEvW4VBi2pfF7dJ1ae0krmULqk+88bKelh7AHzfSWLIB9/3cyk/Qh59ycHpUO/yjY+VHF6+oi3UXOBp8+UwuLvSJbXOwQbL5SXP2ebI1Xuzm5aBl5FJ1OethTzh4J6yBWJFuvAL63q4KdVRUg3WvabS0m9+5BP3jY2ePJRmfvQceFLCKfnX0b1ufpvGTSq8PlNH5NrrhYbtLdstTZG2CHdCos15/CdiD715WSd8kl2MOrc1bF1PtNJh30VWl8DPaduUpdwU8yPPi1I2ODr5RaCatGqja6Y7xUbY4GX2m1A5a7erUdE2LjyIzgq+h/9Q9gQWyZ9ArfTAAAAABJRU5ErkJggg==>

[image13]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAoAAAAZCAYAAAAIcL+IAAAAk0lEQVR4XmNgGAFAC4gfA3EmugQ6cADiJ1CaTqAEiNcD8X0gTkaTg4NKIG6GsicB8QOEFAIwA/ERIGaE8ncD8SmENHbAAcRfgbgXXQId1ALxXyAWQpdAByBrT6MLogOirXUA4v9A7IwmDgZmQGwDZYNMegbEbAhpBPgJxCuA2BiIPwFxJKo0AlwC4rNQ7IkmNzwBADMcGWIRS2g3AAAAAElFTkSuQmCC>

[image14]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAoAAAAaCAYAAACO5M0mAAAAdUlEQVR4XmNgGAUDAnKAeC8Q7wNiCyBeD8R7gLgVWZEmEM8HYi0g/g/El4FYAIhboHwhmMJ6ILYG4miohANUvAeIs6BsFDADiN8BMTO6BDq4BsQb0QXRgRwDxNpSdAl0kMAAUWiKJo4ByoB4GxAzokuMggECAFDrEyaZpH4LAAAAAElFTkSuQmCC>

[image15]: <data:image/png;base64,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>

[image16]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAoAAAAaCAYAAACO5M0mAAAAZUlEQVR4XmNgGFSAEYgV0QWRgRQQBwDxbiBejyYHB3lAfAmIpwDxLwY8CpHBV4ZhpnADuiA2QBuFG9EFsQGQwk3oguiAHYi/A/FeBkjiwABOQHyCAaLoPxTfZYAkEB4kdaOAQgAA/jAdQ1ULpgYAAAAASUVORK5CYII=>

[image17]: <data:image/png;base64,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>

[image18]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAEgAAAAbCAYAAADf98keAAADlElEQVR4Xu2YaahNURTHVzIVyvABKZlnZchMpg8UIZIv6BElU+Z5jlCmDIUi8zxlyJQhPpiLTCVTphKFlJLE/9/a2zt3Oe/ee66r9zznV7/eae1zzz1n3bX23ueJxMTExCSlO1xig0mYAgfaYGGlJrwEi9uBJBSBF2BnO1DYKAFvwaZ2IA2Y2DewjB2IQkm4El6Ft2HDxOF8Zwy8bIMROAon2WAUlsEHcDH8DLskDuc712CODUZgKHwt0drzFyzft3C5O66SOJzv1II/YEU7EAF/jYx++K6iH+5hBwoIg+FHG3Twnlld70V/ZLYhK6VY8CTHMzjRBpPRDZ6FT0QTxGOaUZb/IivgdRsEzeF3OBWWh5XhUtFn4ZxqOQ3X2mA6bBXNbrbhRN/GBjNgn+jDWWaJJoMTuKcc3CC6vFv2wt02mA534XEb/EMqwM3wqR3IgCNwjw1K7tTgfShabaykMDaKdkgkSouWKVevbNNAspMgVhANY7okJom+gqWCJznWw/M2mIr2ohcdYAeyQD3JToLYMmEtxiSwperCUfCA6ETN5+ErhoXtddgGUzFS9IK1A7G28B7cIlpZq+DCwDhvar/oxnIT7BUY44rDm+D8MEcSE8Tv4k3Od+eRTqJtsUD0laCsiwfhu9cVGxS9Dl89gjQWfR5WluUkXG2DqWBfcom0jIOPRPdFhCsdK4LsgoPcMVv0seiNtZDE8maifYK44gR3wlwUqsNj7i9hQsNeB4ZI+D0uEk3GOthINLmT4TfYJHCeh88zwgZTcQMeskHRlYGJ8HCX3Vr0AThn+YcinESnif5qBwNx3qRPEJffm3C2k3MKVzhWzwu4HfZ151pqiCaivomz6k6J7oM4/k50Eu4ZPMlRSfScVnYgGdxMMdvD7YBoO/CmPUwQK4LtxS9i33suipb7XHgiEA8miKW9MzDmqQaHiX7+g2gVhsGK+5N3qd7wvg3mBTeILMsOog/L7FrGyu8J4oROzsF+7pjb/y+wmWibvYRF3ViwxTq6Y99+3Iy2E60+H2OV5bVJZXXescEI7BCdNtLiueiXceLl0mdpCc+IJqU/HA0/iW4oq8I6ohMqd6WcPPmgHv7K20SvvQZ+ldxW5bzF686A40U3c2xJJmam+0zYBo9wZ8wEZ7Lx5ALEvV7Y7jqUCaL/1uCcwJb5V2By+MP4Ck0HJpyrF/dk/wV94DwbTAJbM8cGY2JiYgoiPwGP971TR3kWUAAAAABJRU5ErkJggg==>

[image19]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAD0AAAAbCAYAAADPl4fCAAADSUlEQVR4Xu2YWaiNURiGXzIPZSiJuDBlunCDG/MQQnLlxgUyZZ4i85A58xARmUUylzEipJAphESJIikJFxLv2/fv9trf2XNn49R+6qn9f2vtfdbwreE/QJEiRf4Bi+kAH0zDTDrUB8sSg+lWH8xAeXqVdvcFZYGW9Dmt4guyoCl9T2v6gmzRH11Pb9MHtE1iccE4Spf7YA6cpjN8MFvW0Kd0Bf1KeyQWF4Ra9Dtt5AtyYAR9Ryv5gkxUph/o2uhzw8TigjGKPvTBHGlGfyOPSeoJ+2J/X1BgdtIjPhgxjz6CZd1reoVeSKgRR+XTfTAVfegl+grWaX2WOY9antyFZZdnEv0Ba1912hzW4ethpQCVbfHBTOyFjdbf5iOd64PkIv1GOwSx9nR28ByibDnsg5l4TM+6WDm6gG6jN6Ln0uYLHe+DsIuKMi/mLTqVVg0rBeyAZWjW1KC/YLt2SG+6H7apHUJhOq2ZnuiDsM30FBI7LnU8JWM7bM1nTSfYDw5x8ZHIY53kyDMkT+8GtCKsbUrpc/QnrJ0dg3oxlNonfDAd42A/ps0iRmvYj2ijmU/70hewkd5Hd0X15tCVsHScHMWEGqwsUfbod3R5SDYTN+lqH4TVXepiWgZqpwbCo0HZ5IPp0Hr45INkNN0TPOsIUSN1Lk6IYpsRb8RlxF8YBiE+MErfdbRd9Byymx7zQdgurd1b361HG8OWmE6ZZOtaEzLWB9Nxhx73QZTstNJsY/As6tBhsLL7sKwRyxCvqzehVJvMcNjd2aMB1InyFja7OlkO0rZhpYj6SJ32SVEaaq3oZuQZg5KdDs/UuvQlHRg9K51jm5LO19hFYhZdGH32NIE1ODyackVZ9cQHk6FG6VWuM+yParQ8Spc9wbM2nLDTvZB4tuvI06VC67sLPUk3wN57KwT1PBocLZN8OUCn+GAy3sDuvNostN17NPLnYXX0g93oNdjMxmazGmwD0Tmujmo21YGuiJ8IMe8h9VubUvYz8nu11OarO0ZW350Ge4XU7lzblZUGZ2ir4FkpqF0/FdrdV/lgBvRPBA26Tpr/At3gdNbG0Gz6G59HR18/H0yD9phhPvgv0fG0BHYC6H13EW0RVihSpEip8Ad5zK5gjl5m8wAAAABJRU5ErkJggg==>

[image20]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAFIAAAAbCAYAAADve9g/AAAEGElEQVR4Xu2Ye8gVVRTFl6SlVuKjwswXomYWSSIpUfaZoWAZqFhhqVmkFT4i0VLUz0BRMS18FxpXxRdFVAqaD7LASAsylIrEFNLyhaRiYhG6lnvGe2bHvXfm8x+v3h8suGfvedzZ5+x99gxQoUKFCmVLG+pjbyxCI+pTqoF3XM/cRH1NtXb2UvSkvqBu8I7rlSXU696Ykg+pt7wxK3Wpd6lvqd3UvUl3WdCBOouap2hX6ihVzzuy8A71EzWDOkM9lnSXBXqGZd6YkQPUSG9Mi+qKZmJO9PuupLtsOEQ97Y0ZyVHbvDEtKrQXqCe8o4xoBXuG+72D1KeWU/uov2GZ9w1VHR4U8RJ1yhtL0ZvaQu2H/Qn9lsoxrfvBnqGhd5CPqJ+pTrCgPghbvdPDgyK6w67T1DvSoNlSbShnRlH/eSOsnfmH+gHJ4LxIPRmMY7SiFcj7vCMNe6gN3lgE3WwetQNWk8JxDraq06CmeUowvpPqGIyzMJY64o0RX8KCI52jNlIDqVrhQREtYcdVOXtJboHNpHbrtGyHtRpvwupqOH4DNttp0CSEZeQZakAwzoJW5DFvjFBwfkE+mLHGhQdFNIf5tG9k4mHYiXqItBxEMk38uCa0hvWvNQ3k89S/3hjRAtYnK5VnwXplPfNJWJcSopSWr4uzl+Q12IntnH0iNZN6mxoT2F+h/qLmwmYtHPelVsD+qHiI2gtLd63496hpkU/lYD21MBoPo36lVlGTYZvCcFi6fgVrsmdTf8Lu4+kDe45bnf3GyN7L2ddR5/H/V8J4Yd3h7CX5gDrhjWQ+7KJiK5KF+XeqbYGxAvRb4NPrmgIUz7w6BJUBocY3F/0Wqq1+RU6FbYZCZWBQ3pVAHyoUgAecXStRdt23P2xCtOrUAq0JjosZisIloijfUZ94I2lMvUBNgO14WrkxxQJ5D5KBVO1aHYz1AN2i3yNQOpC63mnYa5veXBSYQhyAXTOkDnUc9sajHlLpryxRBjUJjotZQH3ujaXQTXThl51dN1DzGqfQSlhAYtSDhYEMx1ptYSA1ATo/RoFUygulbi7vuhxIFfz2gX0nNRj5slAIfXT4zBszsot61hsLoUZctekR2LL3G8XjSPaVao1Gw+qlOIxkIMOx2pcwkDrPBzIuGaqvubzrUmY8B7t/VWDXNXQPNdLF0ARpYTTzjpRon/iDqu0dhThI/QibYX128qjQq9daDAteNexb3aPUeFiRVg3t7MYKwPuwLzBqgfTgm2HBU9+mmqjXL9W8p6hNsJ06bjUUCKXgUli2xNwOq7NpWAtrw2pCDlYjU6NeTzvr97Cvw1c7N1OTvLEAt8FKQdYPLz2QzJxrikXUEOpV2EeJtNwNa2/Soo1VnYky8ZpE/aQ2j7SrsUKFChWuhIvt1Nkv26y/igAAAABJRU5ErkJggg==>

[image21]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADkAAAAbCAYAAADGfCe4AAAC90lEQVR4Xu2XWahOURTHVzJGuqRMLzIkIeSJDEUZInmjRCgpQ0rGkGvKzJMMoSSSEOWBPBAP5kwZHnAVkq4HpbxI/P+tvbPPsr/vnKPv6Hs4v/rV+dba556991ln731FSkpK/oFJcIcNVmEUPGiD9UwfeBO2tokUtsB1NliPtIEP4TCbyEBL+AqOtImstIX74R34GA5MpmvGUnjLBnOwEl6ywazsgS/gdvgNjkuma8ZdONcGc8BS/wUH2UQaLKHPcK+77plM14y+oh3sahM5aYKNNpjGeNGHT7GJGjMHfrVBB5/Nt/xFdMJZ0h9hq7CR4wS8aIOVmAivwTeig+Q1LapU98F7NgiGw59wNewMu8Odon3iWmHhCvvUBtPgzLAEiuYsvGqDYL3ogLgoeTrBw7BFEPMsgZ9sMI1n8LINFgBL7IwNyp/PxftS9K3zjcaYDX9IfAKidBAtFa6qRcM3SWOsleRA6QfYPmzkmCWazzxIHpd4wwybKACWX6xcORCWZ3+4GJ4TXXzYr1VBO88iqbyAReEN/GP9ghgXntfOwXCaaHlwtrkQnIRrRM+eE9w9veAN0YVlF7wPG1zOw/a3TYxsEj3mhfC57BefaWGMB5bMHBFdti2TRb8Nwr1zt7vuKDrTXWA72OyuyVj43f3e4NqGzJP4s7aJDuiA6CbPyeHJhhM7NGjnOQpP2WA1OOMXbFD0nMiS4RlzuiQfxk2dFcBVkWXjTx8s/Ue+UYTeooMZYOKb4RXRfZJ5Thy3sqlhowBuH5yETHCj5WwtsAkHT0DcrzYGsdGiZezL+z0c4q45yNg+GNIEV9hgDlgd7HM3m7DwEMDSYIc5c5Vu4CbN/WhZEOOsc18lnCS+yZnOMZI+SH5PT2wwBwvheRuM8U70QVvhoWTqL57DHsFvlib/U+G9HPAxeFq0rLkgcdDc42InFcL4WzjCJjLAT4ilys8lleWiHX0gunT/bzjA66KdzgM/m/k2WM9wMWu0wSrwez9ugyUlJfXPb2Lal72wqiq5AAAAAElFTkSuQmCC>

[image22]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACgAAAAaCAYAAADFTB7LAAACc0lEQVR4Xu2WW4hPURTGF00kEUJyv8aDFI+SKZRSHqREeBEpjZqSJA+UPIh3ClMTmnHN5UFSEg9uEUopZUpTGCWUJ4nva+3Nmq8z53+G+T/5/+p72N9aZ+91zln77GPWoEFlZkIX1SxhCXRMzXoxFLoLTRe/FgehfWqWcQj62YfmhTzlONSqZgWaoFfQYg3UYoZ5UQ81UAAL/waN1EBFdkNX1awFF2WBDzRQwFHolJr9YJb5WvM1oLCP+JpuQp3mF32E7kCXoZW/M3vTDa1Ts590QQfUjEyBHpv3w2hosnmBt1J8WxofTuPMtOQvEJ/MMb+eN/AVegq9hJbFpEQ7dEXNyGnzhbaksRZI7iVvffDWJG9U8Mgw6C3UYd7PI8yfMnOL3gR38gs1Ix/ML16axkUFnkwePw2ZndCPMM40m+fyxocE/4h5zykt0Ds1I2fMJ9yQxkUFsjfprQ3eLuh9GGfGQJ/N86ke8zUWxqTAZug7NFgDGfbgI+i5+YmgBW5M4xvQoOQRPkEuXgR7jb2Xi8xaFJMSef4+CyR5F7OZL5hf8Nr8hLgGrbbexZFN5neuMI83Od588RPmG5BzXg95mR3mT7wybGRO9gSaILHIKvM8boLIcugTNC543DhvzN+Cshd6pmYZ+RXf1oDAdmCe9la+QbYIN95waIX5huLTUrgBz6pZxFbzSe+bL/AljampIS/SBW0Xj8Xwm3fefB4ehTyV2ELaJoSfGB55daHN/uIsDfAMZx+XtdI/wT8RLjBRAxXh07+k5kDDs3uPmhXg7xZf72wNDDRjzX/NJmmgBvvtz/Fad+ZC59Qsgb/87N8G/ze/ANabj7Bbx/rnAAAAAElFTkSuQmCC>

[image23]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACUAAAAZCAYAAAC2JufVAAABNklEQVR4Xu3TMSiEYRjA8YeIC4siLrEoEQalDIbLdduNZ1E2i0USI2VkUFaDAaUsyqCwGCTJoJSU1WiyycD/6fnkvadT6r6bvP/61ed9n7ter49ILBaL/Z+aMI6s3wjKYAy9fiPtGrCDZ7zgE3coBjP1WMYrDrCNQ/QFM6m2hLXkWW9iHu9ih5tFHXZxg55kTusUO5jup5re0olfpDze8IF93KKtbMKaEZut2CQu/ugUrfYxGcRm8uwriN2WmnB7341g1S9WWz82/GLQldihrtHi9rRRrPvFatM/n95epUq4x5HYwfbQWDZhtzTt1lJpESvBz3rQBTyKvdjNOBM7mP4CU2gX+2+8lBq86Jp+qb5X59jCE47RHczoDc3hQX7eM53pCGZqUhdyGHbrviEM+MVYLBb7pS9nWDcihkXeBAAAAABJRU5ErkJggg==>

[image24]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACkAAAAbCAYAAADh0qZQAAACZ0lEQVR4Xu2WS+gNYRjGH/e73JI7KSEiEpHkUpRsKLJQioUFSsmlLKwslGxYkOjIArFwWUhISW4LFKGk/0ZyKbFASDxv74zznuecOWfm5CzU+dVvMc/7njMz33zzfQO0adM04+l5DeswkF6g/bXQKnrQW3Sc5I1YQq/SLlpoBUfoNg1zcoLu1jCLefQ+/Ul/0yf0Gr1N3yRZ6vDkN8Yk+gXNP7Y59B3tpYV62IXZhYzWAlkNv6DOITtAj4fjZuigWzTMojf9Su9qIaErfSjZa7pGsqKU6A0Ns1gGH8U9IRuJ8pzpQ4+F2lh4/7SQpdgNn6Qv4Tf+jN6he2NTwkb6WcMs9sNPOjM57kSP0s1/OypZCe8foAVyjj6n0+EXPBs+6vtiU8IC+P8M00It7E7jC5I6JTYFttJfGsKXlB/0ESpPvIGuCMcp9iTsPFO1oNjiam/25ZDZxdndZ7GdvtUw4SbKN/mNXoG/ePZ0lDHwvoWSV5E+up0hmwif1FnYSL7XMMFO/ALVT2VHbEoYBa/Z4l6XQ/DGuSHrSweHY2UdfPRrYUtYT/jjtbl+D/7/H+E7VMQes9VmSV6FvX2f4MtMXpbD/7yf5N2TfKnkZ+l3VG+D8+H9QyWvYDK86ZIWGmAfFfa7GZLbCFr+iq6C70Y2WjYQp0NfynpkTxsshu8wj1GeM7aulUJPIzroJsm60Q/wncjWSJsST+lB1J4+h1F8gAphHwgXNSzIA7pWw3+JfZjYSI3QQk4mwD9girwLTXGG7tIwJyX4nGw5Q+CfebbPF2ERPaVhK7GF35aYvAyi1+H7eps2/w1/AL8HfonmkYhaAAAAAElFTkSuQmCC>

[image25]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABIAAAAaCAYAAAC6nQw6AAABEklEQVR4XmNgGAV0BYxAvBmIPwLxfyi+DsQWyIqAYC5UDoQfA3E4qjQC+DNAFM1Bl4ACJiA+D8S+6BLowJABYtB2dAkoyAXiZnRBbICPAeEtdCABxPuAmAVdAhd4DsQ/GSDhhgwWAbE6mhhecIwB4ipxJDEfIC5C4hMFljFADDKD8vmhYuguLATifjQxFNDGADEoCMrvA2IFuCwCmDAgLMMK0hkgBuUBsRUQJ6NKEw/cGCAGTQLidjQ5EOBlgCQBUAyKoMmhAEUGiEG3GLArzABiDiB+AMRKqFKoAJR6fwGxO7oEFMgCsR0Qn0GXIAdMBuJiBoihZANmIH7JAElnE9DkSAKg9HSaAWKILprcKBhyAAA6Sy1DiyZmfAAAAABJRU5ErkJggg==>

[image26]: <data:image/png;base64,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>

[image27]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAE0AAAAaCAYAAADygtH/AAACpklEQVR4Xu2XWaiOURSG38yRKYkyFJGhyHBBmX5SFKGEFBdcGCJTUaYbRIYMoZTpRKcj5AIXZLigKJE5pUi5wYVEXCjxvq31+ff5/onUf/jsp57O3muv0zn/OmsPB4hEIpG/lta0hj6nF+kh2j9MiBRSR+cG8+v0M+0RxEqylL6j3+nK1FpW6Qz7vE+C2GqPbQpiZVkA+4aR6YWMoqK9pXeD2FpYDbYHsbLU0g+0cXohw3SkLYL5GVjRckGsJCqUqn4qvfAfMZl+pQvTC6UYDqvwfJ+PglVdrTsmSWpg7sN+x191i31bRQbQs7BdpouhVf3l0myE/aCudBHsRlEBP9IlQV6W0fPjHn1E26XWinIDlryCzvDYRHqYtk2SyDzaNJhnjWmw5qlJxQvogHxLazvq+VGsMLptPqH42r9IEzrIvyb0hNXhGyps05mwxNGw7fmSnqiXYUynl9LBKvK7Z1qlt9ZeWN7OINbbY7J9EC/gAH2P/FNDF4C2qphDx9ML9DXsIahxwhR6ju6BHaI6C/rC3juKi0b0Jm0D69Id9IrnHKW7Pa/aHIEVZ2sQm+CxO0GsKCpQWIjz9DKsiHq7Je2rD57zsdCZ9wz5NtaHX0OX0X70sccHwgouJvn8KeyyUTcU6+pqMJS+oUN8rjebjqcvqPBi0MNOb5PZQWwErEC3YFtWqIN0njVPkmDri4O5irafdqOr6C6P6xZWFwptf52N+v8ufFQ2FDn6kD6A/ZHV+X3ChD9BXaUtJTrRlrA27vUzA7hNp/r4Gh3nY22D5chf47PoVR9nGp0/2vvqtPUee0G7+HgsPelj8Qq21gx2sQyj63ztIN3s40wzGHbWaat195gugdN0Gz0O676EDfQY3Qe7JFRQFVaoy8qeGZFIJBKJRLLED4RplpOgOsRaAAAAAElFTkSuQmCC>

[image28]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAFgAAAAaCAYAAAAzBZtTAAAEHUlEQVR4Xu2YaaxeUxSGl5kQtLSouYioIjHGHGPQHy2h8QNNNRohMU81XmPMxJi2aQw1FDE0RamWECVIjBHBD038oCIxJZpIhPex9u5d37rnu7eJm4h7z5O8yXfevc/5ztlnr7XXPmYtLS0tw4ZTpYXSO9LT0radzV3ZXJojvSG9J53R2fwP06U9w/EG0i3SfsEb0pwlfSyNLsdXS9+H425sKn1qfj5sLX0t9dQOhSXSX0mPSmvHTkOVzaQV0knBW136Vro4eE3cYT7AkTOlH6W1gve69JH0k/SqdE5oG/JMM59Ruyd/sfRK8jLLpKeSd7j59Q4OHtfaLhwPK+4zHxDCO/Kc9Ju0bvIr5F7Om5n8vYp/ZfBes//RAPMAn5s/BHmS8Dulo0cvu0kTs5l43vxaDFhkXvF3SH5lb/N2XlCE/8R/KHgM8FTpBfNc/6G0f2jvF1ZILvCitJX0svlK/I10Yug3WHCDl0k7SutLx0tvm6/8o0I/8ijhOz54TSyy5gF+vPi7JL9yqDUP8LjiPxY8xuaGcHyveXoZGbxGtje/ERYKLvqFdFBpm2IeYmPK8WCwq3RVNsV60p3ScvMXzAOwmlMKDQQP3zTADBD+TsmvMAObBpgXgv9I8HJ+r31uTH4fLpKOkA4xP2FCaOM3Xlyd/y0bSmOzGdhGOl+6XNojtXVjrvl9bpH8J4u/SfIrO5u33598Igb/ruRHiDz6fJYbukHd+Lv5iZVLzS9yTPAGAxadB8zTEg93bGdzB4TgAdlM3G1+n0RjhHyJT6ppghqYdjYZkX2KXyPtBPPImryyh9lq5n1+CF6/vGR9S5oF5hfhRgYTZgZhzaJxnPkMfMKad16nS0dlMzHF/D7jTgvI60uTl/lKejZ5vHCux73BteX4mpU9zDYu3rvB68qa0s/mC0+FxYYZTV6Es80HnEHhzZIjb5ZONi+6qRMPK337gxtjB5Th3Lek66QDzWcdqYtBigV/EyOkX6XTgscOi83CucGr6Wed4LFwfWk+IytsTr6z3v9lESafx/tgnWKAifwB2de8M7OIP+LhyF88MOFM3mR386D0TGnnBfwhHWlOT9FAkPe44SZ40TPMd0sMDosMVc2qMMm88lmjHPM9gZ1XrIFrRJ4XPNqZ5ZwPLLbM6lg2MiZEd/3uwPMTgRQE9B+QC8xnQI/5lP9AutV63zQfNph51H61wiCM3i+/Yb75juq/5GjpNvOUc4/1LaFukv40j4wI/Yjeh6VZ1jwBWEBnm3+TYHyIQiJnlWDHk/NvhlnMzKphwi7n9vKbt/iLtKX1LZWGPUx3Bu763JCgZCMsKnwWrNUFiwGLJKt9tx3ZsKWWJJQi/XGFdEk4XiZtVH5TQ79pHmLk0ZYAKygD/Il0YWpraWlpaWlpaWnmbx2H1xNjZkd+AAAAAElFTkSuQmCC>

[image29]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACwAAAAaCAYAAADMp76xAAACKUlEQVR4Xu2WSahPYRjGH3MydilCWViQhWFzRaYkmZKNxMZQFLtLWRgKmcpGMhZRSiRJElIWLAghZSjFLZGFUgrFgue5z3v6/88Xt7s4OmdxfvWr77zvufe85z3f8AdqampKZRq9Td/R3/RBPt3BTfoLzn+ku/LpcrhF2+GiWvOpDjbRM2mwLPrRV3Q1XPCVXNYcoXPSYFnMo4dpbzS6PLb5BvIQzleCg3RpjDfDBesFMkbT603XpfOYDoxxf/qZfqWDI7YensOVYBi9m8T2wV1Wt8VFOqmR7pQxdG4aLJKVdHsSG06/0w90AH2dT3eKFufWNFgkp+C9OOUE3OVzYVd5QmekwSJ5SXumQTIOLliuSXJiFt1JD8BTYDk8tX7AB5GuxSB6Gu78eboKft5++EDaQHfDi3pU/M0/WUif0m5pIrgEFzwyiXeHOyk20j0xng8Xm6HC7tB1cd1Cv9AFdAp8qmpKih3wbvVXpsM7Q9bBdjqz+YZgKn2eBuGCX9BntI32irgeqAdnLKLv0fiCKljPmwjvRt9on8gdhafnf0H7stxG39IlEb9HZ8MvoOL20rORE4vpoxhrSjV/jfvwSVs46shPNH5vXKWT4a5r/vaFP7PWwFp6PO4TN+iEGG+hx2I8Hn7ZHnFdOPrs2qNPwosmQ7uJ5vSyuNZRfg1enFpw2WkqLsMd1v86RIc05SrJJzo0DVYV/bB6kwarygh6Ae7wiiRXU9MV/gDMTGeahsh4tgAAAABJRU5ErkJggg==>