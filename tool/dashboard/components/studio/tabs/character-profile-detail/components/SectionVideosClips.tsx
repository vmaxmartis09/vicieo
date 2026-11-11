"use client";

import React from "react";
import { useTranslations } from "next-intl";
import type { CharacterClipAsset } from "@/lib/repositories/types";

interface SectionVideosClipsProps {
  clips: CharacterClipAsset[];
}

export function SectionVideosClips({ clips }: SectionVideosClipsProps) {
  const t = useTranslations("projectTab");
  return (
    <div className="space-y-3">
      {clips.length === 0 ? (
        <p className="text-muted-foreground text-footnote">{t("videos.empty")}</p>
      ) : (
        clips.map((clip) => (
          <div
            key={clip.id}
            className="rounded-card border border-border bg-card p-3 space-y-2"
          >
            <video
              src={clip.url}
              controls
              preload="metadata"
              className="h-40 w-full rounded-ctl border border-border object-cover bg-muted/20"
            />
            <div className="flex items-center justify-between">
              <span className="font-medium text-foreground text-footnote">
                {t("videos.episodeScene", {
                  episode: clip.episodeTitle,
                  scene: clip.sceneLabel,
                })}
              </span>
              <span className="font-mono text-body text-muted-foreground tnum">
                {clip.durationSec}s
              </span>
            </div>
          </div>
        ))
      )}
    </div>
  );
}
