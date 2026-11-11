# Toolbox Panel Module (`components/toolbox/panel/`)

- **Vai trò:** Bảng công cụ cục bộ (tách nền, canh từ STT, tách lớp độ sâu, cắt cảnh video, clone giọng) và kho lưu trữ kết quả đầu ra.
- **Luồng dữ liệu:** `ToolboxPanel` nạp danh sách tool (`action=list`), kho kết quả (`action=vaults`) và danh sách dự án/nhân vật (`action=targets`) qua `/api/toolbox`. Các workspace gọi `uploadFile` -> `runTool` -> `onDone()` để làm mới vault, sau đó cho phép lưu qua `ToolboxSaveBar`.
- **Ai gọi ai:** `FloatingPanels.tsx` và `RightPanel.tsx` lazy-load `@/components/toolbox/ToolboxPanel`, file này chỉ re-export `ToolboxPanel` từ `./panel`.
- **Cấu trúc:**
  - `index.tsx`: Container điều hướng 4 sub-tab (`tools`, `cutouts`, `audio`, `voices`) được bọc bởi `ToolboxErrorBoundary`.
  - `workspaces/`: 5 workspace độc lập (`BgRemoveWorkspace`, `AudioWorkspace`, `DepthWorkspace`, `SceneWorkspace`, `VoiceCloneWorkspace`).
  - `tabs/`: Các view hiển thị danh mục tool và 3 tab vault (`ToolsCatalogTab`, `CutoutsVaultTab`, `AudioVaultTab`, `VoicesVaultTab`).
  - `shared/`: `ToolboxDropzone`, `ToolboxSaveBar`, `ToolboxErrorLine`, `ToolboxErrorBoundary`.
  - `api.ts`, `types.ts`, `constants.tsx`.
- **Bẫy đã biết:** Không bọc `motion.div` sâu hơn 1 cấp bên trong `<AnimatePresence mode="wait">`, giữ nguyên `key` ở phần tử con trực tiếp để hiệu ứng chuyển tab hoạt động đúng.
