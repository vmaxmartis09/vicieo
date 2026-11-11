import { socialApi } from "@/lib/api/social";
import { countUnreadNotifications } from "../utils/socialHelpers";
import type { SocialState, SocialStoreGet, SocialStoreSet } from "../types";

export function createAnalyticsAutomationActions(
  set: SocialStoreSet,
  get: SocialStoreGet
): Pick<
  SocialState,
  | "loadAnalytics"
  | "loadHeatmap"
  | "applyGoldenWindows"
  | "loadCreatives"
  | "loadLearnings"
  | "triggerAnalyticsCollect"
  | "loadAutomationRules"
  | "toggleAutomationRule"
  | "runAutomation"
  | "loadNotifications"
  | "markNotificationRead"
  | "clearNotifications"
> {
  return {
    loadAnalytics: async (projectId, window = "30d") => {
      try {
        const overview = await socialApi.getAnalyticsOverview(projectId, window);
        set({ analyticsOverview: overview });
      } catch (e: any) {
        set({ error: e.message });
      }
    },

    loadHeatmap: async (projectId) => {
      try {
        const heatmap = await socialApi.getAnalyticsHeatmap(projectId);
        set({ heatmapData: heatmap });
      } catch (e: any) {
        set({ error: e.message });
      }
    },

    applyGoldenWindows: async (projectId, accountId) => {
      try {
        await socialApi.applyGoldenWindows(projectId, accountId);
        await get().loadAccounts(projectId);
      } catch (e: any) {
        set({ error: e.message });
      }
    },

    loadCreatives: async (projectId) => {
      try {
        const creatives = await socialApi.getCreativePerformance(projectId);
        set({ creativesData: creatives });
      } catch (e: any) {
        set({ error: e.message });
      }
    },

    loadLearnings: async (projectId) => {
      try {
        const learnings = await socialApi.getAnalyticsLearnings(projectId);
        set({ learningsData: learnings });
      } catch (e: any) {
        set({ error: e.message });
      }
    },

    triggerAnalyticsCollect: async (projectId) => {
      try {
        await socialApi.triggerAnalyticsCollect(projectId);
        await Promise.all([
          get().loadAnalytics(projectId),
          get().loadPosts(projectId),
        ]);
      } catch (e: any) {
        set({ error: e.message });
      }
    },

    loadAutomationRules: async (projectId) => {
      try {
        const rules = await socialApi.listAutomationRules(projectId);
        set({ automationRules: rules });
      } catch (e: any) {
        set({ error: e.message });
      }
    },

    toggleAutomationRule: async (projectId, ruleId) => {
      try {
        await socialApi.toggleAutomationRule(projectId, ruleId);
        await get().loadAutomationRules(projectId);
      } catch (e: any) {
        set({ error: e.message });
      }
    },

    runAutomation: async (projectId, ruleId) => {
      try {
        await socialApi.runAutomation(projectId, ruleId);
        await Promise.all([
          get().loadPosts(projectId),
          get().loadNotifications(projectId),
          get().loadAutomationRules(projectId),
        ]);
      } catch (e: any) {
        set({ error: e.message });
      }
    },

    loadNotifications: async (projectId) => {
      try {
        const notifs = await socialApi.listNotifications(projectId);
        set({ notifications: notifs, unreadNotifCount: countUnreadNotifications(notifs) });
      } catch (e: any) {
        set({ error: e.message });
      }
    },

    markNotificationRead: async (projectId, notifId) => {
      try {
        await socialApi.markNotificationRead(projectId, notifId);
        await get().loadNotifications(projectId);
      } catch (e: any) {
        set({ error: e.message });
      }
    },

    clearNotifications: async (projectId) => {
      try {
        await socialApi.clearNotifications(projectId);
        set({ notifications: [], unreadNotifCount: 0 });
      } catch (e: any) {
        set({ error: e.message });
      }
    },
  };
}
