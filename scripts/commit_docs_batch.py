#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""分 3 个 commit 入仓 untracked 文件（docs / scripts / infra）。

使用 git -c core.quotePath=false 确保非 ASCII 路径以 UTF-8 输出。
"""
import os, subprocess, sys

ROOT = r'D:\RustGameServer'
os.chdir(ROOT)

GIT_Q = ['git', '-c', 'core.quotePath=false']

def run_git(args, check=True):
    r = subprocess.run(GIT_Q + args, capture_output=True, text=True, encoding='utf-8', errors='replace')
    if check and r.returncode != 0:
        print('FAIL:', ' '.join(GIT_Q + args))
        print('STDOUT:', r.stdout)
        print('STDERR:', r.stderr)
        sys.exit(1)
    return r.stdout

def untracked():
    return [l for l in run_git(['ls-files', '--others', '--exclude-standard']).splitlines() if l]

def status_short():
    return [l for l in run_git(['status', '--short']).splitlines() if l]

def git_add(path):
    p = path.replace(os.sep, '/')
    r = subprocess.run(
        GIT_Q + ['add', '--', p],
        capture_output=True, text=True, encoding='utf-8', errors='replace'
    )
    if r.returncode != 0:
        print(f'  FAIL add: {p}')
        print(f'  STDERR: {r.stderr}')
        sys.exit(3)
    return True

def git_commit(msg):
    r = subprocess.run(
        GIT_Q + ['commit', '-m', msg],
        capture_output=True, text=True, encoding='utf-8', errors='replace'
    )
    if r.returncode != 0:
        print('COMMIT FAILED')
        print('STDOUT:', r.stdout)
        print('STDERR:', r.stderr)
        sys.exit(4)
    out = r.stdout.strip().splitlines()
    return out[0] if out else '(no output)'

# 1) Get untracked
all_un = untracked()
print(f'[1] total untracked: {len(all_un)}')

# 2) Categorize
INF_FILES = ['.gitattributes', '.github/workflows/docs-verify.yml', 'docs/document-registry.toml']
docs = []
scripts = []
infra = []
for f in all_un:
    if f in INF_FILES:
        infra.append(f)
    elif f.startswith('scripts/') and f.endswith('.py'):
        scripts.append(f)
    else:
        docs.append(f)

print(f'[2] categorized:')
print(f'    docs    : {len(docs)}')
print(f'    scripts : {len(scripts)}')
print(f'    infra   : {len(infra)}')

# 3) Check M files don't overlap
m_files = [s[3:].strip() for s in status_short() if s.startswith(' M')]
print(f'[3] M files in working tree: {len(m_files)} (must NOT be staged)')
overlap = [f for f in docs if f in m_files]
if overlap:
    print('    !! overlap detected:')
    for f in overlap:
        print(f'      {f}')
    sys.exit(2)
print('    no overlap with M files - OK')

# === Commit 1: docs ===
print()
print('=== Commit 1: docs ===')
for f in docs:
    git_add(f)
staged = [s for s in status_short() if s.startswith('A ')]
print(f'  staged: {len(staged)} files')

msg1 = """feat(docs): 整合 74 份 RGS 文档入仓 + 13→05 合并重命名

按 RGS-WF-001 v0.2 的 16 阶段 × RGS 文档体系映射归类入仓。

各域入库内容:
- 00-基准与治理: +3 (TST-UT/IT/ST-00 测试设计书)
- 01-核心架构与设计模式: +7 (REQ-025 addendum 集群内分片 + TST-01 + addendum)
- 02-运维安全与网络: +3 (TST-02)
- 03-数据经济与交易: +3 (TST-03)
- 04-客户端与SDK: +7 (REQ-030 addendum CDN + TST-04 + addendum)
- 05-智能体与Agent: +21 (合并 13-智能体与Agent工程 12 个 + 原有 9 个; 重命名)
- 06-测试与质量保障: +3 (TST-06)
- 07-社交运营与玩家治理: +7 (REQ-028 addendum DSL + TST-07 + addendum)
- 08-架构决策记录: +3 (ADR-0052/0053/0054)
- 09-部署运维: +1 (RGS-OPS-001 保姆级部署说明)
- 11-实施QA: +1 (RGS-QA-001 v0.4 实施前 QA 表)

目录治理:
- 13-智能体与Agent工程 (12 个文件) → 合并到 05
- 05-智能决策层 → 重命名为 05-智能体与Agent
- 13 目录标记为 .merged-to-05 (git 不追踪空目录)

后续 commit: scripts/*.py 单独提交, 基础设施单独提交。"""
r1 = git_commit(msg1)
print(f'  committed: {r1}')

# === Commit 2: scripts ===
print()
print('=== Commit 2: scripts ===')
for f in scripts:
    git_add(f)
staged = [s for s in status_short() if s.startswith('A ')]
print(f'  staged: {len(staged)} files')

msg2 = """feat(scripts): 入仓 17 个文档处理 Python 脚本

工具集:
- add_nfr_index.py: NFR 文档索引生成
- add_toc.py: 自动生成 TOC
- batch_revise_tst.py: 批量修订 TST 文档
- finalize_tbd.py / finalize_tst.py / finalize_topic00.py: 收尾工具
- fix_tst_coverage.py: TST 覆盖率检查
- ipa_dedupe.py / ipa_dedupe2.py / ipa_finalize.py / ipa_fix2.py / ipa_normalize.py: IPA 标准化
- renumber.py: 文档重编号
- st_ac_complete.py: ST 用例完整性检查
- verify_docs.py: 文档结构验证
- reorg_05_13.py: 13-智能体与Agent工程 → 05-智能体与Agent 一次性迁移脚本

注: 这是 1.0 收尾阶段的工具集; 正式 CI 阶段会替换为持续集成 pipeline。"""
r2 = git_commit(msg2)
print(f'  committed: {r2}')

# === Commit 3: infra ===
print()
print('=== Commit 3: infrastructure ===')
for f in infra:
    git_add(f)
staged = [s for s in status_short() if s.startswith('A ')]
print(f'  staged: {len(staged)} files')

msg3 = """chore(infra): 仓库基础设施配置

- .gitattributes: 行尾与编码规则
- .github/workflows/docs-verify.yml: 文档 CI 验证
- docs/document-registry.toml: 文档注册表 (RGS-* 编号 ↔ 路径映射)"""
r3 = git_commit(msg3)
print(f'  committed: {r3}')

# Final
print()
print('=== Final git status ===')
print('\n'.join(status_short()))
print()
print('=== Recent 6 commits ===')
print(run_git(['log', '--oneline', '-6']))
