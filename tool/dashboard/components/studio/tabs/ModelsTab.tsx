"use client";

import React, { useState, useMemo } from "react";
import dynamic from "next/dynamic";
import {
  Sparkles,
  Search,
  Film,
  Images,
  Volume2,
  FileText,
  Check,
  Copy,
  DollarSign,
  Sliders,
  Maximize2,
} from "@/components/ui/icons";
import { EXTENDED_MODELS_CATALOG } from "@/lib/data/extendedModelsCatalog";
import { useStudioStore } from "@/store";
import { cn } from "@/lib/utils";

const AIModelsLibraryModal = dynamic(
  () => import("@/components/studio/AIModelsLibraryModal").then((m) => m.AIModelsLibraryModal),
  { ssr: false }
);

export function ModelsTab() {
  const [selectedCategory, setSelectedCategory] = useState<
    "all" | "video" | "image" | "voice" | "text"
  >("all");
  const [search, setSearch] = useState("");
  const [activeModelKey, setActiveModelKey] = useState<string>("kling-v3-std");
  const [copiedKey, setCopiedKey] = useState(false);
  const [showFullModal, setShowFullModal] = useState(false);

  // Store actions với narrow selector tránh re-render khi dragging
  const selectedNodeId = useStudioStore((s) => s.selectedNodeId);
  const selectedNode = useStudioStore((s) =>
    s.selectedNodeId ? s.nodes.find((n) => n.id === s.selectedNodeId) || null : null
  );
  const updateNodeParams = useStudioStore((s) => s.updateNodeParams);

  const filteredModels = useMemo(() => {
    return EXTENDED_MODELS_CATALOG.filter((m) => {
      const matchCategory =
        selectedCategory === "all" || m.category === selectedCategory;
      const q = search.trim().toLowerCase();
      const matchSearch =
        !q ||
        m.name.toLowerCase().includes(q) ||
        m.brand.toLowerCase().includes(q) ||
        m.description.toLowerCase().includes(q) ||
        m.key.toLowerCase().includes(q) ||
        m.recommendedUse.toLowerCase().includes(q);
      return matchCategory && matchSearch;
    });
  }, [selectedCategory, search]);

  const activeModel = useMemo(() => {
    return (
      EXTENDED_MODELS_CATALOG.find((m) => m.key === activeModelKey) ||
      filteredModels[0] ||
      EXTENDED_MODELS_CATALOG[0]
    );
  }, [activeModelKey, filteredModels]);

  const handleCopyKey = (key: string) => {
    if (typeof navigator !== "undefined" && navigator.clipboard) {
      navigator.clipboard.writeText(key);
      setCopiedKey(true);
      setTimeout(() => setCopiedKey(false), 1800);
    }
  };

  const handleApplyToSelectedNode = () => {
    if (!selectedNodeId || !selectedNode) return;
    const nodeType = selectedNode.data?.nodeType;
    if (nodeType === "video_gen" && activeModel.category === "video") {
      updateNodeParams(selectedNodeId, { model: activeModel.key });
    } else if (
      (nodeType === "image" || nodeType === "generate") &&
      activeModel.category === "image"
    ) {
      updateNodeParams(selectedNodeId, { model: activeModel.key });
    } else {
      updateNodeParams(selectedNodeId, { model: activeModel.key });
    }
  };

  const categoryIcons = {
    all: Sparkles,
    video: Film,
    image: Images,
    voice: Volume2,
    text: FileText,
  };

  return (
    <div className="flex h-full min-h-0 flex-col text-foreground select-text nowheel">
      {/* Top Header & Search */}
      <div className="shrink-0 space-y-3 border-b border-border/70 p-3 bg-card/60 backdrop-blur-xs">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <div className="flex h-7 w-7 items-center justify-center rounded-ctl bg-brand/10 text-brand">
              <Sparkles className="h-3.5 w-3.5" />
            </div>
            <div>
              <h2 className="text-caption font-semibold uppercase tracking-wider text-foreground">
                Kho Mô hình AI (Models Detail)
              </h2>
              <p className="text-caption text-muted-foreground">
                Tra cứu chức năng, thông số & bảng giá {EXTENDED_MODELS_CATALOG.length}+ AI Models
              </p>
            </div>
          </div>
          <button
            type="button"
            title="Mở toàn màn hình"
            onClick={() => setShowFullModal(true)}
            className="flex h-7 w-7 items-center justify-center rounded-ctl text-muted-foreground hover:bg-accent hover:text-foreground cursor-pointer focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
          >
            <Maximize2 className="h-3.5 w-3.5" />
          </button>
        </div>

        {/* Search Input */}
        <div className="relative">
          <Search className="absolute left-2.5 top-2.5 h-3.5 w-3.5 text-muted-foreground" />
          <input
            type="text"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Tìm model (Kling, Veo 2, Flux, Wan, DeepSeek...)"
            className="w-full rounded-ctl border border-border/80 bg-background py-2 pl-8 pr-3 text-footnote text-foreground placeholder:text-muted-foreground/60 outline-none focus:border-brand"
          />
          {search && (
            <button
              type="button"
              onClick={() => setSearch("")}
              className="absolute right-2 top-2 text-caption text-muted-foreground hover:text-foreground"
            >
              ✕
            </button>
          )}
        </div>

        {/* Category Pill Tabs */}
        <div className="flex items-center gap-1 overflow-x-auto no-scrollbar scrollbar-none pb-1">
          {(
            [
              { id: "all", label: "Tất cả" },
              { id: "video", label: "Video AI" },
              { id: "image", label: "Tạo Ảnh" },
              { id: "voice", label: "Giọng nói" },
              { id: "text", label: "Kịch bản" },
            ] as const
          ).map((cat) => {
            const Icon = categoryIcons[cat.id];
            const active = selectedCategory === cat.id;
            return (
              <button
                key={cat.id}
                type="button"
                onClick={() => setSelectedCategory(cat.id)}
                className={cn(
                  "flex items-center gap-1 rounded-ctl px-2 py-1 text-caption font-medium transition-colors shrink-0 cursor-pointer focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring",
                  active
                    ? "bg-foreground text-background font-semibold"
                    : "bg-muted/40 text-muted-foreground hover:bg-muted hover:text-foreground"
                )}
              >
                <Icon className="h-3 w-3" />
                <span>{cat.label}</span>
              </button>
            );
          })}
        </div>
      </div>

      {/* Main Content: 2 Columns on wide screen, or stacked */}
      <div className="flex flex-1 min-h-0 overflow-hidden divide-x divide-border/60">
        {/* Left Column: List of Models */}
        <div
          className="w-[42%] shrink-0 overflow-y-auto p-2 space-y-2 scroll-quiet nowheel"
          onWheel={(e) => e.stopPropagation()}
        >
          <div className="px-2 py-1 text-caption font-medium uppercase tracking-wider text-muted-foreground">
            {filteredModels.length} mô hình phù hợp
          </div>
          {filteredModels.map((m) => {
            const active = m.key === activeModelKey;
            return (
              <button
                key={m.key}
                type="button"
                onClick={() => setActiveModelKey(m.key)}
                className={cn(
                  "flex w-full flex-col gap-1 rounded-card border p-2 text-left transition-all cursor-pointer focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring",
                  active
                    ? "border-foreground bg-accent/70 shadow-xs"
                    : "border-border/60 bg-card/60 hover:border-foreground/40 hover:bg-muted/30"
                )}
              >
                <div className="flex items-start justify-between gap-1">
                  <div className="min-w-0">
                    <div className="truncate text-footnote font-semibold text-foreground">
                      {m.name}
                    </div>
                    <div className="truncate text-caption text-muted-foreground">
                      {m.brand}
                    </div>
                  </div>
                  <span
                    className={cn(
                      "shrink-0 rounded-xs px-2 py-1 text-caption font-semibold",
                      m.tier === "paid"
                        ? "bg-amber-500/10 text-amber-600 dark:text-amber-400"
                        : "bg-emerald-500/10 text-emerald-600 dark:text-emerald-400"
                    )}
                  >
                    {m.badge}
                  </span>
                </div>
                <div className="flex items-center justify-between text-caption text-muted-foreground tnum pt-1 border-t border-border/40">
                  <span className="capitalize">{m.category}</span>
                  <span className="font-semibold text-foreground">{m.costStr}</span>
                </div>
              </button>
            );
          })}
          {filteredModels.length === 0 && (
            <div className="p-4 text-center text-footnote text-muted-foreground">
              Không tìm thấy mô hình nào khớp với &quot;{search}&quot;.
            </div>
          )}
        </div>

        {/* Right Column: Model Detail View */}
        <div
          className="flex-1 overflow-y-auto p-4 space-y-4 scroll-quiet bg-card/40 nowheel"
          onWheel={(e) => e.stopPropagation()}
        >
          {activeModel ? (
            <>
              {/* Header Info */}
              <div className="space-y-2 border-b border-border/60 pb-3">
                <div className="flex items-start justify-between gap-2">
                  <div>
                    <h3 className="text-callout font-bold text-foreground leading-tight">
                      {activeModel.name}
                    </h3>
                    <div className="flex items-center gap-2 text-footnote text-muted-foreground mt-1">
                      <span>Phát triển bởi <strong className="text-foreground">{activeModel.brand}</strong></span>
                      <span>·</span>
                      <span className="capitalize">{activeModel.category} AI</span>
                    </div>
                  </div>
                  <button
                    type="button"
                    onClick={() => handleCopyKey(activeModel.key)}
                    title="Sao chép Model Key"
                    className="flex h-7 items-center gap-1 rounded-ctl border border-border px-2 text-caption text-muted-foreground hover:bg-accent hover:text-foreground cursor-pointer shrink-0 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
                  >
                    {copiedKey ? (
                      <>
                        <Check className="h-3 w-3 text-emerald-500" />
                        <span className="text-emerald-500 font-medium">Đã chép</span>
                      </>
                    ) : (
                      <>
                        <Copy className="h-3 w-3" />
                        <span>{activeModel.key}</span>
                      </>
                    )}
                  </button>
                </div>

                <p className="text-footnote text-muted-foreground leading-relaxed pt-1">
                  {activeModel.description}
                </p>
              </div>

              {/* Pricing & Estimation */}
              <div className="rounded-card border border-amber-500/20 bg-amber-500/5 p-3 space-y-1">
                <div className="flex items-center justify-between">
                  <span className="flex items-center gap-1 text-caption font-semibold text-amber-700 dark:text-amber-300">
                    <DollarSign className="h-3 w-3" />
                    Biểu phí mô hình
                  </span>
                  <span className="font-mono text-footnote font-bold text-amber-700 dark:text-amber-300">
                    {activeModel.costStr}
                  </span>
                </div>
                <div className="text-caption text-amber-800/80 dark:text-amber-200/80">
                  Ước tính: {activeModel.costNote}
                </div>
              </div>

              {/* Technical Specifications */}
              <div className="space-y-2">
                <div className="text-caption font-semibold uppercase tracking-wider text-muted-foreground">
                  Thông số kỹ thuật (Specs)
                </div>
                <div className="grid grid-cols-2 gap-2 text-footnote">
                  <div className="rounded-card border border-border/70 bg-card p-2">
                    <div className="text-caption text-muted-foreground">Độ phân giải</div>
                    <div className="font-semibold text-foreground">
                      {activeModel.specs.resolutions.join(", ")}
                    </div>
                  </div>
                  <div className="rounded-card border border-border/70 bg-card p-2">
                    <div className="text-caption text-muted-foreground">Tỷ lệ khung hình</div>
                    <div className="font-semibold text-foreground">
                      {activeModel.specs.ratios.join(", ")}
                    </div>
                  </div>
                  {activeModel.specs.maxDuration && (
                    <div className="rounded-card border border-border/70 bg-card p-2">
                      <div className="text-caption text-muted-foreground">Thời lượng tối đa</div>
                      <div className="font-semibold text-foreground">
                        {activeModel.specs.maxDuration}
                      </div>
                    </div>
                  )}
                  {activeModel.specs.hasRefImage !== undefined && (
                    <div className="rounded-card border border-border/70 bg-card p-2">
                      <div className="text-caption text-muted-foreground">Ảnh tham chiếu DNA</div>
                      <div className="font-semibold text-foreground">
                        {activeModel.specs.hasRefImage ? "✓ Hỗ trợ chuẩn" : "✕ Không"}
                      </div>
                    </div>
                  )}
                </div>
              </div>

              {/* Strengths */}
              <div className="space-y-2">
                <div className="text-caption font-semibold uppercase tracking-wider text-muted-foreground">
                  Điểm mạnh nổi bật
                </div>
                <ul className="space-y-1 text-footnote">
                  {activeModel.strengths.map((str, idx) => (
                    <li key={idx} className="flex items-start gap-2 text-foreground/90">
                      <Check className="h-3.5 w-3.5 shrink-0 text-emerald-500 mt-1" />
                      <span>{str}</span>
                    </li>
                  ))}
                </ul>
              </div>

              {/* Recommended Use */}
              <div className="space-y-1 rounded-card border border-border/70 bg-muted/20 p-3">
                <div className="text-caption font-semibold text-foreground">
                  Ứng dụng khuyên dùng
                </div>
                <p className="text-footnote text-muted-foreground leading-snug">
                  {activeModel.recommendedUse}
                </p>
              </div>

              {/* Actions */}
              {selectedNode && (
                <div className="pt-2">
                  <button
                    type="button"
                    onClick={handleApplyToSelectedNode}
                    className="flex w-full items-center justify-center gap-2 rounded-ctl bg-foreground py-2 text-footnote font-semibold text-background transition-opacity hover:opacity-90 active:scale-98 cursor-pointer focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
                  >
                    <Sliders className="h-3.5 w-3.5" />
                    <span>
                      Áp dụng cho Node #{selectedNode.id} ({String(selectedNode.data?.customName || selectedNode.data?.nodeType || "")})
                    </span>
                  </button>
                </div>
              )}
            </>
          ) : (
            <div className="flex h-full items-center justify-center text-footnote text-muted-foreground">
              Chọn một mô hình ở cột bên trái để xem chi tiết.
            </div>
          )}
        </div>
      </div>

      {/* Full Screen Modal if user clicks Expand */}
      <AIModelsLibraryModal
        isOpen={showFullModal}
        onClose={() => setShowFullModal(false)}
        onSelectModel={(key) => {
          setActiveModelKey(key);
          setShowFullModal(false);
        }}
      />
    </div>
  );
}
