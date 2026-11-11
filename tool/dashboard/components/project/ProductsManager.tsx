"use client";

import React, { useCallback, useEffect, useState } from "react";
import { intelApi, type Product, type ReviewStatus } from "@/lib/api/intel";
import { CORE_URL } from "@/lib/repositories/core";
import { Button } from "@/components/ui/Button";
import { AlertCircle, Check, CheckCircle2, ExternalLink, Film, Sparkles, Tag, Trash2 } from "@/components/ui/icons";
import { confirm } from "@/lib/confirm";
import { cn } from "@/lib/utils";

const STATUS: { key: ReviewStatus; label: string; short: string; dot: string; chip: string }[] = [
  { key: "chua_lam", label: "Chưa làm video review", short: "Chưa review", dot: "bg-muted-foreground/60", chip: "bg-muted text-muted-foreground" },
  { key: "dang_lam", label: "Đang làm video review", short: "Đang làm", dot: "bg-semantic-warning", chip: "bg-semantic-warning/15 text-semantic-warning" },
  { key: "da_lam", label: "Đã làm video review", short: "Đã review", dot: "bg-semantic-success", chip: "bg-semantic-success/15 text-semantic-success" },
];
const statusOf = (p: Product) => STATUS.find((s) => s.key === (p.review_status ?? "chua_lam")) ?? STATUS[0];

const money = (v?: number | null, cur = "VND") =>
  typeof v === "number" && v > 0 ? `${new Intl.NumberFormat("vi-VN").format(v)}${cur === "VND" ? "₫" : ` ${cur}`}` : "—";

function fileUrl(projectId: string, path?: string | null): string {
  if (!path) return "";
  if (/^(https?:|data:|blob:)/.test(path)) return path;
  return `${CORE_URL}/projects/${projectId}/files/${path.replace(/^\/+/, "")}`;
}

/** Ảnh sản phẩm: cutout (nền trong suốt) ưu tiên, sau đó ảnh gốc. */
function productImages(p: Product): { cut: string[]; raw: string[] } {
  return { cut: p.cutout_images ?? [], raw: p.raw_images ?? [] };
}

function StatusChip({ p }: { p: Product }) {
  const s = statusOf(p);
  return (
    <span className={cn("inline-flex items-center gap-1 rounded-full px-2 py-1 text-caption font-medium", s.chip)}>
      <span className={cn("h-2 w-2 rounded-full", s.dot)} />
      {s.short}
    </span>
  );
}

function ProductPoster({ p, projectId, active, onClick }: { p: Product; projectId: string; active: boolean; onClick: () => void }) {
  const img = productImages(p).cut[0] ?? p.images?.[0];
  return (
    <button
      type="button"
      aria-current={active}
      onClick={onClick}
      title={p.name}
      className={cn(
        "group relative block h-[168px] w-[124px] shrink-0 overflow-hidden rounded-panel border-2 bg-white text-left transition-all active:scale-[0.98]",
        active ? "border-foreground" : "border-border opacity-85 hover:opacity-100"
      )}
    >
      {img ? (
        // eslint-disable-next-line @next/next/no-img-element
        <img decoding="async" loading="lazy" alt="" src={fileUrl(projectId, img)} className="absolute inset-0 h-full w-full object-contain p-2 transition-transform duration-300 group-hover:scale-105" />
      ) : (
        <span className="absolute inset-0 flex items-center justify-center text-4xl text-muted-foreground/50">
          <Tag className="h-8 w-8" />
        </span>
      )}
      <span className="absolute left-2 top-2"><StatusChip p={p} /></span>
      <span className="absolute inset-x-0 bottom-0 bg-gradient-to-t from-black/75 via-black/35 to-transparent px-3 pb-3 pt-10 text-white">
        <span className="line-clamp-2 text-footnote font-semibold leading-tight">{p.name}</span>
        <span className="block truncate text-caption text-white/75 tnum">{money(p.sale_price ?? p.price, p.currency)}</span>
      </span>
    </button>
  );
}

function Row({ k, children }: { k: string; children: React.ReactNode }) {
  if (children === null || children === undefined || children === "" || children === "—") return null;
  return (
    <div className="grid grid-cols-[104px_1fr] gap-2 py-1 text-callout">
      <dt className="text-muted-foreground">{k}</dt>
      <dd className="min-w-0 break-words">{children}</dd>
    </div>
  );
}

