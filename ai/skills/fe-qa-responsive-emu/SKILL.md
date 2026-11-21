---
name: fe-qa-responsive-emu
description: "Live Web E2E, 5-Viewport Responsive, 140+ Mobile/Tablet Presets, Android/iOS Emulator & Runtime SEO QA SOP. Uses Rust fe-core browser-bridge (Ego Lite priority 1 -> AGY Built-in browser / Playwright MCP / Chrome DevTools MCP) and fe-core qa-filter."
---

# Quy Trình Kiểm Thử Web, Responsive 140+ Thiết Bị & Mobile Emulator

## 1. Cầu Nối Trình Duyệt Ưu Tiên 2 Tầng (`./ai/bin/fe-core browser-bridge`)
1. **Ưu tiên số 1 — Ego Lite (`ego-browser`)**: Mở trang trên Ego Lite để tận dụng session thật của Ngài Vmax. Luôn tạo/dùng task space riêng (`FE QA Workspace`), **bảo vệ tuyệt đối space `Dad` (P38)**.
2. **Ưu tiên số 2 (Fallback tự động) — Antigravity Built-in `browser` / Playwright MCP**: Khi không có Ego Lite hoặc cần test 140+ profile thiết bị di động (`@playwright/mcp` + `@executeautomation/playwright-mcp-server`).
3. **Ưu tiên số 3 — Mobile Emulator thật (`adb` / `simctl`)**: Chụp ảnh binary-safe (`adb shell screencap -p /data/local/tmp/cap.png && adb pull`), kiểm tra magic bytes PNG `89 50 4E 47`.

## 2. Ma Trận 5 Mốc Viewport Bắt Buộc (Light & Dark Mode)
- `1440×900` (Desktop Wide) · `1280×800` (Laptop Compact — kiểm tra Above-the-fold Hero) · `768×1024` (Tablet) · `390×844` (Mobile iPhone 15 — kiểm tra vùng chạm `>= 44px`, Smooth Drawer, Swipe Tabs) · `320×640 @ 200% Text Zoom` (Stress test chống tràn ngang `overflow-x`).

## 3. Kiểm Định Console, Network, Web Vitals & Nén Log 0-Token
- Mọi log trình duyệt đi qua `./ai/bin/fe-core qa-filter` để lọc sạch nhiễu, xác nhận `console.error == 0`, `net_err == 0`, `LCP < 2.5s`, `CLS == 0.00`, `INP < 200ms`.
- Chụp ảnh bằng chứng lưu vào `screenshots/` và kiểm tra qua `./ai/bin/fe-core verdict`.
