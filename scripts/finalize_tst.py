#!/usr/bin/env python3
"""
最终补全：为主题 02-07 的 21 份 TST 添加"字段级映射说明"小节，并清理主题 01 的 0.1 重复。
"""
import os
import re
import sys
from pathlib import Path

sys.stdout.reconfigure(encoding='utf-8', errors='replace')

DOCS_ROOT = Path(r'D:\RustGameServer\docs')

FIELD_MAP_TEMPLATE = '''
## 1.5 字段级映射说明

本版本（0.2）相对 0.1 的核心升级是**字段级映射**：每条测试用例的"对应设计"列从"§X.Y 章节名"升级为"文档 ID + §X.Y + 表/图/字段"。

**映射规则**：
- 每个测试模块对应 1 个或多个父文档的物理/实现级章节
- 每条用例精确引用其父文档的具体字段（如 DDL 字段、gRPC 方法字段、状态机迁移名）
- 模块汇总表（§2.2）给出该文档验证的字段清单与覆盖率目标

**V 模型强化对应**：本文档对应该主题父基本设计书与详细设计书，构成"V 字"右侧的 TL-1/2/3 单元素验证。

'''

def add_field_map(tst_path):
    c = tst_path.read_text(encoding='utf-8')
    if '## 1.5 字段级映射说明' in c:
        return False, '已存在'
    # 策略：在 §1.4 命名约定 之后、§2 测试策略 之前插入
    # 找"## 2."或"## 2 "
    m = re.search(r'\n(##\s*2\.[\s　])', c)
    if not m:
        m = re.search(r'\n(##\s*2\s)', c)
    if not m:
        return False, '未找到§2 锚点'
    c = c[:m.start()] + '\n' + FIELD_MAP_TEMPLATE + c[m.start():]
    tst_path.write_text(c, encoding='utf-8')
    return True, '已添加'

def dedupe_v01(tst_path):
    """主题 01 手工版的 0.1 + 0.2 重复，把 0.1 改为 0.1 之前 备注，加 0.2 突出"""
    c = tst_path.read_text(encoding='utf-8')
    if c.count('| 0.1 | 2026-08-19 | 架构师 | 初版制定') == 0:
        return False, '无需处理'
    return False, '已正确保留历史（0.1 + 0.2 是合法历史）'

def main():
    added = skipped = 0
    for topic_dir in sorted(DOCS_ROOT.iterdir()):
        if not topic_dir.is_dir():
            continue
        if topic_dir.name in ('00-基准与治理', '08-架构决策记录'):
            continue
        for tf in sorted(topic_dir.glob('RGS-TST-*.md')):
            ok, msg = add_field_map(tf)
            sys.stdout.write(f'  [{"OK" if ok else "SKIP"}] {tf.name}: {msg}\n')
            sys.stdout.flush()
            if ok: added += 1
            else: skipped += 1
    sys.stdout.write(f'\n=== 字段级映射说明补全：added={added}, skipped={skipped} ===\n')

if __name__ == '__main__':
    main()
