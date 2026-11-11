"use client";

import React from "react";
import { Trash2 } from "@/components/ui/icons";
import { cn } from "@/lib/utils";
import type { JobCounts } from "../utils/mergeJobsList";
import type { CombinedJobItem, FilterTab } from "../types";
import { JobQueueCard } from "./JobQueueCard";

interface JobsQueueSectionProps {
  filter: FilterTab;
  onChangeFilter: (next: FilterTab) => void;
  counts: JobCounts;
  hasFinishedJobs: boolean;
  filteredJobs: CombinedJobItem[];
  activeProjectId: string;
  cancellingId: string | null;
  onClearFinished: () => void;
  onSelectNode: (nodeId: string) => void;
  onCancel: (jobId: string) => void;
  onDelete: (job: CombinedJobItem) => void;
  onSchedulePost: (job: CombinedJobItem) => void;
}

export function JobsQueueSection({
  filter,
  onChangeFilter,
  counts,
  hasFinishedJobs,
  filteredJobs,
  activeProjectId,
  cancellingId,
  onClearFinished,
  onSelectNode,
  onCancel,
  onDelete,
  onSchedulePost,
}: JobsQueueSectionProps) {
  const tabs: Array<{ id: FilterTab; label: string }> = [
    { id: "all", label: `Tất cả (${counts.all})` },
    { id: "running", label: `Chạy (${counts.running})` },
    { id: "completed", label: `Xong (${counts.completed})` },
    { id: "failed", label: `Lỗi/Huỷ (${counts.failed})` },
  ];

  return (
    <>
      <div className="flex items-center gap-2 border-b border-border pb-2">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            onClick={() => onChangeFilter(tab.id)}
            className={cn(
              "px-3 py-1 rounded-ctl text-caption font-medium transition-colors",
              filter === tab.id
                ? "bg-foreground text-background font-semibold"
                : "text-muted-foreground hover:bg-accent hover:text-foreground"
            )}
          >
            {tab.label}
          </button>
        ))}
      </div>

      {hasFinishedJobs && (
        <div className="flex justify-end">
          <button
            type="button"
            onClick={onClearFinished}
            className="flex items-center gap-1 rounded-ctl px-2 py-1 text-caption text-muted-foreground hover:bg-rose-500/10 hover:text-rose-400"
          >
            <Trash2 className="h-3 w-3" />
            <span>Xoá job đã kết thúc</span>
          </button>
        </div>
      )}

      <div className="space-y-3">
        {filteredJobs.length === 0 ? (
          <div className="py-8 text-center text-muted-foreground text-xs">
            Không có job render nào trong mục này.
          </div>
        ) : (
          filteredJobs.map((j) => (
            <JobQueueCard
              key={j.id}
              job={j}
              activeProjectId={activeProjectId}
              cancellingId={cancellingId}
              onSelectNode={onSelectNode}
              onCancel={onCancel}
              onDelete={onDelete}
              onSchedulePost={onSchedulePost}
            />
          ))
        )}
      </div>
    </>
  );
}
