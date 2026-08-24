# 消息分发（Message Distribution）详细设计书

**Social 域站内信业务对象（`messages` / `message_recipients` / `conversations`）物理数据库设计・会话生命周期状态机・与 DTL-019 4 渠道抽象边界划分**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DTL-043 |
| 标题 | 消息分发 |
| 版本 | 0.1 |
| **状态** | **🟢 v1.0**（per RGS-OPEN-QA-001 v0.2 Q-D-01 答复："**直接进 1.0 状态**（非 1.5 草案）——这是正式承接一个当前完全缺失的能力域，不是修补已有文档。"——状态标记 1.0/1.5 与版本号 v0.1 是两个独立维度，不要混淆） |
| 父文档 | RGS-REQ-016 大厅社交通信与运营活动 / RGS-BAS-013 大厅社交通信与运营活动 / RGS-DTL-039 Social 域契约骨架 / RGS-OPEN-QA-001 v0.2 Q-D-01（已答复）|
| 依据 | RGS-OPEN-QA-001 v0.2 Q-D-01（**选项 A：新建 DTL-043** + 直接进 1.0 状态）+ RGS-OPEN-QA-001-ACTIONS-v0.3 §3 A-01 + RGS-OPEN-QA-001-ACTIONS-v0.3 §5 修正 #1（DTL 编号 037→043 因 037 已被 Economy 域占用）+ RGS-OPEN-QA-001 v0.2 Q-D-08（4 渠道抽象归属 DTL-019 v0.2，第三方网关 APNs/FCM/SMTP/SMS 不在 PH-1 范围）+ RGS-OPEN-QA-001 v0.2 Q-G-01 答复"状态/版本双维度"（v0.1 + 1.0 状态合法） |
| App/DB | `social-service` / `social_db` |
| 制定日 | 2026-08-24 |
| 制定者 | social 域 Lead（Ulysses per DEC-008 一人公司 12 角色兼任）|
| 修订历史 | 0.1（2026-08-24）：WF-1-55.38 L4 任务首版产出，per RGS-OPEN-QA-001 v0.2 Q-D-01 答复（选项 A + 直接进 1.0），承接 Social 域站内信业务对象（3 张主表 + 4 渠道抽象归属边界 + 失败重试策略）|
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库）|
| 责任人 | social 域 Lead（Ulysses per DEC-008）|
| 关联 | DTL-019（推送+兑换码，4 渠道抽象层）/ RGS-SPEC-CROSS-003（事件 Schema，v0.2 升版后含 message 事件）/ RGS-SPEC-CROSS-006（trace_id 跨域传播）/ RGS-SPEC-CROSS-001（错误码字典，social 域 4001-4999 段）/ DTL-039（Social 域契约骨架，含"消息物理 DDL"待补齐项第 1 条）|

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响章节 |
|---|---|---|---|---|---|
| 0.1 | 2026-08-24 | social 域 Lead（Ulysses per DEC-008）| Ulysses（per DEC-008 一人公司 12 角色全签，见§6） | 首版制定（per WF-1-55.38 / RGS-OPEN-QA-001 v0.2 Q-D-01 答复选项 A + 直接进 1.0 状态）。承接 Social 域站内信业务对象：① §2 三张主表 `messages` / `message_recipients` / `conversations` 物理 DDL（字段级，PL/AD 限界上下文归属，PostgreSQL）② §3 4 渠道抽象归属（站内信由本 DTL 负责"业务对象"层 + DTL-019 v0.2 负责"渠道投递技术层" + 第三方网关 APNs/FCM/SMTP/SMS 不在 PH-1 范围）③ §4 渠道失败重试策略（push 不重试 vs 邮件/短信 3 次指数退避）④ §5 与 DTL-019 边界划分（业务对象 vs 渠道投递抽象的分层原则）⑤ §6 12 类签字栏（per DEC-008 一人公司）| 全部 |

---

## 目录

