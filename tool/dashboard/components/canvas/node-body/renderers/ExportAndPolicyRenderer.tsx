"use client";

import React from "react";
import { Reveal } from "../../Reveal";
import { NodeExportInput } from "../../NodeInputs";
import { usePreview } from "@/lib/preview";
import { CanvasChip } from "../../CharacterHero";
import { Film, ShieldCheck } from "@/components/ui/icons";
import { cn } from "@/lib/utils";
import { Empty } from "../components/Empty";
import { ImagePreview } from "../components/ImagePreview";
import { PolicyBadge } from "../components/PolicyBadge";
import { EpisodeOutputPreview, posterFor } from "../components/EpisodeOutputPreview";
import type { NodeRendererContextProps } from "../types";

const HYPERFRAMES_TRACKS: [string, string][] = [
  ["video", "bg-[#b4531a]"],
  ["audio", "bg-[#a86f0c]"],
  ["captions", "bg-[#737373]"],
  ["bgm", "bg-[#a86f0c]/50"],
];

export function ExportAndPolicyRenderer({
  nodeType,
  nodeId,
  data,
  params: p,
  previewUrl: url,
  lines,
  connectedInputs,
  compact,
  expanded,
  updateNodeParams,
}: NodeRendererContextProps) {
  switch (nodeType) {
    case "hyperframes_render":
      return (
        <div className="space-y-2">
          <div
            className="space-y-1 rounded-ctl border border-border bg-muted/20 p-2"
            aria-label="timeline"
          >
            {HYPERFRAMES_TRACKS.map(([name, color]) => {
              const on = connectedInputs.includes(name);
              return (
                <div key={name} className="flex items-center gap-2" title={name}>
                  <span className="w-12 shrink-0 text-caption text-muted-foreground">
                    {name}
                  </span>
                  <span
                    className={cn("h-2 flex-1 rounded-full", on ? color : "bg-border/60")}
                  />
                </div>
              );
            })}
          </div>
          {url && data.previewType === "video" && (
            <video
              src={url}
              controls
              preload="metadata"
              className="nodrag max-h-48 w-full rounded-ctl bg-black object-contain"
            />
          )}
        </div>
      );

    case "export":
      return (
        <div className="space-y-2">
          <Reveal show={Boolean(expanded)}>
            <NodeExportInput
              nodeId={nodeId}
              outputFileName={p.outputFileName}
              platforms={p.platforms}
              previewUrl={url}
            />
          </Reveal>
          {!expanded && !(url && data.previewType === "video") && (
            <div className="space-y-1 rounded-ctl border border-dashed border-border p-2 text-footnote">
              <div className="truncate font-medium text-foreground">
                {p.outputFileName || "Chưa đặt tên file"}
              </div>
              {Array.isArray(p.platforms) && p.platforms.length > 0 && (
                <div className="flex flex-wrap gap-1">
                  {p.platforms.map((pl: string) => (
                    <CanvasChip key={pl} title="Nền tảng">
                      {pl}
                    </CanvasChip>
                  ))}
                </div>
              )}
            </div>
          )}
          {!(url && data.previewType === "video") && (
            <EpisodeOutputPreview status={data.status} outputFileName={p.outputFileName} />
          )}
          {url && data.previewType === "video" && (
            <>
              <video
                src={url}
                controls
                preload="metadata"
                poster={posterFor(url)}
                className="nodrag max-h-48 w-full rounded-ctl bg-black object-contain"
              />
              <button
                type="button"
                title="Mở video lớn để xem và tua"
                data-testid="export-open-preview"
                onClick={() =>
                  usePreview.getState().open({
                    kind: "video",
                    title: p.outputFileName || "Thành phẩm",
                    index: 0,
                    items: [
                      {
                        url,
                        name: p.outputFileName || url.split("/").pop() || "video",
                        poster: posterFor(url),
                      },
                    ],
                  })
                }
                className="nodrag flex h-7 w-full items-center justify-center gap-1 rounded-ctl border border-border py-1 text-caption font-medium text-foreground hover:bg-accent"
              >
                <Film className="h-3 w-3" />
                Xem lớn
              </button>
            </>
          )}
        </div>
      );

    case "policy":
      return (
        <div className="space-y-2">
          <PolicyBadge />
          <Reveal show={Boolean(expanded)}>
            <label className="block space-y-1 pt-1">
              <span className="text-body text-muted-foreground">
                Từ khoá bổ sung riêng cho node này (mỗi dòng một mục)
              </span>
              <textarea
                rows={3}
                value={Array.isArray(p.extraTerms) ? p.extraTerms.join("\n") : ""}
                onChange={(e) =>
                  updateNodeParams(nodeId, { extraTerms: e.target.value.split("\n") })
                }
                className="nodrag nowheel w-full resize-y rounded-ctl border border-border bg-card p-2 text-footnote text-foreground focus:outline-none focus:ring-1 focus:ring-brand"
              />
            </label>
          </Reveal>
        </div>
      );

    case "quality_check":
      return (
        <div className="space-y-2">
          {url && data.previewType === "image" && (
            <ImagePreview url={url} alt={nodeType} compact={compact} />
          )}
          {lines.length === 0 && !(url && data.previewType === "image") && (
            <Empty icon={<ShieldCheck className="h-5 w-5" />} text="Chưa kiểm tra" />
          )}
          <PolicyBadge />
        </div>
      );

    case "vision_analyze":
      if (url && data.previewType === "image") {
        return <ImagePreview url={url} alt={nodeType} compact={compact} />;
      }
      return lines.length > 0 ? null : (
        <Empty icon={<ShieldCheck className="h-5 w-5" />} text="Chưa phân tích" />
      );

    default:
      return null;
  }
}
