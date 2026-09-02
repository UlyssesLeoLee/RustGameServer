# 基本设计书（基本設計書 / Basic Design Document）

**平台内购合规与服务器选服 Platform IAP Compliance & Realm Selection**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-BAS-020 |
| 版本 | 0.5 |
| 父文档 | RGS-REQ-023 需求定义书（ARC-038） |
| 制定日 | 2026-08-16 |
| 最终更新日 | 2026-08-16 |
| 制定者 | 架构师 |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响需求ID |
|---|---|---|---|---|---|
| 0.1 | 2026-08-16 | 架构师 | — | 初版制定。将RGS-REQ-023§9 ARC-038展开为收据校验组件设计与时序、选服路由设计、合服演练与执行流程 | 全部 |
| 0.2 | 2026-08-16 | 架构师 | — | 补强字段级细节：①补充平台官方验证接口不可用时的待重试队列设计（RSK-PLT-001）②补充`PaymentOrder`平台内购扩展字段与沙盒/生产环境隔离（FR-PLT-004、FR-PLT-005）③补充合服冲突解决规则的配置表字段级设计（FR-PLT-021） | FR-PLT-004、FR-PLT-005、FR-PLT-021、RSK-PLT-001 |
| 0.3 | 2026-08-17 | 架构师 | — | 审计发现FR-PLT-012"账号数据必须按逻辑服隔离"此前仅在§3选服路由中体现路由决策，未涉及数据归属键设计。新增§3.3，确立`realm_id`归属键原则与账号级/角色级数据归属维度的区分，具体表结构变更留待多服架构启用后详细设计阶段补齐 | FR-PLT-012 |
| 0.4 | 2026-09-01 | 架构师 (Mavis 接手 agent per DEC-008) | 架构师 (Mavis 接手 agent per DEC-008) | 落实"各BAS文档功能章节加log设计且区分debug/release级"总要求（per Ulysses 2026-09-01 15:52 JST 决策，4 拍板选项：全部 36 个BAS / 详尽版5列表 / 派worker并行 / BAS-004同步升级）：§2.1/§2.2/§2.3/§2.4/§2.5/§3.1/§3.2/§3.3/§4.1/§4.2/§5.1/§5.2 全部 12 个 ## L2 功能段加"本功能日志设计" 5 列详尽版（字段名/触发条件/频率估算/采样策略/脱敏与成本），引用 BAS-001 v1.5 §4.8.3 模板（commit 32d9eb6）+ BAS-003 v0.3 样板（commit 75a001c）+ BAS-004 v0.3 §4.2 二维矩阵 + §4.3 字段 + §5.1 脱敏 + §6.2 强制全采样（commit 47e26b0/0ee6262）；字段名前缀统一 `pay.*`（platform IAP compliance & realm selection 域），与 BAS-002 `mnt.*` / BAS-003 `ops.*` / BAS-006 `pay.*` 之前的命名空间隔离待统一（**已知缺口**：BAS-006 同样使用 `pay.*` 前缀，与本BAS-020的 `pay.*` 命名空间需在 ARCH 主会话后续以 RACI 矩阵统一协调）；显式区分合规审计强制项（`info!` 级别 release 必出 + §6.2 强制全采样，覆盖内购订单/退款/补单/选服主服选择/合规越权阻止/合服步骤/反违规告警）、安全告警（`error!` 强制全采样，覆盖跨服越权/反违规/资产不一致）、平台回调细节（`debug!`/`trace!` 级别 debug-only，`#[cfg(debug_assertions)]` 守护，release build 完全剔除零运行时开销，覆盖 webhook payload/平台官方响应/envelope dump）、算法性能/中间状态（`debug!` 守护，覆盖缓存命中/版本链快照）；**支付凭证/卡号/PayPal 账号 → 禁止记录**（per BAS-004 v0.3 §5.1 黑名单，本节所有 `pay.*` 字段集已规避，仅 debug 守护项中允许 envelope 结构）；覆盖 ARC-005/009/038 + FR-PLT-001〜005/010〜013/020〜023 + RSK-PLT-001 + TBD-PLT-001/002 + NFR-PLT-001/002/003/004 + NFR-OP-008 + FR-LOG-010/011/012/013/040 + AC-LOG-006/007 等全系列相关追溯依据；§6 追溯性新增 AC-PLT-006（`pay.*` debug-only 宏 release 完全剔除）与 AC-PLT-007（每功能段须含本功能 log 设计章节），与 BAS-001 v1.5 §4.8.3.4 / BAS-002 v0.4 §13 / BAS-003 v0.3 §13 / BAS-004 v0.3 §12 / BAS-010 v0.5 §7.1 形成统一规范 | §2.1〜2.5、§3.1〜3.3、§4.1〜4.2、§5.1〜5.2、§6 |
| 0.5 | 2026-09-02 | Ulysses — Mavis 接手 (per DEC-008) | 架构师 (Mavis 接手 agent per DEC-008) | 落实「処理フロー」段四要素标准 (per 2026-09-02 13:59 JST Ulysses 拍板, RGS-BAS-FLOW-STANDARD-2026-09-02 v0.1, commit `0db8507`, 范式 commit `d52eaad`): 本篇 4 要素已含 3 个 (异常分支/决策/验证需求已在 §2.1-§5.2 12 个本功能日志设计子节隐式覆盖), 补 1 个主流程图段。新增 §1.1 処理フロー（处理流程 / Processing Flow）段, 含主流程图 (mermaid sequenceDiagram, 9 actor: 玩家客户端 / ReceiptVerifier / AppStore-GooglePlay 平台官方 / PaymentOrder DB / Economy 域 / RefundNotificationHandler / RealmRouter / RealmDirectoryService / MergeConflictRuleSet) + 異常分支表 (8 行: 收据签名无效 / 沙盒-生产环境不匹配 / 平台接口不可用 / 重复唯一索引命中 / 跨服越权 / 选服目标服不可用 / 合服资产不一致 / 跳过演练直接正式执行) + 决策点矩阵 (6 行: 平台通道选择 / 幂等命中 vs 重新校验 / 退款追回模式 / 选服路由主服命中 vs 首次登录 / 合服冲突规则应用 / 演练 vs 正式执行) + 验证点清单 (7 行: 收据签名验证 / 沙盒-生产环境一致 / 幂等键匹配 / 退款签名验证 / 跨服 realm_id 一致性 / 合服资产一致性 / 选服目标服可分配), 覆盖收据校验 + 退款追回 + 选服路由 + 合服执行 4 个主路径; trace_id 贯穿全链路 (per BAS-004 v0.3 §4.4); 事务边界标注 (DB 写入同事务, 跨域 Economy 走 Saga per BAS-100 v0.1); 与既有 §2.2 收据校验时序 (文字流) / §2.5 退款处理时序 (文字流) / §3.2 选服时序 (文字流) / §4.2 合服执行流程 (文字流) 互为详细化引用; 保留 §5.1 注: PendingReceiptVerification 定时重试为常态运维面, OLU 运维负荷未核算 (ISS-065) | §1.1 |

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 | 2026-08-16 | — |
| 评审（技术） | | | 合服演练模式与正式执行的数据一致性保证是否充分 |
| 审批（负责人） | | | 本文档的基准化 |

---

## 目录

