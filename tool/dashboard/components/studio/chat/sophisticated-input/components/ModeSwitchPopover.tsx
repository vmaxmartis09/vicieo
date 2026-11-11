"use client";

import React from "react";
import { Bot, Check, ShieldCheck } from "@/components/ui/icons";
import { cn } from "@/lib/utils";

interface Props {
  isVi: boolean;
  isMockMode: boolean;
  onToggleMockMode?: () => void;
  onClose: () => void;
}

export function ModeSwitchPopover({ isVi, isMockMode, onToggleMockMode, onClose }: Props) {
  return (
    <div
      data-chat-popover=""
      className="scroll-quiet absolute bottom-full mb-2 right-0 sm:right-12 w-64 max-w-full max-h-[min(320px,calc(100dvh-250px))] overflow-y-auto rounded-panel border border-input bg-popover p-2 shadow-2xl ring-1 ring-foreground/15 animate-in fade-in-0 zoom-in-95 z-50 space-y-1 text-footnote"
    >
      <p className="px-3 py-1 text-caption font-semibold text-muted-foreground uppercase tracking-wider">
        {isVi ? "Chế độ thực thi" : "Generate Mode"}
      </p>
      <button
        type="button"
        onClick={() => {
          if (isMockMode && onToggleMockMode) onToggleMockMode();
          onClose();
        }}
        className={cn(
          "flex w-full items-start gap-2 rounded-ctl px-3 py-2 text-left transition-colors",
          !isMockMode ? "bg-accent text-foreground" : "hover:bg-card-elevated text-foreground/85 hover:text-foreground"
        )}
      >
        <ShieldCheck className="mt-1 h-4 w-4 text-brand shrink-0" />
        <div>
          <div className="flex items-center gap-2">
            <span className="font-semibold text-foreground">Turbo · Live</span>
            <span className="rounded-full bg-brand/10 px-2 py-1 text-caption font-medium text-brand">
              SOTA
            </span>
            {!isMockMode && <Check className="ml-auto h-3.5 w-3.5 text-brand" />}
          </div>
          <p className="text-caption text-muted-foreground">
            {isVi ? "Thực thi trực tiếp qua Gemini & HyperFrames" : "Live execution via Gemini & HyperFrames"}
          </p>
        </div>
      </button>
      <button
        type="button"
        onClick={() => {
          if (!isMockMode && onToggleMockMode) onToggleMockMode();
          onClose();
        }}
        className={cn(
          "flex w-full items-start gap-2 rounded-ctl px-3 py-2 text-left transition-colors",
          isMockMode ? "bg-accent text-foreground" : "hover:bg-card-elevated text-foreground/85 hover:text-foreground"
        )}
      >
        <Bot className="mt-1 h-4 w-4 text-muted-foreground shrink-0" />
        <div>
          <div className="flex items-center gap-2">
            <span className="font-semibold text-foreground">
              {isVi ? "Hỏi trước / Chạy thử ($0)" : "Ask First / Mock ($0)"}
            </span>
            <span className="rounded-full bg-muted px-2 py-1 text-caption font-medium text-muted-foreground">
              Free
            </span>
            {isMockMode && <Check className="ml-auto h-3.5 w-3.5 text-brand" />}
          </div>
          <p className="text-caption text-muted-foreground">
            {isVi ? "Xác nhận trước khi render & mô phỏng $0" : "Confirm before generating & $0 simulation"}
          </p>
        </div>
      </button>
    </div>
  );
}
