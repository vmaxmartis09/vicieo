"use client";

import React, { useState } from "react";
import { useTranslations } from "next-intl";
import {
  Bot,
  Copy,
  Check,
  Undo2,
} from "@/components/ui/icons";
import { AiLoading } from "@/components/ui/AiLoading";
import type { ChatMessage } from "@/store/slices/chatSlice";

interface ChatMessageListProps {
  messages: ChatMessage[];
  sending: boolean;
  vi: boolean;
  onUndoAction: (msgId: string, snapshot?: any) => void;
  onViewOnCanvas: (targetNodeId: string) => void;
  endRef: React.RefObject<HTMLDivElement | null>;
}

export const ChatMessageList = React.memo(function ChatMessageList({
  messages,
  sending,
  vi,
  onUndoAction,
  onViewOnCanvas,
  endRef,
}: ChatMessageListProps) {
  const t = useTranslations("chatTab");
  const [copiedId, setCopiedId] = useState<string | null>(null);

  const handleCopy = (id: string, text: string) => {
    if (typeof navigator !== "undefined" && navigator.clipboard) {
      navigator.clipboard.writeText(text);
      setCopiedId(id);
      setTimeout(() => setCopiedId(null), 1200);
    }
  };

  return (
    <div className="scroll-quiet max-h-[28vh] space-y-2 overflow-y-auto border-b border-border bg-background p-3">
      {messages.length === 0 && (
        <div className="rounded-card border border-dashed border-input bg-card-elevated p-3 text-muted-foreground">
          <div className="flex items-center gap-2 text-foreground font-semibold">
            <Bot className="h-3.5 w-3.5 text-brand" />
            <span>{t("assistantTitle")}</span>
          </div>
          <p className="mt-1 leading-relaxed text-xs">{t("assistantSub")}</p>
        </div>
      )}

      {messages.map((m) => (
        <div
          key={m.id}
          className={
            m.role === "user"
              ? "group ml-8 rounded-card rounded-br-xs border border-border/60 bg-accent/45 px-3 py-2 text-foreground"
              : "group mr-4 space-y-2 rounded-card rounded-bl-xs border border-border/60 bg-card px-3 py-2 text-foreground"
          }
        >
          <p className="whitespace-pre-wrap leading-relaxed text-xs">{m.text}</p>
          <button
            type="button"
            aria-label={vi ? "Sao chép tin nhắn" : "Copy message"}
            onClick={() => handleCopy(m.id, m.text)}
            className="hidden h-6 items-center gap-1 rounded-ctl px-2 text-xs text-muted-foreground hover:bg-accent hover:text-foreground group-hover:inline-flex focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring transition-colors"
          >
            {copiedId === m.id ? (
              <Check className="h-3 w-3 text-brand" />
            ) : (
              <Copy className="h-3 w-3" />
            )}
            <span>
              {copiedId === m.id ? (vi ? "Đã chép" : "Copied") : (vi ? "Chép" : "Copy")}
            </span>
          </button>

          {m.actionCard && (
            <div className="space-y-1 rounded-card border border-border/80 bg-muted/40 p-2">
              <p className="text-xs font-medium text-foreground">
                {vi ? m.actionCard.summaryVi : m.actionCard.summaryEn}
              </p>
              <div className="flex items-center gap-2">
                {m.actionCard.undone ? (
                  <span className="rounded-full bg-accent px-2 py-1 text-xs text-muted-foreground">
                    {t("undoneBadge")}
                  </span>
                ) : (
                  m.actionCard.snapshotBefore && (
                    <button
                      type="button"
                      onClick={() => onUndoAction(m.id, m.actionCard?.snapshotBefore)}
                      className="inline-flex h-7 items-center gap-1 rounded-full border border-border bg-card px-2 text-xs font-medium text-foreground hover:bg-accent focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring transition-colors"
                    >
                      <Undo2 className="h-3 w-3" />
                      <span>{t("undoActionBtn")}</span>
                    </button>
                  )
                )}
                {m.actionCard.targetNodeId && (
                  <button
                    type="button"
                    onClick={() => onViewOnCanvas(m.actionCard!.targetNodeId!)}
                    className="inline-flex h-7 items-center rounded-full border border-border bg-card px-2 text-xs font-medium text-foreground hover:bg-accent focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring transition-colors"
                  >
                    {t("viewOnCanvasBtn")}
                  </button>
                )}
              </div>
            </div>
          )}
        </div>
      ))}

      {sending && (
        <div className="mr-4 rounded-card rounded-bl-xs border border-border/60 bg-card p-3">
          <AiLoading lines={2} />
        </div>
      )}
      <div ref={endRef} />
    </div>
  );
});
