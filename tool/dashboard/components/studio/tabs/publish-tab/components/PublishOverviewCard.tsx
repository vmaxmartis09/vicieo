"use client";

import React from "react";
import type { HubView } from "@/store/modules/social";
import { Calendar, RefreshCw, Zap } from "@/components/ui/icons";
import { cn } from "@/lib/utils";
import type { PublishTabBucket } from "../types";
import type { QueueSummaryCounts } from "../utils/publishQueueMappers";

interface PublishOverviewCardProps {
  accountsCount: number;
  loadingLearnings: boolean;
  counts: QueueSummaryCounts;
  onRefreshAll: () => void;
  onOpenHubView: (view: HubView, bucket?: PublishTabBucket) => void;
}

export function PublishOverviewCard({
  accountsCount,
  loadingLearnings,
  counts,
  onRefreshAll,
  onOpenHubView,
}: PublishOverviewCardProps) {
  return (
    <div className="rounded-card border border-border bg-gradient-to-br from-card to-muted/40 p-4 shadow-2xs space-y-3">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <div className="flex h-7 w-7 items-center justify-center rounded-ctl bg-primary/10 text-primary">
            <Calendar className="h-4 w-4" />
          </div>
          <div>
            <h4 className="font-bold text-foreground text-footnote leading-none">
              Lịch Đăng Social
            </h4>
            <p className="text-caption text-muted-foreground mt-1 font-mono">
              {accountsCount}/10 tài khoản kết nối
            </p>
          </div>
        </div>

        <button
          type="button"
          onClick={onRefreshAll}
          className="p-1 rounded-ctl text-muted-foreground hover:text-foreground"
          aria-label="Tải lại"
        >
          <RefreshCw className={cn("h-3.5 w-3.5", loadingLearnings && "animate-spin")} />
        </button>
      </div>

      <button
        type="button"
        onClick={() => onOpenHubView("overview")}
        className="w-full h-9 px-3 rounded-ctl bg-primary text-primary-foreground font-semibold text-footnote flex items-center justify-center gap-2 hover:bg-primary/90 transition-all shadow-sm"
      >
        <Zap className="h-4 w-4" />
        Mở Trung tâm Social (Toàn màn hình)
      </button>

      <div className="grid grid-cols-2 gap-2 pt-1">
        <button
          type="button"
          onClick={() => onOpenHubView("queue", "upcoming")}
          className="p-2 rounded-ctl bg-background/80 hover:bg-background border border-border/60 text-left transition-colors flex items-center justify-between"
        >
          <span className="text-caption text-muted-foreground">Sắp tới</span>
          <span className="font-bold text-blue-400 font-mono text-footnote">
            {counts.upcomingCount}
          </span>
        </button>

        <button
          type="button"
          onClick={() => onOpenHubView("queue", "draft")}
          className="p-2 rounded-ctl bg-background/80 hover:bg-background border border-border/60 text-left transition-colors flex items-center justify-between"
        >
          <span className="text-caption text-muted-foreground">Bản nháp</span>
          <span className="font-bold text-zinc-400 font-mono text-footnote">
            {counts.draftCount}
          </span>
        </button>

        <button
          type="button"
          onClick={() => onOpenHubView("queue", "pending_approval")}
          className="p-2 rounded-ctl bg-background/80 hover:bg-background border border-border/60 text-left transition-colors flex items-center justify-between"
        >
          <span className="text-caption text-muted-foreground">Chờ duyệt</span>
          <span className="font-bold text-amber-400 font-mono text-footnote">
            {counts.pendingCount}
          </span>
        </button>

        <button
          type="button"
          onClick={() => onOpenHubView("queue", "overdue")}
          className="p-2 rounded-ctl bg-background/80 hover:bg-background border border-border/60 text-left transition-colors flex items-center justify-between"
        >
          <span className="text-caption text-muted-foreground">Quá hạn/Lỗi</span>
          <span className="font-bold text-rose-400 font-mono text-footnote">
            {counts.overdueOrFailedCount}
          </span>
        </button>
      </div>
    </div>
  );
}
