"use client";

import React, { useState, useEffect } from "react";
import { fetchProjectPolicy, type ContentPolicy } from "@/lib/api/policy";
import { useStudioStore } from "@/store";
import { ShieldCheck } from "@/components/ui/icons";

/** Dòng tóm tắt chính sách nội dung của dự án (dùng ở card Chính sách và Kiểm tra chất lượng). */
export function PolicyBadge() {
  const projectId = useStudioStore((s: any) => s.activeProjectId) as string;
  const [pol, setPol] = useState<ContentPolicy | null | undefined>(undefined);
  useEffect(() => {
    let alive = true;
    fetchProjectPolicy(projectId).then((p) => alive && setPol(p));
    return () => {
      alive = false;
    };
  }, [projectId]);
  const rules = pol?.rules.filter((r) => r.enabled) || [];
  const terms = rules.reduce((n, r) => n + r.terms.length, 0);
  return (
    <div className="flex items-center gap-2 rounded-ctl border border-dashed border-border px-2 py-2 text-footnote">
      <ShieldCheck className="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
      <span className="min-w-0 flex-1 truncate text-foreground">
        {pol === undefined
          ? "Đang đọc chính sách…"
          : pol === null
          ? "Chưa đọc được chính sách"
          : pol.name}
      </span>
      {pol && (
        <span className="shrink-0 text-footnote text-muted-foreground">
          {rules.length} quy tắc · {terms} từ
        </span>
      )}
    </div>
  );
}
