# RGS-IMPL-006 CI_持续集成规范（CI）

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-IMPL-006 |
| 版本 | 0.1（占位，per 150 工程审计缺失项补全）|
| 依据 | RGS-WF-001 v0.5 §2 150 工程 58 |
| 状态 | 🔴 **占位 NO-GO**（per RGS-PLAN-001 v0.8 §3.3 + RGS-EXEC-001 v0.3）|
| 责任人 | Ulysses（一人公司 12 角色兼任 per DEC-008）|
| 关联 150 工程 | 58 |
| 父文档 | [RGS-IMPL-001 实施约定与工程边界](RGS-IMPL-001_实施约定与工程边界.md) |

---

## 1. 文档目的

本文件是 RGS-WF-001 v0.5 §2 150 工程审计中**缺失 RGS 引用**的 IMPL（CI）类工程占位文档。

**补全范围**：覆盖 150 工程编号 58（CI 工程）。

**特别说明**（per DEC-008）：GitHub Actions 4 workflow（rust-ci/docs-ci/verify-docs-ci/docker-build）。

---

## 2. 规范范围

### §2.1 输入

- RGS-IMPL-001 实施约定与工程边界（**基线**）
- RGS-TS-001 主要技术选型报告
- Cargo workspace + 5 域服务

### §2.2 输出

- 编码 / 静态分析 / 代码审查 / 构建 / CI 的具体规范
- 工具链配置（CI yaml + pre-commit hook + Makefile）

### §2.3 责任人

| 角色 | 责任人 | 签字日 |
|---|---|---|
| CI Lead | Ulysses（一人公司 12 角色兼任 per DEC-008）| 2026-08-21 |
| 评审 | Ulysses（架构师兼）| 2026-08-21 |

---

## 3. 工具链

- **Rust 1.98 stable**（GA 已发，G-CODE-06 待实测）
- **cargo workspace**（per RGS-IMPL-001 §2）
- **5 域独立 service**（player / economy / match / social / admin + cluster-ops + shared-platform）
- **CI 平台**：GitHub Actions（per `docs/deploy/04-ci-cd/`）

---

## 4. 文档结构（待 NO-GO 解除后填充）

```markdown
# §1 规范总则
# §2 工具链配置
# §3 流程定义
# §4 验收标准
# §5 与其他规范的关系
# §6 签字栏
```

---

## 5. 激活条件

🔴 → 🟢：

1. **G-CODE-06** Rust 1.98 + cargo build + cargo test 实际跑过全绿
2. **G-CODE-03** 5 独立 DB 拓扑图实际画过
3. NO-GO 完全解除

满足后由 Ulysses 升 v0.2 填入实际规范内容。

---

## 6. 关联文档

- 父文档：RGS-IMPL-001 实施约定与工程边界
- 上游：RGS-WF-001 v0.5 §2 150 工程 / RGS-TS-001 v0.6
- 部署：docs/deploy/04-ci-cd/（CI workflow 占位）
- worktree：每条 150 工程可单独 worktree 分支执行（per RGS-WT-001）
