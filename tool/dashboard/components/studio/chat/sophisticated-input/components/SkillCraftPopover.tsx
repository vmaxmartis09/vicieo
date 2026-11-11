"use client";

import React from "react";
import { cn } from "@/lib/utils";
import { CRAFT_PRESETS } from "../constants";
import type { SlashCommandItem } from "../types";

interface Props {
  isVi: boolean;
  craftTags: string[];
  slashCommands: SlashCommandItem[];
  onToggleCraftTag?: (tag: string) => void;
  onSlashSelect: (cmd: string) => void;
  onClose: () => void;
  onFocusTextarea: () => void;
}

export function SkillCraftPopover({
  isVi,
  craftTags,
  slashCommands,
  onToggleCraftTag,
  onSlashSelect,
  onClose,
  onFocusTextarea,
}: Props) {
  return (
    <div
      data-chat-popover=""
      className="scroll-quiet absolute bottom-full mb-2 left-0 right-0 sm:right-auto sm:left-9 sm:w-80 max-h-[min(380px,calc(100dvh-250px))] overflow-y-auto rounded-panel border border-input bg-popover p-2 shadow-2xl ring-1 ring-foreground/15 animate-in fade-in-0 zoom-in-95 z-50 space-y-2 text-footnote"
    >
      <div>
        <p className="px-2 py-1 text-caption font-semibold uppercase tracking-wider text-muted-foreground">
          {isVi ? "Kỹ thuật Đạo diễn & Camera (Director Craft)" : "Director & Camera Craft Presets"}
        </p>
        <div className="mt-1 flex flex-wrap gap-1">
          {CRAFT_PRESETS.map((cp) => {
            const active = craftTags.includes(cp.id);
            return (
              <button
                key={cp.id}
                type="button"
                onClick={() => onToggleCraftTag?.(cp.id)}
                className={cn(
                  "rounded-full border px-2 py-1 text-caption transition-colors",
                  active
                    ? "border-brand bg-brand/15 font-medium text-brand"
                    : "border-border bg-card text-muted-foreground hover:bg-accent hover:text-foreground"
                )}
              >
                {isVi ? cp.labelVi : cp.labelEn}
              </button>
            );
          })}
        </div>
      </div>

      <div className="border-t border-border/60 pt-2">
        <p className="px-2 py-1 text-caption font-semibold uppercase tracking-wider text-muted-foreground">
          {isVi ? "Kỹ năng & Lệnh tắt (/Skills)" : "Skills & Slash Workflows"}
        </p>
        <div className="mt-1 max-h-40 space-y-1 overflow-y-auto scroll-quiet">
          {slashCommands.map((s) => (
            <button
              key={s.cmd}
              type="button"
              onClick={() => {
                onSlashSelect(s.cmd);
                onClose();
                onFocusTextarea();
              }}
              className="flex w-full items-center gap-2 rounded-ctl px-2 py-2 text-left hover:bg-accent transition-colors"
            >
              <code className="rounded-xs bg-muted px-2 py-1 text-caption font-semibold text-foreground">{s.cmd}</code>
              <span className="truncate text-caption text-muted-foreground">{s.hint}</span>
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}
