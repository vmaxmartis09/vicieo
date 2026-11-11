"use client";

import React from "react";
import { Images } from "@/components/ui/icons";
import { cn } from "@/lib/utils";
import { Empty } from "./Empty";

export function ImagePreview({
  url,
  alt,
  compact,
}: {
  url?: string;
  alt: string;
  compact?: boolean;
}) {
  if (!url) {
    return (
      <Empty
        icon={<Images className="h-6 w-6" />}
        text="Ảnh sẽ hiện ở đây"
        tall
      />
    );
  }
  return (
    // eslint-disable-next-line @next/next/no-img-element
    <img
      decoding="async"
      src={url}
      alt={alt}
      loading="lazy"
      className={cn(
        "w-full rounded-card border border-border object-cover object-top",
        compact ? "max-h-32" : "max-h-64"
      )}
    />
  );
}
