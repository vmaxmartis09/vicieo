"use client";

import React from "react";
import { showAlert } from "@/lib/confirm";
import { cn } from "@/lib/utils";
import type { PublishableVideo, SocialAccount } from "@/lib/api/social";
import { Layers } from "../../icons";
import { MAX_SELECTED_ACCOUNTS } from "../constants";
import type { CompatibilityMatrix } from "../types";

interface Step2AccountMatrixProps {
  accounts: SocialAccount[];
  selectedAccounts: SocialAccount[];
  selectedAccountIds: string[];
  onChangeSelectedAccountIds: (ids: string[]) => void;
  allGroups: string[];
  selectedVideos: PublishableVideo[];
  compatibilityMatrix: CompatibilityMatrix;
  ignoreErrors: boolean;
  onToggleIgnoreErrors: (value: boolean) => void;
}

export function Step2AccountMatrix({
  accounts,
  selectedAccounts,
  selectedAccountIds,
  onChangeSelectedAccountIds,
  allGroups,
  selectedVideos,
  compatibilityMatrix,
  ignoreErrors,
  onToggleIgnoreErrors,
}: Step2AccountMatrixProps) {
  return (
    <div className="space-y-5">
      {/* Account selection header */}
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div className="flex items-center gap-2">
          <span className="text-xs font-semibold text-muted-foreground uppercase tracking-wider">
            Chọn tài khoản ({selectedAccountIds.length}/{accounts.length}):
          </span>
          {allGroups.map((g) => (
            <button
              key={g}
              type="button"
              onClick={() => {
                const groupAccIds = accounts
                  .filter((a) => (a.groups || []).includes(g))
                  .map((a) => a.id);
                onChangeSelectedAccountIds(
                  Array.from(new Set([...selectedAccountIds, ...groupAccIds]))
                );
              }}
              className="rounded-lg border border-border bg-accent px-2 py-1 text-caption font-medium text-foreground hover:bg-accent/80 transition-colors"
            >
              + Nhóm "{g}"
            </button>
          ))}
        </div>

        <div className="flex items-center gap-2">
          <button
            type="button"
            onClick={() => onChangeSelectedAccountIds(accounts.map((a) => a.id))}
            className="text-xs text-primary hover:underline font-semibold"
          >
            Chọn tất cả ({accounts.length})
          </button>
          <span className="text-muted-foreground">·</span>
          <button
            type="button"
            onClick={() => onChangeSelectedAccountIds([])}
            className="text-xs text-muted-foreground hover:underline"
          >
            Bỏ chọn tất cả
          </button>
        </div>
      </div>

      {/* Accounts selection row */}
      <div className="flex flex-wrap gap-2">
        {accounts.map((acc) => {
          const isSelected = selectedAccountIds.includes(acc.id);
          return (
            <div
              key={acc.id}
              onClick={() => {
                if (isSelected) {
                  onChangeSelectedAccountIds(
                    selectedAccountIds.filter((id) => id !== acc.id)
                  );
                } else {
                  if (selectedAccountIds.length >= MAX_SELECTED_ACCOUNTS) {
                    showAlert(
                      "Giới hạn tài khoản",
                      "Tối đa chọn 10 tài khoản",
                      "warning"
                    );
                    return;
                  }
                  onChangeSelectedAccountIds([...selectedAccountIds, acc.id]);
                }
              }}
              className={cn(
                "flex items-center gap-2 rounded-xl border p-2 cursor-pointer transition-all select-none shadow-2xs",
                isSelected
                  ? "border-primary bg-primary/10 text-foreground"
                  : "border-border bg-card text-muted-foreground hover:border-primary/40"
              )}
            >
              <div className="flex h-6 w-6 items-center justify-center rounded-lg bg-accent text-caption font-bold uppercase text-foreground">
                {acc.platform.slice(0, 2)}
              </div>
              <div className="text-left">
                <p className="text-xs font-semibold leading-none text-foreground">
                  {acc.display_name}
                </p>
                <p className="text-caption text-muted-foreground">{acc.platform}</p>
              </div>
              <div
                className={cn(
                  "ml-1 h-4 w-4 rounded-full border flex items-center justify-center text-caption",
                  isSelected
                    ? "bg-primary border-primary text-white"
                    : "border-muted-foreground/40"
                )}
              >
                {isSelected ? "✓" : ""}
              </div>
            </div>
          );
        })}
      </div>

      {/* Compatibility Matrix: Video x Account */}
      <div className="space-y-2 pt-2 border-t border-border">
        <div className="flex items-center justify-between">
          <h4 className="text-xs font-bold text-foreground flex items-center gap-2">
            <Layers className="h-4 w-4 text-primary" />
            Ma trận Tương thích ({selectedVideos.length} Video ×{" "}
            {selectedAccounts.length} Tài khoản)
          </h4>
          <label className="flex items-center gap-2 text-xs text-muted-foreground cursor-pointer">
            <input
              type="checkbox"
              checked={ignoreErrors}
              onChange={(e) => onToggleIgnoreErrors(e.target.checked)}
              className="rounded border-border text-primary focus:ring-primary"
            />
            <span>Bỏ qua các cặp cảnh báo / lỗi khi lên lịch</span>
          </label>
        </div>

        <div className="overflow-x-auto rounded-xl border border-border bg-card shadow-2xs">
          <table className="w-full text-xs text-left">
            <thead className="bg-muted/50 border-b border-border text-caption font-semibold text-muted-foreground">
              <tr>
                <th className="p-3 font-medium">Video đã chọn</th>
                {selectedAccounts.map((acc) => (
                  <th key={acc.id} className="p-3 text-center min-w-[120px]">
                    <div className="flex flex-col items-center">
                      <span className="font-bold text-foreground">
                        {acc.display_name}
                      </span>
                      <span className="text-caption uppercase font-normal">
                        {acc.platform}
                      </span>
                    </div>
                  </th>
                ))}
              </tr>
            </thead>
            <tbody className="divide-y divide-border/60">
              {selectedVideos.map((v) => (
                <tr key={v.id} className="hover:bg-accent/20 transition-colors">
                  <td className="p-3 max-w-[240px]">
                    <p className="font-semibold text-foreground truncate">
                      {v.title}
                    </p>
                    <p className="text-caption text-muted-foreground font-mono">
                      {v.aspect_ratio} · {v.duration_s.toFixed(0)}s · {v.size_mb}MB
                    </p>
                  </td>
                  {selectedAccounts.map((acc) => {
                    const cell = compatibilityMatrix[v.id]?.[acc.id] || {
                      status: "ok",
                    };
                    return (
                      <td key={acc.id} className="p-3 text-center">
                        {cell.status === "ok" && (
                          <span
                            title={cell.message}
                            className="inline-flex items-center justify-center h-6 w-6 rounded-full bg-emerald-500/10 text-semantic-success font-bold border border-emerald-500/20"
                          >
                            ✓
                          </span>
                        )}
                        {cell.status === "warn" && (
                          <span
                            title={cell.message}
                            className="inline-flex items-center justify-center h-6 w-6 rounded-full bg-amber-500/10 text-semantic-warning font-bold border border-amber-500/20"
                          >
                            ⚠
                          </span>
                        )}
                        {cell.status === "error" && (
                          <span
                            title={cell.message}
                            className="inline-flex items-center justify-center h-6 w-6 rounded-full bg-rose-500/10 text-semantic-danger font-bold border border-rose-500/20"
                          >
                            ✕
                          </span>
                        )}
                      </td>
                    );
                  })}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}
