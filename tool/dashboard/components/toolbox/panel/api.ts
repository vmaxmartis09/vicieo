import type { RunOut } from "./types";

export const previewUrl = (p: string, download = false) =>
  `/api/toolbox/preview?path=${encodeURIComponent(p)}${download ? "&download=1" : ""}`;

export const fileName = (p: string) => p.split(/[\\/]/).pop() || p;

export async function uploadFile(file: File): Promise<{ path: string; name: string }> {
  const fd = new FormData();
  fd.append("file", file);
  const r = await fetch("/api/toolbox/upload", { method: "POST", body: fd });
  const j = await r.json();
  if (!j.ok) throw new Error(j.error || "Tải lên thất bại");
  return { path: j.path, name: j.name };
}

export async function runTool(toolId: string, params: Record<string, unknown>, inputs: string[]) {
  const r = await fetch("/api/toolbox", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ toolId, inData: { params, inputs } }),
  });
  const j = await r.json();
  if (!j.ok) throw new Error(j.error || j.result?.error?.message || "Tool báo lỗi");
  return j.result as { outputs: RunOut[]; elapsed_s?: number; warnings?: string[] };
}
