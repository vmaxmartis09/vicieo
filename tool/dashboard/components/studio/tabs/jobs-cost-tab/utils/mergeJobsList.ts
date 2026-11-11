import type { RenderJob } from "@/lib/api/video";
import type { CombinedJobItem, FilterTab } from "../types";

export interface JobCounts {
  all: number;
  running: number;
  completed: number;
  failed: number;
}

/**
 * Hợp nhất 3 nguồn tác vụ (Core Runs, Legacy Video Render Queue, StudioStore jobs),
 * loại bỏ các job đã dismiss và sắp xếp mới nhất trước. Hàm thuần, không phụ thuộc React.
 */
export function mergeAndSortJobs(
  coreRuns: any[],
  remoteJobs: RenderJob[],
  storeJobs: any[],
  dismissedIds: string[]
): CombinedJobItem[] {
  const list: CombinedJobItem[] = [];

  for (const r of coreRuns) {
    const runStatus =
      r.status === "completed"
        ? "completed"
        : r.status === "cancelled" || r.status === "canceled"
        ? "cancelled"
        : r.status === "failed"
        ? "failed"
        : "running";
    const filesObj: Record<string, string> = {};
    if (r.output?.files?.video) {
      filesObj.video = r.output.files.video;
    }
    list.push({
      id: r.runId || r.id,
      model: `Graph Run (${r.mode || "all"})`,
      status: runStatus,
      stage: r.stage,
      progress: r.progressPct ?? (r.progress ? Math.round(r.progress * 100) : 0),
      actualCostUsd: r.actualCostUsd || r.costUsd || 0.0,
      outputs: filesObj,
      createdAt: r.createdAt || r.updatedAt || new Date().toISOString(),
      elapsedSec: r.elapsedSec,
      nodes: r.nodes,
      error: r.error,
    });
  }

  for (const r of remoteJobs) {
    if (!list.some((item) => item.id === r.job_id)) {
      list.push({
        id: r.job_id,
        model: "HyperFrames Render",
        status: r.status === "done" ? "completed" : (r.status as any),
        stage: r.status === "rendering" ? "Đang render video HyperFrames" : undefined,
        progress: r.status === "done" ? 100 : r.status === "rendering" ? 60 : 0,
        actualCostUsd: 0.0,
        outputs: (r.outputs as any) || {},
        createdAt: r.created_at,
        error: r.error,
      });
    }
  }

  for (const s of storeJobs) {
    if (!list.some((item) => item.id === s.id)) {
      list.push({
        id: s.id,
        model: s.model || s.type,
        status: s.status as any,
        progress: s.progress || 0,
        actualCostUsd: s.actualCostUsd || 0,
        outputs: s.outputUrl ? { default: s.outputUrl } : {},
        createdAt: s.createdAt,
        error: s.error,
      });
    }
  }

  return list
    .filter((j) => !dismissedIds.includes(j.id))
    .sort((a, b) => new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime());
}

export function filterJobsByTab(jobs: CombinedJobItem[], filter: FilterTab): CombinedJobItem[] {
  return jobs.filter((j) => {
    if (filter === "running") return j.status === "running" || j.status === "queued";
    if (filter === "completed") return j.status === "completed";
    if (filter === "failed") return j.status === "failed" || j.status === "cancelled";
    return true;
  });
}

export function computeJobCounts(jobs: CombinedJobItem[]): JobCounts {
  return {
    all: jobs.length,
    running: jobs.filter((j) => j.status === "running" || j.status === "queued").length,
    completed: jobs.filter((j) => j.status === "completed").length,
    failed: jobs.filter((j) => j.status === "failed" || j.status === "cancelled").length,
  };
}

export function extractJobMediaUrl(outputs: Record<string, string> | undefined): string {
  const safe = outputs || {};
  return (
    safe.tiktok ||
    safe.shorts ||
    safe.reels ||
    safe.youtube ||
    safe.video ||
    Object.values(safe)[0] ||
    ""
  );
}

export function computeDailySpendPct(totalSpentUsd: number, dailyLimitUsd: number): number {
  return Math.min(100, Math.round((totalSpentUsd / Math.max(0.5, dailyLimitUsd)) * 100));
}
