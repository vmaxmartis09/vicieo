export type PlatformType = "youtube" | "tiktok" | "facebook" | "instagram" | "douyin" | "threads" | "x";
export type PostStatus = "draft" | "pending_approval" | "approved" | "scheduled" | "publishing" | "published" | "failed" | "cancelled";

export interface PostingWindow {
  weekday: number;
  start: string;
  end: string;
}

export interface AccountAuth {
  mode: "oauth" | "manual" | "browser_assisted";
  token_ref?: string | null;
  scopes?: string[];
  expires_at?: string | null;
}

export interface SocialAccount {
  id: string;
  platform: PlatformType;
  handle: string;
  display_name: string;
  avatar?: string;
  channel_url?: string;
  auth: AccountAuth;
  auth_mode?: "oauth" | "manual";
  status: "connected" | "expired" | "manual" | "disabled";
  groups?: string[];
  group?: string;
  default_hashtags: string[];
  default_caption_footer: string;
  posting_windows: PostingWindow[];
  golden_windows?: string[];
  daily_limit: number;
  limits?: { daily: number };
  min_gap_minutes?: number;
  color?: string;
  affiliate_default?: { link: string; disclosure: boolean };
  tags?: string[];
}

export interface Issue {
  level: "error" | "warn";
  code: string;
  message_vi: string;
}

export interface PublishableVideo {
  id: string;
  title: string;
  thumb?: string;
  duration_s: number;
  duration_sec?: number;
  width: number;
  height: number;
  size_mb: number;
  aspect_ratio: string;
  variants: Record<string, string>;
  primary_path: string;
  source: "job" | "episode" | "library" | "output";
  posted_count: number;
  last_posted_at?: string | null;
  script_excerpt?: string;
}

export interface PlannedTarget {
  video_id: string;
  video_title: string;
  account_id: string;
  account_name: string;
  platform: PlatformType;
  scheduled_at: string;
  issues: Issue[];
  variant_used?: string | null;
  media_path?: string | null;
}

export interface PlanPreviewRequest {
  video_ids: string[];
  account_ids: string[];
  strategy: "now" | "fixed" | "auto";
  start_at?: string;
  end_at?: string;
  per_day_limit?: number;
  caption_mode?: string;
}

export interface TargetCommitSpec {
  video_id: string;
  account_id: string;
  scheduled_at: string;
  title?: string;
  caption?: string;
  hashtags?: string[];
  privacy?: string;
  made_for_kids?: boolean;
  ai_generated_label?: boolean;
  comment_enabled?: boolean;
  duet_enabled?: boolean;
  stitch_enabled?: boolean;
  first_comment?: string;
  product_link?: string;
  disclosure_ad?: boolean;
}

export interface PlanCommitRequest {
  targets: TargetCommitSpec[];
  approval_required: boolean;
}

export interface PlatformCaptionResult {
  platform: PlatformType;
  account_id?: string;
  account_name?: string;
  hook: string;
  caption: string;
  hashtags: string[];
  cta: string;
  title?: string;
  description?: string;
  tags?: string[];
  disclosure_ad: boolean;
  disclosure_text: string;
}

export interface CaptionGenerateRequest {
  video_id: string;
  platforms: string[];
  tone?: string;
  goal?: string;
  include_affiliate?: boolean;
  affiliate_link?: string;
  vary_per_account?: boolean;
  account_ids?: string[];
}

export interface QueueItem {
  post_id: string;
  target_id: string;
  title: string;
  video_title?: string;
  platform: PlatformType;
  account_id: string;
  account_name: string;
  account_avatar: string;
  scheduled_at?: string | null;
  status: string;
  caption: string;
  thumb: string;
  thumbnail?: string;
  error_message?: string;
  bucket: "draft" | "pending_approval" | "upcoming" | "overdue" | "published" | "failed";
  approval_required: boolean;
  approval_status: "approved" | "pending" | "rejected";
  is_manual: boolean;
  has_package: boolean;
  issues: Issue[];
}

