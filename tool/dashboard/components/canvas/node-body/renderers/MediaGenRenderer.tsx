"use client";

import React from "react";
import { Reveal } from "../../Reveal";
import { VideoGenParams } from "../../VideoGenParams";
import { NodeImageGenInput } from "../../NodeInputs";
import { ImagePreview } from "../components/ImagePreview";
import type { NodeRendererContextProps } from "../types";

export function MediaGenRenderer({
  nodeType,
  nodeId,
  data,
  params: p,
  previewUrl: url,
  connectedInputs,
  compact,
  expanded,
  updateNodeParams,
}: NodeRendererContextProps) {
  switch (nodeType) {
    case "image_gen":
    case "image_edit":
      return (
        <div className="space-y-2 w-full min-w-0">
          {url && (
            <div className="space-y-1 w-full min-w-0">
              <ImagePreview url={url} alt={nodeType} compact={compact} />
            </div>
          )}
          {!url && !expanded && (
            <div className="rounded-card border border-border/80 bg-muted/20 p-2 space-y-1 w-full min-w-0">
              <div className="flex items-center justify-between text-caption text-muted-foreground gap-2 min-w-0">
                <span className="font-mono font-medium truncate min-w-0">{p.model || "pollinations-sana"}</span>
                <span className="text-caption uppercase font-semibold shrink-0">{p.aspectRatio || "9:16"}</span>
              </div>
              <p className="line-clamp-2 text-footnote text-foreground/80 italic break-words">
                {p.prompt || "Chưa có prompt mô tả..."}
              </p>
            </div>
          )}
          <Reveal show={Boolean(expanded)}>
            <NodeImageGenInput
              nodeId={nodeId}
              prompt={p.prompt}
              referenceImages={p.referenceImages}
              aspectRatio={p.aspectRatio}
              model={p.model}
              connectedPrompt={connectedInputs.includes("prompt")}
            />
          </Reveal>
        </div>
      );

    case "video_gen":
      return (
        <div className="space-y-2">
          {url && data.previewType === "video" && (
            <video
              src={url}
              controls
              preload="metadata"
              className="nodrag max-h-48 w-full rounded-ctl border border-border bg-black object-contain"
            />
          )}
          <VideoGenParams
            nodeId={nodeId}
            params={p}
            connectedPrompt={connectedInputs.includes("prompt")}
            expanded={Boolean(expanded)}
          />
        </div>
      );

    case "transition":
      return (
        <div className="space-y-2 text-footnote">
          <div className="flex items-center gap-2">
            <span className="text-muted-foreground w-16">Hiệu ứng:</span>
            <select
              value={p.style || "fade"}
              onChange={(e) => updateNodeParams(nodeId, { style: e.target.value })}
              className="nodrag h-7 flex-1 rounded-ctl border border-border bg-card px-2 text-footnote text-foreground"
            >
              <option value="fade">Fade (Mờ dần)</option>
              <option value="dissolve">Dissolve (Hòa tan)</option>
              <option value="wipe_left">Wipe (Gạt ngang)</option>
              <option value="slide_up">Slide (Trượt lên)</option>
            </select>
          </div>
          <div className="flex items-center gap-2">
            <span className="text-muted-foreground w-16">Thời lượng:</span>
            <input
              type="number"
              step="0.1"
              min="0.1"
              max="3"
              value={p.durationSec ?? 0.5}
              onChange={(e) =>
                updateNodeParams(nodeId, {
                  durationSec: Number(e.target.value) || 0.5,
                })
              }
              className="nodrag h-7 w-20 rounded-ctl border border-border bg-card px-2 text-footnote tnum text-foreground"
            />
            <span className="text-muted-foreground">giây</span>
          </div>
        </div>
      );

    default:
      return null;
  }
}
