"""Parity: Rust :9000 must answer every endpoint in endpoints.yaml (rust_expected) and,
if the Python backend is up, return the same JSON shape. Run: pytest tests/parity -q"""
import json, os, urllib.request, pytest, yaml

CFG = yaml.safe_load(open(os.path.join(os.path.dirname(__file__), "endpoints.yaml")))

def get(base, path):
    try:
        with urllib.request.urlopen(base + path, timeout=5) as r:
            return r.status, r.read()
    except urllib.error.HTTPError as e:
        return e.code, b""
    except OSError:
        return None, b""

def shape(v, depth=3):
    if isinstance(v, dict):
        return {k: shape(x, depth - 1) for k, x in sorted(v.items())} if depth else "obj"
    if isinstance(v, list):
        return [shape(v[0], depth - 1)] if v and depth else []
    return type(v).__name__

def up(base):
    return get(base, "/health")[0] == 200

@pytest.mark.parametrize("e", CFG["endpoints"], ids=lambda e: e["path"])
def test_endpoint(e):
    if not up(CFG["rust_base"]):
        pytest.skip("Rust :9000 not running")
    rs_code, rs_body = get(CFG["rust_base"], e["path"])
    if e["rust_expected"]:
        assert rs_code == 200, f"Rust {e['path']} -> {rs_code}"
    elif e["dashboard"]:
        pytest.xfail("dashboard calls it, Rust lacks route")
    if not (e["python"] and up(CFG["python_base"]) and rs_code == 200):
        return
    py_code, py_body = get(CFG["python_base"], e["path"])
    if py_code == 200:
        assert shape(json.loads(rs_body)) == shape(json.loads(py_body)), "JSON shape differs"
