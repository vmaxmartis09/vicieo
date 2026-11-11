"use client";

import React from "react";
import { Link2 } from "@/components/ui/icons";
import type { CandidateNodeItem } from "../types";

export function NodesTab({
  nodes,
  onConnectNode,
}: {
  nodes: CandidateNodeItem[];
  onConnectNode: (node: CandidateNodeItem, outPort?: string) => void;
}) {
  return (
    <div className="max-h-[260px] overflow-y-auto space-y-2 pr-1 scroll-quiet">
      {nodes.length === 0 ? (
        <p className="py-4 text-center text-footnote text-muted-foreground">Không có node khả dụng</p>
      ) : (
        nodes.map((n) => (
          <div
            key={n.id}
            className="rounded-card border border-border/60 p-2 text-left space-y-1 hover:border-foreground/30 bg-card/60"
          >
            <div className="flex items-center justify-between">
              <span className="font-semibold text-footnote text-foreground truncate">{n.title}</span>
              <span className="text-caption text-muted-foreground uppercase">{n.typeName}</span>
            </div>

            {n.summary && (
              <p className="line-clamp-2 text-caption text-muted-foreground leading-tight">
                {n.summary}
              </p>
            )}

            <div className="flex flex-wrap items-center gap-1 pt-1 border-t border-border/40">
              {n.outputs.map((out) => (
                <button
                  key={out}
                  type="button"
                  onClick={() => onConnectNode(n, out)}
                  className="inline-flex items-center gap-1 rounded-xs bg-accent px-2 py-1 text-caption font-mono font-medium text-foreground hover:bg-brand/15 hover:text-brand transition-colors"
                >
                  <Link2 className="h-3 w-3 text-brand" />
                  {`{${n.title}.${out}}`}
                </button>
              ))}
              {n.outputs.length === 0 && (
                <button
                  type="button"
                  onClick={() => onConnectNode(n, "output")}
                  className="inline-flex items-center gap-1 rounded-xs bg-accent px-2 py-1 text-caption font-mono font-medium text-foreground hover:bg-brand/15 hover:text-brand transition-colors"
                >
                  <Link2 className="h-3 w-3 text-brand" />
                  {`{${n.title}.output}`}
                </button>
              )}
            </div>
          </div>
        ))
      )}
    </div>
  );
}
