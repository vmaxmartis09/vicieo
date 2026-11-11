"use client";

import React from "react";
import { AlertTriangle } from "@/components/ui/icons";

interface Props {
  children: React.ReactNode;
}

interface State {
  hasError: boolean;
}

export class BubbleErrorBoundary extends React.Component<Props, State> {
  state: State = { hasError: false };

  static getDerivedStateFromError(): State {
    return { hasError: true };
  }

  render() {
    if (this.state.hasError) {
      return (
        <button
          type="button"
          onClick={() => this.setState({ hasError: false })}
          title="Lỗi menu tài khoản — bấm thử lại"
          className="flex h-10 w-10 items-center justify-center rounded-full border border-semantic-danger/40 bg-card text-semantic-danger"
        >
          <AlertTriangle className="h-4 w-4" />
        </button>
      );
    }
    return this.props.children;
  }
}
