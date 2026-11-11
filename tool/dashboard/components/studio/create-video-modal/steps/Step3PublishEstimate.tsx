"use client";

import React from "react";
import { Check, Film, Loader2, Lock, ShieldCheck } from "@/components/ui/icons";
import { SocialBrandIcon } from "@/components/ui/SocialIcons";
import type { Platform, TemplateMeta } from "@/lib/api/video";
import type { SocialPlatformId } from "@/lib/socialConfig";
import { cn } from "@/lib/utils";
import { resolveSocialKey, type LintReport } from "../types";

export const Step3PublishEstimate = React.memo(function Step3PublishEstimate({
  selectedTemplate,
  platforms,
  setPlatforms,
  connectedMap,
  onOpenConnectModal,
  qualityLevel,
  setQualityLevel,
  isCheckingLint,
  onRunLintCheck,
  lintReport,
  isMockMode,
  isEstimating,
  estimatedCost,
}: {
  selectedTemplate: TemplateMeta;
  platforms: Platform[];
  setPlatforms: (p: Platform[]) => void;
  connectedMap: Record<string, boolean>;
  onOpenConnectModal: (socialKey: SocialPlatformId) => void;
  qualityLevel: "standard" | "high";
  setQualityLevel: (q: "standard" | "high") => void;
  isCheckingLint: boolean;
  onRunLintCheck: () => void;
  lintReport: LintReport | null;
  isMockMode: boolean;
  isEstimating: boolean;
  estimatedCost: number;
}) {
  return (
    <div className="space-y-4">
      <div className="space-y-2">
        <div className="flex items-center justify-between">
          <label className="text-footnote font-semibold text-foreground">
            Nền tảng xuất bản (tự áp chuẩn Safe-Zone & Tỷ lệ):
          </label>
          <span className="text-caption text-muted-foreground">
            * Yêu cầu kết nối tài khoản trước khi chia sẻ
          </span>
        </div>
        <div className="grid grid-cols-2 sm:grid-cols-4 gap-2">
          {(["tiktok", "shorts", "reels", "youtube"] as Platform[]).map((plat) => {
            const checked = platforms.includes(plat);
            const isConnected = Boolean(connectedMap[plat]);
            const socialKey = resolveSocialKey(plat);

            return (
              <button
                key={plat}
                type="button"
                onClick={() => {
                  if (!isConnected) {
                    onOpenConnectModal(socialKey);
                    return;
                  }
                  if (checked && platforms.length > 1) {
                    setPlatforms(platforms.filter((p) => p !== plat));
                  } else if (!checked) {
                    setPlatforms([...platforms, plat]);
                  }
                }}
                className={cn(
                  "flex items-center justify-between rounded-card border p-3 text-footnote font-medium transition-all text-left",
                  isConnected && checked
                    ? "border-primary bg-primary/10 text-primary shadow-sm"
                    : isConnected && !checked
                    ? "border-border bg-background text-muted-foreground hover:bg-accent"
                    : "border-border/40 bg-muted/20 text-muted-foreground/60 hover:border-amber-500/50 hover:bg-amber-500/5"
                )}
                title={
                  isConnected
                    ? `${plat.toUpperCase()}: Đã kết nối (Nhấn để bật/tắt)`
                    : `${plat.toUpperCase()}: Chưa kết nối tài khoản! Bấm để kết nối trước khi xuất bản`
                }
              >
                <div className="flex items-center gap-2">
                  <SocialBrandIcon
                    platform={socialKey}
                    size={18}
                    variant={isConnected ? "brand" : "monochrome"}
                    className={!isConnected ? "opacity-50" : ""}
                  />
                </div>

                {isConnected ? (
                  <div
                    className={cn(
                      "flex h-4 w-4 shrink-0 items-center justify-center rounded-xs border",
                      checked ? "border-primary bg-primary text-white" : "border-border"
                    )}
                  >
                    {checked && <Check className="h-3 w-3" />}
                  </div>
                ) : (
                  <span className="text-amber-500/80" title="Cần kết nối">
                    <Lock className="h-3.5 w-3.5" />
                  </span>
                )}
              </button>
            );
          })}
        </div>
      </div>

      <div className="space-y-2">
        <label className="text-footnote font-semibold text-foreground">Chất lượng video:</label>
        <div className="grid grid-cols-2 gap-3">
          <button
            type="button"
            onClick={() => setQualityLevel("standard")}
            className={cn(
              "flex flex-col gap-1 rounded-card border p-3 text-left transition-all",
              qualityLevel === "standard"
                ? "border-primary bg-primary/10 text-primary"
                : "border-border bg-background text-muted-foreground hover:bg-accent"
            )}
          >
            <span className="text-footnote font-semibold">Chuẩn (1080p Standard)</span>
            <span className="text-caption text-muted-foreground">
              Tối ưu tốc độ render, chuẩn TikTok & Reels
            </span>
          </button>

          <button
            type="button"
            onClick={() => setQualityLevel("high")}
            className={cn(
              "flex flex-col gap-1 rounded-card border p-3 text-left transition-all",
              qualityLevel === "high"
                ? "border-primary bg-primary/10 text-primary"
                : "border-border bg-background text-muted-foreground hover:bg-accent"
            )}
          >
            <span className="text-footnote font-semibold">Cao (1080p High Bitrate)</span>
            <span className="text-caption text-muted-foreground">
              Bitrate cao, chuẩn phát YouTube 4K/FHD
            </span>
          </button>
        </div>
      </div>

      <div className="rounded-card border border-border bg-card/60 p-3 space-y-3">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <ShieldCheck className="h-4 w-4 text-primary" />
            <span className="text-footnote font-semibold text-foreground">Kiểm tra trước khi xuất (Creative QA / Lint)</span>
          </div>
          <button
            type="button"
            onClick={onRunLintCheck}
            disabled={isCheckingLint}
            className="flex items-center gap-2 rounded-ctl border border-border bg-background px-3 py-1 text-footnote font-medium text-foreground hover:bg-accent transition-colors"
          >
            {isCheckingLint ? <Loader2 className="h-3 w-3 animate-spin" /> : <ShieldCheck className="h-3 w-3 text-primary" />}
            <span>Kiểm tra ngay</span>
          </button>
        </div>

        {lintReport && (
          <div
            className={cn(
              "rounded-ctl p-3 text-footnote space-y-2 border",
              lintReport.passed
                ? "bg-emerald-500/10 border-emerald-500/30 text-emerald-700 dark:text-emerald-300"
                : "bg-amber-500/10 border-amber-500/30 text-amber-800 dark:text-amber-200"
            )}
          >
            <div className="flex items-center justify-between font-semibold">
              <span>{lintReport.passed ? "✓ Đạt toàn bộ tiêu chuẩn Creative QA" : `⚠️ Có ${lintReport.error_count} lỗi / ${lintReport.warning_count} cảnh báo`}</span>
              <span className="text-caption uppercase font-mono">{lintReport.total_checks} tiêu chí</span>
            </div>
            {lintReport.issues && lintReport.issues.length > 0 && (
              <ul className="space-y-1 text-caption list-disc list-inside pt-1">
                {lintReport.issues.map((iss, idx) => (
                  <li key={iss.code ? `${iss.code}-${idx}` : `${iss.level}-${idx}`}>
                    <strong>[{iss.level.toUpperCase()}]:</strong> {iss.message_vi}
                  </li>
                ))}
              </ul>
            )}
          </div>
        )}
      </div>

      <div className="rounded-card border border-border bg-accent/30 p-4 space-y-2 text-footnote">
        <span className="font-semibold text-foreground flex items-center gap-2">
          <Film className="h-4 w-4 text-primary" />
          Thông số kỹ thuật tự động hoá (từ preset chuẩn):
        </span>
        <div className="grid grid-cols-2 sm:grid-cols-4 gap-2 text-caption pt-1">
          <div className="space-y-1">
            <span className="text-muted-foreground">Kích thước:</span>
            <p className="font-semibold text-foreground">
              {selectedTemplate.default_platform === "youtube" ? "1920×1080 (16:9)" : "1080×1920 (9:16)"}
            </p>
          </div>
          <div className="space-y-1">
            <span className="text-muted-foreground">Âm thanh:</span>
            <p className="font-semibold text-foreground">-14 LUFS (TikTok SOTA)</p>
          </div>
          <div className="space-y-1">
            <span className="text-muted-foreground">Safe Zone:</span>
            <p className="font-semibold text-foreground">Top 180px · Bottom 320px</p>
          </div>
          <div className="space-y-1">
            <span className="text-muted-foreground">Codec & FPS:</span>
            <p className="font-semibold text-foreground">H.264 / AAC @ 30fps</p>
          </div>
        </div>
      </div>

      <div className="flex items-center justify-between rounded-card border border-border bg-card p-3">
        <div className="space-y-1">
          <span className="text-footnote font-semibold text-foreground">
            Ước tính chi phí sản xuất:
          </span>
          <p className="text-caption text-muted-foreground">
            {isMockMode
              ? "Chế độ Chạy thử (Mock mode: 0 USD)"
              : "Dựa trên mô hình AI đã chọn"}
          </p>
        </div>
        <div className="text-right">
          {isEstimating ? (
            <Loader2 className="h-4 w-4 animate-spin text-muted-foreground" />
          ) : (
            <span className="text-base font-bold text-primary font-mono">
              ${estimatedCost.toFixed(2)} USD
            </span>
          )}
        </div>
      </div>
    </div>
  );
});
