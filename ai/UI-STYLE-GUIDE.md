# ViciEO — UI Style Guide (tinh túy hệ thống UI)

Tổng hợp từ code thật (`tool/dashboard/app/globals.css`, `tailwind.config.ts`, `components/ui/*`) và các rule trong `ai/rules/` (`design-standard.md`, `ui-rules.md`, `morphicons.md`, `node-card-style.md`). Khi mâu thuẫn: **code > rule > file này**.

Stack: Next.js (App Router, next-intl) · Tailwind (`darkMode: class`) · CVA + `cn()` (clsx + tailwind-merge) · framer-motion · zustand · ReactFlow (canvas) · Morphicons.

---

## 1. Triết lý

- **Khung giấy ấm (framed sheet)**: nền khung màu đá ấm (`background`), "tờ giấy" và thẻ trắng (`card`). Màu nằm ở **dữ liệu**, không ở chrome.
- **Phẳng khi nghỉ**: không đổ bóng. Chiều sâu = đổi tông + viền 1px. Chỉ vật nổi (popover, flyout, dock) có `shadow-float`.
- **Nút chính là mực** (`primary` = gần đen / sáng khi dark). Màu thương hiệu `brand` (nâu hạt dẻ — lông Sóc Nhỏ) chỉ dùng cho dữ liệu, focus ring, trạng thái chạy, một đường gạch dưới.
- **Gọn, dày thông tin, nhẹ nhàng**: control 28px, chữ 13px, chuyển động ngắn, không nảy.
- **Hợp đồng đã chốt**: dùng đúng token, không tự đặt giá trị.

---

## 2. Màu (CSS variable HSL, dùng qua Tailwind: `bg-card`, `text-muted-foreground`…)

| Token | Light | Dark | Dùng cho |
|---|---|---|---|
| `background` | 38 36% 96% | 36 14% 7% | Khung: sidebar, top bar |
| `foreground` | 40 8% 8% | 40 8% 94% | Chữ chính |
| `card` | 0 0% 100% | 40 12% 9% | Tờ giấy, thẻ |
| `card-elevated` | 37 29% 95% | 38 13% 12% | Thẻ nâng nhẹ (`<Card elevated>`) |
| `popover` | trắng | 38 13% 12% | Popover, menu |
| `primary` / `-hover` / `-soft` | ink / ink 16% / ink 6% alpha | đảo | Nút chính, nền mờ chọn |
| `brand` | 22 75% 40% | 24 77% 60% | Nhấn thương hiệu, ring, running |
| `secondary` / `muted` | 37 29% 95% | 38 13% 12% | Nền phụ, vùng lõm |
| `muted-foreground` | 38 6% 35% | 39 8% 61% | Chữ phụ |
| `accent` | 38 22% 90% | 40 12% 15% | Nav pill đang chọn, hover step |
| `border` | 38 19% 88% | 40 11% 16% | Mọi viền (mặc định `*`) |
| `input` | 38 12% 78% | 40 10% 24% | Viền ô nhập |
| `ring` | = brand | = brand | Focus |

**Ngữ nghĩa** (`text-semantic-*`, `bg-semantic-*`): `success` xanh lá · `warning` cam nâu · `danger` đỏ · `info` xanh ngọc.

**Màu cạnh/dây node theo kiểu dữ liệu** (`edge-*`): text xám · image xanh ngọc (`info`) · video cam (`brand`) · audio vàng. Riêng `policy` đỏ nâu `#9f3a38`. Màu dây luôn theo kiểu dữ liệu cổng ra nguồn.

Quy tắc: không hardcode hex; mọi màu qua token để dark mode tự đúng. Chữ trên nền màu phải đạt tương phản AA (token semantic đã chỉnh sẵn cho nền sáng/tối).

---

## 3. Kiểu chữ

Font: **IBM Plex Sans** (400/500/600, latin + vietnamese) `--font-sans`; **IBM Plex Mono** (400/500) `--font-mono`. `antialiased`, số liệu dùng `tabular-nums` (`.tnum`, tự động trong `table`).

