"use client";

import React from "react";
import { Film, Image, Images, Trash2, Upload } from "@/components/ui/icons";
import type { CanvasImageItem, ChatAttachment, ComposerActionMode } from "../types";

interface Props {
  isVi: boolean;
  canvasImages: CanvasImageItem[];
  onTriggerUpload: () => void;
  onAddAttachment?: (att: ChatAttachment) => void;
  onActionModeChange?: (mode: ComposerActionMode) => void;
  onClearChat: () => void;
  onClose: () => void;
  onFocusTextarea: () => void;
}

export function PlusActionPopover({
  isVi,
  canvasImages,
  onTriggerUpload,
  onAddAttachment,
  onActionModeChange,
  onClearChat,
  onClose,
  onFocusTextarea,
}: Props) {
  return (
    <div
      data-chat-popover=""
      className="scroll-quiet absolute bottom-full mb-2 left-0 w-72 max-w-full max-h-[min(400px,calc(100dvh-250px))] overflow-y-auto rounded-panel border border-input bg-popover p-2 shadow-2xl ring-1 ring-foreground/15 animate-in fade-in-0 zoom-in-95 z-50 space-y-1 text-footnote"
    >
      <p className="px-3 py-1 text-caption font-semibold uppercase tracking-wider text-muted-foreground">
        {isVi ? "Đưa ảnh vào tạo Ảnh / Video" : "Add Reference Image"}
      </p>
      <button
        type="button"
        onClick={onTriggerUpload}
        className="flex w-full items-center gap-2 rounded-ctl px-3 py-2 hover:bg-accent transition-colors text-left"
      >
        <Upload className="h-4 w-4 text-brand shrink-0" />
        <div>
          <p className="font-medium text-foreground">{isVi ? "Tải ảnh lên từ máy (Upload local)" : "Upload local image"}</p>
          <p className="text-caption text-muted-foreground">
            {isVi ? "Đính kèm 1 hoặc nhiều ảnh để tạo Video / Ảnh" : "Attach 1 or multiple images for Video/Image"}
          </p>
        </div>
      </button>

      {canvasImages.length > 0 && (
        <div className="px-2 py-2">
          <p className="mb-1 text-caption font-semibold uppercase tracking-wider text-muted-foreground">
            {isVi ? "Chọn nhanh từ Canvas (Select from Canvas)" : "Select from Canvas"}
          </p>
          <div className="scroll-quiet flex gap-2 overflow-x-auto pb-1">
            {canvasImages.map((img) => (
              <button
                key={img.id}
                type="button"
                title={img.name}
                onClick={() => {
                  onAddAttachment?.({
                    id: `att_cv_${img.id}`,
                    name: img.name,
                    url: img.url,
                    source: "canvas",
                    nodeId: img.nodeId,
                  });
                  onClose();
                }}
                className="group relative h-12 w-12 shrink-0 overflow-hidden rounded-ctl border border-border hover:border-brand transition-colors"
              >
                {/* eslint-disable-next-line @next/next/no-img-element */}
                <img src={img.url} alt={img.name} loading="lazy" className="h-full w-full object-cover" />
                <span className="absolute inset-x-0 bottom-0 truncate bg-black/65 px-1 py-1 text-caption text-white">
                  {img.name}
                </span>
              </button>
            ))}
          </div>
        </div>
      )}

      <div className="my-1 border-t border-border/60" />
      <p className="px-3 py-1 text-caption font-semibold uppercase tracking-wider text-muted-foreground">
        {isVi ? "Chế độ khởi tạo trực tiếp" : "Direct Generation Mode"}
      </p>
      <button
        type="button"
        onClick={() => {
          onActionModeChange?.("video");
          onClose();
          onFocusTextarea();
        }}
        className="flex w-full items-center gap-2 rounded-ctl px-3 py-2 hover:bg-accent transition-colors text-left"
      >
        <Film className="h-4 w-4 text-brand shrink-0" />
        <div>
          <p className="font-medium text-foreground">{isVi ? "Ảnh → Tạo Video (Image to Video)" : "Image to Video"}</p>
          <p className="text-caption text-muted-foreground">
            {isVi ? "Sinh video từ ảnh tham chiếu & xuất Node Video" : "Generate video from reference image"}
          </p>
        </div>
      </button>
      <button
        type="button"
        onClick={() => {
          onActionModeChange?.("image");
          onClose();
          onFocusTextarea();
        }}
        className="flex w-full items-center gap-2 rounded-ctl px-3 py-2 hover:bg-accent transition-colors text-left"
      >
        <Image className="h-4 w-4 text-brand shrink-0" />
        <div>
          <p className="font-medium text-foreground">{isVi ? "Tạo Node Ảnh đơn (AI Image)" : "Generate Single Image Node"}</p>
          <p className="text-caption text-muted-foreground">
            {isVi ? "Tạo hoặc sửa ảnh từ prompt & ảnh gốc" : "Create or edit image on Canvas"}
          </p>
        </div>
      </button>
      <button
        type="button"
        onClick={() => {
          onActionModeChange?.("image_list");
          onClose();
          onFocusTextarea();
        }}
        className="flex w-full items-center gap-2 rounded-ctl px-3 py-2 hover:bg-accent transition-colors text-left"
      >
        <Images className="h-4 w-4 text-brand shrink-0" />
        <div>
          <p className="font-medium text-foreground">
            {isVi ? "Tạo Node List Ảnh / Multi-Angle Grid" : "Generate Image List / Storyboard Grid"}
          </p>
          <p className="text-caption text-muted-foreground">
            {isVi ? "Sinh bộ nhiều góc máy / storyboard vào 1 node" : "Generate multi-angle image set node"}
          </p>
        </div>
      </button>
      <div className="my-1 border-t border-border/60" />
      <button
        type="button"
        onClick={() => {
          onClearChat();
          onClose();
        }}
        className="flex w-full items-center gap-2 rounded-ctl px-3 py-2 text-semantic-danger hover:bg-semantic-danger/10 transition-colors text-left"
      >
        <Trash2 className="h-4 w-4" />
        <span className="font-medium">{isVi ? "Xóa cuộc trò chuyện này" : "Clear conversation"}</span>
      </button>
    </div>
  );
}