export interface QueueSummary {
  accounts_count: number;
  max_accounts: number;
  draft_count: number;
  pending_approval_count: number;
  overdue_count: number;
  upcoming_count: number;
  published_count: number;
  failed_count: number;
  next_post_time?: string | null;
}

export interface QueueResponse {
  items: QueueItem[];
  summary: QueueSummary;
  total?: number;
  page?: number;
  page_size?: number;
  groups?: Record<string, QueueItem[]>;
}

export interface VideoPostTarget {
  account_id: string;
  platform: PlatformType;
  status: string;
  scheduled_at?: string | null;
  published_url?: string;
  post_id: string;
  target_id: string;
  caption?: string;
  hashtags?: string[];
  title?: string;
  thumb?: string;
  error_message?: string;
}

export interface LibraryVideo extends PublishableVideo {
  posts_by_account: VideoPostTarget[];
}

export interface SocialOverview {
  summary: {
    upcoming_24h_count: number;
    overdue_count: number;
    failed_count: number;
    pending_approval_count: number;
    drafts_count?: number;
    token_expiring_count: number;
    empty_slots_7d_count: number;
  };
  upcoming_24h_count?: number;
  overdue_count?: number;
  failed_count?: number;
  pending_approval_count?: number;
  drafts_count?: number;
  expiring_tokens?: Array<{
    id?: string;
    account_id?: string;
    handle?: string;
    display_name?: string;
    platform: PlatformType;
    expires_at: string;
    days_left?: number;
  }>;
  empty_slots?: Array<{
    date: string;
    weekday?: number;
    account_id: string;
    account_name?: string;
    platform?: PlatformType;
    time_slot?: string;
    start?: string;
    end?: string;
  }>;
  upcoming_24h: Array<{
    post_id: string;
    target_id: string;
    title: string;
    platform: PlatformType;
    account_id: string;
    account_name: string;
    account_avatar?: string;
    status: string;
    scheduled_at?: string | null;
    thumb?: string;
    caption?: string;
  }>;
  overdue: Array<{
    post_id: string;
    target_id: string;
    title: string;
    platform: PlatformType;
    account_id: string;
    account_name: string;
    account_avatar?: string;
    status: string;
    scheduled_at?: string | null;
    thumb?: string;
    caption?: string;
  }>;
  failed: Array<{
    post_id: string;
    target_id: string;
    title: string;
    platform: PlatformType;
    account_id: string;
    account_name: string;
    account_avatar?: string;
    status: string;
    scheduled_at?: string | null;
    thumb?: string;
    caption?: string;
  }>;
  pending_approval: Array<{
    post_id: string;
    target_id: string;
    title: string;
    platform: PlatformType;
    account_id: string;
    account_name: string;
    account_avatar?: string;
    status: string;
    scheduled_at?: string | null;
    thumb?: string;
    caption?: string;
  }>;
  token_expiring_accounts: Array<{
    id: string;
    display_name: string;
    platform: PlatformType;
    expires_at: string;
    days_left: number;
  }>;
  empty_slots_7d: Array<{
    date: string;
    weekday: number;
    account_id: string;
    account_name: string;
    platform: PlatformType;
    start: string;
    end: string;
  }>;
}

export interface ContentTemplate {
  id: string;
  name: string;
  title?: string;
  purpose: "sales" | "koc" | "education" | "story" | "general";
  platforms: PlatformType[];
  caption_hook: string;
  caption_body: string;
  caption_cta: string;
  caption_template?: string;
  cta?: string;
  hashtags: string[];
  first_comment: string;
  footer: string;
  disclosure: string;
  created_at?: string;
  updated_at?: string;
}

export interface AccountStats {
  account_id: string;
  posts_this_week: number;
  posts_today: number;
  daily_limit: number;
  quota_remaining_today: number;
  next_post_time?: string | null;
  last_posted_at?: string | null;
}

