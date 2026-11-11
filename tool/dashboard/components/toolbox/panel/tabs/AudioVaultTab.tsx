"use client";

import React from "react";
import { Mic, FolderOpen } from "@/components/ui/icons";
import { cn } from "@/lib/utils";
import { CARD } from "../constants";
import { ToolboxSaveBar } from "../shared/ToolboxSaveBar";
import type { Targets } from "../types";

export function AudioVaultTab({
  audios,
  targets,
  saveFor,
  setSaveFor,
  onGoToSttAlign,
}: {
  audios: any[];
  targets: Targets | null;
  saveFor: string | null;
  setSaveFor: (id: string | null) => void;
  onGoToSttAlign: () => void;
}) {
  if (audios.length === 0) {
    return (
      <div className="col-span-2 space-y-1 rounded-xl border border-dashed border-border p-4">
        <Mic className="h-5 w-5 text-muted-foreground" />
        <p className="text-footnote font-medium">Chưa có bản phiên âm</p>
        <button type="button" onClick={onGoToSttAlign} className="text-caption text-brand hover:underline">
          Phiên âm một file âm thanh ngay
        </button>
      </div>
    );
  }
  return (
    <>
      {audios.map((item) => (
        <div key={item.id} className={cn(CARD, "space-y-2 border-border")}>
          <div className="flex items-center justify-between text-caption">
            <span className="font-mono font-medium">#{item.id}</span>
            <span className="text-muted-foreground">
              {item.metadata?.duration}s · {String(item.metadata?.language || "").toUpperCase()}
            </span>
          </div>
          <p className="rounded-lg bg-muted/50 p-3 text-footnote leading-relaxed">“{item.metadata?.text_preview}”</p>
          {saveFor === `a${item.id}` ? (
            <ToolboxSaveBar
              targets={targets}
              files={[
                ...(item.files?.timings ? [{ label: "Canh từng từ (.json)", path: item.files.timings }] : []),
                ...(item.files?.subtitles ? [{ label: "Phụ đề (.srt)", path: item.files.subtitles }] : []),
              ]}
            />
          ) : (
            <button type="button" onClick={() => setSaveFor(`a${item.id}`)} className="flex h-7 items-center gap-1 rounded-md border border-border px-2 text-caption font-medium transition-all hover:bg-accent active:scale-95">
              <FolderOpen className="h-3.5 w-3.5" />
              Tải về / Lưu vào…
            </button>
          )}
        </div>
      ))}
    </>
  );
}
