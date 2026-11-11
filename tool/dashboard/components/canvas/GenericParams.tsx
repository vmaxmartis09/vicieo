"use client";

import React, { useState, useEffect } from "react";
import { useStudioStore } from "@/store";
import { FALLBACK_NODE_TYPES } from "@/lib/core-api";
import type { NodeDefinition } from "@/lib/core-contracts";
import type { Character } from "@/lib/repositories";
import { cn } from "@/lib/utils";
import { SocialBrandIcon } from "@/components/ui/SocialIcons";
import { SocialConnectDrawer } from "@/components/social/SocialConnectDrawer";
import { usePreview } from "@/lib/preview";
import {
  SocialPlatformId,
  getSavedSocialAccounts,
} from "@/lib/socialConfig";
import { Plus, Users, ThumbsUp, AlertTriangle, Sparkles, CheckCircle2 } from "@/components/ui/icons";

export const TOOLBOX_NODE_DEFAULTS: Record<string, Record<string, any>> = {
  bg_remove: {
    model: "u2net",
    alpha_matting: false,
    characterId: "",
    imageIndex: 0,
  },
  depth_layers: {
    fg_pct: 70,
    mid_pct: 35,
    feather: 3,
    max_side: 1280,
    characterId: "",
    imageIndex: 0,
  },
  handdraw: {
    duration_sec: 5.0,
    fps: 24,
    width: 540,
    height: 960,
    show_hand: true,
  },
  scene_cut: {
    threshold: 3.0,
    min_scene_s: 1.0,
    export_clips: false,
    aspect: "keep",
    max_scenes: 200,
  },
  voice_clone: {
    name: "",
    display_name: "",
    consent_note: "",
    dialect: "south",
    duration: 4.5,
    verify: true,
  },
  word_captions: {
    model_size: "small",
    language: "vi",
  },
  template_render: {
    template_id: "tiktok-sales-v1",
    platforms: ["tiktok"],
    slot_mapping: {},
  },
  platform_export: {
    fit_strategy: "cover",
    platforms: ["tiktok"],
  },
  source_video: {
    url: "",
    rights: "reference_only",
  },
  localize: {
    mode: "R",
    style: "faithful",
    voice_id: "soc-nho",
    auto_draft: true,
  },
  blueprint_analyze: {},
  script_from_blueprint: {
    generator_type: "koc_review",
    voice_id: "soc-nho",
  },
  channel_insight: {
    max_videos: 50,
    channel_url: "",
  },
};

const PARAM_LABELS: Record<string, string> = {
  model: "Model AI",
  alpha_matting: "Alpha matting (tinh chỉnh viền)",
  characterId: "Nhân vật tham chiếu",
  imageIndex: "Vị trí ảnh DNA",
  model_size: "Kích cỡ model Whisper",
  language: "Ngôn ngữ",
  threshold: "Ngưỡng phát hiện cảnh",
  min_scene_s: "Độ dài tối thiểu cảnh (s)",
  export_clips: "Tự động xuất từng clip",
  aspect: "Tỷ lệ khung hình",
  max_scenes: "Số cảnh tối đa",
  fg_pct: "Tỉ lệ tiền cảnh (%)",
  mid_pct: "Tỉ lệ trung cảnh (%)",
  feather: "Làm mềm viền (feather)",
  max_side: "Độ phân giải tối đa (px)",
  duration_sec: "Thời lượng (giây)",
  fps: "Khung hình / giây (FPS)",
  width: "Chiều rộng (px)",
  height: "Chiều cao (px)",
  show_hand: "Hiện bàn tay vẽ",
  name: "Mã định danh giọng (slug)",
  display_name: "Tên hiển thị giọng",
  consent_note: "Xác nhận đồng ý của chủ giọng",
  dialect: "Phương ngữ",
  duration: "Độ dài mẫu đọc (giây)",
  verify: "Kiểm tra sau khi clone",
  template_id: "Mẫu video (Template ID)",
  fit_strategy: "Chiến lược fit khung hình",
  platforms: "Nền tảng xuất bản",
  target_platforms: "Nền tảng xuất bản",
  slot_mapping: "Dữ liệu phân cảnh (Slot Mapping)",
  url: "Đường dẫn video (URL)",
  rights: "Quyền sử dụng",
  mode: "Chế độ bản địa hoá",
  voice_id: "Mã giọng đọc",
  auto_draft: "Tự động tạo bản nháp",
  generator_type: "Loại kịch bản",
  max_videos: "Số video tối đa",
  channel_url: "Đường dẫn kênh đối thủ (URL)",
};

