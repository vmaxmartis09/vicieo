"use client";

import React from "react";
import { useStudioStore } from "@/store";
import { NodeBodyDispatcher } from "./NodeBodyDispatcher";
import type { NodeBodyProps } from "./types";

export const NodeBody = React.memo(function NodeBody({
  nodeType,
  nodeId,
  data,
  character,
  connectedInputs,
  compact,
  expanded,
  onPickCharacter,
}: NodeBodyProps) {
  const updateNodeParams = useStudioStore((s) => s.updateNodeParams);
  const nodeCatalog = useStudioStore((s) => s.nodeCatalog);
  const p = data.params || {};
  const lines: string[] = Array.isArray(p.lines)
    ? p.lines
    : typeof data.summaryText === "string" && data.summaryText.length > 0
    ? [data.summaryText]
    : [];

  const catalogDef = nodeCatalog.find((d: any) => d.typeName === nodeType);
  const hasSpeakCapability = Array.isArray(catalogDef?.capabilities)
    ? catalogDef.capabilities.some((c: any) => c === "speak" || c?.name === "speak")
    : false;

  return (
    <NodeBodyDispatcher
      nodeType={nodeType}
      nodeId={nodeId}
      data={data}
      character={character}
      connectedInputs={connectedInputs}
      compact={compact}
      expanded={expanded}
      onPickCharacter={onPickCharacter}
      hasSpeakCapability={hasSpeakCapability}
      lines={lines}
      updateNodeParams={updateNodeParams}
    />
  );
});
