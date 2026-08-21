# RGS-SPEC-CROSS-002 gRPC Proto 风格指南（Proto Style Guide）

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-SPEC-CROSS-002 |
| 版本 | 0.1（占位，per WBS v0.3 §2A.6.7 横向规范补全 2026-08-21）|
| 依据 | RGS-WBS-001 v0.3 §2A.6.7（5 域 DTL §3 实现契约横向规范）|
| 状态 | 🔴 **占位 NO-GO**（per RGS-PLAN-001 v0.8 §3.3 + RGS-EXEC-001 v0.3）|
| 责任人 | Ulysses（一人公司 12 角色兼任 per DEC-008）|
| 关联 150 工程 | 54（编码实现）|
| 父文档 | [RGS-SPEC-000 详细设计规格化总表](RGS-SPEC-000_详细设计规格化总表.md) |
| 强制并行 | RGS-SPEC-CROSS-001 错误码字典 / RGS-SPEC-CROSS-003 事件 schema |

---

## 1. 文档目的

本文件是 5 域 gRPC Proto 文件**命名 / 错误处理 / 分页 / 流控**的横向规范，per WBS v0.3 §2A.6.7 "补救❶ 跨 DTL 接口契约不一致"。

**核心问题**：5 域 Proto 文件独立写 → 命名风格不统一 / 错误处理不一致 / 分页机制各异 / 流控参数分散。

**解决方式**：建立 Proto 风格指南，强制所有 5 域 Proto 引用本指南，不得自创命名 / 错误 / 分页 / 流控。

---

## 2. 规范范围

### §2.1 输入

- 5 域 DTL §3 实现契约
- gRPC 官方风格指南（https://google.github.io/styleguide/grpc/）
- Buf Schema Registry（BSR）约束
- tonic + prost 工具链

### §2.2 输出

- Proto 包命名规范（`rgs.<domain>.<version>`，如 `rgs.player.v1`）
- Service / RPC / Message / Field / Enum 命名规范
- 错误处理规范（每 RPC 必带 google.rpc.Status + RGS-SPEC-CROSS-001 错误码）
- 分页规范（PageToken + PageSize + 强制上限 100）
- 流控规范（per-stream + per-connection 限流参数）
- 超时规范（per-RPC deadline + propagation）
- 字段废弃规范（`[deprecated = true]` + 灰度期 ≥ 6 个月）
- Proto 兼容性约束（per buf breaking change rules）

### §2.3 责任人

| 角色 | 责任人 | 签字日 |
|---|---|---|
| Platform Lead | Ulysses（一人公司 12 角色兼任 per DEC-008）| 2026-08-21 |
| 5 域 Lead 各 1 名 | Ulysses（一人公司 12 角色兼任 per DEC-008）| 2026-08-21 |
| 评审 | Ulysses（架构师兼）| 2026-08-21 |

---

## 3. 工具链

- **buf**（Proto lint + breaking change 检测）
- **tonic-build**（Rust 端代码生成）
- **buf-schema-registry**（跨服务 Proto 共享）
- **CI 强制**：`docs/deploy/04-ci-cd/rust-ci.yaml` 中 buf lint + buf breaking 必须通过

---

## 4. 文档结构（待 NO-GO 解除后填充）

```markdown
# §1 Proto 包命名规范
# §2 Service / RPC / Message 命名规范
# §3 错误处理规范（google.rpc.Status + CROSS-001 错误码）
# §4 分页规范（PageToken + PageSize）
# §5 流控规范（per-stream + per-connection）
# §6 超时与 deadline 规范
# §7 字段废弃与兼容性规范
# §8 buf lint 规则集
# §9 buf breaking change 规则集
# §10 Proto → Rust 映射规则（prost 特定约束）
# §11 Proto 样例文件（player 域 GetProfile RPC 完整示例）
# §12 签字栏
```

---

## 5. 激活条件

🔴 → 🟢：

1. **G-CODE-06** Rust 1.98 + cargo build + cargo test 实际跑过全绿
2. **G-CODE-03** 5 独立 DB 拓扑图实际画过
3. NO-GO 完全解除

满足后由 Ulysses 升 v0.2 填入实际 Proto 风格规则。

---

## 6. 关联文档

- 父文档：RGS-SPEC-000 详细设计规格化总表
- 强制并行：CROSS-001 错误码字典 / CROSS-003 事件 schema / CROSS-004 DTO 映射
- 上游：RGS-WF-001 v0.5 §2 150 工程 54 / RGS-TS-001 v0.6 §3.2 gRPC
- 5 域引用方：DTL-015/016/018/019/020/026/031 §3 实现契约 + WF-1-54.2 5 域 gRPC Proto 定义
- worktree：可单独 worktree 分支执行
