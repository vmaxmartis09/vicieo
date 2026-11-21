---
name: web-genres-playbooks
description: "8 Production Web Archetype Playbooks: (1) Enterprise Dashboard & BI, (2) CRM/ERP/Kanban/DataGrid, (3) E-Commerce & Stripe/PayOS Checkout, (4) SaaS Landing & Conversion Funnel, (5) 3D WebGL (R3F/Drei) & Official GSAP Scrollytelling, (6) Content/Blog/Docs/LMS, (7) Interactive Node Canvas Studio, (8) Social/Realtime Chat."
---

# 8 Playbooks Xây Dựng Mọi Loại Hình Web Chuyên Biệt

## 1. Enterprise Dashboard & BI Analytics (`Operate Mode`)
- **Bố cục**: `Framed-Sheet` phẳng, mật độ thông tin cao, 0 layout-shift.
- **KPI & Biểu đồ**: Số liệu thật 100% (`tabular-nums`), biểu đồ tinh gọn (không cầu vồng, không sparkline bịa `"+12.5%"`), đồng bộ bộ lọc ngày/nhóm với URL query (`nuqs`).
- **Realtime**: Nhận SSE stream theo scope hẹp, cô lập timer 1s ở component lá, dừng polling khi `document.hidden`.

## 2. CRM / ERP / Project Management & SaaS Workspace
- **Master Data Grid**: `@tanstack/table` + `@tanstack/virtual` (hoặc `.virtual-row` `content-visibility: auto`) xử lý mượt 10.000+ dòng: Multi-sort, Faceted Filter, Column Pinning/Resize, Bulk Actions, Inline Edit.
- **Kanban & Pipeline**: Kéo thả 60fps (`@dnd-kit`), Optimistic UI cập nhật tức thì kèm rollback khi API lỗi.
- **Command & Forms**: `Cmd+K` Command Palette, Multi-step Form (`React Hook Form` + `Zod`) có Auto-save draft, xác nhận xóa/đổi qua `confirm()` mọc sát icon.

## 3. E-Commerce, Marketplace & Booking (`medusajs/medusa` + `vercel/commerce` + `stripe/agent-toolkit`)
- **Catalog & Filters**: Faceted Search đồng bộ URL chuẩn SEO (SSR/ISR), chọn biến thể (Size/Color/Stock matrix) tức thì.
- **Cart & Checkout State Machine**: Đồng bộ giỏ hàng Guest (`localStorage`) ↔ Authenticated User; Optimistic Inventory; luồng Checkout ít bước tích hợp **Stripe / LemonSqueezy / PayOS / VNPay QR** kèm xác thực Webhook an toàn.

## 4. High-Converting SaaS Landing Page & Product Launch
- **Above-the-Fold (`1280×800`)**: Headline (~5 từ) + 1 câu dẫn + 1 Primary CTA + 1 mảnh giao diện sản phẩm thật (`app.css` fragment từ `taste-skill`) hiển thị trọn vẹn không cần cuộn (~80 từ/màn hình).
- **Trình tự lập luận**: Mỗi section là 1 luận điểm kèm bằng chứng thật (`references/copy.md`), bảng giá Monthly/Annual tự tính chiết khấu, FAQ chuẩn Schema.

## 5. Creative 3D WebGL & GSAP Scrollytelling (`pmndrs/react-three-fiber` + `greensock/gsap-skills`)
- **GSAP**: Dùng `useGSAP()` (tự động cleanup `gsap.context`), `ScrollTrigger` đồng bộ cuộn mượt `Lenis`, chỉ animate `transform` và `opacity`.
- **3D WebGL**: `React Three Fiber` + `@react-three/drei` (hoặc `TresJS` trên Vue), nén model GLTF/Draco, `frameloop="demand"` khi tĩnh để tiết kiệm pin/GPU, tắt hiệu ứng nặng khi `prefers-reduced-motion: reduce`.

## 6. Content-Heavy Web: Blog, News, Docs Hub & LMS Course
- **Kiến trúc**: MDX / Content Collections, Scroll-spy TOC, tìm kiếm toàn văn tức thì (`Pagefind` / Algolia), tô màu code `Shiki`, đa ngôn ngữ (`next-intl` / `@nuxtjs/i18n`), trình phát bài giảng HLS lưu tiến độ học.

## 7. Interactive Canvas & AI Node Studio (`xyflow/xyflow`)
- **Canvas**: Toàn màn hình, Floating Pill Docks, `React.memo` cho toàn bộ Custom Nodes/Edges, hạ state kéo dây xuống `LinkGhostLayer`.
- **Dual-Mode Node Card**: Mặc định *Xem* thu gọn ↔ bấm tiêu đề mở *Chỉnh sửa* qua `Reveal` (`0fr ↔ 1fr`), hợp nhất cổng & tham số theo mô hình `ValueSlot` (`IN / BODY / OUT`).

## 8. Social Feed, Community & Realtime Chat (`ueberdosis/tiptap`)
- **Feed & Chat**: Cuộn vô tận 2 chiều giữ nguyên scroll offset, soạn thảo Rich Text bằng `Tiptap` (`@mention`, `/` slash commands), upload media phân mảnh có preview, Optimistic Reactions.
