#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""临时脚本：把 13-智能体与Agent工程 12 个文件移到 05-智能体运营，05 重命名为 05-智能体与Agent，13 标记为已合并。"""
import os, shutil, sys

ROOT = r'D:\RustGameServer'
DOCS = os.path.join(ROOT, 'docs')

SRC = os.path.join(DOCS, '13-智能体与Agent工程')
DST = os.path.join(DOCS, '05-智能决策层')
DST_NEW = os.path.join(DOCS, '05-智能体与Agent')
SRC_MARKER = SRC + '.merged-to-05'

# 1) 移动 SRC -> DST
if not os.path.isdir(SRC):
    print('SRC not found:', SRC); sys.exit(1)
if not os.path.isdir(DST):
    print('DST not found:', DST); sys.exit(1)

moved = 0
for name in os.listdir(SRC):
    sp = os.path.join(SRC, name)
    dp = os.path.join(DST, name)
    if os.path.isfile(sp):
        shutil.move(sp, dp)
        moved += 1
        print(f'  moved: {name}')
print(f'[1] moved {moved} files from 13 -> 05')

# 2) 重命名 DST
os.rename(DST, DST_NEW)
print(f'[2] renamed: 05-智能体运营 -> 05-智能体与Agent')

# 3) 13 已空，改名为标记
os.rename(SRC, SRC_MARKER)
print(f'[3] marker: 13-智能体与Agent工程 -> 13-智能体与Agent工程.merged-to-05')

# 4) 验证
print('\n[4] verification:')
for d in sorted(os.listdir(DOCS)):
    p = os.path.join(DOCS, d)
    if os.path.isdir(p):
        n = sum(1 for f in os.listdir(p) if os.path.isfile(os.path.join(p, f)))
        print(f'  {d}  {n} files')
