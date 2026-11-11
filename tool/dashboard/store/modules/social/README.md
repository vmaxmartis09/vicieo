# `store/modules/social/`

Zustand store quản lý trạng thái phân hệ Social Hub, lịch xuất bản, phân tích hiệu suất và tự động hoá.

## Cấu trúc
- `index.ts`: Khởi tạo `useSocialStore` bằng cách ghép 4 nhóm action, đồng thời re-export toàn bộ kiểu.
- `types.ts`: `HubView`, `HubQueueBucket`, `SocialFilters`, `SocialState`, cùng helper typings cho action builders.
- `utils/socialHelpers.ts`: Hàm thuần không phụ thuộc React (`countUnreadNotifications`, `findTargetScheduledTime`, `applyTargetTimeToCalendar`, `applyTargetTimeToPosts`).
- `actions/`:
  - `viewActions.ts`: State UI tab, filter, composer, publish center, modal/drawer, Social Hub.
  - `hubDataActions.ts`: Tải tổng quan, thư viện, template, tài khoản, hàng đợi, lịch, `bulkAction` và `loadAll`.
  - `analyticsAutomationActions.ts`: Analytics, heatmap, golden windows, quy tắc tự động và thông báo.
  - `schedulingActions.ts`: Kéo thả đổi lịch lạc quan (`moveTargetTime`) và hoàn tác (`undoLastDrag`).

## Tương thích ngược
- `store/modules/social.ts` là file barrel 1 dòng (`export * from "./social/index"`) giữ nguyên mọi đường dẫn import hiện hữu.
