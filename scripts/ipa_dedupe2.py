#!/usr/bin/env python3
"""
修复脚本叠加副作用：删除所有重复的 ## 二级小节标题。
"""
import re
import sys
from pathlib import Path

sys.stdout.reconfigure(encoding='utf-8', errors='replace')

DOCS_ROOT = Path(r'D:\RustGameServer\docs')

# 已知会出现重复的二级标题模式
DUP_PATTERNS = [
    r'## 1\.1 目的',
    r'## 1\.2 适用范围',
    r'## 1\.2 适用范围（',
    r'## 1\.3 关联文档',
    r'## 1\.4 记述规则',
    r'## 1\.4 命名约定',
    r'## 1\.5 字段级映射说明',
    r'## 2\. 测试策略',
    r'## 3\. 测试用例',
    r'## 3\.1 模块',
    r'## 4\. 追溯性',
    r'## 5\. 测试执行',
    r'## 6\. 通过判定',
    r'## 6\.5 NFR',
    r'## 7\. 风险',
    r'## 7\.5 TBD',
    r'## 目次',
    r'## 目录',
]

def fix_doc(tst_path):
    c = tst_path.read_text(encoding='utf-8')
    changes = []
    for pat in DUP_PATTERNS:
        # 用 find 不用正则
        positions = []
        start = 0
        while True:
            idx = c.find(pat, start)
            if idx == -1: break
            positions.append(idx)
            start = idx + len(pat)
        if len(positions) <= 1:
            continue
        # 保留第一个，删除后续（从后往前）
        removed = 0
        for pos in reversed(positions[1:]):
            # 找该小节结束位置
            next_h = c.find('\n## ', pos + len(pat))
            if next_h == -1:
                next_h = c.find('\n---\n', pos + len(pat))
            if next_h == -1:
                next_h = len(c)
            # 检查小节长度
            section = c[pos:next_h]
            if len(section.strip()) > 50:
                c = c[:pos] + c[next_h:]
                removed += 1
        if removed > 0:
            changes.append(f'{pat} × {removed}')
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
                sys.stdout.write(f'  [FIX] {tf.name}: {len(changes)} 处\n')
                for ch in changes:
                    sys.stdout.write(f'       - {ch}\n')
                fixed += 1
            else:
                sys.stdout.write(f'  [SKIP] {tf.name}\n')
            sys.stdout.flush()
    sys.stdout.write(f'\n=== 去重完成：fixed={fixed} ===\n')

if __name__ == '__main__':
    main()
