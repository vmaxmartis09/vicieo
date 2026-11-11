"use client";

import React from "react";
import { cn } from "@/lib/utils";
import { Calendar, CalendarDays, Layers, X } from "../../icons";
import type { WizardMainTab } from "../types";

interface ModalHeaderProps {
  accountsCount: number;
  activeMainTab: WizardMainTab;
  onChangeTab: (tab: WizardMainTab) => void;
  onClose: () => void;
}

export function ModalHeader({
  accountsCount,
  activeMainTab,
  onChangeTab,
  onClose,
}: ModalHeaderProps) {
  return (
    <div className="flex items-center justify-between border-b border-border bg-muted/40 px-5 py-3 shrink-0">
      <div className="flex items-center gap-3">
        <div className="flex h-9 w-9 items-center justify-center rounded-xl bg-primary/10 text-primary">
          <CalendarDays className="h-5 w-5" />
        </div>
        <div>
          <h2 className="text-sm sm:text-base font-bold text-foreground flex items-center gap-2">
            Trung tâm Quản lý & Xuất bản Social
            <span className="text-caption font-normal text-muted-foreground bg-accent px-2 py-1 rounded-full border border-border">
              {accountsCount}/10 tài khoản
            </span>
          </h2>
          <p className="text-xs text-muted-foreground">
            Lập kế hoạch xuất bản đa kênh, tối ưu caption và kiểm soát an toàn trước khi đăng
          </p>
        </div>
      </div>

      <div className="flex items-center gap-3">
        {/* Tab switchers: Wizard Plan vs Calendar */}
        <div className="flex items-center rounded-xl border border-border bg-background p-1 text-xs">
          <button
            type="button"
            onClick={() => onChangeTab("plan")}
            className={cn(
              "flex items-center gap-2 rounded-lg px-3 py-1 font-semibold transition-all",
              activeMainTab === "plan"
                ? "bg-primary text-primary-foreground shadow-2xs"
                : "text-muted-foreground hover:text-foreground"
            )}
          >
            <Layers className="h-3.5 w-3.5" />
            <span>Kế hoạch đăng (5 bước)</span>
          </button>
          <button
            type="button"
            onClick={() => onChangeTab("calendar")}
            className={cn(
              "flex items-center gap-2 rounded-lg px-3 py-1 font-semibold transition-all",
              activeMainTab === "calendar"
                ? "bg-primary text-primary-foreground shadow-2xs"
                : "text-muted-foreground hover:text-foreground"
            )}
          >
            <Calendar className="h-3.5 w-3.5" />
            <span>Xem Lịch Đăng</span>
          </button>
        </div>

        <button
          type="button"
          onClick={onClose}
          className="flex h-8 w-8 items-center justify-center rounded-lg text-muted-foreground hover:bg-accent hover:text-foreground transition-colors"
        >
          <X className="h-5 w-5" />
        </button>
      </div>
    </div>
  );
}
