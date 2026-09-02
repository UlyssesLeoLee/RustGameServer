# 基本设计书（基本設計書 / Basic Design Document）

**排行榜、任务成就与玩家治理 Leaderboard, Quest/Achievement & Player Governance**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-BAS-014 |
| 版本 | 0.3 |
| 父文档 | RGS-REQ-017 需求定义书（ARC-031） |
| 制定日 | 2026-08-16 |
| 最终更新日 | 2026-09-01 |
| 制定者 | 架构师 |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响需求ID |
|---|---|---|---|---|---|
| 0.1 | 2026-08-16 | 架构师 | — | 初版制定。将RGS-REQ-017§11 ARC-031展开为：派生排行视图的组件设计与更新时序、任务/成就的配置化触发引擎设计、邮件系统的数据模型、举报/黑名单的字段级设计、赛季重置的时序图 | 全部 |
| 0.2 | 2026-08-16 | 架构师 | — | 补强字段级细节：①新增`RankingDimensionConfig`维度可配置表（FR-GSM-001）②补充`RankingViewUpdater`消费失败/死信分支与视图重建路径（FR-GSM-003、NFR-GSM-002）③新增举报者信誉度字段与降权机制设计（FR-GSM-033、RSK-GSM-002）④补充`MailMessage`/`PlayerReport`/`PlayerBlocklist`索引与唯一性约束（复用RGS-BAS-007标准） | FR-GSM-001、FR-GSM-003、FR-GSM-033、NFR-GSM-002、RSK-GSM-002 |
| 0.3 | 2026-09-01 | 架构师 (Mavis 接手 agent per DEC-008) | 架构师(Mavis 接手 agent per DEC-008) | 落实"各BAS文档功能章节加log设计且区分debug/release级"总要求（per Ulysses 2026-09-01 15:52 JST 决策，4 拍板选项：全部 36 个BAS / 详尽版5列表 / 派worker并行 / BAS-004同步升级）：§3／§4／§5／§6／§7 全部 5 个 ## L2 功能段加"本功能日志设计"5 列详尽版（字段名／触发条件／频率估算／采样策略／脱敏与成本），字段名前缀统一为 `rank.*` 区别于其他域；引用 BAS-001 v1.5 §4.8.3 模板（commit 32d9eb6）+ BAS-003 v0.3 样板（commit 75a001c）+ BAS-004 v0.3 §4.2 二维矩阵 + §4.3 字段 + §5.1 脱敏 + §6.2 强制全采样（commit 47e26b0/0ee6262）；覆盖 ARC-031 派生排行视图一致性边界 + FR-GSM-001〜044 / NFR-GSM-001〜006 的"派生视图增量更新/死信重建/赛季结算/任务奖励/邮件系统/举报黑名单/信誉度重算"全链路；显式区分 `info!`／`warn!`／`error!`（release 必出，编译期常驻，§6.2 强制全采样）与 `trace!`／`debug!`（`#[cfg(debug_assertions)]` 守护，debug-only，release build 完全剔除零运行时开销）两类事件；排行榜域特殊：排行榜更新／排名变化／刷新 → release 必出；任务进度更新／完成／奖励发放 → release 必出 + §6.2 强制全采样；成就解锁 → release 必出；作弊检测／封禁 → `error!` 强制全采样；玩家治理（禁言／封号／申诉）→ release 必出 + §6.2 强制全采样（合规审计）；§8.1 标准化检查清单新增 log 章节上线检查项；§9 追溯性新增 AC-GSM-006（debug-only 宏 release 完全剔除）与 AC-GSM-007（每功能 BAS 文档须含本功能 log 章节），与 BAS-001 v1.5 §4.8.3.4 / BAS-002 v0.4 §13 / BAS-003 v0.3 §13 / BAS-004 v0.3 §12 / BAS-006 v0.4 §9 / BAS-009 v0.7 §7 形成统一规范 | §3／§4／§5／§6／§7／§8.1／§9 |
| 0.4 | 2026-09-02 | 架构师 (Mavis 接手 agent per DEC-008) | 架构师 (Mavis 接手 agent per DEC-008) | 落实「処理フロー」段四要素标准 (per 2026-09-02 13:59 JST Ulysses 拍板, RGS-BAS-FLOW-STANDARD-2026-09-02 v0.1): 新增 §2 処理フロー（处理流程 / Processing Flow）段 (作为新 top-level 段插入 §1 之后, 既有 §2-§8 全部 +1 重编号为 §3-§9), 含主流程图 (mermaid sequenceDiagram, 11 actor: Player/RankingSource/RankingViewUpdater/RankingQueryService/QuestStateMachine/QuestRewardGranter/MailService/PlayerReport/GM/AuthoritativeFallback/DB, 覆盖 5 大主路径: 派生视图/任务成就/邮件/举报封禁/赛季切换) + 異常分支表 (12 行, 覆盖派生视图消费失败/排行榜查询越界/任务非法迁移/任务奖励发放失败/邮件过期已领取/黑名单越权查询/举报 dedup/赛季切换 in-flight/赛季结算中间态/补偿本身失败/GM 越权/反作弊) + 决策点矩阵 (9 行, 覆盖派生视图 vs 权威表/任务奖励路径/邮件领取/黑名单查询/举报处理/赛季切换 in-flight/赛季名次/信誉度重算/反作弊处理) + 验证点清单 (10 行, 与 §3.6/§4.4/§5.4/§6.3/§7.4 既定 5 个 log 设计小节呼应); trace_id 贯穿全链路 (per BAS-004 v0.3 §4.4); 事务边界与 Saga 跨域标注 (per BAS-100 v0.1, 任务奖励 + 邮件附件 + 赛季奖励发放同事务, 跨域走 Saga); ARC-031 一致性边界在 §2 顶部明文标注; 与既有 §3 排行榜 / §4 任务成就 / §5 邮件 / §6 举报封禁 / §7 赛季 互为详细化引用, §2 为跨模块全景流程 + 异常分支 + 决策点 + 验证点汇总; v0.3 修订历史 §N 引用同步重编号 (§2→§3 等) | §2、§9 |
| 0.5 | 2026-09-02 | 架构师 (Mavis 接手 agent per DEC-008) | 架构师 (Mavis 接手 agent per DEC-008) | 「補缺口」spot-check 修复 v0.4 重编号遗漏的 11 处内部交叉引用 (per 2026-09-02 15:02 JST Ulysses 拍板, RGS-WEEKLY-2026-W36_v0.1 §3 缺口 1): 派生视图全量重建 (L283 §2.3.1 → §3.3.1) + 重建降级 (L285 §2.3.1 → §3.3.1) + query_served 采样率对比 (L354 §2.6 → §3.6) + 排行榜滞后监控检查 (L574 §2.5 → §3.5) + 派生视图死信/重建路径运维手册 (L583 §2.3.1 → §3.3.1) + release 必出事件清单 5 段 (L588 §2.6/§3.4/§4.4/§5.3/§6.4 → §3.6/§4.4/§5.4/§6.3/§7.4) + 举报治理 3 段域约束 (L590-592 §5.3 → §6.3) + 赛季 2 段域约束 (L593-594 §6.4 → §7.4); 跨文档引用 (BAS-001/BAS-003/BAS-004/BAS-005/BAS-009/RGS-REQ-017) 不动; spotcheck 验证报告 docs/14-项目治理/.bas-014-spotcheck-v2.txt (74 总引用, 0 重编号遗漏) | §3-§9 |

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 | 2026-08-16 | — |
| 评审（技术） | | | 派生视图更新时序是否与既有ARC-009事件基础设施的幂等保证一致 |
| 审批（负责人） | | | 本文档的基准化 |

---

## 目录

