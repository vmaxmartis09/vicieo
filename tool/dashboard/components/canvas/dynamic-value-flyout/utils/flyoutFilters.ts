import type { Character } from "@/lib/repositories";
import type { ExtendedModelItem } from "@/lib/data/extendedModelsCatalog";
import type { CandidateNodeItem } from "../types";

export function buildCandidateNodes(
  nodes: any[],
  nodeCatalog: any[],
  excludeNodeId?: string
): CandidateNodeItem[] {
  const list: CandidateNodeItem[] = [];
  for (const n of nodes) {
    if (excludeNodeId && n.id === excludeNodeId) continue;
    const base = nodeCatalog.find((d: any) => d.typeName === n.data.nodeType);
    const title =
      typeof n.data.params?._customName === "string" && n.data.params._customName.trim()
        ? n.data.params._customName
        : base?.displayName || n.data.nodeType;

    const summary =
      typeof n.data.summaryText === "string"
        ? n.data.summaryText
        : typeof n.data.params?.content === "string"
        ? n.data.params.content
        : typeof n.data.params?.prompt === "string"
        ? n.data.params.prompt
        : undefined;

    const outputs = (base?.outputPorts || []).map((p: any) => p.name);

    list.push({
      id: n.id,
      title,
      typeName: n.data.nodeType,
      summary,
      outputs,
    });
  }
  return list;
}

export function filterCandidateNodes(
  candidates: CandidateNodeItem[],
  search: string
): CandidateNodeItem[] {
  const q = search.trim().toLowerCase();
  if (!q) return candidates;
  return candidates.filter(
    (n) =>
      n.title.toLowerCase().includes(q) ||
      n.typeName.toLowerCase().includes(q) ||
      (n.summary && n.summary.toLowerCase().includes(q))
  );
}

export function filterCharactersList(
  characters: Character[],
  search: string
): Character[] {
  const q = search.trim().toLowerCase();
  if (!q) return characters;
  return characters.filter(
    (c) =>
      c.name.toLowerCase().includes(q) ||
      (c.promptLock && c.promptLock.toLowerCase().includes(q)) ||
      (c.description && c.description.toLowerCase().includes(q))
  );
}

export function filterExtendedModels(
  catalog: ExtendedModelItem[],
  search: string
): ExtendedModelItem[] {
  const q = search.trim().toLowerCase();
  if (!q) return catalog;
  return catalog.filter(
    (m) =>
      m.name.toLowerCase().includes(q) ||
      m.brand.toLowerCase().includes(q) ||
      m.key.toLowerCase().includes(q)
  );
}
