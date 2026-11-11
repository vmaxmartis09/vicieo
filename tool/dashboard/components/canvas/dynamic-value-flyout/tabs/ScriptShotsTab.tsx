"use client";

import React from "react";
import type { ScriptDataState } from "../types";

export function ScriptShotsTab({
  loading,
  scriptData,
  onSelect,
}: {
  loading: boolean;
  scriptData: ScriptDataState | null;
  onSelect: (val: string) => void;
}) {
  return (
    <div className="max-h-[260px] overflow-y-auto space-y-2 pr-1 scroll-quiet">
      {loading ? (
        <p className="py-4 text-center text-footnote text-muted-foreground">Đang tải kịch bản…</p>
      ) : scriptData?.shots && scriptData.shots.length > 0 ? (
        scriptData.shots.map((shot: any, idx: number) => (
          <div key={shot.id || idx} className="rounded-card border border-border/60 p-2 space-y-1 bg-card/60">
            <span className="text-caption font-semibold text-muted-foreground uppercase">
              Cảnh {idx + 1}: {shot.sceneTitle || shot.speaker || "Phân cảnh"}
            </span>

            {shot.prompt && (
              <button
                type="button"
                onClick={() => onSelect(`{Kịch bản.cảnh_${idx + 1}.prompt}`)}
                className="flex w-full items-center justify-between rounded-ctl p-1 text-left text-footnote hover:bg-accent text-foreground transition-colors"
              >
                <span className="line-clamp-1 text-caption text-muted-foreground">{shot.prompt}</span>
                <code className="text-caption font-mono font-semibold text-brand shrink-0 ml-1">
                  {`{cảnh_${idx + 1}.prompt}`}
                </code>
              </button>
            )}

            {shot.dialogue && (
              <button
                type="button"
                onClick={() => onSelect(`{Kịch bản.cảnh_${idx + 1}.dialogue}`)}
                className="flex w-full items-center justify-between rounded-ctl p-1 text-left text-footnote hover:bg-accent text-foreground transition-colors"
              >
                <span className="line-clamp-1 text-caption text-foreground">{shot.dialogue}</span>
                <code className="text-caption font-mono font-semibold text-brand shrink-0 ml-1">
                  {`{cảnh_${idx + 1}.thoại}`}
                </code>
              </button>
            )}
          </div>
        ))
      ) : (
        <p className="py-4 text-center text-footnote text-muted-foreground">Chưa có dữ liệu phân cảnh trong kịch bản</p>
      )}
    </div>
  );
}
