"use client";

import React, { useState, useEffect, useRef } from "react";
import { createPortal } from "react-dom";
import { cn } from "@/lib/utils";
import { useStudioStore } from "@/store";
import { DynamicValueFlyout } from "./DynamicValueFlyout";

export interface DynamicValueSelectorProps {
  nodeId?: string;
  portName?: string;
  currentValue?: string;
  onSelectValue: (val: string) => void;
  onInsertText?: (text: string) => void;
  className?: string;
  buttonClassName?: string;
}

export function DynamicValueSelector({
  nodeId,
  portName,
  currentValue = "",
  onSelectValue,
  onInsertText,
  className,
  buttonClassName,
}: DynamicValueSelectorProps) {
  const [isOpen, setIsOpen] = useState(false);
  const triggerRef = useRef<HTMLButtonElement>(null);
  const [cardEl, setCardEl] = useState<HTMLElement | null>(null);
  const [topOffset, setTopOffset] = useState(0);

  // Calculate card position, elevate node zIndex and open to the right
  useEffect(() => {
    let nodeEl: HTMLElement | null = null;
    let card: HTMLElement | null = null;
    if (isOpen && triggerRef.current) {
      card = triggerRef.current.closest("[data-node-card]") as HTMLElement | null;
      nodeEl = triggerRef.current.closest(".react-flow__node") as HTMLElement | null;
      if (card) {
        setCardEl(card);
        card.style.zIndex = "2000";

        let top = 0;
        let el: HTMLElement | null = triggerRef.current;
        while (el && el !== card) {
          top += el.offsetTop;
          el = el.offsetParent as HTMLElement | null;
        }
        setTopOffset(top);
      }
      if (nodeEl) {
        nodeEl.style.zIndex = "2000";
      }
    }
    return () => {
      if (card) {
        card.style.zIndex = "";
      }
      if (nodeEl) {
        nodeEl.style.zIndex = "";
      }
    };
  }, [isOpen]);

  const flyout = (
    <DynamicValueFlyout
      nodeId={nodeId}
      portName={portName}
      currentValue={currentValue}
      topOffset={topOffset}
      triggerRef={triggerRef}
      onClose={() => setIsOpen(false)}
      onSelectValue={onSelectValue}
      onInsertText={onInsertText}
    />
  );

  return (
    <div className={cn("relative inline-block", className)}>
      <button
        ref={triggerRef}
        type="button"
        title="Chọn nguồn dữ liệu {nguồn.value}"
        aria-label="Chọn nguồn dữ liệu"
        aria-expanded={isOpen}
        onClick={(e) => {
          e.stopPropagation();
          if (nodeId) {
            useStudioStore.getState().setSelectedNodeId(nodeId);
          }
          setIsOpen((v) => !v);
        }}
        className={cn(
          "nodrag flex h-6 w-6 items-center justify-center rounded-ctl font-mono text-caption font-bold transition-all cursor-pointer",
          isOpen
            ? "bg-foreground text-background shadow-xs scale-105"
            : "bg-muted/80 text-muted-foreground hover:bg-muted hover:text-foreground",
          buttonClassName
        )}
      >
        {`{}`}
      </button>

      {isOpen && cardEl ? createPortal(flyout, cardEl) : isOpen ? flyout : null}
    </div>
  );
}
