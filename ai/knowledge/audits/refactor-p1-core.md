# BÁO CÁO HOÀN THÀNH GÓI 1: STORE LÕI VÀ API (P1-CORE)
> **Tài liệu kiểm định:** `ai/knowledge/audits/refactor-p1-core.md`  
> **Gói thực hiện:** Gói 1 (Store Slices & API Clients)  
> **Thời gian:** 2026-11-09  
> **Trạng thái:** HOÀN THÀNH CÁC FILE CHÍNH CỦA GÓI 1 (100% ĐẠT KIỂM ĐỊNH)

---

## 1. Tổng hợp tái cấu trúc các tệp lõi
1. **`lib/graph/wouldCreateCycle.ts`** (MỚI):
   - Tách hàm kiểm tra vòng lặp đồ thị thành tệp thuật toán dùng chung duy nhất (`README.md` đã bổ sung).
   - Đã có test bao phủ 5 tình huống (`test/canvas/wouldCreateCycle.test.ts`).
2. **`lib/core-api.ts` (1.238 LOC -> 6 LOC)** -> **`lib/core/`**:
   - `nodeDefinitions.ts` (~560 dòng), `costEstimator.ts` (~65 dòng), `graphValidator.ts` (~85 dòng), `graphTransformers.ts` (~80 dòng), `commandParser.ts` (~105 dòng), `coreClient.ts` (~330 dòng), `index.ts`, `README.md`.
   - Bảo toàn 16 nơi import nhờ re-export.
   - Đã mở rộng `test/canvas/coreApi.test.ts` (10 tests): bổ sung kiểm thử `toCoreGraph` (lọc `templateGroup`, cộng toạ độ con theo `parentId`), `normalizeGraphForCore` (`succeeded` -> `completed`, bù `nodeId`/`artifactId`) và `parseStudioCommandClientSide`.
3. **`lib/api/social.ts` (982 LOC -> 6 LOC)** -> **`lib/api/social/`**:
   - `types.ts` (~520 dòng), `client.ts` (~25 dòng), `endpoints/` (8 files: `accounts`, `analytics`, `automations`, `calendar`, `hub`, `posts`, `publishing`, `videos`), `index.ts`, `README.md`.
   - Bảo toàn 19 nơi import nhờ re-export.
   - Đã mở rộng `test/social/socialApi.test.ts` (3 tests): kiểm thử URL/query/HTTP method/JSON body qua MSW trên cả 8 nhóm endpoint.
4. **`store/slices/canvasSlice.ts` (1.388 LOC -> 6 LOC)** -> **`store/slices/canvas/`**:
   - `types.ts` (~195 dòng): toàn bộ kiểu dữ liệu và cấu hình collapse.
   - `templates/socNhoStoryTemplate.ts` (~305 dòng): kịch bản mặc định và bộ tạo template mẫu.
   - `utils.ts` (~60 dòng): `collectDownstreamIds`, `syncProjectWorkspace`, `inFlightGraphLoads`.
   - `subSlices/` (4 files): `projectGraphSlice.ts`, `nodesEdgesSlice.ts`, `castWorkflowSlice.ts`, `historyExecutionSlice.ts`.
   - `index.ts`, `README.md`: Khởi tạo `createCanvasSlice` và re-export toàn bộ kiểu/hàm để bảo toàn 10+ nơi import.
   - Đã mở rộng `test/canvas/canvasSlice.test.ts` (8 tests): bổ sung kiểm thử thêm/xoá node qua `setNodesAndEdges` + undo/redo và cơ chế `_bypass` đánh dấu `isStale` hạ nguồn khác biệt với `_collapsed`.
5. **`store/modules/social.ts` (519 LOC -> 1 LOC)** -> **`store/modules/social/`**:
   - `types.ts` (147 dòng): `HubView`, `HubQueueBucket`, `SocialFilters`, `SocialState`, typings `SocialStoreSet`/`SocialStoreGet`.
   - `utils/socialHelpers.ts` (42 dòng pure TS): `countUnreadNotifications`, `findTargetScheduledTime`, `applyTargetTimeToCalendar`, `applyTargetTimeToPosts`.
   - `actions/` (4 files, max 139 dòng): `viewActions.ts` (70 dòng), `hubDataActions.ts` (132 dòng), `analyticsAutomationActions.ts` (139 dòng), `schedulingActions.ts` (46 dòng).
   - `index.ts` (63 dòng) + `README.md`: ghép 4 nhóm action vào `useSocialStore` và re-export giữ nguyên 100% đường dẫn import (13+ nơi dùng).
   - Đã có bộ test hành vi + pure helpers (`test/social/socialStore.test.ts`, 4 tests viết trước trên mã gốc).

---

## 2. Kết quả kiểm tra xác thực (Checklist D.2)
- `npx vitest run`: **17/17 files, 73/73 tests pass (100% XANH)** (riêng Gói 1: `coreApi` 10, `wouldCreateCycle` 5, `canvasSlice` 8, `socialApi` 3, `socialStore` 4 = **30 tests**).
- `npx tsc --noEmit --incremental false`: **0 lỗi kiểu dữ liệu**.
- `npm run -s check:design`: **0 vi phạm trên 275 tệp**.
- `NEXT_DIST_DIR=.next-build-p1 npx next build`: **Compiled successfully trong 7.2s**; đã xoá `.next-build-p1` và giữ nguyên `tsconfig.json`.
