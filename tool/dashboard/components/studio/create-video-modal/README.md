# CreateVideoModal Module (`components/studio/create-video-modal/`)

- **Vai trò:** Wizard 3 bước tạo video tự động trong Studio (chọn mẫu -> nhập/sinh kịch bản & craft -> chọn kênh/ước tính chi phí & khởi chạy đồ thị).
- **Luồng dữ liệu:** `useVideoCreationForm` kết hợp `useCreateVideoData` (nạp templates, presets, trạng thái OAuth) và `useVideoCreationWorkflow` (gọi `api.ts` sinh kịch bản, hook A/B/C, QA lint, estimate cost và đẩy batch graph lên store/Core).
- **Ai gọi ai:** `StudioCanvas.tsx` dynamic-import `@/components/studio/CreateVideoModal`, file cũ chỉ re-export `CreateVideoModal` từ `./create-video-modal`.
- **Cấu trúc:**
  - `index.tsx`: Shell modal bọc trong `ModalErrorBoundary`, header 3 bước và footer điều hướng.
  - `steps/`: `Step1TemplatePicker` (cô lập hover video preview state cục bộ), `Step2ContentCraft`, `Step3PublishEstimate`.
  - `components/`: `VideoCraftSection`, `ModalErrorBoundary`.
  - `hooks/`: `useVideoCreationForm`, `useCreateVideoData`, `useVideoCreationWorkflow`.
  - `api.ts`, `types.ts`, `constants.ts`.
- **Bẫy đã biết:** Giữ các state của `useVideoCreationForm` ở mức shell để quay lại bước trước không bị mất dữ liệu đã nhập.
