import { spawn } from "node:child_process";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const rootDir = path.resolve(path.dirname(__filename));

const isWindows = process.platform === "win32";
const npmCmd = isWindows ? "npm.cmd" : "npm";
const cargoCmd = "cargo";

console.log("\x1b[36m%s\x1b[0m", "==================================================");
console.log("\x1b[36m%s\x1b[0m", " Starting Vicieo Services (Rust Engine + Next.js)");
console.log("\x1b[36m%s\x1b[0m", "==================================================");
console.log(" Dashboard:    http://localhost:3600");
console.log(" Rust Server:  http://localhost:9000 (Health: /health)\n");

const children = [];

function startProcess(name, command, args, cwd) {
  const proc = spawn(command, args, {
    cwd,
    stdio: "inherit",
    shell: true,
  });

  proc.on("error", (err) => {
    console.error(`\x1b[31m[${name}] Failed to start:\x1b[0m`, err.message);
  });

  proc.on("close", (code) => {
    if (code !== 0 && code !== null) {
      console.log(`\x1b[33m[${name}] exited with code ${code}\x1b[0m`);
    }
  });

  children.push(proc);
  return proc;
}

// 1. Rust Engine
startProcess(
  "Rust Engine",
  cargoCmd,
  ["run", "--", "serve", "--port", "9000"],
  path.join(rootDir, "engine-rust")
);

// 2. Next.js Dashboard
startProcess(
  "Next.js Dashboard",
  npmCmd,
  ["run", "dev:web"],
  path.join(rootDir, "tool", "dashboard")
);

function cleanup() {
  console.log("\n\x1b[33mStopping all services...\x1b[0m");
  for (const child of children) {
    if (!child.killed) {
      if (isWindows && child.pid) {
        try {
          spawn("taskkill", ["/pid", child.pid.toString(), "/T", "/F"]);
        } catch {
          child.kill("SIGTERM");
        }
      } else {
        child.kill("SIGTERM");
      }
    }
  }
  process.exit(0);
}

process.on("SIGINT", cleanup);
process.on("SIGTERM", cleanup);
