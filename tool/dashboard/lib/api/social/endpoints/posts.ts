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

export const postsEndpoints = {
// Posts
  listPosts: (projectId: string, params?: Record<string, string>) => {
    const query = new URLSearchParams(params || {}).toString();
    return fetchJson<Post[]>(`${SOCIAL_CORE_URL}/projects/${projectId}/social/posts?${query}`);
  },

  getPost: (projectId: string, postId: string) =>
    fetchJson<Post>(`${SOCIAL_CORE_URL}/projects/${projectId}/social/posts/${postId}`),

  createPost: (projectId: string, post: Partial<Post>) =>
    fetchJson<Post>(`${SOCIAL_CORE_URL}/projects/${projectId}/social/posts`, {
      method: "POST",
      body: JSON.stringify(post),
    }),

  createDraftPost: (projectId: string, data: { media_path?: string; title: string; caption?: string; platforms: string[] }) =>
    fetchJson<Post>(`${SOCIAL_CORE_URL}/projects/${projectId}/social/posts/draft`, {
      method: "POST",
      body: JSON.stringify(data),
    }),

  updatePost: (projectId: string, postId: string, patch: Partial<Post>) =>
    fetchJson<Post>(`${SOCIAL_CORE_URL}/projects/${projectId}/social/posts/${postId}`, {
      method: "PATCH",
      body: JSON.stringify(patch),
    }),

  deletePost: (projectId: string, postId: string) =>
    fetchJson<{ status: string; post_id: string }>(
      `${SOCIAL_CORE_URL}/projects/${projectId}/social/posts/${postId}`,
      { method: "DELETE" }
    ),

  submitPost: (projectId: string, postId: string, note?: string) =>
    fetchJson<Post>(`${SOCIAL_CORE_URL}/projects/${projectId}/social/posts/${postId}/submit`, {
      method: "POST",
      body: JSON.stringify({ actor: "user", note: note || "" }),
    }),

  approvePost: (projectId: string, postId: string, note?: string) =>
    fetchJson<Post>(`${SOCIAL_CORE_URL}/projects/${projectId}/social/posts/${postId}/approve`, {
      method: "POST",
      body: JSON.stringify({ actor: "user", note: note || "" }),
    }),

  rejectPost: (projectId: string, postId: string, reason?: string) =>
    fetchJson<Post>(`${SOCIAL_CORE_URL}/projects/${projectId}/social/posts/${postId}/reject`, {
      method: "POST",
      body: JSON.stringify({ actor: "user", reason: reason || "" }),
    }),

  schedulePost: (projectId: string, postId: string, targets: Record<string, string>) =>
    fetchJson<Post>(`${SOCIAL_CORE_URL}/projects/${projectId}/social/posts/${postId}/schedule`, {
      method: "POST",
      body: JSON.stringify({ targets, actor: "user" }),
    }),

  updateTargetTime: (projectId: string, postId: string, targetId: string, time: string) =>
    fetchJson<Post>(`${SOCIAL_CORE_URL}/projects/${projectId}/social/posts/${postId}/targets/${targetId}/time`, {
      method: "PUT",
      body: JSON.stringify({ time, actor: "user" }),
    }),

  cancelPost: (projectId: string, postId: string) =>
    fetchJson<Post>(`${SOCIAL_CORE_URL}/projects/${projectId}/social/posts/${postId}/cancel`, {
      method: "POST",
      body: JSON.stringify({ actor: "user" }),
    }),

  duplicatePost: (projectId: string, postId: string) =>
    fetchJson<Post>(`${SOCIAL_CORE_URL}/projects/${projectId}/social/posts/${postId}/duplicate`, {
      method: "POST",
      body: JSON.stringify({ actor: "user" }),
    }),
};
