"use client";

import React, { useRef, useState, useEffect } from "react";
import type { Character } from "@/lib/repositories";
import { CORE_URL } from "@/lib/repositories/core";
import { useHubScope } from "@/lib/hub";
import { SpeakTextBox } from "./SpeakTextBox";
import { MorphIcon, Volume2Node, VolumeXNode, RefreshCw, Zap } from "@/components/ui/icons";
import { cn } from "@/lib/utils";

export const CanvasChip = ({
  children,
  title,
}: {
  children: React.ReactNode;
  title?: string;
}) => (
  <span
    title={title}
    className="rounded-xs border border-border/80 bg-muted/40 px-2 py-1 text-caption font-medium text-muted-foreground tnum"
  >
    {children}
  </span>
);

export function VoiceChip({ voiceId, sampleUrl }: { voiceId?: string; sampleUrl?: string }) {
  const ref = useRef<HTMLAudioElement | null>(null);
  const [playing, setPlaying] = useState(false);
  useEffect(() => () => { ref.current?.pause(); }, []);
  const url = sampleUrl ? (/^https?:/.test(sampleUrl) ? sampleUrl : `${CORE_URL}${sampleUrl}`) : "";
  if (!url) return <CanvasChip title="Giọng">{voiceId || "—"}</CanvasChip>;
  return (
    <button
      type="button"
      title={playing ? "Dừng nghe thử" : "Bấm để nghe thử giọng"}
      aria-label="Nghe thử giọng"
      onClick={(e) => {
        e.stopPropagation();
        if (playing) {
          ref.current?.pause();
          setPlaying(false);
          return;
        }
        const a = ref.current ?? new Audio(url);
        ref.current = a;
        a.onended = () => setPlaying(false);
        a.onerror = () => setPlaying(false);
        a.currentTime = 0;
        a.play().then(() => setPlaying(true)).catch(() => setPlaying(false));
      }}
      className="nodrag inline-flex h-5 items-center gap-1 rounded-ctl border border-border/70 bg-card px-2 text-caption font-medium text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
    >
      <MorphIcon icon={playing ? Volume2Node : VolumeXNode} spring="snappy" size={10} />
      <span>{voiceId || "—"}</span>
    </button>
  );
}

export interface CharacterHeroProps {
  c: Character;
  nodeId?: string;
  compact?: boolean;
  detailed?: boolean;
  onPick?: () => void;
  /** Chức năng "Nói" của node Nhân vật (chỉ có khi Core khai báo chức năng speak) */
  speak?: {
    enabled: boolean;
    hasLines: boolean;
    onToggle: () => void;
    text: string;
    onTextChange: (t: string) => void;
  };
}

