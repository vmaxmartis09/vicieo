"use client";

import React from "react";
import { Check, CheckCircle2, ShieldCheck } from "../../icons";
import type { CommitResult, ScheduleStrategy } from "../types";

interface Step5ReviewCommitProps {
  commitResult: CommitResult | null;
  selectedVideosCount: number;
  selectedAccountsCount: number;
  previewTargetsCount: number;
  scheduleStrategy: ScheduleStrategy;
  approvalRequired: boolean;
  onToggleApprovalRequired: (value: boolean) => void;
  isSubmittingPlan: boolean;
  onCommitPlan: () => void;
  onOpenCalendarTab: () => void;
  onClose: () => void;
}

export function Step5ReviewCommit({
  commitResult,
  selectedVideosCount,
  selectedAccountsCount,
  previewTargetsCount,
  scheduleStrategy,
  approvalRequired,
  onToggleApprovalRequired,
  isSubmittingPlan,
  onCommitPlan,
  onOpenCalendarTab,
  onClose,
}: Step5ReviewCommitProps) {
  return (
    <div className="space-y-5 max-w-2xl mx-auto py-4">
      {commitResult ? (
        /* Success Screen */
        <div className="rounded-xl border border-emerald-500/30 bg-emerald-500/5 p-5 text-center space-y-3">
          <div className="flex h-14 w-14 items-center justify-center rounded-full bg-emerald-500/20 text-semantic-success mx-auto">
            <CheckCircle2 className="h-8 w-8" />
          </div>
          <h3 className="text-lg font-bold text-foreground">
            Lên kế hoạch xuất bản thành công!
          </h3>
          <p className="text-xs text-muted-foreground leading-relaxed">
            Đã tạo thành công <strong>{commitResult.postsCount} bài viết</strong>{" "}
            với <strong>{commitResult.targetsCount} mục tiêu xuất bản</strong>.
            Các bài viết đã được đưa vào hàng đợi và sẽ xuất bản đúng khung giờ
            đã lên lịch.
          </p>
          <div className="flex items-center justify-center gap-3 pt-2">
            <button
              type="button"
              onClick={onOpenCalendarTab}
              className="rounded-xl bg-primary px-4 py-2 text-xs font-semibold text-primary-foreground hover:bg-primary/90 transition-colors shadow"
            >
              Xem trên Lịch Đăng
            </button>
            <button
              type="button"
              onClick={onClose}
              className="rounded-xl border border-border bg-background px-4 py-2 text-xs font-semibold text-foreground hover:bg-accent transition-colors shadow-2xs"
            >
              Đóng cửa sổ
            </button>
          </div>
        </div>
      ) : (
        /* Review and Commit Table */
        <div className="space-y-5">
          <div className="rounded-xl border border-border bg-card p-4 space-y-3 shadow-2xs">
            <h4 className="text-xs font-bold text-foreground uppercase tracking-wider">
              Tóm tắt kế hoạch xuất bản:
            </h4>
            <div className="grid grid-cols-2 sm:grid-cols-4 gap-3 text-center">
              <div className="rounded-lg bg-muted p-3">
                <p className="text-caption text-muted-foreground">
                  Video đã chọn
                </p>
                <p className="text-base font-bold text-foreground">
                  {selectedVideosCount}
                </p>
              </div>
              <div className="rounded-lg bg-muted p-3">
                <p className="text-caption text-muted-foreground">Tài khoản</p>
                <p className="text-base font-bold text-foreground">
                  {selectedAccountsCount}
                </p>
              </div>
              <div className="rounded-lg bg-muted p-3">
                <p className="text-caption text-muted-foreground">
                  Tổng lượt đăng
                </p>
                <p className="text-base font-bold text-primary">
                  {previewTargetsCount}
                </p>
              </div>
              <div className="rounded-lg bg-muted p-3">
                <p className="text-caption text-muted-foreground">Khung giờ</p>
                <p className="text-base font-bold text-foreground">
                  {scheduleStrategy === "now" ? "Đăng ngay" : "Tự động"}
                </p>
              </div>
            </div>
          </div>

          {/* Safe lock toggle */}
          <div className="flex items-center justify-between rounded-xl border border-amber-500/30 bg-amber-500/5 p-4">
            <div className="space-y-1">
              <p className="text-xs font-bold text-semantic-warning flex items-center gap-2">
                <ShieldCheck className="h-4 w-4" />
                Duyệt trước khi đăng (Khóa an toàn)
              </p>
              <p className="text-caption text-muted-foreground">
                Mọi bài viết sẽ dừng ở hàng chờ duyệt để bạn kiểm tra trước khi
                tự động xuất bản
              </p>
            </div>
            <input
              type="checkbox"
              checked={approvalRequired}
              onChange={(e) => onToggleApprovalRequired(e.target.checked)}
              className="h-4 w-4 rounded border-border text-primary focus:ring-primary"
            />
          </div>

          {/* Action Commit Button */}
          <button
            type="button"
            disabled={isSubmittingPlan || previewTargetsCount === 0}
            onClick={onCommitPlan}
            className="w-full flex items-center justify-center gap-2 rounded-xl bg-primary py-3 text-xs font-bold text-primary-foreground hover:bg-primary/90 transition-colors shadow-lg disabled:opacity-50"
          >
            <Check className="h-4 w-4" />
            <span>
              {isSubmittingPlan
                ? "Đang lên lịch..."
                : "Xác nhận lên lịch tất cả bài viết"}
            </span>
          </button>
        </div>
      )}
    </div>
  );
}
