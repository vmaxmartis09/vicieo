"use client";

import React, { useState, useEffect, useRef, useMemo } from "react";
import Link from "next/link";
import { useLocale, useTranslations } from "next-intl";
import { cn } from "@/lib/utils";
import {
  fetchModels,
  formatTierBadge,
  type ModelItem,
} from "@/lib/api/models";
import {
  ChevronDown,
  Search,
  Check,
  Sparkles,
  AlertTriangle,
  ExternalLink,
  Plus,
  X,
  Zap,
  Info,
} from "@/components/ui/icons";

const FALLBACK_IMAGE_MODELS: ModelItem[] = [
  {
    key: "mock-image",
    display_name: "Mock Image Generator",
    model_id: "mock-v1",
    provider: "mock",
    kind: "image",
    tier: "mock",
    capabilities: { aspect_ratios: ["16:9", "9:16", "1:1", "4:5"], resolutions: ["720p", "1080p"], supports_reference: true },
    pricing: { cost_per_generation: 0.0 },
    available: true,
  },
  {
    key: "fal-flux-1.1-pro",
    display_name: "FLUX.1.1 Pro (BFL SOTA Photorealism)",
    model_id: "fal-ai/flux-pro/v1.1",
    provider: "Black Forest Labs",
    kind: "image",
    tier: "paid",
    capabilities: { aspect_ratios: ["16:9", "9:16", "1:1", "4:5"], resolutions: ["1080p"], supports_reference: true },
    pricing: { cost_per_generation: 0.04 },
    available: true,
  },
  {
    key: "fal-flux-schnell",
    display_name: "FLUX.1 Schnell (Fast 4-Step)",
    model_id: "fal-ai/flux/schnell",
    provider: "Black Forest Labs",
    kind: "image",
    tier: "paid",
    capabilities: { aspect_ratios: ["16:9", "9:16", "1:1", "4:5"], resolutions: ["720p", "1080p"], supports_reference: false },
    pricing: { cost_per_generation: 0.003 },
    available: true,
  },
  {
    key: "fal-flux-kontext",
    display_name: "FLUX.1 Kontext (DNA & Reference Edit)",
    model_id: "fal-ai/flux-pro/kontext",
    provider: "Black Forest Labs",
    kind: "image",
    tier: "paid",
    capabilities: { aspect_ratios: ["16:9", "9:16", "1:1", "4:5"], resolutions: ["1080p"], supports_reference: true },
    pricing: { cost_per_generation: 0.04 },
    available: true,
  },
  {
    key: "midjourney-v6",
    display_name: "Midjourney v6.1 (Cinematic & Fashion)",
    model_id: "midjourney-v6.1",
    provider: "Midjourney",
    kind: "image",
    tier: "paid",
    capabilities: { aspect_ratios: ["16:9", "9:16", "1:1", "4:5"], resolutions: ["1080p"], supports_reference: true },
    pricing: { cost_per_generation: 0.05 },
    available: true,
  },
  {
    key: "ideogram-v2",
    display_name: "Ideogram v2 (Typography & Graphics)",
    model_id: "ideogram-v2",
    provider: "Ideogram AI",
    kind: "image",
    tier: "paid",
    capabilities: { aspect_ratios: ["16:9", "9:16", "1:1", "4:5"], resolutions: ["1080p"], supports_reference: true },
    pricing: { cost_per_generation: 0.04 },
    available: true,
  },
  {
    key: "recraft-v3",
    display_name: "Recraft v3 (Vector & 3D Illustration)",
    model_id: "recraft-v3",
    provider: "Recraft AI",
    kind: "image",
    tier: "paid",
    capabilities: { aspect_ratios: ["16:9", "9:16", "1:1", "4:5"], resolutions: ["1080p"], supports_reference: true },
    pricing: { cost_per_generation: 0.04 },
    available: true,
  },
  {
    key: "google-imagen-3",
    display_name: "Google Imagen 3 (DeepMind)",
    model_id: "imagen-3.0-generate-002",
    provider: "Google DeepMind",
    kind: "image",
    tier: "paid",
    capabilities: { aspect_ratios: ["16:9", "9:16", "1:1", "4:5"], resolutions: ["1080p"], supports_reference: true },
    pricing: { cost_per_generation: 0.04 },
    available: true,
  },
  {
    key: "gemini-2.5-flash-image",
    display_name: "Google Gemini Flash Image (Nano Banana)",
    model_id: "gemini-2.5-flash-image",
    provider: "Google",
    kind: "image",
    tier: "paid",
    capabilities: { aspect_ratios: ["16:9", "9:16", "1:1", "4:5"], resolutions: ["1080p"], supports_reference: true },
    pricing: { cost_per_generation: 0.03 },
    available: true,
  },
  {
    key: "sd-3.5-large",
    display_name: "Stable Diffusion 3.5 Large",
    model_id: "stability-sd-3.5-large",
    provider: "Stability AI",
    kind: "image",
    tier: "paid",
    capabilities: { aspect_ratios: ["16:9", "9:16", "1:1", "4:5"], resolutions: ["1080p"], supports_reference: true },
    pricing: { cost_per_generation: 0.035 },
    available: true,
  },
  {
    key: "sdxl-lightning",
    display_name: "SDXL Lightning (Fast 1-2s)",
    model_id: "bytedance-sdxl-lightning",
    provider: "ByteDance",
    kind: "image",
    tier: "paid",
    capabilities: { aspect_ratios: ["16:9", "9:16", "1:1", "4:5"], resolutions: ["1080p"], supports_reference: true },
    pricing: { cost_per_generation: 0.002 },
    available: true,
  },
  {
    key: "dall-e-3",
    display_name: "OpenAI DALL-E 3 HD",
    model_id: "dall-e-3",
    provider: "OpenAI",
    kind: "image",
    tier: "paid",
    capabilities: { aspect_ratios: ["16:9", "9:16", "1:1"], resolutions: ["1080p"], supports_reference: false },
    pricing: { cost_per_generation: 0.08 },
    available: true,
  },
  {
    key: "pollinations-sana",
    display_name: "Pollinations Sana (Community Fast)",
    model_id: "sana",
    provider: "Pollinations",
    kind: "image",
    tier: "paid",
    capabilities: { aspect_ratios: ["16:9", "9:16", "1:1"], resolutions: ["1080p"], supports_reference: true },
    pricing: { cost_per_generation: 0.002 },
    available: true,
  },
  {
    key: "pollinations-flux",
    display_name: "Pollinations FLUX.1 (Community)",
    model_id: "flux",
    provider: "Pollinations",
    kind: "image",
    tier: "paid",
    capabilities: { aspect_ratios: ["16:9", "9:16", "1:1", "4:5"], resolutions: ["1080p"], supports_reference: true },
    pricing: { cost_per_generation: 0.003 },
    available: true,
  },
];


