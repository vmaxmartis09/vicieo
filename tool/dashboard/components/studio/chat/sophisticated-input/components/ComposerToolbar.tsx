"use client";

import React from "react";
import { ArrowRight, ChevronUp, Mic, Plus, ShieldCheck, Wand2 } from "@/components/ui/icons";
import { cn } from "@/lib/utils";
import type { ComposerActionMode, ComposerAspectRatio } from "../types";

interface Props {
  isVi: boolean;
  sendLabel: string;
  plusMenuOpen: boolean;
  skillMenuOpen: boolean;
  modelMenuOpen: boolean;
  modeMenuOpen: boolean;
  craftTagsCount: number;
  currentModelName: string;
  aspectRatio: ComposerAspectRatio;
  isMockMode: boolean;
  isListening: boolean;
  disabled: boolean;
  canSubmit: boolean;
  actionMode: ComposerActionMode;
  onTogglePlusMenu: () => void;
  onToggleSkillMenu: () => void;
  onToggleModelMenu: () => void;
  onToggleModeMenu: () => void;
  onToggleSpeech: () => void;
  onSubmitClick: () => void;
}

export function ComposerToolbar({
  isVi,
  sendLabel,
  plusMenuOpen,
  skillMenuOpen,
  modelMenuOpen,
  modeMenuOpen,
  craftTagsCount,
  currentModelName,
  aspectRatio,
  isMockMode,
  isListening,
  disabled,
  canSubmit,
  actionMode,
  onTogglePlusMenu,
  onToggleSkillMenu,
  onToggleModelMenu,
  onToggleModeMenu,
  onToggleSpeech,
  onSubmitClick,
}: Props) {
  return (
    <div className="flex flex-wrap sm:flex-nowrap items-center justify-between gap-2 px-3 pb-2 pt-1 select-none">
      {/* BÊN TRÁI: Nút '+' (Upload/Canvas Image) + Nút 'Skill' + Pill Model · Aspect Ratio */}
      <div className="flex min-w-0 items-center gap-1">
        <button
          type="button"
          title={isVi ? "Đính kèm ảnh từ máy / Chọn ảnh trên Canvas / Chế độ tạo" : "Upload image / Select from Canvas / Mode"}
          aria-label={isVi ? "Đính kèm ảnh hoặc chọn chế độ" : "Upload or select mode"}
          onClick={onTogglePlusMenu}
          className={cn(
            "flex h-7 w-7 shrink-0 items-center justify-center rounded-ctl border border-input bg-card-elevated text-foreground/80 hover:bg-accent hover:text-foreground active:scale-95 transition-colors",
            plusMenuOpen && "border-brand bg-accent text-foreground"
          )}
        >
          <Plus className="h-3.5 w-3.5" />
        </button>

        <button
          type="button"
          onClick={onToggleSkillMenu}
          className={cn(
            "inline-flex h-7 shrink-0 items-center gap-1 rounded-ctl border border-input bg-card-elevated px-2 text-caption font-medium text-foreground/80 hover:bg-accent hover:text-foreground active:scale-95 transition-colors",
            (skillMenuOpen || craftTagsCount > 0) && "border-brand bg-brand/15 text-foreground"
          )}
        >
          <Wand2 className="h-3 w-3 text-brand" />
          <span>Skill</span>
          {craftTagsCount > 0 && (
            <span className="rounded-full bg-brand/20 px-1 text-caption font-semibold text-brand tnum">
              {craftTagsCount}
            </span>
          )}
        </button>

        <button
          type="button"
          onClick={onToggleModelMenu}
          className={cn(
            "inline-flex h-7 min-w-0 items-center gap-2 rounded-ctl border border-input bg-card-elevated px-2 text-caption font-medium text-foreground hover:bg-accent active:scale-95 transition-all cursor-pointer",
            modelMenuOpen && "border-brand bg-accent text-foreground"
          )}
        >
          <span className="truncate max-w-[120px] sm:max-w-none">{currentModelName}</span>
          <span className="text-muted-foreground">·</span>
          <span className="text-caption font-semibold text-brand tnum">{aspectRatio}</span>
          <ChevronUp className={cn("h-3 w-3 shrink-0 text-muted-foreground transition-transform duration-200", modelMenuOpen && "rotate-180")} />
        </button>
      </div>

      {/* BÊN PHẢI: Mode Pill + Mic + Nút Send / Generate */}
      <div className="flex shrink-0 items-center gap-1">
        <button
          type="button"
          onClick={onToggleModeMenu}
          className={cn(
            "inline-flex h-7 items-center gap-1 rounded-ctl border border-input bg-card-elevated px-2 text-caption font-medium text-foreground/85 hover:bg-accent hover:text-foreground active:scale-95 transition-all cursor-pointer",
            modeMenuOpen && "border-brand bg-accent text-foreground"
          )}
        >
          <ShieldCheck className="h-3 w-3 text-brand" />
          <span className="hidden sm:inline">{isMockMode ? (isVi ? "Hỏi trước" : "Ask First") : "Turbo"}</span>
          <ChevronUp className={cn("h-3 w-3 text-muted-foreground transition-transform duration-200", modeMenuOpen && "rotate-180")} />
        </button>

        <button
          type="button"
          title={isListening ? (isVi ? "Đang nghe... bấm để dừng" : "Listening... click to stop") : (isVi ? "Nói bằng giọng nói" : "Voice dictation")}
          aria-label="Voice input"
          onClick={onToggleSpeech}
          className={cn(
            "flex h-7 w-7 items-center justify-center rounded-full transition-all",
            isListening
              ? "bg-semantic-danger text-white animate-pulse"
              : "text-muted-foreground hover:bg-accent hover:text-foreground active:scale-90"
          )}
        >
          <Mic className="h-3.5 w-3.5" />
        </button>

        <button
          type="button"
          disabled={disabled || !canSubmit}
          onClick={onSubmitClick}
          aria-label={sendLabel}
          className={cn(
            "inline-flex h-7 items-center justify-center gap-1 rounded-ctl px-3 text-caption font-semibold transition-all duration-150",
            canSubmit && !disabled
              ? "bg-foreground text-background shadow-xs hover:opacity-90 active:scale-95 cursor-pointer"
              : "bg-muted/70 text-muted-foreground/40 cursor-not-allowed"
          )}
        >
          {actionMode !== "chat" && (
            <span className="hidden sm:inline">
              {actionMode === "video"
                ? isVi
                  ? "Tạo Video"
                  : "Generate Video"
                : actionMode === "image_list"
                ? isVi
                  ? "Tạo List Ảnh"
                  : "Generate Grid"
                : isVi
                ? "Tạo Ảnh"
                : "Generate"}
            </span>
          )}
          <ArrowRight className="h-3.5 w-3.5" />
        </button>
      </div>
    </div>
  );
}
