"use client";

import React from "react";
import { CORE_URL } from "@/lib/repositories/core";
import { usePreview } from "@/lib/preview";
import { CalendarDays, CheckCircle2, Film, Play } from "@/components/ui/icons";

interface ReadyOutputsSectionProps {
  activeProjectId: string;
  outputsList: any[];
  onScheduleOutput: (out: any) => void;
}

export function ReadyOutputsSection({
  activeProjectId,
  outputsList,
  onScheduleOutput,
}: ReadyOutputsSectionProps) {
  if (outputsList.length === 0) return null;

  return (
    <section className="rounded-card border border-emerald-500/30 bg-emerald-500/5 p-4 space-y-3">
      <div className="flex items-center justify-between">
        <h3 className="font-semibold text-foreground flex items-center gap-2">
          <CheckCircle2 className="h-4 w-4 text-semantic-success" />
          <span>Video Thành phẩm ({outputsList.length})</span>
        </h3>
        <span className="text-caption text-emerald-400 font-medium">
          data/projects/{activeProjectId}/output/
        </span>
      </div>

      <div className="space-y-2">
        {outputsList.map((out) => {
          const videoRel = out.files?.video || `output/${out.slug}/final_45s.mp4`;
          const thumbRel = out.files?.thumb || `output/${out.slug}/thumb.jpg`;
          const videoFullUrl = `${CORE_URL}/projects/${activeProjectId}/${videoRel}`;
          const thumbFullUrl = `${CORE_URL}/projects/${activeProjectId}/${thumbRel}`;
          const duration = Number(out.duration_s) || 0;
          const epVerify = out.episodeVerify as
            | { verdict?: string; pass1to5?: boolean }
            | undefined;
          const verifyLabel = epVerify
            ? epVerify.pass1to5
              ? "✓ verify 1–5 ĐẠT"
              : "✗ verify CHƯA ĐẠT"
            : out.verify?.pass === true
              ? "✓ kiểm nhanh đạt"
              : null;
          const verifyOk = epVerify ? Boolean(epVerify.pass1to5) : out.verify?.pass === true;
          const engine = out.engine || out.render_engine;
          const caption = [
            engine ? `engine: ${engine}` : null,
            out.engineWarning ? `⚠ ${out.engineWarning}` : null,
            epVerify?.verdict || null,
          ]
            .filter(Boolean)
            .join(" · ");

          const openInStudio = () =>
            usePreview.getState().open({
              kind: "video",
              title: out.title || out.slug,
              index: 0,
              items: [{ url: videoFullUrl, name: videoRel, poster: thumbFullUrl, caption }],
            });

          return (
            <div
              key={out.id || out.slug}
              className="virtual-row rounded-card border border-border bg-card/80 p-3 flex items-center justify-between gap-3 shadow-xs hover:border-emerald-500/40 transition-colors"
            >
              <div className="relative w-12 h-16 rounded-ctl overflow-hidden bg-black/40 border border-border/80 shrink-0 flex items-center justify-center">
                <img
                  decoding="async"
                  loading="lazy"
                  src={thumbFullUrl}
                  alt={out.title}
                  className="w-full h-full object-cover"
                  onError={(e) => {
                    (e.target as HTMLElement).style.display = "none";
                  }}
                />
                <Film className="w-5 h-5 text-muted-foreground absolute" />
              </div>

              <div className="flex-1 min-w-0 space-y-1">
                <h4 className="font-semibold text-foreground text-xs truncate">
                  {out.title || out.slug}
                </h4>
                <div className="flex items-center gap-2 text-caption text-muted-foreground font-mono">
                  <span>
                    ⏱{" "}
                    {duration > 0
                      ? `${Math.floor(duration / 60)}:${String(Math.round(duration % 60)).padStart(2, "0")}`
                      : "—"}
                  </span>
                  <span>💾 {out.size_mb || 0} MB</span>
                  {engine && (
                    <span title={out.engineWarning || undefined}>
                      {engine === "hyperframes" ? "HF" : engine}
                    </span>
                  )}
                  {verifyLabel && (
                    <span
                      className={
                        verifyOk ? "text-emerald-400 font-semibold" : "text-semantic-warning font-semibold"
                      }
                    >
                      {verifyLabel}
                    </span>
                  )}
                </div>
                <p className="text-caption text-zinc-500 truncate font-mono">
                  {out.folder_rel || `output/${out.slug}`}
                </p>
              </div>

              <div className="flex flex-col gap-1 shrink-0">
                <button
                  type="button"
                  onClick={openInStudio}
                  title="Xem và tua ngay trong Studio"
                  data-testid="output-play"
                  className="flex items-center justify-center gap-1 px-2 py-1 rounded-ctl bg-accent hover:bg-accent/80 text-caption font-medium text-foreground transition-colors"
                >
                  <Play className="w-3 h-3 text-primary" />
                  <span>Xem</span>
                </button>
                <button
                  type="button"
                  onClick={() => onScheduleOutput(out)}
                  className="flex items-center justify-center gap-1 px-2 py-1 rounded-ctl bg-primary text-primary-foreground hover:bg-primary/90 text-caption font-semibold transition-colors shadow-2xs"
                >
                  <CalendarDays className="w-3 h-3" />
                  <span>Đăng</span>
                </button>
              </div>
            </div>
          );
        })}
      </div>
    </section>
  );
}
