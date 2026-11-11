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

interface SocialConnectModalProps {
  platform: SocialPlatformId | null;
  isOpen: boolean;
  onClose: () => void;
  onConnected?: (platform: SocialPlatformId) => void;
}

export function SocialConnectModal({
  platform,
  isOpen,
  onClose,
  onConnected,
}: SocialConnectModalProps) {
  const [formData, setFormData] = useState<Record<string, string>>({});
  const [activeAccount, setActiveAccount] = useState<ConnectedAccountData | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);

  const spec = platform ? SOCIAL_PLATFORMS_SPEC[platform] : null;

  useEffect(() => {
    if (platform && isOpen) {
      const allAccounts = getSavedSocialAccounts();
      const current = allAccounts[platform] || null;
      setActiveAccount(current);
      setErrorMsg(null);

      // Pre-fill fields if already connected or fill defaults
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
  }, [platform, isOpen, spec]);

  if (!isOpen || !platform || !spec) return null;

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
      onClose();
    } catch (err: any) {
      setErrorMsg(err?.message || "Không thể lưu thông tin kết nối!");
      setIsSubmitting(false);
    }
  };

  const handleQuickOAuth = () => {
    setIsSubmitting(true);
    setErrorMsg(null);
    // Simulate quick OAuth login authorization flow with official branding
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
      onClose();
    }, 500);
  };

  const handleDisconnect = () => {
    removeSocialAccountConnection(platform);
    setActiveAccount(null);
    onClose();
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-stretch justify-end bg-black/40 backdrop-blur-xs animate-in fade-in duration-200"
      onClick={onClose}
    >
      <div
        style={{ width: "min(420px, 92vw)" }}
        className="relative h-full flex flex-col border-l border-border bg-card shadow-2xl select-text nodrag animate-in slide-in-from-right duration-200"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Panel Header tuân thủ style dự án */}
        <div className="flex h-12 shrink-0 items-center justify-between border-b border-border px-4 bg-muted/20">
          <div className="flex items-center gap-3 min-w-0">
            <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-ctl border border-border bg-background shadow-xs">
              <SocialBrandIcon platform={platform} size={20} />
            </div>
            <div className="min-w-0">
              <div className="flex items-center gap-2">
                <h3 className="text-footnote font-semibold text-foreground truncate">
                  Kết nối {spec.name}
                </h3>
                {activeAccount?.connected ? (
                  <span className="flex items-center gap-1 rounded-full bg-emerald-500/10 px-2 py-1 text-caption font-medium text-emerald-500 border border-emerald-500/20">
                    <Check className="h-3 w-3" /> Đã kết nối
                  </span>
                ) : (
                  <span className="flex items-center gap-1 rounded-full bg-amber-500/10 px-2 py-1 text-caption font-medium text-amber-500 border border-amber-500/20">
                    <Lock className="h-3 w-3" /> Chưa kết nối
                  </span>
                )}
              </div>
              <p className="text-caption text-muted-foreground truncate">{spec.subtitle}</p>
            </div>
          </div>

          <button
            type="button"
            onClick={onClose}
            className="flex h-7 w-7 items-center justify-center rounded-ctl text-muted-foreground hover:bg-accent hover:text-foreground transition-colors shrink-0"
            title="Đóng panel"
          >
            <X className="h-4 w-4" />
          </button>
        </div>

        {/* Panel Body */}
        <div className="flex-1 overflow-y-auto p-4 space-y-4 text-footnote scroll-quiet">
          {/* Global Notice */}
          <div className="rounded-ctl border border-border/80 bg-muted/30 p-3 text-caption leading-relaxed text-muted-foreground flex items-center justify-between">
            <span className="font-medium text-foreground">Phạm vi áp dụng:</span>
            <span className="rounded-xs bg-brand/10 text-brand font-semibold px-2 py-1 text-caption">
              Dùng chung toàn bộ dự án
            </span>
          </div>

          {/* Explanation Alert */}
          <div className="flex items-start gap-3 rounded-card border border-border/70 bg-card p-3 text-footnote leading-relaxed text-muted-foreground shadow-2xs">
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
                className="w-full flex items-center justify-center gap-2 rounded-card border border-border bg-background hover:bg-accent py-2 px-3 text-footnote font-semibold text-foreground shadow-xs transition-colors hover:border-brand/40"
              >
                <SocialBrandIcon platform={platform} size={16} />
                <span>
                  {isSubmitting
                    ? "Đang xác thực tài khoản..."
                    : `Đăng nhập & Uỷ quyền với ${spec.name}`}
                </span>
              </button>

              <div className="flex items-center gap-2 my-3">
                <div className="h-px flex-1 bg-border/60" />
                <span className="text-caption uppercase font-semibold text-muted-foreground tracking-wider">
                  hoặc thiết lập Key / Token riêng
                </span>
                <div className="h-px flex-1 bg-border/60" />
              </div>
            </div>
          )}

          {/* Connected Info Box */}
          {activeAccount?.connected && (
            <div className="rounded-card border border-emerald-500/30 bg-emerald-500/5 p-3 space-y-2">
              <div className="flex items-center justify-between">
                <span className="text-footnote font-medium text-muted-foreground">Kênh / Tài khoản:</span>
                <span className="text-footnote font-bold text-foreground">
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

          {/* Credentials Form */}
          <form onSubmit={handleConnect} className="space-y-4">
            {spec.fields.map((field) => (
              <div key={field.key} className="space-y-1">
                <label className="text-footnote font-medium text-foreground flex items-center justify-between">
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
                  className="w-full h-7 rounded-ctl border border-border bg-background px-3 text-footnote text-foreground placeholder:text-muted-foreground/50 focus:outline-none focus:ring-1 focus:ring-brand focus:border-brand"
                />
              </div>
            ))}

            {errorMsg && (
              <div className="flex items-center gap-2 rounded-ctl bg-destructive/10 border border-destructive/20 p-3 text-footnote text-destructive">
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
                  className="flex items-center gap-1 rounded-ctl px-3 py-1 text-footnote font-medium text-destructive hover:bg-destructive/10 border border-destructive/20 transition-colors"
                >
                  <Unlink className="h-3.5 w-3.5" />
                  <span>Hủy kết nối</span>
                </button>
              )}
            </div>
          </form>
        </div>

        {/* Panel Footer */}
        <div className="flex items-center justify-between border-t border-border px-4 py-3 shrink-0 bg-muted/20">
          <button
            type="button"
            onClick={onClose}
            className="h-7 rounded-ctl border border-border px-3 text-footnote font-medium text-foreground hover:bg-accent transition-colors"
          >
            Đóng
          </button>

          <button
            type="button"
            onClick={() => handleConnect()}
            disabled={isSubmitting}
            className="h-7 rounded-ctl bg-brand px-4 text-footnote font-semibold text-brand-foreground shadow-sm hover:bg-brand/90 transition-colors disabled:opacity-50"
          >
            {isSubmitting
              ? "Đang lưu..."
              : activeAccount?.connected
              ? "Cập nhật kết nối"
              : "Lưu & Xác thực"}
          </button>
        </div>
      </div>
    </div>
  );
}