| Class | Size / Line | Weight | Dùng |
|---|---|---|---|
| `text-caption` | 11/14 | 400–500 | Nhãn phụ, timestamp, badge, chip kỹ thuật |
| `text-footnote` | 12/16 | 400 | Chú thích, helper, nút `md` |
| `text-body` | 13/18 | 400 | Nội dung chính, mô tả, danh sách |
| `text-callout` | 14/20 | 400/500 | Label form, nút lớn |
| `text-headline` | 15/20 | 600 | Tiêu đề thẻ, nhóm cài đặt |
| `text-title` | 17/22 | 600 | Tiêu đề panel/modal |
| `text-largetitle` | 22/28 | 700 | Tiêu đề màn hình, KPI lớn |

**Cấm**: `text-xs/sm/base/lg…`, `text-[12px]`, cỡ 9–10px (tối thiểu 11px).

---

## 4. Spacing (lưới 4px)

Chỉ dùng: `0 1 2 3 4 5 6 8 10 12` = 0/4/8/12/16/20/24/32/40/48px.

- `1` icon↔nhãn rất nhỏ · `2` giữa chip/icon · `3` **padding thẻ, `space-y-3`/`gap-3` trong panel nổi** · `4` padding panel · `6` panel lớn/tiêu đề trang · `8–12` tách khu vực/gutter.
- **Cấm**: bước `.5` (`gap-1.5`, `p-2.5`), bước lẻ `7/9/11`, giá trị `[..px]` cho padding/margin/gap.
- Gọn: không chừa khoảng trống thừa quanh thành phần.

---

## 5. Bo góc

| Class | px | Dùng |
|---|---|---|
| `rounded-xs` | 4 | chip, tag, badge, progress mini |
| `rounded-ctl` | 8 | button, input, textarea, select, toggle |
| `rounded-card` | 12 | thẻ con, item, preview box |
| `rounded-panel` | 16 | panel ngoài, dock nổi, popover, flyout |
| `rounded-sheet` | 24 | modal, bottom sheet |
| `rounded-full` | — | avatar, dot, pill, nút icon tròn |

**Quy tắc đồng tâm**: radius con = radius cha − padding giữa (tối thiểu 4px). Chuỗi chuẩn: panel 16 → card 12 → ctl 8 → chip 4.

Map legacy: `rounded/sm/[4-6px]`→xs · `md/lg/[8-10px]`→ctl · `xl/[12-14px]`→card · `2xl/[16-18px]`→panel · `3xl/[22-32px]`→sheet.

---

## 6. Chiều cao điều khiển & vùng chạm

`h-6` 24 (xs/sm, chip lọc) · **`h-7` 28 mặc định** (button, input, select) · `h-9` 36 (CTA chính, search chính). Nút chỉ-icon trên cảm ứng: vùng bấm 32–44px (padding/min size).

---

## 7. Đổ bóng & độ cao

`shadow-subtle`/`shadow-card` = **none** (phẳng). `shadow-float` = `0 8px 24px -8px rgba(21,20,18,.18)` (dark `.6`) chỉ cho vật nổi. Phân tầng bằng `bg-card` → `bg-card-elevated` + `border`.

---

## 8. Component chuẩn (`components/ui/`)

**Button** (CVA): variants `primary | secondary | outline | ghost | danger`; sizes `xs/sm` (h-6) · `md` (h-7, mặc định) · `lg` (h-9) · `icon`/`icon-sm`/`icon-lg`. Luôn `rounded-ctl`, `transition-colors duration-150`, focus ring `ring-2 ring-ring ring-offset-2`, `disabled:opacity-50`, `active:scale` nhẹ khi cần.

**Card**: `rounded-card border border-border bg-card` (`elevated` → `bg-card-elevated`). `CardHeader p-3 pb-2`, `CardContent p-3 pt-0`, `CardTitle text-headline`, `CardDescription text-footnote text-muted-foreground`.

Khác: `AiLoading` (trạng thái AI), `ConfirmHost` (xác nhận), `TooltipHost` (tooltip), `EmptyActions` (trống), `Img` (ảnh lazy), `StatusIsland`, `MorphIcon`, `SocialIcons`.

Ví dụ chuẩn:
```tsx
<div className="p-3 rounded-card border bg-card">
  <h4 className="text-headline">Tiêu đề</h4>
  <p className="text-body text-muted-foreground mt-2">Mô tả</p>
  <button className="h-7 px-3 text-footnote rounded-ctl gap-2 inline-flex items-center bg-primary text-primary-foreground">
    <Plus className="w-4 h-4" /> Thêm mới
  </button>
</div>
```

