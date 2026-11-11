import {
  socialApi,
  type PlannedTarget,
  type PublishableVideo,
  type SocialAccount,
  type TargetCommitSpec,
} from "@/lib/api/social";
import { DEFAULT_VIDEO_CAPTION_CONFIG } from "./constants";
import type {
  PlatformCaptionConfig,
  ScheduleStrategy,
  VideoCaptionConfig,
  VideoCaptionsMap,
} from "./types";

export async function generateCaptionsForVideoList(params: {
  projectId: string;
  videosToProcess: PublishableVideo[];
  selectedAccounts: SocialAccount[];
  selectedAccountIds: string[];
  existingCaptions: VideoCaptionsMap;
}): Promise<Record<string, VideoCaptionConfig>> {
  const {
    projectId,
    videosToProcess,
    selectedAccounts,
    selectedAccountIds,
    existingCaptions,
  } = params;
  const targetPlatforms = Array.from(new Set(selectedAccounts.map((a) => a.platform)));
  const updates: Record<string, VideoCaptionConfig> = {};

  for (const v of videosToProcess) {
    if (!v) continue;
    const base = updates[v.id] || existingCaptions[v.id];
    const currentData: VideoCaptionConfig = base || {
      ...DEFAULT_VIDEO_CAPTION_CONFIG,
      platformData: {},
    };

    const res = await socialApi.generateCaptions(projectId, {
      video_id: v.id,
      platforms: targetPlatforms,
      tone: currentData.tone,
      goal: currentData.goal,
      include_affiliate: currentData.includeAffiliate,
      affiliate_link: currentData.affiliateLink,
      vary_per_account: currentData.varyPerAccount,
      account_ids: selectedAccountIds,
    });

    const newPlatformData: Record<string, PlatformCaptionConfig> = {
      ...currentData.platformData,
    };
    let firstCaption = "";
    res.forEach((r) => {
      if (!firstCaption) firstCaption = r.caption;
      const key = r.account_id || r.platform;
      newPlatformData[key] = {
        title: r.title || v.title,
        caption: r.caption,
        hashtags: r.hashtags,
        privacy: "public",
        madeForKids: false,
        aiGeneratedLabel: false,
        firstComment: "",
      };
    });

    updates[v.id] = {
      ...currentData,
      mainCaption: firstCaption || currentData.mainCaption,
      platformData: newPlatformData,
    };
  }

  return updates;
}

export function buildCommitTargetSpecs(
  previewTargets: PlannedTarget[],
  videoCaptions: VideoCaptionsMap
): TargetCommitSpec[] {
  return previewTargets.map((pt) => {
    const vCap = videoCaptions[pt.video_id];
    const pData =
      vCap?.platformData?.[pt.account_id] || vCap?.platformData?.[pt.platform];

    return {
      video_id: pt.video_id,
      account_id: pt.account_id,
      scheduled_at: pt.scheduled_at,
      title: pData?.title || pt.video_title,
      caption: pData?.caption || vCap?.mainCaption || pt.video_title,
      hashtags: pData?.hashtags || [],
      privacy: pData?.privacy || "public",
      made_for_kids: pData?.madeForKids || false,
      ai_generated_label: pData?.aiGeneratedLabel || false,
      firstComment: pData?.firstComment || "",
      product_link: vCap?.affiliateLink || "",
      disclosure_ad: vCap?.includeAffiliate || false,
    };
  });
}

export async function requestPlanPreview(params: {
  projectId: string;
  videoIds: string[];
  accountIds: string[];
  strategy: ScheduleStrategy;
  scheduleStartDate: string;
  perDayLimit: number;
}): Promise<PlannedTarget[]> {
  return socialApi.previewPlan(params.projectId, {
    video_ids: params.videoIds,
    account_ids: params.accountIds,
    strategy: params.strategy,
    start_at: new Date(params.scheduleStartDate).toISOString(),
    per_day_limit: params.perDayLimit,
  });
}
