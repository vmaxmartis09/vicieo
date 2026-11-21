---
name: multi-stack-fe-senior
description: "Senior Multi-Framework Frontend Architecture distilled from vercel-labs/agent-skills (34.6k★), mattpocock/skills (329k★), shadcn-ui/ui (128.6k★), vuejs-ai/skills (3.3k★), unovue/shadcn-vue, huntabyte/shadcn-svelte, and PatrickJS/awesome-cursorrules. Covers Next.js 15/16, React 19, Vue 3.5/Nuxt 3-4, Svelte 5 Runes, Astro 5 Islands, and Two-Tier State."
---

# Multi-Stack Senior Frontend Engineering (Next.js · React 19 · Vue 3/Nuxt · Svelte 5 · Astro 5)

## 1. Next.js 15/16 & React 19 (`vercel-labs/agent-skills` + `shadcn-ui/ui`)
- **RSC First, Client Leaf**: Mặc định mọi Page/Layout là Server Component. Chỉ đặt `'use client'` ở component lá tương tác nhỏ nhất.
- **Eliminate Waterfalls**: Dùng `Promise.all` cho các fetch độc lập hoặc tách `<Suspense fallback={<Skeleton />}>` để stream song song.
- **Server Actions & Forms**: Validate đầu vào bằng `Zod` + `useActionState` / `React Hook Form`.
- **Compound Components**: Dùng Radix Primitives / `shadcn/ui` + `cva` + `cn()`; ưu tiên *Composition over Configuration*.

## 2. Vue 3.5 & Nuxt 3/4 (`vuejs-ai/skills` + `unovue/shadcn-vue` + `nuxt/ui`)
- **Script Setup & Type-safe Macros**: Luôn dùng `<script setup lang="ts">` với `defineProps<Props>()` và `defineEmits<Emits>()`.
- **Fine-Grained Reactivity**: Dùng `ref()` cho nguyên thủy/cấu trúc nhỏ, `shallowRef()` cho danh sách/đồ thị lớn chống deep-proxy overhead, `computed()` có cache.
- **Composables & Nuxt SSR**: Tách logic tái sử dụng vào `composables/use*.ts` (kết hợp `VueUse`); dùng `useFetch` / `useAsyncData` chống double-fetch giữa Server và Client.

## 3. Svelte 5 & SvelteKit (`huntabyte/shadcn-svelte`)
- **100% Svelte 5 Runes**: Dùng `$state()`, `$derived()`, `$derived.by()`, `$effect()`, `$props()`, và `{#snippet}` + `{@render}`. Cấm cú pháp Svelte 4 cũ (`export let`, `$:`).
- **SvelteKit Data Flow**: Tải dữ liệu ở `+page.server.ts`, xử lý mutation qua Progressive Form Actions (`use:enhance`).

## 4. Astro 5 (Zero-JS & Islands Architecture)
- **Zero-JS by Default**: Render tĩnh 100% HTML/CSS; chỉ gắn JS vào các đảo tương tác (`client:load`, `client:visible`, `client:idle`).
- **Content Collections**: Định nghĩa schema chặt chẽ bằng Zod trong `src/content/config.ts`.

## 5. Kiến Trúc Quản Lý State 2 Tầng & TypeScript Strict (`mattpocock/skills`)
- **Tầng 1 — Server State (TanStack Query v5 / `useFetch`)**:
  - Là Single Source of Truth duy nhất cho dữ liệu từ Backend/API.
  - **Cấm tuyệt đối** copy dữ liệu Server vào Zustand/Pinia.
  - Cập nhật realtime qua SSE / WebSocket (`queryClient.setQueryData` + `replaceEqualDeep`), bật `placeholderData: (prev) => prev` chống chớp trắng.
- **Tầng 2 — Client UI State (Zustand / Pinia)**:
  - Chỉ lưu trạng thái giao diện thuần túy (active tab, sidebar open, selection, viewport).
  - Trên Zustand **bắt buộc dùng selector hẹp + `useShallow`** (`useStore(useShallow(s => ({ a: s.a })))`), cấm gọi `useStore()` trần.
- **TypeScript Strict**: 0 `any`, dùng Discriminated Unions cho state (`status: 'idle' | 'loading' | 'error' | 'success'`), hủy request cũ bằng `AbortController`.

## 6. Chuẩn Tổ Chức Component Bền Vững & Dễ Bảo Trì (Elegans Co-located Architecture)
Chống tình trạng code component bừa bãi, rối rắm, phình to không thể maintain:
1. **Cấu Trúc Thư Mục Feature Co-location**:
   ```text
   components/<feature-name>/ (hoặc features/<feature-name>/)
   ├── index.ts                <- Barrel export duy nhất ra ngoài (Clean Boundary)
   ├── <FeatureName>.tsx       <- Orchestrator Component: <= 200 dòng, CHỈ RENDER JSX & bind hooks
   ├── components/             <- Sub-components nội bộ (Private, cấm file ngoài import trực tiếp)
   │   ├── <Feature>Header.tsx
   │   └── <Feature>Item.tsx
   ├── hooks/                  <- Custom Hooks: State & Logic tách biệt (<= 150 dòng)
   │   └── use<Feature>State.ts
   ├── types.ts                <- TypeScript Contracts & Zod Schemas
   └── utils/                  <- Pure helper functions không chứa JSX (Dễ unit test)
   ```
2. **Kỷ Luật Single Responsibility**:
   - Component file chỉ làm 1 việc: nhận props, gọi hook và render JSX. Cấm nhét API fetch, transformation data phức tạp hoặc logic điều kiện chằng chịt vào component.
   - Giới hạn độ dài: Component **tối đa 250 dòng**, Hook **tối đa 150 dòng**. Vượt ngưỡng bắt buộc tách sub-component / sub-hook.
   - Cấm Prop Drilling quá 2 cấp: dùng Compound Component (`<Feature.Header>`, `<Feature.Body>`) hoặc Local Context.
   - Cấm import chéo file private: module bên ngoài chỉ được import qua `index.ts`.
