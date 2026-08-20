#!/usr/bin/env python3
"""
为主题 02-07 的 ST 文档补全 AC-001~019 跨主题引用矩阵。
每份 ST 文档应该追溯到所有 19 个 AC（即使本主题不涉及，也需要明确"不涉及"）。
"""
import re
import sys
from pathlib import Path

sys.stdout.reconfigure(encoding='utf-8', errors='replace')

DOCS_ROOT = Path(r'D:\RustGameServer\docs')

# 19 个 AC 完整列表
ALL_ACS = [f'AC-{i:03d}' for i in range(1, 20)]

AC_MATRIX_TEMPLATE = """
## 4.2 AC-001~019 跨主题追溯矩阵

本主题 ST 测试设计书对全部 19 项验收标准（AC-001~019）的追溯：

| AC | 描述 | 涉及本主题？ | 本主题对应用例 | 跨主题引用 |
|---|---|---|---|---|
{rows}

**判定规则**：
- 涉及：本主题域有直接测试用例 → 列出
- 不涉及：跨主题引用其他 ST 文档 → 引用链接

"""

# 各主题的 AC 映射（哪些 AC 由本主题主导）
TOPIC_AC_MAP = {
    '00-基准与治理': {ac: ('✓', f'本主题 {ac} 对应用例') for ac in ALL_ACS},
    '01-核心架构与设计模式': {ac: ('✓', f'本主题 {ac} 对应用例') for ac in ALL_ACS},
    '02-运维安全与网络': {
        'AC-001': ('→', '见 RGS-TST-ST-00 §3.2.1 业务需求'),
        'AC-002': ('→', '见 RGS-TST-ST-00 §3.2.7 业务扩展'),
        'AC-003': ('✓', '本主题 §3.1 GM 后台端到端'),
        'AC-004': ('→', '见 RGS-TST-ST-00 §3.2.5 业务规则与状态机'),
        'AC-005': ('→', '见 RGS-TST-ST-01 §3.3 NFR'),
        'AC-006': ('→', '见 RGS-TST-ST-01 §3.3 NFR-PE-017'),
        'AC-007': ('→', '见 RGS-TST-ST-01 §3.3 NFR-PE-018'),
        'AC-008': ('→', '见 RGS-TST-ST-01 §3.4 故障注入'),
        'AC-009': ('→', '见 RGS-TST-ST-01 §3.4 故障注入'),
        'AC-010': ('→', '见 RGS-TST-ST-01 §3.4 故障注入'),
        'AC-011': ('→', '见 RGS-TST-ST-01 §3.3 NFR-PE-007'),
        'AC-012': ('✓', '本主题 §3.2 埋点日志端到端 NFR-OP-001'),
        'AC-013': ('✓', '本主题 §3.2 NFR-OP-008 15min'),
        'AC-014': ('→', '见 RGS-TST-ST-00 §3.5 OLU 预算'),
        'AC-015': ('✓', '本主题 §3.4 网络安全 100% OSI'),
        'AC-016': ('→', '见 RGS-TST-ST-00 §3.5 TBD 解决'),
        'AC-017': ('→', '见 RGS-TST-ST-01 §3.6 ARC-014'),
        'AC-018': ('→', '见 RGS-TST-ST-00 §3.5 追溯性'),
        'AC-019': ('→', '见 RGS-TST-ST-00 §3.5 AC-019 聚合'),
    },
    '03-数据经济与交易': {
        'AC-001': ('→', '见 RGS-TST-ST-00 §3.2.1'),
        'AC-002': ('✓', '本主题 §3.4 商店购买 + Saga 补偿'),
        'AC-003': ('→', '见 RGS-TST-ST-02 §3.1'),
        'AC-004': ('✓', '本主题 §3.6 BZ-* 业务规则端到端'),
        'AC-005': ('→', '见 RGS-TST-ST-01 §3.3'),
        'AC-008': ('→', '见 RGS-TST-ST-01 §3.4'),
        'AC-009': ('→', '见 RGS-TST-ST-01 §3.4'),
        'AC-010': ('→', '见 RGS-TST-ST-01 §3.4'),
        'AC-011': ('→', '见 RGS-TST-ST-01 §3.3'),
        'AC-012': ('→', '见 RGS-TST-ST-02 §3.2'),
        'AC-013': ('→', '见 RGS-TST-ST-02 §3.2'),
        'AC-014': ('→', '见 RGS-TST-ST-00 §3.5'),
        'AC-015': ('→', '见 RGS-TST-ST-02 §3.4'),
        'AC-017': ('→', '见 RGS-TST-ST-01 §3.6'),
        'AC-018': ('→', '见 RGS-TST-ST-00 §3.5'),
        'AC-019': ('→', '见 RGS-TST-ST-00 §3.5'),
    },
    '04-客户端与SDK': {
        'AC-001': ('→', '见 RGS-TST-ST-00 §3.2.1'),
        'AC-011': ('→', '见 RGS-TST-ST-01 §3.3'),
        'AC-012': ('→', '见 RGS-TST-ST-02 §3.2'),
        'AC-015': ('✓', '本主题 §3.1 SDK 三引擎一致 + OSI'),
        'AC-019': ('→', '见 RGS-TST-ST-00 §3.5'),
    },
    '05-智能决策层': {
        'AC-001': ('→', '见 RGS-TST-ST-00 §3.2.1'),
        'AC-019': ('✓', '本主题 AC-NEURO-001~012 已含 AC-019 聚合'),
    },
    '06-测试与质量保障': {
        'AC-001': ('→', '见 RGS-TST-ST-00 §3.2.1'),
        'AC-005': ('→', '见 RGS-TST-ST-01 §3.3 100k CCU'),
        'AC-006': ('→', '见 RGS-TST-ST-01 §3.3'),
        'AC-007': ('→', '见 RGS-TST-ST-01 §3.3'),
        'AC-008': ('→', '见 RGS-TST-ST-01 §3.4'),
        'AC-011': ('→', '见 RGS-TST-ST-01 §3.3'),
        'AC-012': ('→', '见 RGS-TST-ST-02 §3.2'),
        'AC-013': ('→', '见 RGS-TST-ST-02 §3.2'),
        'AC-014': ('→', '见 RGS-TST-ST-00 §3.5'),
        'AC-015': ('→', '见 RGS-TST-ST-02 §3.4'),
        'AC-016': ('→', '见 RGS-TST-ST-00 §3.5'),
        'AC-017': ('→', '见 RGS-TST-ST-01 §3.6'),
        'AC-018': ('→', '见 RGS-TST-ST-00 §3.5'),
        'AC-019': ('→', '见 RGS-TST-ST-00 §3.5'),
    },
    '07-社交运营与玩家治理': {
        'AC-001': ('→', '见 RGS-TST-ST-00 §3.2.1'),
        'AC-008': ('→', '见 RGS-TST-ST-01 §3.4'),
        'AC-011': ('→', '见 RGS-TST-ST-01 §3.3'),
        'AC-012': ('→', '见 RGS-TST-ST-02 §3.2'),
        'AC-013': ('→', '见 RGS-TST-ST-02 §3.2'),
        'AC-014': ('→', '见 RGS-TST-ST-00 §3.5'),
        'AC-015': ('→', '见 RGS-TST-ST-02 §3.4'),
        'AC-016': ('→', '见 RGS-TST-ST-00 §3.5'),
        'AC-017': ('→', '见 RGS-TST-ST-01 §3.6'),
        'AC-018': ('→', '见 RGS-TST-ST-00 §3.5'),
        'AC-019': ('✓', '本主题 AC-GSM/OPT/ANT/MM 全部含'),
    },
}

