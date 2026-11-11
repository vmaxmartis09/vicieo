# BÁO CÁO TIẾN ĐỘ GÓI 2: NODE CANVAS (`P2-NODES`)
> **Tài liệu kiểm định:** `ai/knowledge/audits/refactor-p2-nodes.md`  
> **Gói thực hiện:** Gói 2 (`NodeBody`, `CastPreview`, `DynamicValueFlyout`; tạm hoãn `VideoGenParams`, `NodeInputs`, `StudioNode`, `PortValueInput`)  
> **Thời gian:** 2026-11-09  
> **Trạng thái:** HOÀN THÀNH 2/2 NHÓM FILE SẠCH + TẠM HOÃN AN TOÀN 4 FILE ĐANG BỊ KHÓA BỞI SỬA ĐỔI SONG SONG (H.2.2)

---

## 1. Các tệp sạch đã tách xong & kiểm chứng

| Tệp gốc | Trước (LOC) | Barrel mới (LOC) | Cấu trúc mới & Tệp lớn nhất | Commit |
| :--- | :---: | :---: | :--- | :---: |
| `components/canvas/NodeBody.tsx` + `CastPreview.tsx` | 908 + 150 | 1 + 149 | `components/canvas/node-body/` — `index.tsx` (46), `NodeBodyDispatcher.tsx` (113), `components/` (6 helpers, max 118 LOC), `renderers/` (4 nhóm họ node, max 202 LOC ở `ToolboxAndGenericRenderer`), `utils/castExtractor.ts` (76 LOC + **bounded LRU `CAST_CACHE` tối đa 32 tập**, quyết định K.4) | `47d2bee30` |
| `components/canvas/DynamicValueFlyout.tsx` | 510 | 1 | `components/canvas/dynamic-value-flyout/` — `index.tsx` (187), `hooks/` (`usePopoverDismiss` 39, `useFlyoutScriptData` 35), `tabs/` (5 tab, max 70 LOC ở `CharactersTab`), `utils/flyoutFilters.ts` (81 LOC pure TS), `constants.ts` (10), `types.ts` (35). Đổi import danh mục sang `@/lib/data/extendedModelsCatalog` (giảm First Load JS `/studio` từ 362 → 360 kB) | `6305e90ae` |

### Quy chuẩn kiến trúc đã đạt cho phần đã tách
- Toàn bộ component hiển thị đều **≤ 202 dòng** (ngưỡng ≤ 250).
- Toàn bộ hook đều **≤ 39 dòng** (ngưỡng ≤ 150).
- Logic thuần không phụ thuộc React (`castExtractor.ts`, `flyoutFilters.ts`) có unit test riêng.
- Đầy đủ `README.md` trong mỗi thư mục tính năng, Morphicons 100%, giữ nguyên 100% DOM/class Tailwind và đường dẫn import cũ.

---

## 2. Bộ test đặc tả hành vi đã bổ sung (Test First — Xanh trên code gốc trước khi tách)

1. `tool/dashboard/test/p2-nodes/NodeBody.test.tsx` (5 tests):
   - Trả `null` cho 5 node nguồn (`text`, `input`, `image`, `video`, `audio`).
   - Render không lỗi toàn bộ 24 `nodeType` (`script`, `voice_tts`, `image_gen`, `image_edit`, `video_gen`, `cast`, `product`, `character`, `captions`, `transition`, `hyperframes_render`, `export`, `policy`, `quality_check`, `vision_analyze`, `bg_remove`, `depth_layers`, `handdraw`, `scene_cut`, `voice_clone`, `word_captions`, `template_render`, `platform_export`, fallback custom).
   - `PolicyBadge`, `CastPreview` qua MSW, sửa `transition` gọi `updateNodeParams`, nút `Xem lớn` mở `usePreview`.
   - Unit test `castExtractor`: `extractCastView`, `resolveCastMembers` và kiểm chứng cơ chế đuổi LRU khi vượt `MAX_CAST_CACHE_ENTRIES = 32` (K.4).
2. `tool/dashboard/test/p2-nodes/DynamicValueFlyout.test.tsx` (4 tests):
   - Lọc bỏ node hiện tại, bấm output tạo edge qua `setNodesAndEdges`, phát `{<node>.<port>}`, đóng flyout.
   - Đủ 5 tab (`Node`, `Kịch bản` lazy-load từ `/library` + `/script-data`, `Nhân vật`, `Model`, `Biến`), tìm kiếm, `onInsertText` khi đã có text.
   - Đóng khi nhấn `Escape` hoặc click ngoài; unit test `flyoutFilters`.

---

## 3. Các tệp TẠM HOÃN theo quy tắc điều phối H.2.2

Trên `git status -s`, nhóm tệp sau đang có chỉnh sửa chưa commit từ luồng công việc song song (timestamp `2026-11-09 22:29`), nên theo **Quy tắc H.2.2** tuyệt đối không ghi đè hoặc tách đè lên khi chưa commit:

| Tệp chưa tách | LOC hiện tại | Lý do hoãn | Ghi chú khảo sát cho lượt chạy sau |
| :--- | :---: | :--- | :--- |
| `components/canvas/VideoGenParams.tsx` | 768 | Đang có `M` chưa commit (`22:29`) | Bước 1 khi sạch: viết test đổi model/ratio/duration/biến thể, tách thành `video-gen-params/` (`sections/*`, `constants.ts`). |
| `components/canvas/NodeInputs.tsx` | 1.194 | Đang có `M` chưa commit (`22:29`) | `NodeImageInput` (~326 LOC) vượt 250; cần tách sub-section upload/URL/thư viện sau khi chuyển vào `node-inputs/`. |
| `components/canvas/StudioNode.tsx` | 1.207 | Đang có `M` chưa commit (`22:29`) | Đã dùng `memo` (dòng 86, so sánh nông mặc định) nhưng đọc `activeDragPort` từ `useCanvasInteraction()` (dòng 124). Khi tách: bọc `NodeErrorBoundary` và có test cô lập lỗi. |
| `components/canvas/PortValueInput.tsx` | 684 | Đang có `M` chưa commit (`22:29`) | Còn khai báo `wouldCreateCycle` cục bộ ở dòng 59; thay bằng `@/lib/graph/wouldCreateCycle` khi sạch. |
| `components/canvas/GenericParams.tsx` | 586 | Đang có `M` chưa commit (`22:29`) | Nằm ngoài 4 file ưu tiên chính của Gói 2; kiểm tra tách sau nếu còn >400 sau `dynamic-data-nodes`. |

---

## 4. Kết quả kiểm tra xác thực cuối cùng (Checklist D.2)

- `npx tsc --noEmit --incremental false`: **0 lỗi kiểu dữ liệu**.
- `npx vitest run`: **17/17 files, 68/68 tests PASS (100% XANH)**.
- `npm run -s check:icons`: **188/188 exports hợp lệ**.
- `npm run -s check:design`: **0 vi phạm trên 275 tệp được quét**.
- `NEXT_DIST_DIR=.next-build-p2 npx next build`: **Compiled successfully (7.6s)** (`/[locale]/studio` = 96.4 kB / 360 kB First Load JS); đã xoá `.next-build-p2` và khôi phục `tsconfig.json`.
