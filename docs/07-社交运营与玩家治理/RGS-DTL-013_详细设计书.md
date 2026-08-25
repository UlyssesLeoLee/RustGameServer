# 详细设计书（詳細設計書 / Detailed Document）

**大厅、社交通信与运营活动：social_db/economy_db物理数据库设计・频道路由与滥用检测协议格式・购买/活动确定请求算法详细设计**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DTL-013 |
| 版本 | 0.3 |
| 父文档 | RGS-BAS-013 v0.3 大厅、社交通信与运营活动 基本设计书（本文档为其详细化，不改变任何既有决定，仅将逻辑设计落实为物理/实现级设计） |
| 依据标准 | IPA『共通フレーム 2013（SLCP-JCF2013）』详细设计工程 |
| 制定日 | 2026-08-17 |
| 升版日 | 2026-08-25 |
| 制定者 | 架构师 |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响章节 |
|---|---|---|---|---|---|
| 0.1 | 2026-08-17 | 架构师 | — | 初版制定（负责人指示"继续"推进详细设计，本文档与RGS-DTL-011/012/014/019并行产出）。细化RGS-BAS-013§2.2队伍表逻辑字段为`social_db`具体DDL、§4.1商品目录逻辑ER图为`economy_db`具体DDL（原文将物理DDL留待未来集中设计；本文档在DTL体系下以分域方式完成该职责，权威边界见§6）、§3.1/§3.4频道消息与滥用检测落实为具体协议线格式、§3.4滥用检测规则落实为可直接翻译为Rust实现的伪代码、§4.2/§5.1购买与活动奖励时序落实为具体确定请求事务边界（复用RGS-DTL-001§3.2既定OCC+幂等模式）。**本版本不覆盖**：客户端UI渲染细节、活动插件沙箱脚本本身的执行引擎设计（属RGS-BAS-005/009既有范围）。见§6 | 全部 |
| 0.3 | 2026-08-25 | 架构师（Mavis 接手 agent per DEC-008）| — | 同步父 BAS-013 升版至 v0.3 + 补 BAS-013 v0.3 §3.4 ChatAbuseGuard/ChatAbuseSignal 在 DTL 的落实现状复核（§2 `chat_abuse_signals` DDL + §5.2 `chat_abuse_guard_check` 伪代码 + §5.1 路由主流程中的插入位置 + §4 ResultCode 新增 `ABUSE_REPEAT_MESSAGE`/`ABUSE_BANNED_WORD` 三个取值已覆盖，§7 追溯性表对应行存在；不引入新设计，不重写既有结构） | 头部元数据、修订历史、§2/§4/§5.1/§5.2 复核 |

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 | 2026-08-17 | — |
| 评审（技术） | | | DDL是否与RGS-BAS-013§2.2/§4.1逻辑模型/RGS-BAS-007命名规范完全一致，购买/活动事务边界是否真正复用RGS-DTL-001§3.2而非另起一套 |
| 评审（DBA） | | | 索引设计是否覆盖§2.3隐私过滤查询与§3.4滥用检测滚动窗口查询的高频路径 |
| 审批（负责人） | | | 本文档的基准化 |

---

## 目录

