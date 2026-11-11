export const SOCIAL_CORE_URL =
  process.env.NEXT_PUBLIC_SOCIAL_URL ||
  process.env.NEXT_PUBLIC_CORE_URL ||
  "http://127.0.0.1:8103";

export async function fetchJson<T>(url: string, options?: RequestInit): Promise<T> {
  const res = await fetch(url, {
    ...options,
    headers: {
      "Content-Type": "application/json",
      ...(options?.headers || {}),
    },
  });
  if (!res.ok) {
    const errorText = await res.text();
    throw new Error(`API Error [${res.status}]: ${errorText}`);
  }
  return res.json();
}