function ProductDetail({ p, projectId, onChange, onDelete }: { p: Product; projectId: string; onChange: (n: Product) => Promise<void>; onDelete: () => void }) {
  const { cut, raw } = productImages(p);
  const [open, setOpen] = useState(false);
  const [notes, setNotes] = useState(p.notes ?? "");
  const [saving, setSaving] = useState(false);
  const [styles, setStyles] = useState<{ styleId: string; name?: string; archetype?: string }[]>([]);
  const [selectedStyle, setSelectedStyle] = useState<string>("koc_unboxing_tryon");
  const [creatingFlow, setCreatingFlow] = useState(false);
  const [flowSuccess, setFlowSuccess] = useState<string | null>(null);
  const [flowError, setFlowError] = useState<string | null>(null);

  useEffect(() => setNotes(p.notes ?? ""), [p.id, p.notes]);
  const cur = p.currency ?? "VND";

  useEffect(() => {
    let active = true;
    const fetchStyles = async () => {
      try {
        const res = await fetch(`${CORE_URL}/data/projects/${projectId}/koc-styles`);
        if (res.ok) {
          const list = await res.json();
          if (active && Array.isArray(list) && list.length > 0) {
            setStyles(list);
            setSelectedStyle(list[0].styleId);
            return;
          }
        }
      } catch {
        // dùng fallback
      }
      if (active) {
        const defs = [
          { styleId: "koc_unboxing_tryon", name: "Đập hộp & Thử đồ (Unboxing & Try-on)", archetype: "unboxing_tryon" },
          { styleId: "koc_problem_solution", name: "Giải pháp phối đồ (Problem & Solution)", archetype: "problem_solution" },
          { styleId: "koc_outfit_check", name: "Biến hình & Đánh giá (Outfit Check)", archetype: "outfit_check" },
        ];
        setStyles(defs);
        setSelectedStyle(defs[0].styleId);
      }
    };
    void fetchStyles();
    return () => {
      active = false;
    };
  }, [projectId]);

  const handleCreateFlow = async () => {
    if (!selectedStyle) return;
    setCreatingFlow(true);
    setFlowSuccess(null);
    setFlowError(null);
    try {
      const res = await fetch(`${CORE_URL}/data/projects/${projectId}/products/${p.id}/review-flow`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ style_id: selectedStyle }),
      });
      const data = await res.json();
      if (!res.ok) {
        throw new Error(data.detail || data.message || "Tạo flow thất bại");
      }
      setFlowSuccess(data.file || `graph.review-${selectedStyle}.json`);
    } catch (err: any) {
      setFlowError(err?.message || "Có lỗi khi tạo flow");
    } finally {
      setCreatingFlow(false);
    }
  };

  const setStatus = async (key: ReviewStatus) => {
    setSaving(true);
    try {
      await onChange({ ...p, review_status: key, reviewed_at: key === "da_lam" ? new Date().toISOString() : "" });
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="space-y-4 rounded-2xl border border-border bg-card p-3">
      <div>
        <div className="flex items-start justify-between gap-2">
          <h3 className="text-base font-semibold leading-snug">{p.name}</h3>
          <Button variant="ghost" size="icon" aria-label="Xoá sản phẩm" onClick={onDelete}><Trash2 className="h-4 w-4" /></Button>
        </div>
        <div className="mt-1 flex flex-wrap items-baseline gap-2 tnum">
          <span className="text-title font-semibold">{money(p.sale_price ?? p.price, cur)}</span>
          {p.original_price ? <span className="text-callout text-muted-foreground line-through">{money(p.original_price, cur)}</span> : null}
          {p.discount ? <span className="rounded-xs bg-semantic-danger/15 px-2 text-footnote font-medium text-semantic-danger">{p.discount}</span> : null}
        </div>
      </div>

      {/* Tạo flow review theo phong cách KOC */}
      <section aria-label="Tạo flow theo phong cách" className="space-y-3 rounded-card border border-border bg-muted/40 p-3">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2 text-footnote font-semibold text-foreground">
            <Sparkles className="h-4 w-4 text-semantic-warning" />
            <span>Tạo flow review theo phong cách</span>
          </div>
          {styles.length > 0 && (
            <span className="text-caption text-muted-foreground tnum">{styles.length} phong cách</span>
          )}
        </div>

        <div className="space-y-1">
          <label htmlFor={`style-select-${p.id}`} className="text-footnote text-muted-foreground">
            Phong cách KOC:
          </label>
          <select
            id={`style-select-${p.id}`}
            value={selectedStyle}
            onChange={(e) => setSelectedStyle(e.target.value)}
            disabled={creatingFlow}
            className="w-full rounded-ctl border border-border bg-background px-3 py-1 text-footnote font-medium text-foreground focus:outline-none focus:ring-1 focus:ring-foreground"
          >
            {styles.map((st) => (
              <option key={st.styleId} value={st.styleId}>
                {st.name || st.styleId} ({st.archetype || "review"})
              </option>
            ))}
          </select>
        </div>

        <Button
          type="button"
          size="sm"
          className="w-full gap-2 font-medium"
          disabled={creatingFlow || !selectedStyle}
          onClick={handleCreateFlow}
        >
          <Film className="h-3.5 w-3.5" />
          {creatingFlow ? "Đang tạo flow…" : "Tạo flow theo phong cách"}
        </Button>

        {flowSuccess && (
          <div role="status" className="flex items-center gap-2 rounded-ctl border border-semantic-success/40 bg-semantic-success/10 px-3 py-1 text-footnote text-semantic-success">
            <CheckCircle2 className="h-3.5 w-3.5 shrink-0" />
            <span>Đã tạo flow: <code className="font-semibold">{flowSuccess}</code></span>
          </div>
        )}

        {flowError && (
          <div role="alert" className="flex items-center gap-2 rounded-ctl border border-semantic-danger/40 bg-semantic-danger/10 px-3 py-1 text-footnote text-semantic-danger">
            <AlertCircle className="h-3.5 w-3.5 shrink-0" />
            <span>{flowError}</span>
          </div>
        )}
      </section>

      <section aria-label="Trạng thái video review" className="space-y-2">
        <div className="text-footnote font-medium text-muted-foreground">Video review</div>
        <div role="radiogroup" className="grid grid-cols-3 gap-1 rounded-card bg-muted p-1">
          {STATUS.map((s) => {
            const on = (p.review_status ?? "chua_lam") === s.key;
            return (
              <button
                key={s.key}
                type="button"
                role="radio"
                aria-checked={on}
                disabled={saving}
                onClick={() => !on && setStatus(s.key)}
                className={cn("flex items-center justify-center gap-1 rounded-ctl px-2 py-1 text-footnote transition-colors", on ? "bg-card font-semibold shadow-sm" : "text-muted-foreground hover:text-foreground")}
              >
                {on && <Check className="h-3 w-3" />}
                {s.short}
              </button>
            );
          })}
        </div>
        {p.reviewed_at && (p.review_status ?? "") === "da_lam" ? (
          <div className="text-caption text-muted-foreground tnum">Đánh dấu lúc {new Date(p.reviewed_at).toLocaleString("vi-VN")}</div>
        ) : null}
      </section>

      {cut.length > 0 && (
        <section aria-label="Ảnh đã tách nền" className="space-y-2">
          <div className="text-footnote font-medium text-muted-foreground">Đã tách nền ({cut.length})</div>
          <div className="grid grid-cols-3 gap-2">
            {cut.map((c) => (
              <a key={c} href={fileUrl(projectId, c)} target="_blank" rel="noreferrer" className="block aspect-square overflow-hidden rounded-card border border-border bg-[conic-gradient(#e5e5e5_25%,#fff_0_50%,#e5e5e5_0_75%,#fff_0)] bg-[length:14px_14px]">
                {/* eslint-disable-next-line @next/next/no-img-element */}
                <img loading="lazy" decoding="async" alt="" src={fileUrl(projectId, c)} className="h-full w-full object-contain p-1" />
              </a>
            ))}
          </div>
        </section>
      )}

      {raw.length > 0 && (
        <section aria-label="Ảnh gốc" className="space-y-2">
          <div className="text-footnote font-medium text-muted-foreground">Ảnh gốc từ trang bán ({raw.length})</div>
          <div className="no-scrollbar flex gap-2 overflow-x-auto">
            {raw.map((r) => (
              <a key={r} href={fileUrl(projectId, r)} target="_blank" rel="noreferrer" className="block h-20 w-20 shrink-0 overflow-hidden rounded-card border border-border">
                {/* eslint-disable-next-line @next/next/no-img-element */}
                <img loading="lazy" decoding="async" alt="" src={fileUrl(projectId, r)} className="h-full w-full object-cover" />
              </a>
            ))}
          </div>
        </section>
      )}

      <dl className="divide-y divide-border">
        <Row k="Shop">{p.shop_name}</Row>
        <Row k="Danh mục">{p.category && p.category !== "general" ? p.category : ""}</Row>
        <Row k="Thương hiệu">{p.brand}</Row>
        <Row k="Đã bán">{p.sold_count ? new Intl.NumberFormat("vi-VN").format(p.sold_count) : ""}</Row>
        <Row k="Đánh giá">{p.rating ? `${p.rating} ★ (${p.review_count ?? 0} đánh giá)` : ""}</Row>
        <Row k="Màu sắc">{p.colors?.length ? p.colors.map((c) => c.name).join(" · ") : ""}</Row>
        <Row k="Kích cỡ">{p.sizes?.length ? p.sizes.join(" · ") : ""}</Row>
        <Row k="Biến thể">{p.variants?.length ? `${p.variants.length} SKU, còn hàng ${p.variants.reduce((a, v) => a + (v.stock ?? 0), 0)}` : ""}</Row>
        <Row k="Đối tượng">{p.target_audience}</Row>
        <Row k="Điểm bán">{p.usp?.length ? (<ul className="list-disc pl-4">{p.usp.map((u, i) => <li key={i}>{u}</li>)}</ul>) : ""}</Row>
        <Row k="Thông số">
          {p.specs && Object.keys(p.specs).length ? (
            <ul className="space-y-1">{Object.entries(p.specs).map(([k, v]) => <li key={k}><span className="text-muted-foreground">{k}:</span> {String(v)}</li>)}</ul>
          ) : ""}
        </Row>
        <Row k="Link nguồn">
          {p.source_url ? (
            <a href={p.source_url} target="_blank" rel="noreferrer" className="inline-flex items-center gap-1 text-semantic-info underline-offset-2 hover:underline">
              TikTok Shop <ExternalLink className="h-3 w-3" />
            </a>
          ) : ""}
        </Row>
        <Row k="Lưu ý">{p.compliance_notes}</Row>
      </dl>

      {p.description ? (
        <section aria-label="Mô tả" className="space-y-1">
          <div className="text-footnote font-medium text-muted-foreground">Mô tả sản phẩm</div>
          <p className={cn("whitespace-pre-line text-callout leading-relaxed", !open && "line-clamp-6")}>{p.description}</p>
          <button type="button" onClick={() => setOpen((v) => !v)} className="text-footnote text-muted-foreground underline-offset-2 hover:underline">
            {open ? "Thu gọn" : "Xem thêm"}
          </button>
        </section>
      ) : null}

      <section aria-label="Ghi chú" className="space-y-1">
        <label htmlFor={`pn-${p.id}`} className="text-footnote font-medium text-muted-foreground">Ghi chú review</label>
        <textarea
          id={`pn-${p.id}`}
          value={notes}
          onChange={(e) => setNotes(e.target.value)}
          onBlur={() => notes !== (p.notes ?? "") && onChange({ ...p, notes })}
          rows={2}
          className="w-full resize-y rounded-ctl border border-border bg-background px-2 py-1 text-callout"
          placeholder="Góc quay, điểm cần nhấn, lỗi cần tránh..."
        />
      </section>
    </div>
  );
}

