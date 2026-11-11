"use client";

import React from "react";
import { CORE_URL } from "@/lib/repositories/core";
import { useStudioStore } from "@/store";
import { Download } from "@/components/ui/icons";

/** Tải "gói shot" (ảnh khung đầu + prompt video từng shot chính) để làm clip tay rồi thả vào clips/ (C5). */
export function ShotPackLink({ episodeId }: { episodeId: string }) {
  const projectId = useStudioStore((s) => s.activeProjectId);
  if (!projectId) return null;
  const href = `${CORE_URL}/data/projects/${encodeURIComponent(projectId)}/episodes/${encodeURIComponent(episodeId)}/shot-pack`;
  return (
    <a
      href={href}
      download={`${episodeId}_shot-pack.zip`}
      title="ZIP gồm ảnh khung đầu + prompt video của các shot chính (priority 1). Làm clip xong đặt tên S07.mp4 và thả vào content/episodes/<tập>/clips/"
      data-testid="shot-pack-download"
      className="nodrag mt-2 flex h-7 w-full items-center justify-center gap-1 rounded-ctl border border-dashed border-border py-1 text-caption font-medium text-muted-foreground hover:bg-accent hover:text-foreground"
    >
      <Download className="h-3 w-3" />
      Gói shot làm clip tay (ZIP)
    </a>
  );
}
