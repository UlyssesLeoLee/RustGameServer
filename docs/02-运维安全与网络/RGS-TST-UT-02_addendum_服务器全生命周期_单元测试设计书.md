# 单元测试设计書（単体テスト設計書 / Unit Test Design Document）

**主题域 02 运维安全与网络 — 服务器全生命周期管理（补强）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-TST-UT-02-ADD3 |
| 版本 | 0.2 |
| 父文档 | RGS-REQ-037 v0.1 + RGS-DTL-042 v0.1 |
| V模型层级 | TL-1 单元试验 |
| 制定日 | 2026-08-21 |

---

---

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | Ulysses(架构师兼 / Admin 域 Lead兼 per DEC-008) | 2026-08-21 | 一人公司 12 角色兼任 |
| 评审（技术/架构） | Ulysses(架构师兼 per DEC-008) | 2026-08-21 | DEC-008 |
| 评审（平台/客户端/SRE/DBA/安全/合规/法务） | Ulysses(对应角色兼 per DEC-008) | 2026-08-21 | DEC-008 |
| 评审（运营） | Ulysses(运营兼 per DEC-008) | 2026-08-21 | 仅适用全生命周期文档 |
| **集体签字(per DEC-008)** | **Ulysses(一人公司 12 角色兼任)** | **2026-08-21** | **Ulysses 在审批栏各角色中具名签字,完整 12 角色兼任清单见 RGS-WBS-001 §17 集体签字声明。审批栏细化角色意见详见 RGS-REQ-004 §3.10。** |

---

## 1. 目的

覆盖 RGS-REQ-037 §5~§11 + RGS-DTL-042 §4~§8 新增的 6 阶段操作器（`NewRealmOperator` / `ScaleOperator` / `SplitOperator` / `MergeOperator` / `MergeRollbackOperator` / `RetireOperator` / `ArchiveOperator`）+ Saga 步骤 + 状态机 + 6 张 Plan 表的单元级测试。

## 2. 测试用例

### 2.1 RealmLifecycleState 状态机

| 用例 ID | 对应 FR | 测试目的 | 覆盖类型 |
|---|---|---|---|
| TST-UT-02-L001 | FR-LCM-001 | `NotYet → Active` 合法（开新服）| N |
| TST-UT-02-L002 | FR-LCM-001 | `Active → Scaling → Active` 合法（扩缩容）| N |
| TST-UT-02-L003 | FR-LCM-001 | `Active → Splitting` 合法（分服）| N |
| TST-UT-02-L004 | FR-LCM-001 | `Active → Merging` 合法（合服）| N |
| TST-UT-02-L005 | FR-LCM-001 | `Active → Retired` 合法（退场）| N |
| TST-UT-02-L006 | FR-LCM-001 | `Retired → Active` 合法（二次激活，30 天内）| N |
| TST-UT-02-L007 | FR-LCM-001 | `Retired → Archived` 合法（归档）| N |
| TST-UT-02-L008 | FR-LCM-001 | 非法跳转 `NotYet → Splitting` 拒绝 | A |
| TST-UT-02-L009 | FR-LCM-001 | 非法跳转 `Archived → Active` 拒绝（归档后**不**可激活）| A |
| TST-UT-02-L010 | FR-LCM-001 | 阶段变更经 PFAU 编排（`canary_confirmed` 后才更新状态）| S |

### 2.2 SagaOrchestrator 步骤执行与补偿

