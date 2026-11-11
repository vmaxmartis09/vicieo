"use client";

import React, { useRef, useState } from "react";
import { useTranslations } from "next-intl";
import { Volume2 } from "@/components/ui/icons";
import type { Character } from "@/lib/repositories/types";
import { VoiceCloneModal } from "@/components/voice/VoiceCloneModal";
import { useCharacterVoiceTest } from "../hooks/useCharacterVoiceTest";

interface SectionVoiceCloningProps {
  selectedChar: Character;
  activeProjectId: string;
  onSaveCharacter: (ch: Character) => Promise<void>;
}

export function SectionVoiceCloning({
  selectedChar,
  activeProjectId,
  onSaveCharacter,
}: SectionVoiceCloningProps) {
  const t = useTranslations("projectTab");
  const sampleAudioRef = useRef<HTMLAudioElement>(null);
  const [cloneModalOpen, setCloneModalOpen] = useState(false);

  const voiceTest = useCharacterVoiceTest({
    voiceId: selectedChar.voiceId,
    sampleAudioUrl: selectedChar.sampleAudioUrl,
    defaultSentence: t("voice.testInputDefault"),
  });

  return (
    <div className="space-y-3">
      <div className="rounded-card border border-border bg-card p-3 space-y-2">
        <div className="flex items-center justify-between">
          <span className="text-muted-foreground text-footnote">
            {t("voice.voiceIdLabel")}
          </span>
          <code className="rounded-xs bg-accent px-2 py-1 font-mono text-footnote font-semibold text-foreground">
            {selectedChar.voiceId}
          </code>
        </div>

        <audio
          ref={sampleAudioRef}
          src={selectedChar.sampleAudioUrl || "/demo/audio/soc-nho_sample.mp3"}
          controls
          preload="metadata"
          className="h-8 w-full"
        />
      </div>

      <div className="rounded-card border border-border bg-card p-3 space-y-3">
        <label className="block space-y-1">
          <span className="font-medium text-foreground text-footnote">
            {t("voice.tempoLabel", { tempo: voiceTest.ttsTempo.toFixed(2) })}
          </span>
          <input
            type="range"
            min="0.8"
            max="1.4"
            step="0.05"
            value={voiceTest.ttsTempo}
            onChange={(e) => voiceTest.setTtsTempo(Number(e.target.value))}
            className="w-full accent-brand"
          />
        </label>

        <label className="block space-y-1">
          <span className="font-medium text-foreground text-footnote">
            {t("voice.testInputLabel")}
          </span>
          <textarea
            rows={3}
            value={voiceTest.ttsSentence}
            onChange={(e) => voiceTest.setTtsSentence(e.target.value)}
            className="w-full rounded-ctl border border-border bg-background p-2 text-footnote text-foreground"
          />
        </label>

        <button
          type="button"
          onClick={voiceTest.handleTestOmniVoice}
          disabled={voiceTest.ttsSpeaking}
          className="flex h-9 w-full items-center justify-center gap-2 rounded-ctl bg-primary px-3 text-footnote font-medium text-primary-foreground"
        >
          <Volume2 className="h-3.5 w-3.5" />
          <span>
            {voiceTest.ttsSpeaking ? t("voice.speaking") : t("voice.testSpeakBtn")}
          </span>
        </button>

        {voiceTest.ttsPreviewUrl && (
          <div className="pt-1">
            <audio
              src={voiceTest.ttsPreviewUrl}
              controls
              autoPlay
              className="h-8 w-full"
            />
          </div>
        )}
      </div>

      <button
        type="button"
        onClick={() => setCloneModalOpen(true)}
        className="h-9 w-full rounded-ctl border border-border bg-card px-3 text-footnote font-medium text-foreground hover:bg-accent"
      >
        {t("voice.cloneVoiceBtn")}
      </button>

      <VoiceCloneModal
        isOpen={cloneModalOpen}
        onClose={() => setCloneModalOpen(false)}
        defaultVoiceId={selectedChar.voiceId || selectedChar.id}
        projectId={activeProjectId}
        characterId={selectedChar.id}
        sourceSamples={(selectedChar.voiceFiles ?? []).map((f) => ({
          name: f.name,
          url: f.url,
        }))}
        onSuccess={async (res) => {
          await onSaveCharacter({
            ...selectedChar,
            voiceId: res.voiceId,
            sampleAudioUrl: res.sampleUrl || `/voices/${res.voiceId}/sample`,
          });
          setCloneModalOpen(false);
        }}
      />
    </div>
  );
}
