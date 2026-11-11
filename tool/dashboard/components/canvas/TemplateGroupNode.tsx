"use client";

import React, { memo } from "react";
import type { NodeProps, Node } from "@xyflow/react";
import { Layers } from "@/components/ui/icons";
import { cn } from "@/lib/utils";

export const GROUP_TYPE = "templateGroup";

export type TemplateGroupData = {
  nodeType: "group";
  label: string;
  templateId?: string;
  count?: number;
  status?: string;
  params?: Record<string, any>;
  generations?: any[];
} & Record<string, unknown>;

/** Khung Group: chứa toàn bộ node của một template. Kéo khung = kéo cả flow. */
function TemplateGroupNodeImpl({ data, selected }: NodeProps<Node<TemplateGroupData>>) {
  return (
    <div
      className={cn(
        "h-full w-full rounded-panel border-2 border-dashed bg-card/30 transition-colors",
        selected ? "border-foreground/60" : "border-border"
      )}
    >
      <div className="flex items-center gap-2 px-3 py-2 text-left">
        <span className="flex h-6 w-6 items-center justify-center rounded-ctl border border-border bg-card">
          <Layers className="h-4 w-4 text-foreground" />
        </span>
        <span className="text-footnote font-bold text-foreground">{data.label}</span>
        {typeof data.count === "number" && (
          <span className="rounded-xs bg-muted px-2 py-1 text-caption font-semibold text-muted-foreground tnum">
            {data.count} node
          </span>
        )}
      </div>
    </div>
  );
}

export const TemplateGroupNode = memo(TemplateGroupNodeImpl);