| 用例 ID | 对应 FR | 测试目的 | 覆盖类型 |
|---|---|---|---|
| TST-UT-02-L020 | FR-LCM-005 | 6 步 Saga（分服）按顺序全部 Success → `Completed` | N |
| TST-UT-02-L021 | FR-LCM-005 | 步骤 3 失败 → 反向步骤 2 + 1 补偿 → 状态机恢复 | A |
| TST-UT-02-L022 | FR-LCM-005 | 步骤 5 失败 → 反向步骤 4 + 3 + 2 + 1 全部补偿 | A |
| TST-UT-02-L023 | FR-LCM-005 | 已执行步骤重试时返回 `AlreadyApplied` 幂等 | A |
| TST-UT-02-L024 | FR-LCM-005 | Saga 步骤超时（> 60s）触发反向补偿 | B |
| TST-UT-02-L025 | FR-LCM-005 | Saga 步骤执行期间 PFAU 失联 → 状态机 `Failed` | A |
| TST-UT-02-L026 | FR-LCM-053 | 分服 Saga 步骤 6（一致性校验）不通过 → 触发自动补偿 | A |
| TST-UT-02-L027 | FR-LCM-064 | 合服回退 Saga 步骤按 `request_id` 识别已前向步骤 | A |
| TST-UT-02-L028 | FR-LCM-005 | 跨 DB 写入任一失败 → Saga 状态机恢复至变更前 | A |

### 2.3 NewRealmOperator 资源评估

| 用例 ID | 对应 FR | 测试目的 | 覆盖类型 |
|---|---|---|---|
| TST-UT-02-L040 | FR-LCM-020 | `NewRealmPlan` 12 字段完整评估 | N |
| TST-UT-02-L041 | FR-LCM-020 | `target_realm_id` 命名规范校验（与既有不冲突）| N |
| TST-UT-02-L042 | FR-LCM-020 | 三方签字校验：缺一拒绝 | A |
| TST-UT-02-L043 | FR-LCM-021 | ARC-018 挂载清单触发的 5 步骤全部执行 | N |
| TST-UT-02-L044 | FR-LCM-022 | 渐进式挂载：最小配置 → 预热探针 → 扩容到目标 | S |
| TST-UT-02-L045 | FR-LCM-031 | 灰度开放：hidden → white_list → channel_gray → all | N |
| TST-UT-02-L046 | FR-LCM-032 | 灰度期间快速回滚到 hidden | A |
| TST-UT-02-L047 | FR-LCM-011 | 跨分片能力评估：T2+ 新服引入新跨分片能力时拒绝 | A |

### 2.4 SplitOperator 玩家分流

| 用例 ID | 对应 FR | 测试目的 | 覆盖类型 |
|---|---|---|---|
| TST-UT-02-L060 | FR-LCM-051 | `forced` 策略：`hash(account_id) mod N` 分配 | N |
| TST-UT-02-L061 | FR-LCM-051 | `opt_in` 策略：玩家自选 + 超期默认规则 | N |
| TST-UT-02-L062 | FR-LCM-051 | `hybrid` 策略：核心玩家 opt_in + 普通玩家 forced | N |
| TST-UT-02-L063 | FR-LCM-052 | 跨服好友保留（`friend.cross_realm = true`）| N |
| TST-UT-02-L064 | FR-LCM-052 | 工会整体迁移：全部成员到同一新服 | N |
| TST-UT-02-L065 | FR-LCM-052 | 工会拆分：分散到多服时按 `split_plan.cross_realm_relation.guild` 拆分 | N |
| TST-UT-02-L066 | FR-LCM-052 | 私聊记录按玩家归属迁移（不与跨服关系混同）| N |
| TST-UT-02-L067 | FR-LCM-052 | 邮件全部迁移到新归属服 | N |
| TST-UT-02-L068 | FR-LCM-055 | 分服资产不丢不重（FR-LCM-001 一致性校验）| A |

### 2.5 MergeOperator 冲突规则 v2

| 用例 ID | 对应 FR | 测试目的 | 覆盖类型 |
|---|---|---|---|
| TST-UT-02-L080 | FR-LCM-062 | 同名角色按 `auto_rename_with_suffix` 处理 | N |
| TST-UT-02-L081 | FR-LCM-062 | 重复唯一性道具按 `keep_earliest_and_compensate` 处理 | N |
| TST-UT-02-L082 | FR-LCM-062 | 货币冲突**仅**累加（**不**允许重置）| A |
| TST-UT-02-L083 | FR-LCM-062 | 未结算抽奖按 `settle_before_merge` 处理 | N |
| TST-UT-02-L084 | FR-LCM-062 | 未领取邮件按 `carry_over` 处理 | N |
| TST-UT-02-L085 | FR-LCM-062 | 冻结中跨服工会申请按 `keep_pending` 处理 | N |
| TST-UT-02-L086 | FR-LCM-064 | `merge_conflict_rule_set_v2.locked_at` 锁定后**不**允许运行时修改 | A |
| TST-UT-02-L087 | FR-LCM-064 | 合服回退窗口期内可按 Saga 反向步骤回退 | A |
| TST-UT-02-L088 | FR-LCM-064 | 合服回退窗口期外**不**回退到在线服，进入归档查询通道 | A |

