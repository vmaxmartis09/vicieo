"use client";

import React from "react";
import type { ModelItem } from "@/lib/api/models";

export function GoogleLogo({ className = "h-3.5 w-3.5 shrink-0" }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 24 24">
      <path fill="#4285F4" d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z" />
      <path fill="#34A853" d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z" />
      <path fill="#FBBC05" d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.06H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.94l2.85-2.22.81-.63z" />
      <path fill="#EA4335" d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.06l3.66 2.84c.87-2.6 3.3-4.52 6.16-4.52z" />
    </svg>
  );
}

export function AlibabaLogo({ className = "h-3.5 w-3.5 shrink-0" }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 24 24" fill="none">
      <circle cx="12" cy="12" r="10" fill="#FF6A00" />
      <path d="M7 14.5c2-1 4-1.5 5-1.5s3 .5 5 1.5M9 11c1-1 2-1.5 3-1.5s2 .5 3 1.5" stroke="#FFFFFF" strokeWidth="1.8" strokeLinecap="round" />
      <circle cx="12" cy="7.5" r="1.2" fill="#FFFFFF" />
    </svg>
  );
}

export function KlingLogo({ className = "h-3.5 w-3.5 shrink-0" }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 24 24" fill="none">
      <rect width="24" height="24" rx="6" fill="#1C1C1E" />
      <path d="M7 6v12M7 12l6-6M9.5 9.5l7 8.5" stroke="#00F0FF" strokeWidth="2.4" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  );
}

export function ByteDanceLogo({ className = "h-3.5 w-3.5 shrink-0" }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 24 24" fill="none">
      <rect width="24" height="24" rx="5" fill="#3B82F6" />
      <path d="M6 8l3-2v12l-3-2V8zM11 6l3 2v8l-3 2V6zM16 9l3-2v10l-3-2V9z" fill="#FFFFFF" />
    </svg>
  );
}

export function MiniMaxLogo({ className = "h-3.5 w-3.5 shrink-0" }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 24 24" fill="none">
      <rect width="24" height="24" rx="5" fill="#6366F1" />
      <path d="M6 15V9l4 3 4-3v6M18 9v6" stroke="#FFFFFF" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  );
}

export function LightricksLogo({ className = "h-3.5 w-3.5 shrink-0" }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 24 24" fill="none">
      <circle cx="12" cy="12" r="10" fill="#EC4899" />
      <path d="M12 6l2 4 4 2-4 2-2 4-2-4-4-2 4-2 2-4z" fill="#FFFFFF" />
    </svg>
  );
}

export function FalLogo({ className = "h-3.5 w-3.5 shrink-0" }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 24 24" fill="none">
      <rect width="24" height="24" rx="5" fill="#10B981" />
      <path d="M8 7h8M8 12h5M8 17h3" stroke="#FFFFFF" strokeWidth="2.2" strokeLinecap="round" />
    </svg>
  );
}

export function MockLogo({ className = "h-3.5 w-3.5 shrink-0" }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 24 24" fill="none">
      <circle cx="12" cy="12" r="10" fill="#64748B" />
      <path d="M8 12h8M12 8v8" stroke="#FFFFFF" strokeWidth="2" strokeLinecap="round" />
    </svg>
  );
}

export function RunwayLogo({ className = "h-3.5 w-3.5 shrink-0" }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 24 24" fill="none">
      <rect width="24" height="24" rx="5" fill="#18181B" />
      <path d="M7 7h5a3 3 0 010 6H7V7zm5 6l4 5" stroke="#FAFAFA" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  );
}

export function LumaLogo({ className = "h-3.5 w-3.5 shrink-0" }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 24 24" fill="none">
      <rect width="24" height="24" rx="5" fill="#8B5CF6" />
      <path d="M7 7v10h10" stroke="#FFFFFF" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  );
}

export function PikaLogo({ className = "h-3.5 w-3.5 shrink-0" }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 24 24" fill="none">
      <circle cx="12" cy="12" r="10" fill="#F59E0B" />
      <path d="M9 7v10l7-5-7-5z" fill="#FFFFFF" />
    </svg>
  );
}

