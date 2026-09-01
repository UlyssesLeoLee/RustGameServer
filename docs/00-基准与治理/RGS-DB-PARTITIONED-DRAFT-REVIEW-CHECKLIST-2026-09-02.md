# RGS DB 大表分区化 DRAFT 评审检查清单 (per c2acf02, 2026-09-02 09:50 JST)

> **创建日期**: 2026-09-02 09:50 JST
> **创建者**: 架构师(Mavis 接手 agent per DEC-008) 代签 Ulysses
> **状态**: 🟡 评审启动材料 (等 SRE + DBA + admin/economy/match Lead 评审)
> **关联**: `c2acf02` (上游 AI 9/2 08:25 JST commit, 4 DRAFT partitioned SQL) + WBS v0.4.8 §4 (E8 12 GAP 实施) + RGS-DB-BAS-001 v0.2 §9.4 (分区策略)

## 0. 评审目标

把 `c2acf02` 4 DRAFT partitioned SQL 从 "DRAFT 状态" 推进到 "评审通过 + 可 PH-2/PH-3 实施窗口",给 SRE + DBA + 4 域 Lead 提供单一评审入口。

## 1. 4 DRAFT partitioned SQL 清单 (per `git show --stat c2acf02`)

| commit | 文件 | 行数 | 关联 BAS / 决议 | PH | 评审方 |
|---|---|---:|---|---|---|
| `c2acf02` | `crates/admin-service/migrations/0006_audit_log_partitioned.sql` | 111 | RGS-BAS-007 §4 + 17-P0-02 + v0.2 §3.2 T-05 | PH-2 | SRE + DBA + admin Lead |
| `c2acf02` | `crates/economy-service/migrations/0006_transaction_ledger_partitioned.sql` | 101 | T-02, PH-3 | PH-3 | SRE + DBA + economy Lead |
| `c2acf02` | `crates/economy-service/migrations/0007_sagas_partitioned.sql` | 132 | T-03, PH-3 | PH-3 | SRE + DBA + economy Lead |
| `c2acf02` | `crates/match-service/migrations/0041_moves_partitioned.sql` | 107 | 14-§3.5 P1-07 + v0.2 §3.2 T-04 + §9.4 | PH-3 | SRE + DBA + match Lead |

## 2. 评审检查项 (per 17-P0-02 Expand-Contract 模式 + RGS-BAS-001 v0.2 §9.4)

### 2.1 Schema 正确性 (DBA 主审)

- [ ] 分区列与表主键时间戳字段一致 (created_at / occurred_at)
- [ ] 分区间隔合规 (audit_log 按月, transaction_ledger 按月, sagas 按月, moves 按月)
- [ ] 初始分区 (当月 + 下月) 创建逻辑幂等 (per `CREATE TABLE IF NOT EXISTS`)
- [ ] 触发器 (audit_log_no_modify) 用 `DO + ALTER TABLE` 后置,避免 forward ref
- [ ] CHECK 约束用 `DO + EXCEPTION` 幂等块 (per 17-P0-04 + 13-§3.3)
- [ ] 跨表 FK 用 `DO + ALTER TABLE` 后置
- [ ] snake_case 字段名 / TIMESTAMPTZ 时间戳 / 不允许 nullable 主键 (per RGS-SPEC-CROSS-005 §2)
- [ ] `LIKE audit_log INCLUDING ALL` 复制完整约束 (避免漏索引/触发器)
- [ ] 分区表 RANGE 上界 (TO) 与下界 (FROM) 排他不重叠

### 2.2 保留期与合规 (SRE 主审)

- [ ] 保留期符合 RGS-REQ-001 §NFR-SE-010 (audit_log 3 年 = 36 月, transaction_ledger 3 年, sagas 1 年, moves 1 年 — 业务生命周期)
- [ ] 分区滚动 cron job 设计 (per 14-§5 + RGS-BAS-007 §4): 每月 1 号 UTC 00:00 创建 +3 月分区
- [ ] 保留期到期 DROP cron job 设计: 保留期 N 月后, 最老分区 DROP
- [ ] DROP 前 audit_event 不可丢失的合规性 (per 双层审计 3 年)
- [ ] DROP 操作的权限控制 (仅 DBA role)

### 2.3 性能与索引 (DBA + 域 Lead 联合)

- [ ] 分区裁剪友好索引: 4 索引 (actor_id / action / created_at / actor+created_at 复合) 加速分区裁剪
- [ ] 索引在分区表上 (而非仅在父表) 验证 — PG 12+ 自动传播
- [ ] 高频查询模式: actor 维度 / action 维度 / 时间范围 / 复合查询 4 路径 EXPLAIN 验证
- [ ] 写吞吐影响: 触发器 (audit_log_no_modify) 引入的开销 < 5%
- [ ] 触发器函数指向适配 (PH-2 切写目标时, 触发器函数需重指向 audit_log_partitioned)

### 2.4 Expand-Contract 模式 (SRE 主审, per 17-P0-02)

- [ ] PH-1 (本 migration): 仅建 audit_log_partitioned + 当月 + 下月分区, **不** 改原 audit_log
- [ ] PH-2 步骤 1: 数据迁移 (双写期 + 后台 batch copy) — 何时启动? 跑多久?
- [ ] PH-2 步骤 2: 应用层切换写入目标表 (从 audit_log → audit_log_partitioned)
- [ ] PH-2 步骤 3: 切读流量 (从 audit_log → audit_log_partitioned)
- [ ] PH-2 步骤 4: rename audit_log → audit_log_legacy, audit_log_partitioned → audit_log
- [ ] PH-2 步骤 5: 保留 audit_log_legacy 30 天后 DROP
- [ ] PH-3 后续: 分区滚动 cron job 实施
- [ ] 每步回滚方案明确 (per 17-P0-02 修复建议)

