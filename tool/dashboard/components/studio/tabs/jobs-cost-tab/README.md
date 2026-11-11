# `jobs-cost-tab/`

Module tab **Hàng đợi & Chi phí (`JobsCostTab`)** thuộc khung phải Studio.

## Cấu trúc
- `index.tsx`: Điều phối store (`useStudioStore` selector hẹp + `useSocialStore(s => s.openComposer)`), lọc theo tab trạng thái và gắn 3 khối giao diện.
- `types.ts`: Kiểu nội bộ (`FilterTab`, `CombinedJobItem`, `JobNodeChip`).
- `utils/mergeJobsList.ts`: Hàm thuần không phụ thuộc React — gộp 3 nguồn job (Core Runs, Legacy Renders, StudioStore), lọc theo `dismissedIds`, đếm bộ lọc, trích xuất đường dẫn media và tính phần trăm ngân sách ngày.
- `hooks/useJobsPolling.ts`: Đồng bộ `/runs`, `/video/renders`, `/outputs` với exponential backoff khi rảnh và lắng nghe `visibilitychange`; lưu danh sách `dismissed` trong `localStorage`.
- `hooks/useJobActions.ts`: Huỷ job, xoá job (qua `@/lib/confirm`), xoá tất cả job đã kết thúc.
- `components/`: `BudgetSummaryCard.tsx`, `ReadyOutputsSection.tsx`, `JobsQueueSection.tsx`, `JobQueueCard.tsx`.

## Người gọi & Tương thích ngược
- Re-export tại `components/studio/tabs/JobsCostTab.tsx` để giữ nguyên import tại `RightPanel.tsx`.
- Dùng `usePreview.getState().open(...)` mở xem video thành phẩm và `useSocialStore.openComposer(...)` chuyển sang tab `publish`.

## Bẫy đã biết
- Giữ `dismissKey = socnhotv:dismissedJobs:${activeProjectId}` ổn định vì lưu dài hạn ở `localStorage` của trình duyệt người dùng.
- Giữ `data-testid="output-play"` trên nút Xem trong `ReadyOutputsSection` (thuộc DOM gốc trước khi tách).
