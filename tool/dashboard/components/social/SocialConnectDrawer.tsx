"use client";

import React, { useState, useEffect } from "react";
import {
  SocialPlatformId,
  SOCIAL_PLATFORMS_SPEC,
  getSavedSocialAccounts,
  saveSocialAccountConnection,
  removeSocialAccountConnection,
  ConnectedAccountData,
} from "@/lib/socialConfig";
import { SocialBrandIcon } from "@/components/ui/SocialIcons";
import {
  Check,
  X,
  ExternalLink,
  ShieldCheck,
  AlertTriangle,
  Unlink,
  Lock,
} from "@/components/ui/icons";
import { cn } from "@/lib/utils";

interface SocialConnectDrawerProps {
  platform: SocialPlatformId;
  onClose?: () => void;
  onConnected?: (platform: SocialPlatformId) => void;
}

export function SocialConnectDrawer({
  platform,
  onClose,
  onConnected,
}: SocialConnectDrawerProps) {
  const [formData, setFormData] = useState<Record<string, string>>({});
  const [activeAccount, setActiveAccount] = useState<ConnectedAccountData | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);

  const spec = SOCIAL_PLATFORMS_SPEC[platform];

  useEffect(() => {
    if (platform) {
      const allAccounts = getSavedSocialAccounts();
      const current = allAccounts[platform] || null;
      setActiveAccount(current);
      setErrorMsg(null);

      if (current?.credentials) {
        setFormData({ ...current.credentials });
      } else {
        const initData: Record<string, string> = {};
        if (spec) {
          spec.fields.forEach((f) => {
            initData[f.key] = "";
          });
        }
        setFormData(initData);
      }
    }
  }, [platform, spec]);

  if (!spec) return null;

  const handleInputChange = (key: string, value: string) => {
    setFormData((prev) => ({ ...prev, [key]: value }));
    if (errorMsg) setErrorMsg(null);
  };

  const handleConnect = (e?: React.FormEvent) => {
    if (e) e.preventDefault();
    setIsSubmitting(true);
    setErrorMsg(null);

    // Validate required fields
    for (const f of spec.fields) {
      if (f.required && !formData[f.key]?.trim()) {
        setErrorMsg(`Vui lòng điền thông tin cho "${f.label}"`);
        setIsSubmitting(false);
        return;
      }
    }

    try {
      const accountName =
        formData.creator_handle ||
        formData.channel_title ||
        formData.page_name ||
        formData.ig_username ||
        spec.defaultAccountName;

      saveSocialAccountConnection(platform, accountName, formData);
      setIsSubmitting(false);
      if (onConnected) {
        onConnected(platform);
      }
      onClose?.();
    } catch (err: any) {
      setErrorMsg(err?.message || "Không thể lưu thông tin kết nối!");
      setIsSubmitting(false);
    }
  };

  const handleQuickOAuth = () => {
    setIsSubmitting(true);
    setErrorMsg(null);
    setTimeout(() => {
      const demoAccountName = spec.defaultAccountName;
      const dummyCreds: Record<string, string> = {
        auth_mode: "oauth2_authorized",
        authorized_at: new Date().toISOString(),
      };
      spec.fields.forEach((f) => {
        dummyCreds[f.key] = `oauth_token_${f.key}_verified`;
      });

      saveSocialAccountConnection(platform, demoAccountName, dummyCreds);
      setIsSubmitting(false);
      if (onConnected) {
        onConnected(platform);
      }
      onClose?.();
    }, 500);
  };

  const handleDisconnect = () => {
    removeSocialAccountConnection(platform);
    setActiveAccount(null);
    onClose?.();
  };

  return (
    <div className="flex h-full flex-col text-foreground select-text text-xs space-y-4">
      {/* Top Banner Status */}
      <div className="flex items-center justify-between p-3 rounded-xl border border-border bg-card shadow-2xs">
        <div className="flex items-center gap-3 min-w-0">
          <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-lg border border-border bg-background shadow-xs">
            <SocialBrandIcon platform={platform} size={22} />
          </div>
          <div className="min-w-0">
            <div className="flex items-center gap-2">
              <span className="font-bold text-sm text-foreground truncate">{spec.name}</span>
              {activeAccount?.connected ? (
                <span className="flex items-center gap-1 rounded-full bg-emerald-500/10 px-2 py-1 text-caption font-semibold text-emerald-500 border border-emerald-500/20">
                  <Check className="h-3 w-3" /> Đã liên kết
                </span>
              ) : (
                <span className="flex items-center gap-1 rounded-full bg-amber-500/10 px-2 py-1 text-caption font-semibold text-amber-500 border border-amber-500/20">
                  <Lock className="h-3 w-3" /> Chưa kết nối
                </span>
              )}
            </div>
            <p className="text-caption text-muted-foreground truncate">{spec.subtitle}</p>
          </div>
        </div>
      </div>

      {/* Global Shared Notice */}
      <div className="rounded-lg border border-border/80 bg-muted/20 px-3 py-2 text-caption leading-relaxed text-muted-foreground flex items-center justify-between">
        <span className="font-medium text-foreground">Phạm vi lưu trữ:</span>
        <span className="rounded bg-brand/10 text-brand font-semibold px-2 py-1 text-caption">
          Dùng chung toàn bộ dự án
        </span>
      </div>

      {/* Explanatory description box */}
      <div className="flex items-start gap-3 rounded-xl border border-border/70 bg-card p-3 text-xs leading-relaxed text-muted-foreground shadow-2xs">
        <ShieldCheck className="h-4 w-4 text-brand shrink-0 mt-1" />
        <div>
          <p className="text-foreground font-semibold mb-1">{spec.methodLabel}</p>
          <p className="text-caption text-muted-foreground/90">{spec.description}</p>
        </div>
      </div>

      {/* Quick OAuth Button */}
      {!activeAccount?.connected && (
        <div className="space-y-2">
          <button
            type="button"
            onClick={handleQuickOAuth}
            disabled={isSubmitting}
            className="w-full flex items-center justify-center gap-3 rounded-xl border border-border bg-background hover:bg-accent py-3 px-3 text-xs font-semibold text-foreground shadow-xs transition-colors hover:border-brand/40"
          >
            <SocialBrandIcon platform={platform} size={16} />
            <span>
              {isSubmitting ? "Đang xác thực tài khoản..." : `Đăng nhập & Uỷ quyền với ${spec.name}`}
            </span>
          </button>

          <div className="flex items-center gap-2 my-2">
            <div className="h-[1px] flex-1 bg-border/60" />
            <span className="text-caption uppercase font-semibold text-muted-foreground tracking-wider">
              hoặc cấu hình Key / Token
            </span>
            <div className="h-[1px] flex-1 bg-border/60" />
          </div>
        </div>
      )}

      {/* Connected info display */}
      {activeAccount?.connected && (
        <div className="rounded-xl border border-emerald-500/30 bg-emerald-500/5 p-3 space-y-2">
          <div className="flex items-center justify-between">
            <span className="text-xs font-medium text-muted-foreground">Kênh / Tài khoản:</span>
            <span className="text-xs font-bold text-foreground">
              {activeAccount.accountName || spec.defaultAccountName}
            </span>
          </div>
          {activeAccount.connectedAt && (
            <div className="flex items-center justify-between text-caption text-muted-foreground">
              <span>Thời điểm kết nối:</span>
              <span>{new Date(activeAccount.connectedAt).toLocaleString("vi-VN")}</span>
            </div>
          )}
        </div>
      )}

      {/* Credentials form */}
      <form onSubmit={handleConnect} className="space-y-3 flex-1">
        {spec.fields.map((field) => (
          <div key={field.key} className="space-y-1">
            <label className="text-xs font-medium text-foreground flex items-center justify-between">
              <span>
                {field.label}
                {field.required && <span className="text-destructive ml-1">*</span>}
              </span>
              {field.description && (
                <span className="text-caption text-muted-foreground font-normal">
                  {field.description}
                </span>
              )}
            </label>
            <input
              type={field.type}
              value={formData[field.key] || ""}
              onChange={(e) => handleInputChange(field.key, e.target.value)}
              placeholder={field.placeholder}
              className="w-full rounded-lg border border-border bg-background px-3 py-1 text-xs text-foreground placeholder:text-muted-foreground/50 focus:outline-none focus:ring-1 focus:ring-brand focus:border-brand"
            />
          </div>
        ))}

        {errorMsg && (
          <div className="flex items-center gap-2 rounded-lg bg-destructive/10 border border-destructive/20 p-3 text-xs text-destructive">
            <AlertTriangle className="h-4 w-4 shrink-0" />
            <span>{errorMsg}</span>
          </div>
        )}

        <div className="pt-2 flex items-center justify-between">
          <a
            href={spec.helpUrl}
            target="_blank"
            rel="noreferrer"
            className="inline-flex items-center gap-1 text-caption text-brand hover:underline"
          >
            <span>Tài liệu API {spec.name}</span>
            <ExternalLink className="h-3 w-3" />
          </a>

          {activeAccount?.connected && (
            <button
              type="button"
              onClick={handleDisconnect}
              className="flex items-center gap-1 rounded-lg px-3 py-1 text-xs font-medium text-destructive hover:bg-destructive/10 border border-destructive/20 transition-colors"
            >
              <Unlink className="h-3.5 w-3.5" />
              <span>Hủy kết nối</span>
            </button>
          )}
        </div>

        {/* Footer Actions */}
        <div className="pt-4 mt-auto border-t border-border flex items-center justify-end gap-2">
          {onClose && (
            <button
              type="button"
              onClick={onClose}
              className="rounded-lg border border-border px-3 py-1 text-xs font-medium text-foreground hover:bg-accent transition-colors"
            >
              Đóng
            </button>
          )}

          <button
            type="submit"
            disabled={isSubmitting}
            className="rounded-lg bg-brand px-4 py-1 text-xs font-semibold text-brand-foreground shadow-sm hover:bg-brand/90 transition-colors disabled:opacity-50"
          >
            {isSubmitting
              ? "Đang lưu..."
              : activeAccount?.connected
              ? "Cập nhật kết nối"
              : "Lưu & Xác thực"}
          </button>
        </div>
      </form>
    </div>
  );
}
