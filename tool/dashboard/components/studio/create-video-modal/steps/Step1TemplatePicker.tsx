"use client";

import React, { useRef, useState } from "react";
import { Check } from "@/components/ui/icons";
import type { TemplateMeta } from "@/lib/api/video";
import { CORE_URL } from "@/lib/repositories/core";
import { cn } from "@/lib/utils";
import { CATEGORY_CHIPS } from "../constants";

export const Step1TemplatePicker = React.memo(function Step1TemplatePicker({
  selectedCategory,
  onSelectCategory,
  filteredTemplates,
  selectedTemplate,
  onSelectTemplate,
}: {
  selectedCategory: string;
  onSelectCategory: (id: string) => void;
  filteredTemplates: TemplateMeta[];
  selectedTemplate: TemplateMeta | null;
  onSelectTemplate: (tpl: TemplateMeta) => void;
}) {
  const [hoveredTemplateId, setHoveredTemplateId] = useState<string | null>(null);
  const videoRefs = useRef<Record<string, HTMLVideoElement | null>>({});

  return (
    <div className="space-y-4">
      <div className="flex flex-wrap gap-2 pb-1">
        {CATEGORY_CHIPS.map((cat) => (
          <button
            key={cat.id}
            onClick={() => onSelectCategory(cat.id)}
            className={cn(
              "px-3 py-1 rounded-full text-footnote font-medium transition-colors",
              selectedCategory === cat.id
                ? "bg-foreground text-background"
                : "border border-border text-muted-foreground hover:bg-accent hover:text-foreground"
            )}
          >
            {cat.label}
          </button>
        ))}
      </div>

      <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">
        {filteredTemplates.map((tpl) => {
          const isSelected = selectedTemplate?.id === tpl.id;
          const isHovered = hoveredTemplateId === tpl.id;
          const previewJpgUrl = `${CORE_URL}/video/templates/${tpl.id}/preview.jpg`;
          const previewMp4Url = `${CORE_URL}/video/templates/${tpl.id}/preview.mp4`;

          return (
            <div
              key={tpl.id}
              onClick={() => onSelectTemplate(tpl)}
              onMouseEnter={() => {
                setHoveredTemplateId(tpl.id);
                const v = videoRefs.current[tpl.id];
                if (v) {
                  v.currentTime = 0;
                  v.play().catch(() => {});
                }
              }}
              onMouseLeave={() => {
                setHoveredTemplateId(null);
                const v = videoRefs.current[tpl.id];
                if (v) v.pause();
              }}
              className={cn(
                "group relative flex flex-col rounded-card border p-2 cursor-pointer transition-all duration-200 select-none",
                isSelected
                  ? "border-primary bg-primary/5 ring-2 ring-primary/30"
                  : "border-border bg-card/60 hover:border-primary/50 hover:bg-accent/40"
              )}
            >
              <div className="relative aspect-[9/16] w-full overflow-clip rounded-ctl bg-black/40">
                {/* eslint-disable-next-line @next/next/no-img-element */}
                <img
                  decoding="async"
                  loading="lazy"
                  src={previewJpgUrl}
                  alt={tpl.name}
                  className={cn(
                    "h-full w-full object-cover transition-opacity duration-300",
                    isHovered ? "opacity-0" : "opacity-100"
                  )}
                  onError={(e) => {
                    e.currentTarget.style.display = "none";
                  }}
                />

                <video
                  ref={(el) => {
                    videoRefs.current[tpl.id] = el;
                  }}
                  src={previewMp4Url}
                  muted
                  loop
                  playsInline
                  className={cn(
                    "absolute inset-0 h-full w-full object-cover transition-opacity duration-300",
                    isHovered ? "opacity-100" : "opacity-0 pointer-events-none"
                  )}
                />

                <div className="absolute top-2 left-2 flex gap-1 z-10">
                  <span className="rounded-xs bg-black/70 px-2 py-1 text-caption font-medium uppercase text-white backdrop-blur-sm">
                    {tpl.category}
                  </span>
                </div>

                <div className="absolute bottom-2 right-2 z-10">
                  <span className="rounded-xs bg-black/70 px-2 py-1 text-caption font-mono text-white/90 backdrop-blur-sm">
                    {tpl.duration?.min}s - {tpl.duration?.max}s
                  </span>
                </div>

                {isSelected && (
                  <div className="absolute top-2 right-2 z-10 flex h-5 w-5 items-center justify-center rounded-full bg-primary text-primary-foreground shadow">
                    <Check className="h-3 w-3" />
                  </div>
                )}
              </div>

              <div className="mt-2 flex flex-col gap-1">
                <h4 className="line-clamp-2 text-footnote font-semibold text-foreground group-hover:text-primary transition-colors">
                  {tpl.name}
                </h4>
                <div className="flex flex-wrap gap-1 mt-1">
                  {tpl.platforms.map((p) => (
                    <span
                      key={p}
                      className="rounded-xs bg-accent px-2 py-1 text-caption font-medium uppercase text-muted-foreground"
                    >
                      {p}
                    </span>
                  ))}
                </div>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
});
