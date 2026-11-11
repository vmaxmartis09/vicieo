"use client";

import React, { useState, useEffect } from "react";
import { useStudioStore } from "@/store";
import type { Character } from "@/lib/repositories";
import { fetchScriptData } from "@/lib/api/nodes";
import { useHubScope } from "@/lib/hub";
import { CanvasChip } from "./CharacterHero";
import { MorphIcon, UsersNode } from "@/components/ui/icons";
import { cn } from "@/lib/utils";
import {
  extractCastView,
  getCachedCast,
  resolveCastMembers,
  setCachedCast,
  type CastView,
} from "./node-body/utils/castExtractor";

export interface CastPreviewProps {
  nodeId: string;
  data: any;
}

export function CastPreview({ nodeId, data }: CastPreviewProps) {
  const projectId = useStudioStore((st: any) => st.activeProjectId) as string;
  const characters = useStudioStore((st: any) => st.characters) as Character[];
  const episodeId = useStudioStore((st: any) => {
    for (const e of st.edges) {
      if (e.target !== nodeId) continue;
      const src = st.nodes.find((n: any) => n.id === e.source);
      if (src?.data?.nodeType === "script" && src.data.params?.mode === "episode") {
        return String(src.data.params.episodeId || "");
      }
    }
    return "";
  });
  const include: string[] = data?.params?.include || [];
  const exclude: string[] = data?.params?.exclude || [];
  const key = `${projectId}:${episodeId}`;

  // Giữ dàn nhân vật cũ (mờ đi) trong lúc tải dàn mới, và nhớ các tập đã xem -> đổi tập là đổi ngay, không nháy trống
  const [view, setView] = useState<CastView | null>(() => getCachedCast(key) || null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (!episodeId) {
      setView(null);
      return;
    }
    const hit = getCachedCast(key);
    if (hit) {
      setView(hit);
      setLoading(false);
      return;
    }
    let alive = true;
    setLoading(true);
    fetchScriptData(projectId, episodeId).then((d: any) => {
      if (!alive) return;
      const v = extractCastView(d);
      if (d) setCachedCast(key, v);
      setView(v);
      setLoading(false);
    });
    return () => {
      alive = false;
    };
  }, [key, projectId, episodeId]);

  const cast = view?.cast ?? null;
  const lineCount = view?.lineCount ?? {};
  const unmatched = view?.unmatched ?? [];

  if (!episodeId) {
    return (
      <p className="text-footnote text-muted-foreground">
        Nối cổng <span className="font-medium text-foreground">lines</span> của Kịch bản (chế độ Kịch bản tập) để tự lấy nhân vật theo tập.
      </p>
    );
  }

  const members = resolveCastMembers(cast, include, exclude);

  return (
    <div className="space-y-2">
      <div className="text-caption text-muted-foreground">
        Tập <span className="font-mono text-foreground">{episodeId.replace(/^ep(\d+)-.*/, "$1")}</span> · {members.length} nhân vật
      </div>
      {cast === null && <span className="text-footnote text-muted-foreground">Đang đọc kịch bản…</span>}
      <div className={cn("space-y-2 transition-opacity duration-200", loading && "opacity-50")}>
        {members.map((m) => {
          const ch = m.id ? characters.find((c) => c.id === m.id) : undefined;
          return (
            <div key={m.id || m.name} className="flex items-center gap-2 rounded-card border border-border bg-card/60 p-2">
              {ch?.referenceImages?.[0] ? (
                // eslint-disable-next-line @next/next/no-img-element
                <img
                  decoding="async"
                  loading="lazy"
                  src={ch.referenceImages[0]}
                  alt={ch.name}
                  className="h-14 w-14 shrink-0 rounded-ctl border border-border bg-muted/20 object-cover object-top"
                />
              ) : (
                <div className="flex h-14 w-14 shrink-0 items-center justify-center rounded-ctl border border-dashed border-semantic-warning text-caption text-semantic-warning">
                  chưa có
                </div>
              )}
              <div className="min-w-0 flex-1 space-y-1">
                <button
                  type="button"
                  title="Mở hồ sơ nhân vật"
                  onClick={(e) => {
                    e.stopPropagation();
                    if (m.id) useHubScope.getState().openTab("characters", null, m.id);
                  }}
                  className="nodrag block max-w-full truncate text-left text-body font-semibold text-foreground underline-offset-2 hover:underline"
                >
                  {ch?.name || m.name}
                </button>
                <div className="flex flex-wrap items-center gap-1">
                  {ch && <CanvasChip title="Chiều cao">{ch.heightCm} cm</CanvasChip>}
                  {ch && <CanvasChip title="Giọng">{ch.voiceId || "—"}</CanvasChip>}
                  {m.id && lineCount[m.id] ? <CanvasChip title="Số câu thoại">{lineCount[m.id]} câu</CanvasChip> : null}
                </div>
              </div>
            </div>
          );
        })}
      </div>
      {unmatched.length > 0 && <div className="text-caption text-semantic-warning">Chưa ghép được: {unmatched.join(", ")}</div>}
      {members.some((m) => m.id) && (
        <button
          type="button"
          title="Tách thành từng node Nhân vật để xem và chỉnh riêng (nhân vật cố định theo tập hiện tại). Hoàn tác bằng Ctrl+Z."
          onClick={(e) => {
            e.stopPropagation();
            const ids = members.map((m) => m.id).filter(Boolean) as string[];
            (useStudioStore.getState() as any).expandCastToCharacters(nodeId, ids);
          }}
          className="nodrag flex h-7 w-full items-center justify-center gap-2 rounded-ctl border border-border bg-card px-2 py-1 text-caption font-medium text-foreground transition-colors hover:bg-accent"
        >
          <MorphIcon icon={UsersNode} size={12} />
          Phân rã thành {members.filter((m) => m.id).length} node nhân vật
        </button>
      )}
    </div>
  );
}