export function TencentLogo({ className = "h-3.5 w-3.5 shrink-0" }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 24 24" fill="none">
      <circle cx="12" cy="12" r="10" fill="#0284C7" />
      <path d="M8 9h8M12 9v8" stroke="#FFFFFF" strokeWidth="2.5" strokeLinecap="round" />
    </svg>
  );
}

export function ViduLogo({ className = "h-3.5 w-3.5 shrink-0" }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 24 24" fill="none">
      <rect width="24" height="24" rx="5" fill="#059669" />
      <path d="M7 8l5 8 5-8" stroke="#FFFFFF" strokeWidth="2.2" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  );
}

export function ZhipuLogo({ className = "h-3.5 w-3.5 shrink-0" }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 24 24" fill="none">
      <circle cx="12" cy="12" r="10" fill="#2563EB" />
      <circle cx="12" cy="12" r="4" fill="#FFFFFF" />
    </svg>
  );
}

export function HaiperLogo({ className = "h-3.5 w-3.5 shrink-0" }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 24 24" fill="none">
      <rect width="24" height="24" rx="5" fill="#D97706" />
      <path d="M7 7v10M17 7v10M7 12h10" stroke="#FFFFFF" strokeWidth="2" strokeLinecap="round" />
    </svg>
  );
}

export function getModelLogo(modelKey: string, provider: string): React.ComponentType<{ className?: string }> {
  const k = modelKey.toLowerCase();
  const p = provider.toLowerCase();

  if (k.includes("veo") || p.includes("google")) return GoogleLogo;
  if (k.includes("wan") || p.includes("alibaba")) return AlibabaLogo;
  if (k.includes("kling")) return KlingLogo;
  if (k.includes("seedance") || p.includes("bytedance")) return ByteDanceLogo;
  if (k.includes("hailuo") || k.includes("minimax")) return MiniMaxLogo;
  if (k.includes("ltx") || p.includes("lightricks")) return LightricksLogo;
  if (k.includes("runway")) return RunwayLogo;
  if (k.includes("luma")) return LumaLogo;
  if (k.includes("pika")) return PikaLogo;
  if (k.includes("hunyuan") || p.includes("tencent")) return TencentLogo;
  if (k.includes("vidu") || p.includes("shengshu")) return ViduLogo;
  if (k.includes("cogvideo") || p.includes("zhipu")) return ZhipuLogo;
  if (k.includes("haiper")) return HaiperLogo;
  if (k.includes("mock")) return MockLogo;
  if (p.includes("fal")) return FalLogo;

  return MockLogo;
}

export function formatModelName(m: ModelItem): { brand: string; name: string } {
  const raw = (m as any).name || m.display_name || m.displayName || m.key || "";
  if (raw.startsWith("Alibaba ")) return { brand: "Alibaba", name: raw.replace("Alibaba ", "") };
  if (raw.startsWith("ByteDance ")) return { brand: "ByteDance", name: raw.replace("ByteDance ", "") };
  if (raw.startsWith("Google ")) return { brand: "Google", name: raw.replace("Google ", "") };
  if (raw.startsWith("Lightricks ")) return { brand: "Lightricks", name: raw.replace("Lightricks ", "") };
  if (raw.startsWith("Runway ")) return { brand: "Runway", name: raw.replace("Runway ", "") };
  if (raw.startsWith("Luma ")) return { brand: "Luma", name: raw.replace("Luma ", "") };
  if (raw.startsWith("Pika ")) return { brand: "Pika", name: raw.replace("Pika ", "") };
  if (raw.startsWith("Tencent ")) return { brand: "Tencent", name: raw.replace("Tencent ", "") };
  if (raw.startsWith("Shengshu ")) return { brand: "Shengshu", name: raw.replace("Shengshu ", "") };
  if (raw.startsWith("Zhipu ")) return { brand: "Zhipu", name: raw.replace("Zhipu ", "") };
  if (raw.startsWith("Haiper ")) return { brand: "Haiper", name: raw.replace("Haiper ", "") };
  if (raw.startsWith("Kling ")) return { brand: "Kwai", name: raw };
  if (raw.startsWith("Hailuo ")) return { brand: "MiniMax", name: raw };
  if (m.provider === "fal") return { brand: "fal.ai", name: raw };
  if (m.provider === "mock") return { brand: "Local Mock", name: raw };
  return { brand: m.provider.toUpperCase(), name: raw };
}
