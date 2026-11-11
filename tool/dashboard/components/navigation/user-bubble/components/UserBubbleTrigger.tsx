"use client";

import React from "react";
import { AnimatePresence, motion } from "framer-motion";
import { useTranslations } from "next-intl";
import type { CoreHealthState, UserProfile } from "@/store/slices/userSettingsSlice";
import { cn } from "@/lib/utils";

interface UserBubbleTriggerProps {
  compact: boolean;
  merged: boolean;
  open: boolean;
  section: string | null;
  initials: string;
  userProfile: UserProfile;
  coreHealth: CoreHealthState;
  triggerRef: React.RefObject<HTMLButtonElement | null>;
  onToggleOpen: () => void;
  onToggleProfileSection: () => void;
}

export function UserBubbleTrigger({
  compact,
  merged,
  open,
  section,
  initials,
  userProfile,
  coreHealth,
  triggerRef,
  onToggleOpen,
  onToggleProfileSection,
}: UserBubbleTriggerProps) {
  const t = useTranslations("userBubble");
  const Row: any = merged ? "div" : React.Fragment;
  const rowProps = merged
    ? { className: cn("flex shrink-0 items-center", open && "border-t border-border") }
    : {};

  return (
    <Row {...rowProps}>
      <button
        ref={triggerRef}
        type="button"
        onClick={onToggleOpen}
        aria-haspopup="dialog"
        aria-expanded={open}
        aria-label={t("triggerLabel")}
        className={cn(
          "flex shrink-0 items-center gap-2 rounded-full text-left transition-colors hover:bg-accent",
          !merged && "border border-border bg-card",
          compact ? "h-12 w-12 justify-center" : "h-9 w-full px-2 pr-3"
        )}
      >
        {userProfile.avatarDataUrl ? (
          // eslint-disable-next-line @next/next/no-img-element
          <img
            decoding="async"
            loading="lazy"
            src={userProfile.avatarDataUrl}
            alt={userProfile.name}
            className={cn(
              "shrink-0 rounded-full object-cover object-top",
              compact ? "h-10 w-10" : "h-7 w-7"
            )}
          />
        ) : (
          <span
            className={cn(
              "flex shrink-0 items-center justify-center rounded-full bg-accent font-semibold text-foreground",
              compact ? "h-10 w-10 text-callout" : "h-7 w-7 text-footnote"
            )}
          >
            {initials}
          </span>
        )}
        {!compact && (
          <div className="min-w-0 flex-1">
            <p className="truncate text-footnote font-medium text-foreground">
              {userProfile.name}
            </p>
            <p className="truncate text-body text-muted-foreground">
              {coreHealth.online ? `Core v${coreHealth.version}` : "Mock UI"}
            </p>
          </div>
        )}
      </button>
      {merged && (
        <AnimatePresence initial={false}>
          {open && (
            <motion.button
              key="infobar"
              type="button"
              title="Sửa hồ sơ"
              aria-label="Sửa hồ sơ"
              aria-expanded={section === "profile"}
              onClick={onToggleProfileSection}
              initial={{ opacity: 0 }}
              animate={{ opacity: 1, transition: { delay: 0.1, duration: 0.15 } }}
              exit={{ opacity: 0, transition: { duration: 0.05 } }}
              className="mr-2 min-w-0 flex-1 rounded-card px-2 py-2 text-left leading-tight transition-colors hover:bg-accent"
            >
              <p className="truncate text-body font-medium text-foreground">
                {userProfile.name}
              </p>
              <p className="flex items-center gap-2 truncate text-footnote text-muted-foreground">
                <span
                  className={cn(
                    "h-1.5 w-1.5 shrink-0 rounded-full",
                    coreHealth.online ? "bg-semantic-success" : "bg-muted-foreground"
                  )}
                />
                {coreHealth.online ? `Core v${coreHealth.version}` : "Core offline"}
              </p>
            </motion.button>
          )}
        </AnimatePresence>
      )}
    </Row>
  );
}