export interface ModelPickerProps {
  kind: "image" | "video" | "text";
  value?: string;
  onChange: (modelKey: string) => void;
  fallbackModels?: string[];
  onFallbackChange?: (models: string[]) => void;
  allowFallback?: boolean;
  disabled?: boolean;
  compact?: boolean;
  className?: string;
}

export function ModelPicker({
  kind,
  value,
  onChange,
  fallbackModels = [],
  onFallbackChange,
  allowFallback = true,
  disabled = false,
  compact = false,
  className,
}: ModelPickerProps) {
  const locale = useLocale();
  const [models, setModels] = useState<ModelItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [isOpen, setIsOpen] = useState(false);
  const [showFallbackSelector, setShowFallbackSelector] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);

  const [searchQuery, setSearchQuery] = useState("");

  // Nạp danh sách model từ Core và gộp với danh sách chuẩn
  useEffect(() => {
    let active = true;
    setLoading(true);
    fetchModels(kind).then((data) => {
      if (!active) return;
      if (kind === "image") {
        // Gộp các model từ core và bổ sung đầy đủ model ảnh SOTA
        const existingKeys = new Set((data || []).map((m) => m.key));
        const merged = [
          ...(data || []).map((m) => {
            // Chuẩn hóa bỏ chữ Free
            const cleanName = (m.display_name || m.key).replace(/\s*\(Free[^)]*\)/gi, "");
            return { ...m, display_name: cleanName };
          }),
          ...FALLBACK_IMAGE_MODELS.filter((m) => !existingKeys.has(m.key)),
        ];
        setModels(merged);
      } else {
        setModels(data || []);
      }
      setLoading(false);
    });
    return () => {
      active = false;
    };
  }, [kind]);

  // Đóng dropdown khi click ngoài hoặc nhấn Escape, đồng thời nâng z-index của node chứa khi mở trên canvas
  useEffect(() => {
    function handleClickOutside(e: MouseEvent) {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        setIsOpen(false);
        setShowFallbackSelector(false);
      }
    }
    function handleKeyDown(e: KeyboardEvent) {
      if (e.key === "Escape") {
        setIsOpen(false);
        setShowFallbackSelector(false);
      }
    }
    const rfNode = containerRef.current?.closest(".react-flow__node") as HTMLElement | null;
    const prevZ = rfNode ? rfNode.style.zIndex : "";
    if (isOpen || showFallbackSelector) {
      if (rfNode) rfNode.style.zIndex = "1000";
      document.addEventListener("mousedown", handleClickOutside);
      window.addEventListener("keydown", handleKeyDown);
    }
    return () => {
      if (rfNode) rfNode.style.zIndex = prevZ;
      document.removeEventListener("mousedown", handleClickOutside);
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [isOpen, showFallbackSelector]);

  // Model hiện tại được chọn (hỗ trợ cả định dạng biến {model.<key>})
  const normalizedValue = value?.replace(/^\{model\.|\}$/g, "") || value;
  const isDynamicValue = Boolean(value && value.trim().startsWith("{") && value.trim().endsWith("}"));
  const selectedModel = useMemo(() => {
    return models.find(
      (m) =>
        m.key === value ||
        m.model_id === value ||
        m.key === normalizedValue ||
        m.model_id === normalizedValue
    );
  }, [models, value, normalizedValue]);

  // Danh sách model free hoặc local hợp lệ để làm fallback (tuyệt đối không chứa model paid)
  const eligibleFallbackModels = useMemo(() => {
    return models.filter(
      (m) =>
        (m.tier === "free" || m.tier === "local") &&
        m.key !== value &&
        m.model_id !== value
    );
  }, [models, value]);

  const selectedTier = selectedModel ? formatTierBadge(selectedModel.tier, selectedModel.pricing, selectedModel.key) : null;

  return (
    <div className={cn("space-y-2 text-footnote w-full min-w-0", className)} ref={containerRef}>
      {/* Nút chọn model chính */}
      <div className="relative w-full min-w-0">
        <button
          type="button"
          disabled={disabled || loading}
          onClick={() => setIsOpen(!isOpen)}
          aria-haspopup="listbox"
          aria-expanded={isOpen}
          className={cn(
            "nodrag w-full min-w-0 max-w-full flex items-center justify-between gap-2 rounded-ctl border border-border bg-card px-2 text-left transition-colors hover:bg-muted/40",
            disabled && "opacity-50 cursor-not-allowed",
            compact ? "h-8 py-1" : "min-h-9 py-1"
          )}
        >
          <div className="flex-1 min-w-0 max-w-full flex items-center gap-2 flex-nowrap overflow-hidden">
            {loading ? (
              <span className="text-muted-foreground animate-pulse truncate">Đang tải danh sách model...</span>
            ) : selectedModel ? (
              <>
                <span className="font-medium text-foreground truncate min-w-0 shrink">
                  {isDynamicValue ? value : selectedModel.display_name || selectedModel.key}
                </span>

                {/* Badge tier */}
                {selectedTier && (
                  <span
                    className={cn(
                      "text-caption font-semibold px-2 py-1 rounded-xs border leading-none shrink-0 whitespace-nowrap",
                      selectedTier.colorClass
                    )}
                  >
                    {locale === "vi" ? selectedTier.labelVi : selectedTier.labelEn}
                  </span>
                )}

                {/* Biểu tượng nhận ảnh DNA tham chiếu */}
                {selectedModel.supports_reference && (
                  <span
                    title="Hỗ trợ nạp ảnh mẫu DNA nhân vật"
                    className="inline-flex items-center gap-1 text-caption text-brand bg-brand/10 px-2 py-1 rounded-xs font-mono shrink-0 whitespace-nowrap leading-none"
                  >
                    <Sparkles className="h-2.5 w-2.5 shrink-0" />
                    Ref
                  </span>
                )}

                {/* Trạng thái key */}
                {!selectedModel.available && (
                  <span
                    title="Thiếu API key cho model này"
                    className="inline-flex items-center gap-1 text-caption text-amber-600 dark:text-amber-400 bg-amber-500/10 px-2 py-1 rounded-xs shrink-0 whitespace-nowrap leading-none"
                  >
                    <AlertTriangle className="h-2.5 w-2.5 shrink-0" />
                    Thiếu key
                  </span>
                )}
              </>
            ) : (
              <span className="text-muted-foreground font-mono truncate">
                {value || "Chọn model..."}
              </span>
            )}
          </div>

          <ChevronDown
            className={cn("h-3.5 w-3.5 shrink-0 text-muted-foreground transition-transform ml-1", isOpen && "rotate-180")}
          />
        </button>

        {/* Menu danh sách Model với Search Bar */}
        {isOpen && (
          <div className="nodrag nowheel absolute left-0 right-0 top-full mt-1 w-full min-w-[280px] max-w-[calc(100vw-32px)] rounded-panel border border-border bg-popover p-1 text-popover-foreground shadow-xl z-50 animate-in fade-in-50 zoom-in-95 max-h-80 overflow-y-auto">
            {/* Search Input Bar */}
            <div className="p-1 border-b border-border/60 sticky top-0 bg-popover z-10 space-y-1">
              <div className="relative">
                <Search className="absolute left-2 top-1/2 -translate-y-1/2 h-3.5 w-3.5 text-muted-foreground pointer-events-none" />
                <input
                  type="text"
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  placeholder="Tìm model (FLUX, Midjourney, Recraft...)"
                  spellCheck={false}
                  autoCorrect="off"
                  autoCapitalize="off"
                  className="nodrag w-full rounded-ctl border border-border/80 bg-muted/40 pl-8 pr-8 py-1 text-footnote text-foreground placeholder:text-muted-foreground/60 outline-none focus:border-brand"
                  onClick={(e) => e.stopPropagation()}
                />
                {searchQuery && (
                  <button
                    type="button"
                    onClick={(e) => { e.stopPropagation(); setSearchQuery(""); }}
                    className="nodrag absolute right-2 top-1/2 -translate-y-1/2 p-1 text-muted-foreground hover:text-foreground"
                  >
                    <X className="h-3 w-3" />
                  </button>
                )}
              </div>
              <div className="flex items-center justify-between gap-2 px-1 text-caption text-muted-foreground">
                <span className="truncate">{models.filter((m) => !searchQuery || (m.display_name || "").toLowerCase().includes(searchQuery.toLowerCase()) || (m.provider || "").toLowerCase().includes(searchQuery.toLowerCase())).length} mô hình AI</span>
                <span className="text-brand font-medium shrink-0 whitespace-nowrap">Bảng giá thương mại</span>
              </div>
            </div>

            <div className="space-y-1 pt-1">
              {models
                .filter((m) => !searchQuery || (m.display_name || "").toLowerCase().includes(searchQuery.toLowerCase()) || (m.provider || "").toLowerCase().includes(searchQuery.toLowerCase()))
                .map((m) => {
                const isSelected = m.key === value || m.model_id === value;
                const tier = formatTierBadge(m.tier, m.pricing, m.key);
                return (
                  <div
                    key={m.key}
                    onClick={() => {
                      onChange(m.key);
                      setIsOpen(false);
                    }}
                    className={cn(
                      "flex flex-col gap-1 rounded-ctl p-2 cursor-pointer transition-colors hover:bg-muted/60",
                      isSelected && "bg-muted/80 font-medium",
                      !m.available && "opacity-75"
                    )}
                  >
                    <div className="flex items-center justify-between gap-2 min-w-0">
                      <div className="flex items-center gap-2 min-w-0 flex-1">
                        {isSelected ? (
                          <Check className="h-3.5 w-3.5 text-brand shrink-0" />
                        ) : (
                          <span className="h-3.5 w-3.5 shrink-0" />
                        )}
                        <span className="font-semibold text-foreground text-footnote truncate">
                          {m.display_name || m.key}
                        </span>
                      </div>

                      <div className="flex items-center gap-1 shrink-0">
                        {/* Tier */}
                        <span
                          className={cn(
                            "text-caption font-semibold px-2 py-1 rounded-xs border leading-none whitespace-nowrap",
                            tier.colorClass
                          )}
                        >
                          {locale === "vi" ? tier.labelVi : tier.labelEn}
                        </span>

                        {/* Ref */}
                        {m.supports_reference && (
                          <span
                            title="Hỗ trợ nạp ảnh mẫu DNA nhân vật"
                            className="inline-flex items-center text-caption text-brand bg-brand/10 px-2 py-1 rounded-xs font-mono"
                          >
                            <Sparkles className="h-2.5 w-2.5" />
                          </span>
                        )}
                      </div>
                    </div>

                    {/* Dòng phụ: Nhà cung cấp, ghi chú quota, key */}
                    <div className="flex items-center justify-between text-caption text-muted-foreground pl-5 pr-1 gap-2 min-w-0">
                      <span className="capitalize shrink-0">{m.provider}</span>

                      {!m.available ? (
                        <Link
                          href={`/${locale}/settings`}
                          onClick={(e) => {
                            e.stopPropagation();
                            setIsOpen(false);
                          }}
                          className="text-amber-600 dark:text-amber-400 hover:underline flex items-center gap-1 font-medium shrink-0"
                        >
                          <AlertTriangle className="h-2.5 w-2.5" />
                          Chưa có key
                        </Link>
                      ) : m.quota_note ? (
                        <span className="truncate min-w-0 text-right text-caption" title={m.quota_note}>
                          {m.quota_note}
                        </span>
                      ) : (
                        <span className="text-emerald-600 dark:text-emerald-400 text-caption flex items-center gap-1 shrink-0">
                          <Check className="h-2.5 w-2.5" /> Sẵn sàng
                        </span>
                      )}
                    </div>
                  </div>
                );
              })}
            </div>
          </div>
        )}
      </div>

      {/* Phần chuỗi dự phòng (Fallback models) - Chỉ áp dụng model Free / Local */}
      {allowFallback && onFallbackChange && (
        <div className="rounded-card border border-dashed border-border/80 bg-muted/20 p-2 space-y-2">
          <div className="flex items-center justify-between text-caption">
            <span className="text-muted-foreground flex items-center gap-1">
              <Zap className="h-3 w-3 text-amber-500" />
              <span>Dự phòng khi hết hạn mức (chỉ nguồn $0):</span>
            </span>

            <button
              type="button"
              disabled={disabled || eligibleFallbackModels.length === 0}
              onClick={() => setShowFallbackSelector(!showFallbackSelector)}
              className="text-brand hover:underline flex items-center gap-1 font-medium disabled:opacity-50 py-1"
            >
              <Plus className="h-3 w-3" />
              Thêm
            </button>
          </div>

          {/* Danh sách fallback đã chọn */}
          {fallbackModels.length > 0 ? (
            <div className="flex flex-wrap gap-1 pt-1">
              {fallbackModels.map((fbKey) => {
                const fbModel = models.find((m) => m.key === fbKey || m.model_id === fbKey);
                const name = fbModel?.display_name || fbKey;
                return (
                  <span
                    key={fbKey}
                    className="inline-flex items-center gap-1 rounded-xs bg-background px-2 py-1 text-caption font-mono border border-border text-foreground shadow-xs"
                  >
                    <span>{name}</span>
                    <button
                      type="button"
                      onClick={() => {
                        onFallbackChange(fallbackModels.filter((k) => k !== fbKey));
                      }}
                      className="text-muted-foreground hover:text-foreground p-1"
                    >
                      <X className="h-2.5 w-2.5" />
                    </button>
                  </span>
                );
              })}
            </div>
          ) : (
            <p className="text-caption text-muted-foreground italic">
              Chưa chọn model dự phòng (tự động bỏ qua nếu lỗi hạn mức)
            </p>
          )}

          {/* Menu chọn thêm fallback */}
          {showFallbackSelector && (
            <div className="mt-1 rounded-card border bg-popover p-2 shadow-md space-y-1">
              <div className="text-caption font-semibold text-muted-foreground px-1">
                Chọn nguồn Free / Local dự phòng:
              </div>
              {eligibleFallbackModels.length === 0 ? (
                <div className="text-caption text-muted-foreground px-1 py-1">
                  Không còn model $0 khả dụng
                </div>
              ) : (
                eligibleFallbackModels.map((em) => {
                  const isChecked = fallbackModels.includes(em.key);
                  return (
                    <button
                      key={em.key}
                      type="button"
                      onClick={() => {
                        if (isChecked) {
                          onFallbackChange(fallbackModels.filter((k) => k !== em.key));
                        } else {
                          onFallbackChange([...fallbackModels, em.key]);
                        }
                      }}
                      className={cn(
                        "w-full flex items-center justify-between rounded-ctl px-2 py-1 text-left text-caption transition-colors hover:bg-muted",
                        isChecked && "bg-brand/10 text-brand font-medium"
                      )}
                    >
                      <span>{em.display_name || em.key}</span>
                      {isChecked ? (
                        <Check className="h-3 w-3 text-brand" />
                      ) : (
                        <span className="text-caption text-muted-foreground capitalize">{em.tier}</span>
                      )}
                    </button>
                  );
                })
              )}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
