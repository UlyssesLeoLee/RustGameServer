# RGS-SPEC-CROSS-001 错误码字典（Error Code Registry）

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-SPEC-CROSS-001 |
| 版本 | 0.1（占位，per WBS v0.3 §2A.6.7 横向规范补全 2026-08-21）|
| 依据 | RGS-WBS-001 v0.3 §2A.6.7（5 域 DTL §3 实现契约横向规范）|
| 状态 | 🔴 **占位 NO-GO**（per RGS-PLAN-001 v0.8 §3.3 + RGS-EXEC-001 v0.3）|
| 责任人 | Ulysses（一人公司 12 角色兼任 per DEC-008）|
| 关联 150 工程 | 54（编码实现）|
| 父文档 | [RGS-SPEC-000 详细设计规格化总表](RGS-SPEC-000_详细设计规格化总表.md) |
| 强制前置 | RGS-SPEC-CROSS-002 Proto 风格指南 / RGS-SPEC-CROSS-004 DTO 映射规则 |

---

## 1. 文档目的

本文件是 RustGameServer **5 域统一错误码编号空间**的占位文档，per WBS v0.3 §2A.6.7 "补救❶ 跨 DTL 接口契约不一致"。

**核心问题**（per WBS v0.3 §2A.6 审计）：5 域 DTL §3 实现契约各自写错误码 → 客户端无法统一处理。

**解决方式**：建立全局错误码字典，强制所有 5 域 DTL §3 引用本字典的编号空间，不得自创。

---

## 2. 规范范围

### §2.1 输入

- 5 域 DTL §3 实现契约（DTL-015/016/018/019/020/026/031）
- gRPC 标准错误码（per gRPC status codes 13 种）
- 业务域错误分类（参数校验 / 鉴权 / 资源 / 状态 / 系统）

### §2.2 输出

- 错误码编号空间（5 段：0001-0999 通用 / 1001-1999 player / 2001-2999 economy / 3001-3999 match / 4001-4999 social / 5001-5999 admin / 6001-6999 cluster-ops）
- 错误码 ↔ gRPC status 映射矩阵
- 错误码 → HTTP status 映射（gateway 层）
- 错误码 → i18n 文案 key 映射
- 错误码 → Prometheus 错误指标 tag 映射

### §2.3 责任人

| 角色 | 责任人 | 签字日 |
|---|---|---|
| Platform Lead | Ulysses（一人公司 12 角色兼任 per DEC-008）| 2026-08-21 |
| 5 域 Lead 各 1 名 | Ulysses（一人公司 12 角色兼任 per DEC-008）| 2026-08-21 |
| 评审 | Ulysses（架构师兼）| 2026-08-21 |

---

## 3. 工具链

- **tonic gRPC**（per RGS-TS-001 v0.6）
- **prost**（proto 代码生成）
- **thiserror**（Rust 错误派生）
- **OpenTelemetry**（错误指标自动上报）

---

## 4. 文档结构（待 NO-GO 解除后填充）

```markdown
# §1 错误码编号空间总表
# §2 通用错误码（0001-0999）
# §3 player 域错误码（1001-1999）
# §4 economy 域错误码（2001-2999，含 Q-003 Saga 6 场景）
# §5 match 域错误码（3001-3999）
# §6 social 域错误码（4001-4999）
# §7 admin 域错误码（5001-5999，含 RBAC 拒绝）
# §8 cluster-ops 域错误码（6001-6999，含 PFAU / CEM）
# §9 错误码 ↔ gRPC status 映射矩阵
# §10 错误码 → HTTP status / i18n / Prometheus 映射
# §11 错误码扩展示例（新增域 / 新增错误码流程）
# §12 签字栏
```

---

## 5. 激活条件

🔴 → 🟢：

1. **G-CODE-06** Rust 1.98 + cargo build + cargo test 实际跑过全绿
2. **G-CODE-03** 5 独立 DB 拓扑图实际画过
3. NO-GO 完全解除

满足后由 Ulysses 升 v0.2 填入实际错误码编号表。

---

## 6. 关联文档

- 父文档：RGS-SPEC-000 详细设计规格化总表
- 强制并行：CROSS-002 Proto 风格指南 / CROSS-004 DTO 映射规则
- 上游：RGS-WF-001 v0.5 §2 150 工程 54 / RGS-TS-001 v0.6 §3.2 gRPC
- 5 域引用方：DTL-015/016/018/019/020/026/031 §3 实现契约
- worktree：可单独 worktree 分支执行（per RGS-WT-001）