1. [前言](#1-前言)
2. [物理数据库设计：social_db队伍与滥用信号](#2-物理数据库设计socialdb队伍与滥用信号)
3. [物理数据库设计：economy_db商品目录与购买记录](#3-物理数据库设计economydb商品目录与购买记录)
4. [协议线格式：频道消息与在线状态](#4-协议线格式频道消息与在线状态)
5. [算法详细设计](#5-算法详细设计)
6. [本文档的覆盖范围与后续计划](#6-本文档的覆盖范围与后续计划)

---

## 1. 前言

### 1.1 定位

RGS-BAS-013给出了大厅作为场景Actor的状态图、在线状态字段级隐私过滤表、`ChatMessage`字段扩展表、频道路由流程图、`ChatAbuseGuard`滥用检测规则表、商品目录逻辑ER图、购买与活动奖励时序图——均为逻辑层面。本文档将其中涉及持久化数据的部分落实为物理DDL，涉及线上传输的部分落实为具体协议格式，涉及判定逻辑的部分落实为可直接翻译为Rust实现的伪代码。

### 1.2 本文档不做什么

- **不重新决定**RGS-BAS-013已确定的任何结构性选择（大厅不新建独立子系统、私聊路由服务端强制不依赖客户端自觉、`ChatAbuseGuard`基础规则独立于智能层可用性、经济类活动判定收归EC单点执行）。
- **不覆盖**客户端UI渲染细节（活动入口图标/跳转参数等，RGS-BAS-013§2.2已明确"不含具体UI渲染"）。
- **不覆盖**活动插件沙箱脚本本身的执行引擎实现——RGS-BAS-013§5仅描述活动插件与经济服务的交互时序，插件沙箱本身的执行环境设计属RGS-BAS-005/009既有范围，本文档不重复展开。

### 1.3 记述规则

沿用既有DTL文档记述规则：DDL以PostgreSQL为准，事件/消息以Protobuf风格给出，算法伪代码可直接对应Rust `Result`实现。

---

## 2. 物理数据库设计：social_db队伍与滥用信号

对应RGS-BAS-013§2.2（队伍持久化于`social_db`新增`team`表）与§3.4（`ChatAbuseSignal`）。两表落位于既有`social_db`（GD限界上下文数据库），本文档只新增表结构。

```sql
-- 队伍表，对应RGS-BAS-013§2.2"持久化于social_db(GD既有数据库)，新增team表"
CREATE TABLE teams (
    team_id       UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    leader_character_id  UUID NOT NULL,   -- 逻辑引用player_db.characters，跨库不建物理FK(同RGS-DTL-001§2/§3既定跨库约束)
    status                 SMALLINT NOT NULL DEFAULT 0,  -- 0=邀请中 1=已确认 2=已解散，对应队伍状态机
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    version                   INTEGER NOT NULL DEFAULT 0,   -- OCC乐观锁，多成员并发确认/退出场景需要
    CONSTRAINT chk_teams_status CHECK (status BETWEEN 0 AND 2)
);

CREATE TABLE team_members (
    team_id         UUID NOT NULL REFERENCES teams(team_id) ON DELETE CASCADE,
    -- ON DELETE CASCADE而非RESTRICT: 队伍本身是纯粹的临时性关联记录(非账号级持久数据)，
    -- 队伍解散时成员关系随之清空是队伍生命周期的自然组成部分，不同于RGS-DTL-001§2.1
    -- accounts→characters那类"账号删除须走审计编排流程"的场景，两者性质不同，不套用同一FK策略
    character_id     UUID NOT NULL,
    member_status      SMALLINT NOT NULL DEFAULT 0,  -- 0=邀请中 1=已确认
    joined_at             TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (team_id, character_id)
);
CREATE UNIQUE INDEX uq_team_members_character_active
    ON team_members (character_id) WHERE member_status = 1;
    -- 部分唯一索引: 同一角色任意时刻至多属于一个"已确认"队伍(业务规则的数据库层强制，
    -- 避免"角色同时属于两支已确认队伍"这一不一致态仅靠应用层校验兜底)

-- 聊天滥用信号表，对应RGS-BAS-013§3.4 ChatAbuseSignal，落位social_db(GD)
CREATE TABLE chat_abuse_signals (
    signal_id       BIGSERIAL PRIMARY KEY,
    character_id     UUID NOT NULL,
    channel           TEXT NOT NULL CHECK (channel IN ('world', 'guild', 'team', 'whisper')),
    rule_hit            TEXT NOT NULL CHECK (rule_hit IN ('repeat_message', 'banned_word')),
    occurred_at           TIMESTAMPTZ NOT NULL DEFAULT now()
) PARTITION BY RANGE (occurred_at);
-- 月度分区，复用RGS-BAS-007既定短生命周期表分区/保留期标准(同RGS-DTL-025 detection_signals同类做法)

CREATE INDEX idx_chat_abuse_signals_character_time
    ON chat_abuse_signals (character_id, occurred_at);
    -- 支撑§5.2重复消息检测的滚动窗口查询("该sender近N秒内的信号/消息历史")
```

`teams`/`team_members`的拆分（而非单表内嵌成员数组）遵循RGS-BAS-007既定的关系范式标准：队伍成员数量随玩法模式不同（2〜5人不等）变化，拆分为独立表避免变长数组字段带来的更新竞争与索引局限，`uq_team_members_character_active`部分唯一索引是RGS-BAS-013§2.2"队伍状态机（邀请中/已确认/已解散）"这一逻辑约束在数据库层的具体强制点。

---

## 3. 物理数据库设计：economy_db商品目录与购买记录

对应RGS-BAS-013§4.1。RGS-BAS-013原文曾将`PRODUCT_CATALOG`/`PURCHASE_RECORD`物理DDL留待未来集中设计；本文档在DTL详细设计工程体系下以分域方式完成该落地职责（说明见§6的权威边界），两表落位于既有`economy_db`（同RGS-DTL-001§3已挂载的库），本文档只新增表结构。

```sql
-- 商品目录表，对应RGS-BAS-013§4.1 PRODUCT_CATALOG
CREATE TABLE product_catalog (
    product_id           VARCHAR(64) PRIMARY KEY,
    entitlement_type       SMALLINT NOT NULL,   -- 0=道具 1=货币 2=权益，枚举同RGS-DTL-001§2.1"高频WHERE列用SMALLINT"原则
    entitlement_content       JSONB NOT NULL,     -- 结构随entitlement_type变体，同RGS-DTL-025 transaction_ledger.payload同类做法
    price                       BIGINT NOT NULL CHECK (price >= 0),
    available_from                TIMESTAMPTZ NOT NULL,
    available_until                 TIMESTAMPTZ,     -- NULL=长期上架
    daily_purchase_limit               INT,             -- NULL=无限购
    status                                SMALLINT NOT NULL DEFAULT 0,  -- 0=已上架 1=已下架
    version                                 INTEGER NOT NULL DEFAULT 0,   -- OCC，上下架/tick边界原子切换需要(RGS-BAS-013§4.1)
    CONSTRAINT chk_product_catalog_status CHECK (status IN (0, 1))
);
CREATE INDEX idx_product_catalog_available
    ON product_catalog (available_from, available_until) WHERE status = 0;
    -- 支撑"当前可购买商品"高频查询路径(购买发起时的商品状态校验，§5.1既定校验环节)

-- 购买记录表，对应RGS-BAS-013§4.1 PURCHASE_RECORD
CREATE TABLE purchase_records (
    request_id          UUID PRIMARY KEY,   -- 幂等键，客户端/上游生成，同RGS-DTL-001§3幂等去重表设计标准
    product_id            VARCHAR(64) NOT NULL REFERENCES product_catalog(product_id),
    character_id            UUID NOT NULL,   -- 逻辑引用player_db.characters，跨库不建物理FK
    payment_transaction_id    TEXT,             -- 支付渠道侧交易号，支付发起前为NULL
    status                       SMALLINT NOT NULL DEFAULT 0,
    -- 0=待支付 1=已支付待发货 2=已完成 3=已补偿，对应RGS-BAS-013§4.2购买时序状态迁移
    created_at                     TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT chk_purchase_records_status CHECK (status BETWEEN 0 AND 3)
);
CREATE INDEX idx_purchase_records_character_created
    ON purchase_records (character_id, created_at);
    -- 支撑§5.1每日限购计数查询("该character_id对该product_id当日已购买次数")与玩家自查购买历史
CREATE INDEX idx_purchase_records_product
    ON purchase_records (product_id);
```

`daily_purchase_limit`的计数不在`product_catalog`表内维护累计计数器列（避免"计数器列"这一常见并发热点设计），而是在§5.1购买校验时对`purchase_records`按`(character_id, product_id, created_at当日范围)`实时聚合查询——购买本身是低频操作（相对于RGS-DTL-001§3高频的`CommitTransaction`），实时聚合查询的开销可接受，且避免了计数器列自身的并发递减/重置维护复杂度（同RGS-DTL-025§2"案件表`signal_count`"设计取舍的镜像考量：`signal_count`因案件聚合是高频路径而选择计数器列，本表因限购校验是低频路径而选择实时聚合，两处结论不同但推理原则一致）。

---

## 4. 协议线格式：频道消息与在线状态

对应RGS-BAS-013§2.3（`PresenceEntry`字段级隐私过滤）与§3.1（`ChatMessage`字段扩展）。

```protobuf
// PresenceEntry: 大厅差分快照的在线状态条目，字段范围严格对应RGS-BAS-013§2.3表格
// 表中标注"否"的字段(current_scene_id/precise_coordinates)在本消息定义中不存在——
// 不是"存在但服务端不填充"，而是schema层面就没有这两个字段的位置，
// 从协议契约层面即杜绝"客户端本可解析出该字段只是服务端恰好没填"的误用风险
message PresenceEntry {
  string character_id       = 1;
  PresenceState presence_state = 2;  // ONLINE=0 OFFLINE=1 BUSY=2，复用FR-PL-006既有枚举语义
  string current_scene_type    = 3;  // "lobby"/"combat"/"dungeon"等类型字符串，非具体场景实例ID
}

// ChatMessage: 对应RGS-BAS-013§3.1字段扩展表(复用RGS-DTL-001既有Protobuf风格记述规则)
message ChatMessage {
  ChatChannel channel        = 1;  // WORLD=0 GUILD=1 TEAM=2 WHISPER=3
  string sender_character_id   = 2;
  string text                    = 3;
  int64 sent_at_ms                 = 4;
  string recipient_character_id      = 10;  // 编号置于10+区间(低频/可选字段)，仅WHISPER频道必填(FR-LBY-011)
}

message SendChatMessageResponse {
  ResultCode result_code = 1;  // 复用RGS-DTL-001§4.4通用ResultCode枚举，本文档新增以下取值
}
// ResultCode新增取值(在RGS-DTL-001§4.4既有枚举基础上追加，编号延续该文档编号纪律，不复用/变更既有编号):
//   MUTED = 8;               // 对应RGS-BAS-013§3.3禁言校验拒绝
//   ABUSE_REPEAT_MESSAGE = 9;  // 对应§3.4重复消息检测拒绝
//   ABUSE_BANNED_WORD = 10;     // 对应§3.4违禁词检测拒绝
```

---

## 5. 算法详细设计

### 5.1 频道路由与滥用检测主流程（落实RGS-BAS-013§3.2流程图+§3.4追加校验节点）

```rust
fn handle_chat_message(msg: &ChatMessage, ctx: &GdContext) -> Result<(), ChatError> {
    // 1. 禁言校验(§3.3)，查询AdminService.MuteChat写入的权威状态，GD不持有独立判定逻辑副本
    if ctx.query_mute_status(&msg.sender_character_id)?.is_muted() {
        return Err(ChatError::Muted);
    }

    // 2. ChatAbuseGuard基础规则(§3.4)，在MUTE节点之后、ROUTE节点之前，确定性逻辑，不依赖智能层
    if let Some(rule_hit) = chat_abuse_guard_check(msg, ctx)? {
        record_chat_abuse_signal(&msg.sender_character_id, msg.channel, rule_hit)?;  // 落§2 chat_abuse_signals
        return Err(match rule_hit {
            AbuseRule::RepeatMessage => ChatError::AbuseRepeatMessage,
            AbuseRule::BannedWord => ChatError::AbuseBannedWord,
        });
        // 本组件不触发任何AdminService处罚动作(RGS-BAS-013§3.4"信号而非判决"原则)，
        // 仅拒绝当次发送——与下方route_message完全互斥，命中滥用规则时不进入路由阶段
    }

    // 3. 频道路由(§3.2)
    match msg.channel {
        ChatChannel::Whisper => {
            // 黑名单校验(RGS-BAS-014§5.2既定生效点在本处接入): recipient若已拉黑sender则拒绝路由
            if ctx.is_blocked(&msg.recipient_character_id, &msg.sender_character_id)? {
                return Err(ChatError::RecipientBlocked);
            }
            deliver_direct(&msg.recipient_character_id, msg)?;  // 不经过任何频道全体成员广播路径
        }
        ChatChannel::World | ChatChannel::Guild | ChatChannel::Team => {
            fanout_with_backpressure(msg.channel, msg, ctx)?;  // NFR-LBY-002背压保护
        }
    }
    Ok(())
}
```

### 5.2 `ChatAbuseGuard`滚动窗口重复消息检测（落实RGS-BAS-013§3.4规则表第一行）

```rust
fn chat_abuse_guard_check(msg: &ChatMessage, ctx: &GdContext) -> Result<Option<AbuseRule>, ChatError> {
    // 规则1: 短时间内重复消息
    let window = ctx.abuse_config.repeat_window_secs;      // 提案默认10秒(可配置)，同RGS-DTL-025§5"提案默认值"处理方式
    let threshold = ctx.abuse_config.repeat_threshold_count; // 提案默认3条
    let recent = ctx.query_recent_messages(&msg.sender_character_id, window)?;
    let normalized_incoming = normalize_text(&msg.text);  // 归一化: 去除多余空白/大小写折叠，供近似匹配
    let similar_count = recent.iter()
        .filter(|m| edit_distance(&normalize_text(&m.text), &normalized_incoming) <= EDIT_DISTANCE_THRESHOLD)
        .count();
    if similar_count + 1 > threshold as usize {  // +1: 计入本次待发送消息自身
        return Ok(Some(AbuseRule::RepeatMessage));
    }

    // 规则2: 已知违禁词模式
    if ctx.banned_word_matcher.is_match(&msg.text) {
        // banned_word_matcher: 复用既有敏感词过滤基础设施(RGS-BAS-013§3.4已明确"若详细设计阶段确认尚无可复用
        // 的既有词库基础设施，则须登记TBD并走ARC-014评审，不得静默新建")——本文档假定复用对象已存在，
        // 若实现阶段发现并无可复用组件，须先补TBD而非在此处新建，本伪代码本身不构成"已确认可复用"的证明
        return Ok(Some(AbuseRule::BannedWord));
    }

    Ok(None)
}
```

**边界条件**：`similar_count + 1 > threshold`而非`similar_count >= threshold`，是为了使"threshold=3"的语义精确对应RGS-BAS-013§3.4文字表述"累计超过N条"——本次待发送消息本身也计入累计计数，避免"第3条才刚好允许通过、第4条才拒绝"这类差一错误。

### 5.3 购买确定请求事务边界（落实RGS-BAS-013§4.2时序图，复用RGS-DTL-001§3.2既定事务模式）

```sql
BEGIN;
  -- 1. 商品状态/有效期/限购校验（§5.1既定校验环节，§3限购按低频实时聚合查询完成，本处不重复列出SELECT）
  -- 2. 支付成功后的权益发放：与RGS-DTL-001§3.2 CommitTransaction完全相同的OCC+幂等语义，
  --    本文档不重新设计，仅声明request_id延续购买请求同一标识贯穿全链路(同ARC-009关联ID透传原则)
  UPDATE wallets SET balance = balance - $price, version = version + 1
    WHERE character_id = $cid AND version = $expected_version AND balance >= $price;
  -- 影响行数为0: 可能是OCC冲突或余额不足，需与expected_version重新比对区分两种拒绝原因(供上层返回精确ResultCode)
  UPDATE purchase_records SET status = 2, payment_transaction_id = $txn_id WHERE request_id = $request_id;
  INSERT INTO transaction_ledger (request_id, character_id, operation, payload, expected_version, result_version)
    VALUES ($request_id, $cid, 3 /* consume_currency */, $payload, $expected_version, $expected_version + 1);
    -- 复用RGS-DTL-001§3.1 transaction_ledger表与§3.2幂等强制点，购买不新增独立流水表
COMMIT;
```

### 5.4 活动奖励发放幂等键（落实RGS-BAS-013§5.1时序图）

```rust
fn grant_activity_reward(player_id: PlayerId, activity_id: ActivityId, milestone: MilestoneId) -> Result<GrantResult, GrantError> {
    let request_id = deterministic_hash(&[player_id.as_bytes(), activity_id.as_bytes(), milestone.as_bytes()]);
    // request_id = hash(player_id + activity_id + milestone)，RGS-BAS-013§5.1既定构造方式，
    // 同一玩家对同一活动同一里程碑的多次领取请求恒定映射到同一request_id，
    // 天然复用RGS-DTL-001§3.2幂等强制点(transaction_ledger.request_id唯一约束)防重复领取(FR-LBY-052)，
    // 不需要额外的"活动领取记录表"承载幂等语义
    commit_transaction(CommitTransactionRequest {
        request_id,
        character_id: player_id.into(),
        operation: Operation::GrantItem(/* 活动奖励规格 */),
        session_epoch: current_session_epoch(player_id)?,  // 由宿主(RT场景Actor或大厅Actor)注入，同RGS-BAS-013§5.1既定
        expected_version: current_wallet_version(player_id)?,
    })
    // 与普通道具发放走同一路径，无特殊通道(RGS-BAS-013§5.1既定"FR-LBY-051")
}
```

---

## 6. 本文档的覆盖范围与后续计划

本文档覆盖：`social_db`新增`teams`/`team_members`/`chat_abuse_signals`三表物理DDL、`economy_db`新增`product_catalog`/`purchase_records`两表物理DDL、`PresenceEntry`/`ChatMessage`的具体协议线格式（含隐私字段的schema层面排除设计）、频道路由主流程与`ChatAbuseGuard`滚动窗口检测的完整伪代码、购买确定请求事务边界与活动奖励幂等键构造的具体实现。

**分域物理 DDL 的权威边界**：RGS-BAS-013原文将物理DDL留待未来集中设计，该表述写于本仓库尚未确立DTL详细设计工程独立分支之前（RGS-DTL-001是本仓库第一份DTL文档，晚于RGS-BAS-013的制定）。本文档在DTL体系确立后完成该职责，`product_catalog`/`purchase_records`/`teams`/`team_members`/`chat_abuse_signals`的物理DDL以本文档为唯一权威；不再新建集中正文。跨域表结构扩展须按RGS-BAS-016§3.1回写原表权威文档，避免并行维护冲突版本。

本版本明确不覆盖、留待后续：

- 客户端UI渲染细节（活动入口图标/跳转参数展示、聊天界面交互）——属客户端/前端自身设计范围。
- 活动插件沙箱脚本本身的执行引擎设计——属RGS-BAS-005/009既有范围，本文档仅覆盖插件与经济服务交互时的确定请求事务边界。
- `daily_purchase_limit`低频实时聚合查询在极端流量下（如限时抢购活动）的性能验证——本文档提出的设计取舍（不用计数器列）基于"购买是低频路径"假设，该假设在秒杀类活动场景下可能不成立，需按实测数据判断是否需要为特定高热度商品追加缓存计数层，留待PH-4/PH-8负载试验后依RGS-DTL-012既定负载测试基础设施验证。
- `ChatAbuseGuard`§5.2中`EDIT_DISTANCE_THRESHOLD`/`repeat_window_secs`/`repeat_threshold_count`三项参数的最终校准值——均为初始提案，需按实际举报/误判数据校准，非本文档最终交付值。
- 违禁词库基础设施若确认尚不存在，其新建评审（ARC-014）与词库内容本身——按RGS-BAS-013§3.4既定原则，本文档不代为新建。

---

## 追溯性

| 需求/设计来源 | 本文档章节 |
|---|---|
| RGS-BAS-013§2.1〜2.2 大厅作为场景Actor与队伍持久化 | §2 |
| RGS-BAS-013§2.3 在线状态字段级隐私过滤 | §4（`PresenceEntry`） |
| RGS-BAS-013§3.1 ChatMessage字段扩展 | §4（`ChatMessage`） |
| RGS-BAS-013§3.2 路由设计 | §5.1 |
| RGS-BAS-013§3.3 禁言校验 | §5.1 |
| RGS-BAS-013§3.4 轻量级自动化滥用检测 | §2（`chat_abuse_signals`）、§5.2 |
| RGS-BAS-013§4.1 商品目录数据模型 | §3 |
| RGS-BAS-013§4.2 购买时序 | §5.3 |
| RGS-BAS-013§5.1 活动奖励发放时序 | §5.4 |
| RGS-BAS-013§5.2 经济类活动单点判定 | §5.4（`grant_activity_reward`不新增旁路的落实） |
| RGS-DTL-001§3.2 确定请求API物理执行语义 | §5.3、§5.4（复用同一事务/幂等模式） |
| RGS-DTL-025§5（提案默认值处理方式的既定先例） | §5.2参数提案 |
| RGS-BAS-014§5.2 黑名单生效点 | §5.1（whisper路由接入黑名单校验） |
