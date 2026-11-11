"use client";

import React, { useMemo, useRef, useState } from "react";
import type { Edge } from "@xyflow/react";
import { useStudioStore } from "@/store";
import {
  Film,
  Layers,
  ScrollText,
  Search,
  Tag,
  Users,
  X,
} from "@/components/ui/icons";
import { cn } from "@/lib/utils";
import { PORT_TYPE_COLORS } from "@/store/slices/canvasSlice";
import { EXTENDED_MODELS_CATALOG } from "@/lib/data/extendedModelsCatalog";
import { TEMPLATE_VARIABLES } from "./constants";
import { usePopoverDismiss } from "./hooks/usePopoverDismiss";
import { useFlyoutScriptData } from "./hooks/useFlyoutScriptData";
import {
  buildCandidateNodes,
  filterCandidateNodes,
  filterCharactersList,
  filterExtendedModels,
} from "./utils/flyoutFilters";
import { NodesTab } from "./tabs/NodesTab";
import { ScriptShotsTab } from "./tabs/ScriptShotsTab";
import { CharactersTab } from "./tabs/CharactersTab";
import { ModelsTab } from "./tabs/ModelsTab";
import { VariablesTab } from "./tabs/VariablesTab";
import type { CandidateNodeItem, DynamicValueFlyoutProps, FlyoutTabId } from "./types";

export type { DynamicValueFlyoutProps } from "./types";

const FLYOUT_TABS: Array<{ id: FlyoutTabId; label: string; icon: React.ComponentType<{ className?: string }> }> = [
  { id: "nodes", label: "Node", icon: Layers },
  { id: "script", label: "Kịch bản", icon: ScrollText },
  { id: "character", label: "Nhân vật", icon: Users },
  { id: "model", label: "Model", icon: Film },
  { id: "variable", label: "Biến", icon: Tag },
];

export function DynamicValueFlyout({
  nodeId,
  portName,
  currentValue = "",
  topOffset,
  triggerRef,
  onClose,
  onSelectValue,
  onInsertText,
}: DynamicValueFlyoutProps) {
  const [activeTab, setActiveTab] = useState<FlyoutTabId>("nodes");
  const [search, setSearch] = useState("");
  const popoverRef = useRef<HTMLDivElement>(null);

  const nodes = useStudioStore((s) => s.nodes);
  const edges = useStudioStore((s) => s.edges);
  const characters = useStudioStore((s) => s.characters);
  const activeProjectId = useStudioStore((s) => s.activeProjectId);
  const nodeCatalog = useStudioStore((s) => s.nodeCatalog);
  const setNodesAndEdges = useStudioStore((s) => s.setNodesAndEdges);

  usePopoverDismiss(popoverRef, triggerRef, onClose);
  const { scriptData, loadingScript } = useFlyoutScriptData(activeTab, activeProjectId);

  const candidateNodes = useMemo(
    () => buildCandidateNodes(nodes, nodeCatalog, nodeId),
    [nodes, nodeId, nodeCatalog]
  );

  const handleConnectNode = (targetNode: CandidateNodeItem, outPort?: string) => {
    const chosenPort = outPort || targetNode.outputs[0] || (portName || "output");
    const dynamicRef = `{${targetNode.title}.${chosenPort}}`;

    if (nodeId && portName) {
      const edgeColor = PORT_TYPE_COLORS["text"] || "#737373";
      const newEdge: Edge = {
        id: `edge_${Date.now().toString(36)}${Math.random().toString(36).slice(2, 6)}`,
        source: targetNode.id,
        sourceHandle: chosenPort,
        target: nodeId,
        targetHandle: portName,
        type: "default",
        style: { stroke: edgeColor, strokeWidth: 1.75 },
        data: { portType: "text" },
      };
      const rest = edges.filter((e) => !(e.target === nodeId && e.targetHandle === portName));
      setNodesAndEdges(nodes, [...rest, newEdge], true);
    }

    onSelectValue(dynamicRef);
    onClose();
  };

  const filteredNodes = useMemo(() => filterCandidateNodes(candidateNodes, search), [candidateNodes, search]);
  const filteredCharacters = useMemo(() => filterCharactersList(characters, search), [characters, search]);
  const filteredModels = useMemo(() => filterExtendedModels(EXTENDED_MODELS_CATALOG, search), [search]);

  const handleSelectDynamicValue = (val: string) => {
    if (onInsertText && currentValue) {
      onInsertText(val);
    } else {
      onSelectValue(val);
    }
    onClose();
  };

  return (
    <div
      ref={popoverRef}
      onClick={(e) => e.stopPropagation()}
      style={{
        position: "absolute",
        left: "calc(100% + 10px)",
        top: Math.max(0, topOffset - 16),
        width: "330px",
        zIndex: 9999,
      }}
      className="nodrag nowheel cursor-default space-y-2 rounded-card border border-border bg-card p-3 text-foreground shadow-2xl animate-in fade-in-0 zoom-in-95 duration-150"
    >
      {/* Header */}
      <div className="flex items-center justify-between pb-2 border-b border-border/60">
        <span className="flex items-center gap-2 text-footnote font-semibold text-foreground">
          <span className="font-mono text-brand font-bold">{`{ }`}</span>
          <span>Chọn nguồn giá trị {`{nguồn.value}`}</span>
        </span>
        <button
          type="button"
          onClick={onClose}
          className="flex h-6 w-6 items-center justify-center rounded-ctl text-muted-foreground hover:bg-accent hover:text-foreground transition-colors"
          aria-label="Đóng bảng chọn nguồn"
        >
          <X className="h-3 w-3" />
        </button>
      </div>

      {/* Search bar */}
      <div className="relative">
        <Search className="absolute left-2 top-2 h-3 w-3 text-muted-foreground pointer-events-none" />
        <input
          type="text"
          autoFocus
          placeholder="Tìm node, kịch bản, nhân vật, model..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          className="h-7 w-full rounded-ctl border border-border bg-background py-1 pl-8 pr-2 text-footnote text-foreground placeholder:text-muted-foreground/60 outline-none focus:border-brand"
        />
      </div>

      {/* Tabs */}
      <div className="flex items-center gap-1 border-b border-border/60 pb-2 overflow-x-auto no-scrollbar scrollbar-none">
        {FLYOUT_TABS.map((t) => {
          const IconComp = t.icon;
          const active = activeTab === t.id;
          return (
            <button
              key={t.id}
              type="button"
              onClick={() => setActiveTab(t.id)}
              className={cn(
                "flex items-center gap-1 px-2 py-1 rounded-ctl text-caption font-medium transition-colors shrink-0",
                active ? "bg-accent text-foreground font-semibold" : "text-muted-foreground hover:bg-muted"
              )}
            >
              <IconComp className="h-3 w-3" />
              <span>{t.label}</span>
            </button>
          );
        })}
      </div>

      {activeTab === "nodes" && <NodesTab nodes={filteredNodes} onConnectNode={handleConnectNode} />}
      {activeTab === "script" && (
        <ScriptShotsTab loading={loadingScript} scriptData={scriptData} onSelect={handleSelectDynamicValue} />
      )}
      {activeTab === "character" && (
        <CharactersTab characters={filteredCharacters} onSelect={handleSelectDynamicValue} />
      )}
      {activeTab === "model" && <ModelsTab models={filteredModels} onSelect={handleSelectDynamicValue} />}
      {activeTab === "variable" && (
        <VariablesTab variables={TEMPLATE_VARIABLES} onSelect={handleSelectDynamicValue} />
      )}
    </div>
  );
}
