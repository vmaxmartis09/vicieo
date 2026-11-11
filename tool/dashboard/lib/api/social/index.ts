export * from "./types";
export * from "./client";

import { videosEndpoints } from "./endpoints/videos";
import { accountsEndpoints } from "./endpoints/accounts";
import { postsEndpoints } from "./endpoints/posts";
import { calendarEndpoints } from "./endpoints/calendar";
import { publishingEndpoints } from "./endpoints/publishing";
import { analyticsEndpoints } from "./endpoints/analytics";
import { automationsEndpoints } from "./endpoints/automations";
import { hubEndpoints } from "./endpoints/hub";

export const socialApi = {
  ...videosEndpoints,
  ...accountsEndpoints,
  ...postsEndpoints,
  ...calendarEndpoints,
  ...publishingEndpoints,
  ...analyticsEndpoints,
  ...automationsEndpoints,
  ...hubEndpoints,
};
