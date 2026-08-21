"""Commit RGS-QA-001 v0.7 + 2 link fixes。"""
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

def rm(path):
    rc, out, err = run(['rm', '--', path])
    if rc != 0:
        print('FAIL rm ' + path + ': ' + err.strip())
        return False
    return True

# add new v0.7
add('docs/11-实施QA/RGS-QA-001_实施前QA表_v0.7.md')
# remove old v0.6
rm('docs/11-实施QA/RGS-QA-001_实施前QA表_v0.6.md')
# add the 2 link fix docs
add('docs/00-基准与治理/RGS-HANDOFF-001_实施前文档基线交接.md')
add('docs/00-基准与治理/RGS-REV-002_九阶段工作流最终审核报告.md')
add('docs/README.md')

msg = """feat(qa): RGS-QA-001 v0.7 (Q-021 治理闭环落地 + Q-027/Q-031 主题重定义)

按用户决策起草 3 项更新, 同步 handoff §5 Step 1 进展。

Q-021 治理闭环 (v0.7 候选答案落地):
- 状态: 🟣 候选答案已落地执行, 待具名人类确认
- 已执行 9 个 commit (2bb4a77 / c4e83fa / 6043b5a / f198270 / 7690852 / 
  8b55916 / 00cabcd / 67aea03 / dedcc1e) + 3 评审/模板 commit
- 涵盖: 74 docs + 17 scripts + 3 infra + 36 SPECs + 10 核心升版 + handoff 
  + IMPL + 评审 + ENV 模板
- 验证: git clean / 3 verify scripts 全 PASS / +260 -36 行
- 遗留: 41 M 文件 (其他 session) + DTL-031 跨版本说明

Q-027 主题重定义: 文档版本与代码版本同步策略
- 旧: 多域 DTL 未决项管理
- 新: 三层同步 (L1 文档↔文档 / L2 文档↔代码 / L3 文档↔部署制品)
- 工具: verify_docs.py / check-cross-references.py / 新增 check_doc_code_sync.py 草案
- 截止: PH-2 前 (CI 门禁启用)

Q-031 主题重定义: WBS 工作分解结构
- 旧: tonic 0.10+ 与 hyper 1.x 升级窗口
- 新: 5 层 WBS (L1 阶段 / L2 域 / L3 任务簇 / L4 任务 / L5 工作包)
- 资源: 5 域 Lead + Platform + DBA + QA + PM = 9 人, OLU 22 人·天/周
- 截止: PH-0.5 前完成 L3

§9.1 总览表更新:
- §6 文档与治理: 4 → 1 已落地 + 3 待审 (Q-021 落地)
- 总数 35 不变, 🟣 已落地 1 / 待审 34

配套 fix (避免悬空链接):
- RGS-HANDOFF-001: v0.6 → v0.7 引用
- RGS-REV-002: v0.6 → v0.7 引用
- docs/README.md: v0.6 → v0.7 引用
- 删除 v0.6 文件 (per 版本管理惯例)

验证: verify_docs.py / check-cross-references.py / verify_wf_v05.py 全 PASS"""
rc, out, err = run(['commit', '-m', msg])
if rc != 0:
    print('COMMIT FAILED: ' + err)
else:
    print('OK:')
    print(out)
