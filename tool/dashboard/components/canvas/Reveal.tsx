import React, { useEffect, useState } from "react";
import { cn } from "@/lib/utils";

/** Hiện/ẩn mượt bằng grid-rows 0fr↔1fr (không cần biết trước chiều cao); mở tràn sau khi bung xong để dropdown không bị cắt. */
export function Reveal({ show, children }: { show: boolean; children: React.ReactNode }) {
  const [settled, setSettled] = useState(show);

  useEffect(() => {
    if (!show) {
      setSettled(false);
      return;
    }
    const timer = setTimeout(() => setSettled(true), 220);
    return () => clearTimeout(timer);
  }, [show]);

  return (
    <div
      className={cn(
        "grid w-full min-w-0 grid-cols-1 transition-[grid-template-rows,opacity] duration-200 ease-out",
        show ? "grid-rows-[1fr] opacity-100" : "grid-rows-[0fr] opacity-0 pointer-events-none"
      )}
      aria-hidden={!show}
    >
      <div className={cn("min-h-0 min-w-0 w-full", settled && show ? "overflow-visible" : "overflow-hidden")}>
        {children}
      </div>
    </div>
  );
}