const KNOWN_OPTIONS: Record<string, { label: string; value: string }[]> = {
  model: [
    { label: "u2net (Chuẩn CPU)", value: "u2net" },
    { label: "u2netp (Nhẹ/nhanh)", value: "u2netp" },
    { label: "isnet-general-use", value: "isnet-general-use" },
    { label: "birefnet (Chất lượng)", value: "birefnet" },
    { label: "silueta", value: "silueta" },
  ],
  model_size: [
    { label: "small (Khuyên dùng)", value: "small" },
    { label: "tiny (Nhanh nhất)", value: "tiny" },
    { label: "base", value: "base" },
    { label: "medium", value: "medium" },
    { label: "large-v3", value: "large-v3" },
  ],
  language: [
    { label: "Tiếng Việt (vi)", value: "vi" },
    { label: "Tiếng Anh (en)", value: "en" },
    { label: "Tiếng Trung (zh)", value: "zh" },
    { label: "Tiếng Nhật (ja)", value: "ja" },
    { label: "Tiếng Hàn (ko)", value: "ko" },
  ],
  aspect: [
    { label: "Giữ nguyên (keep)", value: "keep" },
    { label: "Dọc 9:16 (TikTok)", value: "9:16" },
    { label: "Ngang 16:9 (YouTube)", value: "16:9" },
    { label: "Vuông 1:1", value: "1:1" },
    { label: "Dọc 4:5 (Instagram)", value: "4:5" },
  ],
  dialect: [
    { label: "Miền Nam (south)", value: "south" },
    { label: "Miền Bắc (north)", value: "north" },
    { label: "Miền Trung (central)", value: "central" },
  ],
  fit_strategy: [
    { label: "Cover (Cắt vừa khung)", value: "cover" },
    { label: "Contain (Giữ nguyên tỉ lệ)", value: "contain" },
    { label: "Blur-pad (Làm mờ viền)", value: "blur-pad" },
  ],
  template_id: [
    { label: "koc-review-v1 (Review KOC 5 sao)", value: "koc-review-v1" },
    { label: "tiktok-sales-v1 (Bán hàng TikTok Hook 3s)", value: "tiktok-sales-v1" },
    { label: "shorts-fact-v1 (Kiến thức Fact Shorts)", value: "shorts-fact-v1" },
    { label: "unboxing-v1 (Đập hộp & Trải nghiệm)", value: "unboxing-v1" },
    { label: "story-cartoon-v1 (Hoạt hình thiếu nhi Sóc Nhỏ)", value: "story-cartoon-v1" },
    { label: "affiliate-compare-v1 (So sánh 2 sản phẩm)", value: "affiliate-compare-v1" },
  ],
  style: [
    { label: "Fade (Mờ dần)", value: "fade" },
    { label: "Dissolve (Hòa tan)", value: "dissolve" },
    { label: "Wipe left (Gạt ngang)", value: "wipe_left" },
    { label: "Slide up (Trượt lên)", value: "slide_up" },
    { label: "Trung thành (faithful)", value: "faithful" },
    { label: "Dí dỏm / hài hước (funny)", value: "funny" },
    { label: "Kịch tính (dramatic)", value: "dramatic" },
  ],
  rights: [
    { label: "Chỉ tham khảo (Reference only)", value: "reference_only" },
    { label: "Đã có bản quyền (Authorized)", value: "authorized" },
  ],
  mode: [
    { label: "Remake (Tạo lại cảnh mới - Mode R)", value: "R" },
    { label: "Localize (Giữ video gốc - Mode L)", value: "L" },
  ],
  generator_type: [
    { label: "Review sản phẩm KOC (koc_review)", value: "koc_review" },
    { label: "Hook bán hàng Affiliate (affiliate_hook)", value: "affiliate_hook" },
  ],
};

