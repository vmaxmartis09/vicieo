"use client";

import { useCallback, useEffect, useState, type RefObject } from "react";
import type { ChatAttachment } from "../types";

export function useComposerMenus(
  containerRef: RefObject<HTMLDivElement | null>,
  textareaRef: RefObject<HTMLTextAreaElement | null>,
  input: string,
  autoFocus: boolean,
  onAddAttachment?: (att: ChatAttachment) => void
) {
  const [modelMenuOpen, setModelMenuOpen] = useState(false);
  const [modelTab, setModelTab] = useState<"agent" | "media">("agent");
  const [plusMenuOpen, setPlusMenuOpen] = useState(false);
  const [skillMenuOpen, setSkillMenuOpen] = useState(false);
  const [modeMenuOpen, setModeMenuOpen] = useState(false);

  useEffect(() => {
    const el = textareaRef.current;
    if (!el) return;
    if (!input) {
      el.style.height = "";
      return;
    }
    el.style.height = "auto";
    el.style.height = `${Math.min(Math.max(el.scrollHeight, 40), 120)}px`;
  }, [input, textareaRef]);

  useEffect(() => {
    if (autoFocus) textareaRef.current?.focus();
  }, [autoFocus, textareaRef]);

  const closeAllMenus = useCallback(() => {
    setModelMenuOpen(false);
    setPlusMenuOpen(false);
    setSkillMenuOpen(false);
    setModeMenuOpen(false);
  }, []);

  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        closeAllMenus();
      }
    };
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape" && (modelMenuOpen || plusMenuOpen || skillMenuOpen || modeMenuOpen)) {
        e.stopPropagation();
        closeAllMenus();
      }
    };
    document.addEventListener("mousedown", handleClickOutside);
    window.addEventListener("keydown", handleKeyDown, true);
    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
      window.removeEventListener("keydown", handleKeyDown, true);
    };
  }, [containerRef, closeAllMenus, modelMenuOpen, plusMenuOpen, skillMenuOpen, modeMenuOpen]);

  const toggleMenu = useCallback((target: "plus" | "skill" | "model" | "mode") => {
    setPlusMenuOpen((v) => (target === "plus" ? !v : false));
    setSkillMenuOpen((v) => (target === "skill" ? !v : false));
    setModelMenuOpen((v) => (target === "model" ? !v : false));
    setModeMenuOpen((v) => (target === "mode" ? !v : false));
  }, []);

  const handleFileUpload = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const files = Array.from(e.target.files || []);
      if (!files.length || !onAddAttachment) return;
      for (const file of files) {
        const reader = new FileReader();
        reader.onload = () => {
          if (typeof reader.result === "string") {
            onAddAttachment({
              id: `att_up_${Date.now().toString(36)}_${Math.random().toString(36).slice(2, 5)}`,
              name: file.name.replace(/\.[^.]+$/, ""),
              url: reader.result,
              source: "upload",
            });
          }
        };
        reader.readAsDataURL(file);
      }
      e.target.value = "";
      setPlusMenuOpen(false);
    },
    [onAddAttachment]
  );

  return {
    modelMenuOpen,
    setModelMenuOpen,
    modelTab,
    setModelTab,
    plusMenuOpen,
    setPlusMenuOpen,
    skillMenuOpen,
    setSkillMenuOpen,
    modeMenuOpen,
    setModeMenuOpen,
    closeAllMenus,
    toggleMenu,
    handleFileUpload,
  };
}
