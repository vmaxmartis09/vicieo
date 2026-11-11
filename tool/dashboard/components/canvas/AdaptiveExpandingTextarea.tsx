"use client";

import React, { useState, useEffect, useRef, useCallback } from "react";
import {
  ClipboardPaste,
  X,
} from "@/components/ui/icons";
import { cn } from "@/lib/utils";
import { DynamicValueSelector } from "./DynamicValueSelector";

export interface AdaptiveExpandingTextareaProps {
  value: string;
  onChange: (val: string) => void;
  placeholder?: string;
  minHeight?: number;
  focusedHeight?: number;
  maxHeight?: number;
  className?: string;
  onPaste?: () => void | Promise<void>;
  showPasteBtn?: boolean;
  nodeId?: string;
  portName?: string;
  showDynamicPicker?: boolean;
}

export function AdaptiveExpandingTextarea({
  value,
  onChange,
  placeholder,
  minHeight = 64,
  focusedHeight = 130,
  maxHeight = 360,
  className,
  onPaste,
  showPasteBtn = false,
  nodeId,
  portName,
  showDynamicPicker = true,
}: AdaptiveExpandingTextareaProps) {
  const [isFocused, setIsFocused] = useState(false);
  const ref = useRef<HTMLTextAreaElement>(null);

  // Đảm bảo con trỏ nhảy xuống cuối cùng và cuộn tới cuối ngay khi click vào ô nhập
  const moveCursorToEnd = useCallback((el: HTMLTextAreaElement) => {
    const applyPos = () => {
      if (!el) return;
      const len = el.value.length;
      el.setSelectionRange(len, len);
      el.scrollTop = el.scrollHeight;
    };
    applyPos();
    requestAnimationFrame(applyPos);
    setTimeout(applyPos, 0);
    setTimeout(applyPos, 30);
    setTimeout(applyPos, 80);
    setTimeout(applyPos, 150);
  }, []);

  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    const prev = el.offsetHeight;
    el.style.height = "auto";
    const target = isFocused
      ? Math.min(Math.max(el.scrollHeight, 120), maxHeight)
      : minHeight;
    el.style.height = `${prev}px`;
    void el.offsetHeight;
    el.style.height = `${target}px`;
  }, [value, isFocused, minHeight, maxHeight]);

  const handleInsertText = (textToInsert: string) => {
    const el = ref.current;
    if (el) {
      const start = el.selectionStart || 0;
      const end = el.selectionEnd || 0;
      const next = value.slice(0, start) + textToInsert + value.slice(end);
      onChange(next);
      setTimeout(() => {
        el.focus();
        el.setSelectionRange(start + textToInsert.length, start + textToInsert.length);
      }, 50);
    } else {
      onChange(value ? `${value} ${textToInsert}` : textToInsert);
    }
  };

  return (
    <div className="relative group/ta w-full min-w-0">
      <textarea
        ref={ref}
        rows={1}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        onFocus={(e) => {
          setIsFocused(true);
          moveCursorToEnd(e.currentTarget);
        }}
        onBlur={() => {
          setIsFocused(false);
        }}
        onKeyDown={(e) => {
          if (e.key === "Escape") {
            e.currentTarget.blur();
            setIsFocused(false);
          }
        }}
        spellCheck={false}
        autoCorrect="off"
        autoCapitalize="off"
        placeholder={placeholder}
        className={cn(
          "nodrag nowheel block w-full resize-none rounded-ctl border px-3 py-2 pr-12 pb-6 text-footnote leading-relaxed text-foreground placeholder:text-muted-foreground/60 transition-[height,background-color,border-color,box-shadow] duration-200 ease-out focus:outline-none",
          isFocused
            ? "border-brand bg-card ring-1 ring-brand/30 overflow-y-auto"
            : "border-border/80 bg-card/60 hover:border-foreground/40 overflow-hidden",
          className
        )}
      />
      {/* Cụm công cụ trên: Chọn nguồn động { } + Dán */}
      <div className="absolute right-2 top-2 flex items-center gap-1 z-10">
        {showDynamicPicker && (
          <DynamicValueSelector
            nodeId={nodeId}
            portName={portName}
            currentValue={value}
            onSelectValue={(val) => {
              onChange(val);
              setTimeout(() => {
                if (ref.current) {
                  ref.current.focus();
                  moveCursorToEnd(ref.current);
                }
              }, 50);
            }}
            onInsertText={handleInsertText}
          />
        )}
        {showPasteBtn && isFocused && (
          <button
            type="button"
            onMouseDown={(e) => e.preventDefault()}
            onClick={onPaste}
            title="Dán từ Clipboard"
            className="nodrag flex h-6 w-6 items-center justify-center rounded-xs bg-muted/80 text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
          >
            <ClipboardPaste className="h-3 w-3" />
          </button>
        )}
      </div>

      {/* Nút clear text tách biệt nằm góc dưới bên phải để tránh nhấn nhầm khi bấm nút { } ở trên */}
      {Boolean(value) && (
        <button
          type="button"
          onMouseDown={(e) => e.preventDefault()}
          onClick={() => {
            onChange("");
            ref.current?.focus();
          }}
          title="Xoá toàn bộ nội dung"
          aria-label="Xoá nội dung"
          className="nodrag absolute right-2 bottom-1.5 z-10 flex h-5 w-5 items-center justify-center rounded-xs bg-muted/80 text-muted-foreground transition-colors hover:bg-destructive/10 hover:text-destructive cursor-pointer"
        >
          <X className="h-3 w-3" />
        </button>
      )}
    </div>
  );
}
