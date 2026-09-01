# 4 DRAFT Partitioned SQL 评审召集时序 v0.1 (per DB-CHECKLIST v0.1.1, 2026-09-02 10:30 JST)

> **创建日期**: 2026-09-02 10:30 JST
> **创建者**: 架构师(Mavis 接手 agent per DEC-008) 代签 Ulysses
> **状态**: 🟡 时序就绪 (等 Phase C SRE 介入 + DBA 评审启动)
> **关联**: `RGS-DB-PARTITIONED-DRAFT-REVIEW-CHECKLIST-2026-09-02.md` v0.1.1 (commit `24ce59c`) + WBS v0.4.9 §4.1 (commit `3501f52`)

## 0. 时序目标

把 4 DRAFT partitioned SQL (audit_log + transaction_ledger + sagas + moves) 的 10 行签字栏从"待签"推进到"全签",按**领域依赖 + 角色可用性**排并行/串行顺序,避免评审召集时域 Lead 反复被打断。

## 1. 依赖关系 (topological order)

| 层 | 角色 | 输入依赖 | 输出 |
|---|---|---|---|
| 1 | SRE Lead | 无 | k3s ulyssespc 节点状态 + 5 域 mTLS 部署状态 + DB 池可用性 |
| 2 | DBA Lead | SRE k3s + 5 域 mTLS 落地证明 | Schema 正确性 + 保留期合规 + 索引性能 主审 |
| 3a | admin Lead (PH-2) | DBA Lead 主审通过 + RGS-BAS-007 §4 + 17-P0-02 | audit_log 业务验证 (P0-02) |
| 3b | economy Lead (PH-3) | DBA Lead 主审通过 + T-02 + T-03 + RGS-DB-BAS-001 v0.2 §9.4 | transaction_ledger + sagas 业务验证 (T-02/T-03) |
| 3c | match Lead (PH-3) | DBA Lead 主审通过 + T-04 + P1-07 | moves 业务验证 (T-04/P1-07) |
| 4 | 架构师 (Mavis 接手 per DEC-008) | 3a/3b/3c 4 域 Lead 全部签字 | 总审批 + 召集 PH-2/PH-3 实施窗口 |

**关键依赖**: SRE (层 1) → DBA (层 2) → 3 域 Lead 并行 (层 3a/3b/3c) → 架构师总审批 (层 4)

## 2. 时序流程

### Phase 0: 启动准备 (~30 min, 架构师触发)

- 架构师发出评审召集通知 (per DEC-008 一人公司 12 角色 + Slack/邮件/口头)
- 通知内容: DB-CHECKLIST v0.1.1 + 本时序 v0.1 + 4 DRAFT SQL git 链接 + 评审截止时间
- 角色清单:
  - SRE Lead: 待 Phase C 介入
  - DBA Lead: 待指派
  - admin Lead: 9/1-9/2 派工签字已落 (per RACI v0.2 §3)
  - economy Lead: 9/1-9/2 派工签字已落
  - match Lead: 9/1-9/2 派工签字已落
  - 架构师: 评审召集人 (Mavis 接手 per DEC-008)

### Phase 1: SRE 落地证明 (1-3 天, 阻塞 Phase C)

- 阻塞: WSL k3s `ulyssespc` 节点注册未恢复 (per OPEN-QA v0.3 §7.1)
- 落地标志: `kubectl get nodes` 看到 ulyssespc Ready + 5 域 mTLS binary 起来
- SRE Lead 签字: 评审表第 7 行 "SRE Lead 签字" + 日期

### Phase 2: DBA 主审 (3-5 天, 并行可选但建议串行)

- 串行依赖: 必须等 SRE 落地证明, 否则 DBA 没法验证 schema 在真实 PG 14+ 跑通
- DBA 主审内容 (per DB-CHECKLIST v0.1.1 §2):
  - 2.1 Schema 正确性 (分区列 + 分区间隔 + 初始分区 + 触发器 + CHECK 约束 + FK)
  - 2.2 保留期与合规 (RANGE 上界 / cron / DROP 权限)
  - 2.3 性能与索引 (分区裁剪 + EXPLAIN 验证)
- DBA Lead 签字: 评审表第 1 行 "DBA Lead 签字" + 日期

### Phase 3: 3 域 Lead 业务验证 (并行, 5-7 天)

3 域 Lead 业务验证可以**完全并行** (DBA 主审通过后), 但要 4 域 Lead **全部签字后才进入 Phase 4**:

