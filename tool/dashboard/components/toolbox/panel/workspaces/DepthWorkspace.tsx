"use client";

import React, { useState } from "react";
import { Play, Image as ImageIcon } from "@/components/ui/icons";
import { AiLoading } from "@/components/ui/AiLoading";
import { cn } from "@/lib/utils";
import { previewUrl, runTool, uploadFile } from "../api";
import { checker } from "../constants";
import { ToolboxDropzone } from "../shared/ToolboxDropzone";
import { ToolboxErrorLine } from "../shared/ToolboxErrorLine";
import { ToolboxSaveBar } from "../shared/ToolboxSaveBar";
import type { RunOut, Targets } from "../types";

export function DepthWorkspace({ targets, onDone }: { targets: Targets | null; onDone: () => void }) {
  const [src, setSrc] = useState<{ path: string; name: string } | null>(null);
  const [busy, setBusy] = useState<"upload" | "run" | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [outs, setOuts] = useState<RunOut[] | null>(null);
  const [elapsed, setElapsed] = useState<number | null>(null);

  const pick = async (f: File) => {
    setError(null);
    setOuts(null);
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
      const r = await runTool("depth-layers", {}, [src.path]);
      setOuts(r.outputs);
      setElapsed(r.elapsed_s ?? null);
      onDone();
    } catch (e: any) {
      setError(e.message);
    } finally {
      setBusy(null);
    }
  };
  const layers = [
    { kind: "layer_bg", label: "Hậu cảnh (0.2x)" },
    { kind: "layer_mid", label: "Trung cảnh (1.0x)" },
    { kind: "layer_fg", label: "Tiền cảnh (1.8x)" },
  ];
  const label: Record<string, string> = { layer_bg: "Hậu cảnh", layer_mid: "Trung cảnh", layer_fg: "Tiền cảnh", depth: "Bản đồ độ sâu", layers_json: "Số đo (.json)" };

  return (
    <div className="space-y-3">
      <ToolboxDropzone
        accept="image/png,image/jpeg,image/webp"
        icon={<ImageIcon />}
        title={src ? `Ảnh: ${src.name} — bấm để đổi` : "Nhập ảnh cần tách lớp"}
        hint="Tách 3 lớp tiền / trung / hậu cảnh để làm parallax (lần đầu tải model ~100MB)"
        busy={busy !== null}
        onFile={pick}
      />
      {error && <ToolboxErrorLine text={error} onClose={() => setError(null)} />}
      {src && (
        <button
          type="button"
          onClick={run}
          disabled={busy !== null}
          data-tip="Chạy Depth Anything V2 Small trên CPU"
          className="flex h-7 items-center gap-2 rounded-md bg-brand px-3 text-footnote font-medium text-white transition-all hover:opacity-90 active:scale-95 disabled:cursor-not-allowed disabled:opacity-40"
        >
          <Play className="h-3.5 w-3.5 fill-current" />
          {busy === "run" ? "Đang tách lớp…" : "Tách 3 lớp"}
        </button>
      )}
      {busy === "run" && <AiLoading label="Đang ước lượng độ sâu…" lines={2} />}
      {outs && (
        <>
          <div className="grid grid-cols-3 gap-2">
            {layers.map((l) => {
              const o = outs.find((x) => x.kind === l.kind);
              return (
                <figure key={l.kind} className="space-y-1">
                  <figcaption className="text-caption text-muted-foreground">{l.label}</figcaption>
                  <div className={cn(checker, "p-1")}>{o && <img decoding="async" loading="lazy" src={previewUrl(o.path)} alt={l.label} className="max-h-28 object-contain" />}</div>
                </figure>
              );
            })}
          </div>
          {elapsed && <p className="text-caption text-muted-foreground">Xong sau {elapsed.toFixed(1)}s</p>}
          <ToolboxSaveBar targets={targets} files={outs.map((o) => ({ label: label[o.kind] || o.kind, path: o.path }))} />
        </>
      )}
    </div>
  );
}
