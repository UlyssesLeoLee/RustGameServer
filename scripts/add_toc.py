#!/usr/bin/env python3
"""
为每份 TST 文档添加"## 目次（目录）"小节。
"""
import re
import sys
from pathlib import Path

sys.stdout.reconfigure(encoding='utf-8', errors='replace')

DOCS_ROOT = Path(r'D:\RustGameServer\docs')

TOC_TEMPLATE = """## 目次（目次 / Table of Contents）

1. 前言（はじめに / Preface）
   1.1 目的（目的 / Purpose）
   1.2 适用范围（適用範囲 / Scope）
   1.3 关联文档（関連文書 / Related Documents）
   1.4 记述规则（記述規則 / Notation Rules）
   1.5 字段级映射说明
   1.6 命名约定（命名規約 / Naming Convention）
2. 测试策略（テスト戦略 / Test Strategy）
3. 测试用例（テストケース / Test Cases）
4. 追溯性矩阵（トレーサビリティ・マトリクス / Traceability Matrix）
5. 测试执行计划（テスト実行計画 / Test Execution Plan）
6. 通过判定基准（合格判定基準 / Pass Criteria）
7. 风险与未决事项（リスクと未決事項 / Risks and TBDs）

注：本文档实际章节以文中二级标题为准。

"""

def add_toc(tst_path):
    c = tst_path.read_text(encoding='utf-8')
    if '## 目次' in c:
        return False, '已存在'
    # 找"## 1. 前言"位置
    m = re.search(r'\n## 1\.\s*前言', c)
    if not m:
        return False, '无 ## 1. 前言 锚点'
    # 在"## 1. 前言"之前插入目次
    c = c[:m.start()] + '\n' + TOC_TEMPLATE + c[m.start():]
    tst_path.write_text(c, encoding='utf-8')
    return True, '已添加'

def main():
    fixed = 0
    for topic_dir in sorted(DOCS_ROOT.iterdir()):
        if not topic_dir.is_dir():
            continue
        for tf in sorted(topic_dir.glob('RGS-TST-*.md')):
            ok, msg = add_toc(tf)
            sys.stdout.write(f'  [{"OK" if ok else "SKIP"}] {tf.name}: {msg}\n')
            sys.stdout.flush()
            if ok: fixed += 1
    sys.stdout.write(f'\n=== 目次添加：fixed={fixed} ===\n')

if __name__ == '__main__':
    main()
