"use client";

import React, { useState } from "react";
import { Play, Music } from "@/components/ui/icons";
import { AiLoading } from "@/components/ui/AiLoading";
import { fileName, previewUrl, runTool, uploadFile } from "../api";
import { ToolboxDropzone } from "../shared/ToolboxDropzone";
import { ToolboxErrorLine } from "../shared/ToolboxErrorLine";
import { ToolboxSaveBar } from "../shared/ToolboxSaveBar";
import type { RunOut, Targets } from "../types";

export function SceneWorkspace({ targets, onDone }: { targets: Targets | null; onDone: () => void }) {
  const [src, setSrc] = useState<{ path: string; name: string } | null>(null);
  const [busy, setBusy] = useState<"upload" | "run" | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [aspect, setAspect] = useState<"keep" | "9:16">("keep");
  const [clips, setClips] = useState(true);
  const [outs, setOuts] = useState<RunOut[] | null>(null);
  const [scenes, setScenes] = useState<{ index: number; start: number; end: number; duration: number }[]>([]);

  const pick = async (f: File) => {
    setError(null);
    setOuts(null);
    setScenes([]);
    setBusy("upload");
    try {
      setSrc(await uploadFile(f));
    } catch (e: any) {
      setError(e.message);
    } finally {
      setBusy(null);
    }
  };
  const run = async () => {
    if (!src) return;
    setError(null);
    setBusy("run");
    try {
      const r = await runTool("scene-cut", { aspect, export_clips: clips }, [src.path]);
      setOuts(r.outputs);
      const m = r.outputs.find((o) => o.kind === "scenes_json");
      if (m) setScenes((await fetch(previewUrl(m.path)).then((x) => x.json())).scenes || []);
      onDone();
    } catch (e: any) {
      setError(e.message);
    } finally {
      setBusy(null);
    }
  };
  const clipFiles = (outs || []).filter((o) => o.kind === "clip");
  const sel = "h-7 rounded-md border border-input bg-card px-2 text-caption";

  return (
    <div className="space-y-3">
      <ToolboxDropzone
        accept="video/*,.mp4,.mov,.mkv,.webm"
        icon={<Music />}
        title={src ? `Video: ${src.name} — bấm để đổi` : "Nhập video cần tách cảnh"}
        hint="MP4, MOV, MKV, WEBM (tối đa 200MB)"
        busy={busy !== null}
        onFile={pick}
      />
      {error && <ToolboxErrorLine text={error} onClose={() => setError(null)} />}
      {src && (
        <div className="flex flex-wrap items-center gap-2">
          <select aria-label="Khung hình" value={aspect} onChange={(e) => setAspect(e.target.value as any)} className={sel}>
            <option value="keep">Giữ nguyên khung</option>
            <option value="9:16">Cắt dọc 9:16 (Shorts)</option>
          </select>
          <label className="flex items-center gap-2 text-caption text-muted-foreground">
            <input type="checkbox" checked={clips} onChange={(e) => setClips(e.target.checked)} />
            Xuất clip từng cảnh
          </label>
          <button
            type="button"
            onClick={run}
            disabled={busy !== null}
            className="flex h-7 items-center gap-2 rounded-md bg-brand px-3 text-footnote font-medium text-white transition-all hover:opacity-90 active:scale-95 disabled:cursor-not-allowed disabled:opacity-40"
          >
            <Play className="h-3.5 w-3.5 fill-current" />
            {busy === "run" ? "Đang tách…" : "Tách cảnh"}
          </button>
        </div>
      )}
      {busy === "run" && <AiLoading label="Đang tìm điểm cắt và xuất clip…" lines={2} />}
      {scenes.length > 0 && (
        <div className="space-y-2">
          <p className="text-caption font-medium">{scenes.length} cảnh</p>
          <ul className="space-y-1 text-caption text-muted-foreground">
            {scenes.map((s) => (
              <li key={s.index} className="flex justify-between rounded bg-muted/40 px-2 py-1">
                <span>Cảnh {s.index}</span>
                <span className="tabular-nums">
                  {s.start.toFixed(1)}s → {s.end.toFixed(1)}s ({s.duration.toFixed(1)}s)
                </span>
              </li>
            ))}
          </ul>
          {clipFiles[0] && <video controls src={previewUrl(clipFiles[0].path)} className="max-h-48 w-full rounded-lg bg-black" />}
        </div>
      )}
      {outs && (
        <ToolboxSaveBar
          targets={targets}
          files={outs.map((o) => ({ label: o.kind === "clip" ? fileName(o.path) : "Danh sách cảnh (.json)", path: o.path }))}
        />
      )}
    </div>
  );
}
