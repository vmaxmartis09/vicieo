"use client";

import React from "react";
import type { SocialAccount, QueueItem } from "@/lib/api/social";
import type { HubView } from "@/store/modules/social";
import { CheckCircle2, Clock, Download, ExternalLink } from "@/components/ui/icons";
import { VideoThumb } from "@/components/social/hub/common/VideoThumb";
import { StatusBadge } from "@/components/social/hub/common/StatusBadge";
import type { PackagePostSelection, PublishTabBucket } from "../types";

interface UpcomingPostsSectionProps {
  upcomingPosts: QueueItem[];
  accounts: SocialAccount[];
  onOpenHubView: (view: HubView, bucket?: PublishTabBucket) => void;
  onOpenPublishCenter: (tab: "plan" | "calendar") => void;
  onSelectPackagePost: (post: PackagePostSelection) => void;
}

export function UpcomingPostsSection({
  upcomingPosts,
  accounts,
  onOpenHubView,
  onOpenPublishCenter,
  onSelectPackagePost,
}: UpcomingPostsSectionProps) {
  return (
    <div className="space-y-2">
      <div className="flex items-center justify-between">
        <span className="text-caption font-semibold text-muted-foreground uppercase tracking-wider">
          Bài đăng & Gói xuất bản
        </span>
        {upcomingPosts.length > 0 && (
          <button
            type="button"
            onClick={() => onOpenHubView("queue", "upcoming")}
            className="text-caption text-primary hover:underline flex items-center gap-1"
          >
            Xem tất cả
            <ExternalLink className="h-3 w-3" />
          </button>
        )}
      </div>

      {upcomingPosts.length === 0 ? (
        <div className="py-6 px-3 text-center border border-dashed border-border rounded-card bg-card/30 space-y-2">
          <Clock className="h-6 w-6 text-muted-foreground/60 mx-auto" />
          <p className="text-footnote text-muted-foreground">
            Chưa có bài đăng nào trong hàng đợi
          </p>
          <button
            type="button"
            onClick={() => onOpenPublishCenter("plan")}
            className="px-3 py-1 rounded-ctl bg-primary/10 text-primary text-caption font-medium hover:bg-primary/20 transition-colors"
          >
            + Lên lịch ngay
          </button>
        </div>
      ) : (
        <div className="space-y-2">
          {upcomingPosts.map((item) => {
            const acc = accounts.find((a) => a.id === item.account_id);
            const timeStr = item.scheduled_at
              ? new Date(item.scheduled_at).toLocaleTimeString("vi-VN", {
                  hour: "2-digit",
                  minute: "2-digit",
                })
              : "Bản nháp";
            const dateStr = item.scheduled_at
              ? new Date(item.scheduled_at).toLocaleDateString("vi-VN", {
                  day: "2-digit",
                  month: "2-digit",
                })
              : "";

            const payload: PackagePostSelection = {
              post_id: item.post_id,
              target_id: item.target_id,
              title: item.video_title || item.title || "Video",
              platform: item.platform,
              caption: item.caption,
            };

            return (
              <div
                key={item.target_id}
                className="p-3 rounded-card border border-border bg-card hover:border-primary/40 transition-colors space-y-2"
              >
                <div className="flex items-center gap-2">
                  <VideoThumb
                    thumb={item.thumbnail}
                    title={item.video_title}
                    className="w-12 h-8 rounded-xs shrink-0"
                  />
                  <div className="min-w-0 flex-1">
                    <span className="font-semibold text-foreground line-clamp-1 text-footnote">
                      {item.video_title || item.title || "Video"}
                    </span>
                    <div className="flex items-center gap-2 text-caption text-muted-foreground font-mono mt-1">
                      <span>
                        {dateStr} {timeStr}
                      </span>
                      {acc && <span>· {acc.platform.toUpperCase()}</span>}
                    </div>
                  </div>
                  <StatusBadge status={item.status} size="sm" />
                </div>

                <div className="flex items-center justify-end gap-2 pt-1 border-t border-border/40">
                  <button
                    type="button"
                    onClick={() => onSelectPackagePost(payload)}
                    className="px-2 py-1 rounded-ctl bg-primary/10 hover:bg-primary/20 text-primary text-caption font-semibold flex items-center gap-1 transition-colors"
                  >
                    <Download className="h-3 w-3" />
                    <span>Gói đăng (.zip)</span>
                  </button>

                  <button
                    type="button"
                    onClick={() => onSelectPackagePost(payload)}
                    className="px-2 py-1 rounded-ctl bg-emerald-500/10 hover:bg-emerald-500/20 text-emerald-500 text-caption font-semibold flex items-center gap-1 transition-colors"
                  >
                    <CheckCircle2 className="h-3 w-3" />
                    <span>Tôi đã đăng</span>
                  </button>
                </div>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
