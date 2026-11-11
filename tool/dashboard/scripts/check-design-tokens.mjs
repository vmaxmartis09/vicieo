#!/usr/bin/env node
/**
 * check-design-tokens.mjs
 * Kiểm tra việc tuân thủ Design Tokens theo Hợp đồng thiết kế (Design Contract).
 * Quét các file .ts, .tsx, .js, .jsx trong app/ và components/:
 * - text-[..px]
 * - rounded-[..]
 * - p/m/gap/space-[..px]
 * - các bước .5 (p-1.5, gap-2.5, m-0.5, ...)
 * - các bước khoảng cách cấm 7, 9, 11 (p-7, gap-9, ...)
 */

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const DASHBOARD_ROOT = path.resolve(__dirname, "..");

const SCAN_DIRS = ["app", "components"];
const EXTENSIONS = new Set([".ts", ".tsx", ".js", ".jsx"]);

// Pattern kiểm tra vi phạm
const RULES = [
  {
    name: "arbitrary-text-size",
    desc: "Cấm cỡ chữ tuỳ tiện text-[..px], dùng token text-caption..text-largetitle",
    regex: /\btext-\[\s*\d+(?:\.\d+)?(?:px|rem|em)?\s*\]/g,
  },
  {
    name: "arbitrary-radius",
    desc: "Cấm bo góc tuỳ tiện rounded-[..], dùng token rounded-xs/ctl/card/panel/sheet",
    regex: /\brounded-\[[^\]]+\]/g,
  },
  {
    name: "arbitrary-spacing",
    desc: "Cấm khoảng cách tuỳ tiện p/m/gap/space-[..px], dùng bước chuẩn lưới 4px",
    regex: /\b(?:p|px|py|pt|pb|pl|pr|m|mx|my|mt|mb|ml|mr|gap|gap-x|gap-y|space-x|space-y)-\[[^\]]+\]/g,
  },
  {
    name: "spacing-half-steps",
    desc: "Cấm bước .5 (như p-1.5, gap-2.5, m-0.5), ánh xạ sang .5->1, 1.5->2, 2.5->3",
    regex: /\b(?:p|px|py|pt|pb|pl|pr|m|mx|my|mt|mb|ml|mr|gap|gap-x|gap-y|space-x|space-y)-\d+\.5\b/g,
  },
  {
    name: "spacing-forbidden-steps",
    desc: "Cấm bước khoảng cách 7, 9, 11 (lưới 4px: 0,1,2,3,4,5,6,8,10,12)",
    regex: /\b(?:p|px|py|pt|pb|pl|pr|m|mx|my|mt|mb|ml|mr|gap|gap-x|gap-y|space-x|space-y)-(?:7|9|11)\b/g,
  },
];

function getAllFiles(dir, fileList = []) {
  if (!fs.existsSync(dir)) return fileList;
  const entries = fs.readdirSync(dir, { withFileTypes: true });
  for (const entry of entries) {
    const fullPath = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      if (
        entry.name === "node_modules" ||
        entry.name.startsWith(".next") ||
        entry.name === ".git"
      ) {
        continue;
      }
      getAllFiles(fullPath, fileList);
    } else if (entry.isFile() && EXTENSIONS.has(path.extname(entry.name))) {
      fileList.push(fullPath);
    }
  }
  return fileList;
}

function checkFile(filePath) {
  const content = fs.readFileSync(filePath, "utf8");
  const lines = content.split("\n");
  const violations = [];

  for (let lineIdx = 0; lineIdx < lines.length; lineIdx++) {
    const line = lines[lineIdx];
    // Bỏ qua dòng comment đơn thuần
    const trimmed = line.trim();
    if (trimmed.startsWith("//") || trimmed.startsWith("/*") || trimmed.startsWith("*")) {
      continue;
    }

    for (const rule of RULES) {
      rule.regex.lastIndex = 0;
      let match;
      while ((match = rule.regex.exec(line)) !== null) {
        violations.push({
          file: path.relative(DASHBOARD_ROOT, filePath),
          line: lineIdx + 1,
          col: match.index + 1,
          rule: rule.name,
          desc: rule.desc,
          matched: match[0],
        });
      }
    }
  }

  return violations;
}

function getSubdirKey(relPath) {
  const parts = relPath.split(path.sep);
  if (parts.length >= 2) {
    return `${parts[0]}/${parts[1]}`;
  }
  return parts[0] || "root";
}

function main() {
  console.log("🔍 Đang quét vi phạm Design Tokens (app/, components/)...\n");

  const files = [];
  for (const dirName of SCAN_DIRS) {
    const fullDir = path.join(DASHBOARD_ROOT, dirName);
    getAllFiles(fullDir, files);
  }

  const allViolations = [];
  const dirStats = {};

  for (const file of files) {
    const fileViolations = checkFile(file);
    if (fileViolations.length > 0) {
      allViolations.push(...fileViolations);
      for (const v of fileViolations) {
        const key = getSubdirKey(v.file);
        dirStats[key] = (dirStats[key] || 0) + 1;
      }
    }
  }

  console.log("📊 BẢNG THỐNG KÊ SỐ LƯỢNG VI PHẠM THEO THƯ MỤC:");
  console.log("────────────────────────────────────────────────────────────");
  const sortedDirs = Object.entries(dirStats).sort((a, b) => b[1] - a[1]);
  if (sortedDirs.length === 0) {
    console.log("✨ Tuyệt vời! Không phát hiện bất kỳ vi phạm token nào.");
  } else {
    for (const [dir, count] of sortedDirs) {
      console.log(`  📁 ${dir.padEnd(30)} : ${count} vi phạm`);
    }
  }
  console.log("────────────────────────────────────────────────────────────");
  console.log(`Tổng số vi phạm: ${allViolations.length} (trên ${files.length} tệp được quét)\n`);

  if (allViolations.length > 0) {
    console.log("Chi tiết 30 vi phạm đầu tiên:");
    for (const v of allViolations.slice(0, 30)) {
      console.log(`  - ${v.file}:${v.line} -> [${v.matched}] (${v.rule})`);
    }
    if (allViolations.length > 30) {
      console.log(`  ... và còn ${allViolations.length - 30} vi phạm khác.`);
    }
    console.log("\n❌ Kiểm tra Design Tokens thất bại. Vui lòng chuẩn hóa theo ai/rules/design-standard.md\n");
    process.exit(1);
  } else {
    console.log("✅ Tất cả Design Tokens hợp lệ!\n");
    process.exit(0);
  }
}

main();
