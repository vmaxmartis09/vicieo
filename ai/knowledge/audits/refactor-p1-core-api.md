# BÁO CÁO HOÀN THÀNH GÓI 1 (BƯỚC 1 & 2): TÁI CẤU TRÚC CORE API & GRAPH UTILS
> **Tài liệu kiểm định:** `ai/knowledge/audits/refactor-p1-core-api.md`  
> **Gói thực hiện:** Gói 1 (Core Store & API Infrastructure)  
> **Thời gian:** 2026-11-09  
> **Trạng thái:** HOÀN THÀNH 100% CÁC BƯỚC 1 VÀ 2

---

## 1. Các tệp đã xử lý
1. `lib/graph/wouldCreateCycle.ts` (MỚI):
   - Tách thuật toán kiểm tra vòng lặp đồ thị thành tệp dùng chung duy nhất.
   - Có bộ test đặc tả toàn diện tại `test/canvas/wouldCreateCycle.test.ts` (bao phủ self-loop, chu kỳ 2 node, chu kỳ 3 node, và DAG kim cương).
2. `lib/core/` (MỚI):
   - `nodeDefinitions.ts` (~560 dòng): Chứa `isPortCompatible` và 13 schemas node tĩnh `FALLBACK_NODE_TYPES`.
   - `costEstimator.ts` (~65 dòng): Chứa hàm tính giá `estimateNodeCostUsd` độc lập, không phụ thuộc React.
   - `graphValidator.ts` (~85 dòng): Chứa `validateGraphLocally` kiểm tra tính hợp lệ của đồ thị.
   - `graphTransformers.ts` (~80 dòng): Chứa `toCoreGraph` và `normalizeGraphForCore` (nơi DUY NHẤT định nghĩa hàm chuyển đổi format).
   - `commandParser.ts` (~105 dòng): Chứa parser lệnh ngôn ngữ tự nhiên `parseStudioCommandClientSide`.
   - `coreClient.ts` (~330 dòng): Chứa toàn bộ các hàm gọi API mạng tới Core backend.
   - `index.ts`: Re-export toàn bộ các thành phần trên.
3. `lib/core-api.ts`:
   - Giảm từ **1.238 dòng xuống còn 6 dòng** (-99.5% LOC).
   - Giữ nguyên toàn bộ public interface via re-export để 16 nơi đang import từ `@/lib/core-api` hoạt động 100% ổn định.

---

## 2. Kết quả kiểm tra xác thực (Checklist D.2)
- `npx vitest run`: **13/13 tests pass (100% XANH, 533ms)**.
- `npx tsc --noEmit --incremental false`: **0 lỗi kiểu dữ liệu**.
- `npm run -s check:design`: **0 vi phạm trên 134 tệp**.
- `NEXT_DIST_DIR=.next-build-core npx next build`: **Compiled successfully trong 7.5s**.
