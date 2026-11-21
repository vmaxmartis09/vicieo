---
name: fe-code-quality-gate
description: "Ruthless Frontend Code Quality, AST Blast-Radius, Re-render Profiler & Dead-Code Gate distilled from obra/superpowers (322.9k★), DietrichGebert/ponytail (195.7k★), garrytan/gstack (142.2k★), biomejs/biome, oxc-project/oxc, aidenybai/react-scan, and webpro-nl/knip. Executed via Rust fe-core."
---

# Cổng Kiểm Định Chất Lượng Code & Cắt Mỡ Thừa (`fe-code-quality-gate`)

## 1. Bộ Công Cụ Kiểm Định Rust Native (`./ai/bin/fe-core`)
- **Trước khi code**: `./ai/bin/fe-core ast-blast <dir> --target <name>` (tính bán kính ảnh hưởng AST trong `< 40ms`) + `./ai/bin/fe-core bus recall "<keyword>"`.
- **Ngay khi ghi file (`Write`/`Edit`)**: Hook `./ai/bin/fe-core check-edit` tự động chặn (`BLOCK`) mọi vi phạm token, anti-slop, bare Zustand store, `window.confirm`, `<img>` thiếu `loading="lazy"`, và lộ secret `NEXT_PUBLIC_*`.
- **Sau khi code**: `./ai/bin/fe-core sloplint <dir>` + `./ai/bin/fe-core env-guard` + `./ai/bin/fe-core seo-check <dir>`.

## 2. 5 Trục Cắt Mỡ Thừa (`ULTRA-RUTHLESS` Ponytail Gate)
1. **Re-render & State (`react-scan`)**: Cấm `useStore()` trần không `useShallow`; cấm copy Server State vào Zustand/Pinia; hạ state thay đổi liên tục xuống component lá; bọc `React.memo` cho node/row lớn.
2. **Modularity Budget**: Component hiển thị `<= 250 dòng`, Custom Hook `<= 150 dòng`; vượt ngưỡng phải tách thư mục kèm `index.ts` re-export.
3. **Dead Code & Bloat (`knip` + `ponytail-review`)**: Xóa ngay wrapper chỉ bọc 1 thẻ, `useEffect` tính derived state, import/export/dependency thừa.
4. **Async & Memory Safety**: Mọi fetch/effect bất đồng bộ phải có `AbortController` hoặc cleanup timer/listener; polling dừng khi `document.hidden`.
5. **Zero Build/Type Error**: `tsc --noEmit` / `vue-tsc` / `svelte-check` / `astro check` = 0 lỗi; build thử vào `.next-build` thành công 100% rồi xóa ngay `.next-build`.
