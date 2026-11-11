import type { Platform, TemplateMeta } from "@/lib/api/video";
import { generateTemplateGraph } from "@/lib/template-graphs";
import { validateGraphWithCore, runGraphOnCore } from "@/lib/core-api";
import { CORE_URL } from "@/lib/repositories/core";
import type { StudioStore } from "@/store";
import type { LintReport, ProductFormInfo } from "./types";

export async function requestCostEstimate(opts: {
  activeProjectId: string;
  selectedTemplate: TemplateMeta;
  topic: string;
  scriptText: string;
  selectedCharacterId: string;
  selectedVoiceId: string;
  platforms: Platform[];
  slotValues: Record<string, any>;
  productInfo: ProductFormInfo;
  isMockMode: boolean;
  runPolicy: StudioStore["runPolicy"];
  topicCount: number;
}): Promise<number> {
  const dummyBundle = generateTemplateGraph({
    projectId: opts.activeProjectId,
    template: opts.selectedTemplate,
    topic: opts.topic || "Video mới",
    scriptText: opts.scriptText,
    characterId: opts.selectedCharacterId,
    voiceId: opts.selectedVoiceId,
    platforms: opts.platforms,
    slotValues: opts.slotValues,
    productInfo: opts.productInfo,
  });

  const res = await fetch(`${CORE_URL}/projects/${opts.activeProjectId}/estimate`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      graph: dummyBundle.coreGraph,
      mockMode: opts.isMockMode || opts.runPolicy?.image === "mock",
      runPolicy: opts.runPolicy,
    }),
  });
  if (!res.ok) return 0.0;
  const data = await res.json();
  return (data.totalEstimatedCostUsd || 0.0) * opts.topicCount;
}

export async function requestScriptAndHooks(opts: {
  activeProjectId: string;
  prompt: string;
  selectedCharacterId: string;
  enableHookABC: boolean;
  isMockMode: boolean;
  runPolicy: StudioStore["runPolicy"];
}) {
  const [scriptRes, hookRes] = await Promise.all([
    fetch(`${CORE_URL}/projects/${opts.activeProjectId}/ai/script`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        prompt: opts.prompt,
        mockMode: opts.isMockMode || opts.runPolicy?.text === "mock",
        runPolicy: opts.runPolicy,
      }),
    }),
    opts.enableHookABC
      ? fetch(`${CORE_URL}/video/craft/hooks`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ topic: opts.prompt, character: opts.selectedCharacterId }),
        }).catch(() => null)
      : Promise.resolve(null),
  ]);
  if (!scriptRes.ok) return null;
  const data = await scriptRes.json();
  let markdown: string = data.markdown || "";
  let hookData: any = null;

  if (hookRes && hookRes.ok) {
    hookData = await hookRes.json();
    if (hookData.selected_hook) {
      const lines = markdown.split("\n");
      if (lines.length > 0) {
        lines[0] = hookData.selected_hook;
        markdown = lines.join("\n");
      }
    }
  }
  return { markdown, hookData };
}

export async function requestCraftLint(opts: {
  script: string;
  platform: Platform;
}): Promise<LintReport | null> {
  const res = await fetch(`${CORE_URL}/video/craft/lint`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      script: opts.script,
      duration_sec: 30.0,
      platform: opts.platform || "tiktok",
    }),
  });
  return res.ok ? ((await res.json()) as LintReport) : null;
}

export async function dispatchBatchVideoCreation(opts: {
  topicsList: string[];
  activeProjectId: string;
  selectedTemplate: TemplateMeta;
  isBatchMode: boolean;
  scriptText: string;
  selectedCharacterId: string;
  selectedVoiceId: string;
  platforms: Platform[];
  slotValues: Record<string, any>;
  productInfo: ProductFormInfo;
  nodeCatalog: StudioStore["nodeCatalog"];
  isMockMode: boolean;
  runPolicy: StudioStore["runPolicy"];
  createEpisode: StudioStore["createEpisode"];
  addJob: StudioStore["addJob"];
  updateJob: StudioStore["updateJob"];
  onProgress: (text: string) => void;
}) {
  for (let i = 0; i < opts.topicsList.length; i++) {
    const itemTopic = opts.topicsList[i];
    opts.onProgress(`Đang tạo video ${i + 1}/${opts.topicsList.length}: "${itemTopic.slice(0, 30)}..."`);

    const bundle = generateTemplateGraph({
      projectId: opts.activeProjectId,
      template: opts.selectedTemplate,
      topic: itemTopic,
      scriptText: opts.isBatchMode ? "" : opts.scriptText,
      characterId: opts.selectedCharacterId,
      voiceId: opts.selectedVoiceId,
      platforms: opts.platforms,
      slotValues: opts.slotValues,
      productInfo: opts.productInfo,
    });

    const newEp = opts.createEpisode(itemTopic, {
      customGraph: bundle.reactFlowGraph,
      autoSelect: i === opts.topicsList.length - 1,
    });

    await validateGraphWithCore(opts.activeProjectId, bundle.coreGraph, opts.nodeCatalog);

    const jobId = `job_${Date.now()}_${i}`;
    opts.addJob({
      id: jobId,
      projectId: opts.activeProjectId,
      type: "render",
      model: opts.selectedTemplate.id,
      status: "running",
      progress: 10,
      estimatedCostUsd: opts.isMockMode ? 0.0 : 0.05,
      actualCostUsd: 0.0,
      createdAt: new Date().toISOString(),
      logs: [`Bắt đầu render mẫu ${opts.selectedTemplate.name} cho tập ${newEp.title}`],
    });

    runGraphOnCore(opts.activeProjectId, {
      graph: bundle.coreGraph,
      mode: "all" as any,
      mockMode: opts.isMockMode,
      runPolicy: opts.runPolicy,
    })
      .then((summary) => {
        if (summary) {
          opts.updateJob(jobId, {
            status: summary.isSuccess ? "completed" : "failed",
            progress: 100,
            actualCostUsd: summary.totalCostUsd || 0.0,
            completedAt: new Date().toISOString(),
            outputUrl: `${CORE_URL}/data/projects/${opts.activeProjectId}/output/final_render.mp4`,
          });
        }
      })
      .catch((err) => {
        opts.updateJob(jobId, { status: "failed", logs: [String(err)] });
      });
  }
}
