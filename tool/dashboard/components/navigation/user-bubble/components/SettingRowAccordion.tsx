"use client";

import React from "react";
import { AnimatePresence, motion } from "framer-motion";
import { ChevronRight } from "@/components/ui/icons";
import { cn } from "@/lib/utils";
import type { SettingRowItem } from "../types";

interface SettingRowAccordionProps {
  row: SettingRowItem;
  isOpen: boolean;
  onToggle: () => void;
}

export function SettingRowAccordion({
  row,
  isOpen,
  onToggle,
}: SettingRowAccordionProps) {
  return (
    <li className="rounded-ctl">
      <button
        type="button"
        aria-expanded={isOpen}
        onClick={onToggle}
        className={cn(
          "flex w-full items-center gap-3 rounded-ctl px-2 py-2 text-left transition-colors hover:bg-accent",
          isOpen && "bg-accent"
        )}
      >
        <span className="flex h-6 w-6 shrink-0 items-center justify-center text-muted-foreground">
          {row.icon}
        </span>
        <span className="min-w-0 flex-1 truncate text-body font-medium text-foreground">
          {row.label}
        </span>
        <span className="max-w-[110px] truncate text-footnote text-muted-foreground tnum">
          {row.summary}
        </span>
        <ChevronRight
          className={cn(
            "h-3.5 w-3.5 shrink-0 text-muted-foreground transition-transform",
            isOpen && "rotate-90"
          )}
        />
      </button>
      <AnimatePresence initial={false}>
        {isOpen && (
          <motion.div
            initial={{ height: 0, opacity: 0 }}
            animate={{ height: "auto", opacity: 1 }}
            exit={{ height: 0, opacity: 0 }}
            transition={{ duration: 0.18 }}
            className="overflow-hidden"
          >
            <div className="space-y-2 px-2 pb-3 pt-2 text-footnote">
              {row.detail}
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </li>
  );
}
