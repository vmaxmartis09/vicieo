export interface CastMemberItem {
  id: string | null;
  name: string;
}

export interface CastView {
  cast: CastMemberItem[];
  lineCount: Record<string, number>;
  unmatched: string[];
}

export const MAX_CAST_CACHE_ENTRIES = 32;

const CAST_CACHE = new Map<string, CastView>();

/**
 * Bounded LRU cache lookup cho dàn diễn viên theo `<projectId>:<episodeId>` (K.4).
 * Giới hạn 32 kịch bản để tránh phình bộ nhớ khi duyệt nhiều tập dài.
 */
export function getCachedCast(key: string): CastView | undefined {
  const hit = CAST_CACHE.get(key);
  if (!hit) return undefined;
  CAST_CACHE.delete(key);
  CAST_CACHE.set(key, hit);
  return hit;
}

export function setCachedCast(key: string, view: CastView): void {
  if (CAST_CACHE.has(key)) {
    CAST_CACHE.delete(key);
  } else if (CAST_CACHE.size >= MAX_CAST_CACHE_ENTRIES) {
    const oldestKey = CAST_CACHE.keys().next().value;
    if (oldestKey !== undefined) {
      CAST_CACHE.delete(oldestKey);
    }
  }
  CAST_CACHE.set(key, view);
}

export function getCastCacheSize(): number {
  return CAST_CACHE.size;
}

export function clearCastCache(): void {
  CAST_CACHE.clear();
}

/** Hàm thuần: trích xuất CastView từ payload GET `/script-data/:episode`. */
export function extractCastView(scriptData: any): CastView {
  const sc = scriptData?.script;
  const counts: Record<string, number> = {};
  for (const sh of scriptData?.shots || []) {
    if (sh.characterId && sh.dialogue) {
      counts[sh.characterId] = (counts[sh.characterId] || 0) + 1;
    }
  }
  return {
    cast: (sc?.cast || []).map((c: any) => ({ id: c.id || null, name: c.name })),
    lineCount: counts,
    unmatched: sc?.unmatched || [],
  };
}

/** Hàm thuần: hợp nhất danh sách diễn viên từ kịch bản với `include`/`exclude`. */
export function resolveCastMembers(
  cast: CastMemberItem[] | null,
  include: string[],
  exclude: string[]
): CastMemberItem[] {
  return [
    ...(cast || []).filter((c) => !(c.id && exclude.includes(c.id))),
    ...include
      .filter((id) => !(cast || []).some((c) => c.id === id))
      .map((id) => ({ id, name: id })),
  ];
}
