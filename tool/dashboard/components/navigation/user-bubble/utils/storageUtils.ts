export function getInitials(name?: string): string {
  return (name || "S")
    .split(" ")
    .map((w) => w.charAt(0))
    .join("")
    .slice(0, 2)
    .toUpperCase();
}

export function estimateLocalStorageBytes(): number {
  if (typeof window === "undefined") return 48120;
  try {
    let total = 0;
    for (let i = 0; i < window.localStorage.length; i++) {
      const k = window.localStorage.key(i) || "";
      const v = window.localStorage.getItem(k) || "";
      total += (k.length + v.length) * 2;
    }
    return Math.max(total, 48120);
  } catch {
    return 48120;
  }
}

export function formatStorageKb(bytes: number): string {
  return `${(bytes / 1024).toFixed(1)} KB`;
}
