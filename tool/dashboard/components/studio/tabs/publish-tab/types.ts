import type { KPISummary } from "@/lib/api/social";

export type PublishTabBucket =
  | "upcoming"
  | "draft"
  | "pending_approval"
  | "overdue"
  | "failed";

export interface PackagePostSelection {
  post_id: string;
  target_id: string;
  title: string;
  platform: string;
  caption?: string;
}

export interface PublishedTargetOption {
  target_id: string;
  post_id: string;
  title: string;
  platform: string;
}

export interface LearningsState {
  kpis?: KPISummary;
  recommendations?: any;
  learnings_md?: string;
}
