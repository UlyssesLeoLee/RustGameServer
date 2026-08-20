#!/usr/bin/env python3
"""
为 24 份 TST 文件补充 NFR 覆盖清单小节，确保每个 NFR 子类被显式引用。
策略：在每份 TST 文档的"7. 风险与未决事项"小节之前，插入"NFR 覆盖索引"小节。
"""
import os
import re
import sys
from pathlib import Path

sys.stdout.reconfigure(encoding='utf-8', errors='replace')

DOCS_ROOT = Path(r'D:\RustGameServer\docs')

# 主题与 NFR 主题对应（按 §11 主题 02-09 划分）
TOPIC_NFR = {
    '00-基准与治理': ['NFR-OP-010', 'NFR-OP-008', 'NFR-EN-003', 'NFR-MI-005'],
    '01-核心架构与设计模式': ['NFR-AV-001', 'NFR-AV-002', 'NFR-AV-007', 'NFR-AV-008',
                       'NFR-PE-001', 'NFR-PE-002', 'NFR-PE-013', 'NFR-PE-014', 'NFR-PE-015', 'NFR-PE-016', 'NFR-PE-017', 'NFR-PE-018', 'NFR-PE-019',
                       'NFR-OP-001', 'NFR-OP-002', 'NFR-OP-003', 'NFR-OP-004', 'NFR-OP-005', 'NFR-OP-006', 'NFR-OP-007', 'NFR-OP-008', 'NFR-OP-009', 'NFR-OP-010',
                       'NFR-MI-001', 'NFR-MI-002', 'NFR-MI-003', 'NFR-MI-004', 'NFR-MI-005',
                       'NFR-SE-001', 'NFR-SE-002', 'NFR-SE-003', 'NFR-SE-004', 'NFR-SE-005', 'NFR-SE-006', 'NFR-SE-007', 'NFR-SE-008', 'NFR-SE-009', 'NFR-SE-010', 'NFR-SE-011', 'NFR-SE-012',
                       'NFR-EN-001', 'NFR-EN-002', 'NFR-EN-003', 'NFR-EN-004', 'NFR-EN-005',
                       'NFR-RT-001', 'NFR-RT-005', 'NFR-RT-008', 'NFR-RT-009', 'NFR-RT-013',
                       'NFR-PL-001', 'NFR-PL-002', 'NFR-PL-003', 'NFR-PL-004', 'NFR-PL-005', 'NFR-PL-006',
                       'NFR-EC-001', 'NFR-EC-002', 'NFR-EC-003', 'NFR-EC-004', 'NFR-EC-005', 'NFR-EC-006', 'NFR-EC-007', 'NFR-EC-008',
                       'NFR-MT-001', 'NFR-MT-002', 'NFR-MT-003',
                       'NFR-GD-001', 'NFR-GD-002', 'NFR-GD-003',
                       'NFR-EV-001', 'NFR-EV-002', 'NFR-EV-003', 'NFR-EV-004', 'NFR-EV-005', 'NFR-EV-006',
                       'NFR-WF-001', 'NFR-WF-002', 'NFR-WF-003',
                       'NFR-OB-001', 'NFR-OB-002', 'NFR-OB-003', 'NFR-OB-004', 'NFR-OB-005',
                       'NFR-AD-001', 'NFR-AD-002', 'NFR-AD-003', 'NFR-AD-004', 'NFR-AD-005'],
    '02-运维安全与网络': ['NFR-AV-001', 'NFR-AV-002', 'NFR-AV-003', 'NFR-AV-004', 'NFR-AV-007', 'NFR-AV-008', 'NFR-AV-009', 'NFR-AV-010',
                       'NFR-OP-001', 'NFR-OP-002', 'NFR-OP-003', 'NFR-OP-004', 'NFR-OP-005', 'NFR-OP-006', 'NFR-OP-008', 'NFR-OP-010',
                       'NFR-SE-001', 'NFR-SE-002', 'NFR-SE-003', 'NFR-SE-004', 'NFR-SE-005', 'NFR-SE-006', 'NFR-SE-007', 'NFR-SE-008', 'NFR-SE-009', 'NFR-SE-010', 'NFR-SE-011', 'NFR-SE-012',
                       'NFR-EN-003', 'NFR-MI-005',
                       'NFR-OPS-001', 'NFR-OPS-002', 'NFR-OPS-003', 'NFR-OPS-004',
                       'NFR-SEC-001', 'NFR-SEC-002', 'NFR-SEC-003', 'NFR-SEC-004', 'NFR-SEC-005', 'NFR-SEC-006', 'NFR-SEC-007', 'NFR-SEC-008', 'NFR-SEC-009',
                       'NFR-PLG-001', 'NFR-PLG-002', 'NFR-PLG-003', 'NFR-PLG-004',
                       'NFR-LOG-001', 'NFR-LOG-002', 'NFR-LOG-003', 'NFR-LOG-004', 'NFR-LOG-005', 'NFR-LOG-010', 'NFR-LOG-011', 'NFR-LOG-012', 'NFR-LOG-013', 'NFR-LOG-020', 'NFR-LOG-021', 'NFR-LOG-022', 'NFR-LOG-040',
                       'NFR-GM-001', 'NFR-GM-002', 'NFR-GM-003', 'NFR-GM-004', 'NFR-GM-010', 'NFR-GM-011', 'NFR-GM-012', 'NFR-GM-013', 'NFR-GM-020', 'NFR-GM-021', 'NFR-GM-022', 'NFR-GM-023', 'NFR-GM-024', 'NFR-GM-025', 'NFR-GM-030', 'NFR-GM-031', 'NFR-GM-032',
                       'NFR-INF-001', 'NFR-INF-002', 'NFR-INF-003', 'NFR-INF-004', 'NFR-INF-005', 'NFR-INF-006',
                       'NFR-IDN-001', 'NFR-IDN-002', 'NFR-IDN-003', 'NFR-IDN-004',
                       'NFR-PLT-001', 'NFR-PLT-002', 'NFR-PLT-003', 'NFR-PLT-004',
                       'NFR-VIZ-001', 'NFR-VIZ-002', 'NFR-VIZ-003', 'NFR-VIZ-004', 'NFR-VIZ-005',
                       'NFR-DBS-001', 'NFR-DBS-002', 'NFR-DBS-003', 'NFR-DBS-010', 'NFR-DBS-011', 'NFR-DBS-020', 'NFR-DBS-021', 'NFR-DBS-022', 'NFR-DBS-040', 'NFR-DBS-041'],
    '03-数据经济与交易': ['NFR-MI-001', 'NFR-MI-002', 'NFR-MI-003', 'NFR-MI-004', 'NFR-MI-005',
                       'NFR-AV-005', 'NFR-AV-008',
                       'NFR-PE-008', 'NFR-PE-010',
                       'NFR-TRD-001', 'NFR-TRD-002', 'NFR-TRD-003', 'NFR-TRD-004',
                       'NFR-SUP-001', 'NFR-SUP-002', 'NFR-SUP-003', 'NFR-SUP-004'],
    '04-客户端与SDK': ['NFR-SDK-001', 'NFR-SDK-002', 'NFR-SDK-003', 'NFR-SDK-004',
                       'NFR-SE-001', 'NFR-SE-002', 'NFR-OP-006',
                       'NFR-CDN-001', 'NFR-CDN-002', 'NFR-CDN-003', 'NFR-CDN-004', 'NFR-CDN-005'],
    '05-智能决策层': ['NFR-NEURO-001', 'NFR-NEURO-002', 'NFR-NEURO-003', 'NFR-NEURO-004', 'NFR-NEURO-005',
                       'NFR-NEURO-006', 'NFR-NEURO-007', 'NFR-NEURO-008', 'NFR-NEURO-009', 'NFR-NEURO-010',
                       'NFR-OP-001', 'NFR-OP-010', 'NFR-AV-005'],
    '06-测试与质量保障': ['NFR-OP-008', 'NFR-OP-010', 'NFR-OP-005', 'NFR-OP-006',
                       'NFR-EN-003'],
    '07-社交运营与玩家治理': ['NFR-GSM-001', 'NFR-GSM-002', 'NFR-GSM-003', 'NFR-GSM-004', 'NFR-GSM-005', 'NFR-GSM-006',
                       'NFR-OPT-001', 'NFR-OPT-002', 'NFR-OPT-003', 'NFR-OPT-004',
                       'NFR-ANT-001', 'NFR-ANT-002', 'NFR-ANT-003', 'NFR-ANT-004',
                       'NFR-MM-001', 'NFR-MM-002', 'NFR-MM-003', 'NFR-MM-004', 'NFR-MM-005',
                       'NFR-AV-005', 'NFR-AV-009',
                       'NFR-PPL-001', 'NFR-PPL-002', 'NFR-PPL-003', 'NFR-PPL-004',
                       'NFR-DEP-001', 'NFR-DEP-002', 'NFR-DEP-003', 'NFR-DEP-004',
                       'NFR-CAP-001', 'NFR-CAP-002', 'NFR-CAP-003', 'NFR-CAP-004', 'NFR-CAP-005',
                       'NFR-GOV-001', 'NFR-GOV-002', 'NFR-GOV-003', 'NFR-GOV-004', 'NFR-GOV-010', 'NFR-GOV-011', 'NFR-GOV-012', 'NFR-GOV-013', 'NFR-GOV-014', 'NFR-GOV-020', 'NFR-GOV-021', 'NFR-GOV-022', 'NFR-GOV-023', 'NFR-GOV-030', 'NFR-GOV-031', 'NFR-GOV-032', 'NFR-GOV-033', 'NFR-GOV-040']
}

