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
  LibraryVideo,
} from "../types";

export const hubEndpoints = {
// Hub APIs
  fetchLibrary: (
    projectId: string,
    params?: { filter?: string; account?: string; platform?: string; q?: string }
  ) => {
    const sp = new URLSearchParams();
    if (params?.filter) sp.append("filter", params.filter);
    if (params?.account) sp.append("account", params.account);
    if (params?.platform) sp.append("platform", params.platform);
    if (params?.q) sp.append("q", params.q);
    const qs = sp.toString() ? `?${sp.toString()}` : "";
    return fetchJson<LibraryVideo[]>(
      `${SOCIAL_CORE_URL}/projects/${projectId}/social/library${qs}`
    );
  },

  fetchOverview: (projectId: string) =>
    fetchJson<SocialOverview>(`${SOCIAL_CORE_URL}/projects/${projectId}/social/overview`),

  bulkAction: (
    projectId: string,
    payload: { ids: string[]; action: string; params?: any }
  ) =>
    fetchJson<{ results: Array<{ id: string; success: boolean; error?: string | null }> }>(
      `${SOCIAL_CORE_URL}/projects/${projectId}/social/targets/bulk`,
      {
        method: "POST",
        body: JSON.stringify(payload),
      }
    ),

  fetchTemplates: (projectId: string) =>
    fetchJson<ContentTemplate[]>(`${SOCIAL_CORE_URL}/projects/${projectId}/social/templates`),

  createTemplate: (projectId: string, template: Partial<ContentTemplate>) =>
    fetchJson<ContentTemplate>(`${SOCIAL_CORE_URL}/projects/${projectId}/social/templates`, {
      method: "POST",
      body: JSON.stringify(template),
    }),

  updateTemplate: (projectId: string, templateId: string, template: Partial<ContentTemplate>) =>
    fetchJson<ContentTemplate>(
      `${SOCIAL_CORE_URL}/projects/${projectId}/social/templates/${templateId}`,
      {
        method: "PUT",
        body: JSON.stringify(template),
      }
    ),

  deleteTemplate: (projectId: string, templateId: string) =>
    fetchJson<{ status: string; template_id: string }>(
      `${SOCIAL_CORE_URL}/projects/${projectId}/social/templates/${templateId}`,
      { method: "DELETE" }
    ),

  fetchAccountStats: (projectId: string) =>
    fetchJson<AccountStats[]>(`${SOCIAL_CORE_URL}/projects/${projectId}/social/accounts/stats`),
};
