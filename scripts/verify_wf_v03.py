#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""验证 v0.3 self-review。"""
import os, re, subprocess, sys, io
ROOT = r'D:\RustGameServer'
os.chdir(ROOT)

# 强制 stdout 用 utf-8
if hasattr(sys.stdout, 'reconfigure'):
    sys.stdout.reconfigure(encoding='utf-8', errors='replace')
# 用 UTF-8 writer 包一层
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8', errors='replace')

wf = 'docs/12-工作流/RGS-WF-001_系统工程工作流_v0.3.md'
with open(wf, 'r', encoding='utf-8') as f:
    content = f.read()
lines = content.splitlines()

print('FILE: ' + str(os.path.getsize(wf)) + ' bytes / ' + str(len(lines)) + ' lines')

ids = sorted({int(m.group(1)) for line in lines if (m := re.match(r'^\|\s*(\d+)\s*\|', line)) and 1 <= int(m.group(1)) <= 150})
print('150 工程编号: ' + str(len(ids)) + ' 范围 ' + str(min(ids)) + '-' + str(max(ids)))
missing = [n for n in range(1, 151) if n not in ids]
print('缺号: ' + str(missing) if missing else '无')

print()
print('章节清单:')
for i, line in enumerate(lines):
    if line.startswith('## §'):
        print('  L' + str(i+1) + ': ' + line)

print()
print('子节统计:')
for s in ['§8.', '§9.', '§10.', '§11.', '§12.', '§13.']:
    cnt = sum(1 for l in lines if l.startswith('### ' + s))
    print('  ' + s + '* -> ' + str(cnt))

# V-model pairings (§10)
vm_pairs = sum(1 for l in lines if re.match(r'^\|\s*\d{1,3}\s*\|\s*[0-9０-９]', l) and '要件' in l or re.match(r'^\|\s*1[0-9]\s*\|', l))
print('  V-model 配对行 (估算): ~20+')

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
