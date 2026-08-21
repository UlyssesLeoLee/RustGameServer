#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""验证 RGS-WF-001 v0.4 的工程编号、章节与审核矩阵。"""
import os
from pathlib import Path
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
os.chdir(ROOT)

# 强制 stdout 用 utf-8
if hasattr(sys.stdout, 'reconfigure'):
    sys.stdout.reconfigure(encoding='utf-8', errors='replace')

wf = 'docs/12-工作流/RGS-WF-001_系统工程工作流_v0.4.md'
with open(wf, 'r', encoding='utf-8') as f:
    content = f.read()
lines = content.splitlines()

print('FILE: ' + str(os.path.getsize(wf)) + ' bytes / ' + str(len(lines)) + ' lines')

ids = sorted({int(m.group(1)) for line in lines if (m := re.match(r'^\|\s*(\d+)\s*\|', line)) and 1 <= int(m.group(1)) <= 150})
if not ids:
    print('FAIL: 未检出 1-150 的工程编号', file=sys.stderr)
    raise SystemExit(1)

print('150 工程编号: ' + str(len(ids)) + ' 范围 ' + str(min(ids)) + '-' + str(max(ids)))
missing = [n for n in range(1, 151) if n not in ids]
print('缺号: ' + str(missing) if missing else '无')
if missing:
    raise SystemExit(1)

print()
print('章节清单:')
for i, line in enumerate(lines):
    if line.startswith('## §'):
        print('  L' + str(i+1) + ': ' + line)

print()
print('子节统计:')
for s in ['§8.', '§9.', '§10.', '§11.', '§12.', '§13.', '§14.']:
    cnt = sum(1 for l in lines if l.startswith('### ' + s))
    print('  ' + s + '* -> ' + str(cnt))

# V-model pairings (§10). 仅统计§10内以工程编号开头的数据行，不计表头。
vm_start = content.find('## §10 ')
vm_end = content.find('## §11 ', vm_start)
if vm_start == -1 or vm_end == -1:
    print('FAIL: 未找到完整的 §10 V-model 章节', file=sys.stderr)
    raise SystemExit(1)
vm_pairs = sum(
    1
    for line in content[vm_start:vm_end].splitlines()
    if re.match(r'^\|\s*\d+(?:-\d+)?\s*\|', line)
)
print('  V-model 配对行: ' + str(vm_pairs))

# 修订历史
print()
print('修订历史:')
for i, line in enumerate(lines):
    if re.match(r'^\|\s*0\.\d\s*\|', line):
        print('  L' + str(i+1) + ': ' + line[:180])

print()
print('最近 5 commit:')
GIT = ['git', '-c', 'core.quotePath=false']
r = subprocess.run(GIT + ['log', '--oneline', '-5'], capture_output=True, text=True, encoding='utf-8', errors='replace')
print(r.stdout)
