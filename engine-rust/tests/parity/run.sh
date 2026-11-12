#!/usr/bin/env bash
# Usage: tests/parity/run.sh [base_url]   (default http://localhost:9000)
BASE=${1:-http://localhost:9000}; fail=0
python3 - "$BASE" <<'PY' || fail=1
import sys,json,urllib.request,re
base=sys.argv[1]; rc=0
for l in open('tests/parity/endpoints.yaml'):
    l=l.strip()
    if not l.startswith('- {'): continue
    d={k.strip():v.strip() for k,v in (x.split(':',1) for x in re.sub(r'keys: \[(.*?)\]',lambda m:'keys: '+m.group(1).replace(',',';'),l[3:-1]).split(', '))}
    try:
        r=urllib.request.urlopen(urllib.request.Request(base+d['path'],method=d['method']),timeout=10); code=r.status; body=r.read()
    except urllib.error.HTTPError as e: code=e.code; body=b''
    ok=str(code)==d['status']
    keys=[k.strip() for k in d['keys'].split(';') if k.strip()]
    if ok and keys:
        j=json.loads(body); ok=all(k in j for k in keys)
    print(('PASS' if ok else 'FAIL'),d['method'],d['path'],code); rc|=not ok
sys.exit(rc)
PY
exit $fail
