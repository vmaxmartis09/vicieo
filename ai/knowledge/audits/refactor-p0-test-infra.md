# BÁO CÁO HOÀN THÀNH GÓI 0: HẠ TẦNG TEST (P0-TEST-INFRA)
> **Tài liệu kiểm định:** `ai/knowledge/audits/refactor-p0-test-infra.md`  
> **Gói thực hiện:** Gói 0 (Hạ tầng kiểm thử cho `tool/dashboard`)  
> **Thời gian:** 2026-11-09  
> **Trạng thái:** HOÀN THÀNH 100% – SẴN SÀNG CHO CÁC GÓI TIẾP THEO

---

## 1. Các thành phần đã triển khai
1. **DevDependencies đã cài đặt:**
   - `vitest` v5.1.0
   - `@testing-library/react` v16.3.3
   - `@testing-library/jest-dom` v7.0.1
   - `@testing-library/user-event` v14.6.8
   - `@vitejs/plugin-react` v6.3.0
   - `jsdom` v31.0.0
   - `msw` v3.0.4
   - `@playwright/test` v1.65.0
2. **Cấu hình kiểm thử:**
   - `tool/dashboard/vitest.config.mts`: thiết lập môi trường `jsdom`, alias `@/` trỏ về root của dashboard, include `test/**/*.test.{ts,tsx}`.
   - `tool/dashboard/test/setup.ts`: giả lập toàn bộ các API trình duyệt cần thiết cho `@xyflow/react` (`ResizeObserver`, `DOMMatrixReadOnly`, `matchMedia`, `IntersectionObserver`) và import `@testing-library/jest-dom`.
   - `tool/dashboard/playwright.config.ts`: cấu hình Playwright cho visual regression testing.
3. **Tiện ích dùng chung (`test/_shared/`):**
   - `tool/dashboard/test/_shared/testData.ts`: 5 node mẫu (`sample5Nodes`) và edges mẫu (`sampleEdges`).
   - `tool/dashboard/test/_shared/mswServer.ts`: máy chủ mock MSW mặc định cho network tests.
   - `tool/dashboard/test/_shared/profilerHelper.ts`: bộ thu thập `<Profiler>` (`createProfilerCollector`, `ProfiledRegion`) phục vụ kịch bản đo Mục E.3 (số lần commit, pha `mount`/`update`, tổng `actualDurationMs`) trước & sau khi tách `StudioCanvas`/`StudioNode`.
4. **Bộ test đặc tả mẫu đã chạy:**
   - `tool/dashboard/test/canvas/coreApi.test.ts`: 10 test cases bao phủ `isPortCompatible`, `estimateNodeCostUsd`, `validateGraphLocally`, `toCoreGraph`, `normalizeGraphForCore`, `parseStudioCommandClientSide`.
5. **Playwright Visual Characterization & Determinism Suite (`e2e/`, Mục D.1.4 & K.2):**
   - `tool/dashboard/playwright.config.ts`: cấu hình cố định viewport `1440x900`, `deviceScaleFactor: 1`, `reducedMotion: "reduce"`, `vi-VN`, `Asia/Ho_Chi_Minh`; tự nhận `chromium.executablePath()` hoặc fallback sang Google Chrome cài sẵn trên macOS (`/Applications/Google Chrome.app/Contents/MacOS/Google Chrome`) vì `~/.cache`/`~/Library/Caches/ms-playwright/chromium*` chưa tải nhị phân riêng.
   - `tool/dashboard/e2e/mockRoutes.ts`: chặn và trả fixture cố định (5 node + 5 cạnh, mẫu video, danh sách video/tài khoản social, SSE `: connected`, PNG 1x1 trong suốt) cho cả `:8000` (Core) và `:8103` (Social), bảo đảm không chạm backend thật hay `data/`.
   - `tool/dashboard/e2e/run-visual.mjs` + `npm run test:e2e:visual`: build cô lập vào `.next-build-e2e`, khởi chạy `next start -p 3019`, chạy `playwright test`, tự dọn `.next-build-e2e` và khôi phục `tsconfig.json` nếu bị Next thêm `.next-build-e2e`.
   - `tool/dashboard/e2e/canvas-visual.spec.ts` + `e2e/__screenshots__/*.png`: chụp 2 lần liên tiếp (isolated browser contexts) qua 5 màn hình, hỗ trợ `UPDATE_BASELINES=1` để ghi ảnh mốc và mặc định so cả `runToRun` lẫn `vsBaseline`.

