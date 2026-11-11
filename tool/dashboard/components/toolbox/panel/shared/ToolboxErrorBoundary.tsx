"use client";

import React from "react";
import { AlertTriangle, RefreshCw } from "@/components/ui/icons";

interface Props {
  children: React.ReactNode;
}

interface State {
  hasError: boolean;
  message: string;
}

export class ToolboxErrorBoundary extends React.Component<Props, State> {
  state: State = { hasError: false, message: "" };

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, message: error?.message || "Lỗi không xác định trong Toolbox" };
  }

  private handleReset = () => {
    this.setState({ hasError: false, message: "" });
  };

  render() {
    if (this.state.hasError) {
      return (
        <div className="flex h-full flex-col items-start justify-center gap-3 p-4 text-left text-foreground">
          <div className="flex items-start gap-2 rounded-xl border border-semantic-danger/30 bg-semantic-danger/5 p-3 text-footnote text-semantic-danger">
            <AlertTriangle className="mt-1 h-4 w-4 shrink-0" />
            <div className="space-y-1">
              <p className="font-semibold">Không thể hiển thị Toolbox</p>
              <p className="text-caption opacity-80">{this.state.message}</p>
            </div>
          </div>
          <button
            type="button"
            onClick={this.handleReset}
            className="flex h-7 items-center gap-2 rounded-md border border-border bg-card px-3 text-caption font-medium text-foreground transition-all hover:bg-accent active:scale-95"
          >
            <RefreshCw className="h-3.5 w-3.5" />
            Thử lại
          </button>
        </div>
      );
    }
    return this.props.children;
  }
}
