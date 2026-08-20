#!/usr/bin/env python3
"""
为缺 §7.5 TBD 处置小节的 6 份（主题 00+01 手工版）补充该小节。
"""
import re
import sys
from pathlib import Path

sys.stdout.reconfigure(encoding='utf-8', errors='replace')

TBD_BY_TOPIC = {
    '00-基准与治理': [
        ('TBD-001', 'OLU 真实工时', 'PH-1 前完成首次校准'),
        ('TBD-003', '重连宽限期', '60s 初始值，PH-3 实测校准'),
        ('TBD-004', '个人信息保护等级', '法务确认，PH-4 前定'),
        ('TBD-GOV-001', '版本回滚 SLA', 'PH-1 前定最终值'),
        ('TBD-008', '客户端协议 N-1 接受期', 'PH-3 前定'),
        ('RSK-006', '100k CCU 实测风险', 'PH-4 1k→10k 渐进'),
        ('RSK-007', '死锁在生产显现', 'FT-007 + 静态检查 + ARC-013'),
    ],
    '01-核心架构与设计模式': [
        ('TBD-CAP-001', 'T3 多区域方案校准', '留待 PH-7 实测'),
        ('TBD-CAP-002', '跨分片能力清单', '留待 PH-6 实测'),
        ('TBD-PPL-001', '限流算法参数', '由 NFR-SEC-008 决定'),
        ('TBD-DEP-001', 'Schema 校验实现语言', '留待 PH-2'),
        ('TBD-008', '客户端协议 N-1 接受期', 'PH-3 前定'),
    ],
}

TBD_TEMPLATE = '''
## 7.5 TBD 处置

本主题涉及的 TBD 处置方式：

| TBD 编号 | 描述 | 处置 |
|---|---|---|
{rows}

'''

def add_tbd(tst_path):
    c = tst_path.read_text(encoding='utf-8')
    if '## 7.5 TBD 处置' in c:
        return False, '已存在'
    topic = tst_path.parent.name
    tbds = TBD_BY_TOPIC.get(topic, [])
    if not tbds:
        return False, '无 TBD 数据'
    rows = '\n'.join(f'| {tid} | {desc} | {act} |' for tid, desc, act in tbds)
    section = TBD_TEMPLATE.format(rows=rows)

    # 找 §7 风险与未决事项
    m = re.search(r'(##\s*7\.?\s*风险[与与]*未决事项)', c)
    if not m:
        return False, '无 §7 锚点'
    # 找该章节后第一个 ## 或文件结尾
    end_m = re.search(r'\n##\s', c[m.end():])
    if end_m:
        end_idx = m.end() + end_m.start()
    else:
        end_idx = len(c)
    c = c[:end_idx] + section + c[end_idx:]
    tst_path.write_text(c, encoding='utf-8')
    return True, '已添加'

def main():
    added = 0
    DOCS_ROOT = Path(r'D:\RustGameServer\docs')
    targets = [
        DOCS_ROOT / '00-基准与治理',
        DOCS_ROOT / '01-核心架构与设计模式',
    ]
    for topic_dir in targets:
        for tf in sorted(topic_dir.glob('RGS-TST-*.md')):
            ok, msg = add_tbd(tf)
            sys.stdout.write(f'  [{"OK" if ok else "SKIP"}] {tf.name}: {msg}\n')
            sys.stdout.flush()
            if ok: added += 1
    sys.stdout.write(f'\n=== TBD 处置补全：added={added} ===\n')

if __name__ == '__main__':
    main()
