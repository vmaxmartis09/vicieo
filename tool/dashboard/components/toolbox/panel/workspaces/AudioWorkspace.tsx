"use client";

import React, { useState } from "react";
import { Play, Music } from "@/components/ui/icons";
import { AiLoading } from "@/components/ui/AiLoading";
import { previewUrl, runTool, uploadFile } from "../api";
import { ToolboxDropzone } from "../shared/ToolboxDropzone";
import { ToolboxErrorLine } from "../shared/ToolboxErrorLine";
import { ToolboxSaveBar } from "../shared/ToolboxSaveBar";
import type { RunOut, Targets } from "../types";

export function AudioWorkspace({ targets, onDone }: { targets: Targets | null; onDone: () => void }) {
  const [src, setSrc] = useState<{ path: string; name: string } | null>(null);
  const [busy, setBusy] = useState<"upload" | "run" | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [lang, setLang] = useState("vi");
  const [outs, setOuts] = useState<RunOut[] | null>(null);
  const [text, setText] = useState<string>("");

  const pick = async (f: File) => {
    setError(null);
    setOuts(null);
    setText("");
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
      const r = await runTool("stt-align", { model_size: "small", language: lang }, [src.path]);
      setOuts(r.outputs);
      const t = r.outputs.find((o) => o.kind === "word_timings");
      if (t) {
        const j = await fetch(previewUrl(t.path)).then((x) => x.json());
        setText(j.text || (j.words || []).map((w: any) => w.word || w.text).join(" "));
      }
      onDone();
    } catch (e: any) {
      setError(e.message);
    } finally {
      setBusy(null);
    }
  };

  const files = [
    ...(src ? [{ label: "File âm thanh gốc", path: src.path }] : []),
    ...(outs || []).map((o) => ({ label: o.kind === "subtitles_srt" ? "Phụ đề (.srt)" : "Canh từng từ (.json)", path: o.path })),
  ];

  return (
    <div className="space-y-3">
      <ToolboxDropzone
        accept="audio/*,.mp3,.wav,.m4a,.ogg,.flac"
        icon={<Music />}
        title={src ? `Âm thanh: ${src.name} — bấm để đổi` : "Nhập mẫu giọng / file âm thanh"}
        hint="Bấm, kéo-thả hoặc dán (MP3, WAV, M4A, OGG, FLAC)"
        busy={busy !== null}
        onFile={pick}
      />
      {error && <ToolboxErrorLine text={error} onClose={() => setError(null)} />}
      {src && <audio controls src={previewUrl(src.path)} className="h-9 w-full" />}
      <div className="flex flex-wrap items-center gap-2">
        <select
          aria-label="Ngôn ngữ"
          value={lang}
          onChange={(e) => setLang(e.target.value)}
          className="h-7 rounded-md border border-input bg-card px-2 text-caption"
        >
          <option value="vi">Tiếng Việt</option>
          <option value="en">English</option>
        </select>
        <button
          type="button"
          onClick={run}
          disabled={!src || busy !== null}
          data-tip={src ? "Phiên âm và canh từng từ (CPU, khoảng 10–30 giây)" : "Nhập âm thanh trước"}
          className="flex h-7 items-center gap-2 rounded-md bg-brand px-3 text-footnote font-medium text-white transition-all hover:opacity-90 active:scale-95 disabled:cursor-not-allowed disabled:opacity-40"
        >
          <Play className="h-3.5 w-3.5 fill-current" />
          {busy === "run" ? "Đang phiên âm…" : "Phiên âm + canh từ"}
        </button>
      </div>
      {busy === "run" && <AiLoading label="Đang nghe và canh từng từ…" lines={2} />}
      {text && <p className="rounded-lg bg-muted/50 p-3 text-footnote leading-relaxed">“{text}”</p>}
      {files.length > 0 && <ToolboxSaveBar targets={targets} files={files} />}
    </div>
  );
}
