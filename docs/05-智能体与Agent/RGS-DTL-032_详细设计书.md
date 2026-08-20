# 详细设计书（詳細設計書 / Detailed Design Document）

**SRE 运维 Agent 与 客服 Agent 体系 SRE Operations Agent & Customer Support Agent System**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DTL-032 |
| 版本 | 0.2 |
| 父文档 | RGS-BAS-032 基本设计书 |
| 制定日 | 2026-08-20 |
| 最终更新日 | 2026-08-20 |
| 制定者 | 架构师 |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |

---

## 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-20 | 架构师 | 初版：SRE 运维 Agent 与客服 Agent 动作接口 |
| **0.2** | 2026-08-20 | 架构师 | 固化 L0 补偿闸门：JWT—载荷绑定、动作/货币/正数校验、intent/order 双幂等、玩家/全局日额度原子预留、重放拒绝审计及测试判定 |

---

## 1. 核心接口与数据结构契约

### 1.1 ActionIntent 统一通信载荷（Protobuf/IDL）

```protobuf
syntax = "proto3";
package rgs.agent.v1;

enum ActionType {
  ACTION_TYPE_UNSPECIFIED = 0;
  ACTION_TYPE_QUARANTINE_NODE = 1;
  ACTION_TYPE_DRAIN_CONNECTIONS = 2;
  ACTION_TYPE_ISSUE_COMPENSATION = 3;
  ACTION_TYPE_ESCALATE_TO_HUMAN = 4;
}

message ActionIntent {
  string intent_id = 1;                  // UUID；也是 JWT jti，禁止复用
  string agent_id = 2;                   // ops-agent / cs-agent；也是 JWT sub
  int64 timestamp = 3;                   // Unix ms；与 iat 的偏差不得超过 10 秒
  ActionType action_type = 4;

  oneof payload {
    QuarantineNodePayload quarantine_node = 5;
    IssueCompensationPayload issue_compensation = 6;
    EscalatePayload escalate = 7;
    DrainConnectionsPayload drain_connections = 10;
  }

  // 仅为已发布客户端保留字段号；L0 不得再以此字段作为授权依据。
  string legacy_signature = 8 [deprecated = true];
  // 完整紧凑 JWS JWT（EdDSA），其受信声明绑定本指令的内容，见 §1.2。
  string authorization_jwt = 9;
}

message QuarantineNodePayload {
  string node_id = 1;
  string cluster_id = 2;
  string reason = 3;
  int32 timeout_seconds = 4;
}

message DrainConnectionsPayload {
  string node_id = 1;
  string cluster_id = 2;
  int32 timeout_seconds = 3;
  string reason = 4;
}

message EscalatePayload {
  string case_ref = 1;
  string queue = 2;
  string rationale = 3;
}

message IssueCompensationPayload {
  uint64 player_id = 1;
  uint32 currency_type = 2;              // 货币型补偿时必须为 L0 配置白名单中的非零值
  int64 amount = 3;                      // 货币型补偿时必须为正整数
  uint32 item_id = 4;
  uint32 item_count = 5;                 // 道具型补偿时必须为正整数
  string order_ref = 6;                  // 不可复用的业务订单引用
  string rationale = 7;                  // 补偿理由与证据摘要
}
```

### 1.2 JWT 授权与载荷一致性

写动作必须同时经服务间 mTLS 身份认证和本节 JWT 授权；`legacy_signature` 只能用于兼容性记录，**不得**绕过 JWT 校验。JWT 使用 Action Gate 受信 Ed25519 JWK 集验签，必须包含并逐字段匹配以下声明：`iss=rgs-agent-authorizer`、`aud=rgs-action-gate`、`sub=agent_id`、`jti=intent_id`、`iat`、`nbf`、`exp`、`action_type`、`payload_sha256`、`scope=action:execute`。`exp-iat` 不得超过 10 秒，且服务端时钟对 `iat/nbf/exp` 的容差不超过 10 秒。

`payload_sha256` 是对按 Protobuf 确定性序列化的 `(intent_id, agent_id, timestamp, action_type, oneof payload)` 所得字节的 SHA-256。L0 必须在验签后重新计算该值；任一声明、指令类型、主体、时间或载荷摘要不一致均以 `JWT_INTENT_MISMATCH` 拒绝并审计，不能降级为仅检查签名。

### 1.3 确定性输入校验

在进入任何外部动作前，L0 必须依次校验：

1. `intent_id` 为规范 UUID，`agent_id` 在授权主体白名单内，时间戳与 JWT 均未过期；
2. `action_type` 与 `oneof payload` 一一对应，恰有一个载荷：隔离/排空/补偿/人工升级分别只能携带同名载荷；
3. 补偿的 `player_id`、`order_ref`、`rationale` 非空且格式合法；`order_ref` 在规范化后长度和字符集受限；
4. 补偿必须为**二选一**：货币型要求 `currency_type` 在 L0 配置白名单、`amount > 0`、`item_id=item_count=0`；道具型要求 `item_id` 在商品白名单、`item_count > 0`、`currency_type=amount=0`。禁止零、负数、未知货币、混合货币/道具载荷及溢出；
5. 货币型的金额不得超过该货币的单次硬上限，且额度配置缺失、读取失败或数值溢出时一律拒绝（fail-closed）。

