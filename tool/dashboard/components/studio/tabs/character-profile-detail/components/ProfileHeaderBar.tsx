"use client";

import React from "react";
import { useTranslations } from "next-intl";
import type { CharacterProfileSection } from "@/store/slices/rightPanelSlice";
import type { Character } from "@/lib/repositories/types";
import { cn } from "@/lib/utils";

interface ProfileHeaderBarProps {
  selectedChar: Character;
  savedToast: boolean;
  refCount: number;
  clipsCount: number;
  activeCharacterSection: CharacterProfileSection;
  onSelectSection: (s: CharacterProfileSection) => void;
  onBackToList: () => void;
  onDropToCanvas: (ch: Character) => void;
}

export function ProfileHeaderBar({
  selectedChar,
  savedToast,
  refCount,
  clipsCount,
  activeCharacterSection,
  onSelectSection,
  onBackToList,
  onDropToCanvas,
}: ProfileHeaderBarProps) {
  const t = useTranslations("projectTab");

  const sections: Array<{ id: CharacterProfileSection; label: string }> = [
    { id: "profile", label: t("sections.profile") },
    { id: "images", label: t("sections.images", { count: refCount }) },
    { id: "videos", label: t("sections.videos", { count: clipsCount }) },
    { id: "voice", label: t("sections.voice") },
    { id: "usage", label: t("sections.usage") },
  ];

  return (
    <>
      {/* Top bar: Back to character list + Drop onto canvas */}
      <div className="flex items-center justify-between gap-2 border-b border-border pb-3">
        <button
          type="button"
          onClick={onBackToList}
          className="h-7 rounded-ctl border border-border bg-card px-3 text-footnote font-medium text-foreground hover:bg-accent"
        >
          ← {t("backToList")}
        </button>
        <button
          type="button"
          onClick={() => onDropToCanvas(selectedChar)}
          className="h-7 rounded-ctl bg-primary px-3 text-footnote font-medium text-primary-foreground"
        >
          + {t("dropToCanvasBtn")}
        </button>
      </div>

      {/* Character summary header */}
      <div className="flex items-center gap-3">
        {selectedChar.referenceImages[0] && (
          // eslint-disable-next-line @next/next/no-img-element
          <img
            decoding="async"
            loading="lazy"
            src={selectedChar.referenceImages[0]}
            alt={selectedChar.name}
            className="h-12 w-12 shrink-0 rounded-ctl border border-border object-cover object-top"
          />
        )}
        <div className="min-w-0 flex-1">
          <h3 className="truncate text-headline font-semibold text-foreground">
            {selectedChar.name}
          </h3>
          <p className="text-body text-muted-foreground tnum">
            {selectedChar.role === "main"
              ? t("profile.roleMain")
              : t("profile.roleSupporting")}{" "}
            · {selectedChar.heightCm} cm ({selectedChar.scaleRatio}x) ·{" "}
            <code>{selectedChar.voiceId}</code>
          </p>
        </div>
        {savedToast && (
          <span className="rounded-xs bg-semantic-success/15 px-2 py-1 text-body font-medium text-semantic-success">
            {t("profile.savedToast")}
          </span>
        )}
      </div>

      {/* 5 Section Tabs */}
      <div
        role="tablist"
        className="flex flex-wrap gap-1 border-b border-border pb-2"
      >
        {sections.map((sec) => (
          <button
            key={sec.id}
            type="button"
            role="tab"
            aria-selected={activeCharacterSection === sec.id}
            onClick={() => onSelectSection(sec.id)}
            className={cn(
              "h-7 rounded-ctl px-3 text-footnote font-medium transition-colors",
              activeCharacterSection === sec.id
                ? "bg-accent text-foreground border border-border"
                : "text-muted-foreground hover:text-foreground"
            )}
          >
            {sec.label}
          </button>
        ))}
      </div>
    </>
  );
}