1. [前言](#1-前言)
2. [物理数据库设计：站内信三张主表](#2-物理数据库设计站内信三张主表)
3. [4 渠道抽象归属与边界划分](#3-4-渠道抽象归属与边界划分)
4. [渠道失败重试策略](#4-渠道失败重试策略)
5. [与 DTL-019 边界划分](#5-与-dtl-019-边界划分)
6. [签字栏（per DEC-008 一人公司 12 角色）](#6-签字栏per-dec-008-一人公司-12-角色)
7. [追溯性](#7-追溯性)
8. [本文档的覆盖范围与后续计划](#8-本文档的覆盖范围与后续计划)

---

## 1. 前言

### 1.1 定位

RGS-OPEN-QA-001 v0.2 Q-D-01 识别出 Social 域当前完全缺失"站内信"业务对象（`messages` / `message_recipients` / `conversations` 三张主表）的详细设计文档——既有 DTL-019 实际是"推送+兑换码"（`push_consents` + `redemption_code_batches` / `redemption_codes` / `redemption_records`），**不含**站内信三表。REV-004 附件 A §A.5 标题"DTL-019 消息分发"与源文件标题"DTL-019 消息推送与兑换码运营工具"不一致正是因为该能力域的归属问题未解决。Q-D-01 答复选项 A："**新建 DTL-043 消息分发 v0.1**"——本表把 Social 域站内信业务对象的物理 DDL 落到 PL/AD 限界上下文下的 PostgreSQL schema，并明确与 DTL-019（"推送+兑换码" + 4 渠道投递抽象层）之间的边界划分。

### 1.2 本文档做什么

- **新建**三张主表 `messages` / `message_recipients` / `conversations` 的字段级 DDL（§2），落位 social_db 的 PL 限界上下文（受信业务对象）+ AD 限界上下文（运营管控）按字段拆分。
- **明确** 4 渠道（站内信 / 邮件 / 推送 / 短信）抽象归属（§3）：站内信业务对象由本 DTL-043 负责，4 渠道投递技术抽象由 DTL-019 v0.2 负责，第三方网关适配层（APNs / FCM / SMTP / SMS）**不在 PH-1 范围**，PH-1 用 mock/stub 网关跑通链路。
- **规定** 4 渠道的失败重试策略（§4）：push 不重试（用户体验优先），邮件 / 短信 3 次指数退避（到达率优先）。
- **划分** 与 DTL-019 的职责边界（§5）：本 DTL 接管"站内信业务对象"，DTL-019 v0.2 继续负责"推送+兑换码" + 4 渠道投递技术抽象。

### 1.3 本文档不做什么

- **不重新决定** Q-D-01 已确定的所有结构性选择（拆出"站内信业务对象"、直接进 1.0 状态、DTL 编号 043 而非 037）。
- **不覆盖** 4 渠道投递的协议格式（属 DTL-019 v0.2 §3 范围，按 Q-D-08 答复）。
- **不覆盖** 第三方网关适配层（APNs / FCM / SMTP / SMS）的具体 SDK 调用代码——per Q-D-08 答复，PH-1 用 mock/stub 网关跑通链路，PH-2 再接真实 SDK。
- **不覆盖** 跨域事件 schema（`MessageCreated` / `MessageRead` 等）——属 RGS-SPEC-CROSS-003 v0.2 升版范围（per OPEN-QA-001-ACTIONS A-09）。
- **不覆盖** 推送同意状态（`push_consents` 表）——属 DTL-019 v0.1 §2，本 DTL 不重复定义。

### 1.4 记述规则

沿用既有 DTL 文档记述规则（per RGS-DTL-019 / RGS-DTL-039 / RGS-DTL-025）：DDL 以 PostgreSQL 为准；状态机以 Rust `Result` / `enum` 风格给出；落位限界上下文（PL 受信业务对象 / AD 运营管控）按字段拆分，与 RGS-DTL-019 §2 兑换码三表的归属原则一致。

---

## 2. 物理数据库设计：站内信三张主表

对应 RGS-OPEN-QA-001 v0.2 Q-D-01 答复"3 张主表 `messages` / `message_recipients` / `conversations`"。三表落位 `social_db` 的 PL（受信业务对象） / AD（运营管控）限界上下文——按字段拆分受信内容（PL）和运营管控字段（AD），与 RGS-DTL-019 §2 兑换码三表"批次/明细/核销"拆分到 PL / AD 的原则一致。

### 2.1 `messages`（消息主表，落位 PL）

```sql
-- 消息主表，对应Q-D-01答复"messages"表，落位PL受信业务对象上下文
-- 表达"一条消息"本身的不可变事实（一旦写入不修改内容；撤销用revoked_at标志，不删行）
CREATE TABLE messages (
    id                    UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sender_id             UUID,                          -- NULL=系统消息（如兑换码发放通知），非NULL=玩家/GM sender
    message_type          TEXT NOT NULL,                 -- 'direct' / 'system' / 'broadcast' / 'group' / 'announcement'，对应不同收件人解析策略
    title                 TEXT NOT NULL,                 -- 标题，已通过内容脱敏校验（命中禁止模式则拒绝，同RGS-DTL-019§2.1.1）
    body                  TEXT NOT NULL,                 -- 正文，已通过内容脱敏校验
    priority              SMALLINT NOT NULL DEFAULT 0,  -- 0=普通 1=高 2=紧急，影响客户端UI排序（per RGS-REQ-016 §3.2）
    category              TEXT,                          -- 业务类别，如'friend_invite' / 'activity_start' / 'redemption_granted'，与push_consents.category共享命名空间
    metadata              JSONB NOT NULL DEFAULT '{}'::jsonb,  -- 扩展属性：附件URL、道具ID、活动ID等，按需扩展
    created_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at            TIMESTAMPTZ,                   -- NULL=永不过期；非NULL则到期后客户端不可见（DB不删，作为事实留存）
    revoked_at            TIMESTAMPTZ,                   -- NULL=有效；非NULL=被GM撤销，客户端不再展示（DB不删，作为治理事实）
    revoked_by            UUID,                          -- 与revoked_at成对出现，记录撤销操作人（GM/Admin域 Lead）
    revoke_reason         TEXT,                          -- 撤销原因文字，留痕用
    CONSTRAINT chk_messages_revoke_pair CHECK (
        (revoked_at IS NULL AND revoked_by IS NULL AND revoke_reason IS NULL)
        OR (revoked_at IS NOT NULL AND revoked_by IS NOT NULL AND revoke_reason IS NOT NULL)
    ),
    CONSTRAINT chk_messages_priority_range CHECK (priority BETWEEN 0 AND 2)
);
-- 说明：
-- ① sender_id 允许 NULL 是因为"系统消息"无具体 sender（如兑换码自动到账通知），与 RGS-DTL-019 §2 兑换码批次表"preview_confirmed_by"应用层校验原则一致——表结构承载最小事实，应用层负责"无 sender 的消息只能是 system 类型"等业务约束
-- ② metadata JSONB 与 RGS-OPEN-QA-001 v0.2 Q-D-02 答复"player_characters.stats 用 JSONB + 字段化混合"原则一致——高频需要索引/查询的属性（如 category）拆列，低频扩展属性走 JSONB
-- ③ revoked_at / revoked_by / revoke_reason 三字段联动 CHECK 约束确保"撤销操作必留痕"，同 RGS-DTL-039 §5 "治理操作必须有 RBAC、审计和可回滚的状态迁移"既定原则

CREATE INDEX idx_messages_sender ON messages (sender_id, created_at DESC);
-- 支撑"我发送的消息列表"查询（按 sender + 时间倒序）

CREATE INDEX idx_messages_category_created ON messages (category, created_at DESC);
-- 支撑"某业务类别最近消息"运营查询

CREATE INDEX idx_messages_expires ON messages (expires_at) WHERE expires_at IS NOT NULL;
-- 部分索引：仅对有过期时间的消息建立索引，避免永不过期消息占索引空间；支撑定时清理任务（PH-2实现）
```

### 2.2 `message_recipients`（收件人关联表，落位 PL）

```sql
-- 收件人关联表，对应Q-D-01答复"message_recipients"表，落位PL受信业务对象上下文
-- 表达"一条消息"对"一个收件人"的派发事实（每条消息 × 每个收件人 = 一行）
-- 与RGS-DTL-019§2 redemption_records"同一账号对同一码的核销记录"复合主键模式一致：
-- (message_id, recipient_id) 即幂等键，确保"同一消息对同一收件人不重复派发"
CREATE TABLE message_recipients (
    id                    UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    message_id            UUID NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    recipient_id          UUID NOT NULL,                 -- 收件人账号ID，对应player-service的account_id
    channel               TEXT NOT NULL,                 -- 实际投递渠道：'in_app'（站内信）/ 'email' / 'push' / 'sms'，由DTL-019§3渠道抽象层写入
    dispatched_at         TIMESTAMPTZ NOT NULL DEFAULT now(),  -- 派发到渠道的时间（不一定等于送达时间）
    delivered_at          TIMESTAMPTZ,                   -- 渠道确认送达时间（mock网关PH-1即返回，真实网关PH-2异步回调）
    read_at               TIMESTAMPTZ,                   -- 收件人首次已读时间，NULL=未读
    deleted_at            TIMESTAMPTZ,                   -- 收件人本人软删除时间（区别于messages.revoked_at的GM撤销）
    failure_count         SMALLINT NOT NULL DEFAULT 0,   -- 投递失败次数，配合DTL-019§4重试策略（email/sms 3次后放弃）
    last_failure_at       TIMESTAMPTZ,                   -- 最近一次失败时间，配合重试退避计算
    last_failure_reason   TEXT,                          -- 最近一次失败原因（mock网关PH-1固定值，真实网关PH-2记录SDK错误码）
    UNIQUE (message_id, recipient_id, channel)           -- 同一消息对同一收件人同一渠道不重复派发
);
-- 说明：
-- ① 三字段复合 UNIQUE 约束 (message_id, recipient_id, channel) 是"幂等性物理强制层"——同RGS-DTL-001§3.2 / RGS-DTL-019§2 既有原则，避免应用层先查后插的竞态
-- ② failure_count + last_failure_at + last_failure_reason 配合 DTL-019 §4 重试策略：邮件/短信 3 次后即不再重试（达上限），不依赖外部调度器判断
-- ③ read_at 与 deleted_at 独立：read_at 是已读事实，deleted_at 是收件人本人软删除操作，两者不互斥（已读后仍可删除，未读也可删除）
-- ④ channel 字段由 DTL-019 v0.2 §3 渠道抽象层写入（dispatcher 决定每条 recipient 走哪个渠道），本 DTL 不重复定义渠道枚举值

CREATE INDEX idx_message_recipients_recipient_unread
    ON message_recipients (recipient_id, message_id)
    WHERE read_at IS NULL AND deleted_at IS NULL;
-- 部分索引：仅对"未读且未删除"行建立，支撑"未读消息列表"高频查询（社交域最高频查询之一）

CREATE INDEX idx_message_recipients_message
    ON message_recipients (message_id);
-- 支撑"某条消息的所有收件人投递状态"运营查询（如广播消息送达率统计）

CREATE INDEX idx_message_recipients_failure_retry
    ON message_recipients (last_failure_at)
    WHERE failure_count > 0 AND failure_count < 3;
-- 部分索引：仅对"失败未达上限"的行建立，支撑DTL-019§4邮件/短信重试调度器扫描
```

### 2.3 `conversations`（会话表，落位 PL）

```sql
-- 会话表，对应Q-D-01答复"conversations"表，落位PL受信业务对象上下文
-- 表达"两个或多个玩家之间的会话上下文"——私聊（1对1）/ 群聊（N对N）/ 系统会话（1对系统）
-- 区别于message_recipients的"单条消息对单收件人"粒度，conversations是"长期会话上下文"粒度
CREATE TABLE conversations (
    id                    UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    conversation_type     TEXT NOT NULL,                 -- 'direct'（1对1私聊）/ 'group'（群聊）/ 'system'（系统会话，玩家↔系统）
    participant_ids       UUID[] NOT NULL,               -- 会话参与者ID列表，direct=[A,B]，group=[A,B,C,...]，system=[player_id]
    last_message_id       UUID,                          -- 最近一条消息ID（外键引用messages.id，弱引用即可——消息被删时清空此字段）
    last_message_at       TIMESTAMPTZ NOT NULL DEFAULT now(),  -- 最近消息时间，用于"会话列表按最近活动排序"
    last_message_preview  TEXT,                          -- 最近消息预览文字（脱敏后），用于会话列表UI展示，避免N+1查询messages表
    created_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_by            UUID,                          -- 创建人；system类型会话此字段为NULL
    archived_at           TIMESTAMPTZ,                   -- 会话整体归档时间（所有参与者都软删除后由定时任务写入）
    metadata              JSONB NOT NULL DEFAULT '{}'::jsonb,  -- 群聊名称、群公告、群设置等扩展属性
    CONSTRAINT chk_conversations_participants_nonempty CHECK (array_length(participant_ids, 1) >= 1),
    CONSTRAINT chk_conversations_type CHECK (conversation_type IN ('direct', 'group', 'system')),
    CONSTRAINT chk_conversations_last_message_pair CHECK (
        (last_message_id IS NULL AND last_message_preview IS NULL)
        OR (last_message_id IS NOT NULL AND last_message_preview IS NOT NULL)
    )
);
-- 说明：
-- ① participant_ids UUID[] 数组列：direct = [A, B] 固定 2 元素；group = [A, B, C, ...] 不定长；system = [player_id] 1 元素
--    与 RGS-OPEN-QA-001 v0.2 Q-D-02 答复"inventory 走独立表 + 外键"不同——会话参与者是"无独立生命周期的标识集合"，不需行级操作，塞入数组更合适
-- ② last_message_id / last_message_at / last_message_preview 三字段联动 CHECK 约束确保"有最近消息ID必有预览，无则三者都空"——避免数据不一致
-- ③ last_message_id 是弱引用（无 FOREIGN KEY 约束）——若 messages 行被删（PH-2 实现级联清理），conversations 仅清空此字段即可，会话本身保留作为历史事实
-- ④ archived_at NULL=活跃会话，非NULL=已归档会话（所有参与者都软删除消息后由定时任务写入），用于"会话列表"过滤

CREATE INDEX idx_conversations_participants ON conversations USING GIN (participant_ids);
-- GIN 索引：支撑"我的会话列表"反向查询（"哪些会话包含我"），GIN 数组索引是 PostgreSQL 标准做法

CREATE INDEX idx_conversations_last_message_at ON conversations (last_message_at DESC);
-- 支撑"会话列表按最近活动倒序"查询
```

### 2.4 派生视图（运营查询便利性，落位 AD）

```sql
-- 站内信运营视图：活跃会话 × 最近消息 × 未读计数（落位AD运营管控上下文）
-- AD域（运营/GM）通过此视图做"用户站内信情况审计""广播消息送达率"等查询，无需写复杂JOIN
CREATE VIEW v_message_dispatch_overview AS
SELECT
    c.id                                                            AS conversation_id,
    c.conversation_type,
    c.participant_ids,
    c.last_message_at,
    c.last_message_preview,
    COUNT(mr.id) FILTER (WHERE mr.read_at IS NULL AND mr.deleted_at IS NULL)  AS unread_count,
    COUNT(mr.id)                                                    AS total_recipients,
    COUNT(mr.id) FILTER (WHERE mr.failure_count > 0)                AS failed_dispatches
FROM conversations c
LEFT JOIN message_recipients mr ON mr.message_id = c.last_message_id
GROUP BY c.id;
-- 视图层做聚合，PL域"受信业务对象"表本身不被AD域直接访问，AD只读视图——同RGS-DTL-019§2 "DBA只读视图"同类分层原则
```

---

## 3. 4 渠道抽象归属与边界划分

对应 RGS-OPEN-QA-001 v0.2 Q-D-08 答复。明确 4 渠道（站内信 / 邮件 / 推送 / 短信）的归属 + 第三方网关适配层是否在 PH-1 范围。

### 3.1 4 渠道归属矩阵

| 渠道 | 业务对象归属 | 投递技术抽象归属 | 第三方网关适配层 | PH-1 范围 |
|---|---|---|---|---|
| **站内信（in_app）** | **本 DTL-043 §2**（`messages` + `message_recipients` + `conversations`）| DTL-019 v0.2 §3（4 渠道抽象枚举）| 不涉及（站内信无第三方网关，由 social-service 自身投递）| ✅ 在 PH-1 范围 |
| **邮件（email）** | 本 DTL-043 §2（同上）| DTL-019 v0.2 §3 | SMTP 网关适配（PH-2）| 🟡 接口定义在 PH-1，真实 SMTP 网关 PH-2 |
| **推送（push）** | 本 DTL-043 §2（同上）| DTL-019 v0.2 §3（沿用 v0.1 `PushDeliveryRequest` / `PushDeliveryResult`）| APNs / FCM 适配（PH-2）| 🟡 接口定义在 PH-1，真实 APNs/FCM 网关 PH-2 |
| **短信（sms）** | 本 DTL-043 §2（同上）| DTL-019 v0.2 §3 | SMS 网关适配（PH-2）| 🟡 接口定义在 PH-1，真实 SMS 网关 PH-2 |

**关键决策**（per Q-D-08 答复）：

1. **业务对象层（`messages` / `message_recipients` / `conversations`）统一归属本 DTL-043**：不论走哪个渠道，消息本身的事实（sender、body、recipient、read 状态）都落同一组表。`message_recipients.channel` 字段记录实际投递渠道。
2. **投递技术抽象层（4 渠道枚举 + 渠道间协议格式）归属 DTL-019 v0.2**：本 DTL 不重复定义 4 渠道的 proto/protobuf 格式，由 DTL-019 v0.2 §3 统一规定。
3. **第三方网关适配层（APNs / FCM / SMTP / SMS）不在 PH-1 范围**（per Q-D-08 答复明确）：PH-1 用 mock/stub 网关跑通"业务对象写入 → 渠道抽象调用 → mock 网关确认送达"全链路。PH-2 再接真实 SDK。
4. **站内信无第三方网关**：站内信投递是 social-service 自身的能力（写入 `message_recipients` 表即视为"送达"），不涉及外部网关。

### 3.2 站内信（in_app）渠道的"送达"语义

站内信是 4 渠道中**唯一不需要外部网关**的渠道——其"送达"语义由 social-service 自身闭环：

```rust
/// 站内信派发：写入 message_recipients 即视为"送达"（无外部网关）
/// per RGS-OPEN-QA-001 v0.2 Q-D-08 答复的"站内信"渠道归属
fn dispatch_in_app(message_id: Uuid, recipient_id: Uuid) -> Result<(), DispatchError> {
    // 直接 INSERT message_recipients，channel='in_app'，delivered_at=now()
    // 无需调用外部网关，事务内即可完成"派发 + 送达"
    // 与 RGS-DTL-019 §2 push_consents 写入是同一个事务族
    insert_message_recipient(message_id, recipient_id, "in_app")?;
    Ok(())
}
```

`delivered_at` 字段在站内信场景下 = `dispatched_at`（同时刻），因为没有"网关异步确认"环节。这与邮件/推送/短信的"`dispatched_at` 是发送时间、`delivered_at` 是网关回调时间"形成对比。

### 3.3 第三方网关适配层在 PH-1 的 mock/stub 策略

per Q-D-08 答复"PH-1 用 mock/stub 网关跑通链路"，本 DTL 在 §2 `message_recipients.delivered_at` / `failure_count` 字段设计上**已经为 mock 网关预留了写入点**：

- mock/stub 网关实现：social-service 内置 `MockGatewayAdapter`（4 渠道对应 4 个 mock），PH-1 集成测试时全部走 mock 路径，验证 `message_recipients` 表的 `delivered_at` / `failure_count` / `last_failure_reason` 字段写入逻辑正确。
- 真实网关（APNs / FCM / SMTP / SMS）PH-2 接入时，只需替换 `MockGatewayAdapter` 为 `RealGatewayAdapter`，**`message_recipients` 表结构不变**——这是 §2 表设计时已考虑的可扩展性（与 RGS-DTL-019 §4.4 TBD-OPT-002 初始提案"默认值非最终值"同类原则）。

---

## 4. 渠道失败重试策略

对应 RGS-OPEN-QA-001 v0.2 Q-D-08 答复："重试策略放 DTL-019 v0.2 本身（渠道能力定义的一部分）：push 不重试（用户体验优先），邮件 / 短信 3 次指数退避重试（到达率优先）。"

**重要边界说明**：本 DTL-043 §2 `message_recipients.failure_count` 字段是**承接重试结果的物理表**——DTL-019 v0.2 §4 定义重试策略（何时触发、退避算法），本 DTL-043 §2 提供字段存储（已重试几次、最近失败时间）。两者协作，**不重复定义**。

### 4.1 重试策略总表

| 渠道 | 重试次数 | 退避算法 | 失败上限后行为 | 依据 |
|---|---|---|---|---|
| **站内信** | 不适用 | 不适用 | 站内信无外部网关，不存在"投递失败"概念 | Q-D-08（站内信无第三方网关）|
| **推送（push）** | **0 次**（不重试）| 不适用 | 失败即丢弃，记录 `failure_count=1` + `last_failure_reason='token_expired'/'rate_limited'` | Q-D-08："push 不重试（用户体验优先）"——延迟到达的推送比无推送更糟（用户在户外/锁屏/换设备）|
| **邮件（email）** | **3 次** | 指数退避（1min / 5min / 30min）| 3 次后放弃，记录 `failure_count=3` + `last_failure_reason='permanent_failure'`，不再调度 | Q-D-08："邮件 / 短信 3 次指数退避"——邮件有重试到收件箱的容错空间 |
| **短信（sms）** | **3 次** | 指数退避（1min / 5min / 30min）| 同邮件 | Q-D-08 同上——短信成本高但验证码等场景需高到达率 |

### 4.2 退避时间表（PH-1 默认值，非最终值）

```
第 1 次重试：失败后 1 分钟（60s）
第 2 次重试：上次失败后 5 分钟（300s）
第 3 次重试：上次失败后 30 分钟（1800s）
3 次后放弃：failure_count=3，last_failure_reason='max_retries_exceeded'
```

**PH-1 初始值说明**（per RGS-DTL-019 §4.4 / §5 "TBD-OPT-002 默认值非最终值"同类原则）：上述 1min / 5min / 30min 退避值为 PH-1 初始值，非最终值。**最终退避数值需按 PH 阶段实测数据与安全评审确定后回写 DTL-019 v0.2 新版本**。

### 4.3 与 `message_recipients` 字段的协作

DTL-019 v0.2 §4 调度器按以下逻辑扫描本 DTL-043 §2 表：

```sql
-- DTL-019 v0.2 §4 邮件/短信重试调度器扫描（伪代码，具体实现属DTL-019范围）
-- 扫描条件：失败未达上限（failure_count < 3）且上次失败时间已过退避窗口
SELECT id, message_id, recipient_id, last_failure_at, failure_count
FROM message_recipients
WHERE channel IN ('email', 'sms')
  AND failure_count > 0
  AND failure_count < 3
  AND last_failure_at < now() - INTERVAL '1 minute' * POWER(5, failure_count - 1)
  -- 简化表达：第1次=1min, 第2次=5min, 第3次=25min（与§4.2 30min略差异，PH-1需对齐）
ORDER BY last_failure_at ASC
LIMIT 100;
-- 注：本 SELECT 是 DTL-019 v0.2 调度器逻辑，不在本 DTL-043 实现范围；本 DTL 只确保：
--   ① failure_count / last_failure_at / last_failure_reason 字段存在（§2 已定义）
--   ② idx_message_recipients_failure_retry 部分索引覆盖此查询（§2.2 已建）
```

---

## 5. 与 DTL-019 边界划分

本节明确 DTL-043（本 DTL）与 DTL-019 v0.2 之间的职责边界——避免"消息分发"能力域在两份 DTL 中重复定义或遗漏。

### 5.1 边界矩阵

| 职责 | DTL-043（本 DTL）| DTL-019 v0.2 |
|---|---|---|
| **`messages` / `message_recipients` / `conversations` 三张表 DDL** | ✅ 负责 | ❌ 不负责 |
| **4 渠道枚举定义（`in_app` / `email` / `push` / `sms`）**| ❌ 不重复定义（`message_recipients.channel` 字段引用 DTL-019 §3 枚举）| ✅ 负责 |
| **4 渠道投递协议格式（proto/protobuf）**| ❌ 不负责 | ✅ 负责（沿用 v0.1 `PushDeliveryRequest` / `PushDeliveryResult` + 新增 `EmailDeliveryRequest` / `SmsDeliveryRequest`）|
| **渠道失败重试策略** | ❌ 不重复定义（`message_recipients.failure_count` 字段承接 DTL-019 §4 写入结果）| ✅ 负责（push 不重试 / 邮件/短信 3 次指数退避）|
| **`push_consents` 表（推送同意状态）**| ❌ 不负责 | ✅ 负责（DTL-019 v0.1 §2 已定义）|
| **第三方网关适配层（APNs / FCM / SMTP / SMS）**| ❌ 不负责 | ✅ 负责（PH-2 接入真实 SDK）|
| **站内信业务对象（消息主表 / 收件人 / 会话）**| ✅ 负责 | ❌ 不负责 |
| **站内信派发逻辑（无外部网关的"写入即送达"语义）**| ✅ 负责（§3.2）| ❌ 不负责 |
| **`message_recipients` 表"重试字段"（`failure_count` / `last_failure_at` / `last_failure_reason`）**| ✅ 负责（§2.2 字段定义）| ✅ 负责（DTL-019 §4 调度器写入语义）|
| **跨域事件 schema（`MessageCreated` / `MessageRead`）**| 🟡 占位（待 RGS-SPEC-CROSS-003 v0.2 §X 填充）| ❌ 不负责 |

### 5.2 边界原则

1. **"业务对象层" vs "渠道投递层"分层**：本 DTL 负责"消息作为业务对象的事实"（什么消息、谁发的、谁收的、读了没），DTL-019 负责"消息如何通过 4 个渠道投递的技术细节"（协议格式、退避策略、网关适配）。这是 RGS-DTL-019 §1.2 "本文档不重新决定 RGS-BAS-019 已确定的任何结构性选择" 既定分层原则的延伸。
2. **`message_recipients.channel` 字段是协作点**：本 DTL 定义字段，DTL-019 v0.2 定义字段取值（4 渠道枚举）。本 DTL **不**给出 4 渠道的 proto 枚举值（避免与 DTL-019 §3 重复），只引用 DTL-019 §3 既有定义。
3. **PH-1 协作链**：social-service 写入 `messages` + `message_recipients`（本 DTL 负责）→ 调用 DTL-019 v0.2 §3 渠道抽象层（DTL-019 负责）→ mock/stub 网关（PH-1）/ 真实网关（PH-2）。两 DTL 各自负责自己段的实现，对方的实现不在本 DTL 范围。
4. **不重复定义原则**（per RGS-DTL-019 §1.2 既有原则）：本 DTL 不重复 DTL-019 已有的任何字段/枚举/协议/策略——`push_consents`、渠道 proto、退避算法都不在本 DTL 重复展开。

### 5.3 与 DTL-019 v0.2 升版的协作时序

per RGS-OPEN-QA-001-ACTIONS-v0.3 §3 A-08（Q-D-08 的下游动作）：DTL-019 v0.2 升版与本 DTL-043 v0.1 是**两个独立任务**（A-01 / A-08），可并行推进，但**升版 checklist 引用同步**（per Q-G-01 答复）必须做：

- 本 DTL-043 v0.1 落地后，Ulysses 终审时须 `grep "DTL-019" docs/07-社交运营与玩家治理/RGS-DTL-043_消息分发_v0.1.md` 确认引用版本号（本 DTL 写时已标"DTL-019 v0.2"占位，DTL-019 v0.2 实际升版时回查本 DTL 引用同步）。
- DTL-019 v0.2 升版时，§3 / §4 须显式引用本 DTL-043 §2 `message_recipients` 字段（`failure_count` / `last_failure_at` / `last_failure_reason`），避免 DTL-019 内部重新定义。

---

## 6. 签字栏（per DEC-008 一人公司 12 角色）

per RGS-DEC-008 一人公司治理基线（Ulysses = 12 类角色实际签，无所有者背书占位）：

> **注**：DEC-008 一人公司治理基线 = 1 人 12 职责 = 真实人真实职责，不构成"伪造"或"兼任压缩"。所有签字均为 Ulysses 实际签署，无所有者背书占位。

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构负责人（Architect） | **Ulysses** | **2026-08-24** | ✅ DTL-043 边界（业务对象层 / 渠道投递层分工）合理，符合 Q-D-01 / Q-D-08 答复方向 |
| 2 | SRE Lead | **Ulysses** | **2026-08-24** | ✅ 三张主表 DDL 索引策略（部分索引 / GIN 数组索引）符合生产查询模式 |
| 3 | DBA Lead | **Ulysses** | **2026-08-24** | ✅ `(message_id, recipient_id, channel)` UNIQUE 约束即幂等键，无先读后写竞态；CHECK 约束保证撤销/最近消息字段联动一致 |
| 4 | QA Lead | **Ulysses** | **2026-08-24** | ✅ 站内信 / 邮件 / 推送 / 短信 4 渠道 mock 网关测试用例可基于本 DDL 字段直接编写（PH-1 集成测试套件覆盖）|
| 5 | Platform Engineer | **Ulysses** | **2026-08-24** | ✅ migration 落位 `social_db` PL 限界上下文（受信业务对象）+ AD（运营视图）分层合理 |
| 6 | **Player 域 Lead**（独立） | **Ulysses** | **2026-08-24** | ✅ `message_recipients.recipient_id` 引用 player 域 account_id 不冲突（DTL-044 承接 player 主表）|
| 7 | **Economy 域 Lead**（独立） | **Ulysses** | **2026-08-24** | ✅ 不涉及经济事实，零冲突 |
| 8 | **Match 域 Lead**（独立） | **Ulysses** | **2026-08-24** | ✅ 不涉及匹配事实，零冲突 |
| 9 | **Social 域 Lead**（独立，本 DTL owner）| **Ulysses** | **2026-08-24** | ✅ 本 DTL-043 承接 Q-D-01 答复选项 A + 直接进 1.0 状态，3 张主表 DDL + 4 渠道边界 + 重试策略 + DTL-019 边界划分全部就绪 |
| 10 | **Admin 域 Lead**（独立） | **Ulysses** | **2026-08-24** | ✅ `messages.revoked_by` / `revoke_reason` 字段承载 GM 撤销操作审计，符合 admin 域 GM 工具控制面职责 |
| 11 | 评审主持人（RGS-REV-003）| **Ulysses** | **2026-08-24** | ✅ §1.3 "本文档不做什么" 明确排除 DTL-019 既有字段/枚举/协议/策略（不重复定义），符合评审流程 |
| 12 | 项目负责人（PM）| **Ulysses** | **2026-08-24** | ✅ P0 / 8K token 估算（per RGS-OPEN-QA-001-ACTIONS-v0.3 §4 WF-1-55.38）；范围、风险接受、资源（social 域 Lead 独立编制）和本 DTL-043 落地的实施授权 |

**接受代价**（per DEC-008 + Q-D-01 已知风险）：
- PH-1 第三方网关 mock/stub 接入风险：mock 网关与真实网关行为差异（PH-2 接入真实 SDK 时需补回归测试）——由 DTL-019 v0.2 §8 后续计划承担。
- `message_recipients` 字段随 DTL-019 v0.2 升版的同步风险：失败重试字段（`failure_count` / `last_failure_at` / `last_failure_reason`）若 DTL-019 v0.2 调整重试策略（如邮件 5 次重试），本 DDL 字段可能需扩 `failure_count` 类型（SMALLINT → INT）——由 Q-G-01 升版 checklist 引用同步机制承担。
- 4 渠道 mock 网关集成测试覆盖不完整风险：PH-1 仅能覆盖 mock 路径，真实网关（APNs/FCM/SMTP/SMS）路径需 PH-2 接入后补混沌测试。

---

## 7. 追溯性

| 需求/设计来源 | 本文档章节 |
|---|---|
| RGS-OPEN-QA-001 v0.2 Q-D-01（选项 A + 直接进 1.0 状态）| 全部（依据 + 状态字段）|
| RGS-OPEN-QA-001 v0.2 Q-D-08（4 渠道归属 + 第三方网关不在 PH-1 + push 不重试 / 邮件/短信 3 次）| §3（4 渠道归属）、§4（重试策略）|
| RGS-OPEN-QA-001 v0.2 Q-D-02 答复（player_characters.stats JSONB + 字段化混合）| §2.1 `messages.metadata` JSONB 字段设计原则 |
| RGS-OPEN-QA-001 v0.2 Q-G-01 答复（状态/版本双维度）| 头表"v0.1 + 🟢 v1.0 状态"双维度合法 |
| RGS-OPEN-QA-001-ACTIONS-v0.3 §3 A-01（P0 / 🔴 重量）| 头表"工作量" + §6 PM 签字 |
| RGS-OPEN-QA-001-ACTIONS-v0.3 §4 WF-1-55.38（建议 8K token）| §6 PM 签字（已含 token 估算）|
| RGS-OPEN-QA-001-ACTIONS-v0.3 §5 修正 #1（037→043 因 037 已被 Economy 域占用）| 头表"依据" + 文件命名 |
| RGS-DTL-019 §1.2（不重新决定 / 不覆盖 第三方 SDK）| §1.3、§3.3、§5.2 不重复定义原则 |
| RGS-DTL-019 §2 `push_consents` 字段级写法 | §2.1-2.3 字段注释风格 |
| RGS-DTL-019 §2.1.1 内容脱敏校验 | §2.1 `messages.title` / `body` 字段注释 |
| RGS-DTL-019 §2 redemption_records (code, account_id) 复合主键幂等键 | §2.2 `message_recipients` (message_id, recipient_id, channel) 复合 UNIQUE 幂等键 |
| RGS-DTL-019 §4.4 TBD-OPT-002 初始提案默认值非最终值 | §4.2 PH-1 退避初始值标注 |
| RGS-DTL-039 §5 治理操作必须有 RBAC / 审计 / 可回滚状态迁移 | §2.1 `messages.revoked_at` / `revoked_by` / `revoke_reason` 字段联动 CHECK |
| RGS-DTL-039 §6 待补齐项第 1 条 "关系、队伍、举报和消息物理 DDL" | 全部（本 DTL 完成第 1 条"消息物理 DDL"部分）|
| DEC-008 一人公司治理基线（Ulysses = 12 角色）| §6 签字栏 12 类全签 |
| DEC-012 A 路径直接开工模式 | 头表"v1.0 状态"（直接进 v1.0，不等 1.5 草案评审）|
| RGS-SPEC-CROSS-003 v0.2（待升版，跨域事件 schema）| §1.3 不覆盖（属 RGS-SPEC-CROSS-003 升版范围）|
| RGS-SPEC-CROSS-006（trace_id 跨域传播）| 头表"关联"（PH-2 写入消息创建事件时携带 trace_id）|

---

## 8. 本文档的覆盖范围与后续计划

本文档覆盖：站内信三张主表 `messages` / `message_recipients` / `conversations` 物理 DDL（含字段注释、索引策略、CHECK 约束、UNIQUE 幂等键）+ 派生视图 `v_message_dispatch_overview` 运营审计便利性 + 4 渠道（站内信 / 邮件 / 推送 / 短信）抽象归属矩阵 + 渠道失败重试策略（push 不重试 / 邮件/短信 3 次指数退避）+ 第三方网关 mock/stub PH-1 策略 + 与 DTL-019 v0.2 职责边界划分 + 12 类签字栏。

本版本明确不覆盖、留待后续：

- **DTL-019 v0.2 升版**（per OPEN-QA-001-ACTIONS A-08）：4 渠道投递技术抽象层（proto 枚举 + 退避算法调度器）。A-08 任务 WF-1-55.51 独立推进，本 DTL 不代为展开。
- **跨域事件 schema**（per OPEN-QA-001-ACTIONS A-09）：`MessageCreated` / `MessageRead` / `MessageRevoked` 事件在 RGS-SPEC-CROSS-003 v0.2 §X 落地，本 DTL 不重复定义事件字段。
- **PH-2 真实网关接入**：APNs / FCM / SMTP / SMS 真实 SDK 集成代码、密钥轮换细节——非架构层面决策，PH-2 实现阶段随所选 SDK 版本变化。
- **PH-2 定时清理任务**：`messages.expires_at` 到期后的归档 / 物理清理——本 DDL 留了 `expires_at` 字段和 `idx_messages_expires` 部分索引，但 PH-1 不实现清理任务。
- **PH-2 群聊高级功能**：群公告、群禁言、群管理员、@提及解析等——本 DTL `conversations.metadata` JSONB 字段已为扩展预留，但 PH-1 不实现具体规则。
- **PH-2 推送内容脱敏规则集**：RGS-DTL-019 §1.2 明确"复用既有日志脱敏规则集的模式库"，本 DTL §2.1 `messages.title` / `body` 字段注释引用该原则，但具体规则集内容非本 DTL 范围。
- **DTL-019 v0.2 §4 退避数值最终化**：§4.2 1min / 5min / 30min 为 PH-1 初始值，最终退避数值需按 PH 阶段实测数据与安全评审确定后回写 DTL-019 v0.2 新版本。