function formatParamLabel(key: string): string {
  if (PARAM_LABELS[key]) return PARAM_LABELS[key];
  return key
    .replace(/_/g, " ")
    .replace(/\b\w/g, (char) => char.toUpperCase());
}

/** Tái sử dụng hiển thị/chọn nền tảng đồng bộ với NodeTab.tsx */
export function PlatformsView({
  platforms,
  onToggle,
}: {
  platforms?: string[];
  onToggle?: (platform: string) => void;
}) {
  const ALL_PLATFORMS: SocialPlatformId[] = ["tiktok", "youtube", "facebook", "instagram"];

  const [connectedMap, setConnectedMap] = useState<Record<string, boolean>>({});

  // Sync connected status
  const refreshConnections = () => {
    const accs = getSavedSocialAccounts();
    const map: Record<string, boolean> = {};
    ALL_PLATFORMS.forEach((p) => {
      map[p] = Boolean(accs[p]?.connected);
    });
    setConnectedMap(map);
  };

  useEffect(() => {
    refreshConnections();

    const handleUpdate = () => {
      refreshConnections();
    };

    window.addEventListener("social-accounts-updated", handleUpdate);
    return () => {
      window.removeEventListener("social-accounts-updated", handleUpdate);
    };
  }, []);

  const connectedPlatforms = ALL_PLATFORMS.filter((plat) => Boolean(connectedMap[plat]));
  const current =
    platforms && platforms.length > 0
      ? platforms.filter((p) => connectedMap[p])
      : connectedPlatforms.slice(0, 1);

  if (connectedPlatforms.length === 0) {
    return (
      <div className="flex items-center justify-between gap-2 py-1">
        <span className="text-caption font-medium text-muted-foreground">
          Nền tảng xuất bản:
        </span>
        <button
          type="button"
          onClick={(e) => {
            e.stopPropagation();
            useStudioStore.getState().setActiveRightTab("publish");
          }}
          className="nodrag flex items-center gap-1 text-caption text-brand hover:underline"
          title="Mở tab Social để kết nối tài khoản xuất bản"
        >
          <Plus className="h-3 w-3" />
          <span>Kết nối ở tab Social</span>
        </button>
      </div>
    );
  }

  return (
    <div className="space-y-2">
      <div className="flex flex-wrap items-baseline justify-between gap-1">
        <span className="text-caption font-medium text-muted-foreground">
          Nền tảng xuất bản:
        </span>
        <span className="text-caption text-muted-foreground/70">
          * Chọn nền tảng để chia sẻ
        </span>
      </div>
      <div className="flex flex-wrap items-center gap-2">
        {connectedPlatforms.map((plat) => {
          const isSelected = current.includes(plat);

          return (
            <div key={plat} className="relative group">
              <button
                type="button"
                onClick={(e) => {
                  e.stopPropagation();
                  if (onToggle) onToggle(plat);
                }}
                title={`${plat.toUpperCase()}: Đã kết nối (Nhấn để bật/tắt xuất bản)`}
                className={cn(
                  "nodrag relative flex h-8 w-8 items-center justify-center rounded-ctl border transition-all shadow-sm",
                  isSelected
                    ? "bg-brand/15 border-brand/50 ring-1 ring-brand/30 shadow-brand/10 scale-105"
                    : "bg-muted/40 border-border/80 hover:bg-muted hover:border-border"
                )}
              >
                {/* Brand Icon */}
                <SocialBrandIcon
                  platform={plat}
                  size={18}
                  variant="brand"
                />

                {/* Connected status badge */}
                <span
                  className={cn(
                    "absolute -top-1 -right-1 flex h-2 w-2 rounded-full",
                    isSelected
                      ? "bg-emerald-500 ring-2 ring-background"
                      : "bg-emerald-500/60 ring-1 ring-background"
                  )}
                  title="Tài khoản đã kết nối"
                />
              </button>

              {/* Quick gear button on hover -> opens right Social tab and drawer */}
              {onToggle && (
                <button
                  type="button"
                  onClick={(e) => {
                    e.stopPropagation();
                    useStudioStore.getState().setActiveRightTab("publish");
                    usePreview.getState().open({
                      kind: "custom",
                      title: `Cấu hình tài khoản ${plat.toUpperCase()}`,
                      render: (close) => (
                        <SocialConnectDrawer
                          platform={plat}
                          onClose={close}
                          onConnected={() => {
                            refreshConnections();
                            close();
                          }}
                        />
                      ),
                    });
                  }}
                  title="Cấu hình tài khoản ở tab Social"
                  className="absolute -top-1 -right-1 hidden group-hover:flex h-4 w-4 items-center justify-center rounded-full bg-background border border-border text-caption text-muted-foreground hover:text-foreground shadow"
                >
                  ⚙
                </button>
              )}
            </div>
          );
        })}

        {/* Plus button to add more platforms from Social tab */}
        <button
          type="button"
          onClick={(e) => {
            e.stopPropagation();
            useStudioStore.getState().setActiveRightTab("publish");
          }}
          title="Kết nối thêm mạng xã hội ở tab Social"
          className="nodrag flex h-8 w-8 items-center justify-center rounded-ctl border border-dashed border-border/70 text-muted-foreground hover:border-brand hover:text-brand transition-colors"
        >
          <Plus className="h-3.5 w-3.5" />
        </button>
      </div>
    </div>
  );
}

