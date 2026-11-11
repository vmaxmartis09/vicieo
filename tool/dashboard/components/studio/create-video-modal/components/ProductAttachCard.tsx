"use client";

import React, { type Dispatch, type SetStateAction } from "react";
import type { ProductFormInfo } from "../types";

export const ProductAttachCard = React.memo(function ProductAttachCard({
  productInfo,
  setProductInfo,
}: {
  productInfo: ProductFormInfo;
  setProductInfo: Dispatch<SetStateAction<ProductFormInfo>>;
}) {
  return (
    <div className="space-y-2 rounded-card border border-border bg-accent/20 p-3">
      <span className="text-footnote font-semibold text-foreground">
        🛍️ Thông tin sản phẩm gắn kèm (nhập trực tiếp):
      </span>
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-2">
        <input
          type="text"
          placeholder="Tên sản phẩm..."
          value={productInfo.title}
          onChange={(e) =>
            setProductInfo((prev) => ({
              ...prev,
              title: e.target.value,
              name: e.target.value,
            }))
          }
          className="rounded-ctl border border-border bg-background px-3 py-2 text-footnote text-foreground outline-none focus:border-primary"
        />
        <input
          type="text"
          placeholder="Giá sản phẩm (vd: 199.000đ)..."
          value={productInfo.price}
          onChange={(e) =>
            setProductInfo((prev) => ({ ...prev, price: e.target.value }))
          }
          className="rounded-ctl border border-border bg-background px-3 py-2 text-footnote text-foreground outline-none focus:border-primary"
        />
        <input
          type="text"
          placeholder="Điểm nổi bật / USP..."
          value={productInfo.usp}
          onChange={(e) =>
            setProductInfo((prev) => ({ ...prev, usp: e.target.value }))
          }
          className="rounded-ctl border border-border bg-background px-3 py-2 text-footnote text-foreground outline-none focus:border-primary"
        />
        <input
          type="text"
          placeholder="URL ảnh sản phẩm (tuỳ chọn)..."
          value={productInfo.image}
          onChange={(e) =>
            setProductInfo((prev) => ({ ...prev, image: e.target.value }))
          }
          className="rounded-ctl border border-border bg-background px-3 py-2 text-footnote text-foreground outline-none focus:border-primary"
        />
      </div>
    </div>
  );
});
