"use client";

import React, { useRef, useState } from "react";
import { useTranslations } from "next-intl";
import { Maximize2, Plus, Trash2, Upload, X } from "@/components/ui/icons";
import type { Character, CharacterImageAsset } from "@/lib/repositories/types";
import { cn } from "@/lib/utils";
import {
  appendUploadedReferenceAsset,
  markPrimaryReferenceAsset,
  removeReferenceAsset,
} from "../utils/characterAssets";

interface SectionImagesGalleryProps {
  selectedChar: Character;
  refLib: CharacterImageAsset[];
  onSaveCharacter: (ch: Character) => Promise<void>;
  onGeneratePhotoByNode: (ch: Character) => void;
}

export function SectionImagesGallery({
  selectedChar,
  refLib,
  onSaveCharacter,
  onGeneratePhotoByNode,
}: SectionImagesGalleryProps) {
  const t = useTranslations("projectTab");
  const c = useTranslations("common");
  const photoInputRef = useRef<HTMLInputElement>(null);
  const [lightboxImage, setLightboxImage] = useState<CharacterImageAsset | null>(null);

  const handleUploadReferencePhoto = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = () => {
      if (typeof reader.result !== "string") return;
      onSaveCharacter(appendUploadedReferenceAsset(selectedChar, reader.result));
    };
    reader.readAsDataURL(file);
  };

  return (
    <div className="space-y-3">
      <div className="flex items-center gap-2">
        <input
          ref={photoInputRef}
          type="file"
          accept="image/*"
          onChange={handleUploadReferencePhoto}
          className="hidden"
        />
        <button
          type="button"
          onClick={() => photoInputRef.current?.click()}
          className="flex h-7 items-center gap-1 rounded-ctl border border-border bg-card px-3 text-footnote font-medium text-foreground hover:bg-accent"
        >
          <Upload className="h-3.5 w-3.5" />
          <span>{t("images.uploadBtn")}</span>
        </button>
        <button
          type="button"
          onClick={() => onGeneratePhotoByNode(selectedChar)}
          className="flex h-7 items-center gap-1 rounded-ctl border border-border bg-card px-3 text-footnote font-medium text-foreground hover:bg-accent"
        >
          <Plus className="h-3.5 w-3.5" />
          <span>{t("images.generateByNodeBtn")}</span>
        </button>
      </div>

      <div className="grid grid-cols-2 gap-2">
        {refLib.map((asset) => (
          <div
            key={asset.id}
            className={cn(
              "rounded-card border bg-card p-2 space-y-2",
              asset.isPrimary ? "border-brand" : "border-border"
            )}
          >
            <div className="relative overflow-hidden rounded-ctl border border-border bg-muted/20">
              {/* eslint-disable-next-line @next/next/no-img-element */}
              <img
                decoding="async"
                loading="lazy"
                src={asset.url}
                alt={asset.angle}
                className="h-28 w-full object-cover object-top"
              />
              {asset.isPrimary && (
                <span className="absolute left-2 top-2 rounded-xs bg-brand px-2 py-1 text-caption font-medium text-white">
                  {t("images.primaryBadge")}
                </span>
              )}
              <button
                type="button"
                onClick={() => setLightboxImage(asset)}
                title={t("images.viewLargeBtn")}
                aria-label={t("images.viewLargeBtn")}
                className="absolute right-2 top-2 flex h-6 w-6 items-center justify-center rounded-xs border border-border bg-card/90 text-foreground"
              >
                <Maximize2 className="h-3 w-3" />
              </button>
            </div>

            <p className="truncate font-medium text-foreground text-footnote">
              {t("images.angleLabel", { angle: asset.angle })}
            </p>
            <p className="truncate text-caption text-muted-foreground">
              {asset.source === "upload"
                ? t("images.sourceUpload")
                : t("images.sourceGenerated", {
                    model: asset.model || "gemini-2.5-flash-image",
                  })}
            </p>
            {asset.prompt && (
              <p className="truncate font-mono text-caption text-muted-foreground">
                {asset.prompt}
              </p>
            )}

            <div className="flex items-center justify-between pt-1">
              {!asset.isPrimary ? (
                <button
                  type="button"
                  onClick={() =>
                    onSaveCharacter(markPrimaryReferenceAsset(selectedChar, asset.id))
                  }
                  className="rounded-ctl border border-border px-2 py-1 text-caption text-foreground hover:bg-accent"
                >
                  {t("images.setPrimaryBtn")}
                </button>
              ) : (
                <span />
              )}
              <button
                type="button"
                onClick={() =>
                  onSaveCharacter(removeReferenceAsset(selectedChar, asset.id))
                }
                aria-label={c("delete")}
                className="rounded-ctl p-1 text-muted-foreground hover:text-semantic-danger"
              >
                <Trash2 className="h-3 w-3" />
              </button>
            </div>
          </div>
        ))}
      </div>

      {lightboxImage && (
        <div
          role="dialog"
          aria-modal="true"
          className="fixed inset-0 z-50 flex items-center justify-center bg-foreground/40 p-4"
        >
          <div className="max-w-lg w-full rounded-sheet border border-border bg-popover p-4 text-popover-foreground space-y-3">
            <div className="flex items-center justify-between">
              <span className="font-semibold text-headline">{lightboxImage.angle}</span>
              <button
                type="button"
                onClick={() => setLightboxImage(null)}
                aria-label={c("close")}
                className="rounded-ctl p-1 hover:bg-accent"
              >
                <X className="h-4 w-4" />
              </button>
            </div>
            {/* eslint-disable-next-line @next/next/no-img-element */}
            <img
              decoding="async"
              loading="lazy"
              src={lightboxImage.url}
              alt={lightboxImage.angle}
              className="max-h-[65dvh] w-full rounded-card border border-border object-contain bg-background"
            />
            <p className="text-body text-muted-foreground">
              {lightboxImage.source === "upload"
                ? t("images.sourceUpload")
                : `${lightboxImage.model || "AI"} — ${lightboxImage.prompt || ""}`}
            </p>
          </div>
        </div>
      )}
    </div>
  );
}