1. [前言](#1-前言)
2. [処理フロー（处理流程 / Processing Flow）](#2-処理フロー处理流程--processing-flow)
   - 2.1 [主流程图 (mermaid sequenceDiagram)](#21-主流程图-mermaid-sequencediagram)
   - 2.2 [異常分支表](#22-異常分支表)
   - 2.3 [决策点矩阵](#23-决策点矩阵)
   - 2.4 [验证点清单](#24-验证点清单)
3. [排行榜：派生视图组件设计](#3-排行榜派生视图组件设计)
4. [任务与成就：配置化触发引擎设计](#4-任务与成就配置化触发引擎设计)
5. [邮件系统：数据模型](#5-邮件系统数据模型)
6. [举报与黑名单：字段级设计](#6-举报与黑名单字段级设计)
7. [赛季与段位：重置时序](#7-赛季与段位重置时序)
8. [标准化检查清单](#8-标准化检查清单)
9. [追溯性](#9-追溯性)

---

# 1. 前言

本文档细化RGS-REQ-017定义的ARC-031（派生排行视图的一致性边界）及其配套的四个功能模块，遵循ARC-018挂载原则——本文档定义的全部组件均**依附**既有限界上下文（EC／GD／MT／AD）运行，**不新建**独立限界上下文、独立数据库或独立部署单元。

命名约定：本文档中的字段级设计以"逻辑字段"表述，物理DDL遵循RGS-BAS-007既定的数据库设计标准（命名规范、索引/分区标准）执行，不在本文档重复定义。

---



# 2. 処理フロー（处理流程 / Processing Flow）

> 落实 RGS-BAS-FLOW-STANDARD-2026-09-02 v0.1 四要素标准 (per 2026-09-02 13:59 JST Ulysses 拍板)
> 本文档覆盖多模块（排行榜派生视图 / 任务成就 / 邮件 / 举报封禁 / 赛季段位），§2 为跨模块全景流程 + 异常分支 + 决策点 + 验证点汇总；各模块详细时序见 §3/§4/§5/§6/§7

## 2.1 主流程图 (mermaid sequenceDiagram)

```mermaid
sequenceDiagram
    autonumber
    actor Player as 玩家
    participant RS as RankingSource (EC/GD)
    participant RVU as RankingViewUpdater
    participant RVS as RankingQueryService
    participant ZSM as QuestStateMachine
    participant QRG as QuestRewardGranter
    participant MS as MailService
    participant PR as PlayerReport
    participant GM as GM 运营后台
    participant AFM as AuthoritativeFallback
    participant DB as player_db/social_db

    Note over Player,DB: trace_id 贯穿全链路, per BAS-004 v0.3 §4.4
    Note over Player,DB: 事务边界: 排名更新最终一致性 (NFR-GSM-002 滞后可接受); 任务奖励 + 邮件附件 + 赛季奖励发放同事务; 跨域走 Saga, per BAS-100 v0.1
    Note over Player,DB: ARC-031 一致性边界: 常态展示走派生视图, 赛季结算/GM 查询必须回落权威数据源

    rect rgb(240, 248, 255)
        Note over Player,DB: 主路径 1: 排行榜派生视图 (per §3 详细时序)
        Player->>RS: 业务事件 (升级/赛季积分/工会声望等)
        RS->>RS: 计算 RankingScore
        RS-->>RVU: 发布 RankingScoreChanged 事件 (ARC-010 基础设施)
        RVU->>RVS: 更新派生视图 (有序集合)
        RVS-->>Player: 排行榜查询 (可能滞后 NFR-GSM-002)
    end

    rect rgb(255, 250, 240)
        Note over Player,DB: 主路径 2: 任务/成就进度与奖励 (per §4 详细时序)
        Player->>ZSM: 任务事件触发 (QuestConditionSubscriber)
        ZSM->>ZSM: 校验状态机合法迁移 (FR-GSM-012)
        alt 非法迁移
            ZSM-->>Player: 拒绝
        else 合法迁移
            ZSM->>ZSM: 状态机推进 + 进度更新
            alt 任务完成
                ZSM->>QRG: 触发奖励发放 (FR-EC-003 路径, 复用不发旁路)
                QRG->>DB: BEGIN 原子事务
                QRG->>DB: 插入 RewardGrantLog
                QRG->>DB: COMMIT
                QRG-->>ZSM: 奖励发放成功
                ZSM-->>Player: 任务完成 + 奖励
            end
        end
    end

    rect rgb(240, 255, 240)
        Note over Player,DB: 主路径 3: 邮件系统 (per §5 详细时序)
        Player->>MS: 接收系统/业务邮件
        MS->>DB: 写入 MailMessage
        alt 邮件含附件
            Player->>MS: 领取附件
            MS->>DB: 校验未过期 + 未领取
            alt 校验通过
                MS->>DB: 标记已领取 (同事务)
                MS-->>Player: 附件发放 (复用 FR-EC-003)
            else 校验失败
                MS-->>Player: 拒绝
            end
        end
    end

    rect rgb(255, 240, 240)
        Note over Player,DB: 主路径 4: 举报与封禁 (per §6 详细时序)
        Player->>PR: 提交举报 (FR-GSM-030)
        PR->>DB: 写入 PlayerReport (dedup_key 唯一索引, FR-GSM-033)
        Note over PR,GM: 举报不直接调用 AdminService, GM 人工处理 (per RGS-REQ-014 仲裁层, ARC-030 闸门)
        GM->>PR: GM 审核 (审计记录 rank.report.audit_recorded)
        alt 举报成立
            GM->>GM: 封禁/禁言 (rank.governance.ban_issued / mute_issued)
        else 举报不成立
            GM->>PR: 标记 unsubstantiated / dismissed
        end
    end

    rect rgb(248, 240, 255)
        Note over Player,DB: 主路径 5: 赛季结算与切换 (per §7 详细时序)
        Player->>AFM: 赛季边界 T 时刻 (tick 边界原子切换, ARC-016)
        AFM->>AFM: 校验 in-flight 比赛归属
        AFM->>DB: BEGIN 原子事务
        AFM->>DB: 写入 SeasonSettlementLog
        AFM->>DB: 推进 season 状态 + 重置权威分
        AFM->>DB: COMMIT
        AFM-->>Player: 赛季切换完成 + 名次判定 (回落权威表, FR-GSM-006)
    end

    Note over Player,DB: 异常通路 (DLQ + 重试): 派生视图消费失败 -> ARC-009 消费者标准模式 (重试 3 次 指数退避 100/200/400ms) -> DLQ 报警 -> 视图重建
```

## 2.2 異常分支表

| 异常点 | 触发条件 | 处理动作 | 用户感知 | 补偿动作 |
|---|---|---|---|---|
| 派生视图消费失败 | `RankingViewUpdater` 消费 `RankingScoreChanged` 失败 (缓存瞬时不可用) | 按 ARC-009 重试 3 次, 仍失败投 DLQ (per §3.3.1) | 排行榜查询滞后 (NFR-GSM-002 阈值内) | DLQ 报警 + 视图重建 (重算 `RankingDimensionConfig.source_event` 对应权威数据) |
| 排行榜查询越界 | GM/玩家查询非权限范围 (NFR-GSM-005 黑名单越权查询) | `rank.blocklist.invalid_query_rejected` 强制全采样 `warn!` | 拒绝查询 | 无 (写审计) |
| 任务非法迁移 | 状态机拒绝非法迁移 (FR-GSM-012) | `rank.quest.illegal_transition_rejected` 强制全采样 `warn!` | 提示"操作不合法" | 无 (客户端重试合法操作) |
| 任务奖励发放失败 | FR-EC-003 路径任一步失败 | 整体回滚, 任务状态保持"待领取" | 提示"服务暂不可用" | Saga 补偿 + 客户端重试 (幂等) |
| 邮件过期/已领取 | 玩家领取邮件附件时 `expire_at < now()` 或 `claimed_at IS NOT NULL` | 拒绝领取 | 提示"邮件已过期/已领取" | 无 (玩家放弃) |
| 黑名单越权查询 | 非 `owner_id` 查得 `PlayerBlocklist` 内容 (NFR-GSM-005) | 拒绝查询, `rank.blocklist.invalid_query_rejected` `warn!` 强制全采样 | 拒绝 | 写审计 (per §6.3 域约束) |
| 举报 dedup 冲突 | `PlayerReport.dedup_key` 唯一索引冲突 (FR-GSM-033) | 直接返回既有记录, 不重复写入 | 重复举报无副作用 | 无 (幂等) |
| 赛季切换 in-flight 比赛 | 切换时刻存在未结束比赛 (FR-GSM-044 未定义行为防护) | `rank.season.inflight_match_orphan` `error!` 强制全采样 | 比赛按 tick 边界规则归属 (per §7.3) | 写审计, GM 介入确认 |
| 赛季结算中间态 | 跨域原子操作违反 (FR-GSM-040) | `rank.season.partial_settlement_detected` `error!` 强制全采样 | 提示"赛季切换异常" | 整体回滚 + GM 人工核对 |
| 补偿本身失败 (RSK-GSM-002 信誉度异步重算) | `ReporterReputation` 异步重算失败 | 重试 3 次, 仍失败入 DLQ | 信誉度延迟更新 | 人工重算 (per §6.1.1) |
| GM 越权操作 | 非 `gm_operator`/`gm_admin` 角色尝试封禁/禁言 | 拒绝 + 写 `rank.governance.unauthorized_attempt` 审计 | 拒绝 | 无 (审计回溯) |
| 反作弊检测 | 反作弊规则触发 (FR-GSM-044) | `rank.governance.cheat_detected` `error!` 强制全采样 | 用户可能被封 | 人工复核 (per RGS-REQ-014) |

## 2.3 决策点矩阵

| 决策点 | 条件 | 主分支 | 备选分支 | 触发后果 |
|---|---|---|---|---|
| 派生视图 vs 权威表 | 查询类型 (常态展示 / 赛季结算 / GM 查询) | 常态展示 → 派生视图 (允许滞后) | 赛季结算/GM → 回落权威表 (per ARC-031 / FR-GSM-006) | 用户感知: 实时 (常态) / 准确 (赛季结算) |
| 任务奖励发放路径 | 任务类型 (虚拟物品 / 货币 / 称号) + 玩家状态 | FR-EC-003 路径 (复用不发旁路, FR-GSM-014) | 独立发放 (错误, 必须拒绝) | 财务一致性 + 审计完整 |
| 邮件附件领取 | `expire_at > now` AND `claimed_at IS NULL` | 允许领取 (同事务标记) | 拒绝 (过期/已领) | 用户感知: 领取成功 / 拒绝 |
| 黑名单查询 | `querying_player == owner_id` | 返回 `PlayerBlocklist` 内容 | 拒绝 + 审计 (NFR-GSM-005) | 用户感知: 查询成功 / 拒绝 + 写审计 |
| 举报处理 | 举报类型 (cheating/harassment/inappropriate_name) + dedup_key | 写入 + GM 人工处理 (per ARC-030) | 自动调用 AdminService (错误, 必须拒绝) | 治理合规 + 全链路审计 |
| 赛季切换 in-flight 比赛 | 切换时刻是否存在未结束比赛 | 按 tick 边界原子切换 (per §7.3) | 强制中断 (错误, 会影响玩家体验) | 用户感知: 比赛正常归属 (新赛季) |
| 赛季名次判定 | 派生视图 vs 权威表 | 回落权威表 (FR-GSM-006 赛季结算) | 派生视图 (滞后可能导致名次错判) | 赛季奖励准确 |
| 信誉度重算策略 | `ReporterReputation` 计算复杂度 (RSK-GSM-002) | 异步重算 (不阻塞举报提交) | 同步重算 (性能瓶颈) | 用户感知: 举报实时提交 / 信誉度延迟更新 |
| 反作弊处理 | 反作弊规则命中 (FR-GSM-044) | `error!` 强制全采样 + GM 复核 (per RGS-REQ-014) | 静默处理 (错误, 必须 production 可见) | 合规审计 + 行为画像 |

## 2.4 验证点清单

| 验证时机 | 验证内容 | 通过标准 | 失败处理 |
|---|---|---|---|
| 派生视图更新 (RankingViewUpdater) | 事件 `event_ts >= view.last_update_ts` (幂等键保证乱序到达不覆盖) | 严格大于等于 | 拒绝写入, 记录 `rank.leaderboard.stale_event_rejected` |
| 任务状态机迁移 (FR-GSM-012) | `from_state -> to_state` 合法迁移 | 状态机定义允许 | 拒绝迁移, 记录 `rank.quest.illegal_transition_rejected` (强制全采样) |
| 任务奖励发放 (FR-GSM-014) | FR-EC-003 路径返回 success | success = true | 整体回滚, 任务状态保持"待领取", 记录 `rank.quest.reward_grant_failed` |
| 邮件附件领取 | `expire_at > now()` AND `claimed_at IS NULL` | 严格大于 + 为空 | 拒绝, 记录 `rank.mail.attachment_claim_failed` |
| 黑名单查询 (NFR-GSM-005) | `querying_player == owner_id` | 严格相等 | 拒绝 + 写审计, 记录 `rank.blocklist.invalid_query_rejected` (强制全采样) |
| 举报 dedup (FR-GSM-033) | `dedup_key` 唯一索引 | 0 行 (首次) 或 1 行 (已存在, 走幂等) | 不重复写入, 记录 `rank.report.duplicate_rejected` |
| 赛季切换 in-flight 比赛 (FR-GSM-044) | 切换时刻无未结束比赛 (per §7.3 归属规则) | 全部归属确定 | `rank.season.inflight_match_orphan` (强制全采样) |
| 赛季结算原子提交 (FR-GSM-040) | 跨域原子操作全部完成 | 全部 COMMIT | 整体回滚, 记录 `rank.season.partial_settlement_detected` (强制全采样) |
| 举报合规审计 (per §6.3 域约束) | GM 操作记录 `rank.governance.ban_issued` / `mute_issued` | 操作记录入库 | 不允许操作 (per ARC-030 闸门) |
| 作弊检测 (per §6.3 域约束) | `rank.governance.cheat_detected` `error!` 强制全采样 | 触发即记录 | 不允许静默 (合规审计要求) |

---
# 3. 排行榜：派生视图组件设计

## 3.1 组件划分

| 组件 | 归属限界上下文 | 职责 |
|---|---|---|
| `RankingSource` | EC／GD（权威数据所在上下文） | 权威分数变更时发布`RankingScoreChanged`事件（复用ARC-010事件基础设施），**不直接**写入排行视图 |
| `RankingViewUpdater` | 缓存基础设施（ARC-012既有缓存边界的具体化） | 订阅`RankingScoreChanged`，对增量变更做局部重排序写入派生视图 |
| `RankingQueryService` | 依附GD/RT既有API网关路由 | 对外提供分页查询与"附近排名"查询（FR-GSM-004），**只读**派生视图，不触达权威表 |
| `RankingAuthoritativeFallback` | EC／GD（权威数据所在上下文） | 仅在赛季结算（FR-GSM-006）时点被调用，从权威表直接计算最终名次 |

## 3.2 派生视图的数据结构（TBD-GSM-001待评审前的默认方案）

复用ARC-012既定缓存基础设施的有序集合能力（如有序集合类型的键值存储）作为默认方案，键为`ranking:{维度}:{赛季ID}`，成员为玩家ID，分值为对应维度分数。选型最终确定见TBD-GSM-001（ISS-046）。

### 3.2.1 排行维度可配置扩展（FR-GSM-001字段级设计）

新增维度不得硬编码，须通过配置表`RankingDimensionConfig`声明（复用ARC-016数值表热更新分发机制，不修改`RankingViewUpdater`代码路径）：

| 字段 | 类型 | 说明 |
|---|---|---|
| `dimension_id` | string | 唯一标识，如`player_level`／`season_score`／`guild_prestige`（PH-6+） |
| `source_context` | enum(`EC`／`GD`／`MT`) | 权威分数来源的限界上下文，决定`RankingSource`订阅哪个事件流 |
| `source_event` | string | 触发分数变更的事件类型（如`PlayerLevelUp`、`SeasonScoreSettled`），须为ARC-010既定事件流中已存在的事件 |
| `score_field_path` | string | 从事件载荷中提取分数值的字段路径 |
| `season_scoped` | bool | 是否随赛季重置（`season_score`为`true`，`player_level`为`false`） |
| `enabled` | bool | 是否对外暴露查询，用于灰度上线新维度而不影响既有维度 |

新增一种排行维度仅需新增一行`RankingDimensionConfig`配置并声明其`source_event`订阅，`RankingSource`/`RankingViewUpdater`按配置驱动，不需为新维度编写专属分支代码，满足FR-GSM-001"应当可配置扩展"要求。

## 3.3 更新时序（增量式，FR-GSM-003）

```
权威数据变更（如玩家升级/赛季积分结算）
  → RankingSource发布RankingScoreChanged{player_id, dimension, new_score}（ARC-010事件基础设施，至少一次投递）
  → RankingViewUpdater消费事件（幂等：以player_id+dimension为幂等键，重复投递不产生重复排序副作用）
  → 对派生视图做单条成员分值更新（局部操作，不全量重算）
  → 更新完成后记录本次更新时间戳，供NFR-GSM-002滞后监控读取
```

> 消费失败重试与死信处理复用ARC-009既定的事件消费者标准模式，不新增专属基础设施。

### 3.3.1 异常分支

```
RankingViewUpdater消费RankingScoreChanged失败（如缓存基础设施瞬时不可用）
  → 按ARC-009标准重试策略重试N次
  → 仍失败 → 投递至既有死信队列（复用ARC-009死信处理），记录告警（RGS-BAS-003§6）
  → 死信事件不阻塞后续事件消费（幂等键保证乱序到达也不产生错误覆盖：仅当new_score对应的事件时间戳晚于视图当前记录的最后更新时间时才写入，防止死信重放导致的乱序覆盖新数据）
  → 运维/告警响应后，可触发"视图重建"（从权威表按`RankingDimensionConfig.source_event`对应的权威数据全量重算一次目标维度的派生视图，作为NFR-GSM-002滞后超限或死信事件堆积后的兜底恢复手段，重建期间该维度查询**应当**降级提示"数据更新中"而非报错）
```

## 3.4 一致性边界的落地规则（ARC-031核心约束）

| 场景 | 是否可用派生视图 | 依据 |
|---|---|---|
| 常态排行榜展示（榜单/附近排名查询） | **可以**，允许NFR-GSM-002定义的滞后 | FR-GSM-002 |
| 赛季结算的名次判定（决定奖励发放） | **不可以**，必须回落权威数据源 | FR-GSM-006、FR-GSM-043 |
| GM后台查询某玩家当前分数 | **不可以**，直接查权威表（低频操作，无性能顾虑） | ARC-031决定 |

## 3.5 滞后监控

`RankingViewUpdater`每次更新记录`last_update_lag_ms`（事件产生时间与视图更新完成时间之差），接入RGS-BAS-004既有黄金指标体系，超过NFR-GSM-002阈值告警（复用RGS-BAS-003§6告警推送通道）。**须与派生视图功能同批上线，不得后补**（RSK-031缓解措施）。

### 3.6 本功能日志设计

本节覆盖**派生排行视图**全链路的观察点——`RankingSource` 发布 → `RankingViewUpdater` 消费 → `RankingQueryService` 查询 → 死信/重建四个环节产生 release 必出事件，便于 SRE 在 Grafana 上按 `rank.leaderboard.*` 维度聚合派生视图的健康度（事件吞吐/消费滞后/死信堆积/重建耗时）。**排行榜域按本 BAS 域特殊考虑：排行榜更新／排名变化／刷新 → release 必出**（per BAS-001 v1.5 §4.8.3.4 模板 + 本节域约束），不允许降级为 debug-only——派生视图的最终一致性是玩家可见行为，事件必须 production 可见。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `rank.leaderboard.score_changed_published` | `RankingSource` 发布 `RankingScoreChanged` 事件（权威分数变更触发） | 与 EC／GD 写流量挂钩，~10-100/s（峰值） | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2） | 含 `player_id`（哈希化）／`dimension_id`／`new_score`；约 220B/条 × 100/s = ~22KB/s |
| `rank.leaderboard.view_updated` | `RankingViewUpdater` 完成派生视图单条成员分值更新（FR-GSM-003 增量式） | 同上，~10-100/s | release 必出（100% 强制全采样） | 含 `player_id`（哈希化）／`dimension_id`／`new_score`／`lag_ms`；约 240B/条 |
| `rank.leaderboard.query_served` | `RankingQueryService` 完成一次分页/附近排名查询（FR-GSM-004） | 与查询 QPS 挂钩，~50-500/s | release 必出（**采样 1%**，派生视图查询 QPS 高，全采样成本不可接受） | 含 `query_kind`（`page`／`neighbor`）／`dimension_id`／`result_count`／`served_ms`；约 200B/条 × 500/s × 1% = ~1KB/s |
| `rank.leaderboard.dead_letter_received` | `RankingViewUpdater` 消费 `RankingScoreChanged` 重试 N 次后仍失败，投递至既有死信队列（ARC-009） | 偶发（缓存瞬时不可用） | release 必出（100% 强制全采样） | 含 `event_id`／`dimension_id`／`last_error`；约 280B/条 |
| `rank.leaderboard.view_rebuild_started` | 运维响应死信堆积或 NFR-GSM-002 滞后超限，触发派生视图全量重建（§3.3.1） | 极少（生产事件） | release 必出（100% 强制全采样） | 含 `dimension_id`／`rebuild_kind`（`scheduled`／`dlq_recovery`／`lag_breach`）／`initiator`；约 300B/条 |
| `rank.leaderboard.view_rebuild_completed` | 派生视图全量重建完成 | 极少 | release 必出（100% 强制全采样） | 含 `dimension_id`／`rebuild_duration_ms`／`rebuilt_entry_count`；约 320B/条 |
| `rank.leaderboard.view_rebuild_degraded` | 重建期间该维度查询降级提示"数据更新中"（§3.3.1 兜底） | 极少 | release 必出（100% 强制全采样） | 含 `dimension_id`／`degraded_window_ms`；约 250B/条 |
| `rank.leaderboard.lag_breach` | `last_update_lag_ms` 超过 NFR-GSM-002 阈值（RSK-031 关注） | 偶发 | release 必出（100% 强制全采样，`warn!` 级别） | 含 `dimension_id`／`lag_ms`／`threshold_ms`；约 260B/条 |
| `rank.leaderboard.dimension_config_added` | `RankingDimensionConfig` 新增一行配置（FR-GSM-001 维度可配置） | 极低（每维度一次性） | release 必出（100% 强制全采样） | 含 `dimension_id`／`source_context`／`source_event`；约 280B/条 |
| `rank.leaderboard.dimension_config_toggled` | 既有维度的 `enabled` 字段切换（灰度上线/下线） | 极低 | release 必出（100% 强制全采样） | 含 `dimension_id`／`old_enabled`／`new_enabled`；约 240B/条 |
| `rank.leaderboard.authoritative_fallback_invoked` | `RankingAuthoritativeFallback` 在赛季结算时点被调用（FR-GSM-006，从权威表直接计算） | 极低（赛季结算一次性） | release 必出（100% 强制全采样） | 含 `dimension_id`／`season_id`／`invocation_reason`；约 300B/条 |
| `rank.leaderboard.debug.event_payload_dump` | 完整 `RankingScoreChanged` 事件载荷 dump（含 source_event 全字段） | 偶发（故障定位） | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除） | 约 500B-1KB/条（release 剔除，零运行时开销） |
| `rank.leaderboard.debug.view_diff` | 派生视图重建前后完整有序集合 diff（mermaid 序列化） | 极少 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 5-20KB/条（大型维度下 release 剔除） |
| `rank.leaderboard.debug.lag_timeseries` | `last_update_lag_ms` 完整时序 dump（用于滞后根因分析） | 偶发 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 2-5KB/条（release 剔除） |

**debug-only 守护要点**（落实 BAS-004 v0.3 §4.3 四铁律 + §4.4 释放必出宏清单）：
- `rank.leaderboard.score_changed_published` / `view_updated` / `dead_letter_received` 是**生产关键事件**（玩家可见的派生视图行为）—— release 必出 + §6.2 强制全采样，不挂 `#[cfg]`
- `rank.leaderboard.lag_breach` 是**警告信号**（NFR-GSM-002 SLA 违反）—— release 必出 + `warn!` 强制全采样
- `rank.leaderboard.query_served` 高频（500/s 峰值），强制全采样会撑爆日志通道—— 按 1% 采样率，但 §6.2 强制全采样的"安全审计事件"清单（认证失败／越权访问／敏感操作）不受此限
- `rank.leaderboard.debug.event_payload_dump` 在事件载荷包含 `attachments`（FR-GSM-021 邮件附件引用）时可能 1KB+ —— release 完全剔除
- `rank.leaderboard.debug.view_diff` 在大型维度（百万级成员）下 20KB+ —— release 完全剔除，避免 RUST_LOG=debug 误开时撑爆生产通道
- 字段最小集已含 `player_id` 哈希化处理（per BAS-004 v0.3 §5.1 末段 hash 规则），不暴露明文 ID
- 释放必出事件清单（强制 production 可见）：`score_changed_published`／`view_updated`／`dead_letter_received`／`view_rebuild_started`／`view_rebuild_completed`／`view_rebuild_degraded`／`lag_breach`／`dimension_config_added`／`dimension_config_toggled`／`authoritative_fallback_invoked` —— 10 个派生视图治理信号必须 production 可见

---

# 4. 任务与成就：配置化触发引擎设计

## 4.1 配置表结构（逻辑字段，复用ARC-016热更新分发）

`QuestDefinition`（任务/成就共享同一配置表结构，通过`category`字段区分）：

| 字段 | 类型 | 说明 |
|---|---|---|
| `quest_id` | string | 唯一标识 |
| `category` | enum(`quest`／`achievement`) | 任务或成就 |
| `trigger_condition` | 声明式表达式（如`event=ItemGranted AND item_type=monster_kill AND count>=100`） | 触发条件，**不得**要求编写专属订阅代码（FR-GSM-010） |
| `reset_policy` | enum(`never`／`season`／`period_days:N`) | 是否随赛季/周期重置（FR-GSM-014，成就默认`never`，任务默认`season`） |
| `reward_spec` | 引用既有物品/货币发放规格（复用FR-EC-003确定请求路径的入参结构） | 领奖时通过`RewardGrantService`发放 |

## 4.2 触发引擎组件

| 组件 | 职责 |
|---|---|
| `QuestConditionSubscriber` | 订阅ARC-010既定事件流（`ItemGranted`、对局结算等），按`trigger_condition`表达式匹配，**异步**更新任务进度（FR-GSM-015，不阻塞事件产生方） |
| `QuestProgressStore` | 持久化玩家任务进度，依附既有EC/GD上下文数据库，不新建独立库 |
| `QuestStateMachine` | 状态机：`可领取→已领取→进行中→已完成→已领奖`，非法迁移拒绝（FR-GSM-012，复用RGS-REQ-001第8章状态机纪律） |
| `QuestRewardGranter` | 复用FR-EC-003确定请求路径发放奖励，**不新设旁路**（FR-GSM-013） |

## 4.3 新增触发条件类型的扩展方式（NFR-GSM-004验证点）

新增一种触发条件类型仅需：①在`trigger_condition`表达式语法中新增操作符/字段（若有必要）②新增对应事件的订阅声明（配置项，非代码）。**不得**要求修改已有任务的代码路径，AC-GSM-003对此验证。

### 4.4 本功能日志设计

本节覆盖**任务与成就**全链路的观察点——`QuestConditionSubscriber` 触发匹配 → `QuestProgressStore` 进度更新 → `QuestStateMachine` 状态迁移 → `QuestRewardGranter` 奖励发放四个环节产生 release 必出事件，便于 SRE 在 Grafana 上按 `rank.quest.*` / `rank.achievement.*` 维度聚合任务/成就完成链路。**排行榜域按本 BAS 域特殊考虑：任务进度更新／完成／奖励发放 → release 必出 + §6.2 强制全采样；成就解锁 → release 必出**（per BAS-001 v1.5 §4.8.3.4 模板 + 本节域约束）——任务奖励发放涉及 FR-EC-003 物品/货币发放路径，玩家对奖励到达极其敏感，事件必须 production 可见 + 全链路可追溯。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `rank.quest.progress_updated` | `QuestProgressStore` 异步更新任务进度（FR-GSM-015 不阻塞事件产生方） | 与事件吞吐挂钩，~10-50/s（任务触发事件） | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2） | 含 `player_id`（哈希化）／`quest_id`／`old_progress`／`new_progress`；约 240B/条 × 50/s = ~12KB/s |
| `rank.quest.completed` | `QuestStateMachine` 迁移至"已完成"（FR-GSM-012） | 与任务完成挂钩，~1-10/s | release 必出（100% 强制全采样） | 含 `player_id`（哈希化）／`quest_id`／`category`（`quest`／`achievement`）；约 220B/条 |
| `rank.quest.reward_granted` | `QuestRewardGranter` 复用 FR-EC-003 路径发放奖励（FR-GSM-013） | 与任务完成挂钩，~1-10/s | release 必出（100% 强制全采样，奖励发放属生产关键事件） | 含 `player_id`（哈希化）／`quest_id`／`reward_spec`／`request_id`（与 FR-EC-003 串联）；约 320B/条 |
| `rank.quest.reward_grant_failed` | `QuestRewardGranter` 调用 FR-EC-003 失败（outbox 重试耗尽） | 极少 | release 必出（100% 强制全采样，`error!` 级别） | 含 `player_id`（哈希化）／`quest_id`／`error_kind`／`retry_count`；约 300B/条 |
| `rank.quest.illegal_transition_rejected` | `QuestStateMachine` 拒绝非法迁移（FR-GSM-012，复用 RGS-REQ-001§8 状态机纪律） | 配置错（应极少） | release 必出（100% 强制全采样，`warn!` 级别） | 含 `player_id`（哈希化）／`quest_id`／`attempted_transition`／`current_state`；约 280B/条 |
| `rank.quest.condition_evaluated` | `QuestConditionSubscriber` 完成一次 `trigger_condition` 表达式求值 | ~50-200/s（事件吞吐挂钩） | release 必出（**采样 5%**，高频事件） | 含 `quest_id`／`event_type`／`result`（`matched`／`not_matched`）／`eval_ms`；约 200B/条 × 200/s × 5% = ~2KB/s |
| `rank.quest.trigger_condition_registered` | 新增 `trigger_condition` 表达式类型（操作符/字段，NFR-GSM-004 扩展点） | 极低（配置变更） | release 必出（100% 强制全采样） | 含 `condition_id`／`expression`／`event_subscription`；约 350B/条 |
| `rank.achievement.unlocked` | 成就解锁（FR-GSM-014 永久保留，`reset_policy=never`） | 偶发（按玩家进度） | release 必出（100% 强制全采样） | 含 `player_id`（哈希化）／`achievement_id`／`unlocked_at`；约 240B/条 |
| `rank.achievement.reset_policy_changed` | `QuestDefinition.reset_policy` 字段修订（如 `season` → `never`） | 极低 | release 必出（100% 强制全采样，`warn!` 级别） | 含 `quest_id`／`old_policy`／`new_policy`／`affected_player_count`；约 320B/条 |
| `rank.quest.debug.condition_eval_trace` | 表达式求值中间步骤（每个子表达式结果） | ~50-200/s | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除） | 约 500B-1KB/条（release 剔除） |
| `rank.quest.debug.progress_snapshot` | 玩家全部进行中任务进度快照（用于故障定位） | 偶发 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 1-3KB/条（release 剔除） |
| `rank.quest.debug.reward_request_payload` | FR-EC-003 请求完整 payload（用于奖励发放链路追溯） | 偶发 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 800B-1.5KB/条（release 剔除） |

**debug-only 守护要点**：
- `rank.quest.reward_granted` / `reward_grant_failed` 是**奖励发放生产关键事件**—— release 必出 + §6.2 强制全采样，**不**挂 `#[cfg]`
- `rank.quest.illegal_transition_rejected` 是**状态机纪律违反信号**—— release 必出 + `warn!` 强制全采样，便于发现配置错误或外挂
- `rank.quest.condition_evaluated` 高频（200/s 峰值）—— 按 5% 采样率（高于 §3.6 `query_served` 的 1%，因表达式求值是任务/成就核心路径）
- `rank.quest.debug.condition_eval_trace` 涉及表达式 AST 多步求值输出—— release 完全剔除
- `rank.quest.debug.reward_request_payload` 可能含 `attachments` 引用—— release 完全剔除
- 释放必出事件清单（强制 production 可见）：`progress_updated`／`completed`／`reward_granted`／`reward_grant_failed`／`illegal_transition_rejected`／`trigger_condition_registered`／`achievement.unlocked`／`achievement.reset_policy_changed` —— 8 个任务/成就治理信号必须 production 可见

---

# 5. 邮件系统：数据模型

## 5.1 逻辑数据模型

`MailMessage`：

| 字段 | 类型 | 说明 |
|---|---|---|
| `mail_id` | uuid | 唯一标识 |
| `recipient_id` | 玩家ID | 收件人 |
| `mail_type` | enum(`system`／`business`) | 系统邮件（运营批量发放，复用FR-AD-002来源）／业务邮件（如交易失败退还），共享同一模型（FR-GSM-021） |
| `subject` / `body` | string | 标题/正文 |
| `attachments` | 引用既有物品/货币发放规格 | 领取时复用FR-EC-003确定请求路径（FR-GSM-022） |
| `read_status` | enum(`unread`／`read`) | 已读/未读 |
| `claim_status` | enum(`unclaimed`／`claimed`) | 附件是否已领取 |
| `expire_at` | timestamp | 保留期截止（默认90天，NFR-GSM-006，可配置） |

索引：`(recipient_id, read_status, expire_at)`复合索引支撑玩家收件箱列表的高频查询（按收件人过滤未读/未过期）；`(expire_at)`单列索引支撑§4.3按月度分区的到期清理批处理扫描。`mail_id`为主键，物理DDL细节遵循RGS-BAS-007命名规范。

## 5.2 批量发送

`MailBatchSender`接受目标条件（全服/玩家列表/满足特定条件的群体），**异步**逐条生成`MailMessage`（FR-GSM-024，复用RGS-BAS-003控制平面既有异步工单处理模式，不阻塞GM操作的即时响应）。

## 5.3 保留期清理

复用RGS-BAS-007§4既定的分区归档标准：`MailMessage`表按`expire_at`月度分区，到期分区归档/清理。清理前T-3天对未领取邮件触发到期提醒（复用既有通知/告警机制，FR-GSM-023）。

### 5.4 本功能日志设计

本节覆盖**邮件系统**全链路的观察点——`MailMessage` 写入 → `MailBatchSender` 批量发送 → 玩家领取 → `expire_at` 到期清理四个环节产生 release 必出事件，便于运营/客服按 `rank.mail.*` 维度追溯玩家邮件状态。**邮件系统涉及玩家公告/补偿/赛季奖励发放**，所有写入与领取事件按 BAS-004 v0.3 §6.2 强制全采样（运营审计需要完整链路），不允许降级为 debug-only。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `rank.mail.sent` | 单封 `MailMessage` 写入（系统邮件/业务邮件，FR-GSM-021） | 与业务挂钩，~1-100/s（峰值批量发放） | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2） | 含 `mail_id`／`recipient_id`（哈希化）／`mail_type`（`system`／`business`）／`source`（FR-AD-002 等）；约 280B/条 × 100/s = ~28KB/s |
| `rank.mail.batch_started` | `MailBatchSender` 启动批量发送任务（FR-GSM-024，异步） | 偶发（运营活动） | release 必出（100% 强制全采样） | 含 `batch_id`／`target_kind`（`all`／`player_list`／`condition`）／`target_count`；约 300B/条 |
| `rank.mail.batch_progress` | 批量发送进度回调（每 100/1000/5000 封一次） | 偶发 | release 必出（**采样 10%**） | 含 `batch_id`／`sent_so_far`／`target_count`；约 220B/条 |
| `rank.mail.batch_completed` | 批量发送任务完成 | 偶发 | release 必出（100% 强制全采样） | 含 `batch_id`／`final_sent_count`／`final_failed_count`／`duration_ms`；约 320B/条 |
| `rank.mail.batch_failed` | 批量发送任务失败（部分/全部未送达） | 极少 | release 必出（100% 强制全采样，`error!` 级别） | 含 `batch_id`／`failed_count`／`error_kind`；约 280B/条 |
| `rank.mail.attachments_claimed` | 玩家领取邮件附件（FR-GSM-022，复用 FR-EC-003 路径） | ~5-50/s | release 必出（100% 强制全采样，奖励领取属生产关键事件） | 含 `mail_id`／`player_id`（哈希化）／`attachment_spec`／`request_id`；约 320B/条 |
| `rank.mail.claim_failed` | 玩家领取邮件附件失败（FR-EC-003 调用失败/邮件已过期） | 偶发 | release 必出（100% 强制全采样，`warn!` 级别） | 含 `mail_id`／`player_id`（哈希化）／`error_kind`（`expired`／`already_claimed`／`ec_call_failed`）；约 320B/条 |
| `rank.mail.read_marked` | 玩家标记邮件已读 | ~5-50/s | release 必出（**采样 5%**，高频低价值事件） | 含 `mail_id`／`player_id`（哈希化）；约 200B/条 × 50/s × 5% = ~500B/s |
| `rank.mail.expiry_warning_sent` | 清理前 T-3 天对未领取邮件触发到期提醒（FR-GSM-023） | 偶发（按月分区） | release 必出（100% 强制全采样） | 含 `partition`（`YYYY-MM`）／`unclaimed_count`；约 240B/条 |
| `rank.mail.partition_archived` | `MailMessage` 表按 `expire_at` 月度分区归档/清理 | 偶发（每月一次） | release 必出（100% 强制全采样） | 含 `partition`／`archived_row_count`／`duration_ms`；约 280B/条 |
| `rank.mail.partition_archived_failed` | 分区归档/清理失败 | 极少 | release 必出（100% 强制全采样，`error!` 级别） | 含 `partition`／`error_kind`；约 260B/条 |
| `rank.mail.debug.batch_recipient_dump` | 批量发送的目标玩家列表 dump（用于活动复盘） | 极少 | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除） | 约 1-10KB/条（活动期间 release 剔除） |
| `rank.mail.debug.mail_body_dump` | 邮件 `subject` / `body` 完整内容 dump（用于内容审核/客诉） | 偶发 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 500B-2KB/条（release 剔除） |
| `rank.mail.debug.attachment_spec_dump` | `attachments` 完整规格 dump（FR-EC-003 调用 payload） | 偶发 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 800B-1.5KB/条（release 剔除） |

**debug-only 守护要点**：
- `rank.mail.sent` / `attachments_claimed` / `batch_completed` 是**邮件生产关键事件**（运营/玩家补偿/赛季奖励）—— release 必出 + §6.2 强制全采样，**不**挂 `#[cfg]`
- `rank.mail.batch_failed` / `partition_archived_failed` 是**生产异常信号**—— release 必出 + `error!` 强制全采样
- `rank.mail.read_marked` 高频（50/s 峰值），但业务价值低（仅已读状态变更）—— 按 5% 采样率
- `rank.mail.debug.batch_recipient_dump` 在全服活动（百万玩家）下 10MB+ —— release 完全剔除
- `rank.mail.debug.mail_body_dump` 可能含运营活动文案/补偿说明—— release 完全剔除
- 字段最小集已含 `recipient_id` / `player_id` 哈希化处理（per BAS-004 v0.3 §5.1 末段 hash 规则），不暴露明文 ID
- 释放必出事件清单（强制 production 可见）：`sent`／`batch_started`／`batch_completed`／`batch_failed`／`attachments_claimed`／`claim_failed`／`expiry_warning_sent`／`partition_archived`／`partition_archived_failed` —— 9 个邮件治理信号必须 production 可见

---

# 6. 举报与黑名单：字段级设计

## 6.1 举报

`PlayerReport`：

| 字段 | 类型 | 说明 |
|---|---|---|
| `report_id` | uuid | 唯一标识 |
| `reporter_id` / `target_id` | 玩家ID | 举报者/被举报者 |
| `report_type` | enum(`cheating`／`harassment`／`inappropriate_name`／`other`) | FR-GSM-030 |
| `context_ref` | 可选，对局ID/聊天记录ID | 上下文引用 |
| `dedup_key` | `reporter_id`+`target_id`+滚动时间窗口的哈希 | 用于FR-GSM-033去重统计，防止重复举报虚增信号强度 |
| `signal_weight` | decimal，默认1.0 | 该条举报计入信号强度的权重，受`ReporterReputation.weight_multiplier`折算（见下） |
| `created_at` | timestamp | 举报提交时间，参与滚动时间窗口计算与RSK-GSM-002持续追踪 |

索引：`(target_id, created_at)`复合索引支撑GM后台按被举报者聚合查询（RGS-BAS-003§3.4只读查询模式）；`(dedup_key)`唯一索引直接在数据库层面阻止同一滚动窗口内的重复计数写入，不依赖应用层去重逻辑单独兜底（FR-GSM-033）。

审计留痕复用RGS-BAS-003§7审计设计（存储结构与保留期一致），GM后台查询复用RGS-BAS-003§3.4只读查询模式。**处置路径**：举报仅产生信号，`PlayerReport`记录**不直接**触发任何`AdminService`调用；处罚必须由GM人工或RGS-REQ-014智能层（若启用，经ARC-030确定性闸门）显式调用既有`BanAccount`/`MuteChat`（FR-GSM-032）。

### 5.1.1 举报者信誉度（RSK-GSM-002缓解机制字段级设计）

`ReporterReputation`（依附既有GD/AD上下文数据库，不新建独立库）：

| 字段 | 类型 | 说明 |
|---|---|---|
| `reporter_id` | 玩家ID | 主键，一个玩家一条信誉度记录 |
| `substantiated_count` | int | 经GM/闸门判定为"举报属实并触发处罚"的历史次数 |
| `unsubstantiated_count` | int | 经GM判定为"举报不实/恶意"的历史次数 |
| `weight_multiplier` | decimal，范围[0.1, 1.5] | 该举报者未来举报计入信号强度的折算系数，按`substantiated_count`/`unsubstantiated_count`比例周期性重算（详细算法留待详细设计，本表仅定义数据结构与边界） |
| `updated_at` | timestamp | 最近一次信誉度重算时间 |

信誉度更新时机：GM在`AdminService`对举报作出处置决定（处罚或标记不实）时，**异步**触发`ReporterReputation`重算（复用ARC-009事件消费幂等机制，不阻塞GM操作），**不得**影响FR-GSM-032"举报本身不自动触发处罚"的既定原则——信誉度仅调节未来举报的信号权重，不构成对被举报者的处罚依据。

## 6.2 黑名单

`PlayerBlocklist`：

| 字段 | 类型 | 说明 |
|---|---|---|
| `owner_id` | 玩家ID | 黑名单所有者 |
| `blocked_id` | 玩家ID | 被拉黑者 |
| `created_at` | timestamp | 生效时间（即时生效） |

主键/唯一性约束：`(owner_id, blocked_id)`复合唯一索引，防止同一玩家对同一目标重复拉黑产生冗余行；`(owner_id)`单列索引支撑"查询自己的黑名单列表"（NFR-GSM-005唯一允许的查询路径）；**不得**在`blocked_id`上建立可被反查`owner_id`集合的索引/接口，避免NFR-GSM-005"不向第三方暴露谁拉黑了谁"被索引可用性间接绕过。

查询边界（NFR-GSM-005）：仅`owner_id`本人可查询自己的`PlayerBlocklist`，**不对**`blocked_id`（含被拉黑者本人）暴露该记录的存在性。

生效点：既有FR-LBY-011私聊路由在建立会话前须查询`PlayerBlocklist`（若`blocked_id`=发起方且`owner_id`=接收方，拒绝路由），组队邀请路径同理接入。**不影响**同一公开频道的可见性——黑名单不等同于隐身（FR-GSM-035）。

### 6.3 本功能日志设计

本节覆盖**举报/黑名单/信誉度/玩家治理**全链路的观察点——举报提交 → 信誉度重算 → GM 处罚（仅显式）→ 黑名单生效四个环节产生 release 必出事件。**玩家治理按本 BAS 域特殊考虑：玩家治理（禁言／封号／申诉）→ release 必出 + §6.2 强制全采样（合规审计）；作弊检测 → `error!` 强制全采样**（per BAS-001 v1.5 §4.8.3.4 模板 + 本节域约束）——所有 `AdminService` 处罚类调用产生的事件属**合规审计事件**（per BAS-004 v0.3 §6.2），必须 production 可见 + 全链路可追溯，不允许降级为 debug-only。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `rank.report.submitted` | `PlayerReport` 新增一行（玩家举报，FR-GSM-030） | ~1-20/s | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2） | 含 `report_id`／`reporter_id`（哈希化）／`target_id`（哈希化）／`report_type`（`cheating`／`harassment`／`inappropriate_name`／`other`）；约 280B/条 |
| `rank.report.duplicate_rejected` | 同一 `dedup_key` 在滚动时间窗口内被数据库唯一索引拒绝（FR-GSM-033） | 偶发 | release 必出（100% 强制全采样，`warn!` 级别） | 含 `dedup_key_hash`／`existing_report_id`／`rejected_reporter_id`（哈希化）；约 280B/条 |
| `rank.report.context_ref_resolved` | 举报的 `context_ref`（对局ID/聊天记录ID）成功解析为可审查的原始内容 | ~1-20/s | release 必出（**采样 10%**，含原始内容成本高） | 含 `report_id`／`context_ref_kind`（`match`／`chat`）；约 220B/条 |
| `rank.report.reputation_updated` | GM 处置后异步触发 `ReporterReputation` 重算（RSK-GSM-002） | 偶发 | release 必出（100% 强制全采样） | 含 `reporter_id`（哈希化）／`old_substantiated`／`new_substantiated`／`old_multiplier`／`new_multiplier`；约 320B/条 |
| `rank.report.audit_recorded` | 举报处置留痕写入审计层（复用 RGS-BAS-003§7 审计设计） | 与 GM 处置挂钩，~0.1-1/s | release 必出（100% 强制全采样，合规审计） | 含 `report_id`／`verifier_id`（GM 标识）／`disposition`（`substantiated`／`unsubstantiated`／`dismissed`）；约 320B/条 |
| `rank.governance.ban_issued` | `AdminService.BanAccount` 显式调用（FR-GSM-032 处罚，per ARC-030 确定性闸门） | 极低（GM 操作） | release 必出（100% 强制全采样，`error!` 级别，合规审计） | 含 `target_id`（哈希化）／`verifier_id`／`ban_duration`／`ban_reason`；约 360B/条 |
| `rank.governance.mute_issued` | `AdminService.MuteChat` 显式调用 | 极低 | release 必出（100% 强制全采样，合规审计） | 含 `target_id`（哈希化）／`verifier_id`／`mute_duration`／`mute_scope`；约 340B/条 |
| `rank.governance.appeal_received` | 玩家提交处罚申诉（合规审计关键事件） | 偶发 | release 必出（100% 强制全采样） | 含 `appeal_id`／`target_id`（哈希化）／`related_ban_id`／`appeal_text_ref`；约 300B/条 |
| `rank.governance.appeal_resolved` | GM/闸门处理申诉完成 | 偶发 | release 必出（100% 强制全采样） | 含 `appeal_id`／`verifier_id`／`disposition`（`uphold`／`overturn`／`partial`）；约 300B/条 |
| `rank.governance.cheat_detected` | 反作弊/对局异常检测告警触发（不直接处罚，仅信号） | 偶发 | release 必出（100% 强制全采样，`error!` 级别） | 含 `detector_id`／`target_id`（哈希化）／`match_id`／`signal_kind`／`confidence`；约 360B/条 |
| `rank.blocklist.added` | `PlayerBlocklist` 新增一行（拉黑） | ~0.5-5/s | release 必出（100% 强制全采样） | 含 `owner_id`（哈希化）／`blocked_id`（哈希化）／`created_at`；约 280B/条 |
| `rank.blocklist.removed` | 玩家解除拉黑 | ~0.1-1/s | release 必出（100% 强制全采样） | 含 `owner_id`（哈希化）／`blocked_id`（哈希化）；约 260B/条 |
| `rank.blocklist.route_blocked` | 既有路径（FR-LBY-011 私聊/组队邀请）因黑名单拒绝路由 | ~0.5-5/s | release 必出（**采样 5%**，高频低价值事件） | 含 `route_kind`（`chat`／`invite`）／`owner_id`（哈希化）／`blocked_id`（哈希化）；约 220B/条 |
| `rank.blocklist.duplicate_rejected` | 同一 `(owner_id, blocked_id)` 复合唯一索引拒绝重复拉黑 | 配置错 | release 必出（100% 强制全采样，`warn!` 级别） | 含 `owner_id`（哈希化）／`blocked_id`（哈希化）／`existing_row_id`；约 280B/条 |
| `rank.blocklist.invalid_query_rejected` | 非 `owner_id` 本人查询 `PlayerBlocklist`（NFR-GSM-005 反查防护） | 偶发（越权尝试） | release 必出（100% 强制全采样，`warn!` 级别，NFR-GSM-005 安全审计） | 含 `attempted_querier`（哈希化）／`attempted_owner_id`（哈希化）／`query_kind`；约 280B/条 |
| `rank.report.debug.reporter_reputation_breakdown` | `ReporterReputation` 重算明细（`substantiated_count`/`unsubstantiated_count`/`weight_multiplier` 三项的逐项数值与算法中间态） | 偶发 | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除） | 约 500B-1KB/条（release 剔除） |
| `rank.governance.debug.enforcement_chain` | `AdminService` 处罚调用链完整 dump（`request_id` 跨域串联） | 极少 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 1-3KB/条（release 剔除） |
| `rank.report.debug.appeal_text_dump` | 申诉原文完整 dump（用于客诉处理） | 偶发 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 1-5KB/条（release 剔除，可能含个人表达） |
| `rank.blocklist.debug.relationship_graph` | 玩家黑名单关系图 dump（用于关系网络分析） | 极少 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 1-10KB/条（release 剔除） |

**debug-only 守护要点**：
- `rank.governance.ban_issued` / `mute_issued` / `cheat_detected` 是**合规审计 + 反作弊生产关键事件**—— release 必出 + §6.2 强制全采样 + `error!` 级别（合规要求 production 完整可见，**不**挂 `#[cfg]`，**不**允许采样降级）
- `rank.report.submitted` / `reputation_updated` / `audit_recorded` 是**举报治理信号**—— release 必出 + §6.2 强制全采样，便于运营周报聚合
- `rank.governance.appeal_received` / `appeal_resolved` 是**合规审计关键事件**—— release 必出 + 强制全采样，便于法务追溯
- `rank.blocklist.invalid_query_rejected` 是**NFR-GSM-005 安全审计事件**—— release 必出 + `warn!` 强制全采样（越权尝试必须 production 可见）
- `rank.report.context_ref_resolved` 含原始对局/聊天内容，成本高—— 按 10% 采样
- `rank.blocklist.route_blocked` 高频（5/s 峰值），业务价值中等—— 按 5% 采样
- `rank.governance.debug.enforcement_chain` 涉及 `request_id` 跨域串联完整路径—— release 完全剔除
- `rank.report.debug.appeal_text_dump` 可能含玩家个人表达（情感宣泄/客诉细节）—— release 完全剔除
- 字段最小集已含 `reporter_id` / `target_id` / `owner_id` / `blocked_id` 哈希化处理（per BAS-004 v0.3 §5.1 末段 hash 规则），不暴露明文 ID
- 释放必出事件清单（强制 production 可见）：`report.submitted`／`report.duplicate_rejected`／`report.reputation_updated`／`report.audit_recorded`／`governance.ban_issued`／`governance.mute_issued`／`governance.appeal_received`／`governance.appeal_resolved`／`governance.cheat_detected`／`blocklist.added`／`blocklist.removed`／`blocklist.duplicate_rejected`／`blocklist.invalid_query_rejected` —— 13 个玩家治理/举报/黑名单治理信号必须 production 可见

---

# 7. 赛季与段位：重置时序

## 7.1 段位状态机

复用RGS-REQ-001第8章状态机纪律，段位迁移（如晋升/降级）**必须**经由既定规则计算，非法迁移（跳级晋升）拒绝（FR-GSM-041）。

## 7.2 赛季边界原子切换时序（复用ARC-016 tick边界原子切换思想）

```
赛季边界时刻T到达
  → 赛季切换协调者（依附既有调度基础设施，不新建独立组件）触发"赛季结算"流程
  → RankingAuthoritativeFallback对权威数据源计算最终名次（不使用派生视图滞后快照，FR-GSM-006/FR-GSM-043）
  → 按TBD-GSM-002确定的继承规则（清零/按比例保留/软重置区间）计算新赛季初始段位/积分
  → 幂等写入新赛季初始状态（重复触发不产生重复奖励，NFR-GSM-003）
  → 赛季奖励通过既有FR-EC-003确定请求路径发放（对应邮件系统或直接背包发放）
  → 原子提交：新赛季状态与旧赛季结算记录在同一事务边界内落地，不产生"部分玩家已按新赛季结算、部分仍按旧赛季"的中间态（FR-GSM-040）
```

## 7.3 切换时正在进行中的对局

赛季边界T到达前已开始、T之后结束的对局，其结算**归属规则**（按旧赛季结算，或不计入任何赛季）须在`QuestDefinition`/赛季配置中显式声明，不得产生未定义行为（FR-GSM-044）。默认规则：以对局**开始时间**所属赛季结算。

### 7.4 本功能日志设计

本节覆盖**赛季与段位**全链路的观察点——赛季边界到达 → 协调者触发结算 → `RankingAuthoritativeFallback` 计算最终名次 → 继承规则计算新赛季初始段位 → 奖励发放 → 原子提交六个环节产生 release 必出事件。**赛季结算属生产关键事件**（跨域原子操作 + 资产发放），所有环节按 BAS-004 v0.3 §6.2 强制全采样，便于事后追溯与对账。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `rank.season.boundary_reached` | 赛季边界时刻 T 到达（调度基础设施触发） | 极低（每赛季一次） | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2） | 含 `old_season_id`／`new_season_id`／`boundary_ts`；约 260B/条 |
| `rank.season.settlement_started` | 协调者启动赛季结算流程 | 极低 | release 必出（100% 强制全采样） | 含 `season_id`／`initiator`；约 220B/条 |
| `rank.season.authoritative_ranking_computed` | `RankingAuthoritativeFallback` 对权威数据源计算最终名次（FR-GSM-006/043） | 极低 | release 必出（100% 强制全采样） | 含 `season_id`／`dimension_id`／`computed_player_count`／`compute_duration_ms`；约 320B/条 |
| `rank.season.inheritance_rule_applied` | 按 TBD-GSM-002 继承规则（清零/按比例保留/软重置）计算新赛季初始段位 | 极低 | release 必出（100% 强制全采样） | 含 `season_id`／`rule_kind`（`reset`／`scaled`／`soft`）／`config_ref`；约 300B/条 |
| `rank.season.reward_distributed` | 赛季奖励通过 FR-EC-003 路径发放（FR-GSM-040/043） | 极低（一次性，~1/赛季） | release 必出（100% 强制全采样，奖励发放属生产关键事件） | 含 `season_id`／`reward_count`／`total_request_id_chain`；约 320B/条 |
| `rank.season.reward_distribution_failed` | 赛季奖励发放失败（部分/全部玩家） | 极少 | release 必出（100% 强制全采样，`error!` 级别） | 含 `season_id`／`failed_count`／`error_kind`；约 280B/条 |
| `rank.season.atomic_commit_succeeded` | 新赛季状态 + 旧赛季结算记录在同一事务边界内落地（FR-GSM-040） | 极低 | release 必出（100% 强制全采样） | 含 `season_id`／`commit_duration_ms`／`written_table_count`；约 280B/条 |
| `rank.season.atomic_commit_failed` | 跨表原子提交失败 | 极少 | release 必出（100% 强制全采样，`error!` 级别） | 含 `season_id`／`failed_table`／`error_kind`；约 280B/条 |
| `rank.season.idempotent_replay` | 赛季结算幂等触发（重复触发被识别，NFR-GSM-003） | 极少（运维/重试场景） | release 必出（100% 强制全采样） | 含 `season_id`／`replay_kind`（`scheduled_retry`／`manual_resume`）／`previous_request_id`；约 300B/条 |
| `rank.season.partial_settlement_detected` | 检出"部分玩家已按新赛季结算、部分仍按旧赛季"中间态（FR-GSM-040 反例） | 配置错（应零） | release 必出（100% 强制全采样，`error!` 级别，**阻断级**信号） | 含 `season_id`／`new_settled_count`／`old_settled_count`；约 320B/条 |
| `rank.season.inflight_match_classified` | 切换时正在进行中的对局按配置规则分类归属（FR-GSM-044） | 极低 | release 必出（100% 强制全采样） | 含 `match_id`／`start_season_id`／`end_season_id`／`rule`（`start_based`／`end_based`／`excluded`）；约 320B/条 |
| `rank.season.inflight_match_orphan` | 切换时正在进行中的对局未匹配任何归属规则（FR-GSM-044 违反） | 配置错（应零） | release 必出（100% 强制全采样，`error!` 级别） | 含 `match_id`／`start_ts`／`end_ts`；约 280B/条 |
| `rank.season.segment_promoted` | 单个玩家段位晋升（FR-GSM-041） | 与玩家挂钩，~10-1000/赛季 | release 必出（**采样 10%**，一次性事件但量大） | 含 `player_id`（哈希化）／`old_segment`／`new_segment`／`season_id`；约 280B/条 |
| `rank.season.segment_demoted` | 单个玩家段位降级（FR-GSM-041） | 与玩家挂钩，~10-1000/赛季 | release 必出（**采样 10%**） | 含 `player_id`（哈希化）／`old_segment`／`new_segment`／`season_id`；约 280B/条 |
| `rank.season.illegal_transition_rejected` | `QuestStateMachine` 拒绝段位跳级晋升等非法迁移（FR-GSM-041） | 配置错（应极少） | release 必出（100% 强制全采样，`warn!` 级别） | 含 `player_id`（哈希化）／`attempted_transition`／`current_segment`；约 300B/条 |
| `rank.season.boundary_config_changed` | 赛季边界 T / 继承规则 TBD-GSM-002 等配置变更 | 极低 | release 必出（100% 强制全采样） | 含 `old_boundary_ts`／`new_boundary_ts`／`old_rule`／`new_rule`；约 320B/条 |
| `rank.season.debug.settlement_breakdown` | 赛季结算每步耗时与逐项数值（权威计算/继承/奖励发放/原子提交） | 极低 | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除） | 约 1-3KB/条（release 剔除） |
| `rank.season.debug.inflight_match_classification` | 进行中对局按规则分类的完整决策路径（含每条规则匹配结果） | 极低 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 800B-2KB/条（release 剔除） |
| `rank.season.debug.player_segment_derivation` | 单玩家段位推算明细（旧分数→映射规则→新段位的逐步推导） | 极低 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 500B-1KB/条（release 剔除） |

**debug-only 守护要点**：
- `rank.season.*` 全部 release 必出—— 赛季结算属跨域原子操作 + 资产发放，所有环节必须 production 完整可见（per BAS-004 v0.3 §6.2），**不**挂 `#[cfg]`
- `rank.season.partial_settlement_detected` 是**FR-GSM-040 阻断级信号**（"部分玩家已按新赛季结算、部分仍按旧赛季"中间态）—— release 必出 + `error!` 强制全采样
- `rank.season.inflight_match_orphan` 是**FR-GSM-044 阻断级信号**（未匹配任何归属规则）—— release 必出 + `error!` 强制全采样
- `rank.season.reward_distributed` / `atomic_commit_succeeded` 是**生产关键事件**（赛季奖励 / 跨表原子提交）—— release 必出 + §6.2 强制全采样
- `rank.season.segment_promoted` / `segment_demoted` 量级 1000/赛季（百万玩家基线），全采样成本不可接受—— 按 10% 采样
- `rank.season.debug.settlement_breakdown` 在大型赛季下可能 3KB+ —— release 完全剔除
- 字段最小集已含 `player_id` 哈希化处理（per BAS-004 v0.3 §5.1 末段 hash 规则），不暴露明文 ID
- 释放必出事件清单（强制 production 可见）：`boundary_reached`／`settlement_started`／`authoritative_ranking_computed`／`inheritance_rule_applied`／`reward_distributed`／`reward_distribution_failed`／`atomic_commit_succeeded`／`atomic_commit_failed`／`idempotent_replay`／`partial_settlement_detected`／`inflight_match_classified`／`inflight_match_orphan`／`illegal_transition_rejected`／`boundary_config_changed` —— 14 个赛季/段位治理信号必须 production 可见

---

# 8. 标准化检查清单

## 8.1 上线前检查清单

- [ ] 排行榜滞后监控（§3.5）已与派生视图功能同批上线
- [ ] 派生视图数据结构选型已完成评审（TBD-GSM-001/ISS-046决议）
- [ ] 赛季结算路径的故障注入试验（中断后重触发）验证幂等，无重复/遗漏奖励
- [ ] 任务奖励、邮件附件领取路径均验证复用FR-EC-003，无独立发放旁路
- [ ] 黑名单查询边界验证：非`owner_id`无法查得黑名单内容
- [ ] 举报路径验证：单次举报不触发`AdminService`自动调用
- [ ] 赛季继承规则（TBD-GSM-002）已与策划评审确定并写入配置
- [ ] 举报处理SLA（TBD-GSM-003）已与运营团队评审确定
- [ ] 新增排行维度已通过`RankingDimensionConfig`配置验证，未修改`RankingViewUpdater`代码路径（FR-GSM-001）
- [ ] 派生视图死信/重建路径（§3.3.1）已具备可操作的运维手册，重建期间查询降级提示已实现
- [ ] `PlayerReport.dedup_key`唯一索引已在DDL中落地，未仅依赖应用层去重（FR-GSM-033）
- [ ] `PlayerBlocklist(owner_id, blocked_id)`唯一约束已落地，`blocked_id`侧无可反查`owner_id`的索引/接口（NFR-GSM-005）
- [ ] 注：`RankingViewUpdater`死信处理、`ReporterReputation`异步重算为本批新增的常态运维面，OLU运维负荷未核算，见ISS-065
- [ ] **每功能 BAS 文档均含"本功能 log 设计"章节**（per BAS-004 v0.3 §4.4 release 必出宏清单与各功能 §X.Y 对应），且 log 章节内明确区分 debug-only（`#[cfg(debug_assertions)]` 守护的 `debug!`/`trace!`）与 release 必出（`info!`/`warn!`/`error!`）两类事件
- [ ] **release 必出事件清单（§3.6／§4.4／§5.4／§6.3／§7.4 全部 5 个本功能 log 设计章节）** 逐项可在治理脚本 `scripts/check-docs-consistency.sh` 中 grep 验证（对应事件名 `rank.*`），未遗漏本域关键事件：派生视图更新（`rank.leaderboard.view_updated`）／任务奖励发放（`rank.quest.reward_granted`）／成就解锁（`rank.achievement.unlocked`）／举报合规审计（`rank.governance.ban_issued` / `rank.report.audit_recorded`）／赛季结算原子提交（`rank.season.atomic_commit_succeeded`）
- [ ] **debug-only 宏未守护 `info!`/`warn!`/`error!`**（per BAS-004 v0.3 §4.3 规则 #1 + §4.4 反例），CI 静态扫描（per BAS-004 v0.3 §9 第 6 项）通过
- [ ] **作弊检测事件**（`rank.governance.cheat_detected`）按域特殊考虑以 `error!` 强制全采样（per §6.3 域约束 + BAS-004 v0.3 §6.2），不允许降级为 debug-only / 不允许采样
- [ ] **玩家治理合规审计事件**（`rank.governance.ban_issued` / `mute_issued` / `appeal_received` / `appeal_resolved`）按域特殊考虑 release 必出 + 强制全采样（per §6.3 域约束 + BAS-004 v0.3 §6.2），不允许降级
- [ ] **NFR-GSM-005 黑名单越权查询防护**（`rank.blocklist.invalid_query_rejected`）以 `warn!` 强制全采样（per §6.3 域约束 + BAS-004 v0.3 §6.2），越权尝试必须 production 可见
- [ ] **FR-GSM-040 赛季结算中间态阻断**（`rank.season.partial_settlement_detected`）以 `error!` 强制全采样（per §7.4 域约束），跨域原子操作违反必须 production 可见
- [ ] **FR-GSM-044 进行中对局归属规则未定义防护**（`rank.season.inflight_match_orphan`）以 `error!` 强制全采样（per §7.4 域约束），未定义行为必须 production 可见

## 8.2 代码评审检查清单

- [ ] 排行榜查询路径未出现对权威表的实时全表排序查询
- [ ] 新增任务触发条件类型未修改已有任务代码路径
- [ ] 邮件系统未出现与FR-AD-002批量补偿重复的独立发放逻辑
- [ ] 赛季切换流程未出现跨越边界的非原子写入

---

# 9. 追溯性

| 需求ID | 本设计书章节 |
|---|---|
| ARC-031、FR-GSM-001〜006 | §3、§3.2.1（FR-GSM-001维度配置）、§3.3.1（死信/重建异常分支） |
| FR-GSM-010〜015 | §4 |
| FR-GSM-020〜024 | §5 |
| FR-GSM-030〜035 | §6、§6.1.1（RSK-GSM-002信誉度机制） |
| FR-GSM-040〜044 | §7 |
| NFR-GSM-001〜006 | §3.5、§4.3、§7.2 |
| AC-GSM-001〜005 | §8.1 |
| TBD-GSM-001〜003 | §3.2、§7.2、§8.1 |
| RSK-GSM-001〜002 | §3.5、§8.1 |
| **AC-GSM-006（debug-only 宏在 release build 完全剔除，零运行时开销）** | §3.6／§4.4／§5.4／§6.3／§7.4 全部 5 个本功能 log 设计章节中 `rank.*.debug.*` 字段（per BAS-004 v0.3 §4.2 二维矩阵 + §4.3 四条铁律 + §9 CI 第 5 项静态检查） | §3.6〜§7.4 |
| **AC-GSM-007（每功能 BAS 文档须含本功能 log 设计章节）** | §3.6／§4.4／§5.4／§6.3／§7.4 全部 5 个本功能 log 设计章节存在性 + §8.1 检查清单 log 章节上线检查项 + release 必出事件 grep 验证（per BAS-004 v0.3 §4.4 + §11.1） | §3.6〜§7.4、§8.1 检查清单 |
| **AC-GSM-008（処理フロー四要素）** | §2（mermaid sequenceDiagram 11 actor + 異常分支表 12 行 + 决策点矩阵 9 行 + 验证点清单 10 行，与 RGS-BAS-FLOW-STANDARD-2026-09-02 v0.1 §3 必含四要素一致；trace_id 贯穿全链路 per BAS-004 v0.3 §4.4；事务边界 + Saga 跨域标注 per BAS-100 v0.1；ARC-031 一致性边界明文标注；与 BAS-019 v0.4 §1.1 范式对齐；多模块汇总 §3-§7） | §2、§9 |

---

> 本文档与RGS-REQ-017（排行榜、任务成就与玩家治理 需求定义书）配套使用。
