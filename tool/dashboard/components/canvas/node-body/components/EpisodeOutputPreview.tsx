"use client";

import React, { useState, useEffect } from "react";
import { CORE_URL } from "@/lib/repositories/core";
import { useStudioStore } from "@/store";
import { usePreview } from "@/lib/preview";
import { fetchProjectOutputs } from "@/lib/core-api";
import { Film } from "@/components/ui/icons";

export function posterFor(url: string): string | undefined {
  return /\/output\/[^/]+\/[^/]+\.mp4(\?.*)?$/.test(url)
    ? url.replace(/[^/]+\.mp4(\?.*)?$/, "thumb.jpg")
    : undefined;
}

/** Thành phẩm mới nhất của tập đang chọn (output/<tập>/video.json qua GET /projects/{id}/outputs):
 * poster + nút "Xem & tua" mở PreviewPane. Chạy thật thì node Xuất không có previewUrl video nên lấy từ đây. */
export function EpisodeOutputPreview({
  status,
  outputFileName,
}: {
  status?: string;
  outputFileName?: string;
}) {
  const projectId = useStudioStore((s) => s.activeProjectId);
  const episodeId = useStudioStore(
    (s) =>
      (s.nodes.find(
        (n) => n.data?.nodeType === "script" && n.data?.params?.mode === "episode"
      )?.data?.params?.episodeId as string | undefined) || ""
  );
  const [out, setOut] = useState<any | null>(null);
  useEffect(() => {
    if (!projectId || !episodeId) return setOut(null);
    let alive = true;
    fetchProjectOutputs(projectId).then((list) => {
      if (!alive) return;
      const want = (outputFileName || "").trim();
      const hit = (list || []).find((o: any) => {
        const v = String(o?.files?.video || "");
        return (
          v.endsWith(`/${episodeId}.mp4`) ||
          (want && v.endsWith(`/${want}`) && String(o?.slug || "").startsWith(episodeId))
        );
      });
      setOut(hit || null);
    });
    return () => {
      alive = false;
    };
  }, [projectId, episodeId, outputFileName, status]);

  if (!out || !projectId) return null;
  const base = `${CORE_URL}/projects/${encodeURIComponent(projectId)}/`;
  const videoUrl = base + String(out.files.video);
  const thumbUrl = out.files.thumb ? base + String(out.files.thumb) : undefined;
  const dur = Number(out.duration_s) || 0;
  const ev = out.episodeVerify as { pass1to5?: boolean; verdict?: string } | undefined;
  const caption = [
    out.engine ? `engine: ${out.engine}` : null,
    ev?.verdict || "chưa chạy verify_episode",
  ]
    .filter(Boolean)
    .join(" · ");
  const open = () =>
    usePreview.getState().open({
      kind: "video",
      title: out.title || episodeId,
      index: 0,
      items: [
        {
          url: videoUrl,
          name: String(out.files.video),
          poster: thumbUrl,
          caption,
        },
      ],
    });
  return (
    <div className="space-y-1" data-testid="episode-output">
      <button
        type="button"
        onClick={open}
        title="Xem và tua thành phẩm ngay trong Studio"
        className="nodrag group relative block w-full overflow-hidden rounded-ctl bg-black"
      >
        {thumbUrl && (
          // eslint-disable-next-line @next/next/no-img-element
          <img
            decoding="async"
            loading="lazy"
            src={thumbUrl}
            alt=""
            className="aspect-video w-full object-cover opacity-90 group-hover:opacity-100"
          />
        )}
        <span className="absolute inset-0 flex items-center justify-center">
          <span className="flex items-center gap-1 rounded-full bg-black/70 px-3 py-1 text-caption font-semibold text-white">
            <Film className="h-3 w-3" /> Xem & tua
          </span>
        </span>
      </button>
      <div className="flex flex-wrap gap-1 text-caption text-muted-foreground tnum">
        <span>{String(out.files.video).split("/").pop()}</span>
        {dur > 0 && (
          <span>
            · {Math.floor(dur / 60)}:{String(Math.round(dur % 60)).padStart(2, "0")}
          </span>
        )}
        {ev && (
          <span className={ev.pass1to5 ? "text-semantic-success" : "text-semantic-warning"}>
            · {ev.pass1to5 ? "verify 1–5 ĐẠT" : "verify CHƯA ĐẠT"}
          </span>
        )}
      </div>
    </div>
  );
}