### 2.5 双写期验证 (SRE + 域 Lead 联合)

- [ ] 双写期 duration: 7 天 (业务低峰期) 还是 30 天 (全业务覆盖)?
- [ ] 双写期一致性校验 SQL: row_count(audit_log) == row_count(audit_log_partitioned) + 抽样 hash 对比
- [ ] 双写期切读流量 AB 比例: 10% → 50% → 100% 灰度
- [ ] 双写期监控: lag (audit_log vs audit_log_partitioned) 告警阈值

### 2.6 已知缺口 (DBA + 域 Lead 自报)

- [ ] audit_log 8 项已知缺口 (per admin 0006_partitioned.sql §comment 段): 数据迁移 / 双写期 / 切读流量 / rename 旧表 / 触发器函数指向适配 / 全量回归 / 分区滚动 cron / OCC FOR UPDATE 适配 — 每项都有处理方案
- [ ] transaction_ledger 4 项已知缺口: OCC FOR UPDATE 在分区表的 row-level lock 性能 / 跨分区查询聚合 / 季度对账 SQL 适配 / 冷热数据分层
- [ ] sagas 4 项已知缺口: saga_instance_id 跨分区查询 / saga state machine 重试幂等性 / 跨域 saga 事件分发一致性 / 历史 saga 状态压缩
- [ ] moves 3 项已知缺口: match 业务 moves 写入 QPS 评估 / 跨赛季 moves 历史归档 / 公平性审计 SQL 适配

## 3. 评审决策矩阵

| 维度 | DRAFT → 评审通过 | 评审通过 → 实施 | 实施 → 完成 |
|---|---|---|---|
| Schema 正确性 | DBA 主审通过 | DBA 签字 commit | migration apply 0 error |
| 保留期与合规 | SRE 主审通过 | SRE 签字 commit | cron job 上线 + 监控就绪 |
| 性能与索引 | 域 Lead 业务验证 | 域 Lead 签字 commit | EXPLAIN 指标 < 5% 退化 |
| Expand-Contract 模式 | SRE 流程签字 | SRE + 域 Lead 联合签字 | 5 步骤全部完成 |
| 双写期验证 | 域 Lead 业务验证 | 域 Lead + SRE 联合签字 | 7-30 天双写期 + 一致性校验 100% |
| 已知缺口 | 域 Lead + DBA 联合方案 | commit 方案到 BAS v0.3 | 全部按方案落实 |

## 4. 评审签字栏

| 角色 | 评审方 | 签字 (评审通过时) | 日期 |
|---|---|---|---|
| Schema 正确性 | DBA Lead | _____________ | _________ |
| 保留期与合规 | SRE Lead | _____________ | _________ |
| 性能与索引 - admin | admin Lead | _____________ | _________ |
| 性能与索引 - economy | economy Lead | _____________ | _________ |
| 性能与索引 - match | match Lead | _____________ | _________ |
| Expand-Contract 模式 | SRE Lead | _____________ | _________ |
| 双写期验证 - admin | admin Lead + SRE Lead | _____________ | _________ |
| 双写期验证 - economy | economy Lead + SRE Lead | _____________ | _________ |
| 双写期验证 - match | match Lead + SRE Lead | _____________ | _________ |
| 总审批 | 架构师 (Mavis 接手 per DEC-008) | _____________ | _________ |

## 5. 实施前置条件 (评审通过后)

- [ ] DRAFT 状态 → v1.0 (评审通过版) commit
- [ ] 本地 PG 演练环境 `cargo sqlx prepare --workspace -- --all-targets` 跑通 + .sqlx/ commit
- [ ] k3s ulyssespc 节点注册恢复 (per Phase C SRE 介入, 当前 0/5)
- [ ] 5 域 gRPC 业务级 mTLS 部署完成 (Phase C 5/5)
- [ ] DBA + SRE + 域 Lead 评审签字齐
- [ ] 双写期开始时间窗口排定 (业务低峰期)
- [ ] 监控告警阈值确认 (lag < 1min / 切读流量成功率 > 99.9%)

## 6. 相关 commit + 文档引用

- `c2acf02` (上游 AI commit, 4 DRAFT partitioned SQL)
- WBS v0.4.5 §4 (E8 12 GAP 实施, 4 DRAFT 状态已 commit)
- STATUS-SNAPSHOT v0.6.16 §0.1 (4 tracked-but-DRAFT 状态记录)
- RGS-DB-BAS-001 v0.2 §9.4 (分区策略 + 14-§3.x + 17-P0-02)
- RGS-REQ-001 §NFR-SE-010 (双层审计 3 年保留)
- RGS-BAS-007 §4 (audit_log 按月 RANGE 分区)

**修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
**审批**: 架构师(Mavis 接手 agent per DEC-008)
**代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)

## 7. 修订历史

| 版本 | 日期 (JST) | 修订人 | 变更 |
|---|---|---|---|
| v0.1 | 2026-09-02 09:50 | 架构师(Mavis 接手 agent per DEC-008) | 初版: 4 DRAFT partitioned SQL 评审启动材料, 7 大评审检查项 + 4 维决策矩阵 + 4 域 Lead + SRE + DBA 签字栏 + 5 项实施前置条件, 代签 per 8/27 19:39/20:56/21:59 JST 三次强化 |
