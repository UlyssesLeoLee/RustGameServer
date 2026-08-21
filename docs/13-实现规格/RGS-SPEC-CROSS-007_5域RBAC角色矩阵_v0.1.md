# RGS-SPEC-CROSS-007 5 域 RBAC 角色矩阵（Cross-Domain RBAC Role Matrix）

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-SPEC-CROSS-007 |
| 版本 | 0.1（占位，per WBS v0.3 §2A.6.7 横向规范补全 2026-08-21）|
| 依据 | RGS-WBS-001 v0.3 §2A.6.7（admin 域 DTL §3 实现契约横向规范）|
| 状态 | 🔴 **占位 NO-GO**（per RGS-PLAN-001 v0.8 §3.3 + RGS-EXEC-001 v0.3）|
| 责任人 | Ulysses（一人公司 12 角色兼任 per DEC-008）|
| 关联 150 工程 | 54（编码实现）|
| 父文档 | [RGS-SPEC-000 详细设计规格化总表](RGS-SPEC-000_详细设计规格化总表.md) |
| 强制并行 | RGS-SPEC-CROSS-001 错误码字典 / RGS-SPEC-CROSS-002 Proto 风格 |

---

## 1. 文档目的

本文件是 **5 域 RBAC 角色 → admin 域 RBAC 角色矩阵**的横向规范，per WBS v0.3 §2A.6.7 "补救❶ 跨 DTL 接口契约不一致"。

**核心问题**：5 域各自定义 RBAC 角色（player / GM / moderator / support）→ admin 域统一管控时角色映射混乱。

**解决方式**：建立 5 域 → admin 域 RBAC 角色矩阵，强制所有域角色引用本矩阵。

---

## 2. 规范范围

### §2.1 输入

- admin 域 DTL §3 实现契约（DTL-031 §3 RBAC）
- 5 域 DTL §5 安全、容错与发布（DTL-015/016/018/019/020/026）
- 业务角色分类（player / GM / moderator / support / ops / auditor）
- RGS-REQ-007 运维与 GM 后台管控

### §2.2 输出

- 业务角色清单（player / GM / moderator / support / ops / auditor / owner / guest）
- admin 域 RBAC 角色定义（super_admin / domain_admin / service_operator / read_only / emergency_lock / audit_compliance）
- 5 域 → admin 域角色映射矩阵（每域每业务角色 → 1 个 admin 角色）
- 权限粒度规范（resource / action / scope 三元组）
- 权限继承规范（admin 域角色继承规则）
- 紧急锁 / 维护模式 / 灰度角色（per RGS-REQ-007）
- 跨域调用鉴权（player 域调 economy 域时，鉴权从 player JWT 提取 subject → admin 域查询角色）
- 角色变更审计（admin 域 `rbac_audit_log` 表 + CEM 事件）
- 错误码映射（RBAC 拒绝 → CROSS-001 错误码 5001）

### §2.3 责任人

| 角色 | 责任人 | 签字日 |
|---|---|---|
| admin 域 Lead | Ulysses（一人公司 12 角色兼任 per DEC-008）| 2026-08-21 |
| Platform Lead | Ulysses（一人公司 12 角色兼任 per DEC-008）| 2026-08-21 |
| 5 域 Lead 各 1 名 | Ulysses（一人公司 12 角色兼任 per DEC-008）| 2026-08-21 |
| 评审 | Ulysses（架构师兼）| 2026-08-21 |

---

## 3. 工具链

- **admin 域 `rbac_roles` / `rbac_permissions` / `rbac_role_permissions` / `rbac_user_roles` 表**（per DTL-031）
- **JWT + OIDC**（外部 IdP 接入，per RGS-REQ-021）
- **tonic gRPC interceptor**（服务端鉴权）
- **OpenTelemetry**（RBAC 拒绝事件 trace，per CROSS-006）

---

## 4. 文档结构（待 NO-GO 解除后填充）

```markdown
# §1 业务角色清单
# §2 admin 域 RBAC 角色定义
# §3 5 域 → admin 域角色映射矩阵
# §4 权限粒度规范（resource / action / scope）
# §5 权限继承规范
# §6 紧急锁 / 维护模式 / 灰度角色
# §7 跨域调用鉴权流程
# §8 角色变更审计
# §9 RBAC 错误码映射（CROSS-001 错误码 5001）
# §10 RBAC 矩阵样例（5 域完整映射表）
# §11 签字栏
```

---

## 5. 激活条件

🔴 → 🟢：

1. **G-CODE-06** Rust 1.98 + cargo build + cargo test 实际跑过全绿
2. **G-CODE-03** 5 独立 DB 拓扑图实际画过
3. NO-GO 完全解除

满足后由 Ulysses 升 v0.2 填入实际 RBAC 矩阵。

---

## 6. 关联文档

- 父文档：RGS-SPEC-000 详细设计规格化总表
- 强制并行：CROSS-001 错误码字典 / CROSS-002 Proto 风格指南
- 上游：RGS-WF-001 v0.5 §2 150 工程 54 + RGS-REQ-007 运维与 GM 后台管控
- 5 域引用方：DTL-015/016/018/019/020/026/031 §3 + WF-1-54.12 RBAC 中间件
- 部署：admin 域 4 张 rbac_* 表 + audit log
- worktree：可单独 worktree 分支执行
