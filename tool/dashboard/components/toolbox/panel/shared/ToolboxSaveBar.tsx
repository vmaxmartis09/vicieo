"use client";

import React, { useEffect, useState } from "react";
import { Download, FolderOpen, Check, AlertTriangle } from "@/components/ui/icons";
import { cn } from "@/lib/utils";
import { previewUrl } from "../api";
import type { Targets } from "../types";

/** Tải về + Lưu vào dự án / nhân vật (chỉ sao chép, không ghi đè). */
export function ToolboxSaveBar({
  files,
  targets,
  onSaved,
}: {
  files: { label: string; path: string }[];
  targets: Targets | null;
  onSaved?: () => void;
}) {
  const [fileIdx, setFileIdx] = useState(0);
  const [where, setWhere] = useState<"project" | "character">("project");
  const [projectId, setProjectId] = useState("");
  const [characterId, setCharacterId] = useState("");
  const [saving, setSaving] = useState(false);
  const [msg, setMsg] = useState<{ ok: boolean; text: string } | null>(null);

  useEffect(() => {
    if (targets && !projectId) setProjectId(targets.active || targets.projects[0]?.id || "");
  }, [targets, projectId]);
  const chars = (targets && targets.characters[projectId]) || [];
  useEffect(() => {
    if (where === "character" && !chars.some((c) => c.id === characterId)) setCharacterId(chars[0]?.id || "");
  }, [where, projectId, chars, characterId]);

  const file = files[Math.min(fileIdx, files.length - 1)];
  if (!file) return null;

  const save = async () => {
    setSaving(true);
    setMsg(null);
    try {
      const r = await fetch("/api/toolbox/save", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          src: file.path,
          projectId,
          characterId: where === "character" ? characterId : undefined,
        }),
      });
      const j = await r.json();
      if (!j.ok) throw new Error(j.error || "Lưu thất bại");
      setMsg({ ok: true, text: `Đã lưu: ${j.relative}` });
      onSaved?.();
    } catch (e: any) {
      setMsg({ ok: false, text: e.message });
    } finally {
      setSaving(false);
    }
  };

  const sel = "h-7 min-w-0 rounded-ctl border border-input bg-card px-2 text-caption text-foreground";
  return (
    <div className="space-y-2 rounded-lg border border-border bg-muted/30 p-2">
      <div className="flex flex-wrap items-center gap-2">
        {files.length > 1 && (
          <select aria-label="Chọn file" value={fileIdx} onChange={(e) => setFileIdx(Number(e.target.value))} className={sel}>
            {files.map((f, i) => (
              <option key={f.path} value={i}>
                {f.label}
              </option>
            ))}
          </select>
        )}
        <a
          href={previewUrl(file.path, true)}
          download
          data-tip="Tải file về máy"
          aria-label="Tải về"
          className="flex h-7 items-center gap-1 rounded-md border border-border px-2 text-caption font-medium text-foreground transition-all hover:bg-accent active:scale-95"
        >
          <Download className="h-3.5 w-3.5" />
          Tải về
        </a>
      </div>
      <div className="flex flex-wrap items-center gap-2">
        <select aria-label="Lưu vào" value={where} onChange={(e) => setWhere(e.target.value as any)} className={sel}>
          <option value="project">Lưu vào dự án</option>
          <option value="character">Lưu vào nhân vật</option>
        </select>
        <select aria-label="Dự án" value={projectId} onChange={(e) => setProjectId(e.target.value)} className={sel}>
          {(targets?.projects || []).map((p) => (
            <option key={p.id} value={p.id}>
              {p.name}
            </option>
          ))}
        </select>
        {where === "character" && (
          <select aria-label="Nhân vật" value={characterId} onChange={(e) => setCharacterId(e.target.value)} className={sel}>
            {chars.length === 0 && <option value="">— chưa có nhân vật —</option>}
            {chars.map((c) => (
              <option key={c.id} value={c.id}>
                {c.name}
              </option>
            ))}
          </select>
        )}
        <button
          type="button"
          onClick={save}
          disabled={saving || !projectId || (where === "character" && !characterId)}
          data-tip={where === "character" ? "Sao chép vào thư mục toolbox của nhân vật" : "Sao chép vào thư viện của dự án"}
          className="flex h-7 items-center gap-1 rounded-md bg-primary px-3 text-caption font-medium text-primary-foreground transition-all hover:opacity-90 active:scale-95 disabled:cursor-not-allowed disabled:opacity-40"
        >
          <FolderOpen className="h-3.5 w-3.5" />
          {saving ? "Đang lưu…" : "Lưu"}
        </button>
      </div>
      {msg && (
        <p className={cn("flex items-start gap-1 text-caption", msg.ok ? "text-semantic-success" : "text-semantic-danger")}>
          {msg.ok ? <Check className="mt-1 h-3 w-3 shrink-0" /> : <AlertTriangle className="mt-1 h-3 w-3 shrink-0" />}
          <span className="break-all">{msg.text}</span>
        </p>
      )}
    </div>
  );
}
