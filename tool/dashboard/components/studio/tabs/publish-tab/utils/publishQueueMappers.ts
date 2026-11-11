import type { QueueResponse } from "@/lib/api/social";
import type { ConnectedAccountData } from "@/lib/socialConfig";
import type { PublishedTargetOption } from "../types";

export interface QueueSummaryCounts {
  upcomingCount: number;
  draftCount: number;
  pendingCount: number;
  overdueOrFailedCount: number;
}

/** Hàm thuần: chuẩn hóa cờ kết nối từng nền tảng từ storage. */
export function mapConnectedPlatformFlags(
  accs: Record<string, ConnectedAccountData>
): Record<string, boolean> {
  return {
    tiktok: Boolean(accs.tiktok?.connected),
    youtube: Boolean(accs.youtube?.connected),
    facebook: Boolean(accs.facebook?.connected),
    instagram: Boolean(accs.instagram?.connected),
  };
}

/** Hàm thuần: lấy tối đa 5 bài đang chờ đăng / nháp / chờ duyệt. */
export function filterUpcomingQueueItems(queueData: QueueResponse | null | undefined) {
  const items = queueData?.items || [];
  return items
    .filter(
      (item) =>
        item.status === "scheduled" ||
        item.status === "draft" ||
        item.status === "pending_approval"
    )
    .slice(0, 5);
}

/** Hàm thuần: map danh sách target phục vụ nhập số liệu thủ công. */
export function mapPublishedTargets(
  queueData: QueueResponse | null | undefined
): PublishedTargetOption[] {
  const items = queueData?.items || [];
  return items.map((i) => ({
    target_id: i.target_id,
    post_id: i.post_id,
    title: i.video_title || i.title || "Video",
    platform: i.platform,
  }));
}

export function computeSummaryCounts(
  queueData: QueueResponse | null | undefined
): QueueSummaryCounts {
  const summary = queueData?.summary;
  const overdue = summary?.overdue_count ?? 0;
  const failed = summary?.failed_count ?? 0;
  return {
    upcomingCount: summary?.upcoming_count ?? 0,
    draftCount: summary?.draft_count ?? 0,
    pendingCount: summary?.pending_approval_count ?? 0,
    overdueOrFailedCount: overdue + failed,
  };
}
