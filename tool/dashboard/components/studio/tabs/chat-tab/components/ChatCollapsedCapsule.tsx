"use client";

import React from "react";
import { ChevronUp, Sparkles } from "@/components/ui/icons";

interface Props {
  vi: boolean;
  input: string;
  attachmentsCount: number;
  chatFocus: "character" | "flow" | "project";
  chatModel: string;
  activeCharName?: string;
  messagesCount: number;
  showHistory: boolean;
  onExpand: () => void;
  onExpandHistory: () => void;
}

export function ChatCollapsedCapsule({
  vi,
  input,
  attachmentsCount,
  chatFocus,
  chatModel,
  activeCharName,
  messagesCount,
  showHistory,
  onExpand,
  onExpandHistory,
}: Props) {
  const placeholderText =
    attachmentsCount > 0
      ? `[${attachmentsCount} ${vi ? "ảnh đính kèm" : "attached"}] ${input || (vi ? "Nhập yêu cầu tạo Video..." : "Type prompt...")}`
      : input ||
        (chatFocus === "character"
          ? vi
            ? `Chat chỉnh sửa ${activeCharName || "nhân vật"} hoặc gõ /char...`
            : `Chat edit ${activeCharName || "character"}...`
          : chatFocus === "flow"
          ? vi
            ? "Hỏi đáp hoặc gõ /run, /flow, /video..."
            : "Ask or type /run, /flow, /video..."
          : vi
          ? "Trợ lý AI Studio: gõ /video, /char, /flow, /grid..."
          : "AI Studio: type /video, /char, /flow...");

  return (
    <div
      onClick={onExpand}
      data-chat-capsule=""
      className="group flex cursor-pointer items-center justify-between gap-2 rounded-full border border-input bg-card-elevated px-3 py-2 shadow-2xl ring-1 ring-foreground/10 transition-all hover:border-foreground/40 hover:bg-card"
    >
      <div className="flex min-w-0 flex-1 items-center gap-2">
        <span className="flex h-7 w-7 shrink-0 items-center justify-center rounded-full bg-brand/15 text-brand">
          <Sparkles className="h-3.5 w-3.5" />
        </span>
        <span className="truncate text-footnote text-muted-foreground group-hover:text-foreground/90 transition-colors">
          {placeholderText}
        </span>
      </div>

      <div className="flex shrink-0 items-center gap-1">
        <span className="inline-flex h-7 items-center gap-1 rounded-full px-2 text-caption font-medium text-muted-foreground hover:bg-card hover:text-foreground transition-colors">
          {chatFocus === "character"
            ? vi
              ? "Nhân vật"
              : "Character"
            : chatFocus === "flow"
            ? "Flow"
            : vi
            ? "Dự án"
            : "Project"}
        </span>

        <span className="inline-flex h-7 items-center gap-1 rounded-full px-2 text-caption font-medium text-muted-foreground hover:bg-card hover:text-foreground transition-colors">
          {chatModel.split("/").pop()}
        </span>

        {messagesCount > 0 && (
          <span
            onClick={(e) => {
              e.stopPropagation();
              onExpandHistory();
            }}
            className={`inline-flex h-7 items-center gap-1 rounded-full px-2 text-caption font-medium transition-colors ${
              showHistory
                ? "bg-brand/20 text-brand"
                : "text-muted-foreground hover:bg-card hover:text-foreground"
            }`}
          >
            <span className="h-1.5 w-1.5 rounded-full bg-brand" />
            <span>{messagesCount}</span>
          </span>
        )}

        <span className="inline-flex h-7 shrink-0 items-center gap-1 rounded-full px-2 text-muted-foreground hover:bg-accent hover:text-foreground transition-colors">
          <ChevronUp className="h-3.5 w-3.5" />
        </span>
      </div>
    </div>
  );
}
