import type { SocialState, SocialStoreSet } from "../types";

export function createViewActions(
  set: SocialStoreSet
): Pick<
  SocialState,
  | "setActiveTab"
  | "setCalendarView"
  | "setSelectedDate"
  | "setFilters"
  | "setQueueBucket"
  | "openComposer"
  | "closeComposer"
  | "openPublishCenter"
  | "closePublishCenter"
  | "setAutomationModalOpen"
  | "setNotifDrawerOpen"
  | "setManualMetricsModalOpen"
  | "setImportCsvModalOpen"
  | "openHub"
  | "closeHub"
  | "setHubView"
  | "setHubSelectedEntity"
> {
  return {
    setActiveTab: (tab) => set({ activeTab: tab }),
    setCalendarView: (view) => set({ calendarView: view }),
    setSelectedDate: (date) => set({ selectedDate: date }),
    setFilters: (newFilters) =>
      set((state) => ({ filters: { ...state.filters, ...newFilters } })),
    setQueueBucket: (bucket) => set({ queueBucket: bucket }),

    openComposer: (post = null, date, accountId) =>
      set({
        isComposerOpen: true,
        editingPost: post,
        composerInitialDate: date || null,
        composerInitialAccount: accountId || null,
      }),

    closeComposer: () =>
      set({
        isComposerOpen: false,
        editingPost: null,
        composerInitialDate: null,
        composerInitialAccount: null,
      }),

    openPublishCenter: (tab = "plan") =>
      set({ isPublishCenterOpen: true, publishCenterTab: tab }),

    closePublishCenter: () => set({ isPublishCenterOpen: false }),

    setAutomationModalOpen: (open) => set({ isAutomationModalOpen: open }),
    setNotifDrawerOpen: (open) => set({ isNotifDrawerOpen: open }),
    setManualMetricsModalOpen: (open) => set({ isManualMetricsModalOpen: open }),
    setImportCsvModalOpen: (open) => set({ isImportCsvModalOpen: open }),

    openHub: (opts) =>
      set({
        isHubOpen: true,
        hubView: opts?.view || "overview",
        hubPreselectedVideoIds: opts?.videoIds || [],
        hubSelectedEntity: opts?.selectedEntity || null,
      }),
    closeHub: () => set({ isHubOpen: false }),
    setHubView: (hubView) => set({ hubView }),
    setHubSelectedEntity: (hubSelectedEntity) => set({ hubSelectedEntity }),
  };
}
