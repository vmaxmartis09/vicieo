"use client";

import React, { useEffect, useState } from "react";
import { useStudioStore } from "@/store";
import { intelApi, type Product } from "@/lib/api/intel";
import { CORE_URL } from "@/lib/repositories/core";
import { cn } from "@/lib/utils";

const STATUS_LABEL: Record<string, { text: string; dot: string }> = {
  chua_lam: { text: "Chưa làm video review", dot: "bg-muted-foreground/60" },
  dang_lam: { text: "Đang làm video review", dot: "bg-semantic-warning" },
  da_lam: { text: "Đã làm video review", dot: "bg-semantic-success" },
};

const imgUrl = (projectId: string, rel?: string) =>
  rel ? (/^https?:/.test(rel) ? rel : `${CORE_URL}/projects/${projectId}/files/${rel.replace(/^\/+/, "")}`) : "";

const money = (v?: number | null) =>
  typeof v === "number" && v > 0 ? `${new Intl.NumberFormat("vi-VN").format(v)}₫` : "";

/** Card node "Sản phẩm": chọn sản phẩm của dự án và ảnh đã tách nền (set/biến thể) để thử đồ và dựng video review. */
export const ProductNodeCard = React.memo(function ProductNodeCard({
  nodeId,
  params,
  compact = false,
}: {
  nodeId: string;
  params: Record<string, any>;
  compact?: boolean;
}) {
  const projectId = useStudioStore((s: any) => s.activeProjectId) as string;
  const updateNodeParams = useStudioStore((s: any) => s.updateNodeParams) as (id: string, p: Record<string, any>) => void;
  const [items, setItems] = useState<Product[] | null>(null);
  const [error, setError] = useState("");

  useEffect(() => {
    let alive = true;
    intelApi
      .fetchProducts(projectId)
      .then((l) => alive && setItems(l))
      .catch((e) => alive && (setItems([]), setError(e instanceof Error ? e.message : "Không tải được sản phẩm")));
    return () => {
      alive = false;
    };
  }, [projectId]);

  const productId = String(params.productId || "");
  const product = items?.find((p) => p.id === productId);
  const cuts = product?.cutout_images ?? [];
  const variant = String(params.variant || "");
  const shown = cuts.find((c) => c === variant || c.split("/").pop() === variant.split("/").pop()) ?? cuts[0];
  const st = STATUS_LABEL[product?.review_status ?? "chua_lam"] ?? STATUS_LABEL.chua_lam;

  return (
    <div className="space-y-2">
      {(!compact || !product) && (
        <select
        aria-label="Chọn sản phẩm"
        value={productId}
        onChange={(e) => updateNodeParams(nodeId, { productId: e.target.value, variant: "" })}
        className="nodrag nowheel h-7 w-full rounded-ctl border border-border bg-card px-2 text-footnote"
      >
        <option value="">{items === null ? "Đang tải…" : items.length ? "Chọn sản phẩm…" : "Dự án chưa có sản phẩm"}</option>
        {(items ?? []).map((p) => (
          <option key={p.id} value={p.id}>
            {p.name}
          </option>
        ))}
        </select>
      )}
      {error && <div role="alert" className="text-caption text-semantic-danger">{error}</div>}

      {product ? (
        <>
          <div className="flex gap-2">
            <div className={cn(
              "shrink-0 overflow-hidden rounded-ctl border border-border bg-[conic-gradient(#e5e5e5_25%,#fff_0_50%,#e5e5e5_0_75%,#fff_0)] bg-[length:12px_12px]",
              compact ? "h-20 w-16" : "h-28 w-20"
            )}>
              {shown ? (
                // eslint-disable-next-line @next/next/no-img-element
                <img alt="" loading="lazy" decoding="async" src={imgUrl(projectId, shown)} className="h-full w-full object-contain p-1" />
              ) : (
                <span className="flex h-full items-center justify-center px-1 text-center text-caption text-muted-foreground">Chưa tách nền</span>
              )}
            </div>
            <div className="min-w-0 flex-1 space-y-1">
              <div className={cn("text-footnote font-semibold leading-snug", compact ? "line-clamp-2" : "line-clamp-3")}>{product.name}</div>
              <div className="tnum text-footnote">
                <span className="font-semibold">{money(product.sale_price ?? product.price)}</span>
                {product.original_price ? <span className="ml-2 text-muted-foreground line-through">{money(product.original_price)}</span> : null}
              </div>
              <div className="flex items-center gap-2 text-caption text-muted-foreground">
                <span className={cn("h-2 w-2 rounded-full", st.dot)} />
                {st.text}
              </div>
            </div>
          </div>
          {cuts.length > 1 && !compact && (
            <div className="nodrag nowheel no-scrollbar flex gap-2 overflow-x-auto" role="radiogroup" aria-label="Ảnh sản phẩm dùng cho flow">
              {cuts.map((c) => {
                const on = c === shown;
                return (
                  <button
                    key={c}
                    type="button"
                    role="radio"
                    aria-checked={on}
                    title={c.split("/").pop()}
                    onClick={() => updateNodeParams(nodeId, { variant: c })}
                    className={cn("h-12 w-9 shrink-0 overflow-hidden rounded-ctl border-2 bg-white", on ? "border-foreground" : "border-border opacity-70 hover:opacity-100")}
                  >
                    {/* eslint-disable-next-line @next/next/no-img-element */}
                    <img alt="" loading="lazy" decoding="async" src={imgUrl(projectId, c)} className="h-full w-full object-contain" />
                  </button>
                );
              })}
            </div>
          )}
        </>
      ) : productId ? (
        <div className="text-caption text-semantic-danger">Không thấy sản phẩm “{productId}” trong dự án này.</div>
      ) : null}
    </div>
  );
});
