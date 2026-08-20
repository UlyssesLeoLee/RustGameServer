#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""追踪 05-智能决策层 → 05-智能体与Agent 的 rename，并补到上一个 commit。"""
import os, subprocess, sys
ROOT = r'D:\RustGameServer'
os.chdir(ROOT)
GIT_Q = ['git', '-c', 'core.quotePath=false']

def run(args, check=True):
    r = subprocess.run(GIT_Q + args, capture_output=True, text=True, encoding='utf-8', errors='replace')
    if check and r.returncode != 0:
        print('FAIL:', ' '.join(GIT_Q + args))
        print('STDOUT:', r.stdout); print('STDERR:', r.stderr); sys.exit(1)
    return r.stdout

# 1) 找到 05-智能决策层 下被 D 的文件 (tracked but file gone)
status = run(['status', '--short'])
delete_lines = [l for l in status.splitlines() if l.startswith(' D docs/05-智能决策层/')]
print(f'[1] D lines for 05-智能决策层: {len(delete_lines)}')
for l in delete_lines:
    print(f'    {l}')

# 2) 找到 05-智能体与Agent 下已存在的同名文件 (新位置的副本)
old_dir = 'docs/05-智能决策层'
new_dir = 'docs/05-智能体与Agent'
import os
moves = []
for line in delete_lines:
    old_path = line[3:].strip()  # 'docs/05-智能决策层/RGS-BAS-011_...'
    fname = os.path.basename(old_path)
    new_path = os.path.join(new_dir, fname)
    new_path = new_path.replace(os.sep, '/')
    if os.path.exists(new_path.replace('/', os.sep)):
        moves.append((old_path, new_path))
    else:
        print(f'  WARN: {new_path} does not exist on disk')

print(f'[2] files to rename in git: {len(moves)}')
for old, new in moves:
    print(f'    {old}')
    print(f'  -> {new}')

if not moves:
    print('  nothing to do'); sys.exit(0)

# 3) 找当前 last commit (1ccd290)
last = run(['log', '-1', '--format=%H']).strip()
print(f'[3] last commit: {last}')

# 4) 用 git rm --cached old + git add new (via filter rename detection)
# 简单方法: 删 + 加，让 git 自己的相似度检测识别为 rename
for old, new in moves:
    r = subprocess.run(GIT_Q + ['rm', '--cached', '--', old],
                       capture_output=True, text=True, encoding='utf-8', errors='replace')
    if r.returncode != 0:
        print(f'  FAIL rm --cached: {old}')
        print(f'  STDERR: {r.stderr}'); sys.exit(2)
    r = subprocess.run(GIT_Q + ['add', '--', new],
                       capture_output=True, text=True, encoding='utf-8', errors='replace')
    if r.returncode != 0:
        print(f'  FAIL add: {new}')
        print(f'  STDERR: {r.stderr}'); sys.exit(3)

print(f'[4] staged {len(moves)} renames')

# 5) 把这些改动 amend 到上一个 commit (1ccd290)
r = subprocess.run(GIT_Q + ['commit', '--amend', '--no-edit'],
                   capture_output=True, text=True, encoding='utf-8', errors='replace')
if r.returncode != 0:
    print('AMEND FAILED')
    print('STDOUT:', r.stdout); print('STDERR:', r.stderr); sys.exit(4)
out = r.stdout.strip().splitlines()
print(f'[5] amended: {out[0] if out else "(no output)"}')

# 6) 验证
print()
print('=== Final git status ===')
print('\n'.join(run(['status', '--short']).splitlines()))
print()
print('=== Final 5 commits ===')
print(run(['log', '--oneline', '-5']))