---

## 9. Icon

- **Chỉ Morphicons**: `import { Plus, Film } from "@/components/ui/icons"`; icon đổi trạng thái dùng `<MorphIcon icon={SunNode} className="h-4 w-4" strokeWidth={1.5} spring="snappy" />`.
- Kích thước chuẩn `h-4 w-4` (`shrink-0`), stroke 1.5. **Cấm** `lucide-react`, `react-icons`, `@heroicons`, SVG inline lặp.
- Icon thay chữ khi được; nút chỉ-icon **bắt buộc** có `title` / `data-tip` / `aria-label` (TooltipHost hiện tooltip tức thì).

---

## 10. Chuyển động

- framer-motion cho mọi chuyển đổi: mở/đóng, đổi tab, list xuất hiện, bấm (`active:scale` rất nhẹ).
- **Nhẹ, không spring nảy, không scale mạnh**: tween **0.15–0.25s** `ease [0.22, 1, 0.36, 1]`; Tailwind `ease-springy` = `cubic-bezier(0.16,1,0.3,1)`. Đổi kích thước một lần; nội dung đổi bằng mờ dần, không trượt xa.
- **Không lặp animation**: `initial={enterOnce(key, {...})}` (`lib/motion.ts`) — đã hiện rồi thì hiện ngay.
- Hiện/ẩn khối: `Reveal` (`components/canvas/Reveal.tsx`, grid-rows 0fr↔1fr + opacity, 200ms).
- Hiệu ứng có sẵn: `.node-running` (viền nhịp brand), `.edge-flowing` (nét đứt trôi), `.animate-node-indeterminate`, `.ai-shimmer`, `.ai-skeleton`.
- `prefers-reduced-motion` được tôn trọng toàn cục.
- AI đang làm → luôn dùng `AiLoading`.

---

## 11. Scroll, ảnh, hiệu năng

- Scrollbar mặc định mỏng 4px bo tròn. Vùng cuộn: `scroll-quiet` (chỉ hiện khi hover); hàng ngang: `no-scrollbar` (hỗ trợ lăn chuột). Node card/flyout ẩn scrollbar textarea.
- Ảnh: `components/ui/Img.tsx` hoặc `<img loading="lazy" decoding="async">`.
- Danh sách dài: `.virtual-item` / `.virtual-row` / `.virtual-card` (`content-visibility: auto`).
- Zustand: **cấm** `useStudioStore()` trần → selector hẹp + `useShallow`.
- Canvas: node/edge/input bọc `React.memo`. Timer 1s cô lập ở component lá (`RunStatusBar`); polling có backoff, dừng khi `document.hidden`; request dùng `AbortController`/`useAbortableEffect`.
- Vùng UI nổi dùng `overflow-clip`, không `overflow-hidden`.

---

## 12. Mẫu tương tác & bố cục (ui-rules)

1. **Hộp hành động mở sát icon gọi nó** (scale từ icon). Xác nhận qua `await confirm({tone, title, message})` (`lib/confirm.ts`; tone danger/warning/info/success). Cấm `window.confirm`.
2. **Mở rộng thay vì modal**: panel mở sang trái để xem trước (`usePreview.open`). Modal chỉ cho xác nhận/form ngắn. Đổi tab/panel/mục → `usePreview.getState().close()`.
3. **Click = chọn + xem trước; nhấn đúp = mở/vào.**
4. **Căn trái** mọi thứ có tab.
5. **Điều hướng là viên thuốc nổi** (Studio toolbar trái-giữa, nav Sảnh/Studio/Công việc trái-dưới, dock phải-giữa tự to ra khi mở). **Không** rail full-height dính cạnh. Studio toàn màn hình; breadcrumb bắt đầu bằng icon Sảnh.
6. **Phím tắt** khai báo một chỗ `lib/shortcuts.ts` (phím `?`). `Esc` đóng preview rồi thu panel.
7. **Trống**: mục không dữ liệu → mờ + disabled (tooltip "— trống"); cụm trống gom một dòng nét đứt; ô trống gợi ý việc (viền nét đứt + icon + tiêu đề + gợi ý + `+`), mỗi ô một việc riêng. Số đếm gắn vào chính tab/nút.
8. **Một thành phần, một giao diện**: cùng thực thể ở nhiều tab dùng chung component (vd `CharacterPoster`); mọi mục bấm được và dẫn tới đúng tab/mục (`useHubScope.openTab(tab, action, focusId)`).
9. **Không hai nút cùng chức năng** trên một mặt giao diện.
10. **Nút "thêm"** của danh sách thẻ = thẻ trống viền nét đứt ngay sau thẻ cuối, cùng kích thước.
11. Chọn dự án → `switchProject(id)` (không gọi `setActiveProjectId` trực tiếp).
12. Chi tiết trong tab: hiện inline + nút quay lại, không mở thêm vùng.

