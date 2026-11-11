"use client";

import React from "react";
import type { Character } from "@/lib/repositories";

export function CharactersTab({
  characters,
  onSelect,
}: {
  characters: Character[];
  onSelect: (val: string) => void;
}) {
  return (
    <div className="max-h-[260px] overflow-y-auto space-y-2 pr-1 scroll-quiet">
      {characters.length === 0 ? (
        <p className="py-4 text-center text-footnote text-muted-foreground">Chưa có nhân vật nào trong dự án</p>
      ) : (
        characters.map((c) => (
          <div key={c.id} className="rounded-card border border-border/60 p-2 space-y-2 bg-card/60">
            <div className="flex items-center gap-2">
              {c.referenceImages?.[0] ? (
                // eslint-disable-next-line @next/next/no-img-element
                <img
                  src={c.referenceImages[0]}
                  alt=""
                  loading="lazy"
                  decoding="async"
                  className="h-6 w-6 rounded-full object-cover object-top"
                />
              ) : (
                <span className="flex h-6 w-6 items-center justify-center rounded-full bg-muted text-caption font-semibold text-muted-foreground">
                  {c.name.charAt(0)}
                </span>
              )}
              <span className="font-semibold text-footnote text-foreground truncate">{c.name}</span>
            </div>

            <div className="flex flex-wrap items-center gap-1">
              {c.promptLock && (
                <button
                  type="button"
                  onClick={() => onSelect(`{${c.name}.promptLock}`)}
                  className="inline-flex items-center gap-1 rounded-xs bg-accent px-2 py-1 text-caption font-mono font-medium text-foreground hover:bg-brand/15 hover:text-brand transition-colors"
                >
                  {`{${c.name}.promptLock}`}
                </button>
              )}
              <button
                type="button"
                onClick={() => onSelect(`{${c.name}.name}`)}
                className="inline-flex items-center gap-1 rounded-xs bg-accent px-2 py-1 text-caption font-mono font-medium text-foreground hover:bg-brand/15 hover:text-brand transition-colors"
              >
                {`{${c.name}.name}`}
              </button>
              {c.voiceId && (
                <button
                  type="button"
                  onClick={() => onSelect(`{${c.name}.voiceId}`)}
                  className="inline-flex items-center gap-1 rounded-xs bg-accent px-2 py-1 text-caption font-mono font-medium text-foreground hover:bg-brand/15 hover:text-brand transition-colors"
                >
                  {`{${c.name}.voiceId}`}
                </button>
              )}
            </div>
          </div>
        ))
      )}
    </div>
  );
}
