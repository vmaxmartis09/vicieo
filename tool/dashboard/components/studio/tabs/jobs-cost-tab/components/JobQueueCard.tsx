"use client";

import React from "react";
import { CORE_URL } from "@/lib/repositories/core";
import { CalendarDays, Film, Loader2, Trash2 } from "@/components/ui/icons";
import { cn } from "@/lib/utils";
import type { CombinedJobItem } from "../types";

interface JobQueueCardProps {
  job: CombinedJobItem;
  activeProjectId: string;
  cancellingId: string | null;
  onSelectNode: (nodeId: string) => void;
  onCancel: (jobId: string) => void;
  onDelete: (job: CombinedJobItem) => void;
  onSchedulePost: (job: CombinedJobItem) => void;
}

export function JobQueueCard({
  job: j,
  activeProjectId,
  cancellingId,
  onSelectNode,
  onCancel,
  onDelete,
  onSchedulePost,
}: JobQueueCardProps) {
  const isDone = j.status === "completed";
  const isRunning = j.status === "running" || j.status === "queued";
  const isFailed = j.status === "failed";
  const isCancelled = j.status === "cancelled";
  const outputPlatforms = Object.keys(j.outputs || {});

  return (
    <div
      className={cn(
        "rounded-card border p-3 space-y-2 transition-all",
        isRunning
          ? "border-primary/50 bg-primary/5 shadow-xs"
          : isDone
          ? "border-border bg-card hover:border-border/80"
          : isCancelled
          ? "border-border bg-muted/30"
          : "border-rose-500/20 bg-rose-500/5"
      )}
    >
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <span className="font-mono text-caption font-bold text-foreground">
            #{j.id.slice(0, 10)}
          </span>
          <span
            className={cn(
              "rounded-full px-2 py-1 text-caption font-semibold uppercase tracking-wider",
              isRunning
                ? "bg-sky-500/10 text-semantic-info border border-sky-500/20"
                : isDone
                ? "bg-emerald-500/10 text-semantic-success border border-emerald-500/20"
                : isCancelled
                ? "bg-muted text-muted-foreground border border-border"
                : "bg-rose-500/10 text-semantic-danger border border-rose-500/20"
            )}
          >
            {isRunning && <Loader2 className="mr-1 inline h-2.5 w-2.5 animate-spin" />}
            {j.status}
          </span>
        </div>

        <span className="text-caption text-muted-foreground">
          {new Date(j.createdAt).toLocaleTimeString([], {
            hour: "2-digit",
            minute: "2-digit",
          })}
        </span>
      </div>

      {/* Stage description if present */}
      {j.stage && (
        <p className="text-caption font-medium text-foreground line-clamp-1">
          {j.stage}
        </p>
      )}

      {/* Node Chips: Bấm vào node để focus trên canvas */}
      {j.nodes && j.nodes.length > 0 && (
        <div className="flex flex-wrap gap-1 pt-1">
          {j.nodes.map((nd) => (
            <button
              key={nd.id}
              type="button"
              onClick={() => onSelectNode(nd.id)}
              className={cn(
                "px-2 py-1 rounded-xs text-caption font-mono border transition-all flex items-center gap-1",
                nd.state === "completed" || nd.state === "succeeded"
                  ? "bg-emerald-500/10 border-emerald-500/30 text-emerald-400"
                  : nd.state === "running"
                  ? "bg-amber-500/10 border-amber-500/40 text-amber-300 animate-pulse"
                  : nd.state === "failed"
                  ? "bg-rose-500/10 border-rose-500/30 text-rose-400"
                  : "bg-zinc-800 border-zinc-700 text-zinc-400"
              )}
              title={`Bấm để xem node ${nd.id} trên Canvas`}
            >
              <span>{nd.id}</span>
              {nd.state === "completed" && <span>✓</span>}
              {nd.state === "running" && <Loader2 className="w-2.5 h-2.5 animate-spin inline" />}
            </button>
          ))}
        </div>
      )}

      {/* Progress bar if running */}
      {isRunning && (
        <div className="space-y-1">
          <div className="flex justify-between text-caption text-muted-foreground">
            <span>{j.stage || "Đang xử lý pipeline HyperFrames..."}</span>
            <span>{j.progress}%</span>
          </div>
          <div className="h-1.5 w-full overflow-hidden rounded-xs bg-accent">
            <div
              style={{ width: `${Math.max(5, j.progress)}%` }}
              className="h-full bg-primary transition-[width] duration-300"
            />
          </div>
        </div>
      )}

      {/* Error message if failed */}
      {isFailed && j.error && (
        <p className="text-caption text-semantic-danger line-clamp-2">
          {j.error}
        </p>
      )}

      {/* Platform outputs & Action buttons */}
      <div className="flex items-center justify-between pt-1 border-t border-border/60">
        <div className="flex items-center gap-2">
          {outputPlatforms.map((p) => {
            const rel = j.outputs[p];
            const fullUrl = rel.startsWith("http")
              ? rel
              : `${CORE_URL}/projects/${activeProjectId}/${rel}`;
            return (
              <a
                key={p}
                href={fullUrl}
                target="_blank"
                rel="noreferrer"
                className="flex items-center gap-1 rounded-xs bg-accent px-2 py-1 text-caption uppercase font-mono text-foreground hover:bg-accent/80"
              >
                <Film className="h-3 w-3 text-primary" />
                <span>{p}</span>
              </a>
            );
          })}
        </div>

        <div className="flex items-center gap-2">
          <button
            type="button"
            onClick={() => onDelete(j)}
            title={isRunning ? "Dừng và xoá job" : "Xoá job"}
            aria-label={isRunning ? "Dừng và xoá job" : "Xoá job"}
            className="flex h-7 w-7 items-center justify-center rounded-ctl text-muted-foreground hover:bg-rose-500/10 hover:text-rose-400 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring transition-colors"
          >
            <Trash2 className="h-3.5 w-3.5" />
          </button>
          {isRunning && (
            <button
              type="button"
              onClick={() => onCancel(j.id)}
              disabled={cancellingId === j.id}
              className="rounded-ctl px-2 py-1 text-caption text-rose-400 hover:bg-rose-500/10 font-semibold focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
            >
              {cancellingId === j.id ? "Đang huỷ..." : "Huỷ"}
            </button>
          )}

          {isDone && (
            <button
              type="button"
              onClick={() => onSchedulePost(j)}
              className="flex items-center gap-1 rounded-ctl bg-primary px-3 py-1 text-footnote font-semibold text-primary-foreground hover:bg-primary/90 transition-colors shadow-2xs focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
            >
              <CalendarDays className="h-3.5 w-3.5" />
              <span>Đăng / Lên lịch</span>
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
