import { useEffect, useState } from "react";
import type React from "react";
import type { ProviderKeyId } from "@/store/slices/userSettingsSlice";
import { checkCoreHealth } from "@/lib/core-api";
import { CORE_URL } from "@/lib/repositories/core";
import { PROVIDERS } from "../constants";
import { estimateLocalStorageBytes } from "../utils/storageUtils";

interface UseUserBubbleSyncParams {
  open: boolean;
  setCoreHealth: (h: { online: boolean; version: string; repoRoot?: string; checkedAt?: string; outdated?: boolean }) => void;
  setProviderKeyConfigured: (provider: ProviderKeyId, configured: boolean, updatedAt?: string | null) => void;
  setStorageUsedBytes: (bytes: number) => void;
  setUserProfile: (updates: { avatarDataUrl?: string; name?: string; email?: string }) => void;
}

export function useUserBubbleSync({
  open,
  setCoreHealth,
  setProviderKeyConfigured,
  setStorageUsedBytes,
  setUserProfile,
}: UseUserBubbleSyncParams) {
  const [editingKeyProvider, setEditingKeyProvider] = useState<ProviderKeyId | null>(null);
  const [keyInputValue, setKeyInputValue] = useState("");
  const [clearStep, setClearStep] = useState<0 | 1 | 2>(0);

  useEffect(() => {
    if (!open) return;
    let active = true;
    checkCoreHealth().then((h) => {
      if (active) setCoreHealth({ ...h, checkedAt: new Date().toISOString() });
    });
    fetch(`${CORE_URL}/settings/api-keys`)
      .then((r) => (r.ok ? r.json() : null))
      .then((data) => {
        if (!active || !data?.providers) return;
        const list: Array<{ provider: string; configured?: boolean; updatedAt?: string | null }> = Array.isArray(
          data.providers
        )
          ? data.providers
          : Object.entries(data.providers).map(([provider, v]) => ({ provider, ...(v as object) }));
        for (const p of PROVIDERS) {
          const info = list.find((x) => x.provider === p.id);
          if (info) {
            setProviderKeyConfigured(p.id, Boolean(info.configured), info.updatedAt);
          }
        }
      })
      .catch(() => {});

    setStorageUsedBytes(estimateLocalStorageBytes());

    return () => {
      active = false;
    };
  }, [open, setCoreHealth, setProviderKeyConfigured, setStorageUsedBytes]);

  const handleAvatarUpload = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = () => {
      if (typeof reader.result === "string") {
        setUserProfile({ avatarDataUrl: reader.result });
      }
    };
    reader.readAsDataURL(file);
  };

  const handleSaveApiKey = async () => {
    if (!editingKeyProvider || !keyInputValue.trim()) return;
    const provider = editingKeyProvider;
    const secret = keyInputValue.trim();
    setKeyInputValue("");
    setEditingKeyProvider(null);
    setProviderKeyConfigured(provider, true, new Date().toISOString());
    try {
      await fetch(`${CORE_URL}/settings/api-keys`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ provider, apiKey: secret }),
      });
    } catch {
      // Stored locally in configured status flag
    }
  };

  const handleClearLocalData = () => {
    if (clearStep === 0) {
      setClearStep(1);
      return;
    }
    if (clearStep === 1) {
      setClearStep(2);
      return;
    }
    if (typeof window !== "undefined") {
      window.localStorage.clear();
      window.location.reload();
    }
  };

  return {
    editingKeyProvider,
    setEditingKeyProvider,
    keyInputValue,
    setKeyInputValue,
    clearStep,
    setClearStep,
    handleAvatarUpload,
    handleSaveApiKey,
    handleClearLocalData,
  };
}
