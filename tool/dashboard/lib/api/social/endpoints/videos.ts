import { SOCIAL_CORE_URL, fetchJson } from "../client";
import type {
  PublishableVideo,
  PlanPreviewRequest,
  PlannedTarget,
  TargetCommitSpec,
  CaptionGenerateRequest,
  PlatformCaptionResult,
  QueueResponse,
  SocialAccount,
  Post,
  CalendarResponse,
  RecurringSlot,
  CalendarNote,
  KPISummary,
  HeatmapData,
  GoldenHourSlot,
  CreativePerformance,
  AutomationRule,
  SocialNotification,
  ContentTemplate,
  AccountStats,
  SocialOverview,
} from "../types";

export const videosEndpoints = {
// Videos & Bulk Planning
  getVideos: (projectId: string) =>
    fetchJson<PublishableVideo[]>(`${SOCIAL_CORE_URL}/projects/${projectId}/social/videos`),

  previewPlan: (projectId: string, payload: PlanPreviewRequest) =>
    fetchJson<PlannedTarget[]>(`${SOCIAL_CORE_URL}/projects/${projectId}/social/plans/preview`, {
      method: "POST",
      body: JSON.stringify(payload),
    }),

  commitPlan: (projectId: string, targets: TargetCommitSpec[], approvalRequired: boolean = true) =>
    fetchJson<{ created_posts_count: number; created_targets_count: number; posts: Post[] }>(
      `${SOCIAL_CORE_URL}/projects/${projectId}/social/plans/commit`,
      {
        method: "POST",
        body: JSON.stringify({ targets, approval_required: approvalRequired }),
      }
    ),

  generateCaptions: (projectId: string, payload: CaptionGenerateRequest) =>
    fetchJson<PlatformCaptionResult[]>(`${SOCIAL_CORE_URL}/projects/${projectId}/social/captions/generate`, {
      method: "POST",
      body: JSON.stringify(payload),
    }),

  getQueue: (
    projectId: string,
    bucket: "draft" | "pending_approval" | "upcoming" | "overdue" | "published" | "failed" | "all" = "upcoming",
    params?: { accountId?: string; platform?: string; fromDate?: string }
  ) => {
    const q = new URLSearchParams();
    if (bucket && bucket !== "all") q.set("bucket", bucket);
    if (params?.accountId) q.set("account", params.accountId);
    if (params?.platform) q.set("platform", params.platform);
    if (params?.fromDate) q.set("from", params.fromDate);
    const qs = q.toString() ? `?${q.toString()}` : "";
    return fetchJson<QueueResponse>(`${SOCIAL_CORE_URL}/projects/${projectId}/social/queue${qs}`);
  },
};
