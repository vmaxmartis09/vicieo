"use client";

import React, { type Dispatch, type SetStateAction } from "react";
import { Layers, Wand2, Loader2 } from "@/components/ui/icons";
import type { TemplateMeta } from "@/lib/api/video";
import { SlotField } from "@/components/templates/SlotField";
import { VoiceSelectorWithClone } from "@/components/voice/VoiceSelectorWithClone";
import { ProductAttachCard } from "../components/ProductAttachCard";
import { VideoCraftSection } from "../components/VideoCraftSection";
import { parseBatchTopics, type HookVariantItem, type ProductFormInfo } from "../types";

export function Step2ContentCraft({
  selectedTemplate,
  activeProjectId,
  isBatchMode,
  setIsBatchMode,
  batchTopicsText,
  setBatchTopicsText,
  topic,
  setTopic,
  isGeneratingScript,
  onGenerateScript,
  scriptText,
  setScriptText,
  slotValues,
  setSlotValues,
  productInfo,
  setProductInfo,
  selectedCharacterId,
  setSelectedCharacterId,
  selectedVoiceId,
  setSelectedVoiceId,
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
  selectedTemplate: TemplateMeta;
  activeProjectId: string;
  isBatchMode: boolean;
  setIsBatchMode: (v: boolean) => void;
  batchTopicsText: string;
  setBatchTopicsText: (v: string) => void;
  topic: string;
  setTopic: (v: string) => void;
  isGeneratingScript: boolean;
  onGenerateScript: () => void;
  scriptText: string;
  setScriptText: (v: string) => void;
  slotValues: Record<string, any>;
  setSlotValues: Dispatch<SetStateAction<Record<string, any>>>;
  productInfo: ProductFormInfo;
  setProductInfo: Dispatch<SetStateAction<ProductFormInfo>>;
  selectedCharacterId: string;
  setSelectedCharacterId: (v: string) => void;
  selectedVoiceId: string;
  setSelectedVoiceId: (v: string) => void;
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
    <div className="space-y-4">
      <div className="flex items-center justify-between rounded-card border border-border bg-accent/30 p-3">
        <div className="space-y-1">
          <span className="text-footnote font-semibold text-foreground flex items-center gap-2">
            <Layers className="h-4 w-4 text-primary" />
            Chế độ Hàng loạt (Batch Video)
          </span>
          <p className="text-caption text-muted-foreground">
            Tạo N video cùng lúc với các chủ đề khác nhau dùng chung mẫu & giọng
          </p>
        </div>
        <label className="relative inline-flex items-center cursor-pointer">
          <input
            type="checkbox"
            checked={isBatchMode}
            onChange={(e) => setIsBatchMode(e.target.checked)}
            className="sr-only peer"
          />
          <div className="w-10 h-5 bg-border peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-background after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-background after:border-border after:border after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-primary"></div>
        </label>
      </div>

      {isBatchMode ? (
        <div className="space-y-2">
          <label className="block text-footnote font-semibold text-foreground">
            Danh sách chủ đề (Mỗi dòng 1 video, hoặc ngăn cách bằng "---"):
          </label>
          <textarea
            rows={6}
            value={batchTopicsText}
            onChange={(e) => setBatchTopicsText(e.target.value)}
            placeholder={`Chủ đề 1: Review kem chống nắng Sóc Nhỏ đi biển\nChủ đề 2: Thử thách tìm hạt dẻ kỳ diệu cùng Koala\nChủ đề 3: Khám phá bí ẩn rừng xanh`}
            className="w-full rounded-ctl border border-border bg-background p-3 text-footnote text-foreground outline-none focus:border-primary font-mono leading-relaxed"
          />
          <p className="text-caption text-muted-foreground">
            Số lượng video dự kiến:{" "}
            <span className="font-semibold text-primary">
              {parseBatchTopics(batchTopicsText).length}
            </span>
          </p>
        </div>
      ) : (
        <div className="space-y-4">
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <label className="text-footnote font-semibold text-foreground">
                Chủ đề / Ý tưởng video:
              </label>
              <button
                type="button"
                onClick={onGenerateScript}
                disabled={isGeneratingScript || !topic.trim()}
                className="flex items-center gap-2 rounded-ctl bg-primary px-3 py-1 text-footnote font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-40 transition-colors shadow-2xs"
              >
                {isGeneratingScript ? (
                  <>
                    <Loader2 className="h-3.5 w-3.5 animate-spin" />
                    <span>Đang viết kịch bản...</span>
                  </>
                ) : (
                  <>
                    <Wand2 className="h-3.5 w-3.5" />
                    <span>Tự viết kịch bản</span>
                  </>
                )}
              </button>
            </div>
            <input
              type="text"
              value={topic}
              onChange={(e) => setTopic(e.target.value)}
              placeholder="Nhập ý tưởng (vd: Sóc Nhỏ review kem chống nắng, Câu chuyện hạt dẻ thần kỳ...)"
              className="w-full rounded-ctl border border-border bg-background px-3 py-2 text-footnote text-foreground outline-none focus:border-primary"
            />
          </div>

          {scriptText && (
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <label className="text-footnote font-semibold text-foreground">
                  Kịch bản chi tiết (sửa trực tiếp nếu muốn):
                </label>
                <span className="text-caption text-muted-foreground">
                  Đã tự điền vào các slot bên dưới
                </span>
              </div>
              <textarea
                rows={4}
                value={scriptText}
                onChange={(e) => setScriptText(e.target.value)}
                className="w-full rounded-ctl border border-border bg-background p-3 text-footnote text-foreground outline-none focus:border-primary font-mono leading-relaxed"
              />
            </div>
          )}

          <VideoCraftSection
            enableHookABC={enableHookABC}
            setEnableHookABC={setEnableHookABC}
            hookVariants={hookVariants}
            selectedHookVariantId={selectedHookVariantId}
            onSelectHookVariant={onSelectHookVariant}
            showAdvancedCraft={showAdvancedCraft}
            setShowAdvancedCraft={setShowAdvancedCraft}
            enable3LayerDepth={enable3LayerDepth}
            setEnable3LayerDepth={setEnable3LayerDepth}
            enableAutoTransitions={enableAutoTransitions}
            setEnableAutoTransitions={setEnableAutoTransitions}
          />

          <div className="space-y-3 pt-2 border-t border-border">
            <h4 className="text-footnote font-semibold text-foreground">
              Các trường nội dung của mẫu ({selectedTemplate.slots.length} trường):
            </h4>
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
              {selectedTemplate.slots.map((slot) => (
                <SlotField
                  key={slot.key}
                  slot={slot}
                  value={slotValues[slot.key]}
                  onChange={(val) =>
                    setSlotValues((prev) => ({ ...prev, [slot.key]: val }))
                  }
                  projectId={activeProjectId}
                />
              ))}
            </div>
          </div>

          {(selectedTemplate.category === "sales" ||
            selectedTemplate.category === "review" ||
            selectedTemplate.category === "affiliate") && (
            <ProductAttachCard productInfo={productInfo} setProductInfo={setProductInfo} />
          )}
        </div>
      )}

      <div className="space-y-3 pt-2 border-t border-border">
        <span className="text-footnote font-semibold text-foreground">
          Nhân vật & Giọng đọc:
        </span>
        <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <div className="space-y-1">
            <label className="text-caption text-muted-foreground">Nhân vật:</label>
            <select
              value={selectedCharacterId}
              onChange={(e) => setSelectedCharacterId(e.target.value)}
              className="w-full rounded-ctl border border-border bg-background px-3 py-2 text-footnote text-foreground"
            >
              <option value="soc_nho">Sóc Nhỏ (Bi) — 100cm, Áo vàng</option>
              <option value="koala">Bạn Koala — 150cm, Bác sĩ</option>
            </select>
          </div>
          <div className="space-y-1">
            <label className="text-caption text-muted-foreground">Giọng lồng tiếng:</label>
            <VoiceSelectorWithClone
              value={selectedVoiceId}
              onChange={setSelectedVoiceId}
              projectId={activeProjectId}
            />
          </div>
        </div>
      </div>
    </div>
  );
}
