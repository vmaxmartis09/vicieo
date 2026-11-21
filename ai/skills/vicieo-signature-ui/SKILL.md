---
name: vicieo-signature-ui
description: "Bộ 9 Cụm Tiêu Chuẩn Làm UI Thông Minh & Tinh Tế của ViciEO kết hợp Leonxlnx/taste-skill và Nutlope/hallmark: Khung giấy ấm (Framed-Sheet), Phẳng khi nghỉ (Flat-at-Rest), Màu nhấn <3%, Bo góc đồng tâm, Lưới 4px, 28 Luật tương tác thông minh, Morphicons, enterOnce, Reveal 0fr->1fr, Dual-Mode Card, và Component <= 250 dòng."
---

# ViciEO Signature UI Craft — Tiêu Chuẩn Làm UI Thông Minh & Tinh Tế

## 1. Phân Quyền Giữa Các Bộ Skill UI
1. **Hợp đồng Token của dự án thắng tuyệt đối**.
2. **`design-taste-frontend` ([Leonxlnx/taste-skill](https://github.com/leonxlnx/taste-skill)) + `hallmark` quyết định Diện mạo**: Dùng `draw.mjs` (trang public) hoặc `draw.mjs --app` (màn hình làm việc) để chọn hướng thiết kế từ 247 trang Awwwards, và kiểm định bằng `lint.mjs` / `inspect.mjs` + `./ai/bin/fe-core sloplint`.
3. **`ui-ux-pro-max` CHỈ dùng cho Trải nghiệm UX, Bàn phím, Focus, Vùng chạm & Stack**: Cấm tuyệt đối dùng bảng màu/style gợi ý của `ui-ux-pro-max` (tránh màu tím `#7C3AED` + hồng `#EC4899` của AI).

## 2. Khung Giấy Ấm · Phẳng Khi Nghỉ · Màu Nhấn `< 3%`
- **Khung giấy ấm (`Framed-Sheet`)**: Nền khung màu đá ấm (`hsl(38 36% 96%)`) hoặc mực sâu (`hsl(36 14% 7%)`), cấm `#000000` / `#FFFFFF` gắt. Tờ giấy làm việc cách mép phải/dưới `8px`, viền `1px`, bo góc `16px`.
- **Phẳng khi nghỉ (`Flat at Rest`)**: Thẻ thường `shadow: none`; tạo chiều sâu bằng đổi tông nền (`bg-card` → `bg-card-elevated`) + viền `1px`. Chỉ vật nổi (Popover, Flyout, Pill Dock) mới có `shadow-float`.
- **Nút chính màu Mực · Màu thương hiệu `< 3%`**: Nút chính dùng màu mực gần đen; màu thương hiệu chỉ dùng cho dữ liệu chính, focus ring, trạng thái `running` và 1 gạch chân tab. Màu nằm ở dữ liệu, không ở vỏ khung.

## 3. Bo Góc Đồng Tâm · Lưới 4px · Thang Chữ 7 Cấp
- **Bo góc đồng tâm**: $R_{\text{con}} = R_{\text{cha}} - \text{Padding}$ (`rounded-sheet` 24px → `rounded-panel` 16px → `rounded-card` 12px → `rounded-ctl` 8px → `rounded-xs` 4px).
- **Lưới 4px tuyệt đối**: Chỉ dùng `0, 1, 2, 3, 4, 5, 6, 8, 10, 12` (cấm `.5`, cấm `7/9/11`, cấm `[..px]`). Panel nổi dùng `p-3` / `gap-3`.
- **Thang chữ >= 11px & `tabular-nums`**: `text-caption` (11/14), `text-footnote` (12/16), `text-body` (13/18), `text-callout` (14/20), `text-headline` (15/20), `text-title` (17/22), `text-largetitle` (22/28). Số liệu bắt buộc bật `tabular-nums`.
- **Chiều cao control**: `h-6` (24px) · **`h-7` (28px mặc định)** · `h-9` (36px). Vùng chạm cảm ứng `>= 32–44px`.

## 4. Vật Lý Không Gian & Mẫu Tương Tác Thông Minh
- **Hộp xác nhận mọc sát Icon (`await confirm()`)**: Mở ngay cạnh nút vừa bấm (scale từ icon), cấm `window.confirm`/`alert`.
- **Mở rộng sang trái thay vì Modal (`usePreview.open`)**: Tự đóng khi đổi tab/panel/mục chọn (`usePreview.getState().close()`).
- **Click đơn = Chọn + Xem trước; Nhấn đúp = Mở / Đi vào trong**.
- **Thanh điều hướng là Viên thuốc nổi (`overflow-clip`)**: Tự nở rộng và hiện nhãn khi mở, cấm làm rail full-height dính cạnh.
- **Icon động `MorphIcon` + `TooltipHost`**: Cấm `lucide-react`; mọi nút chỉ-icon bắt buộc có `title`/`data-tip`/`aria-label`.
- **Đơn nhất & Liên thông**: Cấm 2 nút cùng chức năng trên 1 mặt giao diện; cùng 1 thực thể ở nhiều tab dùng chung 1 component và bấm là dẫn tới đúng việc.

## 5. Trạng Thái Trống · Chuyển Động · Card 2 Chế Độ · Modularity
- **Trạng thái trống thông minh**: Mục trống làm mờ + gom thành 1 dòng viền nét đứt; nút "Thêm" của lưới thẻ là **thẻ trống viền nét đứt có cùng kích thước đặt ngay cuối danh sách**.
- **Chuyển động nhẹ & Không lặp**: Tween `0.15–0.25s` `cubic-bezier(0.22, 1, 0.36, 1)` (cấm spring nảy); dùng `enterOnce(key)` chống lặp hiệu ứng khi đổi tab; ẩn/hiện khối bằng `Reveal` (`grid-rows: 0fr ↔ 1fr`); cuộn `scroll-quiet` 4px.
- **Card/Node 2 chế độ**: Mặc định *Xem* siêu gọn (ẩn nhãn phụ, ẩn chân `$0.00`, ẩn icon `idle`) ↔ bấm tiêu đề/input mở rộng *Chỉnh sửa* (`IN / BODY / OUT` `ValueSlot`).
- **Kỷ luật Modularity**: Component `<= 250 dòng`, Custom Hook `<= 150 dòng`, tách state xuống sát lá kèm `ErrorBoundary`.
