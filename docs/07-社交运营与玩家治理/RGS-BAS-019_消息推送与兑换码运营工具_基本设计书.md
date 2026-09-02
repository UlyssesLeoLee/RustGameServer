# 基本设计书（基本設計書 / Basic Design Document）

**消息推送与兑换码运营工具 Push Notification & Redemption Code**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-BAS-019 |
| 版本 | 0.3 |
| 父文档 | RGS-REQ-022 需求定义书（ARC-037） |
| 制定日 | 2026-08-16 |
| 最终更新日 | 2026-09-01 |
| 制定者 | 架构师 |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响需求ID |
|---|---|---|---|---|---|
| 0.1 | 2026-08-16 | 架构师 | — | 初版制定。将RGS-REQ-022§8 ARC-037展开为推送组件设计与同意校验时序、兑换码数据模型与核销时序 | 全部 |
| 0.2 | 2026-08-16 | 架构师 | — | 补强字段级细节：①补充推送内容脱敏校验组件（FR-OPT-006）②补充`used_count`并发递增的条件更新机制，防止高并发下超发（FR-OPT-012、NFR-OPT-003）③补充`RedemptionCodeBatch`核销进度查询字段与GM二次确认预览字段（FR-OPT-015、RSK-OPT-002） | FR-OPT-006、FR-OPT-012、FR-OPT-015、RSK-OPT-002 |
| 0.3 | 2026-09-01 | 架构师 (Mavis 接手 agent per DEC-008) | 架构师(Mavis 接手 agent per DEC-008) | 落实"各BAS文档功能章节加log设计且区分debug/release级"总要求（per Ulysses 2026-09-01 15:52 JST 决策，4 拍板选项：全部 36 个BAS / 详尽版5列表 / 派worker并行 / BAS-004同步升级）：§2.1（组件划分 + 推送内容脱敏校验 FR-OPT-006 落地）/§2.2（发送时序，同意边界 + 频率限制 + 渠道投递）/§3.1（数据模型，RedemptionCodeBatch/RedemptionCode/RedemptionRecord 生命周期）/§3.2（核销时序，幂等防重放 + 条件更新防超发）共 4 个"本功能日志设计"小节全部新增；每节均含 5 列详尽版（字段名/触发条件/频率估算/采样策略/脱敏与成本），显式区分 `info!`/`warn!`/`error!`（release 必出，编译期常驻，per BAS-004 v0.3 §6.2 强制全采样白名单）与 `debug!`/`trace!`（`#[cfg(debug_assertions)]` 守护，debug-only，release build 完全剔除零运行时开销）两类事件；字段名前缀 `push.*`（区别于 BAS-002 `mnt.*` / BAS-003 `gm.*` / BAS-004 `log.*` / BAS-005 `plugin.*` / BAS-009 `gov.*`），命名严格 snake_case 与 BAS-004 v0.3 §4.3.1/§4.3.2 保持拼写一致（FR-LOG-013）；覆盖 ARC-037 消息推送与兑换码域全链路——组件生命周期（consent store / gateway adapter / sanitizer / dispatcher 启停）/ 推送内容脱敏（FR-OPT-006 拒绝/通过 + 隐私保护 payload dump debug-only）/ 推送发送（到达/点击运营KPI + 失败/重试/DLQ §6.2 强制全采样）/ 推送渠道（APNs/FCM/NATS/邮件/短信 release 必出）/ 兑换码批次生成与二次确认预览（合规审计 §6.2 强制全采样）/ 兑换码核销验证（错误/过期/已用 释放可见）/ 条件更新防超发（并发竞态检测 §6.2 强制全采样）/ 奖励发放结果（合规审计 §6.2 强制全采样）；§4.1 标准化检查清单新增 4 项 log 章节上线检查项（每功能 log 章节存在性 / release 必出 grep 验证 / debug-only 四铁律合规 / release 必出宏未被 `#[cfg]` 守护）；§5 追溯性新增 AC-OPT-006（debug-only 宏 release 完全剔除）与 AC-OPT-007（每功能 BAS 文档须含本功能 log 设计章节），与 BAS-001 v1.5 §4.8.3.4（commit 32d9eb6）/ BAS-002 v0.4（commit f1401a3）/ BAS-003 v0.3（commit 75a001c）/ BAS-004 v0.3 §12（commit 47e26b0+0ee6262）/ BAS-005 v0.3（commit 20b84a1）/ BAS-009 v0.7（commit 9a628cf）形成统一规范 | §2.1、§2.2、§3.1、§3.2、§4.1、§5 |
| 0.4 | 2026-09-02 | 架构师 (Mavis 接手 agent per DEC-008) | 架构师(Mavis 接手 agent per DEC-008) | 落实「処理フロー」段四要素标准 (per 2026-09-02 13:59 JST Ulysses 拍板, RGS-BAS-FLOW-STANDARD-2026-09-02 v0.1): 新增 §1.1 処理フロー（处理流程 / Processing Flow）段, 含主流程图 (mermaid sequenceDiagram, 8 actor: Client / PushDispatcher / ConsentStore / PushGatewayAdapter / APNs-FCM-NATS / RedemptionService / DB / Economy) + 異常分支表 (9 行) + 决策点矩阵 (5 行) + 验证点清单 (9 行), 覆盖推送发送 + 兑换码核销两个主路径; trace_id 贯穿全链路 (per BAS-004 v0.3 §4.4); 事务边界与 Saga 跨域标注 (per BAS-100 v0.1); 与既有 §2.2 发送时序 / §3.2 核销时序 互为详细化引用 | §1.1 |

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 | 2026-08-16 | — |
| 评审（技术） | | | 兑换码核销的幂等键设计是否覆盖并发场景 |
| 审批（负责人） | | | 本文档的基准化 |

---

## 目录

