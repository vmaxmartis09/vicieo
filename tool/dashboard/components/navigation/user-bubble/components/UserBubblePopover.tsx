"use client";

import React from "react";
import { AnimatePresence, motion } from "framer-motion";
import { useLocale, useTranslations } from "next-intl";
import { usePathname, useRouter } from "@/i18n/routing";
import { useTheme } from "@/components/theme/ThemeProvider";
import { cn } from "@/lib/utils";
import {
  Database,
  DollarSign,
  Globe,
  Key,
  Laptop,
  LaptopNode,
  Maximize2,
  Moon,
  MoonNode,
  MorphIcon,
  Sun,
  SunNode,
} from "@/components/ui/icons";
import { ASPECT_RATIO_OPTIONS, PROVIDERS } from "../constants";
import type { SettingRowItem, UserBubblePopoverProps } from "../types";
import { formatStorageKb } from "../utils/storageUtils";
import { ApiKeysSection } from "./ApiKeysSection";
import { ProfileSection } from "./ProfileSection";
import { RunBudgetSection } from "./RunBudgetSection";
import { SegmentedControl } from "./SegmentedControl";
import { SettingRowAccordion } from "./SettingRowAccordion";
import { StorageSection } from "./StorageSection";

const INPUT_CLS =
  "h-7 w-full rounded-ctl border border-border bg-card px-2 text-footnote tnum text-foreground";

