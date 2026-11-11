"use client";

import { useShallow } from "zustand/react/shallow";
import React from "react";
import { useTranslations } from "next-intl";
import { cn } from "@/lib/utils";
import {
  Sliders,
  FolderGit2,
  CalendarDays,
  Layers,
} from "@/components/ui/icons";
import { useStudioStore } from "@/store";
import type { RightPanelTab } from "@/store/slices/rightPanelSlice";
import { UserBubble } from "./UserBubble";

export function BottomNav() {
  const t = useTranslations("rightPanel.tabs");
  const {
    activeRightTab,
    setActiveRightTab,
    rightPanelCollapsed,
    setRightPanelCollapsed,
    mobileSheetOpen,
    setMobileSheetOpen,
  } = useStudioStore(
    useShallow((s) => ({
      activeRightTab: s.activeRightTab,
      setActiveRightTab: s.setActiveRightTab,
      rightPanelCollapsed: s.rightPanelCollapsed,
      setRightPanelCollapsed: s.setRightPanelCollapsed,
      mobileSheetOpen: s.mobileSheetOpen,
      setMobileSheetOpen: s.setMobileSheetOpen,
    }))
  );

  const items: Array<{
    tab: RightPanelTab;
    label: string;
    icon: typeof Sliders;
  }> = [
    { tab: "node", label: t("node"), icon: Sliders },
    { tab: "project", label: t("project"), icon: FolderGit2 },
    { tab: "publish", label: t("publish"), icon: CalendarDays },
    { tab: "jobs", label: t("jobs"), icon: Layers },
  ];

  return (
    <nav className="md:hidden fixed bottom-0 left-0 right-0 z-50 h-14 border-t border-border bg-background px-2 flex items-center justify-around select-none">
      {items.map((item) => {
        const Icon = item.icon;
        const isActive = (!rightPanelCollapsed || mobileSheetOpen) && activeRightTab === item.tab;

        return (
          <button
            key={item.tab}
            type="button"
            onClick={() => {
              if ((!rightPanelCollapsed || mobileSheetOpen) && activeRightTab === item.tab) {
                setMobileSheetOpen(false);
                setRightPanelCollapsed(true);
              } else {
                setActiveRightTab(item.tab);
                setMobileSheetOpen(true);
              }
            }}
            className={cn(
              "relative flex min-w-11 flex-col items-center justify-center px-2 py-1 text-footnote transition-colors",
              isActive
                ? "font-medium text-foreground after:absolute after:inset-x-2 after:-top-px after:h-0.5 after:bg-brand"
                : "text-muted-foreground hover:text-foreground"
            )}
          >
            <Icon className="mb-1 h-4 w-4" strokeWidth={1.5} />
            <span className="truncate max-w-[56px]">{item.label}</span>
          </button>
        );
      })}
      <UserBubble compact />
    </nav>
  );
}
