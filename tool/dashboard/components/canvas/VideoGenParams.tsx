"use client";

import React, { useState, useEffect, useRef, useMemo, useCallback } from "react";
import { cn } from "@/lib/utils";
import { useStudioStore } from "@/store";
import { fetchModels, type ModelItem } from "@/lib/api/models";
import { DynamicValueSelector } from "./DynamicValueSelector";
import {
  Sparkles,
  DollarSign,
  Volume2,
  VolumeX,
  Sliders,
  ChevronDown,
  ChevronUp,
  Clock,
  Hash,
  ChevronRight,
  Check,
  Search,
  X,
} from "@/components/ui/icons";
import { FALLBACK_VIDEO_MODELS } from "./fallbackVideoModels";
import { getModelLogo, formatModelName } from "./videoModelLogos";

export interface VideoGenParamsProps {
  nodeId: string;
  params?: Record<string, any>;
  connectedPrompt?: boolean;
  expanded?: boolean;
}

const ASPECT_RATIOS = [
  { id: "9:16", label: "9:16", hint: "TikTok / Reel" },
  { id: "16:9", label: "16:9", hint: "Ngang TV" },
  { id: "1:1", label: "1:1", hint: "Vuông" },
  { id: "4:5", label: "4:5", hint: "Feed dọc" },
] as const;

