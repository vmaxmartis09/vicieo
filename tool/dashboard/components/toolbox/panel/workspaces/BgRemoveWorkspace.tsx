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

export function BgRemoveWorkspace({ targets, onDone }: { targets: Targets | null; onDone: () => void }) {
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
      const r = await runTool("bg-remove", { model: "u2net" }, [src.path]);
      setOuts(r.outputs);
      setElapsed(r.elapsed_s ?? null);
      onDone();
    } catch (e: any) {
      setError(e.message);
    } finally {
      setBusy(null);
    }
  };
  const cutout = outs?.find((o) => o.kind === "cutout");
  const mask = outs?.find((o) => o.kind === "mask");

  return (
    <div className="space-y-3">
      <ToolboxDropzone
        accept="image/png,image/jpeg,image/webp"
        icon={<ImageIcon />}
        title={src ? `Ảnh: ${src.name} — bấm để đổi` : "Nhập ảnh cần tách nền"}
        hint="Bấm, kéo-thả hoặc dán ảnh (PNG, JPG, WEBP)"
        busy={busy !== null}
        onFile={pick}
      />
      {error && <ToolboxErrorLine text={error} onClose={() => setError(null)} />}
      {src && (
        <div className="grid grid-cols-2 gap-2">
          <figure className="space-y-1">
            <figcaption className="text-caption text-muted-foreground">Ảnh gốc</figcaption>
            <div className={checker}>
              <img decoding="async" loading="lazy" src={previewUrl(src.path)} alt="Ảnh gốc" className="max-h-44 object-contain" />
            </div>
          </figure>
          <figure className="space-y-1">
            <figcaption className="text-caption text-muted-foreground">Đã tách nền {elapsed ? `· ${elapsed.toFixed(1)}s` : ""}</figcaption>
            <div className={cn(checker, "min-h-[96px]")}>
              {busy === "run" ? (
                <AiLoading label="Đang tách nền…" lines={0} />
              ) : cutout ? (
                <img decoding="async" loading="lazy" src={previewUrl(cutout.path)} alt="Ảnh đã tách nền" className="max-h-44 object-contain" />
              ) : (
                <span className="text-caption text-muted-foreground">Chưa chạy</span>
              )}
            </div>
          </figure>
        </div>
      )}
      <div className="flex items-center gap-2">
        <button
          type="button"
          onClick={run}
          disabled={!src || busy !== null}
          data-tip={src ? "Chạy tách nền bằng rembg trên CPU (khoảng 20–30 giây)" : "Nhập ảnh trước"}
          className="flex h-7 items-center gap-2 rounded-md bg-brand px-3 text-footnote font-medium text-white transition-all hover:opacity-90 active:scale-95 disabled:cursor-not-allowed disabled:opacity-40"
        >
          <Play className="h-3.5 w-3.5 fill-current" />
          {busy === "run" ? "Đang chạy…" : "Tách nền"}
        </button>
        {busy === "upload" && <span className="text-caption text-muted-foreground">Đang tải ảnh lên…</span>}
      </div>
      {cutout && (
        <ToolboxSaveBar
          targets={targets}
          files={[
            { label: "Ảnh tách nền (PNG)", path: cutout.path },
            ...(mask ? [{ label: "Mặt nạ (mask)", path: mask.path }] : []),
          ]}
        />
      )}
    </div>
  );
}
