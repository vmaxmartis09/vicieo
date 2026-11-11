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

### Chạy 1 lệnh duy nhất: `vcrun`
Chỉ cần gõ **duy nhất** từ khóa sau tại thư mục gốc `vicieo` để chạy đồng thời cả Rust Engine + Next.js Studio Canvas:

```bash
vcrun
```

*(Hỗ trợ trực tiếp trên Windows PowerShell, CMD, Git Bash và macOS/Linux terminal)*

- Nếu trên macOS/Linux chưa set alias shell, bạn có thể chạy: `./vcrun.sh` hoặc thêm vào `~/.zshrc`:
  ```bash
  alias vcrun="node dev.mjs"
  ```
- Hoặc qua npm: `npm run vcrun`

#### Port & Endpoints:
- **Next.js Studio:** [http://localhost:3600](http://localhost:3600)
- **Rust Engine Server:** [http://localhost:9000](http://localhost:9000) (Health check: [http://localhost:9000/health](http://localhost:9000/health))

---

## 3. YÊU CẦU MÔI TRƯỜNG & CÀI ĐẶT (PREREQUISITES)

1. **Node.js (>= 20)**:
   - Cài đặt dependencies cho frontend (chỉ cần chạy 1 lần):
     ```bash
     cd tool/dashboard && npm install
     ```
2. **Rust & Cargo**:
   - **Windows:** Tải bộ cài `rustup-init.exe` tại [https://rustup.rs/](https://rustup.rs/) và cài đặt. Sau khi cài xong, mở lại Terminal để nhận diện biến môi trường `cargo`.
   - **macOS / Linux:**
     ```bash
     curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
     source "$HOME/.cargo/env"
     ```

