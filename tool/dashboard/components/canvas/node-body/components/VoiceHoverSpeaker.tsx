"use client";

import React, { useState, useEffect, useRef } from "react";
import { CORE_URL } from "@/lib/repositories/core";
import { useStudioStore } from "@/store";
import { Volume2 } from "@/components/ui/icons";
import { cn } from "@/lib/utils";

/** Chế độ xem card giọng đọc: rê chuột vào thì đọc — ưu tiên văn bản nhận từ node trước, nếu không thì văn bản của chính card. */
export function VoiceHoverSpeaker({
  nodeId,
  ownText,
  voiceId,
  audioUrl,
}: {
  nodeId: string;
  ownText: string;
  voiceId?: string;
  audioUrl?: string;
}) {
  const nodes = useStudioStore((s: any) => s.nodes) as any[];
  const edges = useStudioStore((s: any) => s.edges) as any[];
  const [speaking, setSpeaking] = useState(false);
  const audioRef = useRef<HTMLAudioElement | null>(null);

  const edge = edges.find((e) => e.target === nodeId && e.targetHandle === "text");
  const src = edge ? nodes.find((n) => n.id === edge.source) : undefined;
  const sp = src?.data?.params || {};
  const inputText: string =
    (src && (sp.content || sp.text || sp.prompt || src.data?.summaryText)) || "";
  const text = (inputText || ownText || "").trim();

  const stop = () => {
    audioRef.current?.pause();
    if (typeof window !== "undefined" && "speechSynthesis" in window) {
      window.speechSynthesis.cancel();
    }
    setSpeaking(false);
  };

  const start = () => {
    // Không có file kết quả và không có lời thoại: phát mẫu giọng mặc định
    const sampleUrl =
      !audioUrl && !text ? `${CORE_URL}/voices/${voiceId || "soc-nho"}/sample` : undefined;
    const playUrl = audioUrl || sampleUrl;
    if (playUrl) {
      audioRef.current =
        audioRef.current?.src === playUrl ? audioRef.current : new Audio(playUrl);
      audioRef.current.onended = () => setSpeaking(false);
      audioRef.current.onerror = () => setSpeaking(false);
      audioRef.current.currentTime = 0;
      audioRef.current
        .play()
        .then(() => setSpeaking(true))
        .catch(() => setSpeaking(false));
      return;
    }
    if (!text || typeof window === "undefined" || !("speechSynthesis" in window)) return;
    window.speechSynthesis.cancel();
    const u = new SpeechSynthesisUtterance(text);
    u.lang = "vi-VN";
    u.onend = () => setSpeaking(false);
    u.onerror = () => setSpeaking(false);
    window.speechSynthesis.speak(u);
    setSpeaking(true);
  };

  useEffect(() => stop, []); // dừng khi card bị gỡ

  return (
    <div
      onMouseEnter={start}
      onMouseLeave={stop}
      title="Rê chuột để nghe"
      className={cn(
        "flex items-start gap-2 rounded-ctl border border-dashed p-2 text-footnote transition-colors",
        speaking ? "border-brand bg-brand/5" : "border-border"
      )}
    >
      <Volume2
        className={cn(
          "mt-1 h-3.5 w-3.5 shrink-0",
          speaking ? "animate-pulse text-brand" : "text-muted-foreground"
        )}
      />
      <div className="min-w-0 flex-1">
        <p
          className={cn(
            "line-clamp-3 break-words",
            text ? "text-foreground" : "text-muted-foreground"
          )}
        >
          {text || "Chưa có lời thoại — rê chuột để nghe giọng mẫu"}
        </p>
        {voiceId && (
          <span className="mt-1 block truncate text-caption text-muted-foreground">
            {voiceId}
          </span>
        )}
      </div>
    </div>
  );
}
