"use client";

import React from "react";
import { Wrench, X, Check } from "@/components/ui/icons";
import { cn } from "@/lib/utils";
import { CARD, IconBox, TOOL_ICON } from "../constants";
import type { Targets, ToolItem } from "../types";
import { BgRemoveWorkspace } from "../workspaces/BgRemoveWorkspace";
import { AudioWorkspace } from "../workspaces/AudioWorkspace";
import { DepthWorkspace } from "../workspaces/DepthWorkspace";
import { SceneWorkspace } from "../workspaces/SceneWorkspace";
import { VoiceCloneWorkspace } from "../workspaces/VoiceCloneWorkspace";

export function ToolsCatalogTab({
  tools,
  open,
  setOpen,
  loadError,
  targets,
  onDone,
  checking,
  checkMsg,
  onCheckFfmpeg,
}: {
  tools: ToolItem[];
  open: string | null;
  setOpen: (id: string | null) => void;
  loadError: string | null;
  targets: Targets | null;
  onDone: () => void;
  checking: boolean;
  checkMsg: string | null;
  onCheckFfmpeg: () => void;
}) {
  return (
    <>
      {tools.length === 0 && !loadError && (
        <div className="rounded-xl border border-dashed border-border p-4 text-footnote text-muted-foreground">
          Chưa có công cụ nào trong <code>tool/toolbox/tools/</code>.
        </div>
      )}
      <div className="grid grid-cols-2 gap-2">
        {tools.map((tool) => {
          const expanded = open === tool.id;
          return (
            <button
              key={tool.id}
              type="button"
              onClick={() => setOpen(expanded ? null : tool.id)}
              aria-expanded={expanded}
              className={cn(CARD, "flex items-start gap-2 text-left hover:bg-accent/40 active:scale-[0.98]", expanded ? "border-brand ring-1 ring-brand/30" : "border-border")}
            >
              <IconBox>{TOOL_ICON[tool.id] ?? <Wrench className="h-4 w-4" />}</IconBox>
              <span className="min-w-0 flex-1 space-y-1">
                <span className="block truncate text-footnote font-medium">{tool.name}</span>
                <span className="flex items-center gap-1 text-caption text-muted-foreground">
                  <span className="font-mono">v{tool.version}</span>
                  <span className={cn("rounded-full px-2 font-medium", tool.status === "verified" ? "bg-semantic-success/10 text-semantic-success" : tool.status === "installed" ? "bg-brand/10 text-brand" : "bg-muted")}>{tool.status}</span>
                </span>
              </span>
            </button>
          );
        })}
      </div>
      {tools
        .filter((t) => t.id === open)
        .map((tool) => {
          const ready = tool.status === "verified" || tool.status === "installed";
          return (
            <div key={tool.id} className="space-y-3 rounded-xl border border-foreground/25 bg-card p-3">
              <div className="flex items-center gap-2">
                <IconBox>{TOOL_ICON[tool.id] ?? <Wrench className="h-4 w-4" />}</IconBox>
                <span className="min-w-0 flex-1">
                  <span className="block text-body font-medium">{tool.name}</span>
                  <span className="block font-mono text-caption text-muted-foreground">{tool.id}</span>
                </span>
                <button type="button" onClick={() => setOpen(null)} className="rounded p-1 text-muted-foreground hover:bg-accent" aria-label="Đóng">
                  <X className="h-3.5 w-3.5" />
                </button>
              </div>
              <div className="flex flex-wrap items-center gap-2 text-caption text-muted-foreground">
                <span className="rounded bg-muted px-2 py-1">{tool.requires_gpu ? "Cần GPU" : "Chạy CPU"}</span>
                {tool.requires_ffmpeg && <span className="rounded bg-muted px-2 py-1">FFmpeg</span>}
                {tool.license?.code && <span className="rounded bg-muted px-2 py-1">{tool.license.code}</span>}
              </div>
              {ready && tool.id === "bg-remove" && <BgRemoveWorkspace targets={targets} onDone={onDone} />}
              {ready && tool.id === "stt-align" && <AudioWorkspace targets={targets} onDone={onDone} />}
              {ready && tool.id === "depth-layers" && <DepthWorkspace targets={targets} onDone={onDone} />}
              {ready && tool.id === "scene-cut" && <SceneWorkspace targets={targets} onDone={onDone} />}
              {ready && tool.id === "voice-clone" && <VoiceCloneWorkspace onDone={onDone} />}
              {ready && tool.id === "ffmpeg-bin" && (
                <div className="space-y-2">
                  <p className="text-caption text-muted-foreground">Bản ffmpeg dùng chung cho mọi tool (không sửa PATH hệ thống).</p>
                  <button type="button" onClick={onCheckFfmpeg} disabled={checking} className="flex h-7 items-center gap-2 rounded-md bg-brand px-3 text-footnote font-medium text-white transition-all hover:opacity-90 active:scale-95 disabled:opacity-40">
                    <Check className="h-3.5 w-3.5" />
                    {checking ? "Đang kiểm tra…" : "Kiểm tra / cài"}
                  </button>
                  {checkMsg && <p className="break-all text-caption text-muted-foreground">{checkMsg}</p>}
                </div>
              )}
              {!ready && <p className="text-caption text-muted-foreground">Công cụ chưa sẵn sàng (trạng thái: {tool.status}).</p>}
            </div>
          );
        })}
    </>
  );
}
