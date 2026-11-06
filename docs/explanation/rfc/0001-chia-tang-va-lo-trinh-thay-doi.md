# RFC-0001: Chia tầng bài toán, và lập lộ trình thay đổi workload

**Trạng thái:** Đang thảo luận. Chưa vào `ROADMAP.md`, chưa có quyết định nào.
**Ghi lại từ:** trao đổi giữa Founder và Claude ngày 2026-11-06, nhánh
`24-challenge-alibaba`.
**Ngôn ngữ:** tiếng Việt theo yêu cầu của Founder. Đây là tài liệu làm việc nội bộ để
đọc lại, sửa và tranh luận tiếp.

## Tóm tắt

Cuộc trao đổi đi đến ba ý, xếp theo mức độ chắc chắn:

1. **Hệ hình quan trọng nhất mà Kuberina nên học từ hàng hải là cách chia nhỏ bài
   toán.** Tên tàu và phép so sánh từng ràng buộc không quan trọng bằng điều này.
1. **Thuật toán của tầng sau có thể chọn tự do:** GA, giả lập luyện kim (simulated
   annealing, SA) hoặc thứ khác. Chạy lâu không phải là yêu cầu của toán hàng hải.
1. **Tính năng lớn tiếp theo có thể là lập lộ trình thay đổi workload,** tương tự việc
   tàu cập cảng, dỡ hàng rồi xếp hàng mới: tính thứ tự dỡ và chuyển sao cho tài nguyên
   ít mất ổn định nhất, tránh cascading failure.

Cách đọc tài liệu: mục "Đã đo" và "Đã kiểm tra trong repo" là sự thật có nguồn. Mục "Giả
thuyết" chưa được đo. Mục "Câu hỏi mở" là chỗ cần Founder quyết định.

## Diễn biến trao đổi

### Lượt 1: Kuberina có nhất thiết phải chạy lâu không

Founder hỏi (đã sửa lỗi gõ):

> Kuberina, theo nguyên bản có nhất thiết là phải chạy như giải chuyện xếp container lên
> siêu tàu Irina? Tức là ừ thì là áp dụng toán hàng hải, nhưng có nhất thiết là phải chạy
> lâu không? Thứ gì làm nên maritime stowage placement nổi tiếng của MSC Irina?

Claude trả lời:

- Không nhất thiết. Chạy lâu là do GA mà Kuberina chọn. Ngành stowage chia nhỏ bài toán
  chính là để giải nhanh.
- MSC Irina nổi tiếng vì kích thước. Claude không tìm thấy phương pháp xếp hàng nào
  riêng của con tàu này, dù chưa tìm chuyên sâu.
- PAPER §3.1.4 kể về cách chia hai tầng, nhưng solver chưa làm vậy.
- Chữ "isomorphic" trong PAPER §3.1 có lẽ quá mạnh, vì Kuberina không có thứ tự cảng,
  restow hay trọng lực.

### Lượt 2: Founder đề xuất hướng đi

Founder nói (đã sửa lỗi gõ):

> Tức là cái master planning của chúng ta thật ra cũng làm khá tốt rồi, còn phần hai,
> slot planning thì không nhất thiết là GA, mà có thể là một thuật toán khác. Giả lập
> nguội kim loại, GA, ... Cái này ta tự do hơn. Hệ hình quan trọng là cách mà ngành hàng
> hải chia nhỏ bài toán.
>
> Một chức năng mà Kuberina cần làm tiếp theo, có lẽ là... Như bạn nói, tàu có lộ trình
> cập cảng và bóc dỡ hàng xong lại xếp hàng mới lên tiếp. Vậy Kuberina sẽ cần phải có
> chức năng tương tự: nếu có biến động thay đổi workload, nó sẽ phải tính được thứ tự dỡ
> tài nguyên như thế nào ít gây mất ổn định tài nguyên nhất (tránh cascading failure).
> Nói chung là cũng sẽ đối mặt với chuyện lên lộ trình thay đổi workload thôi.

