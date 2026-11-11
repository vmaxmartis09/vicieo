"use client";

import React from "react";
import { cn } from "@/lib/utils";

export function Empty({
  icon,
  text,
  tall,
}: {
  icon: React.ReactNode;
  text?: string;
  tall?: boolean;
}) {
  return (
    <div
      className={cn(
        "flex flex-col items-center justify-center gap-1 rounded-ctl border border-dashed border-border text-muted-foreground",
        tall ? "h-36" : "h-20"
      )}
    >
      {icon}
      {text && <span className="text-caption">{text}</span>}
    </div>
  );
}
