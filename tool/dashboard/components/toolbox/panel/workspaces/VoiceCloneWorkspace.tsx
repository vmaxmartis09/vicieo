"use client";

import React, { useState } from "react";
import { Play, Mic, Check } from "@/components/ui/icons";
import { AiLoading } from "@/components/ui/AiLoading";
import { cn } from "@/lib/utils";
import { previewUrl, runTool, uploadFile } from "../api";
import { ToolboxDropzone } from "../shared/ToolboxDropzone";
import { ToolboxErrorLine } from "../shared/ToolboxErrorLine";

export function VoiceCloneWorkspace({ onDone }: { onDone: () => void }) {
  const [src, setSrc] = useState<{ path: string; name: string } | null>(null);
  const [busy, setBusy] = useState<"upload" | "run" | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [name, setName] = useState("");
  const [display, setDisplay] = useState("");
  const [consent, setConsent] = useState("");
  const [dialect, setDialect] = useState("south");
  const [result, setResult] = useState<{ preview?: string; voiceId: string; elapsed?: number } | null>(null);

  const slugOk = /^[a-z0-9][a-z0-9-]{1,40}$/.test(name);
  const ready = !!src && slugOk && consent.trim().length >= 10;

  const pick = async (f: File) => {
    setError(null);
    setResult(null);
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
    if (!src || !ready) return;
    setError(null);
    setBusy("run");
    try {
      const r = await runTool("voice-clone", { name, display_name: display || name, consent_note: consent.trim(), dialect }, [src.path]);
      setResult({ voiceId: name, preview: r.outputs.find((o) => o.kind === "voice_preview")?.path, elapsed: r.elapsed_s });
      onDone();
    } catch (e: any) {
      setError(e.message);
    } finally {
      setBusy(null);
    }
  };
  const inp = "h-7 w-full rounded-md border border-input bg-card px-2 text-footnote placeholder:text-muted-foreground";

  return (
    <div className="space-y-3">
      <ToolboxDropzone
        accept="audio/*,.mp3,.wav,.m4a,.ogg,.flac"
        icon={<Mic />}
        title={src ? `Mẫu giọng: ${src.name} — bấm để đổi` : "Nhập mẫu giọng cần clone"}
        hint="Mẫu sạch 3.5–6 giây, một người nói, không nhạc nền"
        busy={busy !== null}
        onFile={pick}
      />
      {src && <audio controls src={previewUrl(src.path)} className="h-9 w-full" />}
      <div className="grid grid-cols-2 gap-2">
        <label className="space-y-1 text-caption text-muted-foreground">
          Mã giọng (chữ thường, số, gạch ngang)
          <input value={name} onChange={(e) => setName(e.target.value.toLowerCase())} placeholder="de-bong" className={cn(inp, name && !slugOk && "border-semantic-danger")} />
        </label>
        <label className="space-y-1 text-caption text-muted-foreground">
          Tên hiển thị
          <input value={display} onChange={(e) => setDisplay(e.target.value)} placeholder="Dê Bông" className={inp} />
        </label>
      </div>
      <label className="block space-y-1 text-caption text-muted-foreground">
        Giọng vùng miền
        <select value={dialect} onChange={(e) => setDialect(e.target.value)} className={inp}>
          <option value="south">Miền Nam</option>
          <option value="north">Miền Bắc</option>
          <option value="central">Miền Trung</option>
        </select>
      </label>
      <label className="block space-y-1 text-caption text-muted-foreground">
        Đồng ý sử dụng (bắt buộc): mẫu giọng của ai, được phép dùng làm gì
        <textarea
          value={consent}
          onChange={(e) => setConsent(e.target.value)}
          rows={2}
          placeholder="VD: Giọng của tôi, dùng làm giọng nhân vật Dê Bông trong dự án Sóc Nhỏ TV"
          className="w-full rounded-md border border-input bg-card p-2 text-footnote placeholder:text-muted-foreground"
        />
      </label>
      {error && <ToolboxErrorLine text={error} onClose={() => setError(null)} />}
      <button
        type="button"
        onClick={run}
        disabled={!ready || busy !== null}
        data-tip={ready ? "Clone giọng bằng OmniVoice (vài phút trên CPU)" : "Cần mẫu giọng, mã giọng hợp lệ và nội dung đồng ý"}
        className="flex h-7 items-center gap-2 rounded-md bg-brand px-3 text-footnote font-medium text-white transition-all hover:opacity-90 active:scale-95 disabled:cursor-not-allowed disabled:opacity-40"
      >
        <Play className="h-3.5 w-3.5 fill-current" />
        {busy === "run" ? "Đang clone…" : "Clone giọng"}
      </button>
      {busy === "run" && <AiLoading label="OmniVoice đang học giọng… (có thể mất vài phút)" lines={2} />}
      {result && (
        <div className="space-y-2 rounded-lg border border-semantic-success/30 bg-semantic-success/5 p-3 text-caption">
          <p className="flex items-center gap-1 font-medium text-semantic-success">
            <Check className="h-3.5 w-3.5" />
            Đã thêm giọng “{result.voiceId}” vào kho{result.elapsed ? ` (${result.elapsed.toFixed(0)}s)` : ""}
          </p>
          {result.preview ? <audio controls src={previewUrl(result.preview)} className="h-7 w-full" /> : <p className="text-muted-foreground">Chưa đọc thử được câu mẫu.</p>}
        </div>
      )}
    </div>
  );
}
