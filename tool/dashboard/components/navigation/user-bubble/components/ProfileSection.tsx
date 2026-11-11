"use client";

import React from "react";
import { useTranslations } from "next-intl";
import { Upload } from "@/components/ui/icons";
import type { UserProfile } from "@/store/slices/userSettingsSlice";

interface ProfileSectionProps {
  userProfile: UserProfile;
  onUpdateProfile: (updates: Partial<UserProfile>) => void;
  onClickUpload: () => void;
  inputClassName: string;
}

export function ProfileSection({
  userProfile,
  onUpdateProfile,
  onClickUpload,
  inputClassName,
}: ProfileSectionProps) {
  const t = useTranslations("userBubble");
  return (
    <>
      <input
        type="text"
        value={userProfile.name}
        onChange={(e) => onUpdateProfile({ name: e.target.value })}
        aria-label={t("title")}
        className={inputClassName}
      />
      <input
        type="email"
        value={userProfile.email}
        onChange={(e) => onUpdateProfile({ email: e.target.value })}
        className={inputClassName}
      />
      <button
        type="button"
        onClick={onClickUpload}
        className="flex h-7 items-center gap-2 rounded-ctl border border-border bg-card px-2 text-foreground hover:bg-accent"
      >
        <Upload className="h-3 w-3" />
        <span>{t("uploadAvatar")}</span>
      </button>
    </>
  );
}
