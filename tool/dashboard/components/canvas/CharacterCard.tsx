"use client";

import React from "react";
import type { Character } from "@/lib/repositories";

/** Thẻ nhân vật trong node: ảnh chính, thư viện ảnh, thông số, tính cách, câu cửa miệng, nghe thử giọng. */
export const CharacterCard = React.memo(function CharacterCard({ character: c }: { character: Character }) {
  const gallery = c.referenceImages.slice(1, 6);
  return (
    <div className="space-y-2 rounded-card border border-border bg-muted/20 p-2">
      <div className="flex gap-3">
        {c.referenceImages[0] ? (
          // eslint-disable-next-line @next/next/no-img-element
          <img decoding="async" loading="lazy"
            src={c.referenceImages[0]}
            alt={c.name}
            className="h-16 w-16 shrink-0 rounded-ctl border border-border object-cover object-center"
          />
        ) : (
          <div className="flex h-16 w-16 shrink-0 items-center justify-center rounded-ctl border border-border bg-accent text-headline font-semibold text-muted-foreground">
            {c.name.replace("Bạn ", "").charAt(0)}
          </div>
        )}
        <div className="min-w-0 flex-1 text-body">
          <p className="truncate text-footnote font-semibold text-foreground">{c.name}</p>
          <p className="text-muted-foreground">
            {c.role === "main" ? "Nhân vật chính" : "Nhân vật phụ"}
          </p>
          <dl className="mt-1 grid grid-cols-[auto_1fr] gap-x-2 gap-y-1">
            <dt className="text-muted-foreground">Cao</dt>
            <dd className="tnum font-medium text-foreground">
              {c.heightCm} cm · {c.scaleRatio}x
            </dd>
            <dt className="text-muted-foreground">Giọng</dt>
            <dd className="truncate font-medium text-foreground">
              {c.voiceId || "—"} · {c.defaultTempo ?? 1}x
            </dd>
          </dl>
        </div>
      </div>

      {gallery.length > 0 && (
        <div className="flex gap-1 overflow-hidden">
          {gallery.map((u) => (
            // eslint-disable-next-line @next/next/no-img-element
            <img decoding="async"
              key={u}
              src={u}
              alt=""
              loading="lazy"
              className="h-8 w-8 rounded-ctl border border-border object-cover object-center"
            />
          ))}
          {c.referenceImages.length > 6 && (
            <span className="flex h-8 w-8 items-center justify-center rounded-ctl border border-border text-caption text-muted-foreground tnum">
              +{c.referenceImages.length - 6}
            </span>
          )}
        </div>
      )}

      {c.memory.personality && (
        <p className="line-clamp-2 text-body text-muted-foreground">
          <span className="font-medium text-foreground">Tính cách: </span>
          {c.memory.personality}
        </p>
      )}
      {c.memory.catchphrases.length > 0 && (
        <p className="line-clamp-1 text-body italic text-muted-foreground">
          “{c.memory.catchphrases[0]}”
        </p>
      )}
      {(c.description || c.promptLock) && (
        <p className="line-clamp-2 text-body text-muted-foreground">
          {c.description || c.promptLock}
        </p>
      )}
      {c.sampleAudioUrl && (
        <audio
          controls
          preload="none"
          src={c.sampleAudioUrl}
          className="nodrag nowheel h-7 w-full"
        />
      )}
    </div>
  );
});
