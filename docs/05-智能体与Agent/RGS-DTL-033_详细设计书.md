# 详细设计书（詳細設計書 / Detailed Design Document）

**Agent 平台底座与通用运行时 — Agent Platform Infrastructure & Universal Runtime**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DTL-033 |
| 版本 | 0.2 |
| 父文档 | RGS-BAS-033 基本设计书 |
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
| 0.2 | 2026-08-20 | 架构师 | 补齐 `ActionIntent` 与 `ActionReceipt` 的受控传输契约及闸门不变量；不构成对任何自动写操作的批准。 |

---

## 1. 通用通信协议与数据契约 (gRPC/Protobuf)

```protobuf
syntax = "proto3";
package rgs.agent.platform.v1;

import "google/protobuf/timestamp.proto";

service AgentRuntimeService {
  rpc DispatchTask(TaskRequest) returns (TaskResponse);
  rpc QueryMemory(MemoryQueryRequest) returns (MemoryQueryResponse);
  rpc SubmitActionIntent(ActionIntent) returns (ActionReceipt);
}

message TaskRequest {
  string task_id = 1;
  string session_id = 2;
  string agent_type = 3;  // "sre" / "cs" / "gm" / "npc" / "econsys"
  string prompt = 4;
  map<string, string> metadata = 5;
}

message TaskResponse {
  string task_id = 1;
  string status = 2;      // "COMPLETED" / "FAILED" / "NEEDS_APPROVAL"
  string answer = 3;
  repeated ActionIntent generated_intents = 4;
  int32 total_tokens_used = 5;
}

// Agent 只能提交意图，不能获得业务写权限。
message ActionIntent {
  string intent_id = 1;
  string action_type = 2;
  string subject_type = 3;
  string subject_id = 4;
  bytes canonical_payload = 5;
  string payload_schema_version = 6;
  string trace_id = 7;
  string evidence_hash = 8;
  google.protobuf.Timestamp issued_at = 9;
  google.protobuf.Timestamp expires_at = 10;
  bytes nonce = 11;
  string key_id = 12;
  bytes ed25519_signature = 13;
}

enum ActionReceiptStatus {
  ACTION_RECEIPT_STATUS_UNSPECIFIED = 0;
  ACTION_RECEIPT_STATUS_ACCEPTED = 1;
  ACTION_RECEIPT_STATUS_REJECTED = 2;
  ACTION_RECEIPT_STATUS_EXECUTED = 3;
}

message ActionReceipt {
  string intent_id = 1;
  ActionReceiptStatus status = 2;
  string rejection_code = 3;
  string audit_event_id = 4;
  google.protobuf.Timestamp decided_at = 5;
  bytes receipt_digest = 6;
}
```

## 2. ActionIntent 闸门不变量

1. `ActionIntent` 是候选请求，不是能力凭证；只有 L0 Action Gate 可调用业务写接口。
2. 闸门以服务端时钟验证 `issued_at < expires_at` 与未过期状态，并在验签前后拒绝字段缺失、版本不支持或 payload 不一致的请求。
3. 签名必须覆盖 `action_type`、主体、`canonical_payload`、证据摘要、时效、nonce、`key_id` 与协议版本；`canonical_payload` 的编码规则由后续实现规范固定，禁止以非确定性 JSON 序列化验签。
4. nonce、业务幂等键与权威订单/账本状态均由闸门侧复核。Agent 提供的额度、订单状态和证据只可作为候选输入，不能替代权威数据源。
5. 每次接受、拒绝或执行必须生成 `ActionReceipt` 并写入不可抵赖审计记录；回执不能反向授予任何权限。
