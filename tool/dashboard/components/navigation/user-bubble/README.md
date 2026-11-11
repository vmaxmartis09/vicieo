# user-bubble

Module menu tài khoản, cài đặt Studio nhanh (ngôn ngữ, theme, tỉ lệ khung hình, hạn mức chi tiêu, API keys, lưu trữ cục bộ).

## Cấu trúc

- `index.tsx`: Container `UserBubble` (bọc `BubbleErrorBoundary`) gắn trigger và popover (`motion.div` khi `placement === "beside"`).
- `hooks/useUserBubbleSync.ts`: Đồng bộ `/health`, `/settings/api-keys`, ước tính `localStorage`, xử lý lưu key và xóa dữ liệu 2 bước.
- `hooks/useUserBubbleFocusTrap.ts`: Bẫy focus `Tab`/`Shift+Tab`, phím `Escape` và click ngoài popover.
- `utils/storageUtils.ts`: Hàm thuần `getInitials`, `estimateLocalStorageBytes`, `formatStorageKb`.
- `components/`: `UserBubbleTrigger`, `UserBubblePopover`, `SettingRowAccordion`, `SegmentedControl`, `ProfileSection`, `RunBudgetSection`, `ApiKeysSection`, `StorageSection`, `BubbleErrorBoundary`.

## Ai gọi

- Re-export qua `components/navigation/UserBubble.tsx` cho `Sidebar.tsx`, `AppRail.tsx`, `BottomNav.tsx`.

## Bẫy đã biết

- Ở chế độ `placement="beside"`, trigger và popover nằm chung một vỏ `motion.div` giãn chiều rộng mượt từ `48px -> 316px`; nút `"Sửa hồ sơ"` chỉ hiện khi `open === true` và mở phần `profile` ở đáy popover.
