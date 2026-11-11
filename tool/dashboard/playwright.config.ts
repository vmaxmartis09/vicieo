import fs from "node:fs";
import { defineConfig, devices, chromium } from "@playwright/test";

const PORT = process.env.PLAYWRIGHT_PORT || "3019";
const BASE_URL = process.env.PLAYWRIGHT_BASE_URL || `http://127.0.0.1:${PORT}`;
const MAC_CHROME = "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";

function resolveExecutablePath(): string | undefined {
  const bundled = chromium.executablePath();
  if (fs.existsSync(bundled)) return bundled;
  if (fs.existsSync(MAC_CHROME)) return MAC_CHROME;
  return undefined;
}

const executablePath = resolveExecutablePath();

export default defineConfig({
  testDir: "./e2e",
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: 0,
  workers: 1,
  reporter: "list",
  timeout: 45_000,
  use: {
    baseURL: BASE_URL,
    viewport: { width: 1440, height: 900 },
    deviceScaleFactor: 1,
    reducedMotion: "reduce",
    locale: "vi-VN",
    timezoneId: "Asia/Ho_Chi_Minh",
    trace: "retain-on-failure",
    screenshot: "only-on-failure",
    ...(executablePath ? { launchOptions: { executablePath } } : {}),
  },
  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] },
    },
  ],
});
