# ViciEO: Lightweight High-Performance Multiplatform AI Video Studio

> **Nền tảng sáng tạo & sản xuất video KOC/Hoạt hình 3D đa nền tảng, hiệu năng cao**, xây dựng trên kiến trúc Rust Native Engine và Next.js Studio Canvas.

---

## 1. TỔNG QUAN KIẾN TRÚC MỚI

1. **`engine-rust/`**: Core Native Engine được viết bằng **Rust**.
   - **Kích thước cực nhỏ:** Single executable binary (~15MB), không cần Python runtime hay virtualenv.
   - **Tốc độ:** Khởi động tức thì (<0.01s), RAM <25MB.
   - **Xử lý bất đồng bộ & đa luồng:** Tận dụng `tokio` để điều phối pipeline render, xử lý asset song song.
   - **REST API Server:** Tích hợp máy chủ Axum hiệu năng cao phục vụ trên Port 9000.
   - **Schema hợp đồng 2 tầng (Two-Tier Schema):** Kết hợp giữa định kiểu tĩnh chặt chẽ (`serde`) và payload động (`serde_json::Value`), giúp đón nhận mọi thay đổi từ các API AI bên thứ 3 mà không làm gián đoạn hệ thống.

2. **`tool/dashboard/`**: Giao diện Studio Canvas Next.js / React (phục vụ trên Port 3600).
3. **`data/`**: Hệ thống dữ liệu đa dự án độc lập (`projects/`, `shared/`).

---

## 2. KHỞI CHẠY HỆ THỐNG

### Cách 1: Chạy đồng thời cả Rust + Next.js (Dùng chung Mac & Windows)
Chỉ cần chạy 1 lệnh duy nhất tại thư mục gốc:
```bash
npm run dev
# Hoặc: node dev.mjs
# Trên Windows click/chạy: dev.bat
# Trên macOS / Linux: ./dev.sh
```

### Cách 2: Khởi chạy thủ công từng phần

#### Frontend Next.js Studio (Port 3600):
```bash
cd tool/dashboard
npm run dev:web
# Truy cập: http://localhost:3600
```

#### Backend Rust Native Engine (Port 9000):
```bash
cd engine-rust
cargo run -- serve --port 9000
# Endpoint kiểm tra: http://localhost:9000/health
```

