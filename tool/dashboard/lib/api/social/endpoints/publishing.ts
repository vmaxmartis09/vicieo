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

export const publishingEndpoints = {
// Scheduler & Publishing
  getSchedulerStatus: (projectId: string) =>
    fetchJson<{ heartbeat: string | null; scheduled_count: number; recent_activity: any[] }>(
      `${SOCIAL_CORE_URL}/projects/${projectId}/social/scheduler/status`
    ),

  validateTarget: (projectId: string, postId: string, targetId: string) =>
    fetchJson<{ target_id: string; issues: { level: string; code: string; message: string }[] }>(
      `${SOCIAL_CORE_URL}/projects/${projectId}/social/posts/${postId}/targets/${targetId}/validate`,
      { method: "POST" }
    ),

  publishNow: (projectId: string, postId: string) =>
    fetchJson<{ post_id: string; status: string; targets: any[] }>(
      `${SOCIAL_CORE_URL}/projects/${projectId}/social/posts/${postId}/publish-now`,
      { method: "POST" }
    ),

  markPublished: (projectId: string, postId: string, targetId: string, url: string) =>
    fetchJson<{ post_id: string; target_id: string; status: string; url: string }>(
      `${SOCIAL_CORE_URL}/projects/${projectId}/social/posts/${postId}/targets/${targetId}/mark-published`,
      {
        method: "POST",
        body: JSON.stringify({ url }),
      }
    ),

  getConnectUrl: (projectId: string, accountId: string) =>
    fetchJson<{ account_id: string; platform: string; connect_url: string }>(
      `${SOCIAL_CORE_URL}/projects/${projectId}/social/accounts/${accountId}/connect`
    ),

  refreshAccount: (projectId: string, accountId: string) =>
    fetchJson<{ status: string; details: any }>(
      `${SOCIAL_CORE_URL}/projects/${projectId}/social/accounts/${accountId}/refresh`,
      { method: "POST" }
    ),

  getPackageZipUrl: (projectId: string, postId: string, platform: string) =>
    `${SOCIAL_CORE_URL}/projects/${projectId}/social/packages/${postId}/${platform}.zip`,

  getPackageInfo: (projectId: string, postId: string, platform: string) =>
    fetchJson<{
      post_id: string;
      platform: string;
      package_dir: string;
      zip_path: string | null;
      zip_filename: string | null;
      has_real_video: boolean;
      caption: string;
      hashtags: string;
      checklist: string;
    }>(`${SOCIAL_CORE_URL}/projects/${projectId}/social/packages/${postId}/${platform}`),

  generatePackage: (projectId: string, postId: string, platform: string) =>
    fetchJson<{
      post_id: string;
      platform: string;
      package_dir: string;
      zip_path: string;
      zip_filename: string;
      has_real_video: boolean;
      caption: string;
      hashtags: string;
      checklist: string;
    }>(`${SOCIAL_CORE_URL}/projects/${projectId}/social/packages/${postId}/${platform}/generate`, {
      method: "POST",
    }),
};
