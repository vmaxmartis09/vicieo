"use client";

import React, { useState, useEffect, useMemo } from "react";
import { useStudioStore } from "@/store";
import {
  fetchProjectScriptEpisodes,
  fetchScriptData,
  type ScriptEpisode,
  type ScriptShot,
} from "@/lib/api/nodes";
import {
  Check,
  AlertTriangle,
} from "@/components/ui/icons";
import { cn } from "@/lib/utils";

export interface EpisodeScriptPickerProps {
  nodeId: string;
  episodeId?: string;
  granularity: "shot" | "scene";
}

/** Chọn kịch bản tập (content/episodes/<id>/KICH-BAN-*.md) và độ chia: mỗi shot hay mỗi cảnh là 1 phần tử. */
export function EpisodeScriptPicker({
  nodeId,
  episodeId,
  granularity,
}: EpisodeScriptPickerProps) {
  const updateNodeParams = useStudioStore((s) => s.updateNodeParams);
  const projectId = useStudioStore((s: any) => s.activeProjectId) as string;
  const [episodes, setEpisodes] = useState<ScriptEpisode[] | null>(null);
  const [shots, setShots] = useState<ScriptShot[] | null>(null);

  useEffect(() => {
    let alive = true;
    fetchProjectScriptEpisodes(projectId).then((eps) => alive && setEpisodes(eps));
    return () => {
      alive = false;
    };
  }, [projectId]);

  useEffect(() => {
    if (!episodeId) {
      setShots(null);
      return;
    }
    let alive = true;
    fetchScriptData(projectId, episodeId).then((d) => alive && setShots(d?.shots || []));
    return () => {
      alive = false;
    };
  }, [projectId, episodeId]);

  const hasFixedCharacters = useStudioStore((s: any) =>
    s.nodes.some((n: any) => n.data.nodeType === "character") && !s.nodes.some((n: any) => n.data.nodeType === "cast")
  );
  const convertToCast = useStudioStore((s: any) => s.convertCharactersToCast) as () => boolean;
  const ensureCast = useStudioStore((s: any) => s.ensureCastForEpisode) as (scriptNodeId: string) => boolean;
  const sceneCount = shots ? new Set(shots.map((s) => s.scene)).size : 0;
  const itemCount = granularity === "scene" ? sceneCount : shots?.length || 0;
  // Lời thoại theo người nói (giống cổng 'lines'): đếm câu, báo người nói chưa ghép được nhân vật
  const speakerStats = useMemo(() => {
    const spoken = (shots || []).filter((s) => s.dialogue && s.dialogue.trim());
    const by = new Map<string, { count: number; matched: boolean }>();
    for (const s of spoken) {
      const name = (s.speaker || "").replace(/\s*\([^)]*\)/g, "").trim() || "Người dẫn";
      const cur = by.get(name) || { count: 0, matched: Boolean(s.characterId) || !s.speaker };
      cur.count += 1;
      by.set(name, cur);
    }
    return { total: spoken.length, rows: Array.from(by.entries()) };
  }, [shots]);
  const unmatched = speakerStats.rows.filter(([, v]) => !v.matched);

  return (
    <div className="space-y-2">
      <select
        value={episodeId || ""}
        onChange={(e) => {
          updateNodeParams(nodeId, { episodeId: e.target.value });
          // Chọn kịch bản mới → tự vào chế độ Dàn diễn viên (nhân vật theo tập, không cần thao tác thêm)
          if (e.target.value) ensureCast(nodeId);
        }}
        className="nodrag h-7 w-full rounded-ctl border border-border bg-card px-2 py-1 text-footnote text-foreground"
      >
        <option value="">{episodes === null ? "Đang tải danh sách tập…" : "Chọn kịch bản tập"}</option>
        {(episodes || []).map((ep) => (
          <option key={ep.id} value={ep.id}>
            {ep.title}
          </option>
        ))}
      </select>
      <div className="flex rounded-ctl border border-border bg-muted/30 p-1 text-caption">
        {(["shot", "scene"] as const).map((g) => (
          <button
            key={g}
            type="button"
            onClick={() => updateNodeParams(nodeId, { granularity: g })}
            className={cn(
              "nodrag flex-1 rounded-xs py-1 font-medium transition-all",
              granularity === g ? "bg-card text-foreground shadow-xs" : "text-muted-foreground hover:text-foreground"
            )}
          >
            {g === "shot" ? "Mỗi shot 1 ảnh" : "Mỗi cảnh 1 ảnh"}
          </button>
        ))}
      </div>
      {episodeId && shots && (
        <div className="flex items-center gap-2 rounded-ctl border border-border bg-muted/20 px-2 py-2 text-caption text-muted-foreground">
          <Check className="h-4 w-4 text-semantic-success" />
          <span>
            <span className="font-semibold text-foreground">{itemCount}</span> {granularity === "scene" ? "cảnh" : "shot"} ·{" "}
            {sceneCount} cảnh / {shots.length} shot trong kịch bản
          </span>
        </div>
      )}
      {episodeId && shots && speakerStats.total > 0 && (
        <div className="rounded-ctl border border-border bg-muted/20 px-2 py-2 text-caption text-muted-foreground">
          <span className="font-semibold text-foreground">{speakerStats.total}</span> câu thoại ·{" "}
          {speakerStats.rows.map(([name, v], i) => (
            <span key={name} className={cn(!v.matched && "text-semantic-warning")}>
              {i > 0 && " · "}
              {name} {v.count}
            </span>
          ))}
          {unmatched.length > 0 && (
            <div className="mt-1 flex items-start gap-1 text-semantic-warning">
              <AlertTriangle className="mt-px h-3 w-3 shrink-0" />
              <span>Chưa ghép được nhân vật: {unmatched.map(([n]) => n).join(", ")}. Thêm tên này vào aliases của nhân vật.</span>
            </div>
          )}
        </div>
      )}
      {hasFixedCharacters && (
        <div className="rounded-ctl border border-semantic-warning/50 bg-semantic-warning/5 px-2 py-2 text-caption leading-snug text-foreground">
          Flow đang dùng nhân vật <b>cố định</b>: đổi tập thì nhân vật không đổi theo.
          <button
            type="button"
            onClick={() => convertToCast()}
            className="nodrag mt-1 block h-7 w-full rounded-ctl border border-border bg-card px-2 text-center font-medium transition-colors hover:bg-accent"
          >
            Chuyển sang Dàn diễn viên (tự theo tập)
          </button>
        </div>
      )}
      <p className="text-caption leading-snug text-muted-foreground">
        Nối cổng <span className="font-medium text-foreground">scenes</span> sang node Tạo ảnh để tạo ảnh cho tất cả cảnh trong 1 node.
        Cổng <span className="font-medium text-foreground">lines</span> là lời thoại kèm người nói.
      </p>
    </div>
  );
}
