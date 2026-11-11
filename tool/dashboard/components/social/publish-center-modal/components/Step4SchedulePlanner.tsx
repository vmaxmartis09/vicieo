"use client";

import React from "react";
import { cn } from "@/lib/utils";
import type { PlannedTarget } from "@/lib/api/social";
import { AlertTriangle, RefreshCw } from "../../icons";
import type { ScheduleStrategy } from "../types";

interface Step4SchedulePlannerProps {
  scheduleStrategy: ScheduleStrategy;
  onChangeStrategy: (strategy: ScheduleStrategy) => void;
  scheduleStartDate: string;
  onChangeStartDate: (value: string) => void;
  onReloadPreview: () => void;
  isLoadingPreview: boolean;
  previewTargets: PlannedTarget[];
  onUpdateTargetSchedule: React.Dispatch<React.SetStateAction<PlannedTarget[]>>;
}

export function Step4SchedulePlanner({
  scheduleStrategy,
  onChangeStrategy,
  scheduleStartDate,
  onChangeStartDate,
  onReloadPreview,
  isLoadingPreview,
  previewTargets,
  onUpdateTargetSchedule,
}: Step4SchedulePlannerProps) {
  return (
    <div className="space-y-4">
      {/* Strategy and filters bar */}
      <div className="flex flex-wrap items-center justify-between gap-3 rounded-xl border border-border bg-card p-3 shadow-2xs">
        <div className="flex items-center gap-3">
          <span className="text-xs font-bold text-foreground">
            Chiến lược lên lịch:
          </span>
          <div className="flex items-center rounded-lg border border-border bg-background p-1 text-xs">
            <button
              type="button"
              onClick={() => onChangeStrategy("auto")}
              className={cn(
                "rounded-md px-3 py-1 font-semibold transition-all",
                scheduleStrategy === "auto"
                  ? "bg-primary text-primary-foreground"
                  : "text-muted-foreground hover:text-foreground"
              )}
            >
              Tự xếp lịch chuẩn (Auto)
            </button>
            <button
              type="button"
              onClick={() => onChangeStrategy("now")}
              className={cn(
                "rounded-md px-3 py-1 font-semibold transition-all",
                scheduleStrategy === "now"
                  ? "bg-primary text-primary-foreground"
                  : "text-muted-foreground hover:text-foreground"
              )}
            >
              Đăng ngay
            </button>
            <button
              type="button"
              onClick={() => onChangeStrategy("fixed")}
              className={cn(
                "rounded-md px-3 py-1 font-semibold transition-all",
                scheduleStrategy === "fixed"
                  ? "bg-primary text-primary-foreground"
                  : "text-muted-foreground hover:text-foreground"
              )}
            >
              Giờ cố định
            </button>
          </div>
        </div>

        <div className="flex items-center gap-3 text-xs">
          <label className="flex items-center gap-2 text-muted-foreground">
            <span>Bắt đầu từ:</span>
            <input
              type="datetime-local"
              value={scheduleStartDate}
              onChange={(e) => onChangeStartDate(e.target.value)}
              className="rounded-lg border border-border bg-background px-2 py-1 text-xs text-foreground font-mono"
            />
          </label>

          <button
            type="button"
            onClick={onReloadPreview}
            className="flex items-center gap-1 rounded-lg border border-border bg-background px-3 py-1 text-xs font-semibold text-foreground hover:bg-accent transition-colors shadow-2xs"
          >
            <RefreshCw className="h-3 w-3" />
            <span>Xếp lại</span>
          </button>
        </div>
      </div>

      {/* Preview Table */}
      {isLoadingPreview ? (
        <div className="py-20 text-center text-xs text-muted-foreground">
          Đang phân tích khung giờ vàng và tính toán hạn ngạch quota...
        </div>
      ) : (
        <div className="space-y-3">
          <div className="flex items-center justify-between text-xs text-muted-foreground">
            <span>Tổng cộng: {previewTargets.length} mục tiêu xuất bản</span>
            <span className="font-mono text-caption">
              Múi giờ: Asia/Ho_Chi_Minh (+07:00)
            </span>
          </div>

          <div className="rounded-xl border border-border bg-card overflow-clip shadow-2xs">
            <table className="w-full text-xs text-left">
              <thead className="bg-muted/50 border-b border-border text-caption font-semibold text-muted-foreground">
                <tr>
                  <th className="p-3">Video</th>
                  <th className="p-3">Kênh xuất bản</th>
                  <th className="p-3">Thời gian lên lịch</th>
                  <th className="p-3">Biến thể video</th>
                  <th className="p-3 text-right">Trạng thái / Cảnh báo</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-border/60">
                {previewTargets.map((pt, idx) => (
                  <tr
                    key={`${pt.video_id}_${pt.account_id}_${idx}`}
                    className="hover:bg-accent/20"
                  >
                    <td className="p-3 font-semibold text-foreground max-w-[200px] truncate">
                      {pt.video_title}
                    </td>
                    <td className="p-3">
                      <span className="inline-flex items-center gap-2 font-medium text-foreground">
                        <span className="h-2 w-2 rounded-full bg-primary" />
                        {pt.account_name} ({pt.platform})
                      </span>
                    </td>
                    <td className="p-3 font-mono font-medium text-foreground">
                      <input
                        type="datetime-local"
                        value={pt.scheduled_at.slice(0, 16)}
                        onChange={(e) => {
                          const newTime = e.target.value;
                          onUpdateTargetSchedule((prev) =>
                            prev.map((item, i) =>
                              i === idx
                                ? { ...item, scheduled_at: `${newTime}:00+07:00` }
                                : item
                            )
                          );
                        }}
                        className="rounded-md border border-border bg-background px-2 py-1 text-xs text-foreground font-mono"
                      />
                    </td>
                    <td className="p-3 text-muted-foreground font-mono">
                      {pt.variant_used || "Mặc định"}
                    </td>
                    <td className="p-3 text-right">
                      {pt.issues && pt.issues.length > 0 ? (
                        <span
                          title={pt.issues.map((i) => i.message_vi).join("\n")}
                          className="inline-flex items-center gap-1 text-semantic-warning font-medium"
                        >
                          <AlertTriangle className="h-3 w-3" />
                          <span>{pt.issues[0].code}</span>
                        </span>
                      ) : (
                        <span className="text-semantic-success font-medium">
                          ✓ Sẵn sàng
                        </span>
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}
    </div>
  );
}
