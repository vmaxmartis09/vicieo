"use client";

import React from "react";
import { useTranslations } from "next-intl";

interface BudgetSummaryCardProps {
  totalSpentUsd: number;
  dailySpendingLimitUsd: number;
  pctDaily: number;
}

export function BudgetSummaryCard({
  totalSpentUsd,
  dailySpendingLimitUsd,
  pctDaily,
}: BudgetSummaryCardProps) {
  const t = useTranslations("jobsTab");

  return (
    <section className="rounded-card border border-border bg-card p-4 space-y-2">
      <h3 className="font-semibold text-foreground flex items-center justify-between">
        <span>{t("budgetTitle")}</span>
        <span className="text-caption font-mono text-muted-foreground">
          Trần: ${dailySpendingLimitUsd.toFixed(2)} USD
        </span>
      </h3>
      <div className="space-y-1 text-footnote tnum">
        <div className="flex justify-between">
          <span className="text-muted-foreground">
            {t("spentToday", { spent: totalSpentUsd.toFixed(2) })}
          </span>
          <span className="font-medium text-foreground">{pctDaily}%</span>
        </div>
        <div className="h-1.5 w-full overflow-hidden rounded-xs bg-accent">
          <div
            style={{ width: `${pctDaily}%` }}
            className="h-full bg-primary transition-[width]"
          />
        </div>
      </div>
    </section>
  );
}
