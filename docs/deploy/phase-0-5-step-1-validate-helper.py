"""Phase 0.5 Step 1 manifest validator helper (called from ps1).
Per WF-0-5-1: client-side YAML parse + structural validation since
kubectl --dry-run=client needs a live cluster to download OpenAPI.
"""
import yaml
import sys
import glob
import os
import json

manifest_dir = sys.argv[1]
expected = [
    '00-namespace.yaml',
    '01-player-service.yaml',
    '02-economy-service.yaml',
    '03-match-service.yaml',
    '04-social-service.yaml',
    '05-admin-service.yaml',
    '06-cluster-ops-service.yaml',
    '07-shared-platform.yaml',
    '08-configmap-template.yaml',
    '09-secret-template.yaml',
    '10-rbac-template.yaml',
]
all_ok = True
report = []
for f in expected:
    p = os.path.join(manifest_dir, f)
    if not os.path.exists(p):
        report.append({'file': f, 'status': 'MISSING'})
        all_ok = False
        continue
    try:
        with open(p, encoding='utf-8') as fh:
            docs = list(yaml.safe_load_all(fh))
        bad = []
        kinds = []
        names = []
        for d in docs:
            if not d:
                continue
            kind = d.get('kind', '?')
            api = d.get('apiVersion', '?')
            name = d.get('metadata', {}).get('name', '?')
            kinds.append(kind)
            names.append(name)
            if not kind or not api or not name:
                bad.append({'kind': kind, 'name': name, 'apiVersion': api})
        status = 'PASS' if not bad else 'FAIL'
        if bad:
            all_ok = False
        report.append({
            'file': f,
            'status': status,
            'docs': len(docs),
            'kinds': kinds,
            'names': names,
            'issues': bad,
        })
    except Exception as e:
        report.append({'file': f, 'status': 'PARSE_ERROR', 'error': str(e)})
        all_ok = False

for r in report:
    print(f"  [{r['status']:<11}] {r['file']:<32}  docs={r.get('docs','?')}  kinds={r.get('kinds',[])}")

print()
print(f"OVERALL: {'PASS' if all_ok else 'FAIL'} ({sum(1 for r in report if r.get('status')=='PASS')}/{len(report)} files)")
sys.exit(0 if all_ok else 1)
