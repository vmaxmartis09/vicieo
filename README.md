# ViciEO: Lightweight High-Performance Multiplatform AI Video Studio

> **Dự án kế thừa và tiến hóa từ socnhotv**, hướng tới kiến trúc hiện đại, đa nền tảng, siêu nhẹ và tối ưu hiệu năng.

---

## 1. TỔNG QUAN KIẾN TRÚC MỚI

1. **`engine-rust/`**: Core Native Engine được viết bằng **Rust**.
   - **Kích thước cực nhỏ:** Single executable binary (~15MB), không cần Python runtime hay virtualenv.
   - **Tốc độ:** Khởi động tức thì (<0.01s), RAM <25MB.
   - **Xử lý bất đồng bộ & đa luồng:** Tận dụng `tokio` để điều phối pipeline render, xử lý asset song song.
   - **Schema hợp đồng 2 tầng (Two-Tier Schema):** Kết hợp giữa định kiểu tĩnh chặt chẽ (`serde`) và payload động (`serde_json::Value`), giúp đón nhận mọi thay đổi từ các API AI bên thứ 3 mà không làm gián đoạn hệ thống.

2. **`tool/dashboard/`**: Giao diện Studio Canvas Next.js / React (tương lai có thể đóng gói Desktop App siêu nhẹ qua Tauri).
3. **`data/`**: Hệ thống dữ liệu đa dự án độc lập (`projects/`, `shared/`).

---

## 2. LỘ TRÌNH CHUYỂN ĐỔI (ROADMAP)

- [x] **Giai đoạn 1: Khởi tạo Repo & Bản quyền**
  - Khởi tạo repo `vicieo` độc lập trên GitHub.
  - Clone và bảo tồn toàn bộ dữ liệu lịch sử từ `socnhotv`.
- [x] **Giai đoạn 2: Khung kiến trúc Core Engine (Rust Native)**
  - Cấu trúc module: `workspace`, `schema`, `ai`, `video`, `cli`.
  - Hỗ trợ CLI đa lệnh: `info`, `project`, `render`, `ai`.
- [ ] **Giai đoạn 3: Tích hợp FFmpeg & HyperFrames Pipeline**
  - FFmpeg filtergraph TikTok mastering (-14 LUFS, voice EQ).
  - Headless browser rendering với Ego Lite.
- [ ] **Giai đoạn 4: Đóng gói Desktop App (Tauri + Rust)**
  - Gộp dashboard và core engine thành 1 ứng dụng desktop duy nhất <35MB.