/** Thẻ nhân vật chuẩn studio: Tên & nút đổi nhân vật overlay trên ảnh + Gallery + Giọng đọc bên dưới */
export function CharacterHero({
  c,
  nodeId,
  compact,
  detailed,
  onPick,
  speak,
}: CharacterHeroProps) {
  const gallery = c.referenceImages.slice(1, 5);

  return (
    <div className="space-y-2 pt-1">
      {/* 1. KHU VỰC ẢNH CHÍNH (Tên & icon đổi nhân vật nằm overlay trên ảnh) */}
      <div className="group/hero-img relative overflow-hidden rounded-ctl border border-border bg-muted/20">
        {c.referenceImages[0] ? (
          // eslint-disable-next-line @next/next/no-img-element
          <img
            decoding="async"
            loading="lazy"
            src={c.referenceImages[0]}
            alt={c.name}
            className="max-h-52 w-full object-cover object-top transition-transform duration-300 group-hover/hero-img:scale-[1.02]"
          />
        ) : (
          <div className="flex h-36 w-full items-center justify-center bg-accent text-footnote font-semibold text-muted-foreground">
            {c.name}
          </div>
        )}

        {/* Nút đổi nhân vật: Ẩn mặc định, hover vào ảnh sẽ hiện */}
        {onPick && (
          <button
            type="button"
            title="Đổi nhân vật khác"
            aria-label="Đổi nhân vật"
            onClick={(e) => {
              e.stopPropagation();
              onPick();
            }}
            className="nodrag absolute top-2 right-2 flex h-7 w-7 items-center justify-center rounded-ctl bg-black/60 text-white backdrop-blur-xs opacity-0 group-hover/hero-img:opacity-100 hover:bg-black/80 transition-all duration-200 shadow-sm"
          >
            <RefreshCw className="h-3.5 w-3.5" />
          </button>
        )}

        {/* Text tên nhân vật & thông số chiều cao: Nằm trực tiếp trên dải đáy của ảnh */}
        <div className="absolute inset-x-0 bottom-0 bg-gradient-to-t from-black/80 via-black/40 to-transparent p-2 pt-6 flex items-end justify-between gap-2">
          <button
            type="button"
            title="Mở hồ sơ nhân vật"
            aria-label="Mở hồ sơ nhân vật"
            onClick={(e) => {
              e.stopPropagation();
              useHubScope.getState().openTab("characters", null, c.id);
            }}
            className="nodrag truncate text-footnote font-bold text-white text-left hover:underline underline-offset-2 drop-shadow-sm"
          >
            {c.name}
          </button>

          {c.heightCm && (
            <span
              title="Chiều cao nhân vật"
              className="shrink-0 text-caption font-semibold text-white/90 bg-white/20 backdrop-blur-xs px-2 py-1 rounded-xs font-mono"
            >
              {c.heightCm} cm
            </span>
          )}
        </div>
      </div>

      {/* 2. Dải góc chụp gallery */}
      {gallery.length > 0 && (
        <div className="grid grid-cols-4 gap-2">
          {gallery.map((u, idx) => (
            <div
              key={`${u}-${idx}`}
              className="aspect-square overflow-hidden rounded-ctl border border-border bg-muted/20"
            >
              {/* eslint-disable-next-line @next/next/no-img-element */}
              <img
                decoding="async"
                src={u}
                alt=""
                loading="lazy"
                className="h-full w-full object-cover object-top"
              />
            </div>
          ))}
        </div>
      )}

      {/* 3. KHỐI GIỌNG ĐỌC & CHỨC NĂNG THOẠI (NẰM DƯỚI ẢNH) - Thu gọn tối đa vì là tính năng tuỳ chọn */}
      <div className="rounded-ctl border border-border/60 bg-muted/20 p-1 space-y-1">
        <div className="flex items-center justify-between gap-1">
          <div className="flex items-center gap-1 min-w-0">
            <span className="text-caption font-medium text-muted-foreground shrink-0">Giọng:</span>
            <VoiceChip voiceId={c.voiceId} sampleUrl={c.sampleAudioUrl} />
          </div>

          {speak && (
            <div className="flex items-center gap-1 shrink-0">
              {speak.hasLines ? (
                <span
                  title="Nhân vật tự động nói câu của mình theo kịch bản từ cổng lines"
                  className="inline-flex h-5 items-center gap-1 rounded-ctl border border-brand/40 bg-brand/10 px-2 text-caption font-medium text-brand"
                >
                  <Zap className="h-2.5 w-2.5" />
                  <span>Theo kịch bản</span>
                </span>
              ) : (
                <button
                  type="button"
                  aria-pressed={speak.enabled}
                  title={
                    speak.enabled
                      ? "Đang bật thoại: nhân vật sẽ nói câu thoại nhập bên dưới"
                      : "Bật thoại: nhập lời thoại cho nhân vật nói"
                  }
                  onClick={(e) => {
                    e.stopPropagation();
                    speak.onToggle();
                  }}
                  className={cn(
                    "nodrag inline-flex h-5 items-center gap-1 rounded-ctl border px-2 text-caption font-medium transition-colors cursor-pointer",
                    speak.enabled
                      ? "border-brand/40 bg-brand/10 text-brand font-semibold"
                      : "border-border/70 bg-card text-muted-foreground hover:bg-accent hover:text-foreground"
                  )}
                >
                  <MorphIcon icon={speak.enabled ? Volume2Node : VolumeXNode} spring="snappy" size={10} />
                  <span>{speak.enabled ? "Đang bật thoại" : "Bật thoại"}</span>
                </button>
              )}
            </div>
          )}
        </div>

        {/* Ô nhập thoại khi không nối cổng lines mà người dùng bật thoại */}
        {speak?.enabled && !speak.hasLines && (
          <div className="pt-1">
            <SpeakTextBox
              text={speak.text}
              onChange={speak.onTextChange}
              voiceId={c.voiceId}
              nodeId={nodeId}
            />
          </div>
        )}
      </div>
    </div>
  );
}