Phần còn lại của tài liệu là phân tích của Claude về đề xuất này, kể cả những chỗ
Claude chưa đồng ý hoàn toàn. Mỗi chỗ như vậy được ghi rõ để Founder phản biện.

## Ngành hàng hải làm thế nào

Mức độ kiểm chứng: các điểm dưới đây lấy từ kết quả tra cứu ngày 2026-11-06. Claude chưa
đọc toàn văn các bài báo. Cần đọc trước khi đưa vào `PAPER.md`.

- **Chia hai tầng.** Master planning phân nhóm container vào các khu vực bay. Slot
  planning xếp từng container vào vị trí cụ thể
  ([arXiv 2408.08224](https://arxiv.org/html/2408.08224v1)).
- **Tầng nào cũng khó.** Slot planning đã là NP-hard
  ([ITU Copenhagen](https://pure.itu.dk/en/publications/a-constraint-programming-model-for-fast-optimal-stowage-of-contai/)).
  Master planning được mô tả là khó nhất và ảnh hưởng nhiều nhất đến lượng hàng xếp được
  ([arXiv 2408.08224](https://arxiv.org/html/2408.08224v1)).
- **Tốc độ là yêu cầu vận hành.** Nhóm ITU báo cáo tạo được kế hoạch gần tối ưu cho tàu
  lớn trong vài phút. Tóm tắt tra cứu còn nhắc một nghiên cứu slot planning đặt mục tiêu
  dưới 10 phút để kịp đổi danh sách hàng vào phút cuối; trang này
  ([MDPI Logistics](https://www.mdpi.com/2305-6290/5/4/67/htm)) chặn truy cập tự động nên
  Claude chưa xác nhận được. Chưa kiểm tra các con số này trên một tàu cỡ Irina.
- **Thuật toán ở mỗi tầng rất đa dạng.** Tóm tắt tra cứu và PAPER §2 cho biết nhóm Pacino
  dùng mixed-integer programming (MIP) cho master planning và cách tiếp cận dựa trên ràng
  buộc cho slot planning. Claude chưa mở được luận án gốc để xác nhận. SA và GA cũng đã
  được thử cho stowage
  ([Strathclyde](https://strathprints.strath.ac.uk/62504)).
- **Restow là cái giá của lộ trình.** Hàng bị xếp đè lên hàng phải dỡ trước gây ra
  restow bắt buộc. Một khảo sát năm 2023 dẫn kết quả rằng tối thiểu hóa restow bắt buộc
  là NP-hard
  ([arXiv 2307.07573](https://arxiv.org/pdf/2307.07573)).
- **MSC Irina** dài 399,99 m, rộng 61,33 m, chở khoảng 24.346 TEU
  ([GoComet](https://framer.gocomet.com/vessel-info)), xếp tới 25 lớp
  ([PortNews](https://en.portnews.ru/news/344392)). Các nguồn lệch nhau đôi chút, nên
  cần đối chiếu số liệu chính thức nếu trích dẫn.

## Kuberina đang ở đâu

### Đã đo

Các số dưới đây đo trên seed 42 của trace Alibaba (1.213 node, 10.866 Pod), commit
`624db06`, theo `docs/references/sessions/2026-11-03-alibaba-scale-check.md`:

| Phần | Thời gian | Kết quả |
|---|---|---|
| Walk in-order, không GA | 17,09 giây tổng | 8.312 Pod được xếp, 94,34% GPU |
| GA chạy thêm | 196,47 giây | fitness giảm khoảng 9 phần tỷ, chuyển 43 Pod, allocation không đổi |
| Một lần đánh giá fitness | 3,676 ms | phần GPU bị kẹt chiếm 22% |

- Cùng seed, FGD đạt 95,29% và Best-Fit đạt 93,13%. Pod bị từ chối đầu tiên: FGD ở vị
  trí 7.885, walk ở 7.775, Best-Fit ở 7.727.
- Trên MSC Irina, GA giữ nguyên số node (540 → 540, 615 → 615) ở `096177e`, theo
  [ADR-0003](../adr/0003-ga-must-beat-the-walk.md).
- X-5 đã chạy lại ở `e5f520f` (cùng record). Walk cho blueprint giống hệt và mất 14,62 giây.
  GA mất 182,06 giây, chuyển 50 Pod, và allocation vẫn không đổi.
- Cũng ở `e5f520f`, chế độ `free` chạy 1.313,81 giây và kết thúc ở 6.799 Pod, 93,05% GPU, kém
  hơn walk. Đây là thêm một bằng chứng rằng cách dựng lời giải ban đầu quan trọng hơn thời
  gian tìm kiếm, ít nhất với GA ở cấu hình hiện tại.

### Đã kiểm tra trong repo

- Solver không có tầng nhóm node. Tìm `pool` trong `solver/src` không có kết quả.
- `Blueprint.assignment` là một mảng phẳng gán từng Pod vào node. Phase 1 và GA quyết định
  cùng biến này.
- Việc xếp Pod vào từng GPU device đã tách riêng, bằng best-fit trong
  `solver/src/gpu_devices.rs` (`decode_devices`).
- Depth-1 repair trong `solver/src/admission.rs` đã là một restow nhỏ: chuyển một Pod đã
  nhận sang node khác để có chỗ cho Pod mới.
- Chưa có khái niệm nào về chuyển từ blueprint cũ sang blueprint mới. `docs/DESIGN.md:59`
  từng hỏi về "Diff Engine", PAPER §9 nhắc đến incremental re-planning, và ROADMAP Phase 3
  có drift-triggered re-planning. Mục ROADMAP đó nói về khi nào cần lập kế hoạch mới, chưa
  nói cách chuyển sang kế hoạch ấy.
- IR, solver, forge và inspector chưa biết đến PodDisruptionBudget, `minAvailable` hay
  `maxUnavailable`.

## Ý 1: Chia tầng

### Chỗ đồng ý

Về hiệu quả, Founder đúng: Phase 1 làm gần như toàn bộ công việc, còn GA thêm rất ít. Đây
cũng là lý do [ADR-0003](../adr/0003-ga-must-beat-the-walk.md) tồn tại.

### Chỗ Claude muốn phản biện

Câu "master planning của chúng ta làm khá tốt rồi" đúng về vai trò, nhưng có hai điểm cần
giữ cho chính xác:

- **Về cấu trúc, Kuberina chưa có master planning.** Tầng master của hàng hải quyết định
  một biến thô hơn, là nhóm hàng vào khu vực nào. Walk thì quyết định luôn từng Pod vào
  từng node, tức là làm cả hai tầng một lượt. Sau đó GA tìm lại trên đúng biến đó, nên hai
  phase hiện tại không phải hai tầng theo nghĩa của hàng hải.
- **"Khá tốt" mới có một seed và vẫn thua FGD:** 94,34% so với 95,29%. X-6 sẽ trả lời trên
  mười seed.

### Ba cách hiểu "hai tầng" trong Kuberina

| Cách hiểu | Tầng 1 | Tầng 2 | Hiện trạng | Nhận xét |
|---|---|---|---|---|
| A (Founder) | Walk hoặc FFD dựng lời giải | Thuật toán tinh chỉnh | Đã có | Dễ thực hiện nhất, nhưng hai tầng cùng một biến |
| B | Pod vào node | Pod vào GPU device | Đã tách, tầng 2 là best-fit | Chỉ có ý nghĩa với cluster có GPU device |
| C | Nhóm Pod vào nhóm node | Pod vào node trong nhóm | Chưa có; PAPER §3.1.4 mô tả | Gần hàng hải nhất, và là cách duy nhất thu nhỏ không gian tìm kiếm |

Nghiêng của Claude: C đáng thử nhất nếu mục tiêu là tốc độ ở cluster rất lớn. Trace có sẵn
một cách nhóm tự nhiên là theo loại node và loại GPU. Rủi ro của C là tầng 1 chọn sai nhóm
thì tầng 2 không sửa được, nên chất lượng có thể giảm. Đây là giả thuyết, chưa đo.

## Ý 2: Tầng sau không nhất thiết là GA

Claude đồng ý rằng thuật toán có thể chọn tự do. Có hai lưu ý.

**Luật của ADR-0003 nên áp dụng cho mọi ứng viên.** ADR hiện chỉ nói về GA. Nếu thay bằng
SA, SA cũng phải chứng minh được giá trị so với walk. Như vậy kết quả X-7 không nhất thiết
là dấu chấm hết cho tầng 2, chỉ là dấu chấm hết cho GA nếu GA thua.

**Giả thuyết: giới hạn có thể nằm ở hàm mục tiêu, không phải ở thuật toán tìm kiếm.** Dưới
in-order, tập Pod được nhận đã cố định, và 1.213/1.213 node đều đang dùng, nên số node và
số Pod đều không còn gì để thắng. Theo ADR-0003, trên MSC Irina fragmentation chiếm 99,35%
fitness còn affinity chỉ 0,30%. Trong điều kiện đó, SA có thể cũng chỉ giảm vài phần tỷ. Vì
vậy câu hỏi "tầng 2 cần tối ưu cái gì" nên được trả lời trước câu "dùng thuật toán gì".

Một lưu ý về chi phí: SA thường cần rất nhiều bước đơn lẻ. Với 3,676 ms mỗi lần đánh giá
toàn bộ, một triệu bước tốn khoảng một giờ nếu chạy tuần tự. SA chỉ thực tế khi có đánh giá
gia tăng, là việc đã có trong mục *Fitness evaluation cost* của ROADMAP (S-1 trong
`TODO.md`).

## Ý 3: Lộ trình thay đổi workload

### Bài toán

Đầu vào là blueprint A đang chạy và một thay đổi: thêm, bớt hoặc đổi workload. Đầu ra hiện
nay là blueprint B mới. Đề xuất là thêm một _lộ trình thay đổi_: danh sách có thứ tự các
bước đi từ A đến B, sao cho mỗi trạng thái trung gian đều an toàn.

Phác thảo để dễ thảo luận, chưa phải đặc tả:

```yaml
rotation:
  from: blueprint-a.yaml
  to: blueprint-b.yaml
  steps:
    - move: [paib-gpu/train-a]
      room_needed_on: [gpu-node-07]
      why: "tạo chỗ cho train-c ở bước 2"
    - place: [paib-gpu/train-c]
```

### Phép so sánh với tàu đúng đến đâu

| Tàu | Kubernetes | Giữ được không |
|---|---|---|
| Lịch cảng biết trước | Thay đổi đã lên lịch, Job biết thời điểm kết thúc | Một phần: nhiều thay đổi không biết trước |
| Restow tốn tiền và thời gian | Di chuyển Pod gây gián đoạn | Có. Đây là phần giá trị nhất |
| Hàng dưới không lấy ra được nếu chưa dỡ hàng trên | Không có: lấy Pod nào ra cũng được | Không |
| Ổn định tàu tại mỗi bước bốc dỡ | Đủ tài nguyên và đủ replica tại mỗi bước | Có |

Điểm khác cốt lõi: thứ tự trên tàu đến từ trọng lực. Thứ tự trên cluster đến từ sức chứa và
tính sẵn sàng.

### Những thứ mới phải giải

- **Giá của việc di chuyển phải vào hàm mục tiêu.** Hiện tại di chuyển không tốn gì: GA
  chuyển 43 Pod để đổi lấy 9 phần tỷ fitness (`624db06`). Có lộ trình rồi thì đổi chác như
  vậy trở nên vô lý. Có thể cần một hạng phạt theo số Pod phải chuyển, hoặc giới hạn cứng
  "tối đa k lần chuyển".
- **Trạng thái trung gian phải hợp lệ.** Pod Kubernetes không live-migrate như VM, nên có
  hai cách: xóa trước rồi tạo sau thì có khoảng gián đoạn; tạo trước rồi xóa sau thì tạm
  thời cần chỗ cho cả hai bản. Theo tóm tắt tra cứu, tài liệu về VM cũng ghi nhận việc tạm
  thời chiếm tài nguyên ở cả hai đầu; bài toán này có tên là bin repacking scheduling
  ([Hermenier và cộng sự, CP 2011](https://www.doi.org/10.1007/978-3-642-23786-7_5)). Dự án
  [BtrPlace](https://github.com/btrplace/scheduler/wiki/Network-and-migrations-scheduling)
  là một cài đặt nên đọc trước khi tự thiết kế.
- **Có thể có vòng chờ.** Hai node đầy, X phải sang node của Y và Y phải sang node của X:
  không thứ tự nào làm được nếu không có chỗ tạm. `--headroom` có thể là chỗ tạm đó, và như
  vậy nó có thêm một lý do tồn tại ngoài dự phòng tải.
- **Đơn vị di chuyển chưa chắc là Pod.** Đây là ràng buộc lớn nhất Claude tìm thấy, xem mục
  tiếp theo.

### Ràng buộc từ `kuberina-forge`

`forge/internal/forgeout/link.go:9-20` ghi rõ Kubernetes không cho nói "replica 0 ở node A,
replica 1 ở node B". Vì thế `kuberina-forge out` ghim cả declaration vào một tập node, và
`kube-scheduler` chọn trong tập đó. Hệ quả:

- Với declaration nhiều replica, blueprint không kiểm soát được từng Pod lúc chạy, nên lộ
  trình theo từng Pod có thể không thực hiện được đúng như đã tính.
- Đổi `nodeAffinity` trong pod template sẽ kích hoạt rolling update của cả Deployment. Tài
  liệu Kubernetes ghi rằng rolling upgrade của Deployment và StatefulSet không bị
  PodDisruptionBudget (PDB) giới hạn
  ([Disruptions](https://kubernetes.io/docs/concepts/workloads/pods/disruptions/)). Tức là
  lộ trình có thể phải đi theo từng declaration, với chiến lược rollout của chính nó.
- Eviction API thì có tôn trọng PDB
  ([API-initiated Eviction](https://kubernetes.io/docs/concepts/scheduling-eviction/api-eviction/)),
  còn `descheduler` là công cụ có sẵn đi theo hướng này
  ([kubernetes-sigs/descheduler](https://github.com/kubernetes-sigs/descheduler)). Claude
  chưa kiểm tra cách phối hợp các cơ chế này cho từng trường hợp.

### Kuberina kiểm tra được cascading failure đến đâu

_Cascading failure_ là khi một phần hỏng làm tải dồn sang phần còn lại, khiến phần đó quá
tải và hỏng theo. Kuberina làm việc offline, nên chỉ kiểm tra được một phần:

| Thứ cần biết tại mỗi bước | Hiện có không | Ghi chú |
|---|---|---|
| Node còn đủ sức chứa theo `requests`, GPU device | Có | Dùng lại kiểm tra hiện có cho từng bước |
| Gang và affinity không bị phá | Có | Cần kiểm tra ở từng trạng thái trung gian |
| Số replica tối thiểu còn sống | Chưa | IR chưa có PDB hay `minAvailable` |
| Tải thực tế so với `requests` | Một phần | Schema `observed` đã có; chưa dùng để xếp |
| Service nào phụ thuộc service nào | Chưa | Ngoài phạm vi IR hiện tại |
| Tải dồn sang replica còn lại khi một replica rời đi | Chưa | Cần dữ liệu lưu lượng |

Vì vậy, Claude đề nghị không hứa "tránh được cascading failure". Lời hứa trung thực hơn là:
_mọi bước trong lộ trình đều qua được các kiểm tra đã khai báo_, và inspector kiểm tra lại
từng bước bằng mã độc lập, đúng như cách đang làm với blueprint.

### Giữ đúng ranh giới của ADR-0001

[ADR-0001](../adr/0001-go-component-is-a-cli.md) và README quy định Kuberina không kết nối
cluster và không tự re-plan, vì giá trị của blueprint nằm ở việc nó là một artifact được
người xem xét. Lộ trình thay đổi không phá vỡ điều này nếu nó vẫn là: người vận hành gọi
lệnh, đầu vào là file, đầu ra là file để xem xét. Việc thực thi lộ trình trên cluster thuộc
về Kuberina Integration Platform (KIP), theo ROADMAP.

### Kiểm chứng bằng gì

File `bench/alibaba/data/openb_pod_list_default.csv` có sẵn hai cột `creation_time` và
`deletion_time`. Trong 8.152 dòng, không dòng nào để trống hai cột này. Script của simulator
FGD đang bỏ qua thời điểm xóa, theo `2026-10-31-fgd-simulator-calibration.md`, nên benchmark
hiện tại chỉ có Pod đến. Dữ liệu cho bài toán có Pod rời đi có thể đã có sẵn, nhưng Claude
mới kiểm tra sự có mặt của hai cột, chưa kiểm tra ý nghĩa và độ tin cậy của chúng. Kết quả
trên hướng này cũng không so được trực tiếp với số đã công bố của FGD.

## Câu hỏi mở

Đây là những chỗ cần Founder quyết định hoặc tranh luận thêm:

1. **Chia tầng theo cách nào:** A, B, C hay kết hợp?
1. **Tầng 2 tối ưu cái gì,** khi tập Pod đã cố định và mọi node đều đang dùng?
1. **Có mở rộng luật ADR-0003 cho mọi thuật toán tầng 2 không?** Nếu GA thua ở X-7, có thử
   SA trước khi bỏ hẳn tầng 2 không?
1. **Phạm vi lộ trình:** chỉ một bước từ A sang B, hay nhìn trước nhiều thay đổi như lịch
   cảng? Claude nghiêng về làm một bước trước.
1. **Đơn vị di chuyển:** declaration hay Pod? Câu trả lời có thể buộc phải đổi cách
   `kuberina-forge out` ghim Pod.
1. **Tạo trước hay xóa trước:** ai khai báo, ở đâu? Có cần thêm trường mới vào IR, kể cả
   thông tin kiểu PDB, không? Thay đổi này đụng đến hợp đồng của
   [ADR-0002](../adr/0002-kuberina-ir-v1-stability-contract.md).
1. **Lời hứa về cascading failure:** chấp nhận phiên bản "qua được các kiểm tra đã khai báo"
   ở trên, hay cần mạnh hơn?
1. **Chữ "isomorphic" trong PAPER §3.1:** có đổi thành "tương đồng về cấu trúc ràng buộc"
   không? Theo AGENTS.md, khi paper nói khác repo thì paper phải được sửa.
1. **Thứ tự so với #24:** Claude đề nghị hoàn thành X-5, X-6, X-7 trước, vì X-7 quyết định số
   phận của GA và ảnh hưởng trực tiếp đến ý 2.

## Thí nghiệm nhỏ có thể làm trước

Thí nghiệm này không cần tính năng mới, và cho biết bài toán "chuyển từ A sang B" nặng đến
đâu trong thực tế:

- Giải trên một tiền tố của trace, rồi giải lại khi thêm hoặc bớt một ít Pod, và đếm số Pod
  đổi node giữa hai blueprint.
- Làm với cả walk và GA. Mỗi lần walk mất khoảng 17 giây (`624db06`), nên rất rẻ.
- Giả thuyết: khi chỉ thêm Pod, walk giữ nguyên gần hết vị trí vì đi theo thứ tự đến, còn GA
  xáo trộn nhiều hơn. Khi có Pod rời đi, walk có thể xáo trộn mạnh. Nếu đúng, chỉ cần so
  sánh hai blueprint đã có thể cho số liệu cụ thể để quyết định câu hỏi 4 và 5.

## Tài liệu liên quan

- [ADR-0001: Thành phần Go là CLI](../adr/0001-go-component-is-a-cli.md)
- [ADR-0003: GA chỉ ở lại nếu thắng walk](../adr/0003-ga-must-beat-the-walk.md)
- [Session record về scale check](../../references/sessions/2026-11-03-alibaba-scale-check.md)
- [Kế hoạch #24](../../plans/benchmarks/alibaba-gpu-v2023/00_plan.md)
