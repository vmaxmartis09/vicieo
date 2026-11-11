export interface ToolItem {
  id: string;
  name: string;
  version: string;
  status: "verified" | "installed" | "planned" | "broken";
  license?: { code: string; commercial_ok: boolean };
  requires_gpu: boolean;
  requires_ffmpeg: boolean;
}

export interface Targets {
  projects: { id: string; name: string }[];
  characters: Record<string, { id: string; name: string }[]>;
  active: string | null;
  sharedDir: string;
}

export interface RunOut {
  path: string;
  kind: string;
}

export type SubTab = "tools" | "cutouts" | "audio" | "voices";

export interface VaultsData {
  cutouts: any[];
  audios: any[];
  voices: any[];
}
