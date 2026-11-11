"use client";

import React, { useCallback, useEffect, useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { enterOnce } from "@/lib/motion";
import { Wrench } from "@/components/ui/icons";
import { AiLoading } from "@/components/ui/AiLoading";
import { cn } from "@/lib/utils";
import { fileName, runTool } from "./api";
import { EASE } from "./constants";
import { ToolboxErrorBoundary } from "./shared/ToolboxErrorBoundary";
import { ToolboxErrorLine } from "./shared/ToolboxErrorLine";
import { ToolsCatalogTab } from "./tabs/ToolsCatalogTab";
import { CutoutsVaultTab } from "./tabs/CutoutsVaultTab";
import { AudioVaultTab } from "./tabs/AudioVaultTab";
import { VoicesVaultTab } from "./tabs/VoicesVaultTab";
import type { SubTab, Targets, ToolItem, VaultsData } from "./types";

function ToolboxPanelContent() {
  const [sub, setSub] = useState<SubTab>("tools");
  const [tools, setTools] = useState<ToolItem[]>([]);
  const [vaults, setVaults] = useState<VaultsData>({ cutouts: [], audios: [], voices: [] });
  const [targets, setTargets] = useState<Targets | null>(null);
  const [loading, setLoading] = useState(true);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [open, setOpen] = useState<string | null>("bg-remove");
  const [checking, setChecking] = useState(false);
  const [checkMsg, setCheckMsg] = useState<string | null>(null);
  const [saveFor, setSaveFor] = useState<string | null>(null);

  const refresh = useCallback(async (signal?: AbortSignal) => {
    try {
      const [a, b, c] = await Promise.all([
        fetch("/api/toolbox?action=list", { signal }).then((r) => r.json()),
        fetch("/api/toolbox?action=vaults", { signal }).then((r) => r.json()),
        fetch("/api/toolbox?action=targets", { signal }).then((r) => r.json()),
      ]);
      if (signal?.aborted) return;
      if (a.ok) setTools(a.tools);
      if (b.ok) setVaults(b.vaults);
      if (c.ok) setTargets(c);
      setLoadError(a.ok ? null : a.error || "Không đọc được danh mục công cụ");
    } catch (e: any) {
      if (e.name === "AbortError" || signal?.aborted) return;
      setLoadError(e.message);
    } finally {
      if (!signal?.aborted) setLoading(false);
    }
  }, []);

  useEffect(() => {
    const controller = new AbortController();
    refresh(controller.signal);
    return () => controller.abort();
  }, [refresh]);

  const checkFfmpeg = async () => {
    setChecking(true);
    setCheckMsg(null);
    try {
      const r = await runTool("ffmpeg-bin", {}, []);
      setCheckMsg(`Sẵn sàng: ${r.outputs.map((o) => fileName(o.path)).join(", ")}`);
    } catch (e: any) {
      setCheckMsg(`Lỗi: ${e.message}`);
    } finally {
      setChecking(false);
    }
  };

  const tabs: { id: SubTab; label: string; count: number }[] = [
    { id: "tools", label: "Công cụ", count: tools.length },
    { id: "cutouts", label: "Tách nền", count: vaults.cutouts.length },
    { id: "audio", label: "Canh từ", count: vaults.audios.length },
    { id: "voices", label: "Giọng", count: vaults.voices.length },
  ];

  const sectionAnim = (key: string) => ({
    initial: enterOnce(key, { opacity: 0 }),
    animate: { opacity: 1 },
    exit: { opacity: 0 },
    transition: { duration: 0.18, ease: EASE },
  });

  return (
    <div className="flex h-full flex-col overflow-hidden bg-background text-left text-foreground">
      <div className="shrink-0 space-y-1 border-b border-border p-3">
        <div className="flex items-center gap-2">
          <span className="rounded-xs bg-brand/10 p-1 text-brand">
            <Wrench className="h-4 w-4" />
          </span>
          <h2 className="text-callout font-semibold tracking-tight">Toolbox cục bộ</h2>
        </div>
        <p className="text-footnote text-muted-foreground">Nhập file, chạy tool ngay tại đây, rồi tải về hoặc lưu vào dự án / nhân vật.</p>
        <div className="flex flex-wrap items-center gap-1 pt-2">
          {tabs.map((t) => (
            <button
              key={t.id}
              type="button"
              onClick={() => setSub(t.id)}
              className={cn(
                "flex items-center gap-2 rounded-ctl px-3 py-1 text-caption font-medium transition-colors active:scale-95",
                sub === t.id ? "bg-accent font-semibold text-foreground" : "text-muted-foreground hover:bg-accent/50"
              )}
            >
              {t.label}
              <span className="text-caption opacity-70">({t.count})</span>
            </button>
          ))}
        </div>
      </div>

      <div className="scroll-quiet flex-1 space-y-3 overflow-y-auto p-3">
        {loading ? (
          <AiLoading label="Đang quét danh mục công cụ…" lines={3} />
        ) : (
          <AnimatePresence mode="wait" initial={false}>
            {loadError && <ToolboxErrorLine text={loadError} onClose={() => setLoadError(null)} />}

            {sub === "tools" && (
              <motion.div key="tools" {...sectionAnim("tb-tools")} className="space-y-3">
                <ToolsCatalogTab
                  tools={tools}
                  open={open}
                  setOpen={setOpen}
                  loadError={loadError}
                  targets={targets}
                  onDone={refresh}
                  checking={checking}
                  checkMsg={checkMsg}
                  onCheckFfmpeg={checkFfmpeg}
                />
              </motion.div>
            )}

            {sub === "cutouts" && (
              <motion.div key="cutouts" {...sectionAnim("tb-cutouts")} className="grid grid-cols-2 gap-2">
                <CutoutsVaultTab
                  cutouts={vaults.cutouts}
                  targets={targets}
                  saveFor={saveFor}
                  setSaveFor={setSaveFor}
                  onGoToBgRemove={() => {
                    setSub("tools");
                    setOpen("bg-remove");
                  }}
                />
              </motion.div>
            )}

            {sub === "audio" && (
              <motion.div key="audio" {...sectionAnim("tb-audio")} className="grid grid-cols-2 gap-2">
                <AudioVaultTab
                  audios={vaults.audios}
                  targets={targets}
                  saveFor={saveFor}
                  setSaveFor={setSaveFor}
                  onGoToSttAlign={() => {
                    setSub("tools");
                    setOpen("stt-align");
                  }}
                />
              </motion.div>
            )}

            {sub === "voices" && (
              <motion.div key="voices" {...sectionAnim("tb-voices")} className="space-y-3">
                <VoicesVaultTab
                  voices={vaults.voices}
                  targets={targets}
                  saveFor={saveFor}
                  setSaveFor={setSaveFor}
                  onDone={refresh}
                />
              </motion.div>
            )}
          </AnimatePresence>
        )}
      </div>
    </div>
  );
}

export function ToolboxPanel() {
  return (
    <ToolboxErrorBoundary>
      <ToolboxPanelContent />
    </ToolboxErrorBoundary>
  );
}
