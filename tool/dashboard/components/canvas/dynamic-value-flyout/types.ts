import type React from "react";
import type { ScriptFacet, ScriptShot } from "@/lib/api/nodes";

export type FlyoutTabId = "nodes" | "script" | "character" | "model" | "variable";

export interface DynamicValueFlyoutProps {
  nodeId?: string;
  portName?: string;
  currentValue?: string;
  topOffset: number;
  triggerRef: React.RefObject<HTMLButtonElement | null>;
  onClose: () => void;
  onSelectValue: (val: string) => void;
  onInsertText?: (text: string) => void;
}

export interface CandidateNodeItem {
  id: string;
  title: string;
  typeName: string;
  summary?: string;
  outputs: string[];
}

export interface ScriptDataState {
  facets: ScriptFacet[];
  shotFacets: { id: string; label: string }[];
  shots: ScriptShot[];
}

export interface TemplateVariableItem {
  label: string;
  tag: string;
  desc: string;
}
