# RGS-PM-002 Issue管理（Issue）

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-PM-002 |
| 版本 | 0.1（占位，per 150 工程审计缺失项补全）|
| 依据 | RGS-WF-001 v0.5 §2 150 工程 134 |
| 状态 | 🔴 **占位 NO-GO**（per RGS-PLAN-001 v0.8 §3.3 + RGS-EXEC-001 v0.3）|
| 责任人 | Ulysses（一人公司 12 角色兼任 per DEC-008）|
| 关联 150 工程 | 134 |

---

## 1. 文档目的

本文件是 RGS-WF-001 v0.5 §2 150 工程审计中**缺失 RGS 引用**的 PM（Issue管理）工程的占位文档。

**补全范围**：覆盖 150 工程编号 134。

**特别说明**（per DEC-008）：Jira/GitHub Issues/Linear 任一平台；一人公司用 GitHub Issues 即可。

---

## 2. PM 流程

### §2.1 输入

- 上一阶段产出物
- Issue 列表（GitHub Issues）
- 进度报告（per RGS-WBS-001 v0.3 L4 任务状态）

### §2.2 输出

- PM 记录（决策日志 / 进度跟踪 / 经验教训）
- 签字文档（一人公司 = Ulysses 自审自批，流程化补偿：CI + 自动化）

### §2.3 责任人

| 角色 | 责任人 | 签字日 |
|---|---|---|
| PM | Ulysses（一人公司 12 角色兼任 per DEC-008）| 2026-08-21 |
| 评审 | Ulysses（架构师兼）| 2026-08-21 |

---

## 3. 工具链

- **GitHub Issues**：缺陷 / 任务跟踪
- **GitHub Projects**：看板（WBS 任务看板视图）
- **GitHub Releases**：版本发布（Archive 用途）
- **GitHub Actions**：CI（per RGS-EXEC-001 v0.3 §3.4 G-CODE-06）

---

## 4. 文档结构（待 NO-GO 解除后填充）

```markdown
# §1 流程定义
# §2 输入与输出
# §3 工具链配置
# §4 报告模板
# §5 签字栏
# §6 历史记录
```

---

## 5. 激活条件

🔴 → 🟢：

1. **G-CODE-06** Rust 1.98 + cargo build + cargo test 实际跑过全绿
2. **G-CODE-03** 5 独立 DB 拓扑图实际画过
3. NO-GO 完全解除

满足后由 Ulysses 升 v0.2 填入实际 PM 流程与工具链配置。

---

## 6. 关联文档

- 上游：RGS-WF-001 v0.5 §2 150 工程 / RGS-PLAN-001 v0.8 §3 / RGS-EXEC-001 v0.3
- 同类：RGS-PM-001~009（9 个 PM 文档）
- 主体：RGS-WBS-001 v0.3 瀑布式 WBS（PM 流程承载）
- worktree：每条 PM 任务可单独 worktree 分支执行（per RGS-WT-001）
