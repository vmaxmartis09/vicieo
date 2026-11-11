"use client";

import React from "react";
import {
  Image as ImageIcon,
  Layers,
  Film,
  Pencil,
  Scissors,
  AudioLines,
  Mic,
} from "@/components/ui/icons";
import { cn } from "@/lib/utils";

export const TOOL_ICON: Record<string, React.ReactNode> = {
  "bg-remove": <ImageIcon className="h-4 w-4" />,
  "depth-layers": <Layers className="h-4 w-4" />,
  "ffmpeg-bin": <Film className="h-4 w-4" />,
  handdraw: <Pencil className="h-4 w-4" />,
  "scene-cut": <Scissors className="h-4 w-4" />,
  "stt-align": <AudioLines className="h-4 w-4" />,
  "voice-clone": <Mic className="h-4 w-4" />,
};

export const CARD = "rounded-card border bg-card p-3 transition-colors";

export const IconBox = ({ children, tone = "brand" }: { children: React.ReactNode; tone?: "brand" | "muted" }) => (
  <span className={cn("flex h-8 w-8 shrink-0 items-center justify-center rounded-lg", tone === "brand" ? "bg-brand/10 text-brand" : "bg-muted text-muted-foreground")}>{children}</span>
);

export const EASE = [0.22, 1, 0.36, 1] as const;

export const checker =
  "rounded-lg border border-border bg-[radial-gradient(hsl(var(--border))_1px,transparent_1px)] [background-size:12px_12px] p-2 flex items-center justify-center";
