"use client";

import React from "react";
import { useTranslations } from "next-intl";
import { Key } from "@/components/ui/icons";
import type { ProviderKeyId } from "@/store/slices/userSettingsSlice";
import { cn } from "@/lib/utils";
import { PROVIDERS } from "../constants";

interface ApiKeysSectionProps {
  apiKeyStatus: Record<ProviderKeyId, { configured: boolean; updatedAt?: string | null }>;
  editingKeyProvider: ProviderKeyId | null;
  keyInputValue: string;
  onStartEditKey: (provider: ProviderKeyId) => void;
  onChangeKeyValue: (val: string) => void;
  onCancelEdit: () => void;
  onSaveKey: () => void;
}

export function ApiKeysSection({
  apiKeyStatus,
  editingKeyProvider,
  keyInputValue,
  onStartEditKey,
  onChangeKeyValue,
  onCancelEdit,
  onSaveKey,
}: ApiKeysSectionProps) {
  const t = useTranslations("userBubble");
  const c = useTranslations("common");

  return (
    <>
      <ul className="space-y-2">
        {PROVIDERS.map((p) => {
          const st = apiKeyStatus[p.id];
          return (
            <li key={p.id} className="flex items-center justify-between gap-2">
              <span className="flex items-center gap-2 truncate text-muted-foreground">
                <span
                  className={cn(
                    "h-2 w-2 rounded-full",
                    st?.configured ? "bg-semantic-success" : "bg-border"
                  )}
                  title={st?.configured ? t("keyConfigured") : t("keyMissing")}
                />
                {p.label}
              </span>
              <button
                type="button"
                title={t("enterKeyBtn")}
                aria-label={`${t("enterKeyBtn")}: ${p.label}`}
                onClick={() => onStartEditKey(p.id)}
                className="flex h-6 w-6 items-center justify-center rounded-ctl border border-border bg-card text-foreground hover:bg-accent"
              >
                <Key className="h-3 w-3" />
              </button>
            </li>
          );
        })}
      </ul>
      {editingKeyProvider && (
        <div className="space-y-2 rounded-ctl border border-border bg-card p-2">
          <p className="font-medium text-foreground">
            {t("keyModalTitle", { provider: editingKeyProvider.toUpperCase() })}
          </p>
          <p className="leading-snug text-muted-foreground">{t("keyModalNote")}</p>
          <input
            type="password"
            autoComplete="off"
            value={keyInputValue}
            onChange={(e) => onChangeKeyValue(e.target.value)}
            placeholder={t("keyPlaceholder")}
            className="h-7 w-full rounded-ctl border border-border bg-background px-2 font-mono text-footnote text-foreground"
          />
          <div className="flex justify-end gap-2">
            <button
              type="button"
              onClick={onCancelEdit}
              className="h-6 rounded-ctl border border-border px-2 text-muted-foreground hover:text-foreground"
            >
              {c("cancel")}
            </button>
            <button
              type="button"
              onClick={onSaveKey}
              className="h-6 rounded-ctl bg-primary px-3 font-medium text-primary-foreground"
            >
              {t("saveKeyBtn")}
            </button>
          </div>
        </div>
      )}
    </>
  );
}