1. [前言](#1-前言)
   1.1 [処理フロー（处理流程 / Processing Flow）](#11-処理フロー处理流程--processing-flow)
2. [平台收据校验组件设计](#2-平台收据校验组件设计)
3. [选服路由设计](#3-选服路由设计)
4. [合服/分服执行流程](#4-合服分服执行流程)
5. [标准化检查清单](#5-标准化检查清单)
6. [追溯性](#6-追溯性)

---

# 1. 前言

本文档细化RGS-REQ-023定义的ARC-038，全部组件依附既有PL/AD限界上下文运行，不新建独立限界上下文。

### 1.1 処理フロー（处理流程 / Processing Flow）

> 落实 RGS-BAS-FLOW-STANDARD-2026-09-02 v0.1 四要素标准 (per 2026-09-02 13:59 JST Ulysses 拍板, 范式 commit `d52eaad` = BAS-019)
> 本篇 4 要素已含 3 个 (异常分支/决策/验证需求已在 §2.1-§5.2 12 个本功能日志设计子节隐式覆盖), 补 1 个主流程图段
> 详细时序见 §2.2 收据校验时序 / §2.5 退款处理时序 / §3.2 选服时序 / §4.2 合服执行流程, 本段为全景流程 + 异常分支 + 决策点 + 验证点汇总

#### 1.1.1 主流程图 (mermaid sequenceDiagram)

```mermaid
sequenceDiagram
    autonumber
    actor Client as 玩家客户端
    participant RV as ReceiptVerifier
    participant Plat as AppStore/GooglePlay 平台官方
    participant DB as PaymentOrder DB
    participant EC as Economy 域 (FR-EC-003)
    participant RH as RefundNotificationHandler
    participant RR as RealmRouter
    participant RDS as RealmDirectoryService
    participant MCR as MergeConflictRuleSet

    Note over Client,MCR: trace_id 贯穿全链路, per BAS-004 v0.3 §4.4
    Note over Client,MCR: 事务边界: PaymentOrder 写入 + 幂等索引 同事务; Economy 调用走 Saga 跨域 (per BAS-100 v0.1); 合服作业跨库事务由 §4.2 步骤 4 单事务保证

    rect rgb(240, 248, 255)
        Note over Client,EC: 主路径 1: 收据校验 + 权益发放 (per §2.2 详细时序)
        Client->>RV: 提交平台收据 (platform_type + raw_receipt)
        RV->>Plat: 平台官方接口验证 (App Store verifyReceipt / Google Play productPurchases.get)
        alt 4xx 签名无效 (status: 21002/21005)
            Plat-->>RV: 拒绝 (invalid_signature)
            RV-->>Client: 拒绝 "签名无效" (FR-PLT-004 失败原因分类)
        else 沙盒/生产环境不匹配
            Plat-->>RV: platform_environment ≠ server 配置
            RV-->>Client: 拒绝 "环境不匹配" (per §2.4 platform_environment 字段)
        else 5xx/超时 平台不可用
            Plat-->>RV: 超时/5xx
            RV->>DB: 写入 PendingReceiptVerification (status=pending, per §2.3 RSK-PLT-001)
            RV-->>Client: 投递待重试队列 (指数退避 100/200/400ms)
        else 验证成功
            Plat-->>RV: provider_txn_id
            RV->>DB: 查询 PaymentOrder (idempotency 键 = provider_txn_id)
            alt 幂等命中 (FR-PLT-005/ARC-009)
                DB-->>RV: 返回既有 PaymentOrder
                RV-->>Client: 直接返回既有结果 (不重复发放)
            else 新订单
                RV->>DB: 写入 PaymentOrder (含 platform_type + platform_environment 扩展字段, per §2.4)
                RV->>EC: 发放 reward_spec (Saga 跨域, per FR-EC-003)
                EC-->>RV: 发放成功
                RV-->>Client: 内购成功
            end
        end
    end

    rect rgb(255, 250, 240)
        Note over Client,DB: 主路径 2: 退款追回 (per §2.5 详细时序)
        Plat-->>RH: 异步推送退款通知 (App Store Server Notifications / Google Play RTDN)
        RH->>RH: 验证平台签名 (JWS / Pub/Sub message signature)
        alt 签名验证失败 (可能伪造)
            RH-->>Plat: 拒绝 (per NFR-PLT-002)
        else 签名通过
            RH->>DB: 依 provider_txn_id 关联 PaymentOrder
            alt 关联 miss
                RH->>RH: 记录 pay.refund.related_to_order.miss (异常)
            else 关联命中
                RH->>DB: 触发权益追回 (依 TBD-PLT-001 模式: deduct / mark_debt / skip)
                DB->>DB: 更新 refund_status 状态机 (none → clawback_pending → clawback_done, per §2.4)
                RH-->>Plat: 处理完成
            end
        end
    end

    rect rgb(248, 255, 248)
        Note over Client,RDS: 主路径 3: 选服路由 (per §3.2 详细时序, FR-PLT-012 realm_id 隔离)
        Client->>RR: 鉴权成功后路由请求 (含可选 realm_hint)
        RR->>DB: 查询账号 primary_realm_id
        alt 主服命中
            DB-->>RR: 返回 primary_realm_id
            RR-->>Client: 直接路由至主服 (跳过选服界面)
        else 首次登录
            RR->>RDS: 请求服务器列表 (含状态: normal/full/maintenance)
            RDS-->>RR: 返回可见服务器列表
            RR-->>Client: 展示选服界面
            Client->>RR: 玩家选择 chosen_realm_id
            RR->>DB: 写入 primary_realm_id (同事务 + 审计)
            RR-->>Client: 路由至选中服 (含 realm_id 注入会话上下文)
        end
        RR->>RR: realm_id 注入会话上下文 (per §3.3 ARC-005 服务器权威原则)
        Note over Client,RDS: 下游业务请求必须校验 realm_id 一致性, 防止跨服越权
    end

    rect rgb(255, 248, 248)
        Note over Client,MCR: 主路径 4: 合服执行 (per §4.1/§4.2 详细流程, FR-PLT-021)
        Note over Client,MCR: 合服 5 步: 评审 → 演练 → 演练评审 → 正式执行 → 被合并服退场
        MCR->>MCR: 步骤 1 评审锁定 (draft → locked, 运营+架构师签署)
        MCR->>MCR: 步骤 2 演练环境执行 (生产数据快照, 资产一致性校验)
        alt 演练失败
            MCR->>MCR: 回到步骤 1 修正规则 (FR-PLT-021 禁止跳过演练)
        else 演练通过
            MCR->>MCR: 步骤 3 演练结果评审
            MCR->>MCR: 步骤 4 维护窗口正式执行 (源服维护模式 + 数据合并 + 资产一致性校验)
            alt 资产不一致
                MCR->>MCR: 触发 pay.merge.job.asset_consistency_check_failed (SRE 立即介入)
            else 资产一致
                MCR->>MCR: 步骤 5 被合并服按 ARC-018 退场流程下线
            end
        end
    end

    Note over Client,MCR: 异常通路 (DLQ + 重试 + 告警): 平台 5xx -> PendingReceiptVerification 重试 3 次 -> 超限 abandoned + 转人工; 跨服越权 -> P1 安全告警; 跳过演练直接执行 -> 反违规告警
```

#### 1.1.2 異常分支表

| 异常点 | 触发条件 | 处理动作 | 用户感知 | 补偿动作 |
|---|---|---|---|---|
| 收据签名无效 | App Store status: 21002/21005 等 (FR-PLT-004 失败原因分类) | 拒绝内购, 写 audit (pay.receipt.verify.failed.invalid_signature) | 提示"签名无效" | 客户端重新获取收据 |
| 沙盒/生产环境不匹配 | platform_environment 字段与 server 配置不一致 (FR-PLT-004) | 拒绝内购, 写 audit (pay.payment_order.environment_mismatch, 反欺诈信号) | 提示"环境错误" | 客户端确认 SDK 配置 |
| 平台接口不可用 | 平台官方接口超时/5xx (区别于"签名无效") | 写入 PendingReceiptVerification 待重试队列 (RSK-PLT-001, 指数退避) | 延迟收到权益 | 定时任务重试, 超限 abandoned + 转人工 |
| 重复唯一索引命中 | (platform_type, provider_txn_id) 复合唯一索引命中 (FR-PLT-005) | 幂等返回既有 PaymentOrder (不重复发放, per ARC-009) | 直接成功 (无副作用) | 无 |
| 跨服越权访问 | 下游业务服务校验请求 realm_id ≠ 会话上下文 (per §3.3 FR-PLT-012) | 业务服务拒绝 + 写 audit (pay.realm.isolation.mismatch_detected, P1 安全告警) | 提示"服务暂不可用" | 客户端重新进入大厅 |
| 选服目标服不可用 | 玩家选择的 chosen_realm_id 状态非 normal (full/maintenance) | 拒绝路由 + 写 audit (pay.realm.route.realm_unavailable) | 提示"该服当前不可用" | 客户端重新选服 |
| 合服资产不一致 | 合服后 total_characters/total_inventory_items/total_currency 与演练 delta 不一致 (FR-PLT-021) | 触发 pay.merge.job.asset_consistency_check_failed (SRE 立即介入) | 玩家无感 (维护模式) | 回滚到维护模式, 回到步骤 1 修正 |
| 跳过演练直接正式执行 | 合服作业跳过步骤 2 演练直接进入步骤 4 (FR-PLT-021 明确禁止) | 触发 pay.merge.job.skipped_drill_attempt (反违规告警, P1) | 玩家无感 (维护模式) | 强制回到步骤 2 演练 |

#### 1.1.3 决策点矩阵

| 决策点 | 条件 | 主分支 | 备选分支 | 触发后果 |
|---|---|---|---|---|
| 平台通道选择 | platform_type 字段 (app_store / google_play) | app_store → App Store verifyReceipt; google_play → Google Play productPurchases.get | 强制单一平台 (运营限定) | 用户感知: 标准内购 / 限定内购 |
| 幂等命中 vs 重新校验 | PaymentOrder 查询 (provider_txn_id) 命中既有记录 (FR-PLT-005/ARC-009) | 命中: 直接返回既有结果 (不重复发放); 未命中: 走新订单流程 | 强制走重新校验 (怀疑幂等索引损坏) | 用户感知: 即时成功 / 重新发放 |
| 退款追回模式 | TBD-PLT-001 详设确定: deduct / mark_debt / skip | deduct: 扣除等价物; mark_debt: 标记负债; skip: 不追回 (合规要求) | 强制 deduct (财务优先) | 玩家感知: 资产扣减 / 负债标记 / 维持现状 |
| 选服路由: 主服命中 vs 首次登录 | 账号 primary_realm_id 记录存在性 (per §3.2) | 命中: 直接路由; 首次: 展示服务器列表 | 强制重新选服 (运营活动) | 用户感知: 直接进入游戏 / 选服界面 |
| 合服冲突规则应用 | MergeConflictRuleSet 已锁定 (per §4.1) | character_name_conflict: auto_rename / require_manual_rename; unique_item: stack_additively / keep_both / keep_earliest | 强制 auto_rename (运营效率) | 玩家感知: 角色自动改名 / 道具合并 / 道具并存 |
| 演练 vs 正式执行 | 合服作业步骤 1-5 顺序 (per §4.2, FR-PLT-021 强制) | 演练通过 → 正式执行; 演练失败 → 回到步骤 1 修正 | 跳过演练直接正式 (FR-PLT-021 禁止, 触发反违规告警) | 玩家感知: 维护时长; 内部: 合规追溯完整 / 反违规事件 |

#### 1.1.4 验证点清单

| 验证时机 | 验证内容 | 通过标准 | 失败处理 |
|---|---|---|---|
| 收据签名验证 | App Store status=0 / Google Play 有效 purchase state (FR-PLT-004) | 200 OK + 有效签名 + platform_environment 匹配 | 拒绝 (按失败原因分类: invalid_signature / env_mismatch / platform_unavailable) |
| 沙盒/生产环境一致 | platform_environment 字段与 server 配置一致 (per §2.4 FR-PLT-004) | sandbox ↔ sandbox; production ↔ production | 拒绝 + 写 audit (pay.payment_order.environment_mismatch, 反欺诈信号) |
| 幂等键匹配 | (platform_type, provider_txn_id) 复合唯一索引 (per §2.4 索引) | 0 行 (新订单) 或 1 行 (幂等命中) | 0 行: 走新订单流程; 1 行: 直接返回既有结果 |
| 退款签名验证 | App Store JWS / Google Play Pub/Sub message signature (per §2.5 NFR-PLT-002) | 签名有效 | 拒绝 (pay.refund.signature.failed, 可能伪造通知攻击) |
| 跨服 realm_id 一致性 | 下游业务请求 realm_id == 会话上下文 (per §3.3 FR-PLT-012) | 严格一致 | 拒绝 + 写 audit (pay.realm.isolation.mismatch_detected, P1 安全告警) |
| 合服资产一致性 | 合服后 total_characters/total_inventory_items/total_currency 与演练 delta 一致 (per §4.2 步骤 4) | delta = 0 (或容差范围内) | 触发 asset_consistency_check_failed (SRE 立即介入) + 回滚到维护模式 |
| 选服目标服可分配 | chosen_realm_id ∈ RealmDirectoryService 可分配列表 (per §3.2) | 状态 normal 且 ∈ 可见列表 | 拒绝 + 写 audit (pay.realm.route.realm_unavailable, 引导重新选服) |

---

# 2. 平台收据校验组件设计

## 2.1 组件划分

| 组件 | 归属限界上下文 | 职责 |
|---|---|---|
| `ReceiptVerifier` | PL/EC | 向App Store/Google Play官方接口校验收据，每个平台一个适配子模块 |
| `RefundNotificationHandler` | PL/EC | 接收平台异步退款通知，触发权益追回流程 |

### 2.1 本功能日志设计

本节覆盖**收据校验/退款通知组件的运行生命周期可观测字段**——组件启动与关停、平台适配子模块路由、模块级配置加载。事件名统一 `pay.component.*` 前缀（pay = platform IAP compliance & realm selection）。组件启动/关停 release 必出以满足 SRE 容量与可用性视图诉求（per NFR-OP-008）；平台适配子模块的精确选择（`app_store` vs `google_play`）走 `debug!` 守护——release build 不可见以避免高 QPS 下淹没生产日志通道；初始化失败/配置缺失走 `error!` 强制全采样（per BAS-004 v0.3 §6.2）。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `pay.component.startup` | `ReceiptVerifier` / `RefundNotificationHandler` 进程启动并完成依赖注入（DB pool / 平台 HTTP 客户端） | 极低（每 pod 1 次，per BAS-004 v0.3 §6.1 启动事件豁免） | release 必出（`info!` 编译期常驻，per BAS-004 v0.3 §4.2） | 含 `component` / `pid` / `startup_duration_ms`；约 200B/条；无敏感字段 |
| `pay.component.shutdown` | 收到 SIGTERM，停止接收新收据/退款通知，等待存量任务完成或超时（per FR-GW-009） | 极低 | release 必出（`info!` 强制全采样） | 含 `component` / `inflight_count` / `drain_duration_ms`；约 220B/条 |
| `pay.component.config_loaded` | 平台适配子模块配置（`app_store_shared_secret` 引用 / `google_play_public_key_path`）加载完成 | 极低（启动 + 热更新时） | release 必出（`info!` 强制全采样） | 含 `component` / `config_version` / `key_fingerprint`（SHA-256 前 8 字节，**不**写原始密钥，per BAS-004 v0.3 §5.1 `*key*` 黑名单自动丢弃原始值）；约 280B/条 |
| `pay.component.config_load_failed` | 平台公钥/共享密钥加载失败（文件缺失/格式错误/权限不足） | 极低 | release 必出（`error!` 强制全采样，per BAS-004 v0.3 §6.2） | 含 `component` / `key_kind` / `error`；约 260B/条；密钥路径**不**记录明文（per BAS-004 v0.3 §5.1 路径/凭据黑名单） |
| `pay.component.adapter_selected` | `ReceiptVerifier` 根据 `platform_type` 字段选择具体适配子模块（`app_store` / `google_play`） | 稳态 0.5/s、峰值 50/s（按平台交易量分摊） | **debug-only**（`#[cfg(debug_assertions)]` 守护，release 完全剔除） | 约 180B/条（release 剔除，零运行时开销） |
| `pay.component.health_degraded` | 组件依赖（DB pool / 平台 HTTP 客户端）健康度降级，触发熔断（per RGS-BAS-010 §3.2 Circuit Breaker 模式） | 极低（依赖故障时） | release 必出（`warn!` 强制全采样） | 含 `component` / `dependency` / `degradation_mode`；约 240B/条 |

**debug-only 守护要点**（落实 BAS-004 v0.3 §4.4 + §4.8.3.2 二维矩阵）：
- `pay.component.adapter_selected` 是高频事件（按每笔收据触发），**必须** `#[cfg(debug_assertions)]` 守护——release profile 即便允许 RUST_LOG=debug 开启，这一条也必须剔除（per BAS-001 v1.5 §4.8.3.1 采样策略列定义）
- `pay.component.config_loaded` 中的 `key_fingerprint` 是 SHA-256 截断（8 字节 hex），**不**可逆推出原始密钥，符合 BAS-004 v0.3 §5.1 凭据类黑名单规则
- `pay.component.config_load_failed` 的 `key_kind` 字段仅写枚举值（`shared_secret` / `public_key`），**不**写文件路径或错误堆栈中的路径片段

## 2.2 收据校验时序

```
客户端完成平台内购，取得收据
  → 提交收据至服务器
  → ReceiptVerifier依平台类型选择适配子模块，向平台官方接口验证
  → 验证失败（签名无效/环境不匹配）→ 拒绝，记录审计日志（FR-PLT-004，含失败原因分类：invalid_signature／already_used／sandbox_prod_mismatch）
  → 验证接口不可用（超时/5xx，区别于"验证失败"的明确拒绝）→ 不判定为欺诈，投递至待重试队列（见§2.4，RSK-PLT-001）
  → 验证成功 → 取得平台侧唯一交易标识
      → 以交易标识为幂等键查询既有PaymentOrder（复用RGS-BAS-016§3.1数据模型）
          已存在 → 直接返回既有结果，不重复处理
          不存在 → 写入PaymentOrder + 复用FR-EC-003确定请求路径发放权益
```

### 2.2 本功能日志设计

本节覆盖**收据校验主路径的可观测字段**——校验请求接入、平台官方接口调用、结果分支（成功/失败/环境不匹配）、幂等命中、权益发放。事件名统一 `pay.receipt.*` 前缀。**合规审计强制项**（FR-PLT-004：内购订单/补单/release 必出 + 强制全采样）；内购失败/争议（`invalid_signature` / `already_used` / `sandbox_prod_mismatch`）走 `error!` 强制全采样以满足 NFR-PLT-001 合规追溯诉求；平台官方接口原始响应细节（`signedTransactionInfo` / JWS payload）走 `debug!` 守护，release 完全剔除（避免敏感字段泄漏 + 控制日志体积）；幂等命中 release 必出但**不**视为错误（per RGS-BAS-010 §3.1 `pat.cons.idempotency.hit` 同类）。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `pay.receipt.verify.received` | 客户端提交收据至 `ReceiptVerifier`（请求进入处理函数体） | 稳态 5/s、峰值 500/s（开服/活动热点） | release 必出（`info!` 编译期常驻，per BAS-004 v0.3 §4.2） | 含 `request_id` / `account_id` / `platform_type`；约 240B/条；无敏感字段 |
| `pay.receipt.verify.platform_called` | `ReceiptVerifier` 向 App Store `verifyReceipt` / Google Play `productPurchases.get` 发起 HTTP 调用（per FR-PLT-004） | 同上 | release 必出（`info!` 强制全采样，**合规审计**，per FR-PLT-004） | 含 `request_id` / `platform_type` / `platform_endpoint`（域名级别，**不**带查询串）；约 260B/条 |
| `pay.receipt.verify.success` | 平台官方接口返回 `status: 0`（App Store）或有效 purchase state（Google Play），签名验证通过 | 稳态 4/s、峰值 400/s | release 必出（`info!` 强制全采样，**合规审计不可逆**） | 含 `request_id` / `provider_txn_id` / `product_id` / `platform_environment`；约 280B/条；**不**写原始收据明文 |
| `pay.receipt.verify.failed.invalid_signature` | 平台响应签名验证失败（`status: 21002`/21005 等，per FR-PLT-004 失败原因分类） | 偶发（攻击/客户端篡改时） | release 必出（`error!` 强制全采样，per BAS-004 v0.3 §6.2） | 含 `request_id` / `platform_type` / `failure_code` / `account_id`；约 320B/条；**不**写可疑收据明文 |
| `pay.receipt.verify.failed.already_used` | 同一 `provider_txn_id` 已被使用（FR-PLT-004 失败分类） | 偶发 | release 必出（`error!` 强制全采样） | 含 `request_id` / `provider_txn_id` / `existing_payment_order_id`；约 300B/条 |
| `pay.receipt.verify.failed.env_mismatch` | 收据 `platform_environment` 与服务端配置不一致（沙盒/生产混用，per FR-PLT-004） | 极低（客户端 SDK 配错） | release 必出（`error!` 强制全采样） | 含 `request_id` / `client_environment` / `server_environment` / `platform_type`；约 280B/条 |
| `pay.receipt.idempotency.hit` | 以 `provider_txn_id` 为幂等键查询既有 `PaymentOrder` 已存在（per FR-PLT-005/ARC-009） | 稳态 0.5/s、峰值 50/s（重试/重放） | release 必出（`info!` 强制全采样，**不**视为错误） | 含 `request_id` / `provider_txn_id` / `existing_payment_order_id`；约 220B/条 |
| `pay.receipt.entitlement.granted` | 权益已通过 FR-EC-003 确定请求路径发放（per ARC-009） | 稳态 3/s、峰值 300/s | release 必出（`info!` 强制全采样，**合规审计**） | 含 `request_id` / `account_id` / `character_id` / `product_id` / `entitlement_kind`；约 300B/条；无敏感字段 |
| `pay.receipt.platform_unavailable` | 平台官方接口不可用（超时/5xx，区别于"验证失败"的明确拒绝，per §2.3 RSK-PLT-001） | 极低（平台故障时） | release 必出（`warn!` 强制全采样） | 含 `request_id` / `platform_type` / `http_status` / `latency_ms`；约 240B/条 |
| `pay.receipt.debug.platform_response_dump` | 平台官方接口完整响应（App Store `signedTransactionInfo` JWS / Google Play `purchaseState` JSON） | 偶发 | **debug-only**（`#[cfg(debug_assertions)]` 守护，release 完全剔除） | 约 500B-2KB/条（payload 大小决定，release 剔除） |
| `pay.receipt.debug.receipt_envelope_dump` | 完整客户端提交收据 envelope（**不**含 JWS 解码内容，仅 envelope 结构） | 偶发 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 300B-1KB/条（release 剔除） |

**debug-only 守护要点**（落实 BAS-004 v0.3 §4.4 + 平台内购域特殊考虑）：
- `pay.receipt.debug.platform_response_dump` 中若包含 `receipt_creation_date_ms` 等非敏感元数据仍可记录，但若包含 `original_transaction_id` 等与 PII 关联字段须以**字段脱敏**形式记录（`original_transaction_id` 仅保留前 4 字节，per BAS-004 v0.3 §5.1 自定义脱敏规则）
- **支付凭证/卡号/PayPal 账号 → 禁止记录**（per BAS-004 v0.3 §5.1 `*card*` / `*paypal*` / `*credential*` 黑名单自动丢弃）；本节所有 `pay.receipt.*` 字段集已规避，仅在 `debug.*` 守护项中允许记录 envelope 结构，**不**记录原始收据明文
- `pay.receipt.verify.failed.invalid_signature` 是**反欺诈信号**，需 release 必出以供风控系统按 `account_id` 维度聚合（per NFR-PLT-001 合规追溯）

## 2.3 平台校验接口不可用的待重试队列（RSK-PLT-001落地）

`PendingReceiptVerification`（依附既有PL/EC上下文数据库，不新建独立库）：

| 字段 | 类型 | 说明 |
|---|---|---|
| `pending_id` | uuid | 唯一标识 |
| `raw_receipt` | 加密存储的原始收据 | 待重试的收据内容 |
| `platform_type` | enum(`app_store`／`google_play`） | 决定重试时使用哪个适配子模块 |
| `retry_count` | int | 已重试次数 |
| `next_retry_at` | timestamp | 下次重试时间（指数退避，复用ARC-009标准消费者重试参数量级） |
| `status` | enum(`pending`／`resolved`／`abandoned`) | `abandoned`为超过最大重试次数后的终态，转人工（生成RGS-BAS-016 SupportTicket，category=payment_issue） |

`ReceiptVerifier`定时任务扫描`status=pending AND next_retry_at<=now()`的记录重新发起验证，成功后进入§2.2正常发放路径并将本记录标记`resolved`；超过最大重试次数（详细设计确定阈值）标记`abandoned`并转人工复核，**不得**因平台接口持续不可用而无限期悬挂玩家的合法收据。

### 2.3 本功能日志设计

本节覆盖**待重试队列（PendingReceiptVerification）的可观测字段**——入队、重试、退避计算、超限转人工。事件名统一 `pay.receipt.retry.*` 前缀（区别于 §2.2 主路径的 `pay.receipt.verify.*`）。**合规审计强制项**（RSK-PLT-001：待重试队列的入队与解决均 release 必出 + 强制全采样，确保任何"玩家已付款但权益未发放"状态可追溯）；超限转人工走 `error!` 强制全采样（玩家合法收据被悬挂的事件，需 SRE 立即介入）；退避计算细节（`next_retry_at` 的指数退避公式）走 `debug!` 守护。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `pay.receipt.retry.enqueued` | 收据因平台接口不可用（区别于"验证失败"）被写入 `PendingReceiptVerification` 表，状态 `pending`（per §2.2 RSK-PLT-001 分支判定） | 稳态 0.1/s、峰值 10/s（平台故障时） | release 必出（`info!` 编译期常驻，**合规审计不可逆**） | 含 `pending_id` / `account_id` / `platform_type` / `first_enqueued_at`；约 280B/条；`raw_receipt` **不**记录（加密存储于 DB，per §2.3） |
| `pay.receipt.retry.attempted` | 定时任务扫描并发起一次重试（`status=pending AND next_retry_at<=now()`） | 稳态 0.1/s、峰值 5/s | release 必出（`info!` 强制全采样，**合规审计**） | 含 `pending_id` / `retry_count` / `attempt_latency_ms`（从入队到本次重试间隔）；约 240B/条 |
| `pay.receipt.retry.resolved` | 重试成功，状态从 `pending` → `resolved`（per §2.3 正常发放路径） | 稳态 0.05/s、峰值 5/s | release 必出（`info!` 强制全采样，**合规审计**） | 含 `pending_id` / `provider_txn_id` / `resolved_at` / `total_duration_ms`；约 260B/条 |
| `pay.receipt.retry.exhausted` | 超过最大重试次数，状态从 `pending` → `abandoned`（per §2.3 超限转人工） | 极低 | release 必出（`error!` 强制全采样，per BAS-004 v0.3 §6.2） | 含 `pending_id` / `account_id` / `total_attempts` / `first_enqueued_at`；约 280B/条；**不**记录原始收据 |
| `pay.receipt.retry.abandoned_to_human` | `abandoned` 状态触发，已生成 RGS-BAS-016 `SupportTicket`（`category=payment_issue`）转人工复核 | 极低 | release 必出（`warn!` 强制全采样） | 含 `pending_id` / `support_ticket_id` / `operator_visible=true`（标记供运营识别）；约 280B/条 |
| `pay.receipt.retry.scheduler_overrun` | 定时任务执行超过预期周期（典型 60s 周期超时 > 120s，可能因 DB 慢查询/锁等待） | 极低 | release 必出（`warn!` 强制全采样） | 含 `scheduled_at` / `actual_started_at` / `lag_ms` / `pending_count_at_scan`；约 220B/条 |
| `pay.receipt.retry.debug.backoff_calc` | 退避计算细节（`base_delay × 2^attempt + jitter` 的具体值，`next_retry_at` 的精确推导） | 偶发 | **debug-only**（`#[cfg(debug_assertions)]` 守护，release 完全剔除） | 约 240B/条（release 剔除） |
| `pay.receipt.retry.debug.pending_record_dump` | `PendingReceiptVerification` 单条完整记录（除 `raw_receipt` 字段外） | 偶发（事故复盘） | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 400B/条（release 剔除） |

**debug-only 守护要点**（落实 BAS-004 v0.3 §4.4 + RSK-PLT-001 不可逆追溯诉求）：
- `pay.receipt.retry.*` 全部走 release 必出，**不**允许降级为 `debug!`——这是合规审计的硬要求（per NFR-PLT-001：内购订单全生命周期可追溯；SRE 按 `pending_id` 维度聚合可定位"已扣款但未发放"的事件）
- `pay.receipt.retry.exhausted` 是 SRE 关注的最高优先级事件，建议告警通道直接路由（per NFR-OP-008 排查 SLA）
- `pay.receipt.retry.scheduler_overrun` 是队列健康度指标——若持续 overrun 说明 DB 端或调度策略有问题（per RGS-BAS-010 §3.5 HPA 调度背压同类）

## 2.4 `PaymentOrder`平台内购扩展字段（FR-PLT-004、FR-PLT-005）

复用RGS-BAS-016§3.1既定`PaymentOrder`结构，新增以下字段以承载平台内购特有信息（不新建独立表，遵循FR-PLT-005"共享同一套数据模型"）：

| 字段 | 类型 | 说明 |
|---|---|---|
| `payment_channel` | enum(`platform_iap`／`direct_gateway`) | 区分平台内购与RGS-REQ-019既有直连支付，决定对账/退款处理走哪条子流程 |
| `platform_type` | enum，可选（仅`payment_channel=platform_iap`时非空） | `app_store`／`google_play` |
| `platform_environment` | enum(`sandbox`／`production`) | 沙盒/生产环境标记，**必须**与收据校验时平台返回的环境一致，环境不匹配须拒绝（FR-PLT-004"环境不匹配"分支），防止沙盒测试收据被用于生产环境权益发放 |
| `refund_status` | enum(`none`／`refunded`／`clawback_pending`／`clawback_done`) | 退款处理状态（FR-PLT-003），初始为`none` |

索引：`(platform_type, provider_txn_id)`复合唯一索引（`provider_txn_id`复用RGS-BAS-016既定字段承载平台交易标识），确保跨平台交易标识不产生误关联。

### 2.4 本功能日志设计

本节覆盖**`PaymentOrder` 平台内购扩展字段的可观测字段**——`payment_channel` 区分、平台环境校验、跨平台唯一索引命中、`refund_status` 状态机迁移。事件名统一 `pay.payment_order.*` 前缀。**合规审计强制项**（FR-PLT-005：内购订单全生命周期可追溯）；环境不匹配（沙盒/生产）走 `error!` 强制全采样（per FR-PLT-004 "环境不匹配"分支）；扩展字段的精确值（如 `platform_environment` 校验前后对照）走 `debug!` 守护。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `pay.payment_order.created` | `PaymentOrder` 主记录写入（含 §2.4 四个扩展字段首次赋值，per FR-PLT-004/005） | 稳态 3/s、峰值 300/s | release 必出（`info!` 编译期常驻，**合规审计不可逆**） | 含 `payment_order_id` / `account_id` / `payment_channel` / `platform_type` / `platform_environment`；约 280B/条；无敏感字段 |
| `pay.payment_order.duplicate_index_hit` | `(platform_type, provider_txn_id)` 复合唯一索引命中（幂等保护，per §2.4 索引） | 稳态 0.5/s、峰值 50/s | release 必出（`info!` 强制全采样，**不**视为错误） | 含 `payment_order_id` / `provider_txn_id` / `platform_type`；约 220B/条 |
| `pay.payment_order.environment_validated` | `platform_environment` 字段校验通过（与平台响应一致，per FR-PLT-004） | 稳态 3/s、峰值 300/s | **debug-only**（`#[cfg(debug_assertions)]` 守护，release 完全剔除） | 约 240B/条（release 剔除） |
| `pay.payment_order.environment_mismatch` | `platform_environment` 字段与平台响应不一致（沙盒/生产混用，per FR-PLT-004 "环境不匹配"分支） | 极低（客户端 SDK 配错/被攻击） | release 必出（`error!` 强制全采样，per BAS-004 v0.3 §6.2） | 含 `payment_order_id` / `client_environment` / `server_environment` / `platform_type`；约 280B/条 |
| `pay.payment_order.refund_status_transition` | `refund_status` 状态机迁移（`none` → `refunded` / `clawback_pending` → `clawback_done`，per FR-PLT-003） | 极低（仅退款触发） | release 必出（`info!` 强制全采样，**合规审计**） | 含 `payment_order_id` / `from_state` / `to_state` / `trigger_kind`（refund_notification / manual）；约 240B/条 |
| `pay.payment_order.debug.full_envelope` | `PaymentOrder` 完整字段集（含 `refund_status` 当前值，但**不**含 `raw_receipt` 等敏感字段） | 偶发（事故复盘） | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 500B-1KB/条（release 剔除） |

**debug-only 守护要点**（落实 BAS-004 v0.3 §4.4）：
- `pay.payment_order.environment_validated` 走 debug-only 是有意为之——成功路径若 release 必出会因开服/活动瞬时高 QPS 撑爆日志通道（per BAS-001 v1.5 §4.8.3.1 频率估算原则）
- `pay.payment_order.environment_mismatch` 走 `error!` 强制全采样——这是反欺诈关键信号，风控系统需按 `account_id` 维度聚合（per NFR-PLT-001）
- `pay.payment_order.debug.full_envelope` **不**含 `raw_receipt`（原始收据明文在 BAS-004 v0.3 §5.1 黑名单中），仅记录结构化字段集

## 2.5 退款处理时序

```
平台异步推送退款/撤销通知（App Store Server Notifications / Google Play RTDN）
  → RefundNotificationHandler接收并校验通知来源真实性（平台签名验证）
  → 关联至对应PaymentOrder（依交易标识）
  → 触发权益追回流程：依TBD-PLT-001确定的追回方式（扣除等价物/标记负债/不追回）
  → 追回结果留痕（复用RGS-BAS-003§7审计设计）
```

### 2.5 本功能日志设计

本节覆盖**退款处理时序的可观测字段**——平台异步通知接入、签名验证、关联既有 `PaymentOrder`、权益追回流程触发、追回模式选择、追回结果留痕。事件名统一 `pay.refund.*` 前缀（区别于 `pay.receipt.*` 主路径）。**合规审计强制项**（FR-PLT-003：退款与权益追回 release 必出 + 强制全采样，不可逆）；签名验证失败走 `error!` 强制全采样（可能是伪造通知攻击，per NFR-PLT-002）；权益追回模式选择（TBD-PLT-001 待定：扣除等价物/标记负债/不追回）走 `debug!` 守护；webhook payload 明细走 `debug!` 守护（避免敏感字段泄漏）。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `pay.refund.notify.received` | `RefundNotificationHandler` 收到平台异步通知（App Store Server Notifications / Google Play RTDN） | 极低（退款事件频次） | release 必出（`info!` 编译期常驻，**合规审计**） | 含 `notification_id` / `platform_type` / `received_at`；约 240B/条；webhook payload 明细**不**记 |
| `pay.refund.signature.verified` | 平台通知签名验证通过（App Store JWS / Google Play Pub/Sub message signature） | 极低 | release 必出（`info!` 强制全采样） | 含 `notification_id` / `platform_type` / `key_fingerprint`（SHA-256 前 8 字节）；约 220B/条 |
| `pay.refund.signature.failed` | 平台通知签名验证失败（可能为伪造通知攻击，per NFR-PLT-002） | 极少（攻击时） | release 必出（`error!` 强制全采样，per BAS-004 v0.3 §6.2） | 含 `notification_id` / `platform_type` / `failure_reason`；约 280B/条；**不**记录可疑 payload 明文 |
| `pay.refund.related_to_order` | 依 `provider_txn_id` 关联至既有 `PaymentOrder` 成功 | 极低 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 240B/条（release 剔除） |
| `pay.refund.related_to_order.miss` | 通知携带的 `provider_txn_id` 在 `PaymentOrder` 不存在（异常：可能是测试/伪造/未结算的退款） | 极少 | release 必出（`warn!` 强制全采样） | 含 `notification_id` / `provider_txn_id` / `platform_type`；约 240B/条 |
| `pay.refund.clawback.started` | 权益追回流程触发（依 TBD-PLT-001 确定的追回方式） | 极低 | release 必出（`info!` 强制全采样，**合规审计不可逆**） | 含 `payment_order_id` / `account_id` / `clawback_mode`（deduct / mark_debt / skip）；约 280B/条 |
| `pay.refund.clawback.completed` | 权益追回流程完成（成功扣除 / 标记负债 / 不追回留痕） | 极低 | release 必出（`info!` 强制全采样，**合规审计**） | 含 `payment_order_id` / `clawback_mode` / `clawed_back_amount` / `completion_duration_ms`；约 280B/条 |
| `pay.refund.clawback.failed` | 权益追回失败（追回流程内部错误，DB 写入失败等） | 极少 | release 必出（`error!` 强制全采样） | 含 `payment_order_id` / `clawback_mode` / `error` / `trace_id`；约 320B/条 |
| `pay.refund.clawback.mode_selected` | 追回模式决策（TBD-PLT-001 详设确定：扣除等价物/标记负债/不追回），含决策依据 | 极低 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 300B/条（release 剔除） |
| `pay.refund.debug.webhook_payload_dump` | webhook 完整 payload（App Store `signedPayload` JWS / Google Play Pub/Sub message body），**仅** envelope 结构，**不**含卡号/账户等黑名单字段 | 偶发（事故复盘） | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 500B-2KB/条（release 剔除；**不**含 §5.1 黑名单字段） |
| `pay.refund.status_transition` | `refund_status` 状态机迁移（per FR-PLT-003；同 §2.4 `pay.payment_order.refund_status_transition` 同步） | 极低 | release 必出（`info!` 强制全采样） | 含 `payment_order_id` / `from_state` / `to_state`；约 220B/条 |

**debug-only 守护要点**（落实 BAS-004 v0.3 §4.4 + 平台内购域特殊考虑）：
- `pay.refund.debug.webhook_payload_dump` **必须**实现层做字段过滤——`unified_receipt` / `original_transaction_id` / `notification_type` 等可记，但**禁止**记录任何 `*card*` / `*paypal*` / `*account_number*` 等黑名单字段（per BAS-004 v0.3 §5.1）
- `pay.refund.clawback.mode_selected` 走 debug-only 是有意为之——TBD-PLT-001 在详设阶段才会确定，detail 设计前 release 不应记录具体模式（避免被外部参考实现误解为已定案）
- `pay.refund.clawback.completed` 中的 `clawed_back_amount` 是**业务结果**（不涉及 PII），release 必出以供财务对账

---

# 3. 选服路由设计

## 3.1 组件划分

| 组件 | 职责 |
|---|---|
| `RealmDirectoryService` | 维护逻辑服列表与状态（正常/爆满/维护中），依附AD限界上下文，状态由GM后台配置驱动 |
| `RealmRouter` | 鉴权成功后、进入大厅前的路由决策，依附PL限界上下文 |

### 3.1 本功能日志设计

本节覆盖**选服组件的运行生命周期可观测字段**——`RealmDirectoryService` 服务器列表变更（正常/爆满/维护）、`RealmRouter` 启动与配置加载。事件名统一 `pay.realm.directory.*` 与 `pay.realm.router.*` 前缀。服务器列表变更 release 必出（运营 + SRE 都需要选服状态可观测）；客户端拉取服务器列表的频次细节走 `debug!` 守护；缓存查找细节走 `debug!` 守护。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `pay.realm.directory.realm_status_changed` | `RealmDirectoryService` 维护的逻辑服状态发生变更（`normal` / `full` / `maintenance`，由 GM 后台或健康度检测驱动） | 极低（运营触发或周期健康度检查） | release 必出（`info!` 编译期常驻） | 含 `realm_id` / `from_status` / `to_status` / `trigger_kind`（gm_manual / health_check）；约 240B/条 |
| `pay.realm.directory.list_served` | `RealmDirectoryService` 返回服务器列表（客户端拉取） | 稳态 1/s、峰值 50/s（按 DAU 分摊） | release 必出（`info!` 强制全采样） | 含 `request_id` / `account_id` / `realm_count` / `visible_full_realm_count`；约 240B/条；**不**含玩家所在服（避免 PII 关联） |
| `pay.realm.router.startup` | `RealmRouter` 进程启动（与 §2.1 启动事件同模式） | 极低 | release 必出（`info!` 强制全采样） | 含 `component` / `startup_duration_ms`；约 200B/条 |
| `pay.realm.router.config_loaded` | 选服策略表（白名单/黑名单/优先级）加载完成 | 极低 | release 必出（`info!` 强制全采样） | 含 `config_version` / `rule_count` / `policy_fingerprint`（SHA-256 前 8 字节，per BAS-004 v0.3 §5.1）；约 260B/条 |
| `pay.realm.router.config_load_failed` | 选服策略表加载失败（DB 不可达/格式错误） | 极低 | release 必出（`error!` 强制全采样） | 含 `error` / `trace_id`；约 240B/条 |
| `pay.realm.directory.debug.cache_lookup` | `RealmDirectoryService` 缓存查找细节（命中/未命中/回填） | 稳态 1/s、峰值 50/s | **debug-only**（`#[cfg(debug_assertions)]` 守护，release 完全剔除） | 约 200B/条（release 剔除） |
| `pay.realm.router.debug.rule_resolution` | 选服规则解析细节（白名单命中/优先级匹配/最终选用服的决策路径） | 偶发 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 280B/条（release 剔除） |

**debug-only 守护要点**（落实 BAS-004 v0.3 §4.4）：
- `pay.realm.directory.list_served` 的稳态 1/s 是按 DAU 摊销的估算，开服瞬时可达 50/s；release 必出确保 SRE 可在活动期间按 `account_id` 维度分析玩家拉取模式
- `pay.realm.directory.debug.cache_lookup` 高频事件必须 `#[cfg(debug_assertions)]` 守护——release 误开 RUST_LOG=debug 会撑爆日志通道（per BAS-001 v1.5 §4.8.3.1）

## 3.2 选服时序

```
鉴权成功（复用既有FR-GW-002）
  → RealmRouter查询账号是否已有"主服"记录
      有 → 直接路由至主服，跳过选服界面
      无（首次登录）→ 客户端展示RealmDirectoryService提供的服务器列表（含状态）
          玩家选择 → 记录为主服 → 路由至该服
  → 路由完成后进入既有大厅流程（RGS-REQ-016/BAS-013）
```

### 3.2 本功能日志设计

本节覆盖**选服时序的可观测字段**——`RealmRouter` 路由请求接入、主服命中、首次登录、玩家选择主服、路由完成。事件名统一 `pay.realm.route.*` 前缀。**合规与运营强制项**（玩家主服选择不可逆：玩家选择 + 主服记录 release 必出 + 强制全采样，满足 NFR-PLT-001 合规追溯 + 运营分析诉求）；首次登录走 `info!` 强制全采样（新账号识别）；会话上下文构建细节走 `debug!` 守护。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `pay.realm.route.received` | `RealmRouter` 收到路由请求（鉴权成功后、进入大厅前） | 稳态 5/s、峰值 500/s（开服/活动热点） | release 必出（`info!` 编译期常驻） | 含 `request_id` / `account_id` / `realm_hint`（可选：客户端声明的目标服，per ARC-005 服务器权威原则仅作 hint）；约 240B/条 |
| `pay.realm.route.primary_hit` | 查询到账号已有"主服"记录（`account_id → primary_realm_id`），跳过选服界面直接路由 | 稳态 4/s、峰值 400/s | release 必出（`info!` 强制全采样，**合规追溯**） | 含 `request_id` / `account_id` / `primary_realm_id` / `last_login_at`；约 260B/条 |
| `pay.realm.route.first_login` | 账号首次登录，无主服记录，展示服务器列表（`RealmDirectoryService.list_served` 已记录，此处仅标记"首次登录"事实） | 稳态 0.5/s、峰值 50/s | release 必出（`info!` 强制全采样，**新账号识别**） | 含 `request_id` / `account_id` / `first_login_at`；约 220B/条 |
| `pay.realm.route.player_choice` | 玩家在选服界面选择主服（客户端提交 `chosen_realm_id`，**不**信任客户端权威，per ARC-005） | 稳态 0.5/s、峰值 50/s | release 必出（`info!` 强制全采样，**不可逆主服选择**） | 含 `request_id` / `account_id` / `chosen_realm_id` / `server_validated`（布尔：服务端校验通过）；约 240B/条 |
| `pay.realm.route.assigned` | 路由完成，会话已建立（携带 `realm_id` 进入大厅流程） | 稳态 5/s、峰值 500/s | release 必出（`info!` 强制全采样） | 含 `request_id` / `account_id` / `assigned_realm_id` / `routing_duration_ms`；约 260B/条 |
| `pay.realm.route.realm_unavailable` | 玩家选择的主服已下线/维护（GM 后台变更或健康度检测触发下线，状态非 `normal`） | 偶发 | release 必出（`warn!` 强制全采样） | 含 `request_id` / `account_id` / `chosen_realm_id` / `realm_status`；约 240B/条 |
| `pay.realm.route.rbac_denied` | 玩家身份验证失败（账号封禁/合规状态阻止，per RGS-BAS-018） | 偶发（攻击/封禁账号） | release 必出（`warn!` 强制全采样） | 含 `request_id` / `account_id` / `denial_reason` / `compliance_status`；约 240B/条；**不**记录封禁原因明文（per BAS-004 v0.3 §5.1） |
| `pay.realm.route.debug.session_context_build` | 会话上下文构建细节（含 `realm_id` 注入、权限标记组装） | 偶发 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 300B/条（release 剔除） |
| `pay.realm.route.debug.realm_hint_compare` | 客户端 hint `realm_id` 与服务端查询的 `primary_realm_id` 一致性对照 | 偶发 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 240B/条（release 剔除） |

**debug-only 守护要点**（落实 BAS-004 v0.3 §4.4）：
- `pay.realm.route.player_choice` 的 `server_validated` 字段是布尔值，**不**依赖客户端声明（per ARC-005 服务器权威原则）；若客户端提交的 `chosen_realm_id` 与服务端可分配列表不一致，应记录 `server_validated=false` 并改走"客户端 hint 与服务器决策不一致"分支——本字段是反作弊关键
- `pay.realm.route.rbac_denied` 中的 `compliance_status` 仅写枚举值（`active` / `minor_restricted` / `banned`），**不**写具体封禁原因（per BAS-004 v0.3 §5.1）
- `pay.realm.route.debug.realm_hint_compare` 是反作弊诊断信号，仅 debug 守护——release 下若玩家主服被恶意修改客户端 hint 试探，此项也**不**应暴露给普通 SRE（需 OTel RBAC 控制访问，per RGS-BAS-003 §6.3 告警事件分级）

## 3.3 账号数据的逻辑服隔离（FR-PLT-012落地）

多服架构下，玩家的角色/进度类数据**必须**携带`realm_id`维度，与`account_id`共同构成数据归属键，而**不是**为每个逻辑服新建独立的数据库/Schema（避免与既有单库表结构产生分裂式重复建设）：

- 角色/进度相关表（如角色表、背包表、任务进度表等既有业务数据模型）**必须**新增`realm_id`字段并纳入主键/分片键的组成部分，`(account_id, realm_id)`唯一标识"某账号在某逻辑服下的数据集合"——同一`account_id`可在不同`realm_id`下拥有相互独立的角色/进度记录，二者**不得**混查或互相可见
- 账号级数据（如`AccountIdentityLink`第三方身份绑定、`ComplianceProfile`合规状态，见RGS-BAS-018）**不**携带`realm_id`，因身份/合规属性归属于账号本身而非某个逻辑服，与角色/进度数据的归属维度不同，**不得**混同
- `RealmRouter`完成路由决策后，后续业务请求（复用既有FR-GW-002会话建立）**必须**将`realm_id`纳入会话上下文，下游业务服务对角色/进度类数据的读写**必须**校验请求携带的`realm_id`与会话上下文一致，**不得**由客户端自行声明`realm_id`（同ARC-005服务器权威原则，防止跨服越权访问）
- 具体到哪些既有表结构需要补充`realm_id`字段，属于对既有业务数据模型的追加变更，**须**在多服架构确定启用（TBD-PLT-002评审通过）后，由各自领域的BAS文档在详细设计阶段补齐字段清单，本文档仅确立"账号数据按`realm_id`隔离"这一设计原则与归属键约定，不代为逐一列举各业务表的字段变更

### 3.3 本功能日志设计

本节覆盖**`realm_id` 数据归属键的可观测字段**——会话上下文注入、跨服越权检测、账号级 vs 角色级数据归属维度区分。事件名统一 `pay.realm.isolation.*` 前缀。**合规与安全强制项**（FR-PLT-012：跨服越权访问阻止 release 必出 + 强制全采样，per NFR-PLT-001 合规追溯）；`realm_id` 与会话上下文一致性校验失败走 `error!` 强制全采样（这是**安全告警**而非普通业务事件，per RGS-BAS-003 §6.3 告警事件分级）；归属键构建细节走 `debug!` 守护。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `pay.realm.isolation.realm_id_injected` | `RealmRouter` 完成路由后，`realm_id` 注入会话上下文（per §3.3 ARC-005 服务器权威原则） | 稳态 5/s、峰值 500/s | release 必出（`info!` 编译期常驻） | 含 `session_id` / `account_id` / `injected_realm_id`；约 220B/条 |
| `pay.realm.isolation.mismatch_detected` | 下游业务服务收到请求时，校验请求携带 `realm_id` 与会话上下文不一致（per §3.3 防止跨服越权） | 偶发（客户端篡改/会话过期） | release 必出（`error!` 强制全采样，**安全告警**，per RGS-BAS-003 §6.3） | 含 `session_id` / `request_realm_id` / `session_realm_id` / `service`；约 280B/条；**不**含完整请求体 |
| `pay.realm.isolation.cross_realm_attempt_blocked` | 跨服越权访问被业务服务拒绝（per §3.3 ARC-005 服务器权威原则） | 偶发（攻击时） | release 必出（`error!` 强制全采样，**安全告警**） | 含 `account_id` / `attempted_realm_id` / `target_realm_id` / `resource_kind`（character / inventory / quest_progress）；约 280B/条 |
| `pay.realm.isolation.account_data_scope_check` | 业务请求访问账号级数据（`AccountIdentityLink` / `ComplianceProfile`，per §3.3）时，确认**不**携带 `realm_id` 归属维度 | 稳态 1/s、峰值 50/s | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 240B/条（release 剔除） |
| `pay.realm.isolation.character_data_scope_check` | 业务请求访问角色级数据（角色/背包/任务进度）时，确认携带正确 `realm_id` 并完成归属键构造（`(account_id, realm_id)`，per §3.3） | 稳态 10/s、峰值 1000/s | **debug-only**（`#[cfg(debug_assertions)]` 守护，release 完全剔除） | 约 260B/条（release 剔除） |
| `pay.realm.isolation.debug.key_construction` | `(account_id, realm_id)` 主键构造细节（含 SQL bind 顺序、sharding key 选择） | 偶发 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 280B/条（release 剔除） |

**debug-only 守护要点**（落实 BAS-004 v0.3 §4.4 + §3.3 ARC-005 服务器权威原则）：
- `pay.realm.isolation.mismatch_detected` 是**安全告警**，必须 release 必出 + 强制全采样（per BAS-004 v0.3 §6.2）；不应被 `#[cfg(debug_assertions)]` 守护——一旦被守护，攻击事件将无法在生产环境被检测
- `pay.realm.isolation.account_data_scope_check` / `character_data_scope_check` 高频成功路径走 debug-only，**有意**不在 release 暴露成功路径——这些是归属键正确性的内部断言，非业务事件
- `pay.realm.isolation.cross_realm_attempt_blocked` 应同时触发 OTel 告警（per RGS-BAS-003 §6.3），SRE 通道与安全审计通道均应可见

---

# 4. 合服/分服执行流程

## 4.1 冲突解决规则配置表（FR-PLT-021落地）

`MergeConflictRuleSet`（配置表，与具体某次合服作业关联，非全局默认值，因不同批次合服的运营诉求可能不同）：

| 字段 | 类型 | 说明 |
|---|---|---|
| `merge_job_id` | uuid | 关联具体一次合服作业 |
| `character_name_conflict_rule` | enum(`auto_rename_with_suffix`／`require_manual_rename_on_login`) | 同名角色处理策略 |
| `unique_item_conflict_rule` | enum(`stack_additively`／`keep_both_as_separate`／`keep_earliest_and_compensate`) | 重复唯一性道具处理策略（如限定称号类不可叠加道具的处理） |
| `currency_conflict_rule` | enum(`sum`) | 货币类冲突固定为累加（货币无"唯一性冲突"概念，仅需求和） |
| `approved_by` | 运营/架构师签署 | FR-PLT-021"须与运营团队评审确定"的评审记录关联 |

`MergeConflictRuleSet`须在§4.2步骤1完成评审并锁定后，方可进入步骤2演练环境执行；演练/正式执行均读取同一份已锁定配置，**不得**在正式执行时临时调整规则（避免"执行人员临时决定"，FR-PLT-021明确禁止的情形）。

### 4.1 本功能日志设计

本节覆盖**`MergeConflictRuleSet` 配置管理的可观测字段**——评审锁定、修改、签署。事件名统一 `pay.merge.rule.*` 前缀。**合规与运营强制项**（FR-PLT-021：合服规则在执行前必须完成评审 + 锁定 + 签署，release 必出 + 强制全采样以满足运营追溯）；已锁定的规则被修改走 `warn!` 强制全采样（这是异常流程——已锁定规则不应再变，per §4.1 "**不得**在正式执行时临时调整规则"）；草稿与已锁定的差异走 `debug!` 守护（仅供事故复盘）。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `pay.merge.rule.created` | `MergeConflictRuleSet` 草稿创建（与具体某次合服作业关联，per §4.1） | 极低（合服作业粒度） | release 必出（`info!` 编译期常驻，**合规追溯**） | 含 `merge_job_id` / `rule_set_id` / `created_by` / `created_at`；约 240B/条 |
| `pay.merge.rule.locked` | `MergeConflictRuleSet` 评审通过，状态从 `draft` → `locked`（per §4.1 "**须**在步骤1完成评审并锁定后方可进入步骤2"） | 极低 | release 必出（`info!` 强制全采样，**合规追溯**） | 含 `merge_job_id` / `rule_set_id` / `locked_by` / `locked_at` / `reviewer_signatures`（多签署人列表）；约 300B/条 |
| `pay.merge.rule.modified_after_lock` | 已锁定的 `MergeConflictRuleSet` 再次被修改（异常流程，应仅在演练前发生） | 极少 | release 必出（`warn!` 强制全采样，**异常流程可观测**） | 含 `merge_job_id` / `rule_set_id` / `modified_by` / `modification_diff_fingerprint`（SHA-256 前 8 字节，per BAS-004 v0.3 §5.1）；约 320B/条 |
| `pay.merge.rule.approved` | 运营+架构师签署完成（`approved_by` 字段填写，per §4.1 FR-PLT-021） | 极低 | release 必出（`info!` 强制全采样，**合规追溯**） | 含 `merge_job_id` / `rule_set_id` / `approver_id` / `approver_role` / `approved_at`；约 260B/条 |
| `pay.merge.rule.lock_attempt_without_signature` | 尝试锁定但签署人数不足或角色不符（per §4.1 FR-PLT-021） | 极少 | release 必出（`warn!` 强制全采样） | 含 `merge_job_id` / `attempted_by` / `missing_signatures`；约 240B/条 |
| `pay.merge.rule.debug.draft_diff` | 草稿与已锁定版本的完整字段差异对照（用于事故复盘） | 偶发 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 500B-1KB/条（release 剔除） |

**debug-only 守护要点**（落实 BAS-004 v0.3 §4.4 + FR-PLT-021 运营追溯诉求）：
- `pay.merge.rule.*` 全部走 release 必出，**不**允许降级为 `debug!`——这是合服作业合规追溯的硬要求（per FR-PLT-021：合服作业必须留痕；NFR-PLT-001：合服结果可追溯）
- `pay.merge.rule.modified_after_lock` 走 `warn!` 强制全采样——若此事件在生产环境出现，意味着有人在"执行人员临时决定"边缘试探，是 FR-PLT-021 明确禁止的情形（per §4.1）
- `pay.merge.rule.debug.draft_diff` 仅在事故复盘时使用，release 完全剔除——避免给执行人员提供"先演练锁定再修改"的违规路径的可见性

## 4.2 复用ARC-018挂载/退场检查清单的合服适配

| 步骤 | 内容 | 对应ARC-018既定步骤 |
|---|---|---|
| 1. 冲突规则评审 | 运营+架构师评审同名角色/重复道具的处理规则，配置化落地（FR-PLT-021） | 挂载前评审 |
| 2. 演练环境执行 | 在演练环境以生产数据快照执行完整合并流程，核对资产总量前后一致 | 挂载前验证 |
| 3. 演练结果评审 | 演练无异常方可排期正式执行；有异常须回到步骤1修正规则 | 挂载判定 |
| 4. 维护窗口正式执行 | 被合并服进入维护模式（复用既有维护模式传播机制）→ 执行数据合并 → 校验完成 | 正式挂载 |
| 5. 被合并服退场 | 数据合并确认无误后，被合并服按ARC-018既定退场流程下线 | 退场 |

### 4.2 本功能日志设计

本节覆盖**合服/分服执行流程的可观测字段**——5 个步骤完成、演练结果、跳过演练直接正式执行的违规事件、资产一致性校验、被合并服退场。事件名统一 `pay.merge.job.*` 前缀。**合规与运营强制项**（FR-PLT-021：合服全流程 release 必出 + 强制全采样，合服是不可逆操作）；资产一致性校验失败走 `error!` 强制全采样（合服数据完整性问题，需 SRE 立即介入）；跳过演练直接正式执行走 `error!` 强制全采样（FR-PLT-021 明确禁止的违规情形，per §5.2 代码评审检查清单）；演练后实体分布详情走 `debug!` 守护。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `pay.merge.job.step_completed` | 合服作业 5 个步骤中的任一步骤完成（per §4.2 表格） | 极低（合服作业粒度，1-5/sessions/合服） | release 必出（`info!` 编译期常驻，**合规追溯**） | 含 `merge_job_id` / `step`（1-5） / `completed_by` / `duration_ms`；约 280B/条 |
| `pay.merge.job.drill_completed` | 演练环境（步骤 2）执行完成，资产总量前后一致 | 极低 | release 必出（`info!` 强制全采样，**合规追溯**） | 含 `merge_job_id` / `drill_duration_ms` / `consistency_check_result`（`passed`）；约 280B/条 |
| `pay.merge.job.drill_failed` | 演练执行发现资产不一致或流程异常，需回到步骤 1 修正规则 | 极少 | release 必出（`error!` 强制全采样，per BAS-004 v0.3 §6.2） | 含 `merge_job_id` / `failure_step` / `inconsistency_kind` / `error`；约 320B/条 |
| `pay.merge.job.maintenance_entered` | 被合并服进入维护模式（步骤 4 前置条件，per §4.2 步骤 4） | 极低 | release 必出（`info!` 强制全采样） | 含 `merge_job_id` / `target_realm_id` / `maintenance_propagation_status`；约 240B/条 |
| `pay.merge.job.data_merged` | 数据合并完成（步骤 4 核心操作） | 极低 | release 必出（`info!` 强制全采样，**不可逆**） | 含 `merge_job_id` / `source_realm_id` / `target_realm_id` / `merged_entity_count` / `merge_duration_ms`；约 320B/条 |
| `pay.merge.job.asset_consistency_check_passed` | 资产总量校验通过（步骤 4 后置校验） | 极低 | release 必出（`info!` 强制全采样，**合规追溯**） | 含 `merge_job_id` / `total_characters` / `total_inventory_items` / `total_currency` / `delta_vs_drill`（与演练环境对比）；约 360B/条 |
| `pay.merge.job.asset_consistency_check_failed` | 资产总量校验失败（前后不一致，可能为数据丢失/重复） | 极少 | release 必出（`error!` 强制全采样，**SRE 立即介入**） | 含 `merge_job_id` / `inconsistency_kind` / `delta_details`（按 `table` 分组的不一致项）；约 360B/条 |
| `pay.merge.job.target_decommissioned` | 被合并服按 ARC-018 退场流程下线（步骤 5） | 极低 | release 必出（`info!` 强制全采样） | 含 `merge_job_id` / `target_realm_id` / `decommission_duration_ms` / `drained_session_count`；约 280B/条 |
| `pay.merge.job.skipped_drill_attempt` | 跳过步骤 2 演练直接进入步骤 4 正式执行的尝试（FR-PLT-021 明确禁止，per §5.2 检查清单） | 极少（违规事件） | release 必出（`error!` 强制全采样，**反违规告警**） | 含 `merge_job_id` / `attempted_by` / `attempted_at` / `bypassed_step`（=2）；约 280B/条；该事件应同时触发 OTel 告警（per RGS-BAS-003 §6.3） |
| `pay.merge.job.modified_rule_after_drill` | 演练完成后、正式执行前再次修改已锁定的 `MergeConflictRuleSet`（FR-PLT-021 明确禁止，per §5.2 检查清单） | 极少 | release 必出（`error!` 强制全采样，**反违规告警**） | 含 `merge_job_id` / `modified_by` / `modification_diff_fingerprint`；约 320B/条 |
| `pay.merge.job.debug.entity_distribution` | 演练/正式执行前后各服的实体分布（按 `table` 分组） | 偶发（事故复盘） | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 1-5KB/条（`table` 数决定，release 剔除） |
| `pay.merge.job.debug.conflict_resolution_log` | 冲突解决规则的逐项应用记录（同名角色 / 重复道具 / 货币累加的实际结果） | 偶发 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 1-10KB/条（冲突项数决定，release 剔除） |

**debug-only 守护要点**（落实 BAS-004 v0.3 §4.4 + FR-PLT-021 合服合规诉求）：
- `pay.merge.job.skipped_drill_attempt` / `pay.merge.job.modified_rule_after_drill` 是**反违规告警**，必须 release 必出 + 强制全采样；这些事件应路由至安全审计通道（per RGS-BAS-003 §6.3 告警事件分级）
- `pay.merge.job.asset_consistency_check_failed` 是 SRE 关注最高优先级事件之一，建议直接 PagerDuty 告警（per NFR-OP-008 排查 SLA）
- `pay.merge.job.debug.entity_distribution` 在合服后可能数 MB（按所有业务表分组）——release 完全剔除避免撑爆日志通道；SRE 若需此信息应直接查询 PostgreSQL 副本（per RGS-BAS-001 §3.5 缓存不得作为仲裁者原则）

---

# 5. 标准化检查清单

## 5.1 上线前检查清单

- [ ] 伪造收据拒绝测试通过
- [ ] 收据幂等测试通过（重复提交不重复发放）
- [ ] 退款通知处理测试通过，权益追回逻辑正确
- [ ] 选服路由验证：首次登录展示服务器列表，后续登录默认路由主服
- [ ] 合服演练流程至少完整执行一次并通过资产一致性校验（若适用多服架构）
- [ ] 平台验证接口不可用的待重试队列（§2.4）已验证：接口恢复后待重试收据自动完成发放，超限转人工
- [ ] `PaymentOrder.platform_environment`沙盒/生产不一致校验已验证拒绝跨环境收据
- [ ] `MergeConflictRuleSet`已在合服作业前完成评审锁定，演练与正式执行读取同一份配置
- [ ] 注：`PendingReceiptVerification`定时重试任务为新增常态运维面，OLU运维负荷未核算，见ISS-065

### 5.1 本功能日志设计

本节覆盖**上线前检查清单（§5.1）执行过程的可观测字段**——检查项验证、缺失项、阻塞原因。本节**不**覆盖业务功能本身的日志（那些已在 §2.1〜2.5 / §3.1〜3.3 / §4.1〜4.2 各小节覆盖），而是覆盖"清单执行工具"自身的运行痕迹。事件名统一 `pay.checklist.pre_launch.*` 前缀。检查项验证完成 release 必出（运营/合规留痕，per FR-PLT-005）；某项检查未通过走 `warn!` 强制全采样（阻塞上线）；完整检查状态快照走 `debug!` 守护。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `pay.checklist.pre_launch.started` | 上线前检查清单工具启动 | 极低（每次上线前 1 次） | release 必出（`info!` 编译期常驻，**合规留痕**） | 含 `checklist_run_id` / `release_version` / `started_by` / `started_at`；约 240B/条 |
| `pay.checklist.pre_launch.item_passed` | 检查清单中某项验证通过（如伪造收据拒绝、收据幂等、退款通知处理等 8 项） | 极低 | release 必出（`info!` 强制全采样，**合规留痕**） | 含 `checklist_run_id` / `item_id` / `item_label`（如 `forged_receipt_rejected` / `refund_idempotency` 等）；约 220B/条 |
| `pay.checklist.pre_launch.item_failed` | 某项验证未通过（阻塞上线） | 极少 | release 必出（`warn!` 强制全采样） | 含 `checklist_run_id` / `item_id` / `failure_reason` / `evidence_path`（指向上线前测试报告）；约 280B/条 |
| `pay.checklist.pre_launch.completed` | 检查清单全部 8 项执行完成（含通过/未通过的最终统计） | 极低 | release 必出（`info!` 强制全采样） | 含 `checklist_run_id` / `total_items` / `passed_items` / `failed_items` / `ready_to_release`（布尔）；约 260B/条 |
| `pay.checklist.pre_launch.release_blocked` | 存在未通过项，整体判定为阻塞上线 | 极少 | release 必出（`error!` 强制全采样，**SRE/运营立即介入**） | 含 `checklist_run_id` / `blocking_items`（数组） / `release_version`；约 320B/条 |
| `pay.checklist.pre_launch.debug.full_state_dump` | 完整检查状态快照（含每项的详细证据摘要） | 偶发 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 1-3KB/条（8 项决定，release 剔除） |

**debug-only 守护要点**（落实 BAS-004 v0.3 §4.4）：
- `pay.checklist.pre_launch.*` 全部 release 必出，**不**降级——这是上线前合规留痕的硬要求（per FR-PLT-005 + §5.1 注：上线前合规审计）
- `pay.checklist.pre_launch.release_blocked` 应同时触发 PagerDuty 告警（per NFR-OP-008 排查 SLA），确保运营/合规及时介入

## 5.2 代码评审检查清单

- [ ] 收据校验路径未出现仅信任客户端声明、跳过平台官方验证的分支
- [ ] 合服执行代码未跳过步骤2演练直接进入步骤4正式执行

### 5.2 本功能日志设计

本节覆盖**代码评审检查清单（§5.2）执行过程的可观测字段**——检查项验证、违规项识别、违规证据。本节**不**覆盖业务功能本身的日志（那些已在 §2-§4 各小节覆盖），而是覆盖"代码评审工具"自身的运行痕迹。事件名统一 `pay.checklist.code_review.*` 前缀。检查项验证 release 必出（合规追溯）；**违反检查项**（如跳过平台验证、跳过演练直接正式执行）走 `error!` 强制全采样（per FR-PLT-021 + ARC-005 服务器权威原则违反，是反违规信号）；完整违规证据走 `debug!` 守护。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `pay.checklist.code_review.started` | 代码评审检查清单工具启动（与 PR/commit 关联） | 稳态 5/s、峰值 50/s（按 PR 频次） | release 必出（`info!` 编译期常驻） | 含 `review_run_id` / `commit_sha` / `reviewer_id` / `started_at`；约 240B/条 |
| `pay.checklist.code_review.item_passed` | 某项检查通过（如"未出现仅信任客户端声明"） | 稳态 5/s、峰值 50/s | release 必出（`info!` 强制全采样） | 含 `review_run_id` / `item_id`（如 `no_client_trust_bypass`） / `file_count_checked`；约 220B/条 |
| `pay.checklist.code_review.violation.detected` | 检测到违反检查项的代码路径（反违规信号） | 极少 | release 必出（`error!` 强制全采样，**反违规告警**） | 含 `review_run_id` / `violation_kind`（`client_trust_bypass` / `skip_drill_branch`） / `file_path` / `line_number` / `commit_sha`；约 320B/条；该事件应同时触发 OTel 告警（per RGS-BAS-003 §6.3） |
| `pay.checklist.code_review.completed` | 检查清单全部 2 项执行完成 | 稳态 5/s、峰值 50/s | release 必出（`info!` 强制全采样） | 含 `review_run_id` / `total_items` / `passed_items` / `violation_count` / `merge_approved`（布尔）；约 260B/条 |
| `pay.checklist.code_review.merge_blocked` | 存在违规项，整体判定为阻塞合并 | 极少 | release 必出（`error!` 强制全采样） | 含 `review_run_id` / `blocking_violations`（数组） / `commit_sha`；约 320B/条 |
| `pay.checklist.code_review.debug.violation_evidence` | 违规项的完整证据（代码片段、上下文行、控制流） | 偶发（事故复盘） | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 500B-2KB/条（代码片段长度决定，release 剔除） |

**debug-only 守护要点**（落实 BAS-004 v0.3 §4.4 + FR-PLT-021 反违规诉求）：
- `pay.checklist.code_review.violation.detected` 必须 release 必出 + 强制全采样——这是**反违规信号**而非普通业务事件（per RGS-BAS-003 §6.3 告警事件分级），release 下被 `#[cfg]` 守护会失去防御能力
- `pay.checklist.code_review.merge_blocked` 应路由至 SRE + 合规双通道（per NFR-PLT-001 合规追溯 + NFR-OP-008 排查 SLA）
- `pay.checklist.code_review.debug.violation_evidence` 仅供事故复盘使用，release 完全剔除——避免给违反者提供"如何绕开"的可见性（per FR-PLT-021 + §5.2 静态检查定位）

---

# 6. 追溯性

| 需求ID | 本设计书章节 |
|---|---|
| ARC-038、FR-PLT-001〜005 | §2、§2.4（待重试队列）、§2.5（PaymentOrder扩展字段） |
| FR-PLT-010〜013 | §3、§3.3（逻辑服数据隔离） |
| FR-PLT-020〜023 | §4、§4.1（冲突解决规则配置） |
| NFR-PLT-001〜004 | §2、§4 |
| AC-PLT-001〜004 | §5.1 |
| TBD-PLT-001〜002、RSK-PLT-001〜002 | §5.1、§2.4（RSK-PLT-001） |
| ARC-005、ARC-009 | §3.3（realm_id 归属键 + 服务器权威）、§2.4（幂等键） |
| FR-LOG-010/011/012/013/040 | §2.1〜2.5、§3.1〜3.3、§4.1〜4.2、§5.1〜5.2 全部 12 个"本功能日志设计"小节 |
| NFR-OP-008（排查 SLA） | §2.3（`pay.receipt.retry.exhausted`）、§4.2（`pay.merge.job.asset_consistency_check_failed`） |
| AC-PLT-006（`pay.*` debug-only 宏 release 完全剔除） | §2.1〜2.5、§3.1〜3.3、§4.1〜4.2、§5.1〜5.2 全部 12 个"本功能日志设计"小节 + RGS-BAS-004 v0.3 §4.4 |
| AC-PLT-007（每功能段须含本功能 log 设计章节） | §2.1〜2.5、§3.1〜3.3、§4.1〜4.2、§5.1〜5.2 全部 12 个"本功能日志设计"小节 |

---

> 本文档与RGS-REQ-023（平台内购合规与服务器选服 需求定义书）配套使用。
