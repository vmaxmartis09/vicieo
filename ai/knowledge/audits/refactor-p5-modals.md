# BÁO CÁO HOÀN THÀNH GÓI 5: MODAL LỚN, TOOLBOX VÀ MENU TÀI KHOẢN (P5-MODALS)
> **Tài liệu kiểm định:** `ai/knowledge/audits/refactor-p5-modals.md`  
> **Gói thực hiện:** Gói 5 (`ToolboxPanel`, `CreateVideoModal`, `PublishCenterModal`, `UserBubble`)  
> **Thời gian:** 2026-11-09  
> **Trạng thái:** HOÀN THÀNH 100% (4/4 TỆP MỤC TIÊU, ĐẠT TOÀN BỘ CHECKLIST D.2)

---

## 1. Tổng hợp trước / sau từng tệp

| Tệp gốc | Trước (LOC) | Barrel mới (LOC) | Thư mục tính năng & Tệp lớn nhất | Commit |
| :--- | :---: | :---: | :--- | :---: |
| `components/toolbox/ToolboxPanel.tsx` | 1.096 | 3 | `components/toolbox/panel/` — 5 workspaces riêng + 4 kho vault + `shared/` (`ToolboxErrorBoundary`, max 189 LOC ở `index.tsx`) | `9341d463c` |
| `components/studio/CreateVideoModal.tsx` | 1.226 | 3 | `components/studio/create-video-modal/` — 3 bước (`Step1`..`Step3`, max 245 LOC) + 4 hooks (max 134 LOC) + `ModalErrorBoundary` | `67df3fcba` |
| `components/social/PublishCenterModal.tsx` | 1.188 | 4 | `components/social/publish-center-modal/` — 5 bước (`Step1`..`Step5`, max 225 LOC) + 2 hooks (max 117 LOC) + pure `computeCompatibilityMatrix` + `ModalErrorBoundary` | `3e71bf996` |
| `components/navigation/UserBubble.tsx` | 683 | 4 | `components/navigation/user-bubble/` — Trigger + Popover (max 225 LOC) + 4 section panels + 2 hooks (max 115 LOC) + `BubbleErrorBoundary` | `b6e05dcb5` |

### Quy chuẩn kiến trúc đã đạt
- Toàn bộ component hiển thị đều **≤ 245 dòng** (ngưỡng ≤ 250).
- Toàn bộ custom hook đều **≤ 134 dòng** (ngưỡng ≤ 150).
- Toàn bộ hàm thuần (`computeCompatibilityMatrix`, `getInitials`, `estimateLocalStorageBytes`, `formatStorageKb`, `buildCommitTargetSpecs`) tách riêng thành file `.ts` không phụ thuộc React.
- Mọi thư mục mới đều có `README.md` mô tả vai trò, luồng dữ liệu, người gọi và bẫy đã biết.
- Giữ nguyên 100% đường dẫn import cũ nhờ file barrel re-export.
- Cô lập lỗi bằng `ErrorBoundary` riêng (`ToolboxErrorBoundary`, `ModalErrorBoundary`, `BubbleErrorBoundary`).

---

## 2. Bộ test đặc tả hành vi đã bổ sung (Test First)

Tất cả test được viết và xác minh **XANH trên code chưa tách** trước khi tách từng tệp:

1. `test/p5-modals/ToolboxPanel.test.tsx` (4 tests):
   - Chuyển đủ 5 workspace xử lý media (`BgRemove`, `Audio`, `Depth`, `Scene`, `VoiceClone`).
   - Chuyển qua lại các tab kho lưu trữ (Cutouts / Audio / Voices) và tải danh sách.
   - Cô lập lỗi render bằng `ToolboxErrorBoundary` và khôi phục bằng nút thử lại.
2. `test/p5-modals/CreateVideoModal.test.tsx` (3 tests):
   - Đi đủ 3 bước wizard, giữ nguyên giá trị form khi quay lại Bước 2, chạy AI script + QA lint + submit tạo video.
   - Chế độ tạo hàng loạt đếm đúng số dòng chủ đề và giữ thông tin sản phẩm.
   - Cô lập lỗi render bằng `ModalErrorBoundary`.
3. `test/p5-modals/PublishCenterModal.test.tsx` (4 tests):
   - Chạy đủ 5 bước lập kế hoạch: lọc video, chọn/bỏ tài khoản & xem ma trận tương thích, sinh caption AI + `PhonePreview`, xem trước lịch & đổi chiến lược sang `Đăng ngay`, xác nhận commit plan.
   - Chuyển sang tab `Xem Lịch Đăng` (`CalendarView`).
   - Kiểm thử hàm thuần `computeCompatibilityMatrix` cho OAuth hết hạn, video > 600s và cảnh báo TikTok 16:9.
   - Cô lập lỗi render bằng `ModalErrorBoundary`.
4. `test/p5-modals/UserBubble.test.tsx` (3 tests):
   - Mở popover, đồng bộ `/health`, `/video/templates`, `/settings/api-keys`, đổi ngôn ngữ/theme, lưu API key mới (không để focus trap cướp focus khi gõ), xác nhận xóa dữ liệu cục bộ 2 bước.
   - Hỗ trợ `placement="beside"` mở thanh chỉnh sửa hồ sơ inline và đóng khi nhấn `Escape`.
   - Kiểm thử `getInitials`, `formatStorageKb` và `BubbleErrorBoundary`.

---

## 3. Kết quả kiểm tra xác thực cuối cùng (Checklist D.2)

- `npx tsc --noEmit --incremental false`: **0 lỗi kiểu dữ liệu**.
- `npx vitest run`: **8/8 files, 35/35 tests PASS (100% XANH)** — riêng Gói 5 đóng góp **14/14 tests**.
- `npm run -s check:design`: **0 vi phạm trên 194 tệp được quét**.
- `NEXT_DIST_DIR=.next-build-p5 npx next build`: **Compiled successfully (8.0s)**; đã dọn thư mục build tạm `.next-build-p5` và giữ nguyên `tsconfig.json`.

---

## 4. Ghi chú điều phối / Đề xuất cho gói khác

- **Lazy-load `PublishCenterModal` tại `SocialHub.tsx` (ĐÃ THỰC HIỆN):** `components/social/hub/SocialHub.tsx:29` đã được chuyển từ import tĩnh sang `next/dynamic(..., { ssr: false })` đồng bộ với `PublishTab` và kèm test đặc tả tại `test/p5-modals/SocialHub.test.tsx`.
- **Lỗi phát hiện & đã xử lý trong quá trình tách `UserBubble`:** Test đặc tả phát hiện `useUserBubbleFocusTrap` nhận callback inline từ `index.tsx` làm `useEffect` chạy lại mỗi phím gõ và giật focus khỏi `<input>`; đã cố định bằng `callbacksRef` để effect chỉ chạy theo `[open, editingKeyProvider]`.
