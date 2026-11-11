# `lib/core/` — Lõi API, Chi phí & Đồ thị Core Backend

- **Vai trò:** Tách từ `lib/core-api.ts` (1.238 LOC → barrel re-export `6` LOC) thành 6 module chuyên biệt cho định nghĩa node, tính giá, kiểm tra/biến đổi đồ thị, phân tích lệnh thử nghiệm và client HTTP tới Core Backend.
- **Luồng dữ liệu:**
  1. `nodeDefinitions.ts` cung cấp `FALLBACK_NODE_TYPES` & metadata mặc định khi Core offline.
  2. `costEstimator.ts` (`estimateNodeCostUsd`, `estimateVideoCostUsd`) tính chi phí dự kiến (trả `0` ở chế độ mock).
  3. `graphValidator.ts` (`validateGraphLocally`, `isPortCompatible`) kiểm tra đồ thị cục bộ trước khi chạy.
  4. `graphTransformers.ts` (`toCoreGraph`, `normalizeGraphForCore`) chuyển đổi giữa định dạng React Flow (`nodes`, `edges`) và `Core Graph` hợp đồng backend.
  5. `commandParser.ts` (`parseStudioCommandClientSide`) mô phỏng parser lệnh chat offline/rule-based.
  6. `coreClient.ts` giao tiếp HTTP (`CORE_URL`) cho health, graph save/load, run orchestration và runs/outputs.
  7. `index.ts` & `lib/core-api.ts` re-export toàn bộ để giữ nguyên import cũ.
- **Ai gọi ai:** `StudioCanvas`, `StudioNode`, `store/slices/canvas/*`, `lib/studio-agent.ts`, `JobsCostTab`, `ChatTab`, `CreateVideoModal` → `@/lib/core-api`.
- **Các bẫy đã biết:**
  - `toCoreGraph` phải cộng bù tọa độ `parentId` khi node nằm trong nhóm mẫu (`templateGroup`), đồng thời lọc bỏ chính node `templateGroup`.
  - `normalizeGraphForCore` chuẩn hóa trạng thái cũ `"succeeded"` sang `"completed"` cho Core Backend và điền `nodeId`/`artifactId` còn thiếu.
