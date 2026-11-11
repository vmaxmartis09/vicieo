# BÁO CÁO KIỂM TOÁN VÀ NGHIỆM THU: SESSION 1 (s1-backend-engine)
## Triển Khai Engine Giải Tham Chiếu Động & Danh Mục Dữ Liệu Backend

> **Ngày thực hiện:** 2026-11-09  
> **Phiên làm việc:** Session 1 / 3 song song theo kế hoạch `ai/plans/dynamic-data-nodes.md`  
> **Trạng thái:** HOÀN THÀNH 100% (XANH 74/74 TEST VECTOR VÀ TÍCH HỢP)

---

### 1. Phạm Vi Sở Hữu Tệp Độc Quyền (Zero Conflict)
Tuân thủ tuyệt đối ma trận phân quyền trong `ai/plans/dynamic-data-nodes.md` Mục I.2:
- `tool/src/socnhotv/core/contracts/types.py`: Mở rộng tương thích ngược `Graph.schema_version` (mặc định 1), `Graph.variables` (dict), `NodeDefinition.slots` (list).
- `tool/src/socnhotv/core/data/**`: Gói mới độc quyền 100%:
  - `contracts.py`: Mô hình Pydantic chuẩn cho `ValueSlotSpec`, `SlotZone`, `SlotMode`, `RefEnvelopeV1`, `DiagnosticCode`, `ResolveDiagnostic`, `CatalogSnapshot`, `ResolveContext`, `NodeResolveBundle`, `infer_slots_from_node_def`.
  - `security.py`: Cơ chế Sandbox an toàn: chặn tuyệt đối định danh nhạy cảm (`api_key`, `secret`, `vault`, `token_ref`, `.auth`), chặn truy cập thoát khỏi project sandbox (`..`, `file://`), giới hạn độ sâu `MAX_REF_DEPTH = 8`, chuỗi `MAX_INTERPOLATED_CHARS = 64_000`.
  - `coercion.py`: Ma trận ép kiểu an toàn (`text`, `number`, `integer`, `boolean`, `image`, `video`, `audio`, `image[]`, `string_list`, `enum`, `model_key`, `voice_id`, `character_id`, `product_id`). Hỗ trợ tương thích `Artifact`, URI và `dialogue list` sang văn bản.
  - `lexer.py` & `parser.py`: Bộ phân tích từ vựng và cú pháp đệ quy xuống (Recursive-Descent Parser). Xử lý ký tự thoát `\{{`, chỉ số âm `[-1]`, `.length`, bộ lọc `[k=='v']`, toán tử dự phòng `??`, chuỗi nội suy hỗn hợp.
  - `legacy_adapter.py`: Bộ chuyển đổi tương thích ngược 100% các cú pháp cũ: `{<node title>.<port>}`, `{model.<key>}`, `{Kịch bản.cảnh_N...}`, `{<char name>.promptLock}`, `{product.*}`, `{"$ref": "script", ...}` sang canonical URL v1 `{{$ns.id.path}}`.
  - `catalog.py`: `CatalogBuilder` thu thập đủ 11 nhóm dữ liệu dự án thật (`char`, `script`, `product`, `voice`, `model`, `project`, `run`, `social`, `policy`, `var`). Loại bỏ sạch 100% bí mật OAuth/token từ Social Accounts, sinh mã băm `version_hash` chuẩn hóa.
  - `resolver.py`: Engine giải tham chiếu với liên kết muộn (late-binding), phát hiện chu trình phụ thuộc (`REF_CYCLE_DETECTED`), hàm tiện ích `resolve_node`.
- `tool/src/socnhotv/core/engine/validator.py`: Tích hợp quét phụ thuộc ngầm `$node` trong params/slots, kiểm tra chu trình và bổ sung cạnh vào sắp xếp topo DAG.
- `tool/src/socnhotv/core/engine/graph_runner.py`:
  - Giải muộn `resolve_node` trước khi thực thi node.
  - Tạo bản sao tạm `exec_node` chứa `params` đã giải, giữ nguyên 100% `graph` gốc trong bộ nhớ và lưu trữ.
  - Bảo vệ ngân sách người dùng bằng cơ chế **Fault Isolation**: dừng node an toàn khi tham chiếu hỏng không có fallback, phát sự kiện SSE trạng thái FAILED.
  - Lưu bản chụp `runs/<run_id>/resolved_inputs.json` phục vụ tái lập và kiểm toán provenance.
  - Điều khiển kích hoạt qua biến môi trường `SOCNHOTV_DYNAMIC_SLOTS` (mặc định bật).
- `tool/src/socnhotv/core/api/routes/data_catalog.py`:
  - `GET /projects/{project_id}/data-catalog`: Trả về `CatalogSnapshot` kèm `versionHash`, hỗ trợ `If-None-Match` -> `304 Not Modified`.
  - `POST /projects/{project_id}/resolve-preview`: Hỗ trợ xem trước biểu thức đơn, kiểm tra từng node hoặc ước tính chi phí toàn bộ đồ thị theo `mode="estimate"`.
- `tool/src/socnhotv/core/api/app.py`: Tự động nạp route `data_catalog` và bổ sung suy diễn `slots` ở endpoint `/node-types`.
- `tool/shared/test-vectors/dynamic-refs-v1.json`: 68 ca kiểm thử dùng chung giữa Backend Python và Dashboard TypeScript.
- Bộ test tự động:
  - `tool/tests/test_dynamic_resolver_vectors.py` (68/68 passed)
  - `tool/tests/test_data_catalog.py` (3/3 passed)
  - `tool/tests/test_graph_runner_dynamic.py` (3/3 passed)

---

### 2. Kết Quả Kiểm Thử (Verification Summary)

```bash
tool/.venv/bin/pytest tool/tests/test_dynamic_resolver_vectors.py tool/tests/test_data_catalog.py tool/tests/test_graph_runner_dynamic.py -q
74 passed in 1.46s

tool/.venv/bin/pytest tool/tests/test_core_step1.py tool/tests/test_core_step2_graph_runner.py tool/tests/test_graph_inputs.py -q
20 passed in 23.42s
```
- **Tổng cộng 94 test cases xanh 100%**.
- Sandbox kiểm toán xác nhận: `SOCNHOTV: không có mục mới trong data/projects/*/runs thật.`
- Không có bất kỳ tệp dữ liệu nào trong `data/` hoặc tệp frontend trong `tool/dashboard/` bị thêm vào commit của Session 1.
