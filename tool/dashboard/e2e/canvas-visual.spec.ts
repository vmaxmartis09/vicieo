import fs from "node:fs";
import path from "node:path";
import { test, expect, type Browser, type Page } from "@playwright/test";
import { installCoreMocks } from "./mockRoutes";

const SNAPSHOT_DIR = path.resolve(__dirname, "__screenshots__");

const FREEZE_CSS = `
  *, *::before, *::after {
    transition: none !important;
    animation: none !important;
    caret-color: transparent !important;
  }
`;

async function settleUi(page: Page) {
  await page.addStyleTag({ content: FREEZE_CSS });
  // Wait >350ms so Framer Motion JS tweens (not paused by CSS animation:none) reach resting state
  await page.waitForTimeout(450);
  await page.evaluate(async () => {
    await document.fonts.ready;
    await new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)));
  });
}

async function diffPngBuffers(
  page: Page,
  bufA: Buffer,
  bufB: Buffer
): Promise<{
  identicalBytes: boolean;
  width: number;
  height: number;
  diffPixels: number;
  totalPixels: number;
  diffPct: number;
}> {
  if (bufA.equals(bufB)) {
    return {
      identicalBytes: true,
      width: 1440,
      height: 900,
      diffPixels: 0,
      totalPixels: 1440 * 900,
      diffPct: 0,
    };
  }
  return page.evaluate(
    async ([a64, b64]) => {
      const load = async (src: string) => {
        const res = await fetch(`data:image/png;base64,${src}`);
        return createImageBitmap(await res.blob());
      };
      const [imgA, imgB] = await Promise.all([load(a64), load(b64)]);
      const w = Math.max(imgA.width, imgB.width);
      const h = Math.max(imgA.height, imgB.height);
      const read = (img: ImageBitmap) => {
        const c = document.createElement("canvas");
        c.width = w;
        c.height = h;
        const ctx = c.getContext("2d")!;
        ctx.drawImage(img, 0, 0);
        return ctx.getImageData(0, 0, w, h).data;
      };
      const da = read(imgA);
      const db = read(imgB);
      let diffPixels = 0;
      for (let i = 0; i < da.length; i += 4) {
        if (
          da[i] !== db[i] ||
          da[i + 1] !== db[i + 1] ||
          da[i + 2] !== db[i + 2] ||
          da[i + 3] !== db[i + 3]
        ) {
          diffPixels++;
        }
      }
      const totalPixels = w * h;
      return {
        identicalBytes: false,
        width: w,
        height: h,
        diffPixels,
        totalPixels,
        diffPct: Number(((diffPixels / totalPixels) * 100).toFixed(4)),
      };
    },
    [bufA.toString("base64"), bufB.toString("base64")]
  );
}

async function capturePass(browser: Browser, baseURL: string): Promise<Record<string, Buffer>> {
  const ctx = await browser.newContext({
    viewport: { width: 1440, height: 900 },
    deviceScaleFactor: 1,
    reducedMotion: "reduce",
    locale: "vi-VN",
    timezoneId: "Asia/Ho_Chi_Minh",
  });
  const page = await ctx.newPage();
  await installCoreMocks(page);

  await page.goto(`${baseURL}/studio`, { waitUntil: "networkidle" });
  await expect(page.locator(".react-flow__node")).toHaveCount(5, { timeout: 15_000 });
  await settleUi(page);
  const canvasShot = await page.screenshot({ animations: "disabled" });

  // Open CreateVideoModal and capture steps 1, 2, 3
  await page.getByRole("button", { name: "Tạo video", exact: true }).first().click();
  await expect(page.getByText("Review Sản Phẩm Sóc Nhỏ")).toBeVisible({ timeout: 10_000 });
  await settleUi(page);
  const createStep1 = await page.screenshot({ animations: "disabled" });

  await page.getByRole("button", { name: /Tiếp tục/i }).click();
  await expect(page.getByRole("button", { name: /Quay lại/i })).toBeVisible({ timeout: 10_000 });
  await settleUi(page);
  const createStep2 = await page.screenshot({ animations: "disabled" });

  await page.getByRole("button", { name: /Tiếp tục/i }).click();
  await expect(page.getByRole("button", { name: /Tạo video ngay/i })).toBeVisible({ timeout: 10_000 });
  await settleUi(page);
  const createStep3 = await page.screenshot({ animations: "disabled" });

  // Navigate to SocialHub deep-link and open PublishCenterModal
  await page.goto(`${baseURL}/studio?hub=social`, { waitUntil: "networkidle" });
  await expect(page.getByRole("dialog", { name: "Trung tâm Quản lý Social" })).toBeVisible({ timeout: 10_000 });
  await page.getByRole("button", { name: /Lên kế hoạch đăng/i }).first().click();
  await expect(page.getByText("Video Fixture 9:16")).toBeVisible({ timeout: 10_000 });
  await settleUi(page);
  const publishModal = await page.screenshot({ animations: "disabled" });

  await ctx.close();

  return {
    "studio-canvas-5nodes": canvasShot,
    "create-video-modal-step1": createStep1,
    "create-video-modal-step2": createStep2,
    "create-video-modal-step3": createStep3,
    "publish-center-modal": publishModal,
  };
}

test.describe("Dashboard Visual Characterization & Determinism (D.1.4 / K.2)", () => {
  test.skip(({ isMobile }) => isMobile, "Visual characterization runs on desktop 1440x900 only");

  test("captures 2 consecutive runs of 5-node canvas, CreateVideoModal (3 steps), and PublishCenterModal within <= 0.2% diff", async ({
    browser,
    page,
    baseURL,
  }) => {
    const targetBase = baseURL || "http://127.0.0.1:3019";
    const run1 = await capturePass(browser, targetBase);
    const run2 = await capturePass(browser, targetBase);

    fs.mkdirSync(SNAPSHOT_DIR, { recursive: true });
    await page.setContent("<html><body>diff-worker</body></html>");

    const updateBaselines = process.env.UPDATE_BASELINES === "1";
    const metrics: Record<
      string,
      {
        runToRun: Awaited<ReturnType<typeof diffPngBuffers>>;
        vsBaseline: Awaited<ReturnType<typeof diffPngBuffers>> | null;
      }
    > = {};

    for (const key of Object.keys(run1)) {
      const runToRun = await diffPngBuffers(page, run1[key], run2[key]);
      const baselinePath = path.join(SNAPSHOT_DIR, `${key}.png`);
      let vsBaseline: Awaited<ReturnType<typeof diffPngBuffers>> | null = null;

      if (!updateBaselines && fs.existsSync(baselinePath)) {
        const baselineBuf = fs.readFileSync(baselinePath);
        vsBaseline = await diffPngBuffers(page, run1[key], baselineBuf);
      } else {
        fs.writeFileSync(baselinePath, run1[key]);
      }

      metrics[key] = { runToRun, vsBaseline };
    }

    console.log("PLAYWRIGHT_VISUAL_STABILITY_REPORT=" + JSON.stringify(metrics, null, 2));

    for (const [key, m] of Object.entries(metrics)) {
      expect(
        m.runToRun.diffPct,
        `Run-to-run pixel diff too high for ${key}: ${m.runToRun.diffPct}%`
      ).toBeLessThanOrEqual(0.2);
      if (m.vsBaseline) {
        expect(
          m.vsBaseline.diffPct,
          `Baseline regression pixel diff too high for ${key}: ${m.vsBaseline.diffPct}%`
        ).toBeLessThanOrEqual(0.2);
      }
    }
  });
});
