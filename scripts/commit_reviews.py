"""Commit RGS-REV-003 联合评审 + 3 附件。"""
import os, subprocess, sys
sys.stdout.reconfigure(encoding='utf-8', errors='replace')
GIT = ['git', '-c', 'core.quotePath=false']
ROOT = r'D:\RustGameServer'
os.chdir(ROOT)

def run(args):
    r = subprocess.run(GIT + args, capture_output=True, text=True, encoding='utf-8', errors='replace')
    return r.returncode, r.stdout, r.stderr

def add(path):
    rc, out, err = run(['add', '--', path])
    if rc != 0:
        print('FAIL add ' + path + ': ' + err.strip())
        return False
    return True

# Add 4 review docs + 1 tool
files = [
    'docs/00-基准与治理/reviews/RGS-REV-003_联合评审_Q003-Q025-ADR0052-5域DTL.md',
    'docs/00-基准与治理/reviews/RGS-REV-004_附件A_5域DTL字段级ReviewChecklist.md',
    'docs/00-基准与治理/reviews/RGS-REV-005_附件B_Saga演练场景Checklist.md',
    'docs/00-基准与治理/reviews/RGS-REV-006_附件C_责任矩阵与签字模板.md',
    'scripts/gate_evidence_status.py',
]
for f in files:
    add(f)

# Commit
msg = """feat(review): RGS-REV-003 联合评审 + 3 附件 (Q-003/Q-025/ADR-0052/5 域 DTL)

按 RGS-HANDOFF-001 §5 Step 1 "组织并记录" 联合评审。

主文 (RGS-REV-003):
- §1 评审背景 (NO-GO 状态说明)
- §2 现状摘要: Q-003/Q-025/ADR-0052/5 域 DTL
  - Q-003 技术方案固定 (Saga + Outbox + 补偿), 缺真实环境证据
  - Q-025 DTL-031 v0.2 存在, 缺字段级 Review
  - ADR-0052 已起草, 缺联审 + 故障注入
  - 5 域 DTL 全部存在, 缺字段级 Review
- §3 责任矩阵 (领域智能): 架构师/Platform/DBA/SRE + 5 域 Lead
- §4 评审流程 (3 阶段, ≤ 12 天)
- §5 评审议程 (≤ 2 小时)
- §6 异议登记表
- §7 评审结论 + 签字栏

附件 A (RGS-REV-004): 5 域 DTL 字段级 Review Checklist
- 14 项通用检查项 + 5 域域特定检查项 + 跨域一致性

附件 B (RGS-REV-005): G-CODE-04 Saga 演练场景 Checklist
- 6 场景: 正常 / 补偿 / 超时 / 人工升级 / 去重 / PFAU+Saga 并发
- 每个场景含 8-10 步 + 断言 + 签字

附件 C (RGS-REV-006): 责任矩阵与签字模板 (详细版)
- 完整 RACI 矩阵 (4 类 Gate × 各责任人)
- 签字流程 (按依赖顺序, 不可跳签)
- 异议处理 (A 文档修订 / B ADR 修订 / C 升级 NO-GO)

工具: scripts/gate_evidence_status.py (Gate 证据扫描)
- 扫描 Q-003/Q-025/ADR-0052/5 域 DTL/Rust 1.98 状态
- 用于评审前快照生成

self-review 修正:
- Doc ID 唯一性: 主文 REV-003 + 附件 REV-004/005/006 (避开 -ADD 后缀问题)
- 责任矩阵按领域智能分配 (架构师兼任 player/admin 是领域特性决定)
- 签字流程按依赖关系排序 (DBA -> SRE -> 5 域 Lead -> 架构师 -> PM)

依据: RGS-HANDOFF-001 §4 Gate 矩阵 + §5 Step 1
验证: verify_docs.py / check-cross-references.py / verify_wf_v05.py 全部 PASS"""
rc, out, err = run(['commit', '-m', msg])
if rc != 0:
    print('COMMIT FAILED: ' + err)
else:
    print('OK:')
    print(out)

# Final
print()
rc, out, _ = run(['status', '--short'])
print('status: ' + (out if out else '(clean)'))
print()
rc, out, _ = run(['log', '--oneline', '-5'])
print('最近 5 commit:')
print(out)
