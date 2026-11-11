"use client";

import React from "react";
import { Users } from "@/components/ui/icons";
import type { CanvasImageItem, ChatAttachment, SlashCommandItem } from "../types";

interface Props {
  isSlash: boolean;
  isMention: boolean;
  input: string;
  isVi: boolean;
  slashCommands: SlashCommandItem[];
  filteredChars: { id: string; name: string; heightCm: number }[];
  filteredCanvasImgs: CanvasImageItem[];
  onSlashSelect: (cmd: string) => void;
  onMentionSelect: (charId: string, charName: string) => void;
  onAddAttachment?: (att: ChatAttachment) => void;
  onInputChange: (val: string) => void;
  onFocusTextarea: () => void;
}

export function AutocompletePopovers({
  isSlash,
  isMention,
  input,
  isVi,
  slashCommands,
  filteredChars,
  filteredCanvasImgs,
  onSlashSelect,
  onMentionSelect,
  onAddAttachment,
  onInputChange,
  onFocusTextarea,
}: Props) {
  return (
    <>
      {/* Autocomplete Menu: Slash Commands */}
      {isSlash && (
        <ul
          data-chat-popover=""
          className="absolute bottom-full left-0 right-0 mb-2 max-h-52 overflow-y-auto rounded-panel border border-input bg-popover p-2 shadow-2xl ring-1 ring-foreground/15 animate-in fade-in-0 zoom-in-95 z-50 scroll-quiet"
        >
          {slashCommands
            .filter((x) => x.cmd.startsWith(input.trim()))
            .map((item) => (
              <li key={item.cmd}>
                <button
                  type="button"
                  onClick={() => {
                    onSlashSelect(item.cmd);
                    onFocusTextarea();
                  }}
                  className="flex w-full items-center gap-2 rounded-ctl px-3 py-2 text-left text-footnote hover:bg-accent transition-colors"
                >
                  <code className="rounded-xs bg-muted px-2 py-1 font-semibold text-foreground text-caption">
                    {item.cmd}
                  </code>
                  <span className="text-muted-foreground">{item.hint}</span>
                </button>
              </li>
            ))}
        </ul>
      )}

      {/* Autocomplete Menu: @ Mentions (Nhân vật + Ảnh trên Canvas giống Topview!) */}
      {isMention && (filteredChars.length > 0 || filteredCanvasImgs.length > 0) && (
        <div
          data-chat-popover=""
          className="absolute bottom-full left-0 right-0 mb-2 max-h-60 overflow-y-auto rounded-panel border border-input bg-popover p-2 shadow-2xl ring-1 ring-foreground/15 animate-in fade-in-0 zoom-in-95 z-50 scroll-quiet space-y-1"
        >
          {filteredCanvasImgs.length > 0 && (
            <div>
              <p className="px-3 py-1 text-caption font-semibold uppercase tracking-wider text-muted-foreground">
                {isVi ? "Ảnh & Tài nguyên trên Canvas" : "Canvas Images & Assets"}
              </p>
              {filteredCanvasImgs.map((img) => (
                <button
                  key={img.id}
                  type="button"
                  onClick={() => {
                    onAddAttachment?.({
                      id: `att_cv_${img.id}`,
                      name: img.name,
                      url: img.url,
                      source: "canvas",
                      nodeId: img.nodeId,
                    });
                    onInputChange(input.replace(/@\S*$/, `@${img.name} `));
                    onFocusTextarea();
                  }}
                  className="flex w-full items-center gap-2 rounded-ctl px-3 py-2 text-left text-footnote hover:bg-accent transition-colors"
                >
                  {/* eslint-disable-next-line @next/next/no-img-element */}
                  <img
                    src={img.url}
                    alt={img.name}
                    loading="lazy"
                    className="h-7 w-7 shrink-0 rounded-ctl border border-border object-cover"
                  />
                  <div className="min-w-0 flex-1">
                    <p className="truncate font-medium text-foreground">{img.name}</p>
                    <p className="text-caption text-muted-foreground">
                      {img.kind === "image_list" ? (isVi ? "Node List Ảnh" : "Image List Node") : "Canvas Image"}
                    </p>
                  </div>
                </button>
              ))}
            </div>
          )}

          {filteredChars.length > 0 && (
            <div>
              <p className="px-3 py-1 text-caption font-semibold uppercase tracking-wider text-muted-foreground">
                {isVi ? "Nhân vật dự án (Khóa tỉ lệ)" : "Project Characters (Scale Locked)"}
              </p>
              {filteredChars.map((ch) => (
                <button
                  key={ch.id}
                  type="button"
                  onClick={() => {
                    onMentionSelect(ch.id, ch.name);
                    onFocusTextarea();
                  }}
                  className="flex w-full items-center gap-2 rounded-ctl px-3 py-2 text-left text-footnote hover:bg-accent transition-colors"
                >
                  <Users className="h-3.5 w-3.5 text-brand" />
                  <span className="font-medium text-foreground">{ch.name}</span>
                  <span className="ml-auto text-caption text-muted-foreground tnum">{ch.heightCm} cm</span>
                </button>
              ))}
            </div>
          )}
        </div>
      )}
    </>
  );
}
