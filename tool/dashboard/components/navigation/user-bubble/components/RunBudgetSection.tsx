"use client";

import React from "react";
import { useTranslations } from "next-intl";
import { SegmentedControl } from "./SegmentedControl";

interface RunBudgetSectionProps {
  isMockMode: boolean;
  onChangeMockMode: (isMock: boolean) => void;
  perRunSpendingLimitUsd: number;
  onChangePerRunLimit: (v: number) => void;
  dailySpendingLimitUsd: number;
  onChangeDailyLimit: (v: number) => void;
  inputClassName: string;
}

export function RunBudgetSection({
  isMockMode,
  onChangeMockMode,
  perRunSpendingLimitUsd,
  onChangePerRunLimit,
  dailySpendingLimitUsd,
  onChangeDailyLimit,
  inputClassName,
}: RunBudgetSectionProps) {
  const t = useTranslations("userBubble");
  return (
    <>
      <SegmentedControl
        items={[
          { id: "mock" as const, label: t("mockModeOn") },
          { id: "live" as const, label: t("mockModeOff") },
        ]}
        value={isMockMode ? "mock" : "live"}
        onChange={(v) => onChangeMockMode(v === "mock")}
      />
      <div className="grid grid-cols-2 gap-2">
        <label className="space-y-1">
          <span className="block text-muted-foreground">{t("perRunLimit")}</span>
          <input
            type="number"
            step="0.5"
            min="0.1"
            value={perRunSpendingLimitUsd}
            onChange={(e) => onChangePerRunLimit(Number(e.target.value) || 10.0)}
            className={inputClassName}
          />
        </label>
        <label className="space-y-1">
          <span className="block text-muted-foreground">{t("dailyLimit")}</span>
          <input
            type="number"
            step="1"
            min="1"
            value={dailySpendingLimitUsd}
            onChange={(e) => onChangeDailyLimit(Number(e.target.value) || 50.0)}
            className={inputClassName}
          />
        </label>
      </div>
    </>
  );
}
