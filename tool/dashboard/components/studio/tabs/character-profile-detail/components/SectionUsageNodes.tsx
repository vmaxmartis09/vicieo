"use client";

import React, { useMemo } from "react";
import { useTranslations } from "next-intl";
import { useShallow } from "zustand/react/shallow";
import { useStudioStore, type StudioStore } from "@/store";
import type { Character } from "@/lib/repositories/types";
import { findReferencingNodes } from "../utils/characterAssets";

interface SectionUsageNodesProps {
  selectedChar: Character;
}

export function SectionUsageNodes({ selectedChar }: SectionUsageNodesProps) {
  const t = useTranslations("projectTab");
  const { nodes, setSelectedNodeId, flashNode } = useStudioStore(
    useShallow((s: StudioStore) => ({
      nodes: s.nodes,
      setSelectedNodeId: s.setSelectedNodeId,
      flashNode: s.flashNode,
    }))
  );

  const referencingNodes = useMemo(
    () => findReferencingNodes(nodes as StudioStore["nodes"], selectedChar),
    [nodes, selectedChar]
  );

  return (
    <div className="space-y-4">
      <div className="space-y-2 border-b border-border pb-3">
        <h4 className="font-semibold text-foreground text-footnote">
          {t("usage.episodesTitle")}
        </h4>
        <ul className="space-y-2">
          {(selectedChar.appearances || []).map((ap, i) => (
            <li
              key={`${ap.episodeId || ap.episodeTitle}-${i}`}
              className="rounded-ctl border border-border bg-card p-2 space-y-1"
            >
              <p className="font-medium text-foreground text-footnote">
                {ap.episodeTitle}
              </p>
              <p className="text-body text-muted-foreground">
                {ap.scenes.join(" · ")}
              </p>
            </li>
          ))}
        </ul>
      </div>

      <div className="space-y-2">
        <h4 className="font-semibold text-foreground text-footnote">
          {t("usage.canvasNodesTitle")}
        </h4>
        {referencingNodes.length === 0 ? (
          <p className="text-muted-foreground text-footnote">{t("usage.noNodes")}</p>
        ) : (
          <ul className="space-y-2">
            {referencingNodes.map((n) => (
              <li
                key={n.id}
                className="flex items-center justify-between rounded-ctl border border-border bg-card p-2"
              >
                <div>
                  <p className="font-mono font-medium text-foreground text-footnote">
                    {n.id}
                  </p>
                  <p className="text-body text-muted-foreground">
                    {n.data.nodeType}
                  </p>
                </div>
                <button
                  type="button"
                  onClick={() => {
                    setSelectedNodeId(n.id);
                    flashNode(n.id);
                  }}
                  className="rounded-ctl border border-border bg-card px-2 py-1 text-body font-medium text-foreground hover:bg-accent"
                >
                  {t("usage.jumpToNodeBtn")}
                </button>
              </li>
            ))}
          </ul>
        )}
      </div>
    </div>
  );
}
