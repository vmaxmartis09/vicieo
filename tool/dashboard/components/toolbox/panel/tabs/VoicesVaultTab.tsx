"use client";

import React from "react";
import { Headphones, FolderOpen } from "@/components/ui/icons";
import { cn } from "@/lib/utils";
import { previewUrl } from "../api";
import { CARD, IconBox } from "../constants";
import { ToolboxSaveBar } from "../shared/ToolboxSaveBar";
import { AudioWorkspace } from "../workspaces/AudioWorkspace";
import { VoiceCloneWorkspace } from "../workspaces/VoiceCloneWorkspace";
import type { Targets } from "../types";

export function VoicesVaultTab({
  voices,
  targets,
  saveFor,
  setSaveFor,
  onDone,
}: {
  voices: any[];
  targets: Targets | null;
  saveFor: string | null;
  setSaveFor: (id: string | null) => void;
  onDone: () => void;
}) {
  return (
    <>
      <p className="px-1 text-caption font-medium text-muted-foreground">Kho giọng dùng chung ({voices.length}) — chọn ở bất kỳ đâu cần giọng</p>
      <div className="grid grid-cols-2 gap-2">
        {voices.map((v) => {
          const m = v.metadata || {};
          const sample = v.files?.samples?.[0];
          const samplePath = sample && targets ? `${targets.sharedDir}/voice_vault/${v.id}/${sample}` : null;
          const ready = m.status === "ready";
          const gender = m.gender === "female" ? "Nữ" : m.gender === "male" ? "Nam" : m.gender === "child" ? "Bé" : null;
          const region = ({ north: "Bắc", south: "Nam", central: "Trung" } as Record<string, string>)[m.dialect] || null;
          return (
            <div key={v.id} className={cn(CARD, "space-y-2 border-border", !ready && "opacity-70")}>
              <div className="flex items-start gap-2">
                <IconBox tone={ready ? "brand" : "muted"}><Headphones className="h-4 w-4" /></IconBox>
                <span className="min-w-0 flex-1">
                  <span className="block truncate text-footnote font-medium">{m.display_name || v.id}</span>
                  <span className="block truncate font-mono text-caption text-muted-foreground">{v.id}</span>
                </span>
              </div>
              <div className="flex flex-wrap gap-1 text-caption">
                <span className={cn("rounded-full px-2 py-1 font-medium", ready ? "bg-semantic-success/10 text-semantic-success" : "bg-semantic-warning/10 text-semantic-warning")}>
                  {ready ? "sẵn sàng" : m.status === "todo_clone" ? "chờ clone" : m.status}
                </span>
                {gender && <span className="rounded-full bg-muted px-2 py-1 text-muted-foreground">{gender}</span>}
                {region && <span className="rounded-full bg-muted px-2 py-1 text-muted-foreground">{region}</span>}
              </div>
              {samplePath && ready && <audio controls src={previewUrl(samplePath)} className="h-7 w-full" />}
              {saveFor === `v${v.id}` && samplePath ? (
                <ToolboxSaveBar targets={targets} files={[{ label: "Mẫu giọng", path: samplePath }]} />
              ) : (
                samplePath &&
                ready && (
                  <button type="button" onClick={() => setSaveFor(`v${v.id}`)} className="flex h-7 items-center gap-1 rounded-md border border-border px-2 text-caption font-medium transition-all hover:bg-accent active:scale-95">
                    <FolderOpen className="h-3.5 w-3.5" />
                    Dùng / Tải về
                  </button>
                )
              )}
            </div>
          );
        })}
      </div>
      <details className="rounded-xl border border-border bg-card p-3">
        <summary className="cursor-pointer text-footnote font-medium">+ Thêm giọng mới (clone / phiên âm)</summary>
        <div className="mt-3 space-y-3">
          <div className="space-y-3 border-t border-border pt-3">
            <p className="text-footnote font-medium">Mẫu giọng của bạn</p>
            <AudioWorkspace targets={targets} onDone={onDone} />
          </div>
          <div className="space-y-3 border-t border-border pt-3">
            <p className="text-footnote font-medium">Clone giọng mới</p>
            <VoiceCloneWorkspace onDone={onDone} />
          </div>
        </div>
      </details>
    </>
  );
}
