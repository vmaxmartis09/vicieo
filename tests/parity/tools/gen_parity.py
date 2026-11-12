#!/usr/bin/env python3
"""Generate ai/plans/PARITY.md + tests/parity/endpoints.yaml (read-only on engine code).
Run: python3 tests/parity/tools/gen_parity.py"""
import re, os, glob, json, sys
V = "/Volumes/Data/vicieo"; S = "/Volumes/Data/socnhotv"
API = f"{S}/tool/src/socnhotv/core/api"
RS = f"{V}/engine-rust/src"; DASH = f"{V}/tool/dashboard"

def norm(p):  # {x}/{*x}/{x:path} -> {}
    p = re.sub(r"\{\*?[^}]*\}", "{}", p).rstrip("/")
    return p or "/"

# ---- Python
py = {}
for f in glob.glob(f"{API}/**/*.py", recursive=True):
    t = open(f).read(); pre = ""
    m = re.search(r'APIRouter\(\s*prefix="([^"]*)"', t)
    if m: pre = m.group(1)
    for m in re.finditer(r'@(router|app)\.(get|post|put|delete|patch|websocket)\(\s*"([^"]*)"(.*?)\)\s*\n\s*(?:async )?def (\w+)', t, re.S):
        who, meth, path, rest, fn = m.groups()
        rm = re.search(r'response_model=([\w\[\], .]+)', rest)
        pre_ = "" if who == "app" else pre
        py.setdefault((meth.upper(), norm(pre_ + path)), []).append((fn, rm.group(1) if rm else "", os.path.relpath(f, API)))
# ---- Rust
rs = {}
for f in glob.glob(f"{RS}/**/*.rs", recursive=True):
    t = open(f).read()
    for m in re.finditer(r'\.route\(\s*"([^"]*)"\s*,\s*((?:[a-z]+\([\w:]+\)\s*\.?\s*)+)\)', t, re.S):
        path, h = m.groups()
        for mm in re.finditer(r'(get|post|put|delete|patch)\(([\w:]+)\)', h):
            rs.setdefault((mm.group(1).upper(), norm(path)), []).append((mm.group(2), os.path.relpath(f, RS)))
# stub heuristic: handler body mentions todo/unimplemented/not implemented/stub
def kind(handler, f):
    t = open(f"{RS}/{f}").read(); name = handler.split("::")[-1]
    m = re.search(r'fn %s\b.*?(?=\n(?:pub )?(?:async )?fn |\Z)' % name, t, re.S)
    b = m.group(0) if m else ""
    if not b: return "?"
    if re.search(r'todo!|unimplemented!|not[ _]implemented|\bstub\b|\bmock\b|"fake', b, re.I) or len(b.splitlines()) <= 4 and "json!" in b and "state" not in b: return "giả"
    return "thật"
# ---- Dashboard
calls = {}
exts = ("ts","tsx")
for d in ("app","components","store","lib","hooks"):
    for f in glob.glob(f"{DASH}/{d}/**/*", recursive=True):
        if not f.endswith(exts) or os.path.isdir(f) or "node_modules" in f or "/api/" in f.replace(DASH,"") and f.endswith("route.ts"): continue
        t = open(f, errors="ignore").read()
        for m in re.finditer(r'(?:\$\{(?:CORE_URL|BASE|base|SOCIAL_URL)\}|`/core|"/core|`/(?=(?:projects|data|voices|health|node-types|custom-node-types|models|providers|graph|video|voice|settings|api/voice)\b))([^`"\'\s?]*)', t):
            p = m.group(0)
            full = re.sub(r"^\$\{\w+\}|^[`\"]/core|^`", "", p)
            full = full if full.startswith("/") else "/" + full
            full = re.sub(r"\$\{[^}]*\}", "{}", full)
            calls.setdefault(norm(full), set()).add(os.path.relpath(f, DASH))
rewrites = ["/projects/*","/data/*","/voices/*","/health","/node-types","/custom-node-types","/models","/providers","/graph/*","/video/*","/voice/*"]

def rs_has(m, p):
    return (m, p) in rs
rows = []; keys = sorted(set(py) | set(rs) | {("*", p) for p in calls}, key=lambda k: (k[1], k[0]))
for m, p in keys:
    if m == "*":
        if any(pp == p for _, pp in py) or any(pp == p for _, pp in rs): continue
        rows.append((m, p, False, False, "thiếu", True)); continue
    r = rs.get((m, p)); k = kind(*r[0]) if r else "thiếu"
    rows.append((m, p, (m, p) in py, bool(r), k, p in calls))
