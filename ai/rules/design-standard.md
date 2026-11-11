# QUY CHUẨN THIẾT KẾ DASHBOARD: DESIGN SYSTEM & DESIGN TOKENS
# (DESIGN STANDARD - HỢP ĐỒNG THIẾT KẾ BẮT BUỘC)

> [!CRITICAL]
> **HỢP ĐỒNG THIẾT KẾ ĐÃ CHỐT: HIỆN THỰC ĐÚNG, KHÔNG TỰ ĐỔI GIÁ TRỊ**
> - Mọi thành phần trong `tool/dashboard` bắt buộc tuân thủ đúng bảng token khoảng cách, kiểu chữ, bo góc (radius) và chiều cao điều khiển.
> - Tuyệt đối không dùng các giá trị tự do trong ngoặc vuông như `text-[12px]`, `rounded-[14px]`, `p-[10px]`, `gap-[6px]`.
> - Không dùng các bước `.5` (như `gap-1.5`, `p-2.5`) hoặc các bước khoảng cách không thuộc lưới 4px (bước 7, 9, 11).
> - Thư viện icon duy nhất: Morphicons (`@/components/ui/icons`), tuyệt đối cấm import từ `lucide-react`.

---

## 1. BẢNG DESIGN TOKENS CHUẨN

### 1.1. Kiểu chữ (Typography)
Toàn bộ cỡ chữ có kích thước tối thiểu là 11px. Triệt tiêu hoàn toàn cỡ chữ 9px và 10px để đảm bảo khả năng đọc (accessibility).

| Tên Lớp Tailwind | Cỡ Chữ (Font Size) | Chiều Cao Dòng (Line Height) | Độ Đậm (Font Weight) | Ngữ Cảnh Sử Dụng Chuẩn |
| :--- | :--- | :--- | :--- | :--- |
| `text-caption` | 11px | 14px | 400 (hoặc 500 khi cần nhấn) | Nhãn phụ, timestamp, badge nhỏ, chip thông số kỹ thuật |
| `text-footnote` | 12px | 16px | 400 | Chú thích, helper text bên dưới ô nhập, trạng thái dòng danh sách |
| `text-body` | 13px | 18px | 400 | Văn bản nội dung chính, mô tả thẻ, nội dung danh sách |
| `text-callout` | 14px | 20px | 400 / 500 | Tiêu đề con, nhãn ô nhập form, nội dung nút bấm mặc định |
| `text-headline` | 15px | 20px | 600 (Semibold) | Tiêu đề thẻ, tiêu đề nhóm cài đặt, thanh công cụ section |
| `text-title` | 17px | 22px | 600 (Semibold) | Tiêu đề panel, tiêu đề modal, tiêu đề trang con |
| `text-largetitle` | 22px | 28px | 700 (Bold) | Tiêu đề màn hình chính, chỉ số KPI lớn |

---

### 1.2. Bo góc (Border Radius)
Hệ thống bán kính bo góc thiết kế phân cấp chặt chẽ:

| Tên Lớp Tailwind | Giá Trị Radius | Biến CSS | Đối Tượng Áp Dụng |
| :--- | :--- | :--- | :--- |
| `rounded-xs` | 4px | `--radius-xs` | Chip nhỏ, tag trạng thái, thanh tiến trình siêu nhỏ, badge gọn |
| `rounded-ctl` | 8px | `--radius-ctl` | Điều khiển: Nút bấm (`button`), ô nhập (`input`, `textarea`, `select`), toggle |
| `rounded-card` | 12px | `--radius-card` | Thẻ con bên trong (`card`, item container), khối danh sách, preview box |
| `rounded-panel` | 16px | `--radius-panel` | Khung ngoài panel, thanh dock nổi, popover, flyout drawer |
| `rounded-sheet` | 24px | `--radius-sheet` | Hộp thoại lớn, Modal dialog, bottom sheet, popup trung tâm |
| `rounded-full` | 9999px | — | Avatar tròn, nút icon tròn, indicator dot, capsule pill |

---

### 1.3. Khoảng cách (Spacing) & Lưới 4px
Chỉ sử dụng các bước Tailwind chuẩn trong hệ số chia hết cho 4px.

| Bước Tailwind | Giá Trị Pixel | Sử Dụng Cho |
| :--- | :--- | :--- |
| `0` | 0px | Reset margin/padding/gap |
| `1` | 4px | Padding vi mô, khoảng cách icon với nhãn chữ rất nhỏ |
| `2` | 8px | Khoảng cách giữa các chip, padding nút nhỏ, gap giữa các icon liền kề |
| `3` | 12px | Padding nội bộ của card con, khoảng cách trường form thông dụng |
| `4` | 16px | Padding panel tiêu chuẩn, khoảng cách giữa các khối chức năng |
| `5` | 20px | Khoảng cách phân cách giữa các section con |
| `6` | 24px | Padding panel lớn, khoảng cách tiêu đề trang |
| `8` | 32px | Khoảng trống phân tách khu vực lớn |
| `10` | 40px | Gutter trang |
| `12` | 48px | Khoảng trống hero hoặc layout biên |

