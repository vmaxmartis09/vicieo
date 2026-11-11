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

export const calendarEndpoints = {
// Calendar
  getCalendar: (projectId: string, params?: { from?: string; to?: string; view?: string }) => {
    const query = new URLSearchParams(params as Record<string, string> || {}).toString();
    return fetchJson<CalendarResponse>(`${SOCIAL_CORE_URL}/projects/${projectId}/social/calendar?${query}`);
  },

  // Slots & Notes
  listSlots: (projectId: string, accountId?: string) =>
    fetchJson<RecurringSlot[]>(
      `${SOCIAL_CORE_URL}/projects/${projectId}/social/slots${accountId ? `?account_id=${accountId}` : ""}`
    ),

  saveSlots: (projectId: string, slots: RecurringSlot[]) =>
    fetchJson<RecurringSlot[]>(`${SOCIAL_CORE_URL}/projects/${projectId}/social/slots`, {
      method: "PUT",
      body: JSON.stringify(slots),
    }),

  listNotes: (projectId: string, from?: string, to?: string) => {
    const query = new URLSearchParams({ ...(from ? { from } : {}), ...(to ? { to } : {}) }).toString();
    return fetchJson<CalendarNote[]>(`${SOCIAL_CORE_URL}/projects/${projectId}/social/notes?${query}`);
  },

  saveNotes: (projectId: string, notes: CalendarNote[]) =>
    fetchJson<CalendarNote[]>(`${SOCIAL_CORE_URL}/projects/${projectId}/social/notes`, {
      method: "PUT",
      body: JSON.stringify(notes),
    }),

  // Helpers
  suggestTime: (projectId: string, accountId: string, after?: string) =>
    fetchJson<{ account_id: string; suggested_time: string }>(
      `${SOCIAL_CORE_URL}/projects/${projectId}/social/suggest-time`,
      {
        method: "POST",
        body: JSON.stringify({ account_id: accountId, after }),
      }
    ),

  generateCaptionVariants: (
    projectId: string,
    data: {
      title: string;
      caption: string;
      hashtags: string[];
      platforms: string[];
      product_link?: string;
      footer?: string;
    }
  ) =>
    fetchJson<{ variants: Record<string, any> }>(
      `${SOCIAL_CORE_URL}/projects/${projectId}/social/caption-variants`,
      {
        method: "POST",
        body: JSON.stringify(data),
      }
    ),
};
