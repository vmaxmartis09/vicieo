"use client";

import React from "react";
import type { TemplateVariableItem } from "../types";

export function VariablesTab({
  variables,
  onSelect,
}: {
  variables: TemplateVariableItem[];
  onSelect: (val: string) => void;
}) {
  return (
    <div className="max-h-[260px] overflow-y-auto space-y-1 pr-1 scroll-quiet">
      <p className="px-1 text-caption text-muted-foreground mb-1">
        Bấm để chèn biến động vào văn bản:
      </p>
      {variables.map((v) => (
        <button
          key={v.tag}
          type="button"
          onClick={() => onSelect(v.tag)}
          className="flex w-full items-center justify-between rounded-card border border-border/60 p-2 text-left transition-colors hover:border-foreground/30 hover:bg-accent"
        >
          <div className="min-w-0 pr-2">
            <div className="font-medium text-footnote text-foreground truncate">{v.label}</div>
            <div className="text-caption text-muted-foreground truncate">{v.desc}</div>
          </div>
          <code className="rounded-xs bg-muted px-2 py-1 text-caption font-mono text-brand shrink-0">
            {v.tag}
          </code>
        </button>
      ))}
    </div>
  );
}
