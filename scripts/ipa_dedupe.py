#!/usr/bin/env python3
"""
修复脚本叠加副作用：删除所有重复的 ## 二级小节标题。
策略：对每个 ## 标题，仅保留第一次出现，删除后续重复。
但要注意：## 1.1 目的 跟 ## 1.1 目的（不同子段） 可能出现在不同上下文。
更安全的策略：基于二级标题完全匹配（## 1.1 目的、## 1.2 适用范围 等）
"""
import re
import sys
from pathlib import Path

sys.stdout.reconfigure(encoding='utf-8', errors='replace')

DOCS_ROOT = Path(r'D:\RustGameServer\docs')

# 已知会出现重复的二级标题前缀
DUP_PATTERNS = [
    r'##\s*1\.1\s*目的',
    r'##\s*1\.2\s*适用范围',
    r'##\s*1\.2\s*适用范围（',
    r'##\s*1\.3\s*关联文档',
    r'##\s*1\.4\s*记述规则',
    r'##\s*1\.4\s*命名约定',
    r'##\s*1\.5\s*字段级映射说明',
    r'##\s*2\.\s*测试策略',
    r'##\s*3\.\s*测试用例',
    r'##\s*3\.1\s*模块',
    r'##\s*4\.\s*追溯性',
    r'##\s*5\.\s*测试执行',
    r'##\s*6\.\s*通过判定',
    r'##\s*6\.5\s*NFR',
    r'##\s*7\.\s*风险',
    r'##\s*7\.5\s*TBD',
    r'##\s*目次',
]

def remove_dup_sections(c, pattern):
    """删除重复的二级小节，保留第一个（无论内容），删后续"""
    matches = list(re.finditer(pattern, c, re.MULTILINE))
    if len(matches) <= 1:
        return c, 0
    # 从后往前删
    removed = 0
    for m in reversed(matches[1:]):
        # 找该小节结束（下一个 ## 或 ---）
        next_h = c.find('\n## ', m.end())
        if next_h == -1:
            next_h = c.find('\n---\n', m.end())
        if next_h == -1:
            next_h = len(c)
        # 检查小节内容是否实质性（不只是标题）
        section = c[m.start():next_h]
        if len(section.strip()) > 50:  # 实质性内容
            c = c[:m.start()] + c[next_h:]
            removed += 1
    return c, removed

def fix_doc(tst_path):
    c = tst_path.read_text(encoding='utf-8')
    changes = []
    for pat in DUP_PATTERNS:
        c, n = remove_dup_sections(c, pat)
        if n > 0:
            changes.append(f'删除重复: {pat} × {n}')
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
    sys.stdout.write(f'\n=== 去重完成：fixed={fixed} ===\n')

if __name__ == '__main__':
    main()
