"""执行 5 commit 计划。"""
import os, subprocess, sys
sys.stdout.reconfigure(encoding='utf-8', errors='replace')
GIT = ['git', '-c', 'core.quotePath=false']
ROOT = r'D:\RustGameServer'
os.chdir(ROOT)

def run(args, cwd=None):
    r = subprocess.run(GIT + args, capture_output=True, text=True, encoding='utf-8', errors='replace', cwd=cwd or ROOT)
    return r.returncode, r.stdout, r.stderr

def add(path):
    rc, out, err = run(['add', '--', path])
    if rc != 0:
        print('  FAIL add ' + path + ': ' + err.strip())
        return False
    return True

def commit(msg):
    rc, out, err = run(['commit', '-m', msg])
    if rc != 0:
        print('  FAIL commit: ' + err.strip())
        return False
    lines = out.strip().splitlines()
    print('  OK: ' + (lines[0] if lines else out.strip()))
    return True

# 获取 M 文件清单
rc, out, err = run(['status', '--short'])
m_files = []
for line in out.splitlines():
    if line.startswith(' M'):
        m_files.append(line[3:].strip())

# ============ Commit 1: 37 SPECs ============
print('=== Commit 1: 37 SPECs ===')
spec_files = [f for f in m_files if '13-实现规格/RGS-SPEC' in f]
print('  adding ' + str(len(spec_files)) + ' files...')
ok = True
for f in spec_files:
    if not add(f):
        ok = False; break
if ok:
    msg1 = """feat(specs): finalize 36 份 RGS-SPEC-DTL + RGS-SPEC-000 (实施规格化基线)

按 RGS-HANDOFF-001 §3 "36 份 DTL → SPEC" 落地，将每份 DTL 转为可执行的
实现规格 (目标基线 / 实现单元 / 契约 / 可观测性 / 安全/测试 / DoD / Gate 证据)。

包含:
- RGS-SPEC-000 详细设计规格化总表 (索引 + 跨 SPEC 约定)
- RGS-SPEC-DTL-001..009 (核心架构域)
- RGS-SPEC-DTL-011..027 (运维/数据/客户端/智能体/热交/玩家治理域)
- RGS-SPEC-DTL-031..040 (COC + Agent + 5 域 first slice)

self-review 修正 (amend 同 commit):
- 与 RGS-IMPL-001 工程边界一致
- 与 RGS-QA-001 v0.6 Gate 证据一致
- 5 域 first slice (player/economy/match/social/admin) SPEC 同步"""
    commit(msg1)

# ============ Commit 2: 8 核心文档 + README ============
print()
print('=== Commit 2: 8 核心文档 + README ===')
core_files = [f for f in m_files if f not in spec_files and not f.startswith('scripts/')]
if 'docs/README.md' in m_files and 'docs/README.md' not in core_files:
    core_files.append('docs/README.md')
print('  files: ' + str(len(core_files)))
for f in core_files:
    print('    ' + f)

ok = True
for f in core_files:
    if not add(f):
        ok = False; break
if ok:
    msg2 = """feat(docs): RGS 核心文档基线升版 (10 份 + README)

版本升级:
- RGS-REV-002 v0.1 -> v0.2 (九阶段工作流最终审核报告)
- RGS-BAS-002 v0.2 -> v0.3 (功能挂载架构)
- RGS-DTL-031 v0.1 -> v0.2 (集群运营中心与每功能原子升级)
- RGS-REQ-006 v0.1 -> v0.2 (功能挂载架构)
- RGS-ADR-0052 容错哲学 (Active-Active + all-reachable PFAU)
- RGS-OPS-001 v0.2 -> v0.3 (保姆级部署说明)
- RGS-TS-001 v0.3 -> v0.4 (主要技术选型报告)
- RGS-PLAN-001 v0.2 -> v0.3 (项目实施计划)
- RGS-GOBS-001 (现有可观测性调查) 状态修正
- RGS-GOBS-003 (可观测性基本设计) 状态修正
- RGS-GOBS-004 v0.1 -> v0.2 (Observability 导入计划)
- docs/README.md (索引更新)

与 RGS-HANDOFF-001 §3 资产对齐:
- QA v0.6 / PLAN v0.3 / WF v0.5 / OPS v0.3 / GOBS-004 v0.2"""
    commit(msg2)

# ============ Commit 3: 3 scripts ============
print()
print('=== Commit 3: 3 scripts ===')
scr_files = [f for f in m_files if f.startswith('scripts/')]
print('  files: ' + str(len(scr_files)))
for f in scr_files:
    print('    ' + f)
ok = True
for f in scr_files:
    if not add(f):
        ok = False; break
if ok:
    msg3 = """chore(tools): v0.3 文档基线配套 Python 脚本

- build_wf_v03_legacy.py: 早期 WF-001 v0.2 -> v0.3 生成器 (已被 build_wf_v03.py 替代)
- commit_docs_batch.py: 74 份 RGS 文档批量入仓 (docs/scripts/infra 三组分类)
- recommit_3_steps.py: reset --soft HEAD~3 后 3 步重提交 (用于 amend 误入他 commit)

注: 这些是 v0.3 收尾阶段的临时工具，与 v0.4/v0.5 配套脚本并列保留。"""
    commit(msg3)

# ============ Commit 4: handoff ============
print()
print('=== Commit 4: handoff ===')
handoff = 'docs/00-基准与治理/RGS-HANDOFF-001_实施前文档基线交接.md'
print('  file: ' + handoff)
if add(handoff):
    msg4 = """docs(handoff): RGS-HANDOFF-001 实施前文档基线交接 v0.1

本交接明确:
- 53 開発環境構築 仍为 NO-GO (具名审批 + 真实环境验证 + 容量/阈值实测 + 责任人风险接受)
- Q-101~Q-405 全部冻结于 RGS-IMPL-001 (工程约定真源)
- 36 份 DTL → SPEC 已对齐
- 4 类 Gate 证据要求 (Q-025/G-CODE-02/03/04/05/06/07) 列明

依据: RGS-HANDOFF-001 §6 验证清单
- verify_docs.py PASS
- check-cross-references.py PASS
- verify_wf_v05.py PASS"""
    commit(msg4)

# ============ Commit 5: IMPL-001 ============
print()
print('=== Commit 5: IMPL-001 ===')
impl = 'docs/13-实现规格/RGS-IMPL-001_实施约定与工程边界.md'
print('  file: ' + impl)
if add(impl):
    msg5 = """feat(impl): RGS-IMPL-001 实施约定与工程边界

Q-101~Q-405 候选方案冻结为工程约定真源，覆盖:
- Workspace / crate 布局 (resolver=3, crates/* + services/*)
- ClusterOps / proto / migration 边界 (独立控制面)
- 错误 / 一致性 / 测试 (thiserror + Outbox + Saga, no 2PC/XA)
- CI / lock (Cargo.lock 入仓, --locked)
- 运行时 / 安全 (Tokio + mimalloc + Figment + secrecy)
- 发布 / 观测 (distroless + OTel + Prometheus)
- 版本 (Rust 1.98 stable, Actix 4.14.1, PostgreSQL 18.4)

与 RGS-HANDOFF-001 §2 "已冻结的工程约定" 双向绑定."""
    commit(msg5)

# 最终状态
print()
print('=== 最终状态 ===')
rc, out, _ = run(['status', '--short'])
print(out if out else '  (clean)')
print()
rc, out, _ = run(['log', '--oneline', '-8'])
print('最近 8 commit:')
print(out)
