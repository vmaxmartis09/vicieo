# `store/slices/canvas/` — Slice Quản Lý Trạng Thái Canvas & Quy Trình Tập

- **Vai trò:** Tách từ `store/slices/canvasSlice.ts` (1.388 LOC → barrel re-export `6` LOC) thành 4 sub-slices, template mẫu, hằng số/kiểu và hàm thuần.
- **Cấu trúc module:**
  - `types.ts`: `PORT_TYPE_COLORS`, `StudioNodeData`, `CollapseConfig`, `getNodeCollapseConfig`, `isNodeDefaultOpen`.
  - `templates/socNhoStoryTemplate.ts`: `DEFAULT_EPISODES`, `buildSocNhoStoryTemplate`, `genericizeTemplate`.
  - `utils.ts`: hàm thuần `collectDownstreamIds` (lan truyền `isStale` theo cổng ra bị ảnh hưởng), `syncProjectWorkspace`, `inFlightGraphLoads` (chống đua tải đồ thị trùng dự án).
  - `subSlices/projectGraphSlice.ts`: nạp/chuyển dự án, tập, custom node types.
  - `subSlices/nodesEdgesSlice.ts`: chọn node, cập nhật `params` (phát hiện patch UI-only `_collapsed` để không đánh dấu `isStale` hạ nguồn), đóng/mở hàng loạt, pin thế hệ, `markDownstreamStale`.
  - `subSlices/castWorkflowSlice.ts`: chuyển đổi qua lại giữa node `character` và `cast`.
  - `subSlices/historyExecutionSlice.ts`: undo/redo (giới hạn 25 bước), mock/run policy, nạp template (`blank` / `soc_nho_story`).
- **Ai gọi ai:** `store/index.ts` ghép `createCanvasSlice`; mọi component `StudioCanvas`, `StudioNode`, `NodeInputs`, `NodeBody`, `ChatTab`, `JobsCostTab` đọc qua selector `useStudioStore(...)`.
- **Các bẫy đã biết:**
  - Sửa `_collapsed` trong `updateNodeParams` được coi là patch giao diện thuần, không được phép làm `isStale` các node hạ nguồn.
  - Đổi tham số `_bypass` hoặc nội dung chạy thật phải lan truyền `isStale = true` tới các node `succeeded` phía sau.