> [!WARNING]
> **CẤM TUYỆT ĐỐI:**
> - Bước nửa `.5` (ví dụ `gap-0.5`, `p-1.5`, `space-y-2.5`, `m-3.5`).
> - Bước lẻ không đồng bộ lưới: `gap-7` (28px), `p-9` (36px), `m-11` (44px) cho khoảng cách padding/margin/gap.
> - Các giá trị tuỳ tiện đặt trong ngoặc vuông: `p-[10px]`, `gap-[6px]`, `m-[14px]`.

---

### 1.4. Chiều cao điều khiển (Control Height)
Các phần tử tương tác (button, input, select) chuẩn hóa chiều cao:

| Tên Lớp | Chiều Cao | Đối Tượng & Ngữ Cảnh |
| :--- | :--- | :--- |
| `h-6` | 24px | Nút phụ siêu nhỏ, chip lọc nhanh, badge tương tác |
| `h-7` | 28px | **Chiều cao mặc định** cho hầu hết nút bấm, ô input và dropdown trong dashboard |
| `h-9` | 36px | Điều khiển lớn: nút hành động chính (Primary CTA), ô tìm kiếm chính |
| `min-h-[44px]` / vùng chạm 44px | 44px | Vùng chạm di động tối thiểu trên màn hình cảm ứng (Touch Target) |

---

## 2. QUY TẮC ĐỒNG TÂM TRONG BO GÓC (CONCENTRIC RADIUS RULE)

Để đảm bảo thẩm mỹ thị giác và các đường bo góc song song tự nhiên khi lồng ghép:
$$\text{Radius ô con (bên trong)} = \text{Radius ô cha (bên ngoài)} - \text{Padding giữa cha và con}$$
*(với giá trị nhỏ nhất cho ô con không dưới 4px).*

```
+------------------------------------------------------+  <- Khung ngoài: panel (rounded-panel: 16px)
|  Padding: 4 (16px)                                   |
|                                                      |
|   +----------------------------------------------+   |  <- Thẻ con: card (rounded-card: 12px)
|   |  Padding: 3 (12px)                           |   |
|   |                                              |   |
|   |   +------------------+   +---------------+   |   |  <- Điều khiển: ctl (rounded-ctl: 8px)
|   |   | [ Ô nhập/Button ]|   | [ Chip: xs 4 ]|   |   |
|   |   +------------------+   +---------------+   |   |
|   +----------------------------------------------+   |
+------------------------------------------------------+
```

- **Ô ngoài cùng:** `rounded-panel` (16px) hoặc `rounded-sheet` (24px đối với modal).
- **Ô lồng bên trong (con):** `rounded-card` (12px).
- **Phần tử điều khiển bên trong card:** `rounded-ctl` (8px).
- **Thành phần phụ trợ rất nhỏ (chip/badge/tag):** `rounded-xs` (4px).

---

## 3. BẢNG ÁNH XẠ CHUYỂN ĐỔI (LEGACY → NEW TOKENS)

Khi dọn dẹp và chuẩn hoá mã nguồn, áp dụng chính xác bảng ánh xạ sau:

### 3.1. Typography Mapping
| Lớp Cũ / Vi Phạm | Lớp Chuẩn Mới | Ghi Chú |
| :--- | :--- | :--- |
| `text-[9px]`, `text-[10px]`, `text-[10.5px]`, `text-[11px]` | `text-caption` | Nâng lên tối thiểu 11px, line-height 14px |
| `text-[12px]`, `text-xs` | `text-footnote` | 12px / 16px |
| `text-[13px]` | `text-body` | 13px / 18px |
| `text-[14px]`, `text-sm` | `text-callout` | 14px / 20px |
| `text-[15px]` | `text-headline` | 15px / 20px (font-semibold) |
| `text-[16px]`, `text-base` | `text-headline` hoặc `text-title` | Xét ngữ cảnh: nhãn vừa → `text-callout`, tiêu đề → `text-headline`/`text-title` |
| `text-[17px]`, `text-[18px]`, `text-lg` | `text-title` | 17px / 22px |
| `text-[20px]`, `text-[22px]`, `text-[24px]`, `text-xl`, `text-2xl` | `text-largetitle` | 22px / 28px |

