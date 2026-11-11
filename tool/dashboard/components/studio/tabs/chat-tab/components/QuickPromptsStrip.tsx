"use client";

import React from "react";

interface Props {
  quickPrompts: string[];
  disabled: boolean;
  onSelect: (prompt: string) => void;
}

export function QuickPromptsStrip({ quickPrompts, disabled, onSelect }: Props) {
  return (
    <div className="scroll-quiet flex items-center gap-1 overflow-x-auto border-b border-border bg-card-elevated px-3 py-2">
      {quickPrompts.map((qp) => (
        <button
          key={qp}
          type="button"
          title={qp}
          disabled={disabled}
          onClick={() => onSelect(qp)}
          className="max-w-[260px] shrink-0 truncate rounded-full border border-input bg-card px-3 py-1 text-caption font-medium text-foreground/85 hover:border-foreground/40 hover:bg-accent hover:text-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring transition-colors disabled:opacity-50"
        >
          {qp}
        </button>
      ))}
    </div>
  );
}
