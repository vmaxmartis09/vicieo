"use client";

import React, { useState, useEffect, useRef } from "react";
import { useStudioStore } from "@/store";
import { CORE_URL } from "@/lib/repositories/core";
import { MorphIcon, Volume2Node, X } from "@/components/ui/icons";
import { DynamicValueSelector } from "./DynamicValueSelector";

export interface SpeakTextBoxProps {
  text: string;
  onChange: (t: string) => void;
  voiceId?: string;
  nodeId?: string;
}

export function SpeakTextBox({ text, onChange, voiceId, nodeId }: SpeakTextBoxProps) {
  const projectId = useStudioStore((st: any) => st.activeProjectId) as string;
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState("");
  const [isFocused, setIsFocused] = useState(false);
  const taRef = useRef<HTMLTextAreaElement>(null);
  const ref = useRef<HTMLAudioElement | null>(null);

  useEffect(() => () => { ref.current?.pause(); }, []);

  useEffect(() => {
    const el = taRef.current;
    if (!el) return;
    const prev = el.offsetHeight;
    el.style.height = "auto";
    const target = isFocused ? Math.min(Math.max(el.scrollHeight, 56), 160) : 28;
    el.style.height = `${prev}px`;
    void el.offsetHeight;
    el.style.height = `${target}px`;
  }, [text, isFocused]);

  // Trình duyệt chặn phát âm thanh nếu chờ API quá lâu sau cú bấm: "mở khoá" bằng đoạn lặng ngay lúc bấm.
  const SILENT = "data:audio/wav;base64,UklGRiQAAABXQVZFZm10IBAAAAABAAEAESsAACJWAAACABAAZGF0YQAAAAA=";
  const say = async () => {
    const t = text.trim();
    if (!t || !voiceId) return;
    setBusy(true);
    setErr("");
    const a = new Audio(SILENT);
    ref.current?.pause();
    ref.current = a;
    try {
      await a.play();
    } catch {
      /* bỏ qua */
    }
    try {
      const r = await fetch(`${CORE_URL}/projects/${projectId}/voice/say`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ text: t, voiceId }),
      });
      const j = await r.json();
      if (!r.ok) throw new Error(j?.detail || `Lỗi ${r.status}`);
      a.src = `${CORE_URL}${j.audioUrl}`;
      await a.play();
    } catch (e: any) {
      setErr(String(e?.message || e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="nodrag nowheel space-y-1">
      <div className="relative group/speak">
        <textarea
          ref={taRef}
          value={text}
          onChange={(e) => onChange(e.target.value)}
          onClick={(e) => e.stopPropagation()}
          onFocus={() => setIsFocused(true)}
          onBlur={() => setIsFocused(false)}
          onKeyDown={(e) => {
            e.stopPropagation();
            if (e.key === "Escape") {
              e.currentTarget.blur();
              setIsFocused(false);
            }
          }}
          maxLength={400}
          spellCheck={false}
          autoCorrect="off"
          autoCapitalize="off"
          placeholder="Lời nhân vật nói (tuỳ chọn)..."
          className="w-full resize-none rounded-ctl border border-border/80 bg-card px-2 py-1 pr-6 text-caption leading-tight outline-none focus:border-foreground transition-[height] duration-150"
        />
        <div className="absolute right-1 top-1 flex items-center gap-1 z-10">
          <DynamicValueSelector
            nodeId={nodeId}
            portName="lines"
            currentValue={text}
            onSelectValue={(val) => {
              onChange(val);
              setTimeout(() => {
                taRef.current?.focus();
                setIsFocused(true);
              }, 50);
            }}
          />
        </div>
      </div>
      <div className="flex items-center justify-between pt-1">
        <button
          type="button"
          disabled={busy || !text.trim() || !voiceId}
          onClick={(e) => { e.stopPropagation(); void say(); }}
          className="inline-flex h-5 items-center gap-1 rounded-ctl border border-border/70 bg-card px-2 text-caption font-medium text-muted-foreground transition-colors hover:bg-accent hover:text-foreground disabled:opacity-40 cursor-pointer"
        >
          <MorphIcon icon={Volume2Node} spring="snappy" size={10} />
          {busy ? "Đang đọc…" : "Nghe thử"}
        </button>

        <div className="flex items-center gap-1">
          {Boolean(text) && (
            <button
              type="button"
              onMouseDown={(e) => e.preventDefault()}
              onClick={() => {
                onChange("");
                taRef.current?.focus();
              }}
              title="Xoá nội dung thoại"
              aria-label="Xoá nội dung thoại"
              className="flex h-4 w-4 items-center justify-center rounded-xs text-muted-foreground hover:bg-destructive/10 hover:text-destructive cursor-pointer transition-colors"
            >
              <X className="h-2.5 w-2.5" />
            </button>
          )}
          <span className="text-caption text-muted-foreground/60 font-mono tnum">{text.length}/400</span>
        </div>
      </div>
      {err && <p className="text-caption text-semantic-danger">{err}</p>}
    </div>
  );
}
