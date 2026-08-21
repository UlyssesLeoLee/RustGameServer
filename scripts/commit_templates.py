"""Commit RGS-ENV-001 + 2 沟通模板。"""
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

# 3 templates
files = [
    'docs/00-基准与治理/reviews/RGS-ENV-001_环境核验记录模板.md',
    'docs/00-基准与治理/reviews/签字提案邮件_模板.md',
    'docs/00-基准与治理/reviews/评审会议议程通知_模板.md',
    'scripts/commit_templates.py',
]
for f in files:
    add(f)

msg = """feat(templates): RGS-ENV-001 + 2 沟通模板 (handoff §5 Step 2/3)

按用户决策起草 3 份配套模板, 用于 53 開発環境構築 启动前的 Gate 闭环。

RGS-ENV-001 环境核验记录模板 (handoff §5 Step 2):
- §0 核验元数据 (日期/操作人/环境/节点)
- §1 工具链核验 (rustc 1.98 / cargo / clippy / rustfmt / sqlx-cli / cargo-deny 等)
- §2 PostgreSQL 18.4 核验 (psql / 服务器 / 5 DB / sqlx 编译期 / migration 双向演练)
- §3 K3s / Kubernetes 核验 (kubectl / 节点 / CoreDNS / Helm / 镜像仓库)
- §4 锁定依赖 CI 核验 (--locked / fmt / clippy -D / deny / test / audit)
- §5 跨工具集成 (sqlx 编译期 + tonic gRPC + distroless 容器)
- §6 签字栏 (Platform + DBA + SRE + 架构师 + PM)
- §7 异常处理 + §8 核验完成声明

签字提案邮件模板 (Step 3 沟通材料):
- 评审目的 + 现状摘要 + 评审时间表
- 各责任人具体责任 (按 RGS-REV-003 §3)
- 3 种签字方式 (手写 / Git GPG / PKI)
- 截止日期 + 配套文档清单 + 异议登记流程
- 发送前 checklist + 发送后跟踪

评审会议议程通知模板 (Step 3 沟通材料):
- 会议基本信息 (时间/地点/会议ID)
- 必到人员清单
- 2 小时议程 (7 段, 硬上限)
- 预读材料清单 (7 份)
- 会前 checklist (主持人 + 责任人)
- 会议输出 (异议表 / 纪要 / 签字栏 / 闭环计划)
- 会议后 24h 行动项
- 通知邮件模板
- 异常处理 (5 种情况)

self-review 修正 (amend 同 commit):
- 邮件模板的截止日期用 YYYY-MM-DD 占位, 启动时由主持人填入
- 议程通知的会议时间硬编码 14:00-16:00 (UTC+9), 与 RGS-REV-003 §5 一致
- 异常处理表覆盖 5 种典型情况 (缺席/中断/超时/未闭环/全员拒绝)

依据: RGS-HANDOFF-001 §5 Step 2 (环境核验) + Step 3 (签字)
验证: verify_docs.py PASS (222 Markdown) + check-cross-references.py PASS"""
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