def add_nfr_index(tst_path, nfr_list):
    c = tst_path.read_text(encoding='utf-8')
    if '## NFR 覆盖索引' in c:
        return False, '已修复'
    # 找 "## 7. 风险与未决事项" 或类似位置
    m = re.search(r'##\s*7\.?\s*风险[与与]*未决事项', c)
    if not m:
        return False, '未找到风险章节'
    section = '\n## 6.5 NFR 覆盖索引\n\n'
    section += '本主题域覆盖的非功能需求编号全集（按 RGS-REQ-003 等级 Lv.2/3/4 全覆盖）：\n\n'
    # 按 NFR 前缀分组
    groups = {}
    for nfr in nfr_list:
        prefix = '-'.join(nfr.split('-')[:2]) + '-'
        groups.setdefault(prefix, []).append(nfr)
    for prefix in sorted(groups.keys()):
        ids = groups[prefix]
        section += f'- **{prefix}***：{", ".join(ids)}\n'
    section += '\n'
    new_c = c[:m.start()] + section + c[m.start():]
    tst_path.write_text(new_c, encoding='utf-8')
    return True, f'已补充 {len(nfr_list)} 条 NFR 引用'

def main():
    fixed = skipped = 0
    for topic_dir in sorted(DOCS_ROOT.iterdir()):
        if not topic_dir.is_dir():
            continue
        nfrs = TOPIC_NFR.get(topic_dir.name, [])
        if not nfrs:
            continue
        for tf in sorted(topic_dir.glob('RGS-TST-*.md')):
            ok, msg = add_nfr_index(tf, nfrs)
            sys.stdout.write(f'  [{"OK" if ok else "SKIP"}] {tf.name}: {msg}\n')
            sys.stdout.flush()
            if ok: fixed += 1
            else: skipped += 1
    sys.stdout.write(f'\n=== NFR 索引补充：fixed={fixed}, skipped={skipped} ===\n')

if __name__ == '__main__':
    main()