/** Dán link TikTok Shop: Core thu thập thông tin + ảnh, tách nền ảnh biến thể rồi lưu vào dự án. */
function AddFromLink({ projectId, onDone }: { projectId: string; onDone: (p: Product) => void }) {
  const [url, setUrl] = useState("");
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState("");
  const [warn, setWarn] = useState<string[]>([]);

  const submit = async () => {
    if (!url.trim() || busy) return;
    setBusy(true);
    setErr("");
    setWarn([]);
    try {
      const r = await intelApi.collectProductFromUrl(projectId, url.trim());
      setWarn(r.warnings);
      setUrl("");
      onDone(r.product);
    } catch (e) {
      setErr(e instanceof Error ? e.message : "Không lấy được sản phẩm");
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="space-y-2">
      <form
        onSubmit={(e) => {
          e.preventDefault();
          void submit();
        }}
        className="flex gap-2"
      >
        <input
          value={url}
          onChange={(e) => setUrl(e.target.value)}
          disabled={busy}
          aria-label="Link sản phẩm TikTok Shop"
          placeholder="Dán link sản phẩm TikTok Shop…"
          className="min-w-0 flex-1 rounded-ctl border border-border bg-background px-3 py-1 text-callout placeholder:text-muted-foreground/60"
        />
        <Button type="submit" size="md" disabled={busy || !url.trim()}>
          {busy ? "Đang lấy…" : "Thêm từ link"}
        </Button>
      </form>
      {busy && <div className="text-caption text-muted-foreground">Đang tải thông tin, ảnh và tách nền (thường 20–60 giây, lần đầu lâu hơn do tải mô hình tách nền).</div>}
      {err && <div role="alert" className="rounded-ctl border border-semantic-danger/40 bg-semantic-danger/10 px-3 py-1 text-footnote text-semantic-danger">{err}</div>}
      {warn.length > 0 && (
        <ul className="space-y-1 rounded-ctl border border-semantic-warning/40 bg-semantic-warning/10 px-3 py-1 text-footnote text-semantic-warning">
          {warn.map((w, i) => <li key={i}>{w}</li>)}
        </ul>
      )}
    </div>
  );
}

export function ProductsManager({ projectId, compact = false, onCount }: { projectId: string; compact?: boolean; onCount?: (n: number) => void }) {
  const [items, setItems] = useState<Product[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [sel, setSel] = useState<string | null>(null);
  const [filter, setFilter] = useState<"all" | ReviewStatus>("all");

  const load = useCallback(async () => {
    try {
      const l = await intelApi.fetchProducts(projectId);
      setItems(l);
      setError(null);
      onCount?.(l.length);
      setSel((cur) => (cur && l.some((x) => x.id === cur) ? cur : l[0]?.id ?? null));
    } catch (e) {
      setError(e instanceof Error ? e.message : "Không tải được sản phẩm");
      setItems([]);
    }
  }, [projectId, onCount]);

  useEffect(() => {
    setItems(null);
    void load();
  }, [load]);

  const save = async (n: Product) => {
    const saved = await intelApi.updateProduct(projectId, n.id, n);
    setItems((cur) => (cur ?? []).map((x) => (x.id === saved.id ? saved : x)));
  };
  const remove = async (p: Product) => {
    const ok = await confirm({
      tone: "danger",
      title: `Xoá "${p.name}"?`,
      message: "Sản phẩm được chuyển vào thùng rác của dự án.",
    });
    if (!ok) return;
    await intelApi.deleteProduct(projectId, p.id);
    await load();
  };

  const shown = (items ?? []).filter((p) => filter === "all" || (p.review_status ?? "chua_lam") === filter);
  const current = (items ?? []).find((p) => p.id === sel) ?? null;
  const counts = (k: ReviewStatus) => (items ?? []).filter((p) => (p.review_status ?? "chua_lam") === k).length;

  return (
    <div className={cn("space-y-3", !compact && "p-1")}>
      <AddFromLink projectId={projectId} onDone={(p) => { setSel(p.id); void load(); }} />
      <div className="flex flex-wrap items-center gap-2" role="tablist" aria-label="Lọc theo trạng thái">
        {([{ key: "all", label: `Tất cả ${items?.length ?? 0}` }, ...STATUS.map((s) => ({ key: s.key, label: `${s.short} ${counts(s.key)}` }))] as { key: "all" | ReviewStatus; label: string }[]).map((f) => (
          <button
            key={f.key}
            type="button"
            role="tab"
            aria-selected={filter === f.key}
            onClick={() => setFilter(f.key)}
            className={cn("rounded-full border px-3 py-1 text-footnote tnum transition-colors", filter === f.key ? "border-foreground bg-foreground text-background" : "border-border text-muted-foreground hover:text-foreground")}
          >
            {f.label}
          </button>
        ))}
      </div>

      {error && <div role="alert" className="rounded-ctl border border-semantic-danger/40 bg-semantic-danger/10 px-3 py-2 text-footnote text-semantic-danger">{error}</div>}
      {items === null && <div className="text-callout text-muted-foreground">Đang tải sản phẩm…</div>}
      {items !== null && items.length === 0 && !error && (
        <div className="rounded-panel border border-dashed border-border p-6 text-center text-callout text-muted-foreground">Dự án chưa có sản phẩm. Thêm sản phẩm bằng cách thu thập từ link TikTok Shop.</div>
      )}

      {shown.length > 0 && (
        <div className="no-scrollbar flex gap-3 overflow-x-auto pb-1">
          {shown.map((p) => (
            <ProductPoster key={p.id} p={p} projectId={projectId} active={p.id === sel} onClick={() => setSel(p.id)} />
          ))}
        </div>
      )}
      {items !== null && items.length > 0 && shown.length === 0 && <div className="text-callout text-muted-foreground">Không có sản phẩm ở trạng thái này.</div>}

      {current && <ProductDetail p={current} projectId={projectId} onChange={save} onDelete={() => remove(current)} />}
    </div>
  );
}