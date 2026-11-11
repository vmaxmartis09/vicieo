"use client";

import { useCallback, useEffect, useState, type RefObject } from "react";
import type { ChatAttachment, ComposerActionMode } from "@/components/studio/chat/SophisticatedChatInput";

export function useChatComposerState(dockRef: RefObject<HTMLDivElement | null>, endRef: RefObject<HTMLDivElement | null>, messagesCount: number) {
  const [input, setInput] = useState("");
  const [sending, setSending] = useState(false);
  const [projectNew, setProjectNew] = useState(false);
  const [expanded, setExpanded] = useState(false);
  const [showHistory, setShowHistory] = useState(false);
  const [shouldFocusInput, setShouldFocusInput] = useState(false);
  const [attachments, setAttachments] = useState<ChatAttachment[]>([]);
  const [craftTags, setCraftTags] = useState<string[]>([]);
  const [actionMode, setActionMode] = useState<ComposerActionMode>("chat");

  useEffect(() => {
    if (showHistory) endRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messagesCount, sending, showHistory, endRef]);

  useEffect(() => {
    if (!expanded) return;
    const onPointerDown = (e: MouseEvent) => {
      if (dockRef.current && !dockRef.current.contains(e.target as Node) && !input.trim() && attachments.length === 0) {
        setExpanded(false);
        setShowHistory(false);
      }
    };
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        setExpanded(false);
        setShowHistory(false);
      }
    };
    window.addEventListener("pointerdown", onPointerDown);
    window.addEventListener("keydown", onKeyDown);
    return () => {
      window.removeEventListener("pointerdown", onPointerDown);
      window.removeEventListener("keydown", onKeyDown);
    };
  }, [expanded, input, attachments.length, dockRef]);

  const addAttachmentUnique = useCallback((att: ChatAttachment) => {
    setAttachments((prev) => (prev.some((a) => a.id === att.id || a.url === att.url) ? prev : [...prev, att]));
  }, []);

  const removeAttachment = useCallback((id: string) => {
    setAttachments((prev) => prev.filter((a) => a.id !== id));
  }, []);

  const toggleCraftTag = useCallback((tag: string) => {
    setCraftTags((prev) => (prev.includes(tag) ? prev.filter((x) => x !== tag) : [...prev, tag]));
  }, []);

  return {
    input,
    setInput,
    sending,
    setSending,
    projectNew,
    setProjectNew,
    expanded,
    setExpanded,
    showHistory,
    setShowHistory,
    shouldFocusInput,
    setShouldFocusInput,
    attachments,
    setAttachments,
    craftTags,
    actionMode,
    setActionMode,
    addAttachmentUnique,
    removeAttachment,
    toggleCraftTag,
  };
}
