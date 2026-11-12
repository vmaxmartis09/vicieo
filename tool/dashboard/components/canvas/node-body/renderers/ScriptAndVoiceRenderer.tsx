"use client";

import React from "react";
import { Reveal } from "../../Reveal";
import {
  NodeScriptInput,
  NodeVoiceInput,
  NodeCaptionsInput,
} from "../../NodeInputs";
import { ScrollText } from "@/components/ui/icons";
import { Empty } from "../components/Empty";
import { VoiceHoverSpeaker } from "../components/VoiceHoverSpeaker";
import { ShotPackLink } from "../components/ShotPackLink";
import type { NodeRendererContextProps } from "../types";

export function ScriptAndVoiceRenderer({
  nodeType,
  nodeId,
  data,
  params: p,
  previewUrl: url,
  lines,
  connectedInputs,
  expanded,
}: NodeRendererContextProps) {
  switch (nodeType) {
    case "script":
      return (
        <div>
          <Reveal show={!expanded}>
            {lines.length > 0 ? (
              <ul className="space-y-1 text-footnote text-foreground">
                {lines.slice(0, 4).map((l, i) => (
                  <li key={`${nodeId}-line-${i}-${l.slice(0, 10)}`} className="line-clamp-1">
                    {l}
                  </li>
                ))}
              </ul>
            ) : (
              <Empty icon={<ScrollText className="h-5 w-5" />} text="Chưa có phân cảnh" />
            )}
          </Reveal>
          <Reveal show={Boolean(expanded)}>
            <NodeScriptInput
              nodeId={nodeId}
              mode={p.mode || "paste"}
              rawScript={p.rawScript || p.content || p.text || ""}
              episodeId={p.episodeId}
              granularity={p.granularity}
              ideaText={p.text || p.content || ""}
              targetSceneCount={p.targetSceneCount}
              targetDurationSec={p.targetDurationSec}
              connectedText={connectedInputs.includes("text")}
            />
          </Reveal>
          {p.mode === "episode" && p.episodeId && <ShotPackLink episodeId={String(p.episodeId)} />}
        </div>
      );

    case "voice_tts":
      return (
        <div className="space-y-2">
          <Reveal show={Boolean(expanded)}>
            <NodeVoiceInput
              nodeId={nodeId}
              text={p.text}
              voiceId={p.voiceId}
              tempo={p.tempo}
              connectedText={connectedInputs.includes("text")}
            />
          </Reveal>
          {!expanded && (
            <VoiceHoverSpeaker
              nodeId={nodeId}
              ownText={typeof p.text === "string" ? p.text : ""}
              voiceId={p.voiceId}
              audioUrl={url && data.previewType === "audio" ? url : undefined}
            />
          )}
          {url && data.previewType === "audio" && (
            <div className="pt-1">
              <audio
                src={url}
                controls
                preload="metadata"
                className="nodrag h-7 w-full"
              />
            </div>
          )}
        </div>
      );

    case "captions":
      return (
        <div className="space-y-2">
          <Reveal show={Boolean(expanded)}>
            <NodeCaptionsInput
              nodeId={nodeId}
              style={p.style}
              language={p.language}
            />
          </Reveal>
          {lines.length === 0 && !expanded && (
            <Empty icon={<ScrollText className="h-5 w-5" />} text="Chưa có phụ đề" />
          )}
          {lines.length > 0 && (
            <ul className="space-y-1 rounded-ctl border border-border bg-muted/20 p-2 text-footnote">
              {lines.slice(0, 4).map((l, i) => (
                <li
                  key={`${nodeId}-cap-${i}-${l.slice(0, 10)}`}
                  className="truncate text-muted-foreground"
                >
                  {l}
                </li>
              ))}
            </ul>
          )}
        </div>
      );

    default:
      return null;
  }
}
