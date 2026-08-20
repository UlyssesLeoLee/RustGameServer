#!/usr/bin/env python3
"""
修复二级标题的编号顺序：
原文档：
  ## 1.1 目的
  ## 1.2 适用范围
  ## 1.3 关联文档
  ## 1.4 命名约定      ← 应该是 1.6（因为插入了 1.4 记述规则 和 1.5 字段级映射说明）

目标：
  ## 1.1 目的
  ## 1.2 适用范围
  ## 1.3 关联文档
  ## 1.4 记述规则
  ## 1.5 字段级映射说明
  ## 1.6 命名约定

类似地处理 2./3./4. 等。
"""
import re
import sys
from pathlib import Path

sys.stdout.reconfigure(encoding='utf-8', errors='replace')

DOCS_ROOT = Path(r'D:\RustGameServer\docs')

# 已知的需要重编号的章节（基于 1.4 字段级映射说明/1.4 命名约定 顺序问题）
RENUMBER_RULES = [
    # 找顺序：1.1 目的、1.2 适用范围、1.3 关联文档、1.4 记述规则、1.5 字段级映射说明、1.4 命名约定
    # 把 1.4 命名约定 改为 1.6 命名约定
    (r'\n## 1\.4 命名约定\n', r'\n## 1.6 命名约定\n'),
    # 同理 1.5 字段级映射说明之前的章节需要重编号
    # 如果同时有 1.4 记述规则 和 1.5 字段级映射说明 又有 1.4 命名约定，则 1.4 命名约定 → 1.6
]

def fix_doc(tst_path):
    c = tst_path.read_text(encoding='utf-8')
    changes = []
    for pat, repl in RENUMBER_RULES:
        n = len(re.findall(pat, c))
        if n > 0:
            c = re.sub(pat, repl, c)
            changes.append(f'{pat} → {repl} (×{n})')
    if changes:
        tst_path.write_text(c, encoding='utf-8')
    return changes

def main():
    fixed = 0
    for topic_dir in sorted(DOCS_ROOT.iterdir()):
        if not topic_dir.is_dir():
            continue
        for tf in sorted(topic_dir.glob('RGS-TST-*.md')):
            changes = fix_doc(tf)
            if changes:
                sys.stdout.write(f'  [FIX] {tf.name}:\n')
                for ch in changes:
                    sys.stdout.write(f'       - {ch}\n')
                fixed += 1
            else:
                sys.stdout.write(f'  [SKIP] {tf.name}\n')
            sys.stdout.flush()
    sys.stdout.write(f'\n=== 编号修复完成：fixed={fixed} ===\n')

if __name__ == '__main__':
    main()
