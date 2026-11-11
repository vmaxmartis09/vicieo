"use client";

import React from "react";
import { cn } from "@/lib/utils";
import type { PublishableVideo } from "@/lib/api/social";
import { Search, Video } from "../../icons";

interface Step1VideoSelectorProps {
  videoSearch: string;
  onChangeSearch: (value: string) => void;
  filterUnpostedOnly: boolean;
  onToggleFilterUnpostedOnly: (value: boolean) => void;
  loadingVideos: boolean;
  filteredVideos: PublishableVideo[];
  selectedVideoIds: string[];
  onChangeSelectedVideoIds: (ids: string[]) => void;
}

export function Step1VideoSelector({
  videoSearch,
  onChangeSearch,
  filterUnpostedOnly,
  onToggleFilterUnpostedOnly,
  loadingVideos,
  filteredVideos,
  selectedVideoIds,
  onChangeSelectedVideoIds,
}: Step1VideoSelectorProps) {
  return (
    <div className="space-y-4">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div className="relative flex-1 min-w-[240px]">
          <Search className="absolute left-3 top-2 h-4 w-4 text-muted-foreground" />
          <input
            type="text"
            value={videoSearch}
            onChange={(e) => onChangeSearch(e.target.value)}
            placeholder="Tìm kiếm video theo tiêu đề..."
            className="w-full rounded-ctl border border-border bg-background pl-8 pr-3 py-2 text-xs text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-primary"
          />
        </div>
        <div className="flex items-center gap-3">
          <label className="flex items-center gap-2 text-xs text-muted-foreground cursor-pointer select-none">
            <input
              type="checkbox"
              checked={filterUnpostedOnly}
              onChange={(e) => onToggleFilterUnpostedOnly(e.target.checked)}
              className="rounded border-border text-primary focus:ring-primary"
            />
            <span>Chỉ hiện video chưa đăng</span>
          </label>
          <span className="rounded-lg bg-primary/10 px-3 py-1 text-xs font-semibold text-primary">
            Đã chọn {selectedVideoIds.length} video
          </span>
        </div>
      </div>

      {loadingVideos ? (
        <div className="py-20 text-center text-xs text-muted-foreground">
          Đang quét kho video hoàn tất...
        </div>
      ) : filteredVideos.length === 0 ? (
        <div className="py-16 text-center text-xs text-muted-foreground border-2 border-dashed border-border rounded-xl">
          Không tìm thấy video nào phù hợp. Hãy render thêm video tại Studio.
        </div>
      ) : (
        <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-3">
          {filteredVideos.map((v) => {
            const isSelected = selectedVideoIds.includes(v.id);
            return (
              <div
                key={v.id}
                onClick={() => {
                  if (isSelected) {
                    onChangeSelectedVideoIds(
                      selectedVideoIds.filter((id) => id !== v.id)
                    );
                  } else {
                    onChangeSelectedVideoIds([...selectedVideoIds, v.id]);
                  }
                }}
                className={cn(
                  "group relative flex flex-col rounded-xl border bg-card overflow-clip cursor-pointer transition-all shadow-2xs hover:border-primary/60",
                  isSelected
                    ? "border-primary ring-2 ring-primary/20 bg-primary/5"
                    : "border-border"
                )}
              >
                <div className="relative aspect-video w-full bg-muted flex items-center justify-center overflow-hidden">
                  {v.thumb ? (
                    <img
                      decoding="async"
                      loading="lazy"
                      src={`/api/media?path=${encodeURIComponent(v.thumb)}`}
                      alt={v.title}
                      className="h-full w-full object-cover"
                    />
                  ) : (
                    <Video className="h-8 w-8 text-muted-foreground/50" />
                  )}
                  <div className="absolute top-2 left-2 rounded-md bg-black/60 backdrop-blur-md px-2 py-1 text-caption font-mono font-bold text-white">
                    {v.aspect_ratio} · {v.duration_s.toFixed(0)}s
                  </div>
                  <div className="absolute top-2 right-2">
                    <div
                      className={cn(
                        "h-5 w-5 rounded-md border flex items-center justify-center transition-colors",
                        isSelected
                          ? "bg-primary border-primary text-primary-foreground"
                          : "bg-black/40 border-white/40 text-transparent"
                      )}
                    >
                      ✓
                    </div>
                  </div>
                </div>
                <div className="p-3 space-y-2 flex-1 flex flex-col justify-between">
                  <p
                    className="text-xs font-semibold text-foreground line-clamp-2"
                    title={v.title}
                  >
                    {v.title}
                  </p>
                  <div className="flex items-center justify-between text-caption text-muted-foreground pt-1 border-t border-border/50">
                    <span>{v.size_mb} MB</span>
                    <span
                      className={cn(
                        "font-medium",
                        v.posted_count > 0
                          ? "text-semantic-warning"
                          : "text-semantic-success"
                      )}
                    >
                      {v.posted_count > 0
                        ? `Đã đăng ${v.posted_count} lần`
                        : "Chưa đăng"}
                    </span>
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
