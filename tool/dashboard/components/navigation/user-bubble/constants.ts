import type { ProviderKeyId } from "@/store/slices/userSettingsSlice";

export const PROVIDERS: Array<{ id: ProviderKeyId; label: string }> = [
  { id: "gemini", label: "Google Gemini / Veo" },
  { id: "fal", label: "fal.ai (LTX / Flux)" },
  { id: "openai", label: "OpenAI" },
  { id: "replicate", label: "Replicate" },
];

export const ASPECT_RATIO_OPTIONS = ["9:16", "16:9", "1:1", "4:5"] as const;

export const INLINE_OR_HIDDEN_ROW_IDS = ["profile", "core", "shortcuts"] as const;
