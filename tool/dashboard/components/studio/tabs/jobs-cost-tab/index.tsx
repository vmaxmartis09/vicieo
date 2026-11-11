"use client";

import React, { useMemo, useState } from "react";
import { useShallow } from "zustand/react/shallow";
import { useStudioStore, type StudioStore } from "@/store";
import { useSocialStore } from "@/store/modules/social";
import type { CombinedJobItem, FilterTab } from "./types";
import {
  computeDailySpendPct,
  computeJobCounts,
  extractJobMediaUrl,
  filterJobsByTab,
  mergeAndSortJobs,
} from "./utils/mergeJobsList";
import { useJobsPolling } from "./hooks/useJobsPolling";
import { useJobActions } from "./hooks/useJobActions";
import { BudgetSummaryCard } from "./components/BudgetSummaryCard";
import { ReadyOutputsSection } from "./components/ReadyOutputsSection";
import { JobsQueueSection } from "./components/JobsQueueSection";

export function JobsCostTab() {
  const {
    jobs,
    activeProjectId,
    totalSpentUsd,
    dailySpendingLimitUsd,
    setActiveRightTab,
    setSelectedNodeId,
    updateJob,
    removeJob,
  } = useStudioStore(
    useShallow((s: StudioStore) => ({
      jobs: s.jobs,
      activeProjectId: s.activeProjectId,
      totalSpentUsd: s.totalSpentUsd,
      dailySpendingLimitUsd: s.dailySpendingLimitUsd,
      setActiveRightTab: s.setActiveRightTab,
      setSelectedNodeId: s.setSelectedNodeId,
      updateJob: s.updateJob,
      removeJob: s.removeJob,
    }))
  );

  const openComposer = useSocialStore((s) => s.openComposer);
  const [filter, setFilter] = useState<FilterTab>("all");

  const hasActiveStoreJobs = jobs.some((j) => j.status === "running" || j.status === "queued");
  const { remoteJobs, coreRuns, outputsList, dismissed, rememberDismissed, fetchJobs } =
    useJobsPolling(activeProjectId, hasActiveStoreJobs);

  const combinedJobs = useMemo(
    () => mergeAndSortJobs(coreRuns, remoteJobs, jobs, dismissed),
    [coreRuns, remoteJobs, jobs, dismissed]
  );

  const filteredJobs = useMemo(
    () => filterJobsByTab(combinedJobs, filter),
    [combinedJobs, filter]
  );

  const counts = useMemo(() => computeJobCounts(combinedJobs), [combinedJobs]);
  const hasFinishedJobs = combinedJobs.some(
    (j) => j.status !== "running" && j.status !== "queued"
  );

  const { cancellingId, handleCancel, handleDelete, handleClearFinished } = useJobActions({
    activeProjectId,
    combinedJobs,
    fetchJobs,
    rememberDismissed,
    updateJob,
    removeJob,
  });

  const handleSchedulePost = (job: CombinedJobItem) => {
    const mediaUrl = extractJobMediaUrl(job.outputs);
    openComposer({
      id: `post_from_${job.id}`,
      title: `Video #${job.id.slice(0, 8)}`,
      media: [{ type: "video", path: mediaUrl }],
      targets: [],
      status: "draft",
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    } as any);
    setActiveRightTab("publish");
  };

  const handleScheduleOutput = (out: any) => {
    const mediaUrl = out.files?.video || `output/${out.slug}/final_45s.mp4`;
    openComposer({
      id: `post_from_${out.slug}`,
      title: out.title || `Video ${out.slug}`,
      media: [{ type: "video", path: mediaUrl }],
      targets: [],
      status: "draft",
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    } as any);
    setActiveRightTab("publish");
  };

  const handleSelectNode = (nodeId: string) => {
    setSelectedNodeId(nodeId);
    setActiveRightTab("node");
  };

  const pctDaily = computeDailySpendPct(totalSpentUsd, dailySpendingLimitUsd);

  return (
    <div className="p-4 space-y-4 text-footnote select-none">
      <BudgetSummaryCard
        totalSpentUsd={totalSpentUsd}
        dailySpendingLimitUsd={dailySpendingLimitUsd}
        pctDaily={pctDaily}
      />

      <ReadyOutputsSection
        activeProjectId={activeProjectId}
        outputsList={outputsList}
        onScheduleOutput={handleScheduleOutput}
      />

      <JobsQueueSection
        filter={filter}
        onChangeFilter={setFilter}
        counts={counts}
        hasFinishedJobs={hasFinishedJobs}
        filteredJobs={filteredJobs}
        activeProjectId={activeProjectId}
        cancellingId={cancellingId}
        onClearFinished={handleClearFinished}
        onSelectNode={handleSelectNode}
        onCancel={handleCancel}
        onDelete={handleDelete}
        onSchedulePost={handleSchedulePost}
      />
    </div>
  );
}
