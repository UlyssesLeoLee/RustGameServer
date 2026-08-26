# 详细设计书（詳細設計書 / Detailed Design Document）

**Admin 域 Atomic App 与控制面契约骨架**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DTL-040 |
| 版本 | 0.2 |
| 状态 | **契约骨架・待评审・不得作为实施授权** |
| 父文档 | RGS-REQ-007、RGS-BAS-003、RGS-REQ-031、RGS-BAS-031、RGS-DTL-031 |
| App/DB | `admin-service` / `admin_db` |
| 制定日 | 2026-08-21 |
| 制定者 | 架构师 |
| 修订历史 | 0\.1（2026\-08\-21）：建立\ Admin\ 域契约骨架;\ 0\.2\(2026\-08\-26\):\ 同步父\ BAS\-003/BAS\-031\ 升版至\ v0\.2,元数据层对齐\(per\ 2026\-08\-26\ 08:40\ JST\ 新规则,审批者\ =\ 架构师\(Mavis\ 接手\ agent\ per\ DEC\-008\)\) |
| 审批 | 未审批；与 DTL-031 / ADR-0052 联合 DD Review 前不得作为实施授权 |

## 1. 领域职责与非职责

- 负责 GM/COC 统一入口、RBAC、操作审计、Feature/CEM/PFAU 控制面转发。
- `admin_db` 保存控制面状态；不保存 Player/Economy/Match/Social 业务事实。
- `ClusterOpsService` 是 Admin 限界上下文的独立控制面服务，不由 AdminService 复制实现。

## 2. 集群契约

```yaml
app_id: admin-service
db: admin_db
depends_on: [event-bus, config, observability, secrets]
scaffold_ref: services/admin-service/deploy/helm
feature_host: true
health: [/healthz, /readyz]
control_plane_peer: cluster-ops-service
```

## 3. API 与转发骨架

| 层 | 契约 | 规则 |
|---|---|---|
| AdminService | 既有 GM API + COC 转发 API | RBAC、request_id、approval_ref、统一审计 |
| ClusterOpsService | `RegisterFeature`、`DeclareFeatureUpgrade`、`AdvanceCanary`、`Rollback`、CEM/DLQ API | 双副本、OCC/fencing、状态机唯一实现 |
| COC UI | 只读查询与受控写操作 | 不持有 K8s/DB 凭证，不直连 ClusterOpsService |

## 4. 控制面数据与插件边界

`feature_registry`、版本历史、PFAU 状态和控制面审计关联均落 `admin_db`。Admin 插件只能扩展 Feature 矩阵、审计查询或受控页面；不得绕过 RBAC、不得改变 PFAU 状态、不得调用 K8s/Helm。

## 5. 迁移、回滚与测试

- 控制面表使用 Expand-Contract 和 OCC；状态迁移必须有合法转移表。
- ClusterOpsService 重启从 `admin_db` 恢复；Redis 租约不可用时写入 fail-closed。
- 必须覆盖：双副本并发写、旧 fencing token、重复 request、AdminService 转发、COC 凭证边界、PFAU all-reachable 和审计完整性。

## 6. 待补齐项

- [ ] AdminService 到 ClusterOpsService 的字段级 protobuf 与错误映射。
- [ ] `admin_db` 迁移脚本、唯一约束和索引评审。
- [ ] `cluster_operator/cluster_admin` 与既有 RBAC 的最终矩阵。
- [ ] COC UI 写操作、DLQ replay 和 PFAU 演练验收证据。


---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响章节 |
|---|---|---|---|---|---|
| 0.1 | 2026-08-21 | 架构师 | — | 建立 Admin 域契约骨架 | 全部 |
| 0.2 | 2026-08-26 | 架构师(Mavis 接手 agent per DEC-008) | 架构师(Mavis 接手 agent per DEC-008) | **同步父 BAS-003/BAS-031 升版至 v0.2**(BAS-003 v0.2 装饰性升版 + BAS-031 v0.2 元数据升版): 本 DTL 是父 BAS 的详细化(per DTL 头部'不改变任何既有决定'), 父 BAS 升版为元数据层对齐, DTL-040 既有章节内容无实质重写, 本升版仅做元数据层对齐; **正文本不重写**。 审批按 2026-08-26 08:40 JST 新规则由 Mavis 接手 agent 代签。 | 全部 |
