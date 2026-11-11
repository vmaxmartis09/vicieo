"use client";

import React from "react";
import { Sparkles, ChevronDown } from "@/components/ui/icons";
import { cn } from "@/lib/utils";
import type { HookVariantItem } from "../types";

export const VideoCraftSection = React.memo(function VideoCraftSection({
  enableHookABC,
  setEnableHookABC,
  hookVariants,
  selectedHookVariantId,
  onSelectHookVariant,
  showAdvancedCraft,
  setShowAdvancedCraft,
  enable3LayerDepth,
  setEnable3LayerDepth,
  enableAutoTransitions,
  setEnableAutoTransitions,
}: {
  enableHookABC: boolean;
  setEnableHookABC: (v: boolean) => void;
  hookVariants: HookVariantItem[];
  selectedHookVariantId: string;
  onSelectHookVariant: (v: HookVariantItem) => void;
  showAdvancedCraft: boolean;
  setShowAdvancedCraft: (v: boolean) => void;
  enable3LayerDepth: boolean;
  setEnable3LayerDepth: (v: boolean) => void;
  enableAutoTransitions: boolean;
  setEnableAutoTransitions: (v: boolean) => void;
}) {
  return (
    <div className="rounded-card border border-border bg-card/60 p-3 space-y-3">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Sparkles className="h-4 w-4 text-primary" />
          <span className="text-footnote font-semibold text-foreground">Kỹ thuật giữ chân Video Craft (Session K)</span>
        </div>
        <button
          type="button"
          onClick={() => setShowAdvancedCraft(!showAdvancedCraft)}
          className="text-caption text-muted-foreground hover:text-foreground flex items-center gap-1"
        >
          <span>{showAdvancedCraft ? "Thu gọn" : "Nâng cao"}</span>
          <ChevronDown className={cn("h-3 w-3 transition-transform", showAdvancedCraft && "rotate-180")} />
        </button>
      </div>

      <div className="flex items-center justify-between pt-1 border-t border-border/50">
        <div>
          <span className="text-footnote font-medium text-foreground">Hook Lab (Biến thể A/B/C & Heuristic Score)</span>
          <p className="text-caption text-muted-foreground">Tự động chọn hook ≤3s điểm cao nhất, tránh chào hỏi</p>
        </div>
        <label className="relative inline-flex items-center cursor-pointer">
          <input
            type="checkbox"
            checked={enableHookABC}
            onChange={(e) => setEnableHookABC(e.target.checked)}
            className="sr-only peer"
          />
          <div className="w-8 h-4 bg-border peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-background after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-background after:border-border after:border after:rounded-full after:h-3 after:w-3 after:transition-all peer-checked:bg-primary"></div>
        </label>
      </div>

      {enableHookABC && hookVariants.length > 0 && (
        <div className="space-y-2 pt-1">
          <span className="text-caption font-semibold text-muted-foreground">Chọn 1 trong 3 biến thể Hook:</span>
          <div className="grid grid-cols-1 sm:grid-cols-3 gap-2">
            {hookVariants.map((v) => (
              <div
                key={v.variant_id}
                onClick={() => onSelectHookVariant(v)}
                className={cn(
                  "p-2 rounded-ctl border text-left cursor-pointer transition-all",
                  selectedHookVariantId === v.variant_id
                    ? "border-primary bg-primary/10 ring-1 ring-primary/40"
                    : "border-border hover:bg-accent/40"
                )}
              >
                <div className="flex items-center justify-between mb-1">
                  <span className="text-caption font-bold text-primary">Phương án {v.variant_id}</span>
                  <span className="text-caption bg-emerald-500/20 text-emerald-600 dark:text-emerald-400 font-bold px-2 py-1 rounded-xs">
                    {v.score?.total_score || 0}đ
                  </span>
                </div>
                <p className="text-caption text-foreground line-clamp-2 italic">"{v.text}"</p>
              </div>
            ))}
          </div>
        </div>
      )}

      {showAdvancedCraft && (
        <div className="space-y-3 pt-2 border-t border-border/50">
          <div className="flex items-center justify-between">
            <div>
              <span className="text-footnote font-medium text-foreground">Chiều sâu 3 lớp (3-Layer Depth)</span>
              <p className="text-caption text-muted-foreground">Tiền cảnh mờ + nhân vật DNA nét + hậu cảnh khí quyển</p>
            </div>
            <label className="relative inline-flex items-center cursor-pointer">
              <input
                type="checkbox"
                checked={enable3LayerDepth}
                onChange={(e) => setEnable3LayerDepth(e.target.checked)}
                className="sr-only peer"
              />
              <div className="w-8 h-4 bg-border peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-background after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-background after:border-border after:border after:rounded-full after:h-3 after:w-3 after:transition-all peer-checked:bg-primary"></div>
            </label>
          </div>

          <div className="flex items-center justify-between">
            <div>
              <span className="text-footnote font-medium text-foreground">Chuyển cảnh tự động (Transition Foresight)</span>
              <p className="text-caption text-muted-foreground">Tiên liệu vector chuyển động 0.5s cuối cảnh trước</p>
            </div>
            <label className="relative inline-flex items-center cursor-pointer">
              <input
                type="checkbox"
                checked={enableAutoTransitions}
                onChange={(e) => setEnableAutoTransitions(e.target.checked)}
                className="sr-only peer"
              />
              <div className="w-8 h-4 bg-border peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-background after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-background after:border-border after:border after:rounded-full after:h-3 after:w-3 after:transition-all peer-checked:bg-primary"></div>
            </label>
          </div>
        </div>
      )}
    </div>
  );
});
