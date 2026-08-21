# RGS-SPEC-CROSS-003 跨域事件 Schema 字典（Cross-Domain Event Schema Registry）

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-SPEC-CROSS-003 |
| 版本 | 0.1（占位，per WBS v0.3 §2A.6.7 横向规范补全 2026-08-21）|
| 依据 | RGS-WBS-001 v0.3 §2A.6.7（5 域 DTL §3 实现契约横向规范）|
| 状态 | 🔴 **占位 NO-GO**（per RGS-PLAN-001 v0.8 §3.3 + RGS-EXEC-001 v0.3）|
| 责任人 | Ulysses（一人公司 12 角色兼任 per DEC-008）|
| 关联 150 工程 | 54（编码实现）|
| 父文档 | [RGS-SPEC-000 详细设计规格化总表](RGS-SPEC-000_详细设计规格化总表.md) |
| 强制并行 | RGS-SPEC-CROSS-002 Proto 风格指南 |

---

## 1. 文档目的

本文件是**中心事件管理（CEM，per RGS-ADR-0051）**的事件主题命名空间 + payload 模板的横向规范，per WBS v0.3 §2A.6.7 "补救❶ 跨 DTL 接口契约不一致"。

**核心问题**：CEM 事件跨 5 域发布，事件主题命名 / payload schema 各自定义 → 订阅方无法可靠订阅 / 版本管理混乱。

**解决方式**：建立全局事件 schema 字典，强制所有事件主题 + payload 引用本字典。

---

## 2. 规范范围

### §2.1 输入

- 5 域 DTL §3 实现契约
- RGS-ADR-0051 中心事件管理（CEM）
- RGS-SPEC-DTL-031 admin 域（COC + CEM + PFAU）
- 业务事件分类（域内事件 / 跨域事件 / 集群事件）

### §2.2 输出

- 事件主题命名空间（`rgs.events.<domain>.<aggregate>.<action>.<version>`，如 `rgs.events.economy.wallet.committed.v1`）
- 事件 payload 模板（CloudEvents 1.0 兼容）
- 事件 schema 版本管理（major.minor 语义化）
- 事件订阅者注册表（per DTL-031 §3 event_producer_registry）
- 事件回放 / 重放 / 死信处理规范
- 事件去重 / 幂等 key 规范

### §2.3 责任人

| 角色 | 责任人 | 签字日 |
|---|---|---|
| cluster-ops Lead | Ulysses（一人公司 12 角色兼任 per DEC-008）| 2026-08-21 |
| Platform Lead | Ulysses（一人公司 12 角色兼任 per DEC-008）| 2026-08-21 |
| 5 域 Lead 各 1 名 | Ulysses（一人公司 12 角色兼任 per DEC-008）| 2026-08-21 |
| 评审 | Ulysses（架构师兼）| 2026-08-21 |

---

## 3. 工具链

- **CEM**（per RGS-ADR-0051，admin 域 `event_schema_registry` 表）
- **CloudEvents SDK**（Rust：`cloudevents` crate）
- **tonic gRPC**（事件发布流）
- **OpenTelemetry**（事件 trace 传播，per CROSS-006）

---

## 4. 文档结构（待 NO-GO 解除后填充）

```markdown
# §1 事件主题命名空间规范
# §2 事件 payload 模板（CloudEvents 兼容）
# §3 事件 schema 版本管理（语义化版本 + 兼容性约束）
# §4 域内事件清单（5 域各自常用事件）
# §5 跨域事件清单（Q-003 Saga 相关 + Outbox 相关）
# §6 集群事件清单（COC / PFAU / CEM 自身事件）
# §7 事件订阅者注册表（event_producer_registry）
# §8 事件回放 / 重放 / 死信处理
# §9 事件去重 / 幂等 key 规范
# §10 事件 schema 样例（WalletCommitted 完整示例）
# §11 签字栏
```

---

## 5. 激活条件

🔴 → 🟢：

1. **G-CODE-06** Rust 1.98 + cargo build + cargo test 实际跑过全绿
2. **G-CODE-03** 5 独立 DB 拓扑图实际画过
3. NO-GO 完全解除

满足后由 Ulysses 升 v0.2 填入实际事件 schema 字典。

---

## 6. 关联文档

- 父文档：RGS-SPEC-000 详细设计规格化总表
- 强制并行：CROSS-002 Proto 风格指南 / CROSS-006 trace_id 传播
- 上游：RGS-ADR-0051 中心事件管理 + RGS-SPEC-DTL-031 §3
- 5 域引用方：DTL-015/016/018/019/020/026/031 §3 实现契约 + WF-1-54.10 CEM 事件订阅
- worktree：可单独 worktree 分支执行
