"use client";

import React from "react";
import { useTranslations } from "next-intl";
import type { Character } from "@/lib/repositories/types";

interface SectionProfileFormProps {
  selectedChar: Character;
  onSaveCharacter: (ch: Character) => Promise<void>;
}

export function SectionProfileForm({
  selectedChar,
  onSaveCharacter,
}: SectionProfileFormProps) {
  const t = useTranslations("projectTab");

  return (
    <div className="space-y-3">
      <div className="grid grid-cols-2 gap-2">
        <label className="space-y-1">
          <span className="text-muted-foreground">{t("profile.name")}</span>
          <input
            type="text"
            value={selectedChar.name}
            onChange={(e) =>
              onSaveCharacter({ ...selectedChar, name: e.target.value })
            }
            className="h-7 w-full rounded-ctl border border-border bg-card px-2 text-footnote text-foreground"
          />
        </label>
        <label className="space-y-1">
          <span className="text-muted-foreground">{t("profile.role")}</span>
          <select
            value={selectedChar.role}
            onChange={(e) =>
              onSaveCharacter({
                ...selectedChar,
                role: e.target.value as "main" | "supporting",
              })
            }
            className="h-7 w-full rounded-ctl border border-border bg-card px-2 text-footnote text-foreground"
          >
            <option value="main">{t("profile.roleMain")}</option>
            <option value="supporting">{t("profile.roleSupporting")}</option>
          </select>
        </label>
      </div>

      <div className="grid grid-cols-2 gap-2">
        <label className="space-y-1">
          <span className="text-muted-foreground">{t("profile.heightCm")}</span>
          <input
            type="number"
            value={selectedChar.heightCm}
            onChange={(e) => {
              const cm = Number(e.target.value) || 100;
              onSaveCharacter({
                ...selectedChar,
                heightCm: cm,
                scaleRatio: Number((cm / 100).toFixed(2)),
              });
            }}
            className="h-7 w-full rounded-ctl border border-border bg-card px-2 text-footnote tnum text-foreground"
          />
        </label>
        <label className="space-y-1">
          <span className="text-muted-foreground">
            {t("profile.scaleRatio")}
          </span>
          <input
            type="number"
            step="0.1"
            value={selectedChar.scaleRatio}
            readOnly
            className="h-7 w-full rounded-ctl border border-border bg-muted px-2 text-footnote tnum text-muted-foreground"
          />
        </label>
      </div>

      <label className="block space-y-1">
        <span className="text-muted-foreground">{t("profile.description")}</span>
        <textarea
          rows={2}
          value={selectedChar.description || ""}
          onChange={(e) =>
            onSaveCharacter({ ...selectedChar, description: e.target.value })
          }
          className="w-full rounded-ctl border border-border bg-card p-2 text-footnote text-foreground"
        />
      </label>

      <label className="block space-y-1">
        <span className="text-muted-foreground">{t("profile.promptLock")}</span>
        <textarea
          rows={2}
          value={selectedChar.promptLock}
          onChange={(e) =>
            onSaveCharacter({ ...selectedChar, promptLock: e.target.value })
          }
          className="w-full rounded-ctl border border-border bg-card p-2 font-mono text-body text-foreground"
        />
      </label>

      <label className="block space-y-1">
        <span className="text-muted-foreground">{t("profile.personality")}</span>
        <input
          type="text"
          value={selectedChar.memory.personality}
          onChange={(e) =>
            onSaveCharacter({
              ...selectedChar,
              memory: {
                ...selectedChar.memory,
                personality: e.target.value,
              },
            })
          }
          className="h-7 w-full rounded-ctl border border-border bg-card px-2 text-footnote text-foreground"
        />
      </label>

      <label className="block space-y-1">
        <span className="text-muted-foreground">{t("profile.catchphrases")}</span>
        <textarea
          rows={2}
          value={selectedChar.memory.catchphrases.join("\n")}
          onChange={(e) =>
            onSaveCharacter({
              ...selectedChar,
              memory: {
                ...selectedChar.memory,
                catchphrases: e.target.value.split("\n").filter((x) => x.trim()),
              },
            })
          }
          className="w-full rounded-ctl border border-border bg-card p-2 text-footnote text-foreground"
        />
      </label>

      <label className="block space-y-1">
        <span className="text-muted-foreground">{t("profile.habits")}</span>
        <textarea
          rows={2}
          value={selectedChar.memory.habits.join("\n")}
          onChange={(e) =>
            onSaveCharacter({
              ...selectedChar,
              memory: {
                ...selectedChar.memory,
                habits: e.target.value.split("\n").filter((x) => x.trim()),
              },
            })
          }
          className="w-full rounded-ctl border border-border bg-card p-2 text-footnote text-foreground"
        />
      </label>

      <label className="block space-y-1">
        <span className="text-muted-foreground">{t("profile.pastEvents")}</span>
        <textarea
          rows={3}
          value={(
            selectedChar.memory.pastEvents ||
            selectedChar.memory.pastEventsSummary ||
            []
          ).join("\n")}
          onChange={(e) => {
            const evs = e.target.value.split("\n").filter((x) => x.trim());
            onSaveCharacter({
              ...selectedChar,
              memory: {
                ...selectedChar.memory,
                pastEvents: evs,
                pastEventsSummary: evs,
              },
            });
          }}
          className="w-full rounded-ctl border border-border bg-card p-2 text-footnote text-foreground"
        />
      </label>
    </div>
  );
}
