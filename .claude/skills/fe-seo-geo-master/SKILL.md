---
name: fe-seo-geo-master
description: "Full-Spectrum SEO, Technical SEO, JSON-LD Rich Snippets, Dynamic Edge OG, Programmatic SEO (pSEO) & AI Search GEO (llms.txt) distilled from AgriciDaniel/claude-seo (31.2k★) and verified by Rust fe-core seo-check."
---

# Master SEO, Programmatic SEO & AI Search GEO (`llms.txt`)

## 1. Technical SEO & Semantic HTML5 Outline
- **Cấu trúc thẻ chuẩn**: Mỗi trang có **duy nhất 1 thẻ `<h1>`** chứa từ khóa chính; phân cấp `<h2>` → `<h3>` không nhảy bậc; dùng đúng `<header>`, `<main>`, `<article>`, `<section>`, `<nav>`, `<footer>`.
- **Crawlable Links & Images**: Mọi liên kết điều hướng phải là thẻ `<a href="...">` thật (cấm dùng `<div onClick={router.push}>`); mọi ảnh có `alt` mô tả thực chất, `width`/`height` tường minh (`CLS = 0`), `loading="lazy"` (riêng ảnh LCP Hero dùng `fetchPriority="high"` / `priority`).
- **Metadata & Indexing**: Khai báo đầy đủ `title` (50–60 ký tự), `description` (140–160 ký tự), `<link rel="canonical">`, `hreflang` (đa ngôn ngữ), `/robots.txt` và `/sitemap.xml` tự động.

## 2. Structured Data — JSON-LD Schema.org Rich Snippets
Nhúng `<script type="application/ld+json">` hợp lệ theo đúng loại trang:
- **SaaS / App / Landing**: `Organization`, `WebSite` (kèm `SearchAction`), `SoftwareApplication`, `FAQPage`.
- **E-Commerce**: `Product`, `Offer` (`price`, `priceCurrency`, `availability: "https://schema.org/InStock"`), `AggregateRating`, `BreadcrumbList`.
- **Blog / Docs / LMS**: `Article` / `TechArticle` (`author`, `datePublished`, `dateModified`), `Course`, `VideoObject`.

## 3. Social Graph & Dynamic Edge OG Images (`1200×630`)
- Đầy đủ `og:title`, `og:description`, `og:image`, `og:url`, `og:type`, `twitter:card` (`summary_large_image`).
- Sinh ảnh OG động tại Edge (`app/api/og/route.tsx` với `@vercel/og` hoặc `nuxt-og-image`) theo tiêu đề và dữ liệu thật của từng trang.

## 4. Programmatic SEO (pSEO) Scaling
- Khi dựng cụm trang theo mẫu dữ liệu (`/solutions/[slug]`, `/compare/[a]-vs-[b]`): dùng SSG/ISR (`generateStaticParams`), bắt buộc mỗi trang có bảng dữ liệu so sánh thật và FAQ riêng (chống Thin Content), nối mạng lưới Hub-and-Spoke internal links (độ sâu click `<= 3`).

## 5. GEO / AI Search Optimization (`/llms.txt` & Answer-First)
- Tự động cung cấp file **`/llms.txt`** (bản đồ Markdown tóm tắt cấu trúc sản phẩm, tài liệu, bảng giá, API) và **`/llms-full.txt`** để **ChatGPT Search, Perplexity, Claude và Google AI Overviews** thu thập và trích dẫn chính xác.
- Kiểm định bằng lệnh Rust: `./ai/bin/fe-core seo-check <dir>`.
