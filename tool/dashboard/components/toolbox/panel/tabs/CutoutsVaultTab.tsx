"use client";

import React from "react";
import { Image as ImageIcon, FolderOpen } from "@/components/ui/icons";
import { cn } from "@/lib/utils";
import { previewUrl } from "../api";
import { CARD, checker } from "../constants";
import { ToolboxSaveBar } from "../shared/ToolboxSaveBar";
import type { Targets } from "../types";

export function CutoutsVaultTab({
  cutouts,
  targets,
  saveFor,
  setSaveFor,
  onGoToBgRemove,
}: {
  cutouts: any[];
  targets: Targets | null;
  saveFor: string | null;
  setSaveFor: (id: string | null) => void;
  onGoToBgRemove: () => void;
}) {
  if (cutouts.length === 0) {
    return (
      <div className="col-span-2 space-y-1 rounded-xl border border-dashed border-border p-4">
        <ImageIcon className="h-5 w-5 text-muted-foreground" />
        <p className="text-footnote font-medium">Chưa có ảnh tách nền</p>
        <button type="button" onClick={onGoToBgRemove} className="text-caption text-brand hover:underline">
          Tách nền một ảnh ngay
        </button>
      </div>
    );
  }
  return (
    <>
      {cutouts.map((item) => (
        <div key={item.id} className={cn(CARD, "virtual-card space-y-2 border-border")}>
          <div className="flex items-center justify-between text-caption">
            <span className="font-mono font-medium">#{item.id}</span>
            <span className="text-muted-foreground">
              {item.metadata?.width}×{item.metadata?.height}
            </span>
          </div>
          <div className={checker}>
            <img decoding="async" loading="lazy" src={previewUrl(item.files?.cutout || "")} alt="Ảnh tách nền" className="max-h-44 object-contain" onError={(e) => ((e.target as HTMLElement).style.display = "none")} />
          </div>
          <p className="truncate text-caption text-muted-foreground" title={item.source}>
            Nguồn: {item.source}
          </p>
          {saveFor === `c${item.id}` ? (
            <ToolboxSaveBar
              targets={targets}
              files={[
                { label: "Ảnh tách nền (PNG)", path: item.files?.cutout },
                ...(item.files?.mask ? [{ label: "Mặt nạ (mask)", path: item.files.mask }] : []),
              ]}
            />
          ) : (
            <button type="button" onClick={() => setSaveFor(`c${item.id}`)} className="flex h-7 items-center gap-1 rounded-md border border-border px-2 text-caption font-medium transition-all hover:bg-accent active:scale-95">
              <FolderOpen className="h-3.5 w-3.5" />
              Tải về / Lưu vào…
            </button>
          )}
        </div>
      ))}
    </>
  );
}