1. [前言](#1-前言)
   1.1 [処理フロー（处理流程 / Processing Flow）](#11-処理フロー处理流程--processing-flow)
2. [推送组件设计](#2-推送组件设计)
3. [兑换码数据模型与核销时序](#3-兑换码数据模型与核销时序)
4. [标准化检查清单](#4-标准化检查清单)
5. [追溯性](#5-追溯性)

---

# 1. 前言

本文档细化RGS-REQ-022定义的ARC-037，全部组件依附既有PL/AD限界上下文运行，不新建独立限界上下文。

### 1.1 処理フロー（处理流程 / Processing Flow）

> 落实 RGS-BAS-FLOW-STANDARD-2026-09-02 v0.1 四要素标准 (per 2026-09-02 13:59 JST Ulysses 拍板)
> 详细时序见 §2.2 发送时序 / §3.2 核销时序, 本段为全景流程 + 异常分支 + 决策点 + 验证点汇总

#### 1.1.1 主流程图 (mermaid sequenceDiagram)

```mermaid
sequenceDiagram
    autonumber
    actor Client as 玩家客户端
    participant PD as PushDispatcher
    participant CS as ConsentStore
    participant PGA as PushGatewayAdapter
    participant APNs as APNs/FCM/NATS
    participant RS as RedemptionService
    participant DB as player_db/social_db
    participant EC as Economy (FR-EC-003)

    Note over Client,EC: trace_id 贯穿全链路, per BAS-004 v0.3 §4.4
    Note over Client,EC: 事务边界: DB 写入同事务; EC 调用走 Saga 跨域, per BAS-100 v0.1

    rect rgb(240, 248, 255)
        Note over Client,APNs: 主路径 1: 推送发送 (per §2.2 详细时序)
        Client->>PD: 业务事件触发推送需求
        PD->>CS: 校验玩家对该类别同意状态
        alt 未同意
            PD-->>Client: 记录跳过原因,直接丢弃
        else 已同意
            PD->>PD: 频率限制校验 (FR-OPT-004)
            alt 超限
                PD-->>Client: 丢弃或排队至下一窗口
            else 未超限
                PD->>PGA: 投递至第三方网关
                PGA->>APNs: 推送 (apns/fcm/nats/email/sms)
                APNs-->>PGA: 投递回执 (delivered)
                PGA-->>PD: 成功
            end
        end
    end

    rect rgb(255, 250, 240)
        Note over Client,EC: 主路径 2: 兑换码核销 (per §3.2 详细时序)
        Client->>RS: 提交兑换码
        RS->>RS: 速率限制 (账号+IP, NFR-OPT-002)
        RS->>DB: 查询 RedemptionCode
        alt 不存在
            RS-->>Client: 拒绝 "码不存在"
        else 已过期 / 已用完
            RS-->>Client: 拒绝 "已过期" / "已用完"
        else 可用
            RS->>DB: 幂等校验 (code, account_id) -> RedemptionRecord
            alt 已存在
                RS-->>Client: 直接返回既有结果 (幂等, NFR-OPT-003)
            else 不存在
                RS->>DB: 条件更新 used_count (FR-OPT-012 防超发)
                alt 受影响行数 = 0
                    RS-->>Client: 拒绝 "已用完" (并发竞态)
                else 受影响行数 = 1
                    RS->>DB: 追加 RedemptionRecord (同事务)
                    RS->>EC: 发放 reward_spec (FR-EC-003 既有路径)
                    EC-->>RS: 发放成功
                    RS-->>Client: 核销成功
                end
            end
        end
    end

    Note over Client,EC: 异常通路 (DLQ + 重试): 网关失败 / EC 不可达 -> ARC-009 消费者标准模式 (重试 3 次 指数退避 100/200/400ms) -> DLQ 报警
```

#### 1.1.2 異常分支表

| 异常点 | 触发条件 | 处理动作 | 用户感知 | 补偿动作 |
|---|---|---|---|---|
| 推送未同意 | ConsentStore 查询=false | 记录跳过原因, 直接丢弃 | 无感知 (符合预期) | 无 |
| 推送频率超限 | 当前窗口已超 limit | 丢弃或排队至下一窗口 (配置决定) | 延迟收到或未收到 | 下一窗口自动重发 (如配置排队) |
| 第三方网关 5xx | APNs/FCM 5xx 或超时 | 重试 3 次 指数退避 100/200/400ms (per ARC-009) | 推送延迟 | DLQ 路由, 运营 SRE 介入 |
| 兑换码不存在 | RedemptionCode 查询 0 行 | 拒绝 "码不存在" | 提示"码不存在" | 无 (用户重新输入) |
| 兑换码过期 | expire_at < now | 拒绝 "已过期" | 提示"已过期" | 无 (用户放弃) |
| 兑换码已用完 (预检) | used_count >= max_uses_per_code | 拒绝 "已用完" | 提示"已用完" | 无 (用户放弃) |
| 条件更新竞态 | UPDATE 受影响行数=0 (并发) | 拒绝 "已用完" (实际并发命中) | 提示"已用完" | 事务回滚 (本事务不写 RedemptionRecord) |
| EC 发放失败 | economy 域不可达 / request_id 冲突 | 整体回滚 (used_count 递增 + RedemptionRecord 全部回滚) | 提示"服务暂不可用" | Saga 补偿 / DLQ 报警 |
| 事务提交失败 | DB 写失败 (网络/约束冲突) | 整体回滚 | 提示"核销失败,请重试" | 客户端重试 (幂等键 request_id 保证) |

#### 1.1.3 决策点矩阵

| 决策点 | 条件 | 主分支 | 备选分支 | 触发后果 |
|---|---|---|---|---|
| 推送通道选择 | 玩家在线状态 + 设备类型 | 在线 -> NATS 实时; 离线 -> APNs (iOS) / FCM (Android) | 强制 APNs/FCM (即使在线) | 用户感知: 实时 (NATS) / 系统通知 (APNs/FCM) |
| 频率超限处理 | 类别配置 (drop vs queue) | drop: 丢弃; queue: 排队至下一窗口 | 强制 drop (紧急公告场景, operator 临时关闭) | 用户感知: 未收到 / 延迟收到 |
| 兑换码可用性 | used_count < max_uses_per_code AND expire_at > now | 接受核销 | 拒绝 "已用完" / "已过期" | 用户感知: 进入核销流程 / 拒绝 |
| 幂等命中 | RedemptionRecord(code, account_id) 已存在 | 直接返回既有结果 (NFR-OPT-003) | 重发奖励 (不推荐, 可能超发) | 用户感知: 重复提交无副作用 |
| 条件更新结果 | UPDATE RedemptionCode SET used_count = used_count + 1 WHERE code = ? AND used_count < max_uses_per_code 受影响行数 | =1 -> 继续; =0 -> 拒绝 "已用完" | 不条件更新 (先读后写, 高并发竞态) | 用户感知: 防超发保证 |

#### 1.1.4 验证点清单

| 验证时机 | 验证内容 | 通过标准 | 失败处理 |
|---|---|---|---|
| ConsentStore 查询 | 玩家对该推送类别的同意状态 | query result = true (同意) | 直接丢弃, 记录 `push.dispatch.consent_denied` |
| 频率限制预检 | 当前窗口内该类别推送计数 | count < limit (未超限) | 丢弃或排队, 记录 `push.dispatch.frequency_limited` |
| 兑换码存在性 | RedemptionCode.code 唯一索引查询 | result row exists (码存在) | 拒绝"码不存在", 记录 `push.redemption.code_not_found` |
| 兑换码有效期 | expire_at > now() | expire_at 严格大于当前时间 | 拒绝"已过期", 记录 `push.redemption.expired` |
| 兑换码可用量 (预检) | used_count < max_uses_per_code | 严格小于 | 拒绝"已用完", 记录 `push.redemption.used_up` |
| 幂等性 | RedemptionRecord(code, account_id) 唯一索引 | 0 行 (本账号本码首次) 或 1 行 (已存在, 走幂等返回) | 不写新记录, 记录 `push.redemption.idempotent_replay` |
| 条件更新成功 | UPDATE 受影响行数 | = 1 (严格条件更新成功) | 拒绝"已用完" (实际并发命中), 记录 `push.redemption.used_count_conditional_update_failed` |
| EC 发放成功 | economy 域 request_id 返回 success | success = true | 整体回滚 (used_count + RedemptionRecord), 记录 `push.redemption.reward_grant_failed` |
| 事务提交 | DB tx_id COMMIT 返回 | tx_id 成功写入 | 整体回滚, 记录 `push.redemption.transaction_rolled_back` |

---

# 2. 推送组件设计

## 2.1 组件划分

| 组件 | 归属限界上下文 | 职责 |
|---|---|---|
| `PushConsentStore` | PL | 存储玩家分类别的推送同意状态 |
| `PushDispatcher` | PL | 发送前校验同意状态与频率限制，投递至第三方推送网关 |
| `PushGatewayAdapter` | PL | 对接APNs/FCM等第三方网关的适配层，密钥复用既有Secrets管理 |
| `PushContentSanitizer` | PL | 推送内容发送前的敏感信息校验（FR-OPT-006），复用既有日志脱敏同类规则集 |

### 2.1.1 推送内容脱敏校验（FR-OPT-006落地）

`PushContentSanitizer`在`PushDispatcher`投递前对推送标题/正文做规则校验：**禁止**出现账号密文以外的个人可识别信息模式（邮箱、手机号、身份证号等正则模式匹配，复用既有日志脱敏规则集的模式库）。命中禁止模式时**拒绝发送**并记录告警（而非静默脱敏后继续发送，因为推送场景文案通常为运营预先配置的模板，命中禁止模式大概率意味着模板变量注入了不该出现的字段，应阻断而非静默处理）。

### 2.1 本功能日志设计

本节覆盖**推送组件生命周期 + `PushContentSanitizer` 脱敏校验**两条链路的观察点——`PushConsentStore`／`PushGatewayAdapter`／`PushContentSanitizer`／`PushDispatcher` 四个组件**不**直接产生业务推送事件（业务推送事件归 §2.2 发送时序），但其就绪状态、配置加载、敏感规则命中/拒绝是 SRE 在 Prometheus/Grafana 上追踪"推送能力是否可用"与"是否有泄漏 PII 的模板外发"的核心信号。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `push.component.consent_store.loaded` | `PushConsentStore` 启动时已加载分类别同意配置（玩家级覆盖） | 每节点启动 1 次 | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2） | 无敏感字段；含 `node_id` / `category_count`；约 250B/条 × 启动频次 = 极低 |
| `push.component.consent_store.load_failed` | `PushConsentStore` 启动加载失败（DB 连接失败等） | 极少（部署事故） | release 必出（100% 强制全采样） | 含 `node_id` / `error` / `trace_id`；约 320B/条 |
| `push.component.gateway_adapter.initialised` | `PushGatewayAdapter` 完成 APNs/FCM 凭据加载与健康探测 | 每节点启动 1 次 | release 必出（100% 强制全采样） | 无敏感字段；含 `node_id` / `channel` / `healthcheck_result`；约 280B/条 |
| `push.component.gateway_adapter.healthcheck_failed` | 任一第三方网关（APNs/FCM）健康探测失败 | 偶发 | release 必出（100% 强制全采样） | 含 `node_id` / `channel` / `error` / `trace_id`；约 320B/条 |
| `push.component.sanitizer.initialised` | `PushContentSanitizer` 启动时已加载禁止模式正则库 | 每节点启动 1 次 | release 必出（100% 强制全采样） | 无敏感字段；含 `node_id` / `pattern_count`；约 250B/条 |
| `push.component.sanitizer.rejected` | `PushContentSanitizer` 命中禁止模式并**拒绝发送**（FR-OPT-006 落地，安全事件） | 偶发（运营配置错） | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2） | **不**含 PII payload 原文（仅含匹配模式类别 / 模板 ID）；含 `template_id` / `pattern_category`（邮箱/手机号/身份证号/...）/ `matched_at_offset`；约 350B/条 |
| `push.component.sanitizer.allowed` | 推送内容通过 `PushContentSanitizer` 校验，可继续投递 | 取决于业务触发（典型 10-100/s 集群） | release 必出（100% 强制全采样） | 含 `template_id` / `template_version` / `category`（推送类别，FR-OPT-004 频率限制键）；约 280B/条 × 100/s ≈ 28KB/s 稳态 |
| `push.component.dispatcher.started` | `PushDispatcher` 已就绪，开始接受业务事件触发 | 每节点启动 1 次 | release 必出（100% 强制全采样） | 无敏感字段；含 `node_id` / `bound_channel_count`；约 230B/条 |
| `push.component.dispatcher.shutdown.completed` | 宿主服务优雅关闭，`PushDispatcher` 已停止接受新事件（in-flight 任务已 await） | 每节点关闭 1 次 | release 必出（100% 强制全采样） | 含 `node_id` / `inflight_count` / `shutdown_kind`（SIGTERM/HPA scale-in）；约 280B/条 |
| `push.component.sanitizer.debug.payload_dump` | 推送 payload 完整 dump（标题/正文/模板变量值） | 业务触发同频 | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除） | 约 1-5KB/条（payload 大小决定，**含 PII 风险**，release 剔除避免生产误开） |
| `push.component.sanitizer.debug.regex_match_trace` | 禁止模式正则的匹配细节（哪条规则命中 + 字符偏移） | 偶发 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 300B/条（release 剔除） |

**debug-only 守护要点**（落实 BAS-004 v0.3 §4.4）：
- `push.component.sanitizer.debug.payload_dump` **必须** `#[cfg(debug_assertions)]` 守护——推送 payload 可能含 PII（运营填错的真实玩家昵称/邮箱前缀），release 误开 `RUST_LOG=debug` 时**不能**让其泄密通道
- `push.component.sanitizer.rejected` 是**安全事件**——release 必出 + §6.2 强制全采样，便于安全审计识别"是否有运营误配模板注入 PII"
- `push.component.sanitizer.rejected` 的字段最小集**严格不**含 PII payload 原文（仅记录模板 ID + 模式类别 + 字符偏移），规避"日志自身变成 PII 泄漏源"的反模式

## 2.2 发送时序（同意边界落地，ARC-037①）

```
业务事件触发推送需求（如好友邀请、活动开始，复用既有事件基础设施ARC-010）
  → PushDispatcher查询PushConsentStore，校验目标玩家对该类别的同意状态
  → 未同意 → 直接丢弃，不投递，记录跳过原因
  → 已同意 → 校验频率限制（FR-OPT-004）
      超限 → 丢弃或排队至下一可发送窗口（由类别配置决定）
      未超限 → 经PushGatewayAdapter投递至第三方网关
  → 网关返回失败（如设备令牌过期）→ 记录失败，不无限重试（复用ARC-009消费者标准模式）
```

### 2.2 本功能日志设计

本节覆盖**推送发送全链路**的观察点——同意校验、频率限制、渠道投递、到达回执、失败重试、DLQ 路由是运营 KPI（推送到达率/点击率）与 SRE 告警（推送渠道不可用/重试风暴/DLQ 堆积）的核心数据源。所有推送发送结果均 release 必出；推送失败/重试/DLQ 走 BAS-004 v0.3 §6.2 强制全采样白名单（错误路径 + 高危操作）；推送 payload 内容仅 debug-only（隐私 + 性能）。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `push.dispatch.consent_denied` | 业务事件触发推送但玩家未同意该类别（`PushConsentStore` 查询结果=false） | 偶发（受推送类别开通率影响） | release 必出（100% 强制全采样） | 含 `player_id` / `category` / `event_id`；约 250B/条 |
| `push.dispatch.frequency_limited` | 玩家对类别已超频率限制（FR-OPT-004 落地），本轮丢弃 | 偶发（取决于运营推送强度） | release 必出（100% 强制全采样） | 含 `player_id` / `category` / `current_count` / `limit` / `window`；约 300B/条 |
| `push.dispatch.queued_for_next_window` | 类别配置为排队（而非丢弃），推送被排入下一可发送窗口 | 偶发 | release 必出（100% 强制全采样） | 含 `player_id` / `category` / `next_window_at`；约 280B/条 |
| `push.dispatch.channel_dispatched` | `PushGatewayAdapter` 已将推送投递至第三方网关（APNs/FCM/NATS/邮件/短信） | 取决于业务触发（典型 10-100/s 集群） | release 必出（100% 强制全采样） | 含 `player_id` / `category` / `channel`（apns/fcm/nats/email/sms）/ `template_id` / `request_id`；约 320B/条 × 100/s ≈ 32KB/s 稳态 |
| `push.dispatch.delivered` | 第三方网关回执"已到达设备"（到达事件） | 典型约为发送的 70-95%（受网络影响） | release 必出（100% 强制全采样，运营 KPI 关键） | 含 `player_id` / `category` / `channel` / `delivered_at` / `latency_ms`；约 300B/条 |
| `push.dispatch.clicked` | 玩家点击推送通知（点击事件，运营 KPI） | 典型约为到达的 5-20% | release 必出（100% 强制全采样，运营 KPI 关键） | 含 `player_id` / `category` / `channel` / `clicked_at` / `click_latency_ms`；约 300B/条 |
| `push.dispatch.failed.gateway_rejected` | 第三方网关返回非 2xx（如设备令牌过期、payload 超限） | 偶发 | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 错误路径强制全采样） | 含 `player_id` / `category` / `channel` / `gateway_status_code` / `error` / `request_id`；约 350B/条 |
| `push.dispatch.failed.sanitizer_blocked` | 推送内容被 `PushContentSanitizer` 拒绝（FR-OPT-006，安全事件，§2.1.1 落地） | 极少（运营配置错） | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2） | **不**含 payload 原文；含 `template_id` / `pattern_category` / `request_id`；约 350B/条 |
| `push.dispatch.retry_scheduled` | 网关临时不可用（5xx/超时），已入重试队列（按 ARC-009 既有 outbox+saga 模式） | 偶发 | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 错误路径） | 含 `player_id` / `category` / `channel` / `retry_count` / `next_retry_at` / `request_id`；约 320B/条 |
| `push.dispatch.retry_exhausted` | 重试次数达上限，转 DLQ（per BAS-004 v0.3 §6.2 错误路径强制全采样） | 极少 | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 错误路径） | 含 `player_id` / `category` / `channel` / `final_retry_count` / `last_error` / `request_id`；约 380B/条 |
| `push.dispatch.dlq_routed` | 推送事件进入 DLQ（ARC-009 既有 DLQ 通路，per BAS-004 v0.3 §6.2 错误路径强制全采样） | 极少 | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 错误路径） | 含 `player_id` / `category` / `channel` / `dlq_topic` / `error` / `request_id`；约 380B/条 |
| `push.dispatch.frequency_limit_disabled` | 类别频率限制被运营临时关闭（如紧急公告场景） | 极低 | release 必出（100% 强制全采样） | 含 `category` / `operator_id` / `disabled_until`；约 280B/条 |
| `push.dispatch.debug.gateway_response_body` | 第三方网关（APNs/FCM）的完整响应体 dump | 业务触发同频 | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除） | 约 0.5-2KB/条（release 剔除） |
| `push.dispatch.debug.dispatch_timing` | `PushDispatcher` 内部各阶段耗时（consent 查询 + 频率判定 + sanitizer + 投递）的逐次时间戳 | 业务触发同频 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 250B/条（release 剔除） |
| `push.dispatch.debug.outbox_payload_dump` | outbox 事件 payload dump（含 `event_id` / `aggregate_id` / `partition_key`） | 业务触发同频 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 0.5-3KB/条（release 剔除） |

**debug-only 守护要点**（落实 BAS-004 v0.3 §4.4）：
- `push.dispatch.debug.gateway_response_body` **必须** `#[cfg(debug_assertions)]` 守护——APNs/FCM 响应体可能含设备级 PII（设备 ID/用户标识），release 误开 RUST_LOG=debug 时**不能**泄漏
- `push.dispatch.debug.outbox_payload_dump` 同样**必须**守护——outbox payload 包含**全部**业务字段（含推送正文/玩家 ID），隐私风险极高
- `push.dispatch.failed.*` / `push.dispatch.retry_*` / `push.dispatch.dlq_routed` 走 BAS-004 v0.3 §6.2 强制全采样白名单（错误路径）——这是运营 SRE 排查"为什么玩家没收到推送"的核心证据链，**不能**采样丢弃
- `push.dispatch.delivered` / `push.dispatch.clicked` 是**运营 KPI 数据源**——release 必出 + 100% 采样，便于按 `category` / `channel` 维度做"按周到达率/点击率"报表（FR-OPT-004 频率限制调优与运营投放 ROI 评估均依赖此数据）

---

# 3. 兑换码数据模型与核销时序

## 3.1 数据模型

`RedemptionCodeBatch`：

| 字段 | 类型 | 说明 |
|---|---|---|
| `batch_id` | uuid | 批次标识 |
| `reward_spec` | 引用既有物品/货币发放规格 | 兑换奖励内容 |
| `expire_at` | timestamp | 有效期 |
| `max_uses_per_code` | int | 单码可兑换次数（1为一次性） |
| `preview_confirmed_by` | 可选，GM操作者ID | RSK-OPT-002二次确认预览的确认人，批次生成前须先展示`reward_spec`/`expire_at`等关键参数预览并记录确认人，未确认不得进入实际批量生成（复用RGS-BAS-003控制平面二次确认机制） |

`RedemptionCode`：

| 字段 | 类型 | 说明 |
|---|---|---|
| `code` | string，唯一索引 | 兑换码本体（高熵随机生成，TBD-OPT-002） |
| `batch_id` | 引用`RedemptionCodeBatch` | 所属批次 |
| `used_count` | int | 已使用次数 |

索引：`code`唯一索引兼作主键；`(batch_id)`索引支撑FR-OPT-015批次核销进度查询（`SELECT count(*), sum(used_count) ... WHERE batch_id=? GROUP BY ...`聚合已核销数/剩余数，作为GM后台批次进度展示的数据来源，复用RGS-BAS-003§3.4只读查询模式，无需专属查询工具）。

`RedemptionRecord`：

| 字段 | 类型 | 说明 |
|---|---|---|
| `code` / `account_id` | — | 幂等键组合：同一账号对同一码的核销请求 |
| `redeemed_at` | timestamp | 核销时间 |

### 3.1 本功能日志设计

本节覆盖**兑换码数据模型生命周期**的观察点——`RedemptionCodeBatch` / `RedemptionCode` / `RedemptionRecord` 三表 CRUD 与状态变化是合规审计的核心证据（"谁在何时生成了多少兑换码 / 兑换码在何时被使用 / 作废何时发生"）。全部批次生成/分发/兑换/作废事件 release 必出 + §6.2 强制全采样（合规审计白名单）；批量生成细节（单码 dump）debug-only（隐私 + 性能）。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `push.redemption.batch.preview_requested` | GM 在 GM 后台请求预览（`preview_confirmed_by` 二次确认机制，RSK-OPT-002 落地） | 极低（运营触发） | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 错误路径） | 含 `batch_draft_id` / `operator_id` / `reward_spec_summary` / `code_count` / `expire_at`；约 380B/条 |
| `push.redemption.batch.preview_confirmed` | GM 确认预览后进入实际批量生成（RSK-OPT-002 二次确认完成留痕） | 极低 | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 错误路径 / 合规审计） | 含 `batch_draft_id` / `operator_id` / `preview_hash`（预览内容 hash，用于审计核验）；约 320B/条 |
| `push.redemption.batch.preview_rejected` | GM 拒绝预览，二次确认未通过 | 极低 | release 必出（100% 强制全采样） | 含 `batch_draft_id` / `operator_id` / `rejection_reason`；约 300B/条 |
| `push.redemption.batch.generation_started` | `RedemptionCodeBatch` 已写入，批量生成 `RedemptionCode` 异步任务开始 | 极低（运营活动触发） | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 错误路径 / 合规审计） | 含 `batch_id` / `operator_id` / `code_count` / `reward_spec_summary` / `expire_at`；约 380B/条 |
| `push.redemption.batch.generation_completed` | 全部 `RedemptionCode` 已写入，生成任务成功结束 | 极低 | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 合规审计） | 含 `batch_id` / `code_count` / `duration_ms`；约 280B/条 |
| `push.redemption.batch.generation_failed` | 生成任务失败（如 DB 写失败、`code` 唯一索引冲突） | 极少 | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 错误路径） | 含 `batch_id` / `error` / `partial_written_count` / `trace_id`；约 380B/条 |
| `push.redemption.batch.distributed` | 批次兑换码通过运营渠道下发（邮件/活动页/外部系统） | 极低 | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 合规审计） | 含 `batch_id` / `distribution_channel` / `recipient_summary`（不含具体玩家 ID）；约 300B/条 |
| `push.redemption.batch.expire_warning` | 批次即将过期（如 24h 内）已发出运营告警 | 偶发 | release 必出（100% 强制全采样） | 含 `batch_id` / `expire_at` / `unused_count`；约 280B/条 |
| `push.redemption.batch.expired` | 批次已过期（FR-OPT-015 批次的 GM 视图字段触发） | 偶发 | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 合规审计） | 含 `batch_id` / `expire_at` / `unused_count`；约 280B/条 |
| `push.redemption.batch.revoked` | GM 主动作废批次（如发现泄漏/活动取消） | 极低 | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 高危操作强制全采样） | 含 `batch_id` / `operator_id` / `reason` / `revoked_at`；约 320B/条 |
| `push.redemption.code.created` | 单个 `RedemptionCode` 已写入（批量生成期间逐条） | 取决于批次规模（典型 100-100000/批） | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 合规审计） | **不**含 `code` 原文（仅含 `code_hash` 截断 hash，用于审计关联但不可逆推出真实码）；含 `batch_id` / `code_hash` / `max_uses_per_code`；约 280B/条 × 1000/批 = 280KB/批 |
| `push.redemption.record.appended` | `RedemptionRecord` 已写入（核销成功后，作为**不可变**审计行追加） | 取决于核销频次（典型 1-100/s 集群，营销活动时峰值更高） | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 合规审计） | 含 `code_hash` / `account_id`（明文允许，per BAS-004 v0.3 §5.1）/ `redeemed_at` / `request_id`；约 300B/条 × 100/s ≈ 30KB/s 稳态 |
| `push.redemption.record.immutable_violation` | 检测到 `RedemptionRecord` 被尝试 UPDATE/DELETE（不可变违规，安全事件） | 极少（应用代码错） | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 错误路径） | 含 `record_id` / `attempted_op` / `db_user` / `trace_id`；约 300B/条 |
| `push.redemption.code.debug.code_entropy_dump` | 单个 `RedemptionCode` 的明文 dump（用于生成期一次性熵值验证，**绝不**进入 release） | 批量生成期间逐条 | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除） | 约 50-100B/条（release 剔除，单码本身已是高熵随机字符串） |

**debug-only 守护要点**（落实 BAS-004 v0.3 §4.4）：
- `push.redemption.code.debug.code_entropy_dump` **必须** `#[cfg(debug_assertions)]` 守护——单码明文**绝对不能**进入 release 日志通道（一旦 release 误开 RUST_LOG=debug，运营/SRE 可读到全部明文码，即合规事故）。该字段**仅**用于生成期一次性熵值验证（开发/CI 阶段）
- `push.redemption.code.created` 的字段最小集**严格不**含 `code` 原文（仅含 `code_hash` 截断 hash）——这是与 §2.1.1 `push.component.sanitizer.rejected` 同源的"PII 防控"设计纪律：审计留痕用 hash 关联即可，明文码的访问通过业务路径核销
- `push.redemption.batch.preview_confirmed` / `push.redemption.batch.generation_started` / `push.redemption.batch.revoked` 全部走 §6.2 强制全采样（高危操作 / 合规审计）——这是"事后追溯'谁在何时生成了哪些批次'、'谁在何时作废了哪些批次'"的唯一证据链，**不能**采样丢弃
- `push.redemption.record.immutable_violation` 是**安全事件**——release 必出 + §6.2 强制全采样，便于安全审计链路完整追溯（与 BAS-005 §3.3 `plugin.registry.audit_immutable_violation` 同源设计）

## 3.2 核销时序（幂等防重放，ARC-037②）

```
玩家提交兑换码
  → 速率限制校验（NFR-OPT-002，按账号/IP多层限制，复用既有NFR-SEC-008）
  → 查询RedemptionCode：不存在 → 拒绝（"码不存在"）
  → 校验expire_at：已过期 → 拒绝（"已过期"）
  → 校验used_count < max_uses_per_code：已用完 → 拒绝（"已用完"）
  → 幂等校验：查询RedemptionRecord(code, account_id)是否已存在
      已存在 → 直接返回既有结果，不重复发放（NFR-OPT-003）
      不存在 → 原子递增used_count + 写入RedemptionRecord + 复用FR-EC-003确定请求路径发放reward_spec（同一事务边界）
```

> 并发防超发（FR-OPT-012补强）：`used_count`递增**必须**以条件更新实现（`UPDATE RedemptionCode SET used_count = used_count + 1 WHERE code = ? AND used_count < max_uses_per_code`），并以受影响行数（0或1）判定本次递增是否成功，**不得**采用"先SELECT读取used_count再判断再UPDATE"的先读后写模式——后者在高并发核销同一兑换码（如营销活动导致的瞬时并发提交）时存在竞态窗口，可能突破`max_uses_per_code`上限超发。条件更新失败（受影响行数为0）时按"已用完"分支返回拒绝，即使此前的`used_count < max_uses_per_code`预检已通过（预检结果可能因并发已过期，须以本次条件更新的实际结果为准）。

### 3.2 本功能日志设计

本节覆盖**兑换码核销全链路**的观察点——核销验证（错误/过期/已用）、幂等防重放、条件更新防超发、奖励发放结果是合规审计的"事后追溯'某个码被谁在何时兑换'"的核心证据链。所有核销结果事件 release 必出；条件更新竞态检测、奖励发放结果走 BAS-004 v0.3 §6.2 强制全采样（错误路径 / 合规审计 / 高危操作）；核销请求体与事务边界细节 debug-only。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `push.redemption.request.received` | `RedemptionService.Redeem` 进入处理（携带 `request_id` 幂等键） | 取决于业务触发（典型 1-100/s 集群，营销活动时峰值更高） | release 必出（100% 强制全采样） | 含 `request_id` / `code_hash` / `account_id` / `client_ip`（per BAS-004 v0.3 §5.1 网段掩码）/ `entry_kind`（gm_ui/player_api/...）；约 320B/条 × 100/s ≈ 32KB/s 稳态 |
| `push.redemption.rate_limited.account` | 账号级速率限制命中（NFR-OPT-002 / NFR-SEC-008 复用） | 偶发 | release 必出（100% 强制全采样） | 含 `account_id` / `current_rate` / `limit` / `window` / `request_id`；约 300B/条 |
| `push.redemption.rate_limited.ip` | IP 级速率限制命中（NFR-OPT-002 暴力枚举防护） | 偶发 | release 必出（100% 强制全采样） | 含 `client_ip`（网段掩码）/ `current_rate` / `limit` / `window` / `request_id`；约 300B/条 |
| `push.redemption.code_not_found` | `RedemptionCode` 查询不存在（"码不存在"分支） | 偶发（可能为暴力枚举试探） | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 错误路径） | 含 `code_hash` / `account_id` / `client_ip`（网段掩码）/ `request_id`；约 300B/条 |
| `push.redemption.expired` | `expire_at` 已过（"已过期"分支） | 偶发 | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 错误路径） | 含 `code_hash` / `batch_id` / `expire_at` / `request_id`；约 300B/条 |
| `push.redemption.used_up` | 预检发现 `used_count >= max_uses_per_code`（"已用完"分支） | 偶发 | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 错误路径） | 含 `code_hash` / `batch_id` / `used_count` / `max_uses_per_code` / `request_id`；约 320B/条 |
| `push.redemption.idempotent_replay` | 幂等键命中既有 `RedemptionRecord`（`code` + `account_id` 已存在），直接返回既有结果（NFR-OPT-003 落地，**不**视为错误） | 偶发 | release 必出（100% 强制全采样） | 含 `code_hash` / `account_id` / `existing_record_id` / `original_redeemed_at` / `request_id`；约 350B/条 |
| `push.redemption.used_count_conditional_update_succeeded` | `used_count` 条件更新成功（受影响行数=1，FR-OPT-012 落地，FR-OPT-012 是高危操作的"防超发"关键） | 取决于核销频次 | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 高危操作强制全采样） | 含 `code_hash` / `batch_id` / `new_used_count` / `max_uses_per_code` / `request_id`；约 350B/条 |
| `push.redemption.used_count_conditional_update_failed` | 条件更新失败（受影响行数=0，**可能**为预检通过后的并发竞态，FR-OPT-012 落地，"已用完"分支实际命中） | 偶发（高并发核销时） | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 错误路径 / 高危操作） | 含 `code_hash` / `batch_id` / `precheck_used_count` / `request_id`；约 350B/条 |
| `push.redemption.reward_granted` | `reward_spec` 已通过 FR-EC-003 既有发放路径成功发放（同事务边界内） | 取决于核销频次 | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 合规审计） | 含 `code_hash` / `batch_id` / `account_id` / `reward_spec_summary` / `ec_request_id`（EC 既有 `request_id` 幂等键）/ `request_id`；约 380B/条 |
| `push.redemption.reward_grant_failed` | 奖励发放失败（EC 不可达 / `request_id` 冲突 / 事务回滚） | 极少 | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 错误路径 / 合规审计） | 含 `code_hash` / `batch_id` / `account_id` / `error` / `ec_request_id` / `trace_id`；约 400B/条 |
| `push.redemption.transaction_committed` | 核销主事务已提交（`used_count` 递增 + `RedemptionRecord` 追加 + 奖励发放在同一事务边界内原子完成） | 取决于核销频次 | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 合规审计） | 含 `code_hash` / `account_id` / `db_tx_id` / `request_id`；约 320B/条 |
| `push.redemption.transaction_rolled_back` | 核销主事务回滚（任意子步骤失败，按 ARC-005 既有事务纪律整体回滚） | 极少 | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 错误路径） | 含 `code_hash` / `account_id` / `error` / `db_tx_id` / `trace_id` / `request_id`；约 380B/条 |
| `push.redemption.debug.request_envelope` | 核销请求全部字段（含入参明文 `code` / `account_id` / 客户端元数据） | 业务触发同频 | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除） | 约 0.3-1KB/条（**含 PII 风险**，release 剔除避免生产误开） |
| `push.redemption.debug.transaction_boundary_trace` | 核销主事务各阶段（条件更新 / 记录追加 / 奖励发放）的逐次时间戳与 sql 状态 | 业务触发同频 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 400B/条（release 剔除） |
| `push.redemption.debug.rate_limit_bucket_state` | 账号级 / IP 级速率限制桶的当前状态（令牌数/补充速率） | 偶发 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 200B/条（release 剔除） |

**debug-only 守护要点**（落实 BAS-004 v0.3 §4.4）：
- `push.redemption.debug.request_envelope` **必须** `#[cfg(debug_assertions)]` 守护——核销请求体含 `code` 明文 + `account_id`，release 误开 RUST_LOG=debug 时**不能**泄漏（与 §2.1.1 / §3.1 `push.redemption.code.debug.code_entropy_dump` 同源 PII 防控纪律）
- `push.redemption.used_count_conditional_update_succeeded` / `push.redemption.used_count_conditional_update_failed` / `push.redemption.reward_granted` / `push.redemption.reward_grant_failed` 走 BAS-004 v0.3 §6.2 强制全采样白名单（高危操作 / 错误路径 / 合规审计）——这是"事后追溯'某个码是否被超发'、'某个玩家是否在何时获得了某奖励'"的核心证据链，**不能**采样丢弃
- `push.redemption.idempotent_replay` 虽**不**视为错误，但 release 必出 + 100% 采样——便于安全审计识别"是否有客户端在重放核销请求探测幂等性"，与 NFR-OPT-003 / NFR-SEC-008 落地纪律一致
- `push.redemption.rate_limited.ip` 是**安全事件**信号（疑似暴力枚举试探）——release 必出 + 100% 采样，便于安全运营关联分析（同一 IP 多次 `code_not_found` + `rate_limited.ip` 出现 = 暴力枚举攻击进行中）

---

# 4. 标准化检查清单

## 4.1 上线前检查清单

- [ ] 推送同意边界验证：关闭类别后不再收到该类别推送
- [ ] 兑换码并发重复提交测试：验证仅发放一次
- [ ] 兑换码暴力枚举防护测试：速率限制生效
- [ ] 批量生成兑换码异步任务验证：不阻塞GM后台其他操作
- [ ] 推送内容脱敏校验（§2.1.1）命中禁止模式时验证为拒绝发送而非静默继续
- [ ] `used_count`条件更新并发压测：模拟同一码高并发核销，验证不超过`max_uses_per_code`
- [ ] 兑换码批次生成前的二次确认预览（`preview_confirmed_by`）流程已验证不可绕过
- [ ] **每功能章节（§2.1/§2.2/§3.1/§3.2）均含"本功能日志设计"子节**，且明确区分 `info!`/`warn!`/`error!`（release 必出）与 `debug!`/`trace!`（`#[cfg(debug_assertions)]` 守护，debug-only）两类
- [ ] release 必出事件清单（§2.1/§2.2/§3.1/§3.2）逐项可在本功能代码中检索到对应调用点（grep 验证），未遗漏业务关键事件（推送发送/到达/点击运营KPI、推送失败/重试/DLQ §6.2 强制全采样、兑换码生成/分发/兑换/作废 §6.2 强制全采样、兑换码验证错误/过期/已用、推送渠道 APNs/FCM/NATS/邮件/短信）
- [ ] debug-only 事件严格遵守 RGS-BAS-004 v0.3 §4.3 四条铁律（宏直接守护、避免 `if cfg!` 外层、参数 O(1)、关联 ID 预先 `let` 绑定）
- [ ] release build 中**不**存在 `info!`/`warn!`/`error!` 被 `#[cfg(debug_assertions)]` 守护的代码点（grep 验证）

## 4.2 代码评审检查清单

- [ ] 推送发送路径均先查询`PushConsentStore`，无绕过校验的直接投递代码
- [ ] 兑换码发放路径复用FR-EC-003，无独立发放旁路

---

# 5. 追溯性

| 需求ID | 本设计书章节 |
|---|---|
| ARC-037、FR-OPT-001〜006 | §2、§2.1.1（内容脱敏校验） |
| FR-OPT-010〜016 | §3、§3.2（并发防超发） |
| NFR-OPT-001〜004 | §2.2、§3.2 |
| AC-OPT-001〜004 | §4.1 |
| TBD-OPT-001〜002、RSK-OPT-001〜002 | §4.1 |
| **AC-OPT-006（debug-only 宏在 release build 完全剔除）** | §2.1/§2.2/§3.1/§3.2 各节"debug-only 守护要点"项 + RGS-BAS-004 v0.3 §4.3 四条铁律 + §4.1 检查项第 8/9/10/11 条 | FR-LOG-012 |
| **AC-OPT-007（每功能 BAS 文档须含本功能 log 设计章节）** | §2.1/§2.2/§3.1/§3.2 各"本功能日志设计"小节 + §4.1 检查项第 8 条（每功能 log 章节存在性）+ §4.1 检查项第 9 条（release 必出事件 grep 验证）+ §4.1 检查项第 10 条（debug-only 四铁律合规）+ §4.1 检查项第 11 条（release 必出宏未被 `#[cfg]` 守护） | FR-LOG-010/011/012 |

---

> 本文档与RGS-REQ-022（消息推送与兑换码运营工具 需求定义书）配套使用。
