"use client";

import React from "react";
import { ProductNodeCard } from "../ProductNodeCard";
import { CharacterHero } from "../CharacterHero";
import { CastPreview } from "../CastPreview";
import { NodeImageInput } from "../NodeInputs";
import { ScriptAndVoiceRenderer } from "./renderers/ScriptAndVoiceRenderer";
import { MediaGenRenderer } from "./renderers/MediaGenRenderer";
import { ExportAndPolicyRenderer } from "./renderers/ExportAndPolicyRenderer";
import { ToolboxAndGenericRenderer } from "./renderers/ToolboxAndGenericRenderer";
import type { NodeBodyProps, NodeRendererContextProps } from "./types";

const SOURCE_TYPES = new Set(["text", "input", "video", "audio"]);
const SCRIPT_VOICE_TYPES = new Set(["script", "voice_tts", "captions"]);
const MEDIA_GEN_TYPES = new Set(["image_gen", "image_edit", "video_gen", "transition"]);
const EXPORT_POLICY_TYPES = new Set([
  "hyperframes_render",
  "export",
  "policy",
  "quality_check",
  "vision_analyze",
]);

interface DispatcherProps extends NodeBodyProps {
  hasSpeakCapability: boolean;
  lines: string[];
  updateNodeParams: (id: string, patch: Record<string, any>) => void;
}

export function NodeBodyDispatcher({
  nodeType,
  nodeId,
  data,
  character,
  connectedInputs,
  compact,
  expanded,
  onPickCharacter,
  hasSpeakCapability,
  lines,
  updateNodeParams,
}: DispatcherProps) {
  if (SOURCE_TYPES.has(nodeType)) {
    return null;
  }

  const p = data.params || {};

  if (nodeType === "image") {
    return (
      <NodeImageInput
        nodeId={nodeId}
        uri={p.uri}
        name={p.name}
        nodeType="image"
        previewUrl={data.previewUrl}
        compact={compact}
        expanded={expanded}
      />
    );
  }

  if (nodeType === "cast") {
    return <CastPreview nodeId={nodeId} data={data} />;
  }

  if (nodeType === "product") {
    return <ProductNodeCard nodeId={nodeId} params={p} compact={compact} />;
  }

  if (nodeType === "character") {
    return (
      <div className="space-y-2">
        {character && (
          <CharacterHero
            c={character}
            compact={compact}
            detailed={Boolean(expanded)}
            onPick={onPickCharacter}
            speak={
              hasSpeakCapability
                ? {
                    enabled: Boolean(p.speak?.enabled),
                    hasLines: connectedInputs.includes("lines"),
                    onToggle: () =>
                      updateNodeParams(nodeId, {
                        speak: { ...(p.speak || {}), enabled: !p.speak?.enabled },
                      }),
                    text: String(p.speak?.text || ""),
                    onTextChange: (t: string) =>
                      updateNodeParams(nodeId, { speak: { ...(p.speak || {}), text: t } }),
                  }
                : undefined
            }
          />
        )}
      </div>
    );
  }

  const ctx: NodeRendererContextProps = {
    nodeType,
    nodeId,
    data,
    params: p,
    previewUrl: data.previewUrl,
    lines,
    connectedInputs,
    compact,
    expanded,
    updateNodeParams,
  };

  if (SCRIPT_VOICE_TYPES.has(nodeType)) {
    return <ScriptAndVoiceRenderer {...ctx} />;
  }

  if (MEDIA_GEN_TYPES.has(nodeType)) {
    return <MediaGenRenderer {...ctx} />;
  }

  if (EXPORT_POLICY_TYPES.has(nodeType)) {
    return <ExportAndPolicyRenderer {...ctx} />;
  }

  return <ToolboxAndGenericRenderer {...ctx} />;
}
