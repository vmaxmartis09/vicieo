---
name: fe-multi-cloud-deploy
description: "Multi-Cloud Packaging, Zero-Leak .env Guardian (gitleaks, dotenvx, t3-env via Rust fe-core env-guard), Git Remote Sync (gh/glab), and Automated Deployment to Vercel (OIDC), Cloudflare Pages/Workers (Wrangler), Netlify, Railway, Fly.io & Docker VPS via CLI + Ego Lite / AGY Browser."
---

# Đóng Gói, Bảo Vệ `.env` & Auto-Deploy Đa Nền Tảng (`fe-multi-cloud-deploy`)

## 1. Cổng Bảo Vệ Biến Môi Trường `./ai/bin/fe-core env-guard`
- Chạy `./ai/bin/fe-core env-guard` trước mọi lệnh commit/push/deploy (`0 token LLM`, không bao giờ in giá trị secret ra chat).
- Kiểm tra parity giữa `process.env.*` / `import.meta.env.*` trong code với `.env.local`, `.env.example` và Cloud (`vercel env ls` / `wrangler secret list`).
- Chặn cứng 100% nếu phát hiện secret key nằm trong `NEXT_PUBLIC_*` / `VITE_*` / `NUXT_PUBLIC_*` hoặc hardcoded API token trong file mã nguồn.

## 2. Đóng Gói & Auto-Deploy Đa Đích (CLI + Browser Fallback)
- **Vercel (`vercel-labs/agent-skills`)**: `vc build`, `vc deploy` (Preview) / `vc deploy --prod`; dùng `vc project token` (OIDC ngắn hạn) để trình duyệt QA truy cập trang Preview có bật Deployment Protection.
- **Cloudflare Pages/Workers (`cloudflare/skills`)**: `@opennextjs/cloudflare` (Next.js), `NITRO_PRESET=cloudflare-pages` (Nuxt), `@sveltejs/adapter-cloudflare`, `@astrojs/cloudflare` + `wrangler deploy` / `wrangler secret put`.
- **Netlify / Railway / Fly.io / Docker Standalone**: Cấu hình tối giản theo chuẩn `ZERO-CONFIG-SAFE`.
- **Tự động hóa Trình duyệt (`./ai/bin/fe-core browser-bridge`)**: Khi cần bấm OAuth nối repo, lấy key từ Dashboard thứ 3 (Supabase/Stripe/Clerk), cấu hình DNS, hoặc mở URL live nghiệm thu HTTP 200 sau khi deploy → **ưu tiên số 1 mở Ego Lite (`ego-browser`, bảo vệ space `Dad`)**, không có thì fallback sang **AGY Built-in `browser`**.
