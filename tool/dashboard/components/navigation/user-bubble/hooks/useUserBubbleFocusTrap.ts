import { useEffect, useRef } from "react";
import type React from "react";
import type { ProviderKeyId } from "@/store/slices/userSettingsSlice";

interface UseUserBubbleFocusTrapParams {
  open: boolean;
  editingKeyProvider: ProviderKeyId | null;
  rootRef: React.RefObject<HTMLDivElement | null>;
  triggerRef: React.RefObject<HTMLButtonElement | null>;
  popoverRef: React.RefObject<HTMLDivElement | null>;
  onClose: () => void;
  onCancelEditKey: () => void;
  onResetClearStep: () => void;
}

export function useUserBubbleFocusTrap({
  open,
  editingKeyProvider,
  rootRef,
  triggerRef,
  popoverRef,
  onClose,
  onCancelEditKey,
  onResetClearStep,
}: UseUserBubbleFocusTrapParams) {
  const callbacksRef = useRef({ onClose, onCancelEditKey, onResetClearStep });
  callbacksRef.current = { onClose, onCancelEditKey, onResetClearStep };

  useEffect(() => {
    if (!open) return;
    const prevFocused = document.activeElement as HTMLElement | null;
    const focusables = popoverRef.current?.querySelectorAll<HTMLElement>(
      'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
    );
    focusables?.[0]?.focus();

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        if (editingKeyProvider) {
          callbacksRef.current.onCancelEditKey();
        } else {
          callbacksRef.current.onClose();
          triggerRef.current?.focus();
        }
        return;
      }
      if (e.key === "Tab" && popoverRef.current) {
        const nodes = popoverRef.current.querySelectorAll<HTMLElement>(
          'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
        );
        if (nodes.length === 0) return;
        const first = nodes[0];
        const last = nodes[nodes.length - 1];
        if (e.shiftKey && document.activeElement === first) {
          e.preventDefault();
          last.focus();
        } else if (!e.shiftKey && document.activeElement === last) {
          e.preventDefault();
          first.focus();
        }
      }
    };

    const handleMouseDown = (e: Event) => {
      const target = e.target as Node;
      if (rootRef.current && !rootRef.current.contains(target)) {
        callbacksRef.current.onClose();
        callbacksRef.current.onCancelEditKey();
        callbacksRef.current.onResetClearStep();
      }
    };

    document.addEventListener("keydown", handleKeyDown, true);
    document.addEventListener("pointerdown", handleMouseDown, true);
    return () => {
      document.removeEventListener("keydown", handleKeyDown, true);
      document.removeEventListener("pointerdown", handleMouseDown, true);
      prevFocused?.focus?.();
    };
  }, [open, editingKeyProvider, rootRef, triggerRef, popoverRef]);
}