### 2.6 RetireOperator + ArchiveOperator

| 用例 ID | 对应 FR | 测试目的 | 覆盖类型 |
|---|---|---|---|
| TST-UT-02-L100 | FR-LCM-070 | 退场 ≠ 删除：保留全部玩家数据 | A |
| TST-UT-02-L101 | FR-LCM-072 | 只读维护模式：停止新会话、新场景、新交易撮合 | N |
| TST-UT-02-L102 | FR-LCM-073 | RBAC 查询通道：仅 `cs_agent` / `sre` / `legal` 可见 | A |
| TST-UT-02-L103 | FR-LCM-074 | `RealmDirectoryService` 状态置为 `retired`，对玩家隐藏 | N |
| TST-UT-02-L104 | FR-LCM-075 | 二次激活窗口期（30 天）内可重新上线 | B |
| TST-UT-02-L105 | FR-LCM-080 | 归档启动阈值：退场后 30~90 天 | B |
| TST-UT-02-L106 | FR-LCM-081 | 归档**不**删除数据，**仅**迁移存储位置 | A |
| TST-UT-02-L107 | FR-LCM-082 | 热归档 → 冷归档 → 超期 三级存储正确切换 | N |
| TST-UT-02-L108 | FR-LCM-083 | 归档后审计链完整（开新服到归档的每步操作留痕）| A |
| TST-UT-02-L109 | FR-LCM-084 | GDPR "被遗忘权"删除通路：定位并删除归档中玩家数据 | A |
| TST-UT-02-L110 | FR-LCM-085 | 跨服合并回溯保留（合服前资产归属记录可还原）| A |

### 2.7 OLU 预算上报

| 用例 ID | 对应 NFR | 测试目的 | 覆盖类型 |
|---|---|---|---|
| TST-UT-02-L120 | NFR-LCM-007 | 阶段变更 OLU 消耗上报至 `rgs-arc-olu` 成功 | S |
| TST-UT-02-L121 | NFR-LCM-007 | OLU 预算超限时阶段变更被拒绝 | A |
| TST-UT-02-L122 | NFR-LCM-007 | 高密度期间串行调度避免并发 OLU 击穿 | S |

## 3. 追溯性

| 需求 | 用例 |
|---|---|
| FR-LCM-001 | TST-UT-02-L001~L010 |
| FR-LCM-005 | TST-UT-02-L020~L028 |
| FR-LCM-011 | TST-UT-02-L047 |
| FR-LCM-020~022 | TST-UT-02-L040~L044 |
| FR-LCM-031~032 | TST-UT-02-L045~L046 |
| FR-LCM-051~055 | TST-UT-02-L060~L068 |
| FR-LCM-062~064 | TST-UT-02-L080~L088 |
| FR-LCM-070~075 | TST-UT-02-L100~L104 |
| FR-LCM-080~085 | TST-UT-02-L105~L110 |
| NFR-LCM-007 | TST-UT-02-L120~L122 |
| AC-LCM-001~010 | 全部（覆盖 §2 全部用例） |

## 4. 通过判定

- §2 全部 56 条用例 PASS
- 状态机非法跳转 100% 拒绝（L008/L009）
- Saga 步骤补偿 100% 正确（L021/L022）
- 退场 ≠ 删除 100% 校验通过（L100）
- GDPR 删除通路 100% 命中（L109）
- 跨服合并回溯 100% 可还原（L110）

---

> 与 RGS-TST-UT-02 + RGS-TST-UT-02-ADD1/ADD2 共存。