track = lambda p: ("A" if re.search(r"/(graph|run|runs|events|estimate|spend|outputs|artifacts|node-types|custom-node-types)\b", p) else
                   "B" if re.search(r"/(voice|voices|characters|video|editor|caption|audio)", p) else
                   "C" if "/social" in p else "D" if re.search(r"/(products|library|policy|channels|koc|episodes|scripts|script-data|projects|data)", p) else "E")
ui_err = [r for r in rows if r[5] and not r[3]]
out = ["# PARITY — Python (FastAPI) vs Rust (Axum) vs Dashboard", "",
 "Sinh tự động bởi `tests/parity/tools/gen_parity.py` (chạy lại sau mỗi track merge). Track A–E là heuristic theo path — chỉnh tay nếu sai.", "",
 f"- Python: {len(py)} endpoint · Rust: {len(rs)} · Dashboard path gọi: {len(calls)}",
 f"- **Lỗi UI (dashboard gọi, Rust thiếu): {len(ui_err)}**", "",
 "## Dashboard → Rust :9000 rewrites (next.config.mjs)", "", ", ".join(f"`{r}`" for r in rewrites), "",
 "Không có rewrite: `/settings/*`, `/api/voice/preview` (Rust có route), `/core/*` (Next route = snapshot Vercel read-only, KHÔNG proxy Python), social client gọi cổng `:8103`.",
 "Route Next.js `app/api/*` (toolbox, templates): xử lý cục bộ trong Next, không proxy sang Python (xem mục dưới).", "",
 "## Lỗi UI — dashboard gọi nhưng Rust thiếu", "", "| Method | Path | Python | Track | Gọi từ |", "|---|---|---|---|---|"]
for m, p, a, b, k, c in ui_err:
    out.append(f"| {m} | `{p}` | {'✓' if a else '✗'} | {track(p)} | {', '.join(sorted(calls.get(p, [])))[:80]} |")
out += ["", "## Bảng đầy đủ", "", "| Method | Path | Python | Rust | Rust thật/giả/thiếu | Dashboard | Track |", "|---|---|---|---|---|---|---|"]
for m, p, a, b, k, c in rows:
    out.append(f"| {m} | `{p}` | {'✓' if a else ''} | {'✓' if b else ''} | {k} | {'✓' if c else ''} | {track(p)} |")
open(f"{V}/ai/plans/PARITY.md", "w").write("\n".join(out) + "\n")
# ---- yaml: read-only GET with no path params
y = ["# Sinh bởi tests/parity/tools/gen_parity.py. python_base chỉ dùng khi backend Python chạy.",
     "rust_base: http://127.0.0.1:9000", "python_base: http://127.0.0.1:8000", "endpoints:"]
for m, p, a, b, k, c in rows:
    if m == "GET" and "{" not in p and p not in ("/video/templates/preview.mp4",):
        y.append(f"  - {{method: GET, path: {p}, python: {str(a).lower()}, rust_expected: {str(b).lower()}, dashboard: {str(c).lower()}}}")
open(f"{V}/tests/parity/endpoints.yaml", "w").write("\n".join(y) + "\n")
print(len(rows), "rows;", len(ui_err), "ui errors")

# ---- node types
nt_py = set()
for f in glob.glob(f"{S}/tool/src/socnhotv/core/nodes/**/*.py", recursive=True):
    nt_py |= set(re.findall(r'type_name="([\w.-]+)"', open(f).read()))
nt_rs = set()
for cand in glob.glob(f"{V}/data/shared/node_types.json"):
    j = json.load(open(cand)); nt_rs = {x.get("type_name") or x.get("type") for x in j if isinstance(x, dict)} - {None}
exec_rs = set()
for f in glob.glob(f"{RS}/**/*.rs", recursive=True):
    t = open(f).read()
    for m in re.finditer(r'impl NodeExecutor for \w+.*?fn node_types\(&self\).*?&\[(.*?)\]', t, re.S):
        exec_rs |= set(re.findall(r'"([\w.-]+)"', m.group(1)))
sec = ["", "## Node types (core/nodes vs Rust /node-types)", "",
 f"Python {len(nt_py)} · Rust node_types.json {len(nt_rs)} · executor thật đăng ký: {len(exec_rs)}", "",
 "| type | Python | Rust /node-types | Executor |", "|---|---|---|---|"]
for t in sorted(nt_py | nt_rs):
    sec.append(f"| {t} | {'✓' if t in nt_py else ''} | {'✓' if t in nt_rs else 'thiếu'} | {'thật' if t in exec_rs else 'Fallback'} |")
open(f"{V}/ai/plans/PARITY.md", "a").write("\n".join(sec) + "\n")
