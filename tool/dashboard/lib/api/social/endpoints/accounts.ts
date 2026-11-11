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

export const accountsEndpoints = {
// Accounts
  listAccounts: (projectId: string) =>
    fetchJson<SocialAccount[]>(`${SOCIAL_CORE_URL}/projects/${projectId}/social/accounts`),

  createAccount: (projectId: string, account: Partial<SocialAccount>) =>
    fetchJson<SocialAccount>(`${SOCIAL_CORE_URL}/projects/${projectId}/social/accounts`, {
      method: "POST",
      body: JSON.stringify(account),
    }),

  updateAccount: (projectId: string, accountId: string, account: Partial<SocialAccount>) =>
    fetchJson<SocialAccount>(`${SOCIAL_CORE_URL}/projects/${projectId}/social/accounts/${accountId}`, {
      method: "PUT",
      body: JSON.stringify(account),
    }),

  deleteAccount: (projectId: string, accountId: string) =>
    fetchJson<{ status: string; account_id: string }>(
      `${SOCIAL_CORE_URL}/projects/${projectId}/social/accounts/${accountId}`,
      { method: "DELETE" }
    ),
};
