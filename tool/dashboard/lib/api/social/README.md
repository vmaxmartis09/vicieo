# `lib/api/social/` — Client API Phân Hệ Social & Xuất Bản

- **Vai trò:** Tách từ `lib/api/social.ts` (982 LOC → barrel re-export) theo 8 miền nghiệp vụ độc lập kèm `types.ts` và `client.ts`.
- **Cấu trúc module:**
  - `client.ts`: cấu hình `SOCIAL_CORE_URL` và helper `fetchJson`.
  - `types.ts`: toàn bộ interface/type Social (`SocialAccount`, `Post`, `QueueResponse`, `CalendarResponse`, `AutomationRule`, `ContentTemplate`...).
  - `endpoints/accounts.ts`, `videos.ts`, `posts.ts`, `calendar.ts`, `publishing.ts`, `analytics.ts`, `automations.ts`, `hub.ts`: từng nhóm endpoint thuần.
  - `index.ts`: ghép `socialApi` và re-export qua `lib/api/social.ts` để bảo toàn ~19 nơi import.
- **Ai gọi ai:** `store/modules/social/*`, `PublishTab`, `PublishCenterModal`, `SocialHub`, `AccountsView`, `QueueView`, `CalendarView` → `socialApi`.
- **Các bẫy đã biết:**
  - Một số phương thức (`getPackageZipUrl`) trả thẳng chuỗi URL tải `.zip` thay vì gọi `fetchJson`.
  - Kiểm tra query string (`bucket=all` bị lược bỏ trên `getQueue`, `account_id`/`platform` thêm có điều kiện trên `getAnalyticsOverview`).