### 3.2. Border Radius Mapping
| Lớp Cũ / Vi Phạm | Lớp Chuẩn Mới | Ghi Chú |
| :--- | :--- | :--- |
| `rounded`, `rounded-sm`, `rounded-[4px]`, `rounded-[5px]`, `rounded-[6px]` | `rounded-xs` | Dùng cho chip, tag, badge nhỏ (4px) |
| `rounded-md`, `rounded-lg`, `rounded-[8px]`, `rounded-[10px]` | `rounded-ctl` | Dùng cho button, input, toggle, select (8px) |
| `rounded-xl`, `rounded-[12px]`, `rounded-[14px]` | `rounded-card` | Dùng cho thẻ con (12px), hoặc `rounded-panel` khi là panel ngoài |
| `rounded-2xl`, `rounded-[16px]`, `rounded-[18px]` | `rounded-panel` | Dùng cho khung panel, popover, flyout drawer (16px) |
| `rounded-3xl`, `rounded-[22px]`, `rounded-[24px]`, `rounded-[32px]` | `rounded-sheet` | Dùng cho modal dialog, bottom sheet (24px) |

### 3.3. Spacing Mapping
| Bước Cũ Vi Phạm | Thay Thế Bằng | Lý Do & Ngữ Cảnh |
| :--- | :--- | :--- |
| `.5` (2px): `p-0.5`, `gap-0.5`, `m-0.5` | `1` (4px): `p-1`, `gap-1`, `m-1` | Làm tròn lên bước 1 chuẩn lưới 4px |
| `1.5` (6px): `p-1.5`, `gap-1.5`, `m-1.5` | `2` (8px): `p-2`, `gap-2`, `m-2` | Chuyển sang 8px (hoặc 4px `1` nếu không gian cực hẹp) |
| `2.5` (10px): `p-2.5`, `gap-2.5`, `m-2.5` | `3` (12px): `p-3`, `gap-3`, `m-3` | Chuẩn hóa sang 12px theo nhịp thiết kế |
| `3.5` (14px): `p-3.5`, `gap-3.5` | `4` (16px) hoặc `3` (12px) | Làm tròn lên 16px hoặc giảm về 12px |
| Bước `7` (28px): `gap-7`, `p-7` | `6` (24px) hoặc `8` (32px) | Giữ nhịp 24px hoặc 32px |
| Bước `9` (36px): `gap-9`, `p-9` | `8` (32px) hoặc `10` (40px) | Giữ nhịp 32px hoặc 40px |
| `gap-[6px]`, `p-[10px]`, `m-[14px]` | `gap-2`, `p-3`, `m-4` | Chuyển giá trị tự đặt sang bước chuẩn |

---

## 4. VÍ DỤ MINH HỌA ĐÚNG / SAI

### Ví dụ 1: Nút bấm (Button)
```tsx
// ❌ SAI: Dùng text-xs, rounded-md, py-1.5, px-2.5, font tự đặt
<button className="text-xs px-2.5 py-1.5 rounded-md gap-1.5">
  <Plus className="w-3.5 h-3.5" /> Thêm mới
</button>

// ✅ ĐÚNG: Chiều cao h-7 chuẩn, text-footnote/callout, rounded-ctl, gap-2, icon 4x4
<button className="h-7 px-3 text-footnote rounded-ctl gap-2 inline-flex items-center">
  <Plus className="w-4 h-4" /> Thêm mới
</button>
```

### Ví dụ 2: Thẻ danh mục (Card)
```tsx
// ❌ SAI: p-[14px], rounded-[14px], text-[13px]
<div className="p-[14px] rounded-[14px] border bg-card">
  <h4 className="text-[15px] font-bold">Tiêu đề thẻ</h4>
  <p className="text-[13px] text-muted-foreground mt-1.5">Mô tả nội dung</p>
</div>

// ✅ ĐÚNG: p-3 hoặc p-4, rounded-card, text-headline, text-body, mt-2
<div className="p-3 rounded-card border bg-card">
  <h4 className="text-headline">Tiêu đề thẻ</h4>
  <p className="text-body text-muted-foreground mt-2">Mô tả nội dung</p>
</div>
```

### Ví dụ 3: Chip trạng thái (Badge / Tag)
```tsx
// ❌ SAI: text-[9px], rounded-[4px], py-0.5, px-1.5
<span className="text-[9px] px-1.5 py-0.5 rounded-[4px] bg-accent">
  Đang chạy
</span>

// ✅ ĐÚNG: text-caption (11px), rounded-xs, px-2 py-1 (hoặc h-6)
<span className="text-caption px-2 py-1 rounded-xs bg-accent">
  Đang chạy
</span>
```

### Ví dụ 4: Panel nổi và Popover
```tsx
// ❌ SAI: rounded-xl, p-5, gap-2.5
<div className="rounded-xl p-5 gap-2.5 bg-popover">
  ...
</div>

// ✅ ĐÚNG: rounded-panel, p-4 hoặc p-6, gap-3
<div className="rounded-panel p-4 gap-3 bg-popover">
  ...
</div>
```
