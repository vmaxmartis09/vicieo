# `lib/graph/` — Thuật Toán Đồ Thị Dùng Chung

- **Vai trò:** Chứa các hàm thuật toán đồ thị thuần TypeScript không phụ thuộc React hay Store UI.
- **Thành phần:** `wouldCreateCycle(edges, sourceId, targetId)` (26 LOC) — duyệt DFS xuôi từ `targetId` để phát hiện nối ngược/tự nối gây vòng lặp DAG.
- **Ai gọi ai:** Đang chuẩn hóa thành nguồn duy nhất dùng chung cho `StudioCanvas` (Gói 3) và `PortValueInput` (Gói 2); kiểm thử trực tiếp tại `test/canvas/wouldCreateCycle.test.ts`.
- **Các bẫy đã biết:** Khác hướng kiểm tra cạnh đơn chiều: cần phát hiện cả `sourceId === targetId` (tự nối vòng).