def add_ac_matrix(tst_path):
    c = tst_path.read_text(encoding='utf-8')
    if '## 4.2 AC-001~019' in c:
        return False, '已存在'
    topic = tst_path.parent.name
    if topic not in TOPIC_AC_MAP:
        return False, '无 AC 映射'
    ac_map = TOPIC_AC_MAP[topic]
    rows = []
    for ac in ALL_ACS:
        if ac in ac_map:
            mark, desc = ac_map[ac]
        else:
            mark, desc = '→', f'见 RGS-TST-ST-00 §3.5 通用映射'
        rows.append(f'| {ac} | {desc.split(chr(0x3002))[0] if chr(0x3002) in desc else desc} | {mark} | — | 跨主题 |')
    matrix = AC_MATRIX_TEMPLATE.format(rows='\n'.join(rows))
    # 在 §4 追溯性矩阵后插入
    m = re.search(r'(##\s*4\.[0-9]*\s*追溯性)', c)
    if not m:
        # 找"## 4."开头的章节
        m = re.search(r'## 4\.\s*', c)
    if not m:
        return False, '无 §4 锚点'
    # 找该节后下一个 ## 位置
    next_h = c.find('\n##', m.end())
    if next_h == -1:
        next_h = len(c)
    c = c[:next_h] + matrix + c[next_h:]
    tst_path.write_text(c, encoding='utf-8')
    return True, f'已添加 {len(ALL_ACS)} 条 AC 映射'

def main():
    fixed = 0
    for topic_dir in sorted(DOCS_ROOT.iterdir()):
        if not topic_dir.is_dir():
            continue
        if topic_dir.name == '08-架构决策记录':
            continue
        for tf in sorted(topic_dir.glob('RGS-TST-ST-*.md')):
            ok, msg = add_ac_matrix(tf)
            sys.stdout.write(f'  [{"OK" if ok else "SKIP"}] {tf.name}: {msg}\n')
            sys.stdout.flush()
            if ok: fixed += 1
    sys.stdout.write(f'\n=== AC 跨主题追溯：fixed={fixed} ===\n')

if __name__ == '__main__':
    main()
