"use client";

import React, { useRef, useState } from "react";
import { cn } from "@/lib/utils";

/** Vùng nhập file: bấm, kéo-thả hoặc dán (Ctrl+V). */
export function ToolboxDropzone({
  accept,
  icon,
  title,
  hint,
  busy,
  onFile,
}: {
  accept: string;
  icon: React.ReactNode;
  title: string;
  hint: string;
  busy?: boolean;
  onFile: (f: File) => void;
}) {
  const ref = useRef<HTMLInputElement>(null);
  const [over, setOver] = useState(false);
  return (
    <div
      role="button"
      tabIndex={0}
      aria-label={title}
      onClick={() => !busy && ref.current?.click()}
      onKeyDown={(e) => (e.key === "Enter" || e.key === " ") && ref.current?.click()}
      onDragOver={(e) => {
        e.preventDefault();
        setOver(true);
      }}
      onDragLeave={() => setOver(false)}
      onDrop={(e) => {
        e.preventDefault();
        setOver(false);
        const f = e.dataTransfer.files?.[0];
        if (f && !busy) onFile(f);
      }}
      onPaste={(e) => {
        const f = e.clipboardData.files?.[0];
        if (f && !busy) onFile(f);
      }}
      className={cn(
        "flex cursor-pointer flex-col items-start gap-1 rounded-xl border border-dashed p-3 text-left transition-colors",
        over ? "border-brand bg-brand/5" : "border-border hover:border-foreground/30 hover:bg-accent/40",
        busy && "pointer-events-none opacity-60"
      )}
    >
      <span className="flex items-center gap-2 text-footnote font-medium text-foreground">
        <span className="text-muted-foreground [&_svg]:h-4 [&_svg]:w-4">{icon}</span>
        {title}
      </span>
      <span className="text-caption text-muted-foreground">{hint}</span>
      <input
        ref={ref}
        type="file"
        accept={accept}
        className="hidden"
        onChange={(e) => {
          const f = e.target.files?.[0];
          if (f) onFile(f);
          e.target.value = "";
        }}
      />
    </div>
  );
}