---

## 2. Kết quả đo Playwright Visual Determinism (Mục K.2)

- **Phát hiện thực nghiệm trong lần đo đầu (chỉ đợi 2 `rAF` + CSS freeze):**
  - `create-video-modal-step1` sai khác **9.431 / 1.296.000 px (`0.7277%`)** (vượt ngưỡng `0.2%`) dù CSS animations đã tắt. Nguyên nhân thực tế: `framer-motion` chạy tween JS bằng `requestAnimationFrame` (`initial={{ opacity: 0, scale: 0.96, y: 8 }}`) và không bỏ qua tween dưới `prefers-reduced-motion` khi thiếu `<MotionConfig reducedMotion="user">`. Các bước sau (`step2`, `step3`, `publish-center-modal`) mở muộn hơn nên đã kết thúc tween và đạt **0 px**.
- **Sau khi thêm bước chờ ổn định Framer Motion (`450ms` trước 2 `rAF` trong `settleUi`):**

| Màn hình (`1440x900`) | Run 1 vs Run 2 (`runToRun`) | So ảnh mốc (`vsBaseline`) | Kết luận ngưỡng `≤ 0.2%` |
| :--- | :---: | :---: | :---: |
| `studio-canvas-5nodes` | `11 px` (`0.0008%`) | `0 px` (`0.0000%`, byte-identical) | ĐẠT |
| `create-video-modal-step1` | `0 px` (`0.0000%`, byte-identical) | `0 px` (`0.0000%`, byte-identical) | ĐẠT |
| `create-video-modal-step2` | `0 px` (`0.0000%`, byte-identical) | `0 px` (`0.0000%`, byte-identical) | ĐẠT |
| `create-video-modal-step3` | `0 px` (`0.0000%`, byte-identical) | `0 px` (`0.0000%`, byte-identical) | ĐẠT |
| `publish-center-modal` | `0 px` (`0.0000%`, byte-identical) | `2 px` (`0.0002%`) | ĐẠT |

- **Khuyến nghị chốt Quyết định K.2:** Giữ ngưỡng **`<= 0.2%`** kèm backend giả lập qua `page.route` (`mockRoutes.ts`). Sai số thực đo tối đa sau settle chỉ là **`0.0008%`** (thấp hơn ngưỡng >250 lần), đủ chặt để bắt lệch layout thực sự mà không bị nhiễu subpixel.

---

## 3. Kết quả kiểm tra xác thực (Checklist D.2)
- `npx vitest run`: **18/18 files, 74/74 tests PASS (100% XANH, 3.27s)**.
- `npm run test:e2e:visual`: **1/1 suite PASS (11.5s)**, `.next-build-e2e` đã dọn sạch, `tsconfig.json` không bị sửa.
- `npx tsc --noEmit --incremental false`: **0 lỗi kiểu dữ liệu**.
- `npm run -s check:icons`: **188/188 exports hợp lệ**.
- `npm run -s check:design`: **0 vi phạm trên 275 tệp**.

---

## 4. Hướng dẫn cho các gói tiếp theo
Mọi gói tiếp theo có thể:
1. Dùng `npx vitest run` cho unit/component tests trong `tool/dashboard/test/<tên-gói>/**` và `test/_shared/profilerHelper.ts` cho phép đo `<Profiler>` (Mục E.3).
2. Chạy `npm run test:e2e:visual` sau các commit tách UI để kiểm tra hồi quy hình ảnh trước khi merge (`UPDATE_BASELINES=1 npm run test:e2e:visual` khi chủ đích cập nhật ảnh mốc sau một thay đổi UI đã hợp nhất).
