"use client";

import { useState, useCallback } from "react";
import { showAlert, confirm } from "@/lib/confirm";
import { videoApi } from "@/lib/api/video";
import { cancelRunOnCore, deleteRunOnCore } from "@/lib/core-api";
import type { CombinedJobItem } from "../types";

interface UseJobActionsParams {
  activeProjectId: string;
  combinedJobs: CombinedJobItem[];
  fetchJobs: () => Promise<void>;
  rememberDismissed: (ids: string[]) => void;
  updateJob: (id: string, patch: any) => void;
  removeJob: (id: string) => void;
}

export function useJobActions({
  activeProjectId,
  combinedJobs,
  fetchJobs,
  rememberDismissed,
  updateJob,
  removeJob,
}: UseJobActionsParams) {
  const [cancellingId, setCancellingId] = useState<string | null>(null);

  const handleCancel = useCallback(
    async (jobId: string) => {
      setCancellingId(jobId);
      try {
        if (jobId.startsWith("run_")) {
          await cancelRunOnCore(activeProjectId);
        } else {
          await videoApi.cancelRenderJob(activeProjectId, jobId);
        }
        await fetchJobs();
        updateJob(jobId, { status: "cancelled" });
      } catch (err: any) {
        await showAlert("Huỷ job thất bại", err.message, "danger");
      } finally {
        setCancellingId(null);
      }
    },
    [activeProjectId, fetchJobs, updateJob]
  );

  /** Xoá job: đang chạy thì dừng trước. Chỉ xoá hồ sơ job, không xoá video/file đã xuất. */
  const handleDelete = useCallback(
    async (job: CombinedJobItem) => {
      const live = job.status === "running" || job.status === "queued";
      const ok = await confirm({
        title: live ? "Dừng và xoá job?" : "Xoá job?",
        message: live
          ? "Job đang chạy sẽ bị dừng rồi xoá khỏi danh sách. File đã xuất vẫn được giữ."
          : "Job sẽ bị xoá khỏi danh sách. File đã xuất vẫn được giữ.",
        tone: "danger",
        confirmLabel: live ? "Dừng & xoá" : "Xoá",
      });
      if (!ok) return;
      try {
        if (live) {
          if (job.id.startsWith("run_")) await cancelRunOnCore(activeProjectId);
          else await videoApi.cancelRenderJob(activeProjectId, job.id).catch(() => undefined);
        }
        if (job.id.startsWith("run_")) await deleteRunOnCore(activeProjectId, job.id);
        removeJob(job.id);
        rememberDismissed([job.id]);
        await fetchJobs();
      } catch (err: any) {
        await showAlert("Xoá job thất bại", err?.message, "danger");
      }
    },
    [activeProjectId, fetchJobs, rememberDismissed, removeJob]
  );

  const handleClearFinished = useCallback(async () => {
    const done = combinedJobs.filter((j) => j.status !== "running" && j.status !== "queued");
    if (done.length === 0) return;
    const ok = await confirm({
      title: `Xoá ${done.length} job đã kết thúc?`,
      message: "Các job đang chạy được giữ nguyên. File đã xuất vẫn được giữ.",
      tone: "danger",
      confirmLabel: "Xoá hết",
    });
    if (!ok) return;
    for (const j of done) {
      if (j.id.startsWith("run_")) await deleteRunOnCore(activeProjectId, j.id);
      removeJob(j.id);
    }
    rememberDismissed(done.map((j) => j.id));
    await fetchJobs();
  }, [activeProjectId, combinedJobs, fetchJobs, rememberDismissed, removeJob]);

  return {
    cancellingId,
    handleCancel,
    handleDelete,
    handleClearFinished,
  };
}