export function UserBubblePopover(props: UserBubblePopoverProps) {
  const {
    open, compact, merged, placement, section, onChangeSection, popoverRef, fileInputRef,
    onAvatarUpload, userProfile, onUpdateProfile, aspectRatio, onChangeAspectRatio,
    isMockMode, onChangeMockMode, perRunSpendingLimitUsd, onChangePerRunLimit,
    dailySpendingLimitUsd, onChangeDailyLimit, apiKeyStatus, editingKeyProvider,
    keyInputValue, onStartEditKey, onChangeKeyValue, onCancelEditKey, onSaveKey,
    storageBackend, onChangeStorageBackend, storageUsedBytes, clearStep,
    onResetClearStep, onAdvanceClearStep,
  } = props;

  const t = useTranslations("userBubble");
  const locale = useLocale() as "vi" | "en";
  const pathname = usePathname();
  const router = useRouter();
  const { theme, setTheme } = useTheme();

  const keysConfigured = PROVIDERS.filter((p) => apiKeyStatus[p.id]?.configured).length;
  const formattedStorage = formatStorageKb(storageUsedBytes);
  const currentThemeNode = theme === "dark" ? MoonNode : theme === "light" ? SunNode : LaptopNode;

  const rows: SettingRowItem[] = [
    {
      id: "language",
      icon: <Globe className="h-4 w-4" />,
      label: t("languageLabel"),
      summary: locale.toUpperCase(),
      detail: (
        <SegmentedControl
          items={[
            { id: "vi" as const, label: "Tiếng Việt" },
            { id: "en" as const, label: "English" },
          ]}
          value={locale}
          onChange={(l) => router.replace(pathname, { locale: l })}
        />
      ),
    },
    {
      id: "theme",
      icon: <MorphIcon icon={currentThemeNode} spring="snappy" className="h-4 w-4" />,
      label: t("themeLabel"),
      summary: theme === "light" ? t("themeLight") : theme === "dark" ? t("themeDark") : t("themeSystem"),
      detail: (
        <SegmentedControl
          items={[
            { id: "light" as const, label: <span className="flex items-center gap-1"><Sun className="h-3.5 w-3.5" />{t("themeLight")}</span> },
            { id: "dark" as const, label: <span className="flex items-center gap-1"><Moon className="h-3.5 w-3.5" />{t("themeDark")}</span> },
            { id: "system" as const, label: <span className="flex items-center gap-1"><Laptop className="h-3.5 w-3.5" />{t("themeSystem")}</span> },
          ]}
          value={theme as "light" | "dark" | "system"}
          onChange={(v) => setTheme(v)}
        />
      ),
    },
    {
      id: "aspect",
      icon: <Maximize2 className="h-4 w-4" />,
      label: "Tỉ lệ khung hình",
      summary: aspectRatio,
      detail: (
        <SegmentedControl
          items={ASPECT_RATIO_OPTIONS.map((r) => ({ id: r, label: r }))}
          value={aspectRatio}
          onChange={onChangeAspectRatio}
        />
      ),
    },
    {
      id: "run",
      icon: <DollarSign className="h-4 w-4" />,
      label: t("spendingTitle"),
      summary: `${isMockMode ? "Mock" : "Live"} · $${perRunSpendingLimitUsd}/${dailySpendingLimitUsd}`,
      detail: (
        <RunBudgetSection
          isMockMode={isMockMode}
          onChangeMockMode={onChangeMockMode}
          perRunSpendingLimitUsd={perRunSpendingLimitUsd}
          onChangePerRunLimit={onChangePerRunLimit}
          dailySpendingLimitUsd={dailySpendingLimitUsd}
          onChangeDailyLimit={onChangeDailyLimit}
          inputClassName={INPUT_CLS}
        />
      ),
    },
    {
      id: "keys",
      icon: <Key className="h-4 w-4" />,
      label: t("apiKeysTitle"),
      summary: `${keysConfigured}/${PROVIDERS.length}`,
      detail: (
        <ApiKeysSection
          apiKeyStatus={apiKeyStatus}
          editingKeyProvider={editingKeyProvider}
          keyInputValue={keyInputValue}
          onStartEditKey={onStartEditKey}
          onChangeKeyValue={onChangeKeyValue}
          onCancelEdit={onCancelEditKey}
          onSaveKey={onSaveKey}
        />
      ),
    },
    {
      id: "storage",
      icon: <Database className="h-4 w-4" />,
      label: t("storageTitle"),
      summary: formattedStorage,
      detail: (
        <StorageSection
          storageBackend={storageBackend}
          onChangeStorageBackend={onChangeStorageBackend}
          formattedStorage={formattedStorage}
          clearStep={clearStep}
          onResetClearStep={onResetClearStep}
          onAdvanceClearStep={onAdvanceClearStep}
          inputClassName={INPUT_CLS}
        />
      ),
    },
  ];

  return (
    <AnimatePresence>
      {open && (
        <motion.div
          ref={popoverRef}
          role="dialog"
          aria-modal="true"
          aria-label={t("title")}
          initial={merged ? { height: 0, opacity: 0 } : { opacity: 0, y: 8, scale: 0.97 }}
          animate={merged ? { height: "auto", opacity: 1 } : { opacity: 1, y: 0, scale: 1 }}
          exit={merged ? { height: 0, opacity: 0 } : { opacity: 0, y: 8, scale: 0.97 }}
          transition={{ duration: merged ? 0.24 : 0.16, ease: "easeOut" }}
          className={cn(
            merged
              ? "w-[min(316px,calc(100vw-24px))] max-h-[70dvh] overflow-y-auto scroll-quiet p-2 text-popover-foreground"
              : "fixed md:absolute z-50 w-[min(300px,calc(100vw-24px))] max-h-[82dvh] overflow-y-auto scroll-quiet rounded-card border border-border bg-popover p-2 text-popover-foreground",
            merged
              ? ""
              : placement === "below"
              ? "right-0 top-11"
              : placement === "aboveLeft"
              ? "bottom-11 left-0"
              : compact
              ? "bottom-16 right-3"
              : "bottom-11 left-0"
          )}
        >
          <div className="flex items-center justify-between px-2 pb-2 pt-1">
            <h2 className="text-callout font-semibold">{t("title")}</h2>
          </div>
          <input ref={fileInputRef} type="file" accept="image/*" onChange={onAvatarUpload} className="hidden" />

          <ul className="space-y-1">
            {rows.map((row) => (
              <SettingRowAccordion
                key={row.id}
                row={row}
                isOpen={section === row.id}
                onToggle={() => onChangeSection(section === row.id ? null : row.id)}
              />
            ))}
          </ul>

          <AnimatePresence initial={false}>
            {section === "profile" && (
              <motion.div
                key="profile-editor"
                initial={{ height: 0, opacity: 0 }}
                animate={{ height: "auto", opacity: 1 }}
                exit={{ height: 0, opacity: 0 }}
                transition={{ duration: 0.18 }}
                className="overflow-hidden"
              >
                <div className="mt-1 space-y-2 border-t border-border px-2 pb-1 pt-3 text-xs">
                  <ProfileSection
                    userProfile={userProfile}
                    onUpdateProfile={onUpdateProfile}
                    onClickUpload={() => fileInputRef.current?.click()}
                    inputClassName={INPUT_CLS}
                  />
                </div>
              </motion.div>
            )}
          </AnimatePresence>
        </motion.div>
      )}
    </AnimatePresence>
  );
}
