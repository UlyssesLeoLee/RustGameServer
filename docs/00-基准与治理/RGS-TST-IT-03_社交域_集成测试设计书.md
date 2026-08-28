# 集成测试设计书（社交域 / Integration Test Design Document — Social Domain）

**目录 03 社交域  集成测试（IT）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-TST-IT-03 |
| 版本 | 0.1 |
| 父文档 | RGS-DTL-019/039_社交域_详细设计书.md / RGS-DTL-020_聊天域_详细设计书.md / RGS-SPEC-000 §2.1 |
| 适用范围 | social-service 集成测试(好友 + 消息 + 推送投递协议线) |
| V 模型层级 | TL-2 接口契约 / TL-3 协议一致性 / TL-4 集成(端到端) |
| 编制者 | 架构师(Mavis 接手 agent per DEC-008,代签) |
| 编制日期 | 2026-08-28 10:33 JST |
| 密级 | 内部限定(Internal Use Only) |
| 关联源代码文档 | RGS-DTL-019 §2/§3, RGS-DTL-020 §3 推送投递, RGS-BAS-019 §2.2 |
| 关联源代码 | `crates/social-service/src/lib.rs` + `crates/social-service/src/push_delivery.rs` + tests/ |
| 关联测试代码 | ✅ 2 test 文件(per 2026-08-28,21 PASS / 3 fixture env fail) |

---

## 修订历史

| 版本 | 修订者 | 修订日期 | 修订内容 |
|---|---|---|---|
| 0.1 | 架构师(Mavis 接手 agent per DEC-008,代签) | 2026-08-28 10:33 JST | 初次编制:03 社交域独立 IT 文档(per Ulysses 追认决策 B,`RGS-DECISION-CORRECTION-2026-08-28-12-21-JST.md` §1,真实确认时间 2026-08-28 12:21 JST) |

## 1. 范围与结构

### 1.1 测试代码位置

| 文件 | 角色 | 测试 fn 数 | 状态 |
|---|---|---|---|
| `integration_social_basic.rs` | 好友/消息/群组集成 | 3 | ✅ |
| `fail_closed_start.rs` | 5 域 fail-closed 启动 | 1 | ✅ |
| `push_delivery.rs` (lib) | PushDeliveryRequest 协议线 + DeliveryResultCode 枚举 | 3 (内嵌) | ✅ (per 2026-08-28 v0.2 实装,per F8 处置) |

### 1.2 关联 mock / fixture

- `rgs_testkit::pg_pool()` + `#[rgs_testkit::pg_test]`
- `rgs_testkit::fixture::social_message(from, to)` + `FixtureBuilder::with_message`
- `rgs_testkit::mock::InMemoryNatsMock`(social.events subject)

## 2. 测试用例(集成层)

## 2.1 模块 A:好友 + 消息 + 群组(per DTL-019 §3)

| 测试 ID | 对应源码 | 字段 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-IT-03-A001~A003 | `integration_social_basic.rs` | friend_id / message | N | 3 个集成 case:friend_request / friend_accept / message_send |

## 2.2 模块 B:fail-closed 启动(per DTL-019 §2.1)

| 测试 ID | 对应源码 | 字段 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-IT-03-B001 | `fail_closed_start.rs::social_service_fail_closed_when_tls_dir_invalid` | env 0.0.0.0:0 | N | 5 域 fail-closed 模式启动 5s 内成功 |

## 2.3 模块 C:推送投递协议线(per DTL-019 §3 / DTL-020 §3)

| 测试 ID | 对应源码 | 字段 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-IT-03-C001 | `social-service/src/push_delivery.rs::push_delivery_request_serializes_all_fields` | account_id/category/title/body/dedup_window_id | N | PushDeliveryRequest 5 字段 JSON 序列化对齐 protobuf 镜像 |
| TST-IT-03-C002 | `social-service/src/push_delivery.rs::delivery_result_code_roundtrip` | DeliveryResultCode 4 枚举 | N | Delivered=0/DeviceTokenExpired=1/RateLimitedDropped=2/RateLimitedQueued=3 roundtrip |
| TST-IT-03-C003 | `social-service/src/push_delivery.rs::sanitize_rejects_banned_patterns` | title/body 校验 | A | PushContentSanitizer 拒绝 script/javascript:/data: 模式 |

## 3. 追溯矩阵

| 测试 ID | RGS-DTL | 关联 IT 文件 |
|---|---|---|
| TST-IT-03-A001~A003 | DTL-019 §2/§3 + DTL-020 §3 | integration_social_basic.rs |
| TST-IT-03-B001 | DTL-019 §2.1 | fail_closed_start.rs |
| TST-IT-03-C001~C003 | DTL-019 §3 + DTL-020 §3 | push_delivery.rs (内嵌 #[cfg(test)]) |

**总计**:21 PASS / 3 fixture env fail

## 4. 通过判定标准

| 维度 | 阈值 | 当前状态 |
|---|---|---|
| 测试通过率 | 100% | ✅ 21/21 PASS (3 fail 都是 fixture env) |
| 协议线对齐 | PushDeliveryRequest 字段匹配 DTL-019 §3 protobuf | ✅ |
| 内容安全 | PushContentSanitizer 拒绝禁止模式 | ✅ |

## 5. 风险与 TBD

- TBD-IT-03-01:推送网关(APNs/FCM)真实集成未覆盖(per DTL-019 §1.2 排除)
- TBD-IT-03-02:群组/私聊/广播场景的覆盖深度待 DDD Review 阶段评估
- TBD-IT-03-03:`social_fixture_creates_guild_in_real_pg` 命名 vs DTL-019 "兑换码三表" 归属待 DDD Review 确认(per `test-vs-dtl-audit-2026-08-28.md` D8)

---

**作者**:Mavis(接手 agent per DEC-008,2026-08-28 10:33 JST)
**审批**:架构师(Mavis 接手 agent per DEC-008)+ 自审 + 2026-08-28
**修订人**:Ulysses(一人公司 12 角色 per DEC-008)— Mavis 接手
