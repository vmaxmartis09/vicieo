import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { execSync, spawn } from "node:child_process";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const DASHBOARD_DIR = path.resolve(__dirname, "..");
const REPO_ROOT = path.resolve(DASHBOARD_DIR, "..", "..");
const DIST_DIR = ".next-build-e2e";
const PORT = process.env.PLAYWRIGHT_PORT || "3019";
const BASE_URL = `http://127.0.0.1:${PORT}`;

function restoreTsconfigIfMutated() {
  try {
    const diff = execSync("git diff -- tool/dashboard/tsconfig.json", { cwd: REPO_ROOT, encoding: "utf8" });
    if (diff.includes(DIST_DIR)) {
      execSync("git checkout -- tool/dashboard/tsconfig.json", { cwd: REPO_ROOT, stdio: "inherit" });
    }
  } catch {
    // Ignore git check failure outside repo
  }
}

async function waitForServer(url, timeoutMs = 30000) {
  const started = Date.now();
  while (Date.now() - started < timeoutMs) {
    try {
      const res = await fetch(url, { redirect: "manual" });
      if (res.status >= 200 && res.status < 500) return;
    } catch {
      // Retry until ready
    }
    await new Promise((r) => setTimeout(r, 300));
  }
  throw new Error(`Timed out waiting for isolated Next server at ${url}`);
}

async function main() {
  let serverProc = null;
  try {
    console.log(`[e2e-visual] Building isolated production dist into ${DIST_DIR}...`);
    execSync("npx next build", {
      cwd: DASHBOARD_DIR,
      env: { ...process.env, NEXT_DIST_DIR: DIST_DIR },
      stdio: "inherit",
    });

    console.log(`[e2e-visual] Starting isolated next start at ${BASE_URL}...`);
    const nextBin = path.join(DASHBOARD_DIR, "node_modules", ".bin", "next");
    serverProc = spawn(nextBin, ["start", "-p", PORT], {
      cwd: DASHBOARD_DIR,
      env: { ...process.env, NEXT_DIST_DIR: DIST_DIR },
      stdio: "inherit",
    });

    await waitForServer(`${BASE_URL}/studio`);
    console.log("[e2e-visual] Running Playwright determinism suite...");
    const pwBin = path.join(DASHBOARD_DIR, "node_modules", ".bin", "playwright");
    execSync(`${JSON.stringify(pwBin)} test`, {
      cwd: DASHBOARD_DIR,
      env: { ...process.env, PLAYWRIGHT_BASE_URL: BASE_URL },
      stdio: "inherit",
    });
  } finally {
    if (serverProc && !serverProc.killed) {
      serverProc.kill("SIGTERM");
    }
    fs.rmSync(path.join(DASHBOARD_DIR, DIST_DIR), { recursive: true, force: true });
    restoreTsconfigIfMutated();
  }
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
