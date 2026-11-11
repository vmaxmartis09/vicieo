import type { Platform } from "@/lib/api/video";
import type { SocialPlatformId } from "@/lib/socialConfig";

export interface CreateVideoModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export interface ProductFormInfo {
  title: string;
  name: string;
  price: string;
  usp: string;
  image: string;
}

export interface HookVariantItem {
  variant_id: string;
  text: string;
  score?: {
    total_score?: number;
  };
}

export interface LintIssueItem {
  code?: string;
  level: string;
  message_vi: string;
}

export interface LintReport {
  passed: boolean;
  error_count: number;
  warning_count: number;
  total_checks: number;
  issues?: LintIssueItem[];
}

export function resolveSocialKey(plat: Platform): SocialPlatformId {
  return (plat === "shorts" ? "youtube" : plat === "reels" ? "instagram" : plat) as SocialPlatformId;
}

export function parseBatchTopics(text: string): string[] {
  return text
    .split("\n")
    .map((l) => l.trim())
    .filter((l) => l.length > 0);
}
