import { useCallback, useEffect, useState } from "react";
import { showAlert } from "@/lib/confirm";
import { socialApi, type PlannedTarget, type PublishableVideo, type SocialAccount } from "@/lib/api/social";
import { useSocialStore } from "@/store/modules/social";
import { buildCommitTargetSpecs, generateCaptionsForVideoList, requestPlanPreview } from "../api";
import type { CommitResult, ScheduleStrategy, VideoCaptionsMap } from "../types";

interface UsePublishPlanWorkflowParams {
  projectId: string;
  currentStep: number;
  selectedVideoIds: string[];
  selectedVideos: PublishableVideo[];
  selectedAccountIds: string[];
  selectedAccounts: SocialAccount[];
}

export function usePublishPlanWorkflow({
  projectId,
  currentStep,
  selectedVideoIds,
  selectedVideos,
  selectedAccountIds,
  selectedAccounts,
}: UsePublishPlanWorkflowParams) {
  const loadAll = useSocialStore((s) => s.loadAll);

  const [activeVideoIndex, setActiveVideoIndex] = useState<number>(0);
  const [videoCaptions, setVideoCaptions] = useState<VideoCaptionsMap>({});
  const [isGeneratingCaptions, setIsGeneratingCaptions] = useState<boolean>(false);

  const [scheduleStrategy, setScheduleStrategy] = useState<ScheduleStrategy>("auto");
  const [scheduleStartDate, setScheduleStartDate] = useState<string>(() => {
    const d = new Date();
    d.setMinutes(d.getMinutes() + 15);
    return d.toISOString().slice(0, 16);
  });
  const [perDayLimit] = useState<number>(3);
  const [previewTargets, setPreviewTargets] = useState<PlannedTarget[]>([]);
  const [isLoadingPreview, setIsLoadingPreview] = useState<boolean>(false);

  const [approvalRequired, setApprovalRequired] = useState<boolean>(true);
  const [isSubmittingPlan, setIsSubmittingPlan] = useState<boolean>(false);
  const [commitResult, setCommitResult] = useState<CommitResult | null>(null);

  const handleGenerateCaptions = useCallback(
    async (forAllVideos = false) => {
      setIsGeneratingCaptions(true);
      try {
        const videosToProcess = forAllVideos ? selectedVideos : [selectedVideos[activeVideoIndex]];
        const updates = await generateCaptionsForVideoList({
          projectId,
          videosToProcess,
          selectedAccounts,
          selectedAccountIds,
          existingCaptions: videoCaptions,
        });
        setVideoCaptions((prev) => ({ ...prev, ...updates }));
      } catch (err: any) {
        await showAlert("Lỗi sinh caption", err.message, "danger");
      } finally {
        setIsGeneratingCaptions(false);
      }
    },
    [projectId, selectedVideos, activeVideoIndex, selectedAccounts, selectedAccountIds, videoCaptions]
  );

  const handleLoadPreview = useCallback(async () => {
    if (selectedVideoIds.length === 0 || selectedAccountIds.length === 0) return;
    setIsLoadingPreview(true);
    try {
      const planned = await requestPlanPreview({
        projectId,
        videoIds: selectedVideoIds,
        accountIds: selectedAccountIds,
        strategy: scheduleStrategy,
        scheduleStartDate,
        perDayLimit,
      });
      setPreviewTargets(planned);
    } catch (err: any) {
      await showAlert("Lỗi xem trước lịch", err.message, "danger");
    } finally {
      setIsLoadingPreview(false);
    }
  }, [projectId, selectedVideoIds, selectedAccountIds, scheduleStrategy, scheduleStartDate, perDayLimit]);

  useEffect(() => {
    if (currentStep === 4) {
      handleLoadPreview();
    }
  }, [currentStep, scheduleStrategy, perDayLimit]);

  const handleCommitPlan = useCallback(async () => {
    setIsSubmittingPlan(true);
    try {
      const commitSpecs = buildCommitTargetSpecs(previewTargets, videoCaptions);
      const res = await socialApi.commitPlan(projectId, commitSpecs, approvalRequired);
      setCommitResult({
        postsCount: res.created_posts_count,
        targetsCount: res.created_targets_count,
      });
      await loadAll(projectId);
    } catch (err: any) {
      await showAlert("Lỗi lưu lịch xuất bản", err.message, "danger");
    } finally {
      setIsSubmittingPlan(false);
    }
  }, [projectId, previewTargets, videoCaptions, approvalRequired, loadAll]);

  return {
    activeVideoIndex, setActiveVideoIndex, videoCaptions, setVideoCaptions,
    isGeneratingCaptions, handleGenerateCaptions, scheduleStrategy, setScheduleStrategy,
    scheduleStartDate, setScheduleStartDate, previewTargets, setPreviewTargets,
    isLoadingPreview, handleLoadPreview, approvalRequired, setApprovalRequired,
    isSubmittingPlan, commitResult, handleCommitPlan,
  };
}
