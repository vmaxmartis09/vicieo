import React, { Profiler, type ProfilerOnRenderCallback, type ReactNode } from "react";

export interface ProfilerCommitRecord {
  id: string;
  phase: "mount" | "update" | "nested-update";
  actualDuration: number;
  baseDuration: number;
  startTime: number;
  commitTime: number;
}

export interface ProfilerCollector {
  commits: ProfilerCommitRecord[];
  onRender: ProfilerOnRenderCallback;
  reset: () => void;
  totalCommits: (id?: string) => number;
  updateCommits: (id?: string) => number;
  totalActualDurationMs: (id?: string) => number;
}

/**
 * Bộ thu thập commit React `<Profiler>` cho kịch bản đo Section E.3
 * (gõ prompt / kéo node / nối dây) trên bản gốc trước và sau khi tách.
 */
export function createProfilerCollector(): ProfilerCollector {
  const commits: ProfilerCommitRecord[] = [];

  const onRender: ProfilerOnRenderCallback = (
    id,
    phase,
    actualDuration,
    baseDuration,
    startTime,
    commitTime
  ) => {
    commits.push({
      id,
      phase,
      actualDuration,
      baseDuration,
      startTime,
      commitTime,
    });
  };

  const filterById = (id?: string) =>
    id ? commits.filter((c) => c.id === id) : commits;

  return {
    commits,
    onRender,
    reset: () => {
      commits.length = 0;
    },
    totalCommits: (id?: string) => filterById(id).length,
    updateCommits: (id?: string) =>
      filterById(id).filter((c) => c.phase !== "mount").length,
    totalActualDurationMs: (id?: string) =>
      Number(
        filterById(id)
          .reduce((acc, item) => acc + item.actualDuration, 0)
          .toFixed(3)
      ),
  };
}

export function ProfiledRegion({
  id,
  collector,
  children,
}: {
  id: string;
  collector: ProfilerCollector;
  children: ReactNode;
}) {
  return React.createElement(
    Profiler,
    { id, onRender: collector.onRender },
    children
  );
}
