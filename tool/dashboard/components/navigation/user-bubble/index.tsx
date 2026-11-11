"use client";

import React, { useRef, useState } from "react";
import { motion } from "framer-motion";
import { useShallow } from "zustand/react/shallow";
import { useStudioStore, type StudioStore } from "@/store";
import { cn } from "@/lib/utils";
import { useUserBubbleFocusTrap } from "./hooks/useUserBubbleFocusTrap";
import { useUserBubbleSync } from "./hooks/useUserBubbleSync";
import { BubbleErrorBoundary } from "./components/BubbleErrorBoundary";
import { UserBubblePopover } from "./components/UserBubblePopover";
import { UserBubbleTrigger } from "./components/UserBubbleTrigger";
import type { UserBubbleProps } from "./types";
import { getInitials } from "./utils/storageUtils";

export type { UserBubblePlacement, UserBubbleProps } from "./types";

function UserBubbleInner({ compact = false, placement = "above" }: UserBubbleProps) {
  const store = useStudioStore(
    useShallow((s: StudioStore) => ({
      aspectRatio: s.aspectRatio,
      setAspectRatio: s.setAspectRatio,
      isMockMode: s.isMockMode,
      setIsMockMode: s.setIsMockMode,
      userProfile: s.userProfile,
      setUserProfile: s.setUserProfile,
      perRunSpendingLimitUsd: s.perRunSpendingLimitUsd,
      setPerRunSpendingLimitUsd: s.setPerRunSpendingLimitUsd,
      dailySpendingLimitUsd: s.dailySpendingLimitUsd,
      setDailySpendingLimitUsd: s.setDailySpendingLimitUsd,
      setSpendingLimitUsd: s.setSpendingLimitUsd,
      storageBackend: s.storageBackend,
      setStorageBackend: s.setStorageBackend,
      storageUsedBytes: s.storageUsedBytes,
      setStorageUsedBytes: s.setStorageUsedBytes,
      apiKeyStatus: s.apiKeyStatus,
      setProviderKeyConfigured: s.setProviderKeyConfigured,
      coreHealth: s.coreHealth,
      setCoreHealth: s.setCoreHealth,
    }))
  );

  const [open, setOpen] = useState(false);
  const [section, setSection] = useState<string | null>(null);

  const rootRef = useRef<HTMLDivElement>(null);
  const triggerRef = useRef<HTMLButtonElement>(null);
  const popoverRef = useRef<HTMLDivElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const sync = useUserBubbleSync({
    open,
    setCoreHealth: store.setCoreHealth,
    setProviderKeyConfigured: store.setProviderKeyConfigured,
    setStorageUsedBytes: store.setStorageUsedBytes,
    setUserProfile: store.setUserProfile,
  });

  useUserBubbleFocusTrap({
    open,
    editingKeyProvider: sync.editingKeyProvider,
    rootRef,
    triggerRef,
    popoverRef,
    onClose: () => setOpen(false),
    onCancelEditKey: () => {
      sync.setEditingKeyProvider(null);
      sync.setKeyInputValue("");
    },
    onResetClearStep: () => sync.setClearStep(0),
  });

  const merged = placement === "beside";
  const Shell: any = merged ? motion.div : React.Fragment;
  const shellProps = merged
    ? {
        initial: false,
        animate: { width: open ? 316 : 48 },
        transition: { duration: 0.24, ease: [0.22, 1, 0.36, 1] },
        className: cn(
          "absolute bottom-0 left-0 flex flex-col-reverse overflow-clip border border-border bg-card shadow-lg",
          open ? "rounded-sheet" : "rounded-full"
        ),
      }
    : {};

  return (
    <div ref={rootRef} className={cn("relative", merged && "h-12 w-12")}>
      <Shell {...shellProps}>
        <UserBubbleTrigger
          compact={compact}
          merged={merged}
          open={open}
          section={section}
          initials={getInitials(store.userProfile.name)}
          userProfile={store.userProfile}
          coreHealth={store.coreHealth}
          triggerRef={triggerRef}
          onToggleOpen={() => {
            setOpen((v) => !v);
            sync.setClearStep(0);
            sync.setEditingKeyProvider(null);
          }}
          onToggleProfileSection={() =>
            setSection((v) => (v === "profile" ? null : "profile"))
          }
        />

        <UserBubblePopover
          open={open}
          compact={compact}
          merged={merged}
          placement={placement}
          section={section}
          onChangeSection={setSection}
          popoverRef={popoverRef}
          fileInputRef={fileInputRef}
          onAvatarUpload={sync.handleAvatarUpload}
          userProfile={store.userProfile}
          onUpdateProfile={store.setUserProfile}
          aspectRatio={store.aspectRatio}
          onChangeAspectRatio={store.setAspectRatio}
          isMockMode={store.isMockMode}
          onChangeMockMode={store.setIsMockMode}
          perRunSpendingLimitUsd={store.perRunSpendingLimitUsd}
          onChangePerRunLimit={store.setPerRunSpendingLimitUsd}
          dailySpendingLimitUsd={store.dailySpendingLimitUsd}
          onChangeDailyLimit={(v) => {
            store.setDailySpendingLimitUsd(v);
            store.setSpendingLimitUsd(v);
          }}
          apiKeyStatus={store.apiKeyStatus}
          editingKeyProvider={sync.editingKeyProvider}
          keyInputValue={sync.keyInputValue}
          onStartEditKey={(p) => {
            sync.setEditingKeyProvider(p);
            sync.setKeyInputValue("");
          }}
          onChangeKeyValue={sync.setKeyInputValue}
          onCancelEditKey={() => {
            sync.setEditingKeyProvider(null);
            sync.setKeyInputValue("");
          }}
          onSaveKey={sync.handleSaveApiKey}
          storageBackend={store.storageBackend}
          onChangeStorageBackend={store.setStorageBackend}
          storageUsedBytes={store.storageUsedBytes}
          clearStep={sync.clearStep}
          onResetClearStep={() => sync.setClearStep(0)}
          onAdvanceClearStep={sync.handleClearLocalData}
        />
      </Shell>
    </div>
  );
}

export function UserBubble(props: UserBubbleProps) {
  return (
    <BubbleErrorBoundary>
      <UserBubbleInner {...props} />
    </BubbleErrorBoundary>
  );
}
