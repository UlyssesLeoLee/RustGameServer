"""6th commit: 收尾 v0.4→v0.5 / v0.5→v0.6 升版 + commit_5groups.py。"""
import os, subprocess, sys
sys.stdout.reconfigure(encoding='utf-8', errors='replace')
GIT = ['git', '-c', 'core.quotePath=false']
ROOT = r'D:\RustGameServer'
os.chdir(ROOT)

def run(args):
    r = subprocess.run(GIT + args, capture_output=True, text=True, encoding='utf-8', errors='replace')
    return r.returncode, r.stdout, r.stderr

# 1) 添加 3 M + 1 ??
files = [
    'docs/11-实施QA/RGS-QA-001_实施前QA表_v0.6.md',
    'docs/12-工作流/RGS-WF-001_系统工程工作流_v0.5.md',
    'scripts/verify_wf_v05.py',
    'scripts/commit_5groups.py',
]
for f in files:
    rc, out, err = run(['add', '--', f])
    print('add ' + f + ': rc=' + str(rc))

# 2) 检查 staged
rc, out, _ = run(['diff', '--cached', '--stat'])
print()
print('=== staged ===')
print(out)

# 3) commit
msg = """chore(cleanup): v0.4->v0.5 / v0.5->v0.6 升版 + 5 commit 工具

收尾第二轮差异:
- RGS-QA-001 v0.5 -> v0.6 (实施前 QA 表, +1 行 31 项 AI 建议说明)
- RGS-WF-001 v0.4 -> v0.5 (系统工程工作流, 顶层阶段 9 + 工作包 16)
- scripts/verify_wf_v05.py v0.4 -> v0.5 (docstring + 路径常量)
- scripts/commit_5groups.py (5 commit 计划执行工具)

依赖关系: 这 4 个变更与上一轮 5 commit 是同一文档基线的一部分，
合并提交确保 53 開発環境構築 前的 7 个 Gate 验证全部指向稳定版本。"""
rc, out, err = run(['commit', '-m', msg])
if rc != 0:
    print('COMMIT FAILED: ' + err)
else:
    print('OK:')
    print(out)

# 4) 最终
print()
rc, out, _ = run(['status', '--short'])
print('status: ' + (out if out else '(clean)'))
print()
rc, out, _ = run(['log', '--oneline', '-8'])
print('最近 8 commit:')
print(out)