| 域 Lead | 评审 SQL | 业务验证 | 签字行 |
|---|---|---|---|
| admin Lead (PH-2) | `0006_audit_log_partitioned.sql` | 审计日志查询 (actor 维度 / action 维度 / 时间范围) | 评审表第 2 行 |
| economy Lead (PH-3) | `0006_transaction_ledger_partitioned.sql` + `0007_sagas_partitioned.sql` | 交易查询 (OCC FOR UPDATE 跨分区) + saga 状态机 (跨分区查询聚合) | 评审表第 3 行 |
| match Lead (PH-3) | `0041_moves_partitioned.sql` | match 业务 moves 写入 QPS + 跨赛季历史归档 | 评审表第 5 行 |
| 双写期验证 (3 域 × 1 联合) | (admin + SRE) / (economy + SRE) / (match + SRE) | 双写期验证 (per DB-CHECKLIST v0.1.1 §2.5) | 评审表第 6/7/8 行 |

### Phase 4: 架构师总审批 + 召集 PH-2/PH-3 (~1 天)

- 架构师检查全部签字到位 (DBA + 3 域 Lead + SRE + 双写期验证)
- 评审表第 9 行 "架构师 (Mavis 接手 per DEC-008)" 签字
- 召集 PH-2 (admin audit_log) / PH-3 (economy + match) 实施窗口
- 落地动作: DRAFT 状态 → v1.0 (评审通过版) commit
- 召集人: 架构师 (per DEC-008)

## 3. 评审截止时间 (per WBS v0.2 §2.5 E8 节奏)

| 阶段 | 截止时间 | 备注 |
|---|---|---|
| Phase 0 启动 | 评审召集发出后立即 | 架构师触发 |
| Phase 1 SRE 落地 | Phase C 介入后 ~1-3 天 | 阻塞项, 等 SRE |
| Phase 2 DBA 主审 | Phase 1 后 ~3-5 天 | 串行 |
| Phase 3 3 域 Lead 业务验证 | Phase 2 后 ~5-7 天 | 并行 |
| Phase 4 总审批 + 召集 PH | Phase 3 后 ~1 天 | 全部签字后 |

**总周期**: ~10-16 天 (取决于 SRE 介入时间 + DBA + 3 域 Lead 可用性)

## 4. 评审召集 SLA

- 评审启动后 7 天内 4 域 Lead 业务验证完成
- DBA 主审后 5 天内 3 域 Lead 启动
- 任何角色 7 天未响应 → 架构师升级 (per DEC-008 一人公司 12 角色, 升级到 Ulysses 拍板)

## 5. 评审召集 checklist

- [ ] Phase 0: 召集通知发出 (Slack/邮件/口头) + 4 角色通知到位
- [ ] Phase 1: SRE Lead 签字 (k3s ulyssespc Ready + 5 域 mTLS 部署完成)
- [ ] Phase 2: DBA Lead 签字 (Schema + 保留期 + 索引 3 维度主审通过)
- [ ] Phase 3a: admin Lead 签字 (PH-2 业务验证)
- [ ] Phase 3b: economy Lead 签字 (PH-3 业务验证)
- [ ] Phase 3c: match Lead 签字 (PH-3 业务验证)
- [ ] Phase 3 双写期验证: 3 域 × SRE 联合签字
- [ ] Phase 4: 架构师总审批 + DRAFT→v1.0 commit

## 6. 派生约束守护

- L11 (build dir lock 防御): 评审过程不涉及 cargo build, 隔离 target dir 不冲突
- L12 (临时文件不入 commit): 评审过程产生 log 放 L12 临时目录, 不入 commit
- L13 (自指字段全 deferred 实时查询): 本文档第 3 节"评审截止时间"用相对时间 (Phase X 后 ~Y 天), 不写绝对日期, 避免自指污染

## 7. 关联文档

- `RGS-DB-PARTITIONED-DRAFT-REVIEW-CHECKLIST-2026-09-02.md` v0.1.1 (主评审材料)
- `RGS-PLAN-WBS-token-bucket-v0.4.md` v0.4.9 §4.1 (WBS 评审启动源)
- `RGS-STATUS-SNAPSHOT-2026-09-02.md` v0.6.25 §5.1 (Phase C 落地后解锁)
- `RGS-RACI-*-V1_*-v1.1.md` (5 域 Lead + batch Lead 派工签字)
- `OPEN-QA v0.3 §7.1` (WSL k3s ulyssespc 节点注册阻塞)

**修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
**审批**: 架构师(Mavis 接手 agent per DEC-008)
**代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)

## 8. 修订历史

| 版本 | 日期 (JST) | 修订人 | 变更 |
|---|---|---|---|
| v0.1 | 2026-09-02 10:30 | 架构师(Mavis 接手 agent per DEC-008) | 初版: 4 DRAFT partitioned SQL 评审召集时序 (Phase 0 启动 / Phase 1 SRE / Phase 2 DBA / Phase 3 3 域 Lead 并行 / Phase 4 架构师总审批), topological 依赖 + 评审截止时间 + 评审 checklist + L11 + L12 + L13 派生约束守护, 代签 per 8/27 19:39/20:56/21:59 JST 三次强化 |
