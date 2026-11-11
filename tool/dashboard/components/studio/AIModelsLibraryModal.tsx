"use client";

import React, { useState, useEffect, useMemo } from "react";
import {
  Sparkles,
  Search,
  X,
  Check,
  Film,
  Images,
  Volume2,
  FileText,
  Clock,
  Layers,
  Tag,
  DollarSign,
  ExternalLink,
  ChevronRight,
} from "@/components/ui/icons";
import { fetchModels, type ModelItem } from "@/lib/api/models";
import { cn } from "@/lib/utils";

import { EXTENDED_MODELS_CATALOG, type ExtendedModelItem } from "@/lib/data/extendedModelsCatalog";
export { EXTENDED_MODELS_CATALOG, type ExtendedModelItem };

export interface AIModelsLibraryModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSelectModel?: (modelKey: string) => void;
}

export function AIModelsLibraryModal({
  isOpen,
  onClose,
  onSelectModel,
}: AIModelsLibraryModalProps) {
  const [selectedCategory, setSelectedCategory] = useState<"all" | "video" | "image" | "voice" | "text">("all");
  const [search, setSearch] = useState("");
  const [activeModelKey, setActiveModelKey] = useState<string>("kling-v3-std");

  // Filtered models
  const filteredModels = useMemo(() => {
    return EXTENDED_MODELS_CATALOG.filter((m) => {
      const matchCategory = selectedCategory === "all" || m.category === selectedCategory;
      const q = search.trim().toLowerCase();
      const matchSearch =
        !q ||
        m.name.toLowerCase().includes(q) ||
        m.brand.toLowerCase().includes(q) ||
        m.description.toLowerCase().includes(q) ||
        m.key.toLowerCase().includes(q);
      return matchCategory && matchSearch;
    });
  }, [selectedCategory, search]);

  const activeModel = useMemo(() => {
    return EXTENDED_MODELS_CATALOG.find((m) => m.key === activeModelKey) || EXTENDED_MODELS_CATALOG[0];
  }, [activeModelKey]);

  // Close on Escape
  useEffect(() => {
    if (!isOpen) return;
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        onClose();
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [isOpen, onClose]);

  if (!isOpen) return null;

  return (
    <div
      className="fixed inset-0 z-[9999] flex items-center justify-center bg-black/60 backdrop-blur-xs p-4 animate-in fade-in-0 duration-200 nowheel"
      onClick={onClose}
    >
      <div
        onClick={(e) => e.stopPropagation()}
        className="flex h-[90vh] max-h-[780px] w-full max-w-5xl flex-col rounded-sheet border border-border bg-card text-foreground shadow-2xl overflow-hidden animate-in zoom-in-95 duration-150 nowheel"
      >
        {/* Header */}
        <div className="flex items-center justify-between border-b border-border/70 px-5 py-3 bg-muted/20">
          <div className="flex items-center gap-2">
            <div className="flex h-8 w-8 items-center justify-center rounded-ctl bg-brand/10 text-brand">
              <Sparkles className="h-4 w-4" />
            </div>
            <div>
              <h2 className="text-callout font-semibold tracking-tight text-foreground">
                Kho Mô hình AI (AI Models Library)
              </h2>
              <p className="text-footnote text-muted-foreground">
                Tra cứu chi tiết năng lực, thông số kỹ thuật và biểu phí các mô hình AI tích hợp trong Studio
              </p>
            </div>
          </div>

          <button
            type="button"
            onClick={onClose}
            className="flex h-8 w-8 items-center justify-center rounded-ctl text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
          >
            <X className="h-4 w-4" />
          </button>
        </div>

        {/* Toolbar: Search & Categories */}
        <div className="flex flex-wrap items-center justify-between gap-3 border-b border-border/60 px-5 py-2 bg-card">
          {/* Categories */}
          <div className="flex items-center gap-2 overflow-x-auto no-scrollbar scrollbar-none">
            {[
              { id: "all", label: "Tất cả mô hình", icon: Layers },
              { id: "video", label: "Tạo Video", icon: Film },
              { id: "image", label: "Tạo Ảnh", icon: Images },
              { id: "voice", label: "Giọng đọc AI", icon: Volume2 },
              { id: "text", label: "Kịch bản & LLM", icon: FileText },
            ].map((cat) => {
              const IconComp = cat.icon;
              const active = selectedCategory === cat.id;
              return (
                <button
                  key={cat.id}
                  type="button"
                  onClick={() => setSelectedCategory(cat.id as any)}
                  className={cn(
                    "flex items-center gap-2 rounded-ctl px-3 py-2 text-footnote font-medium transition-colors",
                    active
                      ? "bg-foreground text-background font-semibold shadow-xs"
                      : "text-muted-foreground hover:bg-accent hover:text-foreground"
                  )}
                >
                  <IconComp className="h-3.5 w-3.5" />
                  <span>{cat.label}</span>
                </button>
              );
            })}
          </div>

          {/* Search */}
          <div className="relative w-64">
            <Search className="absolute left-2.5 top-2.5 h-3.5 w-3.5 text-muted-foreground" />
            <input
              type="text"
              placeholder="Tìm theo tên model, lab, tính năng..."
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              className="w-full rounded-ctl border border-border bg-background py-2 pl-8 pr-3 text-footnote text-foreground placeholder:text-muted-foreground/60 outline-none focus:border-foreground"
            />
          </div>
        </div>

        {/* Content: 2-column layout (List + Detail) */}
        <div className="grid flex-1 min-h-0 grid-cols-12 overflow-hidden">
          {/* Left: Model List (5 cols) */}
          <div
            className="col-span-5 border-r border-border/60 overflow-y-auto p-3 space-y-2 scroll-quiet nowheel"
            onWheel={(e) => e.stopPropagation()}
          >
            {filteredModels.length === 0 ? (
              <div className="p-8 text-center text-footnote text-muted-foreground">
                Không tìm thấy mô hình nào phù hợp với &ldquo;{search}&rdquo;
              </div>
            ) : (
              filteredModels.map((m) => {
                const isSelected = m.key === activeModelKey;
                return (
                  <button
                    key={m.key}
                    type="button"
                    onClick={() => setActiveModelKey(m.key)}
                    className={cn(
                      "flex w-full flex-col gap-2 rounded-card border p-3 text-left transition-all",
                      isSelected
                        ? "border-foreground bg-accent shadow-xs"
                        : "border-border/70 hover:border-foreground/30 hover:bg-accent/40"
                    )}
                  >
                    <div className="flex w-full items-center justify-between">
                      <div className="min-w-0 pr-2">
                        <span className="font-semibold text-footnote text-foreground block truncate">
                          {m.name}
                        </span>
                        <span className="text-caption text-muted-foreground block truncate">
                          {m.brand}
                        </span>
                      </div>

                      <div className="text-right shrink-0">
                        <span className="font-mono text-footnote font-semibold text-brand block">
                          {m.costStr}
                        </span>
                        <span className="inline-block rounded-xs bg-muted/80 px-2 py-1 text-caption font-medium text-muted-foreground">
                          {m.badge}
                        </span>
                      </div>
                    </div>

                    <p className="line-clamp-2 text-caption text-muted-foreground leading-snug">
                      {m.description}
                    </p>

                    <div className="flex flex-wrap items-center gap-1 pt-1">
                      {m.specs.resolutions.map((r) => (
                        <span key={r} className="rounded-xs bg-muted px-2 py-1 text-caption font-mono text-muted-foreground">
                          {r}
                        </span>
                      ))}
                      {m.specs.maxDuration && (
                        <span className="rounded-xs bg-muted px-2 py-1 text-caption font-mono text-muted-foreground">
                          Max {m.specs.maxDuration}
                        </span>
                      )}
                    </div>
                  </button>
                );
              })
            )}
          </div>

          {/* Right: Model Detail (7 cols) */}
          <div
            className="col-span-7 overflow-y-auto p-5 space-y-4 scroll-quiet bg-muted/5 nowheel"
            onWheel={(e) => e.stopPropagation()}
          >
            {activeModel && (
              <>
                {/* Detail Header */}
                <div className="flex items-start justify-between border-b border-border/60 pb-3">
                  <div>
                    <div className="flex items-center gap-2">
                      <h3 className="text-title font-bold text-foreground">{activeModel.name}</h3>
                      <span className="rounded-full bg-brand/10 border border-brand/20 px-2 py-1 font-mono text-caption font-semibold text-brand">
                        {activeModel.costStr}
                      </span>
                    </div>
                    <p className="text-footnote text-muted-foreground mt-1">
                      Phát triển bởi <span className="font-medium text-foreground">{activeModel.brand}</span> · Nhà cung cấp: {activeModel.provider}
                    </p>
                  </div>

                  <div className="text-right">
                    <span className="text-footnote font-mono font-medium text-muted-foreground block">
                      {activeModel.costNote}
                    </span>
                    <code className="text-caption font-mono text-brand block mt-1">
                      {`{model.${activeModel.key}}`}
                    </code>
                  </div>
                </div>

                {/* Description */}
                <div className="space-y-1">
                  <span className="text-caption font-semibold uppercase tracking-wider text-muted-foreground">
                    Mô tả tổng quan
                  </span>
                  <p className="text-footnote leading-relaxed text-foreground">
                    {activeModel.description}
                  </p>
                </div>

                {/* Capabilities & Specs */}
                <div className="space-y-2">
                  <span className="text-caption font-semibold uppercase tracking-wider text-muted-foreground">
                    Thông số kỹ thuật & Năng lực
                  </span>
                  <div className="grid grid-cols-2 gap-2 text-footnote">
                    <div className="rounded-card border border-border bg-card p-3">
                      <span className="text-caption text-muted-foreground block">Độ phân giải hỗ trợ</span>
                      <span className="font-medium text-foreground">
                        {activeModel.specs.resolutions.join(", ")}
                      </span>
                    </div>

                    <div className="rounded-card border border-border bg-card p-3">
                      <span className="text-caption text-muted-foreground block">Tỷ lệ khung hình</span>
                      <span className="font-medium text-foreground">
                        {activeModel.specs.ratios.length > 0 ? activeModel.specs.ratios.join(", ") : "Theo ngữ cảnh"}
                      </span>
                    </div>

                    {activeModel.specs.maxDuration && (
                      <div className="rounded-card border border-border bg-card p-3">
                        <span className="text-caption text-muted-foreground block">Thời lượng video tối đa</span>
                        <span className="font-medium text-foreground">{activeModel.specs.maxDuration}</span>
                      </div>
                    )}

                    <div className="rounded-card border border-border bg-card p-3">
                      <span className="text-caption text-muted-foreground block">Tham chiếu hình ảnh (DNA)</span>
                      <span className="font-medium text-foreground">
                        {activeModel.specs.hasRefImage ? "Hỗ trợ (Giữ dáng nhân vật)" : "Tạo từ prompt thuần"}
                      </span>
                    </div>
                  </div>
                </div>

                {/* Strengths */}
                <div className="space-y-2">
                  <span className="text-caption font-semibold uppercase tracking-wider text-muted-foreground">
                    Điểm mạnh vượt trội
                  </span>
                  <div className="space-y-1">
                    {activeModel.strengths.map((s, i) => (
                      <div key={i} className="flex items-center gap-2 text-footnote text-foreground">
                        <Check className="h-3.5 w-3.5 text-brand shrink-0" />
                        <span>{s}</span>
                      </div>
                    ))}
                  </div>
                </div>

                {/* Recommended Use */}
                <div className="rounded-card border border-brand/20 bg-brand/5 p-3 space-y-1">
                  <span className="text-caption font-semibold uppercase tracking-wider text-brand">
                    Khuyến nghị ứng dụng tốt nhất
                  </span>
                  <p className="text-footnote text-foreground leading-snug">
                    {activeModel.recommendedUse}
                  </p>
                </div>

                {/* Action button */}
                <div className="pt-2">
                  <button
                    type="button"
                    onClick={() => {
                      if (onSelectModel) {
                        onSelectModel(activeModel.key);
                      }
                      onClose();
                    }}
                    className="flex h-9 w-full items-center justify-center gap-2 rounded-ctl bg-foreground px-4 text-footnote font-semibold text-background transition-transform active:scale-[0.98] hover:opacity-90"
                  >
                    <span>Sử dụng {activeModel.name} trong Studio</span>
                    <code className="rounded-xs bg-background/20 px-2 py-1 text-caption font-mono text-background">
                      {`{model.${activeModel.key}}`}
                    </code>
                  </button>
                </div>
              </>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
