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
  OutlierItem,
  PostingWindow,
} from "../types";

export const analyticsEndpoints = {
// Analytics & Insights
  getAnalyticsOverview: (
    projectId: string,
    window: string = "30d",
    platform?: string,
    accountId?: string
  ) => {
    const params = new URLSearchParams({ window });
    if (platform) params.append("platform", platform);
    if (accountId) params.append("account_id", accountId);
    return fetchJson<{
      kpis: KPISummary;
      views_by_day: Record<string, Record<string, number>>;
      outliers: { top_outliers: OutlierItem[]; underperforming: OutlierItem[] };
    }>(`${SOCIAL_CORE_URL}/projects/${projectId}/social/analytics/overview?${params}`);
  },

  getAnalyticsHeatmap: (projectId: string) =>
    fetchJson<HeatmapData>(`${SOCIAL_CORE_URL}/projects/${projectId}/social/analytics/heatmap`),

  applyGoldenWindows: (projectId: string, accountId?: string, windows?: PostingWindow[]) =>
    fetchJson<{ status: string; updated_accounts: string[]; windows: PostingWindow[] }>(
      `${SOCIAL_CORE_URL}/projects/${projectId}/social/analytics/apply-golden-windows`,
      {
        method: "POST",
        body: JSON.stringify({ account_id: accountId, windows }),
      }
    ),

  getCreativePerformance: (projectId: string) =>
    fetchJson<CreativePerformance>(`${SOCIAL_CORE_URL}/projects/${projectId}/social/analytics/creatives`),

  getAnalyticsLearnings: (projectId: string) =>
    fetchJson<{ learnings_md: string; recommendations: any }>(
      `${SOCIAL_CORE_URL}/projects/${projectId}/social/analytics/learnings`
    ),

  generateInsights: (projectId: string) =>
    fetchJson<{
      status: string;
      date: string;
      kpis: KPISummary;
      learnings_md: string;
      recommendations: any;
    }>(`${SOCIAL_CORE_URL}/projects/${projectId}/social/analytics/generate-insights`, {
      method: "POST",
    }),

  triggerAnalyticsCollect: (projectId: string) =>
    fetchJson<{ status: string; count: number }>(
      `${SOCIAL_CORE_URL}/projects/${projectId}/social/analytics/collect`,
      { method: "POST" }
    ),

  saveManualMetrics: (projectId: string, payload: any) =>
    fetchJson<{ status: string; snapshot: any }>(
      `${SOCIAL_CORE_URL}/projects/${projectId}/social/analytics/manual-metrics`,
      {
        method: "POST",
        body: JSON.stringify(payload),
      }
    ),

  importCsvMetrics: (projectId: string, csvText: string, platform?: string) =>
    fetchJson<{ status: string; total_records: number; matched_count: number; records: any[] }>(
      `${SOCIAL_CORE_URL}/projects/${projectId}/social/analytics/import-csv`,
      {
        method: "POST",
        body: JSON.stringify({ csv_text: csvText, platform: platform || "youtube" }),
      }
    ),
};
