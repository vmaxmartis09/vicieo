/**
 * API client tra cứu sổ Model và Nhà cung cấp AI (Session G / Hợp đồng C2)
 */

import { CORE_URL } from "@/lib/repositories/core";

export interface ModelPricing {
  cost_per_generation?: number;
  cost_per_second?: number;
  cost_per_minute?: number;
  cost_per_1k_tokens?: number;
  cost_per_1k_chars?: number;
}

export interface ModelItem {
  key: string;
  display_name: string;
  displayName?: string;
  provider: string;
  model_id: string;
  modelId?: string;
  kind: string;
  tier: "free" | "freemium" | "paid" | "local" | string;
  supports_reference?: boolean;
  supportsReference?: boolean;
  max_references?: number;
  maxReferences?: number;
  quota_note?: string;
  quotaNote?: string;
  speed_note?: string;
  speedNote?: string;
  durations_sec?: number[];
  durationsSec?: number[];
  capabilities?: Record<string, any>;
  pricing?: ModelPricing;
  available: boolean;
}

export interface ProviderItem {
  id: string;
  label: string;
  tier: string;
  needs_key: boolean;
  configured: boolean;
  quota_note?: string;
  capabilities?: Record<string, any>;
  models?: string[];
}

export async function fetchModels(kind?: string, provider?: string): Promise<ModelItem[]> {
  try {
    const params = new URLSearchParams();
    if (kind) params.set("kind", kind);
    if (provider) params.set("provider", provider);
    const qs = params.toString() ? `?${params.toString()}` : "";
    const res = await fetch(`${CORE_URL}/models${qs}`, { cache: "no-store" });
    if (!res.ok) return [];
    return await res.json();
  } catch (err) {
    console.warn("fetchModels failed:", err);
    return [];
  }
}

export async function fetchProviders(): Promise<ProviderItem[]> {
  try {
    const res = await fetch(`${CORE_URL}/providers`, { cache: "no-store" });
    if (!res.ok) return [];
    const data = await res.json();
    return data.providers || [];
  } catch (err) {
    console.warn("fetchProviders failed:", err);
    return [];
  }
}

export function formatTierBadge(tier: string, pricing?: ModelPricing, modelKey?: string): {
  labelVi: string;
  labelEn: string;
  colorClass: string;
} {
  const isMock = tier === "mock" || tier === "local" || (modelKey && modelKey.includes("mock"));
  if (isMock) {
    return {
      labelVi: "$0.00 (Test)",
      labelEn: "$0.00 (Test)",
      colorClass: "bg-muted text-muted-foreground border-border",
    };
  }

  let costStr = "";
  if (pricing?.cost_per_generation) costStr = `~$${pricing.cost_per_generation}/ảnh`;
  else if (pricing?.cost_per_second) costStr = `~$${pricing.cost_per_second}/s`;
  else costStr = "Thương mại";

  return {
    labelVi: costStr,
    labelEn: costStr,
    colorClass: "bg-brand/10 text-brand border-brand/20 font-mono",
  };
}
