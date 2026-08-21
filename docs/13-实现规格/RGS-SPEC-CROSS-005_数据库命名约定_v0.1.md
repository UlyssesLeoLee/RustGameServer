# RGS-SPEC-CROSS-005 数据库命名 / 字段类型统一约定（DB Naming & Type Conventions）

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-SPEC-CROSS-005 |
| 版本 | 0.1（占位，per WBS v0.3 §2A.6.7 横向规范补全 2026-08-21）|
| 依据 | RGS-WBS-001 v0.3 §2A.6.7（5 域 DTL §3 实现契约横向规范）|
| 状态 | 🔴 **占位 NO-GO**（per RGS-PLAN-001 v0.8 §3.3 + RGS-EXEC-001 v0.3）|
| 责任人 | Ulysses（一人公司 12 角色兼任 per DEC-008）|
| 关联 150 工程 | 54（编码实现）|
| 父文档 | [RGS-SPEC-000 详细设计规格化总表](RGS-SPEC-000_详细设计规格化总表.md) |
| 强制并行 | RGS-IMPL-002 PG 编码规范 / RGS-SPEC-CROSS-004 DTO 映射 |

---

## 1. 文档目的

本文件是 **5 独立 DB**（player_db / economy_db / match_db / social_db / admin_db + cluster_ops_db，per ARC-008 5 独立 DB 原则）的**命名 / 字段类型 / 跨域约束**统一约定，per WBS v0.3 §2A.6.7 "补救❶ 跨 DTL 接口契约不一致"。

**核心问题**：5 域 DB schema 独立建表 → 命名风格 / 字段类型 / 跨域 join 限制不统一。

**解决方式**：建立 DB 命名 / 字段类型统一约定，强制 5 域 DDL 引用本约定。

---

## 2. 规范范围

### §2.1 输入

- 5 域 DTL §3 实现契约（DTL-015/016/018/019/020/026/031）
- RGS-IMPL-002 PG 编码规范
- ARC-008 5 独立 DB 原则（**与团队规模无关，是架构原则**）
- PostgreSQL 18.6 + sqlx（per RGS-TS-001 v0.6 §5.1/§5.2）

### §2.2 输出

- Schema 命名规范（每 DB 一个 schema：`player_db.player / economy_db.economy / match_db.match / social_db.social / admin_db.admin / cluster_ops_db.cluster_ops`）
- 表命名规范（snake_case + 域前缀可选 + 业务名）
- 字段命名规范（snake_case + 禁用缩写）
- 字段类型映射（i64 → BIGINT / Decimal → NUMERIC(20,8) / 时间戳 → TIMESTAMPTZ / JSONB / 枚举 → SMALLINT + CHECK）
- 主键规范（`id BIGSERIAL PRIMARY KEY` 或 UUID v7）
- 外键规范（跨 DB **禁用**外键，仅逻辑外键 + 应用层校验）
- 索引规范（B-tree / GIN / BRIN / 部分索引 / 表达式索引）
- 分区表规范（按 created_at 月分区 / 按 tenant_id hash 分区）
- 跨域 join 限制（5 独立 DB 禁止跨 DB join；通过 Outbox + CEM 异步协调）

### §2.3 责任人

| 角色 | 责任人 | 签字日 |
|---|---|---|
| DBA | Ulysses（一人公司 12 角色兼任 per DEC-008）| 2026-08-21 |
| Platform Lead | Ulysses（一人公司 12 角色兼任 per DEC-008）| 2026-08-21 |
| 5 域 Lead 各 1 名 | Ulysses（一人公司 12 角色兼任 per DEC-008）| 2026-08-21 |
| 评审 | Ulysses（架构师兼）| 2026-08-21 |

---

## 3. 工具链

- **PostgreSQL 18.6**（per RGS-TS-001 v0.6 §5.2）
- **sqlx**（Rust 端 query 宏 + 编译期校验）
- **sqruff**（SQL lint 强制命名 + 风格）
- **atlas**（schema migration 工具）

---

## 4. 文档结构（待 NO-GO 解除后填充）

```markdown
# §1 Schema 命名规范
# §2 表命名规范
# §3 字段命名规范
# §4 字段类型映射（Rust ↔ PostgreSQL）
# §5 主键 / 外键规范（跨 DB 禁用外键）
# §6 索引规范（B-tree / GIN / BRIN / 部分 / 表达式）
# §7 分区表规范（月分区 / hash 分区）
# §8 跨域 join 限制 + 异步协调（Outbox + CEM）
# §9 迁移规范（atlas migrate + 兼容性约束）
# §10 表 / 字段样例（5 域 DDL 示例）
# §11 签字栏
```

---

## 5. 激活条件

🔴 → 🟢：

1. **G-CODE-06** Rust 1.98 + cargo build + cargo test 实际跑过全绿
2. **G-CODE-03** 5 独立 DB 拓扑图实际画过（**包含本规范定义的 schema 命名示例**）
3. NO-GO 完全解除

满足后由 Ulysses 升 v0.2 填入实际 DB 命名 / 类型约定。

---

## 6. 关联文档

- 父文档：RGS-SPEC-000 详细设计规格化总表
- 强制并行：CROSS-004 DTO 映射 / RGS-IMPL-002 PG 编码规范
- 上游：RGS-WF-001 v0.5 §2 150 工程 54 + ARC-008 5 独立 DB 原则 + RGS-TS-001 v0.6 §5.1/§5.2
- 5 域引用方：DTL-015/016/018/019/020/026/031 §3 + WF-1-53.10 5 独立 PG 18.6 DB
- 部署：docs/deploy/03-db-migrations/
- worktree：可单独 worktree 分支执行
