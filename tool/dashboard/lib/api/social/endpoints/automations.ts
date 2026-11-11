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

export const automationsEndpoints = {
// Automation Rules
  listAutomationRules: (projectId: string) =>
    fetchJson<AutomationRule[]>(`${SOCIAL_CORE_URL}/projects/${projectId}/social/automation/rules`),

  saveAutomationRule: (projectId: string, rule: Partial<AutomationRule>) =>
    fetchJson<AutomationRule>(`${SOCIAL_CORE_URL}/projects/${projectId}/social/automation/rules`, {
      method: "POST",
      body: JSON.stringify(rule),
    }),

  toggleAutomationRule: (projectId: string, ruleId: string) =>
    fetchJson<{ id: string; enabled: boolean }>(
      `${SOCIAL_CORE_URL}/projects/${projectId}/social/automation/rules/${ruleId}/toggle`,
      { method: "POST" }
    ),

  deleteAutomationRule: (projectId: string, ruleId: string) =>
    fetchJson<{ status: string; id: string }>(
      `${SOCIAL_CORE_URL}/projects/${projectId}/social/automation/rules/${ruleId}`,
      { method: "DELETE" }
    ),

  runAutomation: (projectId: string, ruleId?: string) => {
    const q = ruleId ? `?rule_id=${ruleId}` : "";
    return fetchJson<{ reports: any[] }>(
      `${SOCIAL_CORE_URL}/projects/${projectId}/social/automation/run${q}`,
      { method: "POST" }
    );
  },

  // Notifications
  listNotifications: (projectId: string, unreadOnly: boolean = false) =>
    fetchJson<SocialNotification[]>(
      `${SOCIAL_CORE_URL}/projects/${projectId}/social/notifications?unread_only=${unreadOnly}`
    ),

  markNotificationRead: (projectId: string, notifId: string) =>
    fetchJson<{ status: string; success: boolean }>(
      `${SOCIAL_CORE_URL}/projects/${projectId}/social/notifications/${notifId}/read`,
      { method: "POST" }
    ),

  clearNotifications: (projectId: string) =>
    fetchJson<{ status: string }>(
      `${SOCIAL_CORE_URL}/projects/${projectId}/social/notifications`,
      { method: "DELETE" }
    ),
};
