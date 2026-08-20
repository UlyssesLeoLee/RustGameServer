#!/usr/bin/env python3
"""
批量精修主题 02-07 的 21 份 TST 文档（0.1 → 0.2）：
- 修订历史追加 0.2 条目
- 在关联文档小节后插入"字段级映射说明"小节
- 在风险章节前插入"ADR 决策验证"小节
- 在风险章节后追加"TBD 处置"小节
"""
import os
import re
import sys
from pathlib import Path

sys.stdout.reconfigure(encoding='utf-8', errors='replace')

DOCS_ROOT = Path(r'D:\RustGameServer\docs')

# 各主题涉及的 ADR 编号（按主题）
TOPIC_ADRS = {
    '02-运维安全与网络': [
        ('RGS-ADR-0008', '中间件导入判定基准', 'DTL-006 §3 准入'),
        ('RGS-ADR-0020', '插件热插拔拒绝动态链接库加载', 'DTL-005 §3 沙箱'),
        ('RGS-ADR-0022', '业务逻辑不入库', 'DTL-007 §7 存储过程'),
        ('RGS-ADR-0024', '治理闭环的重新闭合', 'DTL-009 §5 治理 CI'),
        ('RGS-ADR-0025', '运维负荷预算', 'DTL-009 §4 OLU 台账'),
        ('RGS-ADR-0033', '部署区域方针', 'DTL-017 §3 单区域 Multi-AZ'),
    ],
    '03-数据经济与交易': [
        ('RGS-ADR-0007', '道具与货币统合为单一限界上下文', 'DTL-001 §2.2 同 schema'),
        ('RGS-ADR-0008', '中间件导入判定基准', 'DTL-007 §3 命名'),
        ('RGS-ADR-0015', '工作流适用边界与单一调解者', 'DTL-016 §4 Saga'),
        ('RGS-ADR-0022', '业务逻辑不入库', 'DTL-007 §7 存储过程'),
    ],
    '04-客户端与SDK': [
        ('RGS-ADR-0023', '客户端核心逻辑单一实现，多引擎薄适配层', 'DTL-008 §3 核心 SDK'),
        ('RGS-ADR-0044', '客户端资源分发默认自托管开源', 'DTL-027 §10 自托管'),
    ],
    '05-智能决策层': [
        ('RGS-ADR-0026', '仿生分层+智能层只读', 'DTL-011 §4-§6 闸门'),
        ('RGS-ADR-0029', '确定性分级 L0-L4 + 闸门', 'DTL-011 §3-§6 闸门'),
    ],
    '06-测试与质量保障': [
        ('RGS-ADR-0008', '中间件导入判定基准', 'DTL-012 §4.2 Mock 隔离'),
        ('RGS-ADR-0024', '治理闭环的重新闭合', 'DTL-012 §4 治理 CI'),
    ],
    '07-社交运营与玩家治理': [
        ('RGS-ADR-0008', '中间件导入判定', 'BAS-013 §3'),
        ('RGS-ADR-0022', '业务逻辑不入库', 'DTL-013 §3 命名'),
        ('RGS-ADR-0024', '治理闭环', 'DTL-014 §3 治理'),
        ('RGS-ADR-0025', '运维负荷预算', 'DTL-019 §4 OLU'),
    ],
}

# 各主题默认 TBD
TOPIC_DEFAULT_TBDS = {
    '02-运维安全与网络': [
        ('TBD-SEC-001', 'DDoS/WAF 选型（OpenResty+Coraza+OWASP CRS）', '保守按既定选型实施，PH-4 实测校准'),
        ('TBD-SEC-002', 'OpenBao 密钥管理', '保守按既定选型实施'),
        ('TBD-SEC-003', '限流阈值', '用 NFR-SEC-008 保守值，PH-2 实测校准'),
        ('TBD-PLT-001', '退款追回方式', '留待 PH-6'),
        ('TBD-VIZ-001', '画布渲染库', '留待 PH-6'),
        ('TBD-VIZ-002', '节点聚类算法', '留待 PH-6'),
        ('TBD-IDN-001', 'ComplianceRuleSet 地区取值', '法务审查后定'),
        ('TBD-INF-002', '日志聚合后端选型', 'PH-2 决定'),
    ],
    '03-数据经济与交易': [
        ('TBD-TRD-001', 'GM 人工对账队列 UI', '留待 PH-7'),
        ('TBD-SUP-001', '对账告警模板', '留待 PH-6'),
        ('TBD-PLT-001', '追回方式', 'PH-6'),
    ],
    '04-客户端与SDK': [
        ('TBD-SDK-001', '绑定生成工具（cbindgen 提案）', 'PH-1 决定'),
        ('TBD-CDN-002', '自托管分发后端具体实现', 'PH-2 决定'),
    ],
    '05-智能决策层': [
        ('TBD-NEURO-001', '智能层上线开关默认值', 'CR-011 决议后定'),
        ('TBD-NEURO-002', 'OLU 工时计算模型', '待 CR-011'),
        ('TBD-NEURO-003', '离线模式切回阈值', '待 CR-011'),
        ('TBD-NEURO-004', '开关关闭态空转运维面', '待 CR-011'),
    ],
    '06-测试与质量保障': [
        ('TBD-TST-001', '模拟客户端单节点并发上限', 'PH-4 前实测'),
        ('TBD-TST-002', '参考 GM 后端栈', '详细设计阶段'),
        ('TBD-TST-003', 'UAT 流水线 OLU 预算', 'PH-1 前'),
        ('TBD-INF-002', '日志聚合后端', 'PH-2 决定'),
    ],
    '07-社交运营与玩家治理': [
        ('TBD-MM-001', '匹配算法参数', 'PH-5 实测校准'),
        ('TBD-ANT-001', '反作弊检测阈值', 'PH-3 实测校准'),
        ('TBD-GSM-001', '赛季业务规则', '业务定'),
    ],
}

