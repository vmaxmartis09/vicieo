import type { StoreApi } from "zustand";
import type {
  AccountStats,
  AutomationRule,
  CalendarResponse,
  ContentTemplate,
  CreativePerformance,
  HeatmapData,
  KPISummary,
  LibraryVideo,
  OutlierItem,
  PlatformType,
  Post,
  PostStatus,
  QueueResponse,
  SocialAccount,
  SocialNotification,
  SocialOverview,
} from "@/lib/api/social";

export type HubView = "overview" | "library" | "queue" | "calendar" | "accounts" | "templates";
export type HubQueueBucket = "draft" | "pending_approval" | "upcoming" | "overdue" | "published" | "failed";

export interface SocialFilters {
  platform: PlatformType | "all";
  status: PostStatus | "all";
  accountId: string | "all";
  search: string;
}

export interface SocialState {
  // Data
  accounts: SocialAccount[];
  posts: Post[];
  calendarData: CalendarResponse | null;
  queueData: QueueResponse | null;
  queueBucket: HubQueueBucket;
  loading: boolean;
  error: string | null;

  // Social Hub state
  isHubOpen: boolean;
  hubView: HubView;
  hubSelectedEntity: { type: "video" | "account" | "post" | "target" | "template"; id?: string; data: any } | null;
  hubPreselectedVideoIds: string[];
  overviewData: SocialOverview | null;
  libraryVideos: LibraryVideo[];
  templates: ContentTemplate[];
  accountsStats: AccountStats[];

  // Analytics & Automation Data
  analyticsOverview: {
    kpis: KPISummary;
    views_by_day: Record<string, Record<string, number>>;
    outliers: { top_outliers: OutlierItem[]; underperforming: OutlierItem[] };
  } | null;
  heatmapData: HeatmapData | null;
  creativesData: CreativePerformance | null;
  learningsData: { learnings_md: string; recommendations: any } | null;
  automationRules: AutomationRule[];
  notifications: SocialNotification[];
  unreadNotifCount: number;

  // View state
  activeTab: "calendar" | "approvals" | "posts" | "accounts" | "analytics";
  calendarView: "month" | "week" | "agenda";
  selectedDate: string; // YYYY-MM-DD
  filters: SocialFilters;

  // Modals & Drawers state
  isComposerOpen: boolean;
  editingPost: Post | null;
  composerInitialDate: string | null;
  composerInitialAccount: string | null;
  isPublishCenterOpen: boolean;
  publishCenterTab: "plan" | "calendar";
  isAutomationModalOpen: boolean;
  isNotifDrawerOpen: boolean;
  isManualMetricsModalOpen: boolean;
  isImportCsvModalOpen: boolean;

  // History for Undo
  lastDragAction: { postId: string; targetId: string; previousTime: string | null } | null;

  // Actions
  setActiveTab: (tab: "calendar" | "approvals" | "posts" | "accounts" | "analytics") => void;
  setCalendarView: (view: "month" | "week" | "agenda") => void;
  setSelectedDate: (date: string) => void;
  setFilters: (filters: Partial<SocialFilters>) => void;
  setQueueBucket: (bucket: HubQueueBucket) => void;
  openComposer: (post?: Post | null, date?: string, accountId?: string) => void;
  closeComposer: () => void;
  openPublishCenter: (tab?: "plan" | "calendar") => void;
  closePublishCenter: () => void;
  setAutomationModalOpen: (open: boolean) => void;
  setNotifDrawerOpen: (open: boolean) => void;
  setManualMetricsModalOpen: (open: boolean) => void;
  setImportCsvModalOpen: (open: boolean) => void;

  // Social Hub actions
  openHub: (opts?: { view?: HubView; videoIds?: string[]; selectedEntity?: any }) => void;
  closeHub: () => void;
  setHubView: (view: HubView) => void;
  setHubSelectedEntity: (entity: any) => void;
  loadOverview: (projectId: string) => Promise<void>;
  loadLibrary: (projectId: string, params?: any) => Promise<void>;
  loadTemplates: (projectId: string) => Promise<void>;
  loadAccountsStats: (projectId: string) => Promise<void>;
  bulkAction: (projectId: string, ids: string[], action: string, params?: any) => Promise<any>;

  // Async API Actions
  loadAccounts: (projectId: string) => Promise<void>;
  loadPosts: (projectId: string) => Promise<void>;
  loadQueue: (projectId: string, bucket?: HubQueueBucket) => Promise<void>;
  loadCalendar: (projectId: string, fromDate?: string, toDate?: string) => Promise<void>;
  loadAll: (projectId: string) => Promise<void>;

  // Analytics Actions
  loadAnalytics: (projectId: string, window?: string) => Promise<void>;
  loadHeatmap: (projectId: string) => Promise<void>;
  applyGoldenWindows: (projectId: string, accountId?: string) => Promise<void>;
  loadCreatives: (projectId: string) => Promise<void>;
  loadLearnings: (projectId: string) => Promise<void>;
  triggerAnalyticsCollect: (projectId: string) => Promise<void>;

  // Automation Actions
  loadAutomationRules: (projectId: string) => Promise<void>;
  toggleAutomationRule: (projectId: string, ruleId: string) => Promise<void>;
  runAutomation: (projectId: string, ruleId?: string) => Promise<void>;

  // Notification Actions
  loadNotifications: (projectId: string) => Promise<void>;
  markNotificationRead: (projectId: string, notifId: string) => Promise<void>;
  clearNotifications: (projectId: string) => Promise<void>;

  // Optimistic Target Time & Undo
  moveTargetTime: (
    projectId: string,
    postId: string,
    targetId: string,
    newTime: string
  ) => Promise<void>;
  undoLastDrag: (projectId: string) => Promise<void>;
}

export type SocialStoreSet = StoreApi<SocialState>["setState"];
export type SocialStoreGet = StoreApi<SocialState>["getState"];
