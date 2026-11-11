"use client";

import React, { useState } from "react";
import { FileText, Sparkles } from "@/components/ui/icons";
import type { LearningsState } from "../types";

interface LearningsAnalyticsCardProps {
  learningsData: LearningsState | null;
  onRefreshLearnings: () => void;
  onOpenMetricsModal: () => void;
}

export function LearningsAnalyticsCard({
  learningsData,
  onRefreshLearnings,
  onOpenMetricsModal,
}: LearningsAnalyticsCardProps) {
  const [showFullLearnings, setShowFullLearnings] = useState(false);
  const nextEp = learningsData?.recommendations?.next_episode_suggestions;

  return (
    <div className="rounded-card border border-border bg-card p-4 space-y-3 shadow-2xs">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <div className="flex h-6 w-6 items-center justify-center rounded-xs bg-amber-500/10 text-amber-500">
            <Sparkles className="h-3.5 w-3.5" />
          </div>
          <h4 className="font-bold text-foreground text-footnote leading-none">
            Số Liệu & Gợi Ý Tập Sau
          </h4>
        </div>
        <button
          type="button"
          onClick={onRefreshLearnings}
          className="text-caption text-primary hover:underline"
        >
          Cập nhật
        </button>
      </div>

      {/* Real KPI summary chips */}
      <div className="grid grid-cols-3 gap-2 text-center font-mono">
        <div className="p-2 rounded-ctl bg-muted/40 border border-border/40">
          <span className="text-caption text-muted-foreground block font-sans">Lượt xem</span>
          <span className="font-bold text-foreground text-footnote">
            {(learningsData?.kpis?.total_views ?? 0).toLocaleString()}
          </span>
        </div>

        <div className="p-2 rounded-ctl bg-muted/40 border border-border/40">
          <span className="text-caption text-muted-foreground block font-sans">Giữ chân</span>
          <span className="font-bold text-emerald-500 text-footnote">
            {learningsData?.kpis?.avg_retention_pct ?? 0}%
          </span>
        </div>

        <div className="p-2 rounded-ctl bg-muted/40 border border-border/40">
          <span className="text-caption text-muted-foreground block font-sans">
            Tương tác ER
          </span>
          <span className="font-bold text-blue-400 text-footnote">
            {learningsData?.kpis?.engagement_rate ?? 0}%
          </span>
        </div>
      </div>

      {/* AI Next Episode Recommendation Box */}
      {nextEp ? (
        <div className="p-3 rounded-ctl border border-amber-500/20 bg-amber-500/5 space-y-2 text-caption">
          <div className="font-semibold text-amber-400 flex items-center gap-1 text-caption">
            <Sparkles className="h-3.5 w-3.5 shrink-0" />
            <span>Gợi ý cho kịch bản tập sau:</span>
          </div>
          <p className="text-foreground leading-relaxed">{nextEp.hook_recommendation}</p>
          <div className="flex flex-wrap gap-2 text-caption text-muted-foreground font-mono pt-1 border-t border-border/40">
            <span>Nhịp dựng: ~{nextEp.recommended_shot_pacing_s}s/shot</span>
            <span>Thời lượng: {nextEp.target_duration_s}s</span>
            <span>Tỉ lệ: {nextEp.character_ratio}</span>
          </div>
        </div>
      ) : (
        <div className="p-3 rounded-ctl border border-dashed border-border bg-muted/20 text-center text-muted-foreground text-caption space-y-1">
          <p>Chưa có dữ liệu phân tích tập trước.</p>
          <button
            type="button"
            onClick={onOpenMetricsModal}
            className="text-primary hover:underline font-semibold"
          >
            + Nhập số liệu lượt xem / giữ chân để sinh gợi ý
          </button>
        </div>
      )}

      {/* Action Button */}
      <div className="flex items-center justify-between pt-1">
        <button
          type="button"
          onClick={onOpenMetricsModal}
          className="px-3 py-1 rounded-ctl bg-muted hover:bg-muted/80 text-foreground text-caption font-medium transition-colors border border-border flex items-center gap-1"
        >
          <FileText className="h-3 w-3" />
          <span>Nhập số liệu (Tay / CSV)</span>
        </button>

        {learningsData?.learnings_md && (
          <button
            type="button"
            onClick={() => setShowFullLearnings(!showFullLearnings)}
            className="text-caption text-muted-foreground hover:text-foreground underline"
          >
            {showFullLearnings ? "Thu gọn" : "Xem đúc kết chi tiết"}
          </button>
        )}
      </div>

      {showFullLearnings && learningsData?.learnings_md && (
        <div className="p-3 rounded-ctl border border-border bg-muted/30 text-caption font-mono text-muted-foreground whitespace-pre-wrap max-h-48 overflow-y-auto leading-relaxed select-text">
          {learningsData.learnings_md}
        </div>
      )}
    </div>
  );
}
