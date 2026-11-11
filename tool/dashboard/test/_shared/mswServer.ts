import { setupServer } from "msw/node";
import { http, HttpResponse } from "msw";

export const defaultHandlers = [
  http.get("*/core/projects", () => {
    return HttpResponse.json([
      { id: "soc-nho-tv", name: "Sóc Nhỏ TV" },
    ]);
  }),
  http.get("*/core/nodes", () => {
    return HttpResponse.json([]);
  }),
];

export const server = setupServer(...defaultHandlers);
