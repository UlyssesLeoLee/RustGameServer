"""扫描 Gate 证据状态。"""
import os, re, subprocess, sys
sys.stdout.reconfigure(encoding='utf-8', errors='replace')
GIT = ['git', '-c', 'core.quotePath=false']
ROOT = r'D:\RustGameServer'
os.chdir(ROOT)

def run(args):
    r = subprocess.run(GIT + args, capture_output=True, text=True, encoding='utf-8', errors='replace')
    return r.returncode, r.stdout, r.stderr

# 1) 列出 5 域 DTL
print('=== 5 域 DTL first slice (player/economy/match/social/admin) ===')
five_domains = {
    'player':   ['RGS-DTL-018', 'RGS-SPEC-DTL-018'],
    'economy':  ['RGS-DTL-015', 'RGS-DTL-016', 'RGS-SPEC-DTL-015', 'RGS-SPEC-DTL-016'],
    'match':    ['RGS-DTL-026', 'RGS-SPEC-DTL-026'],
    'social':   ['RGS-DTL-019', 'RGS-DTL-020', 'RGS-SPEC-DTL-019', 'RGS-SPEC-DTL-020'],
    'admin':    ['RGS-DTL-031', 'RGS-SPEC-DTL-031'],
}
all_files = set()
for d in os.listdir('docs'):
    full = os.path.join('docs', d)
    if os.path.isdir(full):
        for f in os.listdir(full):
            all_files.add(f)

for dom, prefixes in five_domains.items():
    found_dtl = []
    found_spec = []
    for f in all_files:
        for p in prefixes:
            if p in f and 'DTL' in f and 'SPEC' not in f:
                found_dtl.append(f)
            elif p in f and 'SPEC' in f:
                found_spec.append(f)
    print(f'  {dom:<10}: DTL={len(found_dtl)} SPEC={len(found_spec)}')

# 2) DTL-031 详细
print()
print('=== DTL-031 状态 ===')
p = 'docs/01-核心架构与设计模式/RGS-DTL-031_集群运营中心与每功能原子升级_详细设计书.md'
if os.path.exists(p):
    sz = os.path.getsize(p)
    with open(p, 'r', encoding='utf-8', errors='replace') as f:
        head = ''.join([next(f, '') for _ in range(20)])
    ver = re.search(r'\|\s*版本\s*\|\s*([0-9.]+)\s*\|', head)
    print(f'  {p}')
    print(f'  size: {sz} bytes  version: {ver.group(1) if ver else "?"}')

# 3) ADR-0052 状态
print()
print('=== ADR-0052 状态 ===')
p = 'docs/08-架构决策记录/RGS-ADR-0052_Active-Active_ClusterOpsService与all-reachable_PFAU容错哲学.md'
if os.path.exists(p):
    sz = os.path.getsize(p)
    with open(p, 'r', encoding='utf-8', errors='replace') as f:
        head = ''.join([next(f, '') for _ in range(20)])
    ver = re.search(r'\|\s*版本\s*\|\s*([0-9.]+)\s*\|', head)
    print(f'  {p}')
    print(f'  size: {sz} bytes  version: {ver.group(1) if ver else "?"}')

# 4) QA-001 v0.6 找 Q-003 / Q-025
print()
print('=== Q-003 / Q-025 状态 (in QA-001 v0.6) ===')
p = 'docs/11-实施QA/RGS-QA-001_实施前QA表_v0.6.md'
if os.path.exists(p):
    with open(p, 'r', encoding='utf-8', errors='replace') as f:
        content = f.read()
    for qid in ['Q-003', 'Q-025']:
        # 找包含 Q-XXX 的行
        for line in content.splitlines():
            if qid in line and '|' in line and line.strip().startswith('|'):
                if '问题' in line or '答案' in line or '候选' in line or '状态' in line or '优先级' in line or 'Blk' in line or '备注' in line:
                    print(f'  {qid}: {line[:200]}')
                    break

# 5) Rust 1.98 stable GA 状态 (查 rust release)
print()
print('=== Rust 1.98 stable 状态 ===')
import urllib.request, json
try:
    url = 'https://api.github.com/repos/rust-lang/rust/releases/latest'
    req = urllib.request.Request(url, headers={'User-Agent': 'curl/7'})
    with urllib.request.urlopen(req, timeout=8) as r:
        data = json.loads(r.read())
    print(f'  Latest: {data.get("tag_name")}  date: {data.get("published_at")}  prerelease: {data.get("prerelease")}')
except Exception as e:
    print(f'  网络检查失败: {e}')

# 6) git 最新 commit
print()
print('=== 当前 HEAD ===')
rc, out, _ = run(['log', '--oneline', '-1'])
print(out)