---

## 13. Node card (Studio canvas)

- **2 chế độ**: *xem* (chỉ nội dung/kết quả, ẩn mô tả/thống kê/footer, input không viền) ↔ *chỉnh sửa* (hiện "Nhận/Làm/Trả", tham số, footer). Bấm **tiêu đề** để đổi; focus input → chỉnh sửa; bỏ chọn → về xem. Mỗi card toggle độc lập. Mặc định chi tiết: `character`, `hyperframes_render`, `export`.
- Nhóm: `SOURCE_TYPES` (text, image, character), `PROCESS_TYPES` (script, image_gen, video_gen, voice_tts…).
- **Cổng**: icon làm chính handle (text→file-text, image→images, character→users…), nhỏ, mờ, nền muted, viền rất nhạt (đỏ khi thiếu đầu vào bắt buộc, brand khi nối được). Ở chế độ xem handle chia đều dọc mép card.
- Tiêu đề `line-clamp-2`; không icon "ready/idle" (chỉ running/ok/lỗi/stale); chi phí `$0.00` ẩn; chân card chỉ khi chi phí > 0 hoặc có lịch sử.
- Dây: bấm → hiện nút xoá; không nối ngược (từ cổng ra → cổng vào). `autoLayout` ngang trái→phải khi tải.
- Form tham số trong card: dùng token chuẩn (h-7, `rounded-ctl`, nhãn `text-body text-muted-foreground`).

---

## 14. Dark mode & i18n

- `class="dark"` trên `<html>` qua `ThemeProvider` (light/dark/system, lưu zustand). Chỉ dùng token → tự đúng; không dùng `dark:` với màu thô trừ khi bất khả kháng.
- Chuỗi UI qua next-intl (`messages/`), tiếng Việt là chính; font có subset vietnamese. Giữ dấu tiếng Việt trong dữ liệu khớp từ.

---

## 15. Checklist trước khi merge UI

- [ ] Chỉ token typography/radius/spacing; không `[..px]`, không `.5`, không 9–10px
- [ ] Control `h-7` (CTA `h-9`), radius đồng tâm
- [ ] Màu qua token, kiểm light + dark
- [ ] Icon Morphicons, nút icon có `aria-label`/`title`
- [ ] Animation 0.15–0.25s, `enterOnce`, tôn trọng reduced-motion
- [ ] Selector zustand hẹp + `useShallow`; node canvas `React.memo`
- [ ] Ảnh lazy; danh sách dài virtualized; scroll dùng `scroll-quiet`/`no-scrollbar`
- [ ] Xác nhận qua `confirm()`; trống có gợi ý; không nút trùng chức năng
- [ ] Tab/preview đóng khi chuyển ngữ cảnh

## 16. Skill hỗ trợ làm UI (`ai/skills/`)

| Skill | Dùng khi |
|---|---|
| `design-taste-frontend` | Trang public/landing, app screens — chống "AI slop", có inspector trình duyệt + vòng look-and-fix |
| `ui-ux-pro-max` | Tra cứu style/palette/font/UX guideline/chart/stack (dữ liệu local, có CLI search) |
| `design-system` | Kiến trúc token 3 lớp, component spec, slide theo brand |
| `ui-styling` | Tailwind/shadcn-style styling |
| `design`, `brand`, `banner-design`, `slides` | Logo/CIP, nhận diện, banner, slide |
| `hyperframes-creative`, `motion-graphics` | Video/motion (không phải UI dashboard) |

Ưu tiên: **hợp đồng token của dashboard (mục 2–7) thắng mọi skill thẩm mỹ chung** — skill chỉ dùng để quyết định ý tưởng/bố cục, không để đổi token.