export const VideoGenParams = React.memo(function VideoGenParams({
  nodeId,
  params = {},
  connectedPrompt = false,
  expanded = false,
}: VideoGenParamsProps) {
  const updateNodeParams = useStudioStore((s) => s.updateNodeParams);
  const setSelectedNodeId = useStudioStore((s) => s.setSelectedNodeId);

  const [models, setModels] = useState<ModelItem[]>(FALLBACK_VIDEO_MODELS);
  const [showAdvanced, setShowAdvanced] = useState(false);
  const [expandPrompt] = useState(false);
  const [isPromptFocused, setIsPromptFocused] = useState(false);
  const [modelMenuOpen, setModelMenuOpen] = useState(false);
  const [searchQuery, setSearchQuery] = useState("");
  const menuRef = useRef<HTMLDivElement | null>(null);
  const promptRef = useRef<HTMLTextAreaElement | null>(null);
  const isMouseDownRef = useRef(false);

  // Đóng menu model khi click ra ngoài hoặc nhấn Escape, đồng thời nâng z-index của node chứa khi mở trên canvas
  useEffect(() => {
    const rfNode = menuRef.current?.closest(".react-flow__node") as HTMLElement | null;
    const prevZ = rfNode ? rfNode.style.zIndex : "";
    if (!modelMenuOpen) return;
    if (rfNode) rfNode.style.zIndex = "1000";
    const handleClickOutside = (e: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        setModelMenuOpen(false);
      }
    };
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        setModelMenuOpen(false);
      }
    };
    document.addEventListener("mousedown", handleClickOutside);
    window.addEventListener("keydown", handleKeyDown);
    return () => {
      if (rfNode) rfNode.style.zIndex = prevZ;
      document.removeEventListener("mousedown", handleClickOutside);
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [modelMenuOpen]);

  // Tham số hiện tại
  const currentModel = params.model || "mock-video";
  const variants = Math.max(1, Math.min(4, Number(params.variants || 1)));
  const aspectRatio = params.aspectRatio || "9:16";
  const resolution = params.resolution || "720p";
  const durationSec = Number(params.durationSec || 5);
  const hasAudio = Boolean(params.hasAudio);
  const seed = params.seed ?? "";
  const initialPrompt = params.prompt || "";
  const [promptVal, setPromptVal] = useState(initialPrompt);

  const debounceTimer = useRef<NodeJS.Timeout | null>(null);

  useEffect(() => {
    setPromptVal(initialPrompt || "");
  }, [initialPrompt]);

  // Nạp danh sách model kind="video" từ API
  useEffect(() => {
    let active = true;
    fetchModels("video").then((fetched) => {
      if (active && fetched && fetched.length > 0) {
        setModels(fetched);
      }
    });
    return () => {
      active = false;
    };
  }, []);

  // Model được chọn
  const selectedModel = useMemo(() => {
    return (
      models.find((m) => m.key === currentModel || m.model_id === currentModel) ||
      models[0]
    );
  }, [models, currentModel]);

  // Bộ lọc tìm kiếm model AI
  const filteredModels = useMemo(() => {
    if (!searchQuery.trim()) return models;
    const q = searchQuery.toLowerCase().trim();
    return models.filter((m) => {
      const formatted = formatModelName(m);
      return (
        m.key.toLowerCase().includes(q) ||
        m.display_name.toLowerCase().includes(q) ||
        m.provider.toLowerCase().includes(q) ||
        formatted.name.toLowerCase().includes(q) ||
        formatted.brand.toLowerCase().includes(q)
      );
    });
  }, [models, searchQuery]);

  const activeFormatted = useMemo(() => formatModelName(selectedModel), [selectedModel]);
  const ActiveLogo = useMemo(() => getModelLogo(selectedModel.key, selectedModel.provider), [selectedModel]);

  // Các resolution model này hỗ trợ
  const supportedResolutions: ("720p" | "1080p" | "4k")[] = useMemo(() => {
    const raw = selectedModel?.capabilities?.resolutions;
    if (Array.isArray(raw) && raw.length > 0) {
      return raw.filter((r): r is "720p" | "1080p" | "4k" =>
        ["720p", "1080p", "4k"].includes(r)
      );
    }
    return ["720p", "1080p"];
  }, [selectedModel]);

  useEffect(() => {
    if (resolution && !supportedResolutions.includes(resolution as any)) {
      updateNodeParams(nodeId, { resolution: supportedResolutions[0] });
    }
  }, [supportedResolutions, resolution, nodeId, updateNodeParams]);

  // Tính chi phí: giá/giây × durationSec × variants (theo bảng giá thương mại chính thức)
  const isMock = currentModel === "mock-video" || selectedModel?.provider === "mock";
  const costPerSec = isMock ? 0 : (selectedModel?.pricing?.cost_per_second ?? 0.05);
  const totalCost = Number((costPerSec * durationSec * variants).toFixed(4));

  useEffect(() => {
    const el = promptRef.current;
    if (!el) return;
    const prev = el.offsetHeight;
    el.style.height = "auto";
    const target = (expandPrompt || isPromptFocused)
      ? Math.min(Math.max(el.scrollHeight, 120), 400)
      : 56;
    el.style.height = `${prev}px`;
    void el.offsetHeight; // force reflow
    el.style.height = `${target}px`;
  }, [promptVal, isPromptFocused, expandPrompt]);

  const wasFocusedRef = useRef(false);

  // Đảm bảo con trỏ LUÔN nhảy xuống cuối cùng và cuộn tới cuối ngay khi click vào ô nhập
  const moveCursorToEnd = useCallback((el: HTMLTextAreaElement) => {
    const applyPos = () => {
      if (!el) return;
      const len = el.value.length;
      el.setSelectionRange(len, len);
      el.scrollTop = el.scrollHeight;
    };
    applyPos();
    requestAnimationFrame(applyPos);
    setTimeout(applyPos, 0);
    setTimeout(applyPos, 30);
    setTimeout(applyPos, 80);
    setTimeout(applyPos, 150);
  }, []);

  const handlePromptFocus = (e: React.FocusEvent<HTMLTextAreaElement>) => {
    setIsPromptFocused(true);
    moveCursorToEnd(e.currentTarget);
  };

  const handlePromptMouseDown = () => {
    wasFocusedRef.current = isPromptFocused;
    isMouseDownRef.current = true;
  };

  const handlePromptMouseUp = (e: React.MouseEvent<HTMLTextAreaElement>) => {
    if (!wasFocusedRef.current) {
      moveCursorToEnd(e.currentTarget);
    }
    isMouseDownRef.current = false;
  };

  const handlePromptClick = (e: React.MouseEvent<HTMLTextAreaElement>) => {
    if (!wasFocusedRef.current) {
      moveCursorToEnd(e.currentTarget);
      wasFocusedRef.current = true;
    }
  };

  const handlePromptBlur = () => {
    setIsPromptFocused(false);
    wasFocusedRef.current = false;
    isMouseDownRef.current = false;
  };

  const handlePromptChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    const newVal = e.target.value;
    setPromptVal(newVal);
    if (debounceTimer.current) clearTimeout(debounceTimer.current);
    debounceTimer.current = setTimeout(() => {
      updateNodeParams(nodeId, { prompt: newVal });
    }, 350);
  };

  // =========================================================================
  // 1. CHẾ ĐỘ THU GỌN (UNFOCUSED / COMPACT - Khi chưa chọn node)
  // =========================================================================
  if (!expanded) {
    return (
      <div
        role="button"
        tabIndex={0}
        onClick={(e) => {
          e.stopPropagation();
          setSelectedNodeId(nodeId);
        }}
        className="group/compact cursor-pointer space-y-2 rounded-ctl p-2 transition-colors hover:bg-muted/40 text-footnote"
        title="Nhấn để mở rộng & cấu hình chi tiết video"
      >
        {/* Dải Chips / Pills tóm tắt gọn gàng */}
        <div className="flex flex-wrap items-center gap-2">
          <span className="inline-flex items-center gap-2 rounded-ctl bg-muted/70 px-2 py-1 font-medium text-foreground text-caption">
            <ActiveLogo className="h-3.5 w-3.5 shrink-0 rounded-xs" />
            <span className="max-w-[130px] truncate">
              {activeFormatted.name}
            </span>
          </span>

          <span className="rounded-ctl bg-muted/60 px-2 py-1 font-mono text-caption text-muted-foreground font-semibold">
            {aspectRatio}
          </span>

          <span className="rounded-ctl bg-muted/60 px-2 py-1 font-mono text-caption text-muted-foreground uppercase">
            {resolution}
          </span>

          <span className="rounded-ctl bg-muted/60 px-2 py-1 font-mono text-caption text-muted-foreground">
            {durationSec}s
          </span>

          {variants > 1 && (
            <span className="rounded-ctl bg-muted/60 px-2 py-1 font-mono text-caption text-muted-foreground">
              x{variants}
            </span>
          )}

          {hasAudio && (
            <span
              className="rounded-ctl bg-muted/60 px-2 py-1 text-caption text-brand inline-flex items-center gap-1"
              title="Có âm thanh AI"
            >
              <Volume2 className="h-3.5 w-3.5" />
            </span>
          )}

          <span className="ml-auto font-mono text-caption font-semibold px-2 py-1 rounded-ctl bg-brand/10 text-brand">
            ${totalCost > 0 ? totalCost.toFixed(2) : "0.00"} USD
          </span>
        </div>

        {/* 1 Dòng preview prompt ngắn gọn */}
        {connectedPrompt ? (
          <div className="flex items-center gap-2 rounded-ctl border border-dashed border-border bg-muted/20 px-2 py-1 text-caption text-muted-foreground">
            <Sparkles className="h-3.5 w-3.5 text-brand shrink-0" />
            <span className="truncate">Prompt nhận tự động từ dây nối</span>
          </div>
        ) : promptVal ? (
          <div
            className="rounded-ctl border border-border/60 bg-muted/20 px-2 py-1 text-caption text-muted-foreground leading-snug line-clamp-2"
            title={promptVal}
          >
            <span className="font-semibold text-foreground/80">Kịch bản: </span>
            {promptVal}
          </div>
        ) : (
          <div className="text-caption italic text-muted-foreground/60 px-1 flex items-center justify-between">
            <span>Chưa có prompt kịch bản.</span>
            <span className="flex items-center gap-1 text-brand not-italic opacity-0 group-hover/compact:opacity-100 transition-opacity">
              Mở rộng <ChevronRight className="h-3.5 w-3.5" />
            </span>
          </div>
        )}
      </div>
    );
  }

  // =========================================================================
  // 2. CHẾ ĐỘ MỞ RỘNG (FOCUSED / EXPANDED - RỘNG 420px THOÁNG ĐÃNG)
  // =========================================================================
  return (
    <div className="space-y-3 pt-1 text-footnote w-full min-w-0">
      {/* 1. CHỌN MÔ HÌNH VIDEO VỚI LOGO THƯƠNG HIỆU & GIÁ THỜI GIAN THỰC */}
      <div className="relative w-full min-w-0" ref={menuRef}>
        {/* Custom Modern Model Selector */}
        <button
          type="button"
          onClick={() => setModelMenuOpen((v) => !v)}
          className="nodrag flex h-8 w-full min-w-0 max-w-full items-center justify-between rounded-ctl border border-border bg-card px-3 text-footnote text-foreground transition-all hover:border-foreground/60 focus:outline-none"
        >
          <div className="flex items-center gap-2 min-w-0 truncate">
            <ActiveLogo className="h-4 w-4 shrink-0 rounded-xs" />
            <span className="truncate font-semibold text-foreground text-footnote">{activeFormatted.name}</span>
            <span className="text-caption text-muted-foreground hidden sm:inline truncate">· {activeFormatted.brand}</span>
          </div>
          <div className="flex items-center gap-2 shrink-0 ml-2">
            <span className="font-mono text-caption font-semibold px-2 py-1 rounded-xs bg-brand/10 text-brand">
              ${costPerSec > 0 ? costPerSec.toFixed(2) : "0.00"}/s
            </span>
            <ChevronDown
              className={cn("h-3.5 w-3.5 text-muted-foreground transition-transform duration-200", modelMenuOpen && "rotate-180")}
            />
          </div>
        </button>

        {/* Floating Dropdown List với ô tìm kiếm Search */}
        {modelMenuOpen && (
          <div className="nodrag nowheel absolute left-0 right-0 top-full z-50 mt-1 max-h-72 overflow-y-auto rounded-card border border-border bg-popover shadow-xl text-footnote animate-in fade-in zoom-in-95 duration-150">
            {/* Search Input Bar */}
            <div className="p-2 border-b border-border/60 sticky top-0 bg-popover z-10 space-y-1">
              <div className="relative">
                <Search className="absolute left-2 top-1/2 -translate-y-1/2 h-3.5 w-3.5 text-muted-foreground" />
                <input
                  type="text"
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  placeholder="Tìm model (Runway, Luma, Kling, Wan, Pika, Vidu...)..."
                  spellCheck={false}
                  autoCorrect="off"
                  autoCapitalize="off"
                  className="nodrag w-full h-7 rounded-ctl border border-border/80 bg-muted/40 pl-8 pr-6 py-1 text-footnote text-foreground placeholder:text-muted-foreground/60 outline-none focus:border-brand"
                  onClick={(e) => e.stopPropagation()}
                />
                {searchQuery && (
                  <button
                    type="button"
                    onClick={(e) => { e.stopPropagation(); setSearchQuery(""); }}
                    className="nodrag absolute right-2 top-1/2 -translate-y-1/2 p-1 text-muted-foreground hover:text-foreground"
                  >
                    <X className="h-3.5 w-3.5" />
                  </button>
                )}
              </div>
              <div className="flex items-center justify-between px-1 text-caption text-muted-foreground">
                <span>{filteredModels.length} mô hình AI Video</span>
                <span className="text-brand font-medium">Bảng giá thương mại</span>
              </div>
            </div>

            <div className="p-1 space-y-1">
              {filteredModels.length === 0 ? (
                <div className="p-3 text-center text-footnote text-muted-foreground">
                  Không tìm thấy mô hình nào phù hợp với &ldquo;{searchQuery}&rdquo;
                </div>
              ) : (
                filteredModels.map((m) => {
                  const formatted = formatModelName(m);
                  const LogoComp = getModelLogo(m.key, m.provider);
                  const isSelected = m.key === currentModel || m.model_id === currentModel;
                  const isMockModel = m.key === "mock-video" || m.provider === "mock";
                  const pCost = isMockModel ? 0 : (m.pricing?.cost_per_second ?? 0.05);

                  return (
                    <button
                      key={m.key}
                      type="button"
                      onClick={() => {
                        updateNodeParams(nodeId, { model: m.key });
                        setModelMenuOpen(false);
                      }}
                      className={cn(
                        "flex w-full items-center justify-between rounded-ctl px-3 py-2 text-left transition-colors",
                        isSelected
                          ? "bg-accent text-accent-foreground font-semibold"
                          : "hover:bg-muted/70 text-foreground"
                      )}
                    >
                      <div className="flex items-center gap-2 min-w-0">
                        <LogoComp className="h-4 w-4 shrink-0 rounded-xs" />
                        <div className="min-w-0">
                          <div className="truncate text-footnote font-medium text-foreground">{formatted.name}</div>
                          <div className="truncate text-caption text-muted-foreground">{formatted.brand}</div>
                        </div>
                      </div>

                      <div className="flex items-center gap-2 shrink-0 ml-2">
                        <span className={cn(
                          "font-mono text-caption font-semibold px-2 py-1 rounded-xs",
                          pCost > 0 ? "bg-brand/10 text-brand" : "bg-muted text-muted-foreground"
                        )}>
                          {pCost > 0 ? `${pCost.toFixed(2)}/s` : "$0.00 (Test)"}
                        </span>
                        {isSelected && <Check className="h-3.5 w-3.5 text-foreground shrink-0" />}
                      </div>
                    </button>
                  );
                })
              )}
            </div>
          </div>
        )}
      </div>

      {/* 2. PROMPT CHUYỂN ĐỘNG / KỊCH BẢN */}
      <div>
        {connectedPrompt ? (
          <div className="flex w-full items-center justify-between rounded-ctl border border-brand/30 bg-brand/5 p-2 text-footnote">
            <div className="flex items-center gap-2 min-w-0">
              <span className="flex h-5 w-5 items-center justify-center rounded-xs bg-brand/15 text-brand font-mono font-bold text-caption shrink-0">
                {`{}`}
              </span>
              <span className="font-mono font-semibold text-brand text-footnote truncate">
                {promptVal && promptVal.startsWith("{") ? promptVal : "{Kịch bản.prompt}"}
              </span>
            </div>
            <div className="flex items-center gap-1 shrink-0 ml-2">
              <span className="text-caption bg-muted px-2 py-1 rounded-xs font-mono">Tự động</span>
              <DynamicValueSelector
                nodeId={nodeId}
                portName="prompt"
                currentValue={promptVal}
                onSelectValue={(val) => {
                  setPromptVal(val);
                  updateNodeParams(nodeId, { prompt: val });
                }}
              />
            </div>
          </div>
        ) : promptVal && promptVal.trim().startsWith("{") && promptVal.trim().endsWith("}") && !promptVal.includes("\n") && !isPromptFocused ? (
          <div
            style={{ minHeight: "64px" }}
            className="flex w-full items-center justify-between rounded-ctl border border-brand/40 bg-brand/5 px-3 py-2 text-footnote transition-all"
          >
            <div
              role="button"
              tabIndex={0}
              onClick={() => setIsPromptFocused(true)}
              className="nodrag flex flex-col justify-center min-w-0 flex-1 cursor-pointer pr-2 select-none"
            >
              <div className="flex items-center gap-1 min-w-0">
                <span className="flex h-5 w-5 items-center justify-center rounded-xs bg-brand/15 text-brand font-mono font-bold text-caption shrink-0">
                  {`{}`}
                </span>
                <span className="font-mono font-semibold text-brand text-footnote truncate" title={promptVal.trim()}>
                  {promptVal.trim()}
                </span>
              </div>
              <span className="text-caption text-muted-foreground/80 mt-1 truncate">
                Giá trị tham chiếu động · Bấm để sửa tay
              </span>
            </div>
            <div className="flex items-center gap-1 shrink-0 ml-2">
              <DynamicValueSelector
                nodeId={nodeId}
                portName="prompt"
                currentValue={promptVal}
                onSelectValue={(val) => {
                  setPromptVal(val);
                  updateNodeParams(nodeId, { prompt: val });
                }}
              />
              <button
                type="button"
                title="Xoá giá trị"
                aria-label="Xoá giá trị"
                onClick={() => {
                  setPromptVal("");
                  updateNodeParams(nodeId, { prompt: "" });
                }}
                className="nodrag flex h-7 w-7 items-center justify-center rounded-ctl text-muted-foreground hover:bg-destructive/10 hover:text-destructive cursor-pointer transition-colors"
              >
                <X className="h-4 w-4" />
              </button>
            </div>
          </div>
        ) : (
          <div className="relative group/prompt">
            <textarea
              ref={promptRef}
              rows={1}
              spellCheck={false}
              autoCorrect="off"
              autoCapitalize="off" 
              value={promptVal}
              onFocus={handlePromptFocus}
              onMouseDown={handlePromptMouseDown}
              onMouseUp={handlePromptMouseUp}
              onClick={handlePromptClick}
              onBlur={handlePromptBlur}
              onChange={handlePromptChange}
              onKeyDown={(e) => {
                if (e.key === "Escape") {
                  (e.target as HTMLElement).blur();
                  setIsPromptFocused(false);
                }
              }}
              placeholder="Mô tả chi tiết kịch bản, camera, hành động nhân vật..."
              className={cn(
                "nodrag nowheel block w-full resize-none rounded-ctl border p-3 pr-20 text-footnote leading-relaxed text-foreground placeholder:text-muted-foreground/60 transition-[height,background-color,border-color,box-shadow] duration-200 ease-out focus:outline-none",
                isPromptFocused
                  ? "border-foreground bg-card ring-1 ring-ring/30 shadow-xs"
                  : "border-border/60 bg-muted/20 hover:border-foreground/40"
              )}
            />
            <div className="absolute right-2 top-2 flex items-center gap-1 z-10">
              <DynamicValueSelector
                nodeId={nodeId}
                portName="prompt"
                currentValue={promptVal}
                onSelectValue={(val) => {
                  setPromptVal(val);
                  updateNodeParams(nodeId, { prompt: val });
                }}
              />
              {Boolean(promptVal) && (
                <button
                  type="button"
                  onMouseDown={(e) => e.preventDefault()}
                  onClick={() => {
                    setPromptVal("");
                    updateNodeParams(nodeId, { prompt: "" });
                  }}
                  title="Xoá nội dung"
                  aria-label="Xoá nội dung"
                  className="nodrag flex h-6 w-6 items-center justify-center rounded-xs bg-muted/80 text-muted-foreground hover:bg-destructive/10 hover:text-destructive cursor-pointer transition-colors"
                >
                  <X className="h-3 w-3" />
                </button>
              )}
            </div>
          </div>
        )}
      </div>

      {/* 3. TỶ LỆ KHUNG HÌNH (Tối giản 4 cột) */}
      <div className="grid grid-cols-4 gap-1 p-1 w-full min-w-0 rounded-ctl bg-muted/40 border border-border/60">
        {ASPECT_RATIOS.map((r) => (
          <button
            key={r.id}
            type="button"
            onClick={() => updateNodeParams(nodeId, { aspectRatio: r.id })}
            className={cn(
              "nodrag h-7 min-w-0 rounded-ctl text-footnote font-medium transition-all text-center flex flex-col items-center justify-center py-1",
              aspectRatio === r.id
                ? "bg-card text-foreground font-semibold shadow-xs border border-border/80"
                : "text-muted-foreground hover:text-foreground hover:bg-card/50"
            )}
            title={r.hint}
          >
            <span className="truncate w-full px-1">{r.label}</span>
          </button>
        ))}
      </div>

      {/* 4. ĐỘ PHÂN GIẢI & THỜI LƯỢNG (Tối giản 2 cột) */}
      <div className="grid grid-cols-2 gap-2">
        <div
          className={cn(
            "grid gap-1 p-1 rounded-ctl bg-muted/40 border border-border/60",
            supportedResolutions.length === 1
              ? "grid-cols-1"
              : supportedResolutions.length === 2
              ? "grid-cols-2"
              : "grid-cols-3"
          )}
        >
          {supportedResolutions.map((res) => (
            <button
              key={res}
              type="button"
              onClick={() => updateNodeParams(nodeId, { resolution: res })}
              className={cn(
                "nodrag h-7 rounded-ctl text-footnote font-mono font-medium transition-all uppercase flex items-center justify-center",
                resolution === res
                  ? "bg-card text-foreground font-bold shadow-xs border border-border/80"
                  : "text-muted-foreground hover:text-foreground hover:bg-card/50"
              )}
            >
              {res}
            </button>
          ))}
        </div>

        <div className="flex h-8 items-center rounded-ctl border border-border bg-card px-3 focus-within:border-foreground transition-all">
          <Clock className="h-3.5 w-3.5 text-muted-foreground shrink-0 mr-2" />
          <input
            type="number"
            min={1}
            max={60}
            value={durationSec}
            onChange={(e) =>
              updateNodeParams(nodeId, {
                durationSec: Math.max(1, Number(e.target.value) || 5),
              })
            }
            className="nodrag h-full w-full bg-transparent text-footnote text-foreground font-mono font-medium focus:outline-none"
          />
          <span className="text-footnote text-muted-foreground font-mono ml-1">s</span>
        </div>
      </div>

      {/* 5. SỐ BIẾN THỂ & ÂM THANH (Tối giản 2 cột) */}
      <div className="grid grid-cols-2 gap-2">
        <div className="grid grid-cols-4 gap-1 p-1 rounded-ctl bg-muted/40 border border-border/60">
          {[1, 2, 3, 4].map((v) => (
            <button
              key={v}
              type="button"
              onClick={() => updateNodeParams(nodeId, { variants: v })}
              className={cn(
                "nodrag h-7 rounded-ctl text-footnote font-mono font-medium transition-all flex items-center justify-center",
                variants === v
                  ? "bg-card text-foreground font-bold shadow-xs border border-border/80"
                  : "text-muted-foreground hover:text-foreground hover:bg-card/50"
              )}
            >
              x{v}
            </button>
          ))}
        </div>

        <div className="grid grid-cols-2 gap-1 p-1 rounded-ctl bg-muted/40 border border-border/60">
          <button
            type="button"
            onClick={() => updateNodeParams(nodeId, { hasAudio: false })}
            className={cn(
              "nodrag h-7 rounded-ctl text-footnote font-medium transition-all flex items-center justify-center gap-1",
              !hasAudio
                ? "bg-card text-foreground font-semibold shadow-xs border border-border/80"
                : "text-muted-foreground hover:text-foreground hover:bg-card/50"
            )}
          >
            <VolumeX className="h-3.5 w-3.5" />
            <span>Tắt</span>
          </button>
          <button
            type="button"
            onClick={() => updateNodeParams(nodeId, { hasAudio: true })}
            className={cn(
              "nodrag h-7 rounded-ctl text-footnote font-medium transition-all flex items-center justify-center gap-1",
              hasAudio
                ? "bg-card text-brand font-semibold shadow-xs border border-border/80"
                : "text-muted-foreground hover:text-foreground hover:bg-card/50"
            )}
          >
            <Volume2 className="h-3.5 w-3.5 text-brand" />
            <span>Bật</span>
          </button>
        </div>
      </div>

      {/* 6. ƯỚC TÍNH CHI PHÍ = GIÁ/GIÂY × DURATIONSEC × VARIANTS */}
      <div className="rounded-card border border-brand/30 bg-brand/5 p-3 space-y-1">
        <div className="flex items-center justify-between text-footnote">
          <span className="font-semibold text-foreground flex items-center gap-2">
            <DollarSign className="h-3.5 w-3.5 text-brand" />
            Ước tính chi phí:
          </span>
          <span className="font-mono font-bold text-footnote text-brand">
            ${totalCost > 0 ? totalCost.toFixed(2) : "0.00"} USD
          </span>
        </div>
        <div className="text-caption text-muted-foreground font-mono flex items-center justify-between">
          <span>Công thức:</span>
          <span>
            ${costPerSec.toFixed(2)}/s × {durationSec}s × {variants} clip
          </span>
        </div>
      </div>

      {/* 7. NÂNG CAO (ACCORDION GỌN, PHÂN BỔ ĐỀU) */}
      <div className="pt-1">
        <button
          type="button"
          onClick={() => setShowAdvanced((v) => !v)}
          className="nodrag flex items-center justify-between w-full rounded-ctl px-2 py-1 text-caption font-medium text-muted-foreground hover:text-foreground hover:bg-muted/40 transition-colors"
        >
          <span className="flex items-center gap-2">
            <Sliders className="h-3 w-3" />
            Cấu hình phân cảnh nâng cao (Seed, Shots C4)
          </span>
          {showAdvanced ? (
            <ChevronUp className="h-3 w-3" />
          ) : (
            <ChevronDown className="h-3 w-3" />
          )}
        </button>

        {showAdvanced && (
          <div className="mt-2 rounded-card border border-border/70 bg-card/60 p-3 space-y-3 text-footnote">
            {/* Seed & MaxClips trên 2 cột đều nhau */}
            <div className="grid grid-cols-2 gap-3">
              <div className="space-y-1">
                <label className="text-caption font-medium text-muted-foreground flex items-center gap-1">
                  <Hash className="h-3 w-3" /> Seed (tùy chọn):
                </label>
                <input
                  type="number"
                  value={seed ?? ""}
                  onChange={(e) =>
                    updateNodeParams(nodeId, {
                      seed: e.target.value === "" ? null : Number(e.target.value),
                    })
                  }
                  placeholder="Ngẫu nhiên"
                  className="nodrag h-7 w-full rounded-ctl border border-border bg-card px-3 text-footnote text-foreground font-mono placeholder:text-muted-foreground/60 focus:border-foreground focus:outline-none"
                />
              </div>

              <div className="space-y-1">
                <label className="text-caption font-medium text-muted-foreground">
                  Số clip tối đa:
                </label>
                <input
                  type="number"
                  min={1}
                  max={20}
                  value={params.maxClips ?? 5}
                  onChange={(e) =>
                    updateNodeParams(nodeId, {
                      maxClips: Number(e.target.value) || 5,
                    })
                  }
                  className="nodrag h-7 w-full rounded-ctl border border-border bg-card px-3 text-footnote text-foreground font-mono focus:border-foreground focus:outline-none"
                />
              </div>
            </div>

            {/* perShot & clipSource */}
            <div className="pt-2 border-t border-border/50 flex flex-wrap items-center justify-between gap-2">
              <label className="flex items-center gap-2 cursor-pointer select-none">
                <input
                  type="checkbox"
                  checked={params.perShot ?? true}
                  onChange={(e) =>
                    updateNodeParams(nodeId, { perShot: e.target.checked })
                  }
                  className="nodrag rounded-xs border-border text-brand focus:ring-brand h-4 w-4"
                />
                <span className="font-medium text-foreground text-caption">
                  Sinh clip theo từng shot
                </span>
              </label>

              {(params.perShot ?? true) && (
                <select
                  value={params.clipSource ?? "auto"}
                  onChange={(e) =>
                    updateNodeParams(nodeId, {
                      clipSource: e.target.value as any,
                    })
                  }
                  className="nodrag h-7 min-w-0 max-w-[180px] rounded-ctl border border-border bg-card px-2 text-caption text-muted-foreground focus:text-foreground"
                >
                  <option value="auto">Tự động (Ưu tiên thủ công → AI)</option>
                  <option value="manual">Chỉ clip thủ công</option>
                  <option value="ai">Chỉ sinh AI</option>
                </select>
              )}
            </div>
          </div>
        )}
      </div>
    </div>
  );
});