export interface PostMedia {
  path: string;
  platform_variant?: string | null;
  kind: "video" | "image" | "carousel";
  duration_s: number;
  thumb?: string;
}

export interface PostTarget {
  id: string;
  account_id: string;
  platform: PlatformType;
  scheduled_at?: string | null;
  caption: string;
  hashtags: string[];
  title?: string;
  description?: string;
  tags?: string[];
  privacy?: "public" | "unlisted" | "private";
  made_for_kids?: boolean;
  cover_time_s?: number;
  first_comment?: string;
  product_link?: string;
  disclosure_ad?: boolean;
  status: PostStatus;
  external_id?: string | null;
  external_url?: string | null;
  attempts?: number;
  last_error?: string | null;
  published_at?: string | null;
}

export interface PostApproval {
  required: boolean;
  by?: string | null;
  at?: string | null;
  note?: string | null;
}

export interface Post {
  id: string;
  project_id: string;
  title: string;
  media: PostMedia[];
  source_ref?: string | null;
  render_job_id?: string | null;
  idea_id?: string | null;
  status: PostStatus;
  approval: PostApproval;
  targets: PostTarget[];
  created_at: string;
  updated_at: string;
}

export interface RecurringSlot {
  id: string;
  account_id: string;
  rrule: string;
  label?: string;
  template_hint?: string | null;
}

export interface CalendarNote {
  id: string;
  date: string;
  text: string;
  color: string;
}

export interface KPISummary {
  total_views: number;
  total_watch_time_s: number;
  avg_retention_pct: number;
  engagement_rate: number;
  total_likes: number;
  total_comments: number;
  total_shares: number;
  total_saves: number;
  follower_gain: number;
  avg_ctr: number;
  affiliate_revenue: number;
  post_count: number;
}

export interface GoldenHourSlot {
  weekday: number;
  hour: number;
  median_views: number;
  sample_count: number;
  score: number;
}

export interface HeatmapData {
  matrix: GoldenHourSlot[][];
  top_slots: GoldenHourSlot[];
  recommended_windows: PostingWindow[];
}

export interface OutlierItem {
  post_id: string;
  target_id: string;
  title: string;
  platform: PlatformType;
  views: number;
  benchmark_views: number;
  ratio: number;
  kind: "top_outlier" | "underperforming";
  hypothesis: string;
}

export interface CreativePerformance {
  templates: {
    template_id: string;
    post_count: number;
    total_views: number;
    avg_views: number;
    avg_er: number;
  }[];
  blueprints: {
    blueprint_ref: string;
    post_count: number;
    total_views: number;
    avg_views: number;
    avg_er: number;
  }[];
}

export interface AutomationRule {
  id: string;
  name: string;
  enabled: boolean;
  trigger: "render_done" | "idea_scripted" | "schedule_gap";
  conditions: Record<string, any>;
  actions: Record<string, any>;
  last_run_at?: string | null;
  run_count: number;
  last_log?: string | null;
}

export interface SocialNotification {
  id: string;
  type: "publish_success" | "publish_failed" | "token_expired" | "manual_package_due" | "schedule_gap" | "system";
  title: string;
  message: string;
  created_at: string;
  read: boolean;
  action_url?: string | null;
  data?: Record<string, any>;
}


export interface CalendarItem {
  post_id: string;
  target_id: string;
  title: string;
  platform: PlatformType;
  account_id: string;
  account_name: string;
  scheduled_at?: string | null;
  status: PostStatus;
  caption: string;
  thumb?: string;
  views?: number;
}

export interface EmptySlot {
  account_id: string;
  platform: PlatformType;
  weekday: number;
  start: string;
  end: string;
  label: string;
}

export interface CalendarResponse {
  view: "month" | "week" | "agenda";
  from?: string;
  to?: string;
  items: CalendarItem[];
  empty_slots: EmptySlot[];
  notes: CalendarNote[];
  slots: RecurringSlot[];
}