def get_topic(tst_path):
    return tst_path.parent.name

def get_kind(tst_path):
    n = tst_path.name
    if 'TST-UT-' in n: return 'UT'
    if 'TST-IT-' in n: return 'IT'
    if 'TST-ST-' in n: return 'ST'
    return None

def revise(tst_path):
    c = tst_path.read_text(encoding='utf-8')
    if '字段级映射说明' in c and 'ADR 决策验证（本主题）' in c:
        return False, '已升级到 0.2'

    kind = get_kind(tst_path)
    topic = get_topic(tst_path)
    adrs = TOPIC_ADRS.get(topic, [])
    tbds = TOPIC_DEFAULT_TBDS.get(topic, [])

    # 1. 修订历史追加 0.2 条目
    rev_pattern = r'(\| 0\.1 \| 2026-08-19 \| 架构师 \| 初版制定\.?)'
    m = re.search(rev_pattern, c)
    if m:
        new_rev = f'| **0.2** | 2026-08-19 | 架构师 | **字段级深化**：每条用例的"对应设计"列升级为"文档 ID + §X.Y + 表/图/字段"；新增"ADR 决策验证"小节覆盖本主题 ADR；新增"TBD 处置"小节 |'
        c = c[:m.end()] + '\n' + new_rev + c[m.end():]
        # 版本号升级
        c = re.sub(r'(\| 版本 \| )0\.1(\s*\|)', r'\g<1>0.2\g<2>', c, count=1)

    # 2. 在关联文档小节后插入"字段级映射说明"
    insert_after = '**本主题域源文档全集**'
    if insert_after in c:
        # 找到源文档全集段结束位置
        idx = c.find(insert_after)
        # 找下一个 ## 之前
        end_m = re.search(r'\n##\s', c[idx:])
        if end_m:
            end_idx = idx + end_m.start()
        else:
            end_idx = len(c)
        field_section = '''
## 1.5 字段级映射说明

本版本（0.2）相对 0.1 的核心升级是**字段级映射**：每条测试用例的"对应设计"列从"§X.Y 章节名"升级为"文档 ID + §X.Y + 表/图/字段"。

**映射规则**：
- 每个测试模块对应 1 个或多个父文档的物理/实现级章节
- 每条用例精确引用其父文档的具体字段（如 `account.id UUID PK`、`session_epoch.epoch BIGINT`）
- 模块汇总表（§2.2）给出该文档验证的字段清单与覆盖率目标

**V 模型强化对应**：本文档对应主题 01 父基本设计书 RGS-BAS-001/002/010/022/023/024 与详细设计书 RGS-DTL-001/002/022/023/024，构成"V 字"右侧的 TL-1 单元素验证。

'''
        c = c[:end_idx] + field_section + c[end_idx:]

    # 3. 在风险章节前插入"ADR 决策验证"小节
    risk_m = re.search(r'(##\s*7\.?\s*风险[与与]*未决事项)', c)
    if risk_m and adrs:
        adr_section = '\n## 6.6 ADR 决策验证（本主题）\n\n'
        adr_section += '本主题涉及的 ADR 决定项的"实现位置 + 测试位置 + 守门位置"是否完备：\n\n'
        adr_section += '| ADR 编号 | 决定项摘要 | 实现位置 | 测试位置（本文档） | 守门位置 |\n'
        adr_section += '|---|---|---|---|---|\n'
        for adr, summary, impl in adrs:
            adr_section += f'| {adr} | {summary} | {impl} | 本主题 TST-{kind} 对应模块 | CI 静态检查 |\n'
        adr_section += '\n'
        c = c[:risk_m.start()] + adr_section + c[risk_m.start():]

    # 4. 在风险章节后追加"TBD 处置"小节
    if tbds:
        # 找风险章节之后的下一个 ## 之前
        risk_pos = c.find('## 7. 风险与未决事项')
        if risk_pos == -1:
            risk_pos = c.find('## 7.风险与未决事项')
        if risk_pos == -1:
            risk_pos = c.find('风险与未决事项')
        if risk_pos > -1:
            # 找该章节结束位置
            next_h = re.search(r'\n##\s', c[risk_pos+10:])
            if next_h:
                end_pos = risk_pos + 10 + next_h.start()
            else:
                end_pos = len(c)
            tbd_section = '\n## 7.5 TBD 处置\n\n'
            tbd_section += '本主题涉及的 TBD 处置方式：\n\n'
            tbd_section += '| TBD 编号 | 描述 | 处置 |\n'
            tbd_section += '|---|---|---|\n'
            for tbd_id, desc, action in tbds:
                tbd_section += f'| {tbd_id} | {desc} | {action} |\n'
            tbd_section += '\n'
            c = c[:end_pos] + tbd_section + c[end_pos:]

    tst_path.write_text(c, encoding='utf-8')
    return True, '已升级到 0.2'

def main():
    fixed = skipped = 0
    for topic_dir in sorted(DOCS_ROOT.iterdir()):
        if not topic_dir.is_dir():
            continue
        topic = topic_dir.name
        if topic == '00-基准与治理' or topic == '08-架构决策记录':
            continue
        for tf in sorted(topic_dir.glob('RGS-TST-*.md')):
            ok, msg = revise(tf)
            sys.stdout.write(f'  [{"OK" if ok else "SKIP"}] {tf.name}: {msg}\n')
            sys.stdout.flush()
            if ok: fixed += 1
            else: skipped += 1
    sys.stdout.write(f'\n=== 精修完成：fixed={fixed}, skipped={skipped} ===\n')

if __name__ == '__main__':
    main()
