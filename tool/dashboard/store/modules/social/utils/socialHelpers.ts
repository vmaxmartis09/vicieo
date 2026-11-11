import type { CalendarResponse, Post, SocialNotification } from "@/lib/api/social";

export function countUnreadNotifications(notifs: SocialNotification[]): number {
  return notifs.filter((n) => !n.read).length;
}

export function findTargetScheduledTime(
  calendarData: CalendarResponse | null,
  targetId: string
): string | null {
  if (!calendarData) return null;
  const item = calendarData.items.find((i) => i.target_id === targetId);
  return item ? item.scheduled_at || null : null;
}

export function applyTargetTimeToCalendar(
  calendarData: CalendarResponse,
  targetId: string,
  newTime: string
): CalendarResponse {
  const nextItems = calendarData.items.map((i) =>
    i.target_id === targetId ? { ...i, scheduled_at: newTime } : i
  );
  return { ...calendarData, items: nextItems };
}

export function applyTargetTimeToPosts(
  posts: Post[],
  postId: string,
  targetId: string,
  newTime: string
): Post[] {
  return posts.map((p) => {
    if (p.id !== postId) return p;
    return {
      ...p,
      targets: p.targets.map((t) =>
        t.id === targetId ? { ...t, scheduled_at: newTime } : t
      ),
    };
  });
}
