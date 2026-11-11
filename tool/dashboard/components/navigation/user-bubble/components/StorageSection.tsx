"use client";

import React from "react";
import { useTranslations } from "next-intl";
import { Trash2 } from "@/components/ui/icons";

interface StorageSectionProps {
  storageBackend: "local" | "supabase";
  onChangeStorageBackend: (backend: "local" | "supabase") => void;
  formattedStorage: string;
  clearStep: 0 | 1 | 2;
  onResetClearStep: () => void;
  onAdvanceClearStep: () => void;
  inputClassName: string;
}

export function StorageSection({
  storageBackend,
  onChangeStorageBackend,
  formattedStorage,
  clearStep,
  onResetClearStep,
  onAdvanceClearStep,
  inputClassName,
}: StorageSectionProps) {
  const t = useTranslations("userBubble");
  const c = useTranslations("common");

  return (
    <>
      <select
        value={storageBackend}
        onChange={(e) =>
          onChangeStorageBackend(e.target.value as "local" | "supabase")
        }
        className={inputClassName}
      >
        <option value="local">{t("storageLocal")}</option>
        <option value="supabase">{t("storageSupabase")}</option>
      </select>
      <p className="text-muted-foreground">
        {t("storageUsed", { size: formattedStorage })} · IndexedDB / JSON
      </p>
      {clearStep === 0 ? (
        <button
          type="button"
          onClick={onAdvanceClearStep}
          className="flex h-7 w-full items-center justify-center gap-2 rounded-ctl border border-border bg-card px-2 text-semantic-danger hover:bg-accent"
        >
          <Trash2 className="h-3.5 w-3.5" />
          <span>{t("clearDataBtn")}</span>
        </button>
      ) : (
        <div className="space-y-2 rounded-ctl border border-semantic-danger/40 bg-card p-2">
          <p className="font-medium text-semantic-danger">
            {clearStep === 1 ? t("clearStep1") : t("clearStep2")}
          </p>
          <div className="flex gap-2">
            <button
              type="button"
              onClick={onResetClearStep}
              className="h-6 flex-1 rounded-ctl border border-border px-2 text-foreground"
            >
              {t("clearCancel")}
            </button>
            <button
              type="button"
              onClick={onAdvanceClearStep}
              className="h-6 flex-1 rounded-ctl bg-semantic-danger px-2 font-medium text-white"
            >
              {c("confirm")}
            </button>
          </div>
        </div>
      )}
    </>
  );
}
