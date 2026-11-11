import type { PlatformType, PublishableVideo } from "@/lib/api/social";

export interface PublishCenterModalProps {
  projectId: string;
  isOpen: boolean;
  initialTab?: "plan" | "calendar";
  initialVideoIds?: string[];
  onClose: () => void;
}

export interface PlatformCaptionConfig {
  title?: string;
  caption: string;
  hashtags: string[];
  privacy: string;
  madeForKids: boolean;
  aiGeneratedLabel: boolean;
  firstComment: string;
}

export interface VideoCaptionConfig {
  mainCaption: string;
  tone: string;
  goal: string;
  includeAffiliate: boolean;
  affiliateLink: string;
  varyPerAccount: boolean;
  platformData: Record<string, PlatformCaptionConfig>;
}

export type VideoCaptionsMap = Record<string, VideoCaptionConfig>;

export type ScheduleStrategy = "auto" | "now" | "fixed";

export interface CompatibilityCell {
  status: "ok" | "warn" | "error";
  message?: string;
}

export type CompatibilityMatrix = Record<string, Record<string, CompatibilityCell>>;

export interface CommitResult {
  postsCount: number;
  targetsCount: number;
}

export type WizardMainTab = "plan" | "calendar";

export interface CaptionPreviewSummary {
  platform: PlatformType;
  video: PublishableVideo | undefined;
  captionConfig: VideoCaptionConfig | null;
}
