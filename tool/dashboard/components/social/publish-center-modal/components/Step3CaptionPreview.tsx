"use client";

import React from "react";
import { cn } from "@/lib/utils";
import type { PublishableVideo, SocialAccount } from "@/lib/api/social";
import { PhonePreview } from "../../PhonePreview";
import { Sparkles, Tag } from "../../icons";
import type { VideoCaptionConfig, VideoCaptionsMap } from "../types";

interface Step3CaptionPreviewProps {
  selectedVideos: PublishableVideo[];
  selectedAccounts: SocialAccount[];
  activeVideoIndex: number;
  onChangeActiveVideoIndex: (index: number) => void;
  videoCaptions: VideoCaptionsMap;
  onChangeVideoCaptions: React.Dispatch<React.SetStateAction<VideoCaptionsMap>>;
  isGeneratingCaptions: boolean;
  onGenerateCaptions: (forAllVideos?: boolean) => void;
}

export function Step3CaptionPreview({
  selectedVideos,
  selectedAccounts,
  activeVideoIndex,
  onChangeActiveVideoIndex,
  videoCaptions,
  onChangeVideoCaptions,
  isGeneratingCaptions,
  onGenerateCaptions,
}: Step3CaptionPreviewProps) {
  const curVid = selectedVideos[activeVideoIndex];
  const capData: Partial<VideoCaptionConfig> | null = curVid
    ? videoCaptions[curVid.id] || {
        mainCaption: curVid.title,
        includeAffiliate: false,
        affiliateLink: "",
        platformData: {},
      }
    : null;
  const firstAcc = selectedAccounts[0];

  return (
    <div className="grid grid-cols-1 lg:grid-cols-12 gap-6 items-start">
      {/* Left side: Content Editor (7 cols) */}
      <div className="lg:col-span-7 space-y-4">
        {/* Video selector switcher if multiple */}
        {selectedVideos.length > 1 && (
          <div className="flex items-center gap-2 overflow-x-auto pb-1">
            {selectedVideos.map((v, idx) => (
              <button
                key={v.id}
                type="button"
                onClick={() => onChangeActiveVideoIndex(idx)}
                className={cn(
                  "flex items-center gap-2 rounded-lg px-3 py-1 text-xs font-semibold whitespace-nowrap transition-colors border",
                  activeVideoIndex === idx
                    ? "bg-primary text-primary-foreground border-primary"
                    : "bg-muted text-muted-foreground border-border hover:bg-accent"
                )}
              >
                <span>Video {idx + 1}</span>
                <span className="truncate max-w-[120px] font-normal">
                  {v.title}
                </span>
              </button>
            ))}
          </div>
        )}

        {/* AI Generator Bar */}
        <div className="flex flex-wrap items-center justify-between gap-2 rounded-xl border border-primary/20 bg-primary/5 p-3">
          <div className="flex items-center gap-2">
            <Sparkles className="h-4 w-4 text-primary" />
            <span className="text-xs font-bold text-foreground">
              Trợ lý Kịch bản AI:
            </span>
            <select
              value={videoCaptions[curVid?.id || ""]?.goal || "story"}
              onChange={(e) => {
                const curVidId = curVid?.id;
                if (!curVidId) return;
                onChangeVideoCaptions((prev) => ({
                  ...prev,
                  [curVidId]: {
                    ...(prev[curVidId] || {}),
                    goal: e.target.value,
                  } as VideoCaptionConfig,
                }));
              }}
              className="rounded-lg border border-border bg-background px-2 py-1 text-xs text-foreground"
            >
              <option value="story">Kể chuyện / Giải trí</option>
              <option value="education">Giáo dục / Kiến thức</option>
              <option value="sales">Bán hàng / KOC Affiliate</option>
            </select>
          </div>

          <div className="flex items-center gap-2">
            <label className="flex items-center gap-1 text-caption text-muted-foreground cursor-pointer">
              <input
                type="checkbox"
                checked={
                  videoCaptions[curVid?.id || ""]?.varyPerAccount || false
                }
                onChange={(e) => {
                  const curVidId = curVid?.id;
                  if (!curVidId) return;
                  onChangeVideoCaptions((prev) => ({
                    ...prev,
                    [curVidId]: {
                      ...(prev[curVidId] || {}),
                      varyPerAccount: e.target.checked,
                    } as VideoCaptionConfig,
                  }));
                }}
                className="rounded border-border text-primary"
              />
              <span>Mỗi tài khoản một biến thể</span>
            </label>

            <button
              type="button"
              disabled={isGeneratingCaptions}
              onClick={() => onGenerateCaptions(false)}
              className="flex items-center gap-1 rounded-lg bg-primary px-3 py-1 text-xs font-semibold text-primary-foreground hover:bg-primary/90 transition-colors shadow-2xs disabled:opacity-50"
            >
              <Sparkles className="h-3 w-3" />
              <span>
                {isGeneratingCaptions ? "Đang viết..." : "Viết caption tự động"}
              </span>
            </button>
          </div>
        </div>

        {/* Main Caption Textarea */}
        {curVid && capData && (
          <div className="space-y-3 rounded-xl border border-border bg-card p-4 shadow-2xs">
            <div className="flex items-center justify-between">
              <label className="text-xs font-bold text-foreground flex items-center gap-2">
                <Tag className="h-3.5 w-3.5 text-primary" />
                Caption chính:
              </label>
              <span className="text-caption font-mono text-muted-foreground">
                {capData.mainCaption?.length || 0} ký tự
              </span>
            </div>

            <textarea
              rows={4}
              value={capData.mainCaption}
              onChange={(e) => {
                onChangeVideoCaptions((prev) => ({
                  ...prev,
                  [curVid.id]: {
                    ...(capData as VideoCaptionConfig),
                    mainCaption: e.target.value,
                  },
                }));
              }}
              placeholder="Nhập nội dung caption bài viết..."
              className="w-full rounded-xl border border-border bg-background p-3 text-xs text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-primary leading-relaxed"
            />

            {/* Affiliate link input */}
            <div className="flex flex-wrap items-center justify-between gap-2 pt-2 border-t border-border">
              <label className="flex items-center gap-2 text-xs text-foreground cursor-pointer">
                <input
                  type="checkbox"
                  checked={capData.includeAffiliate}
                  onChange={(e) => {
                    onChangeVideoCaptions((prev) => ({
                      ...prev,
                      [curVid.id]: {
                        ...(capData as VideoCaptionConfig),
                        includeAffiliate: e.target.checked,
                      },
                    }));
                  }}
                  className="rounded border-border text-primary"
                />
                <span className="font-medium">
                  Chèn liên kết tiếp thị liên kết (Affiliate / KOC)
                </span>
              </label>

              {capData.includeAffiliate && (
                <input
                  type="text"
                  value={capData.affiliateLink}
                  onChange={(e) => {
                    onChangeVideoCaptions((prev) => ({
                      ...prev,
                      [curVid.id]: {
                        ...(capData as VideoCaptionConfig),
                        affiliateLink: e.target.value,
                      },
                    }));
                  }}
                  placeholder="https://shopee.vn/..."
                  className="flex-1 min-w-[200px] rounded-lg border border-border bg-background px-3 py-1 text-xs text-foreground"
                />
              )}
            </div>
          </div>
        )}
      </div>

      {/* Right side: Phone Preview (5 cols) */}
      <div className="lg:col-span-5 flex flex-col items-center justify-center p-2 rounded-xl border border-border bg-muted/20">
        <PhonePreview
          platform={firstAcc ? firstAcc.platform : "tiktok"}
          title={curVid?.title || "Sóc Nhỏ TV"}
          caption={capData?.mainCaption || curVid?.title || ""}
          accountHandle={firstAcc ? firstAcc.handle : "@socnhotv.official"}
          videoThumb={curVid?.thumb}
          productLink={
            capData?.includeAffiliate ? capData?.affiliateLink : undefined
          }
        />
      </div>
    </div>
  );
}
