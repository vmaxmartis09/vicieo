import { socialApi } from "@/lib/api/social";
import { countUnreadNotifications } from "../utils/socialHelpers";
import type { SocialState, SocialStoreGet, SocialStoreSet } from "../types";

export function createHubDataActions(
  set: SocialStoreSet,
  get: SocialStoreGet
): Pick<
  SocialState,
  | "loadOverview"
  | "loadLibrary"
  | "loadTemplates"
  | "loadAccountsStats"
  | "bulkAction"
  | "loadAccounts"
  | "loadPosts"
  | "loadQueue"
  | "loadCalendar"
  | "loadAll"
> {
  return {
    loadOverview: async (projectId) => {
      try {
        const data = await socialApi.fetchOverview(projectId);
        set({ overviewData: data });
      } catch (e: any) {
        console.error("Failed to load overview:", e);
      }
    },

    loadLibrary: async (projectId, params) => {
      try {
        const data = await socialApi.fetchLibrary(projectId, params);
        set({ libraryVideos: data });
      } catch (e: any) {
        console.error("Failed to load library:", e);
      }
    },

    loadTemplates: async (projectId) => {
      try {
        const data = await socialApi.fetchTemplates(projectId);
        set({ templates: data });
      } catch (e: any) {
        console.error("Failed to load templates:", e);
      }
    },

    loadAccountsStats: async (projectId) => {
      try {
        const data = await socialApi.fetchAccountStats(projectId);
        set({ accountsStats: data });
      } catch (e: any) {
        console.error("Failed to load accounts stats:", e);
      }
    },

    bulkAction: async (projectId, ids, action, params) => {
      const res = await socialApi.bulkAction(projectId, { ids, action, params });
      await get().loadQueue(projectId, get().queueBucket);
      await get().loadOverview(projectId);
      return res;
    },

    loadAccounts: async (projectId) => {
      try {
        const accounts = await socialApi.listAccounts(projectId);
        set({ accounts });
      } catch (e: any) {
        set({ error: e.message });
      }
    },

    loadPosts: async (projectId) => {
      try {
        const posts = await socialApi.listPosts(projectId);
        set({ posts });
      } catch (e: any) {
        set({ error: e.message });
      }
    },

    loadQueue: async (projectId, bucket) => {
      try {
        const b = bucket || get().queueBucket || "upcoming";
        const queueData = await socialApi.getQueue(projectId, b);
        set({ queueData, queueBucket: b });
      } catch (e: any) {
        set({ error: e.message });
      }
    },

    loadCalendar: async (projectId, fromDate, toDate) => {
      try {
        const { calendarView } = get();
        const calendarData = await socialApi.getCalendar(projectId, {
          from: fromDate,
          to: toDate,
          view: calendarView,
        });
        set({ calendarData });
      } catch (e: any) {
        set({ error: e.message });
      }
    },

    loadAll: async (projectId) => {
      set({ loading: true, error: null });
      try {
        const currentBucket = get().queueBucket || "upcoming";
        const [accounts, posts, calendarData, notifs, queueData] = await Promise.all([
          socialApi.listAccounts(projectId).catch(() => []),
          socialApi.listPosts(projectId).catch(() => []),
          socialApi.getCalendar(projectId, { view: get().calendarView }).catch(() => null),
          socialApi.listNotifications(projectId).catch(() => []),
          socialApi.getQueue(projectId, currentBucket).catch(() => null),
        ]);
        set({
          accounts,
          posts,
          calendarData,
          notifications: notifs,
          unreadNotifCount: countUnreadNotifications(notifs),
          queueData,
          loading: false,
        });
      } catch (e: any) {
        set({ error: e.message, loading: false });
      }
    },
  };
}
