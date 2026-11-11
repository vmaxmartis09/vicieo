import type React from "react";
import type { ProviderKeyId, UserProfile } from "@/store/slices/userSettingsSlice";

export type AspectRatio = "9:16" | "16:9" | "1:1" | "4:5";

export type UserBubblePlacement = "above" | "below" | "aboveLeft" | "beside";

export interface UserBubbleProps {
  compact?: boolean;
  placement?: UserBubblePlacement;
}

export interface SettingRowItem {
  id: string;
  icon: React.ReactNode;
  label: string;
  summary: string;
  detail: React.ReactNode;
}

export interface UserBubblePopoverProps {
  open: boolean;
  compact: boolean;
  merged: boolean;
  placement: UserBubblePlacement;
  section: string | null;
  onChangeSection: (s: string | null) => void;
  popoverRef: React.RefObject<HTMLDivElement | null>;
  fileInputRef: React.RefObject<HTMLInputElement | null>;
  onAvatarUpload: (e: React.ChangeEvent<HTMLInputElement>) => void;
  userProfile: UserProfile;
  onUpdateProfile: (u: Partial<UserProfile>) => void;
  aspectRatio: AspectRatio;
  onChangeAspectRatio: (r: AspectRatio) => void;
  isMockMode: boolean;
  onChangeMockMode: (m: boolean) => void;
  perRunSpendingLimitUsd: number;
  onChangePerRunLimit: (v: number) => void;
  dailySpendingLimitUsd: number;
  onChangeDailyLimit: (v: number) => void;
  apiKeyStatus: Record<ProviderKeyId, { configured: boolean; updatedAt?: string | null }>;
  editingKeyProvider: ProviderKeyId | null;
  keyInputValue: string;
  onStartEditKey: (p: ProviderKeyId) => void;
  onChangeKeyValue: (v: string) => void;
  onCancelEditKey: () => void;
  onSaveKey: () => void;
  storageBackend: "local" | "supabase";
  onChangeStorageBackend: (b: "local" | "supabase") => void;
  storageUsedBytes: number;
  clearStep: 0 | 1 | 2;
  onResetClearStep: () => void;
  onAdvanceClearStep: () => void;
}
