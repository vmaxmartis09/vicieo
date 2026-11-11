import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";
import path from "path";

export default defineConfig({
  plugins: [react()],
  test: {
    environment: "jsdom",
    globals: true,
    setupFiles: ["./test/setup.ts"],
    include: ["test/**/*.test.{ts,tsx}"],
    exclude: ["e2e/**", "node_modules/**", ".next/**", ".next-build*/**"],
  },
  resolve: {
    alias: {
      "@": path.resolve(process.cwd(), "./"),
    },
  },
});
