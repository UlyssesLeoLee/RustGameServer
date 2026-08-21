# RGS-SPEC-CROSS-004 DTO ↔ 领域实体映射规则（DTO/Domain Mapping Rules）

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-SPEC-CROSS-004 |
| 版本 | 0.1（占位，per WBS v0.3 §2A.6.7 横向规范补全 2026-08-21）|
| 依据 | RGS-WBS-001 v0.3 §2A.6.7（5 域 DTL §3 实现契约横向规范）|
| 状态 | 🔴 **占位 NO-GO**（per RGS-PLAN-001 v0.8 §3.3 + RGS-EXEC-001 v0.3）|
| 责任人 | Ulysses（一人公司 12 角色兼任 per DEC-008）|
| 关联 150 工程 | 54（编码实现）|
| 父文档 | [RGS-SPEC-000 详细设计规格化总表](RGS-SPEC-000_详细设计规格化总表.md) |
| 强制并行 | RGS-SPEC-CROSS-001 错误码字典 / RGS-SPEC-CROSS-002 Proto 风格 |

---

## 1. 文档目的

本文件是 **gRPC DTO ↔ 领域实体**映射 + 防腐层（Anti-Corruption Layer）规则的横向规范，per WBS v0.3 §2A.6.7 "补救❶ 跨 DTL 接口契约不一致"。

**核心问题**：5 域 DTL §3 各自定义 gRPC message 与领域实体的映射 → 跨域调用时字段裁剪 / 类型转换 / 必填校验不统一。

**解决方式**：建立 DTO 映射规则，强制所有 5 域使用统一 ACL 层 + DTO 转换函数。

---

## 2. 规范范围

### §2.1 输入

- 5 域 DTL §3 实现契约
- gRPC Proto 风格指南（CROSS-002）
- 5 域领域实体定义（DTL §2 实现单元）

### §2.2 输出

- DTO 命名规范（`<Domain>Proto` 远程 / `<Domain>Entity` 本地）
- ACL 层架构（`rgs-acl` shared crate，per CROSS-002 §10）
- 字段裁剪规则（敏感字段如 password/token/email 不出 DTO）
- 类型转换规则（i64/u64/i32 时间戳 / Decimal 金额 / 枚举等）
- 必填字段校验规则（per CROSS-001 错误码 0001 参数校验）
- 默认值规则（缺失字段 vs 显式 null 区分）

### §2.3 责任人

| 角色 | 责任人 | 签字日 |
|---|---|---|
| Platform Lead | Ulysses（一人公司 12 角色兼任 per DEC-008）| 2026-08-21 |
| 5 域 Lead 各 1 名 | Ulysses（一人公司 12 角色兼任 per DEC-008）| 2026-08-21 |
| 评审 | Ulysses（架构师兼）| 2026-08-21 |

---

## 3. 工具链

- **prost**（Proto → Rust struct）
- **serde**（JSON 序列化兜底）
- **validator**（字段校验）
- **thiserror**（ACL 错误类型）

---

## 4. 文档结构（待 NO-GO 解除后填充）

```markdown
# §1 DTO / Entity 命名规范
# §2 ACL 层架构与 rgs-acl crate 设计
# §3 字段裁剪规则（敏感字段黑名单）
# §4 类型转换规则（i64 / Decimal / 时间戳 / 枚举）
# §5 必填字段校验规则
# §6 默认值规则（Option vs 显式 default）
# §7 DTO 转换函数样例（player 域 GetProfile ACL）
# §8 跨域 DTO 转换注意事项（5 独立 DB 字段差异）
# §9 签字栏
```

---

## 5. 激活条件

🔴 → 🟢：

1. **G-CODE-06** Rust 1.98 + cargo build + cargo test 实际跑过全绿
2. **G-CODE-03** 5 独立 DB 拓扑图实际画过
3. NO-GO 完全解除

满足后由 Ulysses 升 v0.2 填入实际映射规则。

---

## 6. 关联文档

- 父文档：RGS-SPEC-000 详细设计规格化总表
- 强制并行：CROSS-001 错误码字典 / CROSS-002 Proto 风格指南
- 上游：RGS-WF-001 v0.5 §2 150 工程 54
- 5 域引用方：DTL-015/016/018/019/020/026/031 §3 + WF-1-54.6 domain entity + Repository trait
- worktree：可单独 worktree 分支执行
