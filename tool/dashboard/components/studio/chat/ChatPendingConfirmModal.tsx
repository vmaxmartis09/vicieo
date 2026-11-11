"use client";

import React from "react";
import { useTranslations } from "next-intl";
import type { PendingChatConfirmation } from "@/store/slices/chatSlice";

interface ChatPendingConfirmModalProps {
  confirmation: PendingChatConfirmation;
  locale: "vi" | "en";
  onCancel: () => void;
  onProceed: () => void;
}

export const ChatPendingConfirmModal = React.memo(function ChatPendingConfirmModal({
  confirmation,
  locale,
  onCancel,
  onProceed,
}: ChatPendingConfirmModalProps) {
  const t = useTranslations("chatTab");
  const c = useTranslations("common");

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-label={t("confirmModalTitle")}
      className="fixed inset-0 z-50 flex items-center justify-center bg-foreground/25 p-4 backdrop-blur-xs"
    >
      <div className="w-full max-w-md rounded-panel border border-border bg-popover p-4 text-popover-foreground shadow-xl space-y-3">
        <h4 className="text-callout font-semibold">
          {locale === "vi" ? confirmation.titleVi : confirmation.titleEn}
        </h4>
        <p className="text-footnote text-muted-foreground leading-relaxed tnum">
          {locale === "vi" ? confirmation.detailVi : confirmation.detailEn}
        </p>
        <div className="flex justify-end gap-2 pt-1">
          <button
            type="button"
            onClick={onCancel}
            className="h-8 rounded-ctl border border-border bg-card px-3 text-footnote text-foreground hover:bg-accent focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring transition-colors"
          >
            {c("cancel")}
          </button>
          <button
            type="button"
            onClick={onProceed}
            className="h-8 rounded-ctl bg-primary px-3 text-footnote font-medium text-primary-foreground hover:bg-primary/90 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring transition-colors"
          >
            {t("confirmProceed")}
          </button>
        </div>
      </div>
    </div>
  );
});
