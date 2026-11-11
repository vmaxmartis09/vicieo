# `publish-tab/`

Module tab **Lịch đăng Social (`PublishTab`)** thuộc khung phải Studio.

## Cấu trúc
- `index.tsx`: Hợp nhất 4 section và 3 modal (`ManualPackageModal`, `ManualMetricsModal`, dynamic `PublishCenterModal`).
- `types.ts`: Kiểu nội bộ (`PublishTabBucket`, `PackagePostSelection`, `PublishedTargetOption`, `LearningsState`).
- `constants.ts`: Danh sách nền tảng thương hiệu `BRAND_PLATFORMS`.
- `utils/publishQueueMappers.ts`: Hàm thuần chuẩn hóa cờ kết nối nền tảng, lọc 5 bài sắp đăng và map `targets`.
- `hooks/usePublishTabData.tsx`: Selector `useShallow` tới `useSocialStore`, lắng nghe sự kiện `social-accounts-updated`, mở `SocialConnectDrawer` qua `usePreview`, nạp queue/accounts/learnings.
- `components/`: `PublishOverviewCard.tsx`, `UpcomingPostsSection.tsx`, `LearningsAnalyticsCard.tsx`, `ConnectedChannelsSection.tsx`.

## Người gọi & Tương thích ngược
- Re-export qua `components/studio/tabs/PublishTab.tsx` cho `RightPanel.tsx` và `FloatingPanels.tsx`.
- Giữ `next/dynamic` cho `PublishCenterModal`.

## Bẫy đã biết
- Giữ lắng nghe custom event `social-accounts-updated` trên `window` để đồng bộ trạng thái chấm xanh ngay khi người dùng kết nối ở `SocialConnectDrawer`.
