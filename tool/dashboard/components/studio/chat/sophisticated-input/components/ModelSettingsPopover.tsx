"use client";

import React from "react";
import { Bot, Check } from "@/components/ui/icons";
import { cn } from "@/lib/utils";
import type { ChatModel } from "@/store/slices/chatSlice";
import { ASPECT_RATIO_OPTIONS, MODEL_OPTIONS, OUTPUT_MODE_OPTIONS } from "../constants";
import type { ComposerActionMode, ComposerAspectRatio } from "../types";

interface Props {
  isVi: boolean;
  model: ChatModel;
  modelTab: "agent" | "media";
  aspectRatio: ComposerAspectRatio;
  actionMode: ComposerActionMode;
  onModelTabChange: (tab: "agent" | "media") => void;
  onModelChange: (m: ChatModel) => void;
  onAspectRatioChange?: (r: ComposerAspectRatio) => void;
  onActionModeChange?: (mode: ComposerActionMode) => void;
  onClose: () => void;
}

export function ModelSettingsPopover({
  isVi,
  model,
  modelTab,
  aspectRatio,
  actionMode,
  onModelTabChange,
  onModelChange,
  onAspectRatioChange,
  onActionModeChange,
  onClose,
}: Props) {
  return (
    <div
      data-chat-popover=""
      className="scroll-quiet absolute bottom-full mb-2 left-0 right-0 sm:right-auto sm:left-16 sm:w-80 max-h-[min(420px,calc(100dvh-250px))] overflow-y-auto rounded-panel border border-input bg-popover p-2 shadow-2xl ring-1 ring-foreground/15 animate-in fade-in-0 zoom-in-95 z-50 space-y-2 text-footnote"
    >
      <div className="flex rounded-full bg-accent p-1 border border-input/70">
        <button
          type="button"
          onClick={() => onModelTabChange("agent")}
          className={cn(
            "flex-1 rounded-full py-1 text-caption font-medium transition-colors",
            modelTab === "agent" ? "bg-card text-foreground shadow-xs font-semibold" : "text-muted-foreground hover:text-foreground"
          )}
        >
          {isVi ? "Mô hình Agent" : "Agent Model"}
        </button>
        <button
          type="button"
          onClick={() => onModelTabChange("media")}
          className={cn(
            "flex-1 rounded-full py-1 text-caption font-medium transition-colors",
            modelTab === "media" ? "bg-card text-foreground shadow-xs font-semibold" : "text-muted-foreground hover:text-foreground"
          )}
        >
          {isVi ? "Ảnh / Video / Tỉ lệ" : "Image / Video / Ratio"}
        </button>
      </div>

      {modelTab === "agent" ? (
        <div className="space-y-1">
          {MODEL_OPTIONS.map((opt) => (
            <button
              key={opt.id}
              type="button"
              onClick={() => {
                onModelChange(opt.id);
                onClose();
              }}
              className={cn(
                "flex w-full items-start gap-2 rounded-ctl px-3 py-2 text-left transition-colors",
                model === opt.id ? "bg-accent text-foreground" : "hover:bg-card-elevated text-foreground/85 hover:text-foreground"
              )}
            >
              <Bot className="mt-1 h-4 w-4 text-brand shrink-0" />
              <div className="min-w-0 flex-1">
                <div className="flex items-center gap-2">
                  <span className="text-footnote font-semibold text-foreground">{opt.name}</span>
                  <span className="rounded-full bg-brand/10 px-2 py-1 text-caption font-medium text-brand tnum">
                    {opt.tag}
                  </span>
                  {model === opt.id && <Check className="ml-auto h-3.5 w-3.5 text-brand" />}
                </div>
                <p className="mt-1 text-caption leading-relaxed text-muted-foreground line-clamp-2">{opt.desc}</p>
              </div>
            </button>
          ))}
        </div>
      ) : (
        <div className="space-y-3 p-1">
          <div>
            <p className="mb-1 text-caption font-medium text-muted-foreground">
              {isVi ? "Tỉ lệ khung hình (Aspect Ratio)" : "Aspect Ratio"}
            </p>
            <div className="grid grid-cols-4 gap-2">
              {ASPECT_RATIO_OPTIONS.map((r) => (
                <button
                  key={r}
                  type="button"
                  onClick={() => onAspectRatioChange?.(r)}
                  className={cn(
                    "rounded-ctl border py-2 text-center text-footnote font-medium tnum transition-colors",
                    aspectRatio === r
                      ? "border-brand bg-brand/15 text-brand font-semibold"
                      : "border-border bg-card text-muted-foreground hover:bg-accent hover:text-foreground"
                  )}
                >
                  {r}
                </button>
              ))}
            </div>
          </div>

          <div>
            <p className="mb-1 text-caption font-medium text-muted-foreground">
              {isVi ? "Chế độ đầu ra khi gửi lệnh" : "Output Mode"}
            </p>
            <div className="grid grid-cols-2 gap-2">
              {OUTPUT_MODE_OPTIONS.map((opt) => (
                <button
                  key={opt.mode}
                  type="button"
                  onClick={() => onActionModeChange?.(opt.mode)}
                  className={cn(
                    "rounded-ctl border px-2 py-2 text-left text-caption font-medium transition-colors",
                    actionMode === opt.mode
                      ? "border-brand bg-brand/15 text-brand font-semibold"
                      : "border-border bg-card text-muted-foreground hover:bg-accent hover:text-foreground"
                  )}
                >
                  {isVi ? opt.labelVi : opt.labelEn}
                </button>
              ))}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
