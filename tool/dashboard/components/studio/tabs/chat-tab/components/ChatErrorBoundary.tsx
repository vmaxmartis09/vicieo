"use client";

import React from "react";
import { AlertTriangle, RotateCcw } from "@/components/ui/icons";

interface Props {
  children: React.ReactNode;
  onReset?: () => void;
}

interface State {
  hasError: boolean;
  errorMessage: string;
}

export class ChatErrorBoundary extends React.Component<Props, State> {
  state: State = { hasError: false, errorMessage: "" };

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, errorMessage: error?.message || "Unexpected chat error" };
  }

  handleRetry = () => {
    this.setState({ hasError: false, errorMessage: "" });
    this.props.onReset?.();
  };

  render() {
    if (this.state.hasError) {
      return (
        <div
          role="alert"
          className="pointer-events-none fixed inset-x-0 bottom-14 md:bottom-3 z-40 flex justify-center px-3"
        >
          <div className="pointer-events-auto flex w-full max-w-[540px] items-center justify-between gap-3 rounded-panel border border-semantic-danger/50 bg-card-elevated px-4 py-3 text-footnote shadow-2xl ring-1 ring-foreground/10">
            <div className="flex min-w-0 items-center gap-2 text-semantic-danger">
              <AlertTriangle className="h-4 w-4 shrink-0" />
              <span className="truncate font-medium">
                Trợ lý Chat gặp sự cố hiển thị ({this.state.errorMessage})
              </span>
            </div>
            <button
              type="button"
              onClick={this.handleRetry}
              className="inline-flex h-7 shrink-0 items-center gap-1 rounded-ctl border border-border bg-card px-2 text-caption font-semibold text-foreground hover:bg-accent"
            >
              <RotateCcw className="h-3 w-3" />
              <span>Thử lại</span>
            </button>
          </div>
        </div>
      );
    }
    return this.props.children;
  }
}
