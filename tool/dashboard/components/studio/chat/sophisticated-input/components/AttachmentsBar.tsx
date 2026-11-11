"use client";

import React from "react";
import { Film, Image, Images, Wand2, X } from "@/components/ui/icons";
import type { ChatAttachment, ComposerActionMode } from "../types";

interface Props {
  isVi: boolean;
  actionMode: ComposerActionMode;
  attachments: ChatAttachment[];
  craftTags: string[];
  onActionModeChange?: (mode: ComposerActionMode) => void;
  onRemoveAttachment?: (id: string) => void;
  onToggleCraftTag?: (tag: string) => void;
}

export function AttachmentsBar({
  isVi,
  actionMode,
  attachments,
  craftTags,
  onActionModeChange,
  onRemoveAttachment,
  onToggleCraftTag,
}: Props) {
  if (attachments.length === 0 && craftTags.length === 0 && actionMode === "chat") {
    return null;
  }

  return (
    <div className="flex flex-wrap items-center gap-2 px-4 pt-3 pb-1">
      {actionMode !== "chat" && (
        <span className="inline-flex items-center gap-1 rounded-ctl bg-brand/15 px-2 py-1 text-caption font-semibold text-brand">
          {actionMode === "video" ? (
            <>
              <Film className="h-3 w-3" />
              {isVi ? "Chế độ: Ảnh → Video" : "Mode: Image → Video"}
            </>
          ) : actionMode === "image_list" ? (
            <>
              <Images className="h-3 w-3" />
              {isVi ? "Chế độ: Tạo Node List Ảnh (4x)" : "Mode: Image List Node (4x)"}
            </>
          ) : (
            <>
              <Image className="h-3 w-3" />
              {isVi ? "Chế độ: Tạo Node Ảnh" : "Mode: Single Image Node"}
            </>
          )}
          <button
            type="button"
            onClick={() => onActionModeChange?.("chat")}
            className="ml-1 rounded-full p-1 hover:bg-brand/20"
            aria-label="Reset mode"
          >
            <X className="h-2.5 w-2.5" />
          </button>
        </span>
      )}

      {attachments.map((att) => (
        <div
          key={att.id}
          className="group relative inline-flex items-center gap-2 rounded-ctl border border-border/80 bg-muted/60 pr-2 pl-1 py-1 text-caption text-foreground"
        >
          <img
            src={att.url}
            alt={att.name}
            loading="lazy"
            className="h-5 w-5 rounded-xs object-cover"
          />
          <span className="max-w-[110px] truncate font-medium">{att.name}</span>
          <button
            type="button"
            onClick={() => onRemoveAttachment?.(att.id)}
            className="rounded-full p-1 text-muted-foreground hover:bg-accent hover:text-foreground"
            aria-label={`Remove ${att.name}`}
          >
            <X className="h-3 w-3" />
          </button>
        </div>
      ))}

      {craftTags.map((tag) => (
        <span
          key={tag}
          className="inline-flex items-center gap-1 rounded-full border border-border/70 bg-muted/40 px-2 py-1 text-caption font-medium text-foreground"
        >
          <Wand2 className="h-2.5 w-2.5 text-brand" />
          {tag}
          <button
            type="button"
            onClick={() => onToggleCraftTag?.(tag)}
            className="text-muted-foreground hover:text-foreground"
            aria-label={`Remove ${tag}`}
          >
            <X className="h-2.5 w-2.5" />
          </button>
        </span>
      ))}
    </div>
  );
}
