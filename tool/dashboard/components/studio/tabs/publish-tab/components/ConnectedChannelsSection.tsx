"use client";

import React from "react";
import type { SocialAccount } from "@/lib/api/social";
import type { SocialPlatformId } from "@/lib/socialConfig";
import type { HubView } from "@/store/modules/social";
import { ExternalLink } from "@/components/ui/icons";
import { AccountChip } from "@/components/social/hub/common/AccountChip";
import { SocialBrandIcon } from "@/components/ui/SocialIcons";
import { cn } from "@/lib/utils";
import { BRAND_PLATFORMS } from "../constants";

interface ConnectedChannelsSectionProps {
  accounts: SocialAccount[];
  connectedMap: Record<string, boolean>;
  onOpenConnectDrawer: (platform: SocialPlatformId) => void;
  onOpenHubView: (view: HubView) => void;
}

export function ConnectedChannelsSection({
  accounts,
  connectedMap,
  onOpenConnectDrawer,
  onOpenHubView,
}: ConnectedChannelsSectionProps) {
  return (
    <div className="space-y-3 pt-2 border-t border-border/60">
      <div className="flex items-center justify-between">
        <span className="text-caption font-semibold text-muted-foreground uppercase tracking-wider">
          Tài khoản kết nối ({accounts.length}/10)
        </span>
        <button
          type="button"
          onClick={() => onOpenHubView("accounts")}
          className="text-caption text-primary hover:underline flex items-center gap-1"
        >
          Quản lý
          <ExternalLink className="h-3 w-3" />
        </button>
      </div>

      <div className="grid grid-cols-2 gap-2">
        {BRAND_PLATFORMS.map((pId) => {
          const isConn = Boolean(connectedMap[pId]);
          return (
            <button
              key={pId}
              type="button"
              onClick={() => onOpenConnectDrawer(pId)}
              className={cn(
                "flex items-center justify-between p-2 rounded-card border text-footnote font-medium transition-all text-left",
                isConn
                  ? "border-emerald-500/30 bg-emerald-500/5 hover:bg-emerald-500/10 text-foreground"
                  : "border-border/60 bg-card hover:bg-accent/60 text-muted-foreground"
              )}
            >
              <div className="flex items-center gap-2 min-w-0">
                <SocialBrandIcon
                  platform={pId}
                  size={16}
                  variant={isConn ? "brand" : "monochrome"}
                />
                <span className="capitalize text-caption font-medium truncate">{pId}</span>
              </div>
              <span
                className={cn(
                  "h-2 w-2 rounded-full shrink-0",
                  isConn ? "bg-emerald-500" : "bg-muted-foreground/30"
                )}
                title={isConn ? "Đã kết nối" : "Chưa kết nối"}
              />
            </button>
          );
        })}
      </div>

      {accounts.length > 0 && (
        <div className="flex flex-wrap gap-2 pt-1">
          {accounts.map((acc) => (
            <AccountChip
              key={acc.id}
              account={acc}
              size="sm"
              onClick={() => onOpenHubView("accounts")}
            />
          ))}
        </div>
      )}
    </div>
  );
}
