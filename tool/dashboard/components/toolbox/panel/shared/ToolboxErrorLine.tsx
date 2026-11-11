"use client";

import React from "react";
import { AlertTriangle, X } from "@/components/ui/icons";

export function ToolboxErrorLine({ text, onClose }: { text: string; onClose: () => void }) {
  return (
    <div className="flex items-start gap-2 rounded-lg border border-semantic-danger/30 bg-semantic-danger/5 p-2 text-caption text-semantic-danger">
      <AlertTriangle className="mt-1 h-3.5 w-3.5 shrink-0" />
      <span className="min-w-0 flex-1 break-words">{text}</span>
      <button type="button" aria-label="Đóng" onClick={onClose} className="shrink-0 active:scale-95">
        <X className="h-3.5 w-3.5" />
      </button>
    </div>
  );
}
