"use client";

import React from "react";
import type { ExtendedModelItem } from "@/lib/data/extendedModelsCatalog";

export function ModelsTab({
  models,
  onSelect,
}: {
  models: ExtendedModelItem[];
  onSelect: (val: string) => void;
}) {
  return (
    <div className="max-h-[260px] overflow-y-auto space-y-1 pr-1 scroll-quiet">
      {models.map((m) => (
        <button
          key={m.key}
          type="button"
          onClick={() => onSelect(`{model.${m.key}}`)}
          className="flex w-full items-center justify-between rounded-card border border-border/60 p-2 text-left transition-colors hover:border-foreground/30 hover:bg-accent"
        >
          <div className="min-w-0 pr-2">
            <div className="font-semibold text-footnote text-foreground truncate">{m.name}</div>
            <div className="text-caption text-muted-foreground truncate">{m.brand} · {m.costStr}</div>
          </div>
          <code className="rounded-xs bg-muted px-2 py-1 text-caption font-mono text-brand shrink-0">
            {`{model.${m.key}}`}
          </code>
        </button>
      ))}
    </div>
  );
}
