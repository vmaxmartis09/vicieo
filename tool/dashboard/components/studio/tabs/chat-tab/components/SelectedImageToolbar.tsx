"use client";

import React from "react";
import { Sparkles } from "@/components/ui/icons";
import type { CanvasImageItem, ChatAttachment, ComposerActionMode } from "../../../chat/SophisticatedChatInput";

interface Props {
  selectedCanvasImage: CanvasImageItem;
  vi: boolean;
  onAddAttachment: (att: ChatAttachment) => void;
  onSelectModeAndExpand: (mode?: ComposerActionMode) => void;
}

export function SelectedImageToolbar({
  selectedCanvasImage,
  vi,
  onAddAttachment,
  onSelectModeAndExpand,
}: Props) {
  const buildAtt = (): ChatAttachment => ({
    id: `canvas_${selectedCanvasImage.nodeId}`,
    name: selectedCanvasImage.name,
    url: selectedCanvasImage.url,
    source: "canvas",
    nodeId: selectedCanvasImage.nodeId,
  });

  return (
    <div className="mx-auto flex flex-wrap items-center gap-2 rounded-full border border-input bg-card-elevated px-3 py-1 shadow-2xl ring-1 ring-foreground/10 animate-in fade-in-0 slide-in-from-bottom-1">
      <img
        src={selectedCanvasImage.url}
        alt={selectedCanvasImage.name}
        loading="lazy"
        className="h-5 w-5 rounded-ctl object-cover border border-border"
      />
      <span className="max-w-[120px] truncate text-caption font-medium text-foreground">
        {selectedCanvasImage.name}
      </span>
      <button
        type="button"
        onClick={() => {
          onAddAttachment(buildAtt());
          onSelectModeAndExpand("video");
        }}
        className="inline-flex h-7 items-center gap-1 rounded-full bg-brand/15 px-2 text-caption font-semibold text-brand hover:bg-brand/25 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring transition-colors"
      >
        <Sparkles className="h-3 w-3" />
        <span>{vi ? "Tạo Video" : "Gen Video"}</span>
      </button>
      <button
        type="button"
        onClick={() => {
          onAddAttachment(buildAtt());
          onSelectModeAndExpand("image_list");
        }}
        className="inline-flex h-7 items-center gap-1 rounded-full border border-border/70 bg-muted/50 px-2 text-caption font-medium text-foreground hover:bg-accent focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring transition-colors"
      >
        <span>{vi ? "Storyboard 4 Góc" : "4-Angle Storyboard"}</span>
      </button>
      <button
        type="button"
        onClick={() => {
          onAddAttachment(buildAtt());
          onSelectModeAndExpand();
        }}
        className="inline-flex h-7 items-center gap-1 rounded-full border border-border/70 bg-muted/50 px-2 text-caption font-medium text-muted-foreground hover:text-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring transition-colors"
      >
        <span>{vi ? "Đính kèm" : "Attach"}</span>
      </button>
    </div>
  );
}
