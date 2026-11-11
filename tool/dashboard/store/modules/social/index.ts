import { create } from "zustand";
import type { SocialState } from "./types";
import { createViewActions } from "./actions/viewActions";
import { createHubDataActions } from "./actions/hubDataActions";
import { createAnalyticsAutomationActions } from "./actions/analyticsAutomationActions";
import { createSchedulingActions } from "./actions/schedulingActions";

export type { HubView, HubQueueBucket, SocialFilters, SocialState } from "./types";

export const useSocialStore = create<SocialState>((set, get) => ({
  accounts: [],
  posts: [],
  calendarData: null,
  queueData: null,
  queueBucket: "upcoming",
  loading: false,
  error: null,

  analyticsOverview: null,
  heatmapData: null,
  creativesData: null,
  learningsData: null,
  automationRules: [],
  notifications: [],
  unreadNotifCount: 0,

  activeTab: "calendar",
  calendarView: "month",
  selectedDate: new Date().toISOString().slice(0, 10),
  filters: {
    platform: "all",
    status: "all",
    accountId: "all",
    search: "",
  },

  isComposerOpen: false,
  editingPost: null,
  composerInitialDate: null,
  composerInitialAccount: null,
  isPublishCenterOpen: false,
  publishCenterTab: "plan",
  isAutomationModalOpen: false,
  isNotifDrawerOpen: false,
  isManualMetricsModalOpen: false,
  isImportCsvModalOpen: false,
  lastDragAction: null,

  // Social Hub state
  isHubOpen: false,
  hubView: "overview",
  hubSelectedEntity: null,
  hubPreselectedVideoIds: [],
  overviewData: null,
  libraryVideos: [],
  templates: [],
  accountsStats: [],

  ...createViewActions(set),
  ...createHubDataActions(set, get),
  ...createAnalyticsAutomationActions(set, get),
  ...createSchedulingActions(set, get),
}));