/** Tái sử dụng hiển thị Slot Mapping phân cảnh dạng thu gọn tinh gọn */
export function SlotMappingView({
  slotMapping,
}: {
  slotMapping?: Record<string, any>;
}) {
  const [isOpen, setIsOpen] = useState(false);
  const [expandedKeys, setExpandedKeys] = useState<Record<string, boolean>>({});

  if (!slotMapping || typeof slotMapping !== "object" || Object.keys(slotMapping).length === 0) {
    return null;
  }

  const entries = Object.entries(slotMapping);

  const toggleKey = (k: string) => {
    setExpandedKeys((prev) => ({ ...prev, [k]: !prev[k] }));
  };

  return (
    <div className="rounded-ctl border border-border/60 bg-muted/15 overflow-hidden transition-all">
      <button
        type="button"
        onClick={(e) => {
          e.stopPropagation();
          setIsOpen(!isOpen);
        }}
        className="nodrag w-full flex items-center justify-between px-2.5 py-1.5 text-caption font-medium text-muted-foreground hover:text-foreground hover:bg-muted/30 transition-colors"
      >
        <span className="flex items-center gap-1.5 min-w-0">
          <Sparkles className="h-3 w-3 text-brand/70 shrink-0" />
          <span className="truncate">Dữ liệu nạp template (Slot Payload)</span>
        </span>
        <div className="flex items-center gap-1.5 shrink-0">
          <span className="text-[10px] bg-muted/60 px-1.5 py-0.5 rounded font-mono">
            {entries.length} slots
          </span>
          <span className="text-[10px] text-muted-foreground/80">
            {isOpen ? "Thu gọn ▲" : "Chi tiết ▼"}
          </span>
        </div>
      </button>

      {isOpen && (
        <div className="p-2 border-t border-border/40 grid grid-cols-1 gap-1.5 bg-background/50">
          {entries.map(([slotKey, slotVal]) => {
            const valStr = String(slotVal ?? "");
            const isExpanded = Boolean(expandedKeys[slotKey]);

            // 1. Reviewer badge / Tên KOC
            if (slotKey === "reviewer_badge") {
              return (
                <div
                  key={slotKey}
                  className="flex items-center justify-between gap-2 rounded-ctl border border-brand/30 bg-brand/5 px-2.5 py-1.5 shadow-2xs"
                >
                  <div className="flex items-center gap-1.5 min-w-0">
                    <Users className="h-3.5 w-3.5 text-brand shrink-0" />
                    <span className="text-caption font-medium text-muted-foreground shrink-0">Reviewer:</span>
                    <span className="text-footnote font-semibold text-foreground truncate">{valStr}</span>
                  </div>
                  <span className="text-[10px] bg-brand/15 text-brand px-1.5 py-0.5 rounded font-mono font-medium shrink-0">
                    Badge
                  </span>
                </div>
              );
            }

            // 2. Điểm đánh giá (Rating)
            if (slotKey === "rating_score") {
              return (
                <div
                  key={slotKey}
                  className="flex items-center justify-between gap-2 rounded-ctl border border-amber-500/20 bg-amber-500/5 px-2.5 py-1.5 shadow-2xs"
                >
                  <div className="flex items-center gap-1.5 min-w-0">
                    <Sparkles className="h-3.5 w-3.5 text-amber-500 shrink-0" />
                    <span className="text-caption font-medium text-amber-600 dark:text-amber-400 shrink-0">Đánh giá:</span>
                    <span className="text-footnote font-semibold text-foreground truncate">{valStr}</span>
                  </div>
                  <span className="text-[10px] bg-amber-500/15 text-amber-600 dark:text-amber-300 px-1.5 py-0.5 rounded font-mono font-bold shrink-0">
                    ★ Rating
                  </span>
                </div>
              );
            }

            // 3. Ưu điểm (Pros)
            if (slotKey === "pros_text") {
              return (
                <div
                  key={slotKey}
                  onClick={() => toggleKey(slotKey)}
                  className="group/slot cursor-pointer rounded-ctl border border-emerald-500/25 bg-emerald-500/5 p-2 space-y-1 hover:border-emerald-500/40 transition-colors shadow-2xs"
                  title="Bấm để mở rộng / thu gọn nội dung ưu điểm"
                >
                  <div className="flex items-center justify-between gap-1 text-caption">
                    <div className="flex items-center gap-1.5 text-emerald-600 dark:text-emerald-400 font-semibold">
                      <ThumbsUp className="h-3.5 w-3.5 shrink-0" />
                      <span>Ưu điểm sản phẩm</span>
                    </div>
                    <span className="text-[10px] text-muted-foreground group-hover/slot:text-foreground">
                      {isExpanded ? "Thu gọn" : "Xem thêm"}
                    </span>
                  </div>
                  <p className={cn("text-footnote text-foreground/90 leading-relaxed", !isExpanded && "line-clamp-2")}>
                    {valStr}
                  </p>
                </div>
              );
            }

            // 4. Nhược điểm / Lưu ý (Cons)
            if (slotKey === "cons_text") {
              return (
                <div
                  key={slotKey}
                  onClick={() => toggleKey(slotKey)}
                  className="group/slot cursor-pointer rounded-ctl border border-amber-500/25 bg-amber-500/5 p-2 space-y-1 hover:border-amber-500/40 transition-colors shadow-2xs"
                  title="Bấm để mở rộng / thu gọn nội dung lưu ý"
                >
                  <div className="flex items-center justify-between gap-1 text-caption">
                    <div className="flex items-center gap-1.5 text-amber-600 dark:text-amber-400 font-semibold">
                      <AlertTriangle className="h-3.5 w-3.5 shrink-0" />
                      <span>Lưu ý / Nhược điểm</span>
                    </div>
                    <span className="text-[10px] text-muted-foreground group-hover/slot:text-foreground">
                      {isExpanded ? "Thu gọn" : "Xem thêm"}
                    </span>
                  </div>
                  <p className={cn("text-footnote text-foreground/90 leading-relaxed", !isExpanded && "line-clamp-2")}>
                    {valStr}
                  </p>
                </div>
              );
            }

            // 5. Thẻ chung cho các slot khác
            const isLong = valStr.length > 60;
            return (
              <div
                key={slotKey}
                onClick={() => isLong && toggleKey(slotKey)}
                className={cn(
                  "rounded-ctl border border-border/60 bg-card/80 p-2 space-y-1 shadow-2xs transition-colors",
                  isLong && "group/slot cursor-pointer hover:border-border"
                )}
              >
                <div className="flex items-center justify-between gap-1 text-caption">
                  <div className="flex items-center gap-1.5 font-mono text-muted-foreground font-medium">
                    <CheckCircle2 className="h-3 w-3 text-brand/70 shrink-0" />
                    <span className="truncate">{slotKey}</span>
                  </div>
                  {isLong && (
                    <span className="text-[10px] text-muted-foreground group-hover/slot:text-foreground">
                      {isExpanded ? "Thu gọn" : "Xem thêm"}
                    </span>
                  )}
                </div>
                <p className={cn("text-footnote text-foreground/90 leading-relaxed", !isExpanded && isLong && "line-clamp-2")}>
                  {valStr}
                </p>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}

export interface GenericParamsProps {
  nodeId: string;
  nodeType?: string;
  params?: Record<string, any>;
  def?: NodeDefinition;
  excludeKeys?: string[];
  className?: string;
}

export function GenericParams({
  nodeId,
  nodeType,
  params,
  def,
  excludeKeys = [],
  className,
}: GenericParamsProps) {
  const updateNodeParams = useStudioStore((s) => s.updateNodeParams);
  const nodeCatalog = useStudioStore((s) => s.nodeCatalog);
  const characters = useStudioStore((s) => s.characters) as Character[];

  const actualDef =
    def ||
    nodeCatalog.find((d) => d.typeName === nodeType) ||
    FALLBACK_NODE_TYPES.find((d) => d.typeName === nodeType);

  const fallbackDefaults = (nodeType && TOOLBOX_NODE_DEFAULTS[nodeType]) || {};
  const defaultParams: Record<string, any> = {
    ...fallbackDefaults,
    ...(actualDef?.defaultParams || {}),
  };

  const currentParams: Record<string, any> = params || {};

  // Lọc bỏ khóa bắt đầu bằng "_" và khóa nằm trong excludeKeys
  const keys = Object.keys(defaultParams).filter(
    (k) => !k.startsWith("_") && !excludeKeys.includes(k)
  );

  if (keys.length === 0) return null;

  return (
    <div className={cn("space-y-2 text-footnote", className)}>
      {keys.map((key) => {
        const defaultVal = defaultParams[key];
        const val = currentParams[key] !== undefined ? currentParams[key] : defaultVal;
        const label = formatParamLabel(key);

        // 1. Boolean = Công tắc (Switch toggle)
        if (typeof defaultVal === "boolean" || typeof val === "boolean") {
          const isChecked = Boolean(val);
          return (
            <div key={key} className="flex items-center justify-between gap-2 py-1">
              <span className="text-caption font-medium text-muted-foreground">{label}</span>
              <button
                type="button"
                role="switch"
                aria-checked={isChecked}
                onClick={(e) => {
                  e.stopPropagation();
                  updateNodeParams(nodeId, { [key]: !isChecked });
                }}
                className={cn(
                  "nodrag relative inline-flex h-4 w-7 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none",
                  isChecked ? "bg-primary" : "bg-muted"
                )}
              >
                <span
                  className={cn(
                    "pointer-events-none inline-block h-3 w-3 transform rounded-full bg-background shadow-xs transition duration-200 ease-in-out",
                    isChecked ? "translate-x-3" : "translate-x-0"
                  )}
                />
              </button>
            </div>
          );
        }

        // 2. Number = Ô số
        if (typeof defaultVal === "number" || (typeof val === "number" && !isNaN(val))) {
          const isFloat = typeof defaultVal === "number" && !Number.isInteger(defaultVal);
          return (
            <div key={key} className="flex items-center justify-between gap-2 py-1">
              <span className="truncate text-caption font-medium text-muted-foreground" title={label}>
                {label}
              </span>
              <input
                type="number"
                value={val ?? ""}
                step={isFloat ? "0.1" : "1"}
                onChange={(e) => {
                  const num = e.target.value === "" ? 0 : Number(e.target.value);
                  updateNodeParams(nodeId, { [key]: num });
                }}
                onClick={(e) => e.stopPropagation()}
                onKeyDown={(e) => e.stopPropagation()}
                className="nodrag h-7 w-24 rounded-ctl border border-border bg-card px-2 text-right font-mono text-footnote text-foreground focus:outline-none focus:border-foreground"
              />
            </div>
          );
        }

        // 3. Chuỗi có tập giá trị (hoặc characterId) = Select
        let options: { label: string; value: string }[] | null = null;
        if (key === "characterId") {
          options = [
            { label: "(Không chọn — dùng cổng vào)", value: "" },
            ...characters.map((c) => ({ label: `${c.name} (${c.id})`, value: c.id })),
          ];
        } else if (KNOWN_OPTIONS[key]) {
          options = [...KNOWN_OPTIONS[key]];
        }

        if (options) {
          const strVal = String(val ?? "");
          if (strVal && !options.some((o) => o.value === strVal)) {
            options = [{ label: strVal, value: strVal }, ...options];
          }
          return (
            <div key={key} className="flex items-center justify-between gap-2 py-1">
              <span className="truncate text-caption font-medium text-muted-foreground" title={label}>
                {label}
              </span>
              <select
                value={strVal}
                onChange={(e) => updateNodeParams(nodeId, { [key]: e.target.value })}
                onClick={(e) => e.stopPropagation()}
                className="nodrag h-7 max-w-[60%] rounded-ctl border border-border bg-card px-2 text-footnote text-foreground focus:outline-none focus:border-foreground"
              >
                {options.map((opt) => (
                  <option key={opt.value} value={opt.value}>
                    {opt.label}
                  </option>
                ))}
              </select>
            </div>
          );
        }

        // 4. Chuỗi dài = Textarea
        const isLongString =
          key === "consent_note" ||
          key.includes("note") ||
          key.includes("desc") ||
          key.includes("prompt") ||
          key.includes("criteria") ||
          key.includes("script") ||
          (typeof val === "string" && (val.length > 50 || val.includes("\n")));

        if (isLongString) {
          return (
            <div key={key} className="space-y-1 py-1">
              <span className="block text-caption font-medium text-muted-foreground">{label}</span>
              <textarea
                rows={2}
                value={typeof val === "string" ? val : ""}
                onChange={(e) => updateNodeParams(nodeId, { [key]: e.target.value })}
                onClick={(e) => e.stopPropagation()}
                onKeyDown={(e) => e.stopPropagation()}
                className="nodrag nowheel w-full resize-y rounded-ctl border border-border bg-card p-2 text-footnote text-foreground focus:outline-none focus:border-foreground"
                placeholder={`Nhập ${label.toLowerCase()}...`}
              />
            </div>
          );
        }

        // 5. Chuỗi thường = Input text
        return (
          <div key={key} className="space-y-1 py-1">
            <span className="block text-caption font-medium text-muted-foreground">{label}</span>
            <input
              type="text"
              value={typeof val === "string" ? val : ""}
              onChange={(e) => updateNodeParams(nodeId, { [key]: e.target.value })}
              onClick={(e) => e.stopPropagation()}
              onKeyDown={(e) => e.stopPropagation()}
              className="nodrag h-7 w-full rounded-ctl border border-border bg-card px-2 font-mono text-footnote text-foreground focus:outline-none focus:border-foreground"
              placeholder={`Nhập ${label.toLowerCase()}...`}
            />
          </div>
        );
      })}
    </div>
  );
}
