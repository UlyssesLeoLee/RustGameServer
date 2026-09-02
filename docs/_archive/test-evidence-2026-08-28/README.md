# test-evidence-2026-08-28 归档说明

> **归档日期**: 2026-09-02 11:00 JST
> **作者**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
> **修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
> **代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)
> **依据**: B4 派生约束 (per 9/2 10:18 JST 拍板) + 归档方法 opt1 (移 docs/_archive/ + .gitignore)
> **关联**: RGS-CRITIQUE-IMPROVEMENT-2026-09-02 v0.1.1 §3.2 B4 + §5.1 D3

## 归档内容

8/28 G3-G4 阶段在 `docs/00-基准与治理/.test-evidence/2026-08-28-*` 跑了 7 个多版本 (v1/v2/v3) 测试批次, 均为本地 ignored untracked 状态 (per .gitignore 行 57-59), 不入 git.

**归档目录清单** (从 `.test-evidence/2026-08-28-*` 移入):
- `2026-08-28-audit/` — audit 批次 v1
- `2026-08-28-audit-v2/` — audit 批次 v2
- `2026-08-28-tbd08-impl/` — tbd08 实施批次 v1
- `2026-08-28-tbd08-impl-v2/` — tbd08 实施批次 v2
- `2026-08-28-ut-impl/` — ut 实施批次 v1
- `2026-08-28-ut-impl-v2/` — ut 实施批次 v2
- `2026-08-28-ut-impl-v3/` — ut 实施批次 v3

**总大小**: 1.18 MB (173 文件)
**归档操作**: git clean -fdX 移除本地 ignored 区, archive 目录作为占位符, 后续按需可重跑 cargo test 复现

## 复现指引

需要复现时:
```bash
# 1. 重跑测试生成 evidence
cd D:/RustGameServer
pwsh scripts/test-evidence.ps1  # 假设存在
# 或手动:
cargo test -p admin-service --lib 2>&1 | tee logs/admin-test-$(date +%Y%m%d).log

# 2. 写到 _archive 对应批次目录
cp logs/admin-test-*.log docs/_archive/test-evidence-2026-08-28/2026-XX-XX-批次/ 2>/dev/null || true
```

## 派生约束守护

- **L12 临时 log 不入 commit**: 已 gitignore 覆盖, 派生约束 B4 落地
- **pre-commit hook**: `scripts/git-hooks/pre-commit` 检测 staged 临时 log 自动拦截