---

## 2. L0 确定性动作闸门（Action Gate）实现标准

### 2.1 幂等、额度与审计的事务边界

补偿的幂等键同时为 `intent_id` 和规范化 `order_ref`。同一 `intent_id` 加同一指令摘要的重试必须返回首次已持久化的 `ExecutionReceipt`，不得二次记账；相同 `intent_id` 但摘要不同、或已由另一指令消费的 `order_ref`，必须以 `REPLAY_REJECTED` 拒绝。`intent_id`、`order_ref`、`player_id`、货币、UTC 自然日组成的日额度桶、全局货币/UTC 自然日额度桶、经济账本分录和成功审计/Outbox 必须在**同一数据库事务**中写入与提交。

日额度采取数据库行锁或等价的原子条件更新，不能采用“先查再加”的应用层逻辑：玩家桶与全局桶任一超限即回滚全部写入。所有拒绝（含 JWT、载荷、额度和重放）必须写入不可篡改审计流，至少记录 `intent_id`、`order_ref` 哈希、主体、动作、载荷哈希、拒绝码、验证时间及关联追踪 ID；不得记录完整 JWT 或敏感理由原文。

```rust
pub struct ActionGate {
    compensation_limits: CompensationLimits, // 货币白名单、单次/玩家日/全局日硬上限
    jwt_verifier: JwtVerifier,               // 仅接受受信 EdDSA JWK
    audit_logger: AuditLogger,
}

impl ActionGate {
    pub async fn execute_action(
        &self,
        intent: ActionIntent,
        ctx: &mut ExecutionContext,
    ) -> Result<ExecutionReceipt, GateError> {
        let canonical = validate_action_payload(&intent, &self.compensation_limits)
            .map_err(|e| self.reject_and_return(&intent, e))?;
        let claims = self.jwt_verifier.verify(&intent.authorization_jwt)
            .map_err(|e| self.reject_and_return(&intent, GateError::InvalidJwt(e)))?;
        validate_claim_binding(&claims, &canonical, Duration::from_secs(10))
            .map_err(|e| self.reject_and_return(&intent, e))?;

        ctx.transaction(|tx| async move {
            // 同一摘要的安全重试返回原回执；摘要不同的 intent_id 视为重放。
            if let Some(receipt) = tx.load_intent_receipt(intent.intent_id, canonical.hash).await? {
                return Ok(receipt);
            }
            tx.reject_conflicting_intent_or_order(intent.intent_id, canonical.hash, canonical.order_ref_hash).await?;

            match canonical.payload {
                ValidatedPayload::Compensation(comp) => {
                    tx.reserve_compensation_quotas_atomically(
                        comp.player_id, comp.currency_type, comp.amount, utc_today(),
                        self.compensation_limits.for_currency(comp.currency_type),
                    ).await?; // 玩家桶和全局桶均用锁定行/条件 UPDATE；任何超限即失败
                    tx.credit_player_asset(comp.player_id, comp.currency_type, comp.amount).await?;
                }
                ValidatedPayload::Quarantine(q) => tx.quarantine_node_idempotently(q).await?,
                ValidatedPayload::Drain(d) => tx.drain_node_idempotently(d).await?,
                ValidatedPayload::Escalate(e) => tx.create_escalation_idempotently(e).await?,
            }

            let receipt = tx.store_intent_receipt(intent.intent_id, canonical.hash).await?;
            tx.append_success_audit_and_outbox(&intent, &canonical, &receipt).await?;
            Ok(receipt)
        }).await.or_else(|e| self.reject_and_return(&intent, e))
    }
}
```

### 2.2 测试设计与可验收条件

| 层级 | 覆盖项 | 通过判定 |
|---|---|---|
| UT | JWT 的 `jti/sub/action_type/payload_sha256/exp` 任一不匹配、过期或未知签发键 | L0 不调用动作接口，并记录对应拒绝码 |
| UT | 动作—载荷不匹配、零/负金额、未知货币、货币与道具混合、超单次限额 | 全部被确定性拒绝；账本和额度桶均无变化 |
| UT | 同 `intent_id` 安全重试、同 ID 不同摘要、重复/冲突 `order_ref` | 仅安全重试返回原回执；其余产生 `REPLAY_REJECTED` 审计 |
| IT | 并发请求争用同一玩家及全局日额度桶 | 成功金额不超过任一上限，失败请求无账本分录，两个桶与账本一致 |
| IT/ST | 账本或审计/Outbox 写入故障、重启后的重试 | 事务全回滚或返回既有回执；不存在“已发补偿而无审计/额度”的状态 |
