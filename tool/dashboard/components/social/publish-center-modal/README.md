# publish-center-modal

Module điều phối Trung tâm Quản lý & Xuất bản Social 5 bước và chế độ xem lịch đăng (`CalendarView`).

## Luồng dữ liệu & Cấu trúc

- `index.tsx`: Shell modal (`PublishCenterModal`), bọc bởi `ModalErrorBoundary`, điều phối chuyển tab (`plan` / `calendar`) và 5 bước wizard.
- `hooks/usePublishPlanSelection.ts`: Tải danh sách video/tài khoản từ `socialApi` và `useSocialStore`, lọc video và tính `compatibilityMatrix`.
- `hooks/usePublishPlanWorkflow.ts`: Quản lý sinh caption AI, xem trước lịch tự động và xác nhận lên lịch (`commitPlan`).
- `utils/compatibilityMatrix.ts`: Hàm thuần tính ma trận tương thích Video × Tài khoản (không phụ thuộc React).
- `api.ts`: Đóng gói lời gọi `socialApi.generateCaptions`, `previewPlan`, `buildCommitTargetSpecs`.
- `components/`: `ModalHeader`, `ModalErrorBoundary`, `Step1VideoSelector` .. `Step5ReviewCommit`.

## Ai gọi

- Re-export qua `components/social/PublishCenterModal.tsx` để bảo toàn import tại `PublishTab.tsx` (dynamic) và `SocialHub.tsx`.

## Bẫy đã biết

- Khi bước nhảy sang 4 (`currentStep === 4`) hoặc đổi `scheduleStrategy`, hook tự gọi `handleLoadPreview()`; cần giữ dependency list gọn tránh gọi lặp khi người dùng chỉnh giờ thủ công từng dòng.
- Toàn bộ icon trong wizard dùng Morphicons qua `../icons` (`components/social/icons.ts`).
