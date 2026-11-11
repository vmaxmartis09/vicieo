import { socialApi } from "@/lib/api/social";
import {
  applyTargetTimeToCalendar,
  applyTargetTimeToPosts,
  findTargetScheduledTime,
} from "../utils/socialHelpers";
import type { SocialState, SocialStoreGet, SocialStoreSet } from "../types";

export function createSchedulingActions(
  set: SocialStoreSet,
  get: SocialStoreGet
): Pick<SocialState, "moveTargetTime" | "undoLastDrag"> {
  return {
    moveTargetTime: async (projectId, postId, targetId, newTime) => {
      const { calendarData, posts } = get();
      const previousTime = findTargetScheduledTime(calendarData, targetId);

      if (calendarData) {
        set({ calendarData: applyTargetTimeToCalendar(calendarData, targetId, newTime) });
      }

      set({
        posts: applyTargetTimeToPosts(posts, postId, targetId, newTime),
        lastDragAction: { postId, targetId, previousTime },
      });

      try {
        await socialApi.updateTargetTime(projectId, postId, targetId, newTime);
      } catch (e: any) {
        if (previousTime) {
          get().undoLastDrag(projectId);
        }
        throw e;
      }
    },

    undoLastDrag: async (projectId) => {
      const { lastDragAction } = get();
      if (!lastDragAction || !lastDragAction.previousTime) return;
      const { postId, targetId, previousTime } = lastDragAction;
      set({ lastDragAction: null });
      await socialApi.updateTargetTime(projectId, postId, targetId, previousTime);
      await get().loadCalendar(projectId);
    },
  };
}
