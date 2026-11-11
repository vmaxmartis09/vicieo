import type { Graph, GraphValidationResult, NodeDefinition } from "../core-contracts";
import { isPortCompatible } from "./nodeDefinitions";

export function validateGraphLocally(
  graph: Graph,
  nodeCatalog: NodeDefinition[]
): GraphValidationResult {
  const errors: string[] = [];
  const warnings: string[] = [];
  const defMap = new Map(nodeCatalog.map((d) => [d.typeName, d]));
  const nodeMap = new Map(graph.nodes.map((n) => [n.id, n]));
  const adj = new Map<string, string[]>();
  const inDegree = new Map<string, number>();
  const connectedTargetPorts = new Set<string>();

  for (const n of graph.nodes) {
    adj.set(n.id, []);
    inDegree.set(n.id, 0);
  }

  for (const e of graph.edges) {
    const src = nodeMap.get(e.sourceNodeId);
    const tgt = nodeMap.get(e.targetNodeId);
    if (!src || !tgt) {
      errors.push(`Dây nối '${e.id}' trỏ tới node không tồn tại.`);
      continue;
    }
    const srcDef = defMap.get(src.type);
    const tgtDef = defMap.get(tgt.type);
    const srcPort = srcDef?.outputPorts.find((p) => p.name === e.sourcePort);
    const tgtPort = tgtDef?.inputPorts.find((p) => p.name === e.targetPort);
    if (srcPort && tgtPort && !isPortCompatible(srcPort.portType, tgtPort.portType)) {
      errors.push(
        `Không tương thích kiểu cổng: '${e.sourcePort}' (${srcPort.portType}) -> '${e.targetPort}' (${tgtPort.portType}) tại node '${tgt.id}'.`
      );
    }
    adj.get(src.id)?.push(tgt.id);
    inDegree.set(tgt.id, (inDegree.get(tgt.id) || 0) + 1);
    connectedTargetPorts.add(`${tgt.id}:${e.targetPort}`);
  }

  const queue: string[] = [];
  inDegree.forEach((deg, nid) => {
    if (deg === 0) queue.push(nid);
  });
  const topo: string[] = [];
  while (queue.length > 0) {
    const u = queue.shift()!;
    topo.push(u);
    for (const v of adj.get(u) || []) {
      const nextDeg = (inDegree.get(v) || 1) - 1;
      inDegree.set(v, nextDeg);
      if (nextDeg === 0) queue.push(v);
    }
  }

  if (topo.length !== graph.nodes.length) {
    errors.push("Phát hiện chu trình (vòng lặp) trong đồ thị.");
  }

  for (const n of graph.nodes) {
    const def = defMap.get(n.type);
    if (!def) continue;
    for (const port of def.inputPorts) {
      if (!port.required) continue;
      const isConnected = connectedTargetPorts.has(`${n.id}:${port.name}`);
      const hasParam =
        n.params && n.params[port.name] !== undefined && n.params[port.name] !== "";
      if (!isConnected && !hasParam) {
        warnings.push(
          `Node '${n.id}' (${def.displayName}) chưa nối cổng bắt buộc '${port.name}'.`
        );
      }
    }
  }

  return {
    isValid: errors.length === 0,
    errors,
    warnings,
    topologicalOrder: topo,
  };
}

/** Chuẩn hoá đồ thị theo hợp đồng của core trước khi lưu (tránh 422): state "succeeded" -> "completed", bản sinh thiếu nodeId/artifactId. */
