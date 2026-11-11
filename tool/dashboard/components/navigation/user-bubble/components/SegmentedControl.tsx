"use client";

import React from "react";
import { cn } from "@/lib/utils";

interface SegmentedControlProps<T extends string> {
  items: { id: T; label: React.ReactNode }[];
  value: T;
  onChange: (v: T) => void;
}

export function SegmentedControl<T extends string>({
  items,
  value,
  onChange,
}: SegmentedControlProps<T>) {
  return (
    <div className="flex rounded-ctl border border-border bg-card p-1">
      {items.map((it) => (
        <button
          key={it.id}
          type="button"
          onClick={() => onChange(it.id)}
          className={cn(
            "flex h-7 flex-1 items-center justify-center rounded-xs px-2 text-footnote font-medium transition-colors active:scale-95",
            value === it.id
              ? "bg-accent text-foreground"
              : "text-muted-foreground hover:text-foreground"
          )}
        >
          {it.label}
        </button>
      ))}
    </div>
  );
}
