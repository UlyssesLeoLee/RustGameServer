# 详细设计书（詳細設計書 / Detailed Design Document）

**运营管控与服务 Agent 矩阵 — Operations & Service Agent Matrix**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DTL-034 |
| 版本 | 0.2 |
| 父文档 | RGS-BAS-034 基本设计书 |
| 制定日 | 2026-08-20 |
| 最终更新日 | 2026-08-20 |
| 制定者 | 架构师 |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |

---

## 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-20 | 架构师 | 初版制定。 |
| 0.2 | 2026-08-20 | 架构师 | 为补偿意图补齐签名、时效、nonce、密钥标识与服务端验证/审计约束；不批准自动补偿范围或额度。 |

---

## 1. SRE 与 客服 Agent 结构化输出 JSON Schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "CSCompensationIntentSchema",
  "type": "object",
  "additionalProperties": false,
  "properties": {
    "intent_id": { "type": "string", "format": "uuid" },
    "action_type": { "const": "issue_compensation" },
    "payload_version": { "const": "v1" },
    "player_id": { "type": "integer", "minimum": 1 },
    "currency_type": { "type": "integer", "minimum": 0 },
    "amount": { "type": "integer", "minimum": 1 },
    "order_id": { "type": "string", "minLength": 1 },
    "trace_id": { "type": "string", "format": "uuid" },
    "evidence_hash": { "type": "string", "minLength": 1 },
    "issued_at": { "type": "string", "format": "date-time" },
    "expires_at": { "type": "string", "format": "date-time" },
    "nonce": { "type": "string", "minLength": 1 },
    "key_id": { "type": "string", "minLength": 1 },
    "signature": { "type": "string", "minLength": 1 },
    "rationale": { "type": "string" }
  },
  "required": ["intent_id", "action_type", "payload_version", "player_id", "currency_type", "amount", "order_id", "trace_id", "evidence_hash", "issued_at", "expires_at", "nonce", "key_id", "signature"]
}
```

## 2. 服务端验证与审计约束

1. Schema 校验只是入口条件。L0 Action Gate 必须基于规范化的无 `signature` 载荷重建签名输入，使用 `key_id` 查找受控公钥并验证 Ed25519 签名。
2. 闸门以服务端时钟校验 `issued_at < expires_at`、过期状态及 nonce 唯一性；重复 nonce 或已结算的 `order_id` 一律拒绝。nonce 的保存期不得短于其意图的可接受时效窗口。
3. `amount` 的最终上限、货币范围、玩家资格和订单状态必须从权威账本/订单服务读取，不能相信 Schema 中的候选值；这些业务阈值仍受 TBD-AGO-001 的具名审批约束。
4. 接受、拒绝和实际执行均须写入审计记录，最少包含 `intent_id`、`trace_id`、证据摘要、`key_id`、校验结果、策略版本和回执摘要。
