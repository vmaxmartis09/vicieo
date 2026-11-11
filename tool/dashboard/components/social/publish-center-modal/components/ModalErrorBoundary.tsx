"use client";

import React from "react";
import { AlertTriangle, RefreshCw, X } from "@/components/ui/icons";

interface Props {
  children: React.ReactNode;
  onClose?: () => void;
}

interface State {
  hasError: boolean;
  message: string;
}

export class ModalErrorBoundary extends React.Component<Props, State> {
  state: State = { hasError: false, message: "" };

  static getDerivedStateFromError(error: Error): State {
    return {
      hasError: true,
      message: error?.message || "Lỗi không xác định khi hiển thị Trung tâm xuất bản",
    };
  }

  render() {
    if (this.state.hasError) {
      return (
        <div
          role="dialog"
          aria-modal="true"
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm p-3 sm:p-6"
        >
          <div className="w-full max-w-md space-y-3 rounded-xl border border-border bg-card p-5 shadow-2xl text-foreground">
            <div className="flex items-start gap-2 rounded-xl border border-rose-500/30 bg-rose-500/5 p-3 text-xs text-semantic-danger">
              <AlertTriangle className="mt-1 h-4 w-4 shrink-0" />
              <div className="space-y-1">
                <p className="font-semibold">Không thể hiển thị Trung tâm xuất bản</p>
                <p className="text-caption opacity-80">{this.state.message}</p>
              </div>
            </div>
            <div className="flex justify-end gap-2">
              <button
                type="button"
                onClick={() => this.setState({ hasError: false, message: "" })}
                className="flex items-center gap-1 rounded-lg border border-border bg-background px-3 py-1 text-xs font-semibold text-foreground hover:bg-accent"
              >
                <RefreshCw className="h-3.5 w-3.5" />
                Thử lại
              </button>
              {this.props.onClose && (
                <button
                  type="button"
                  onClick={this.props.onClose}
                  className="flex items-center gap-1 rounded-lg bg-primary px-3 py-1 text-xs font-semibold text-primary-foreground hover:bg-primary/90"
                >
                  <X className="h-3.5 w-3.5" />
                  Đóng
                </button>
              )}
            </div>
          </div>
        </div>
      );
    }
    return this.props.children;
  }
}
