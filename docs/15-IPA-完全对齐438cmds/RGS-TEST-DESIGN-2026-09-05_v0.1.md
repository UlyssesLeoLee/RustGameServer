# RGS 测试设计书 v0.1

> **创建日期**: 2026-09-05 06:50 JST
> **创建者**: Mavis 接手 agent per DEC-008 (代签 Ulysses)
> **依据**: 2026-09-05 06:43 JST 拍板 (scope=opt4 设计书+用例+plugin架构部分, depth=opt3 业务路径+错误码+边界+异常)
> **状态**: ⏳ 待 DDD Review 二审
> **关联**: REQ/BDD/DDD v0.2 + 3 addendum, RGS-PLUGIN-APP-ARCH-2026-09-05 v0.1

## 0. 目的

全面更新 RGS 测试设计, 覆盖:
1. **闪烁之光 438 cmds 全部 42 module** (per REQ/BDD/DDD v0.2 + 协议号映射 addendum)
2. **RGS 新架构 (plugin 集群 + app 集群)** (per RGS-PLUGIN-APP-ARCH-2026-09-05 v0.1)
3. **rgs-flash-mock 工具项目** (42 module fixture + 回归脚本)

**目标读者**: 测试工程师 / Ulysses 二审 / 5 域 Lead 签字

## 1. 测试分层架构 (per 8/27 JST L1/L1.1/L1.2 + 9/2 10:18 JST D2 拍板)

```
┌─────────────────────────────────────────────────────────────────┐
│ L1: 单元测试 (UT)                                                │
│   - cargo check --tests (限时 60s)                               │
│   - 每个 crate 独立, 跨工具链场景由主会话统一验证                 │
│   - 5 域主链路 + 平台层 + 工具 crate 全部覆盖                     │
└─────────────────────────────────────────────────────────────────┘
         ↓ ↑
┌─────────────────────────────────────────────────────────────────┐
│ L1.1: Lib 集成测试 (Lib Test)                                    │
│   - cargo test --lib (限时 120s)                                 │
│   - 域内 cross-module 场景                                       │
│   - 域 Lead 拍板签字 (per 8/21 JST 5 域独立 Lead 原则)            │
└─────────────────────────────────────────────────────────────────┘
         ↓ ↑
┌─────────────────────────────────────────────────────────────────┐
│ L1.2: 业务级 E2E 测试                                             │
│   - cargo test --test '*' -- --test-threads=1 (限时 300s+)        │
│   - 跨域 saga + mTLS 业务级                                      │
│   - 主会话统一跑, 验证业务 0 破坏 (per 8/27 401ac5c 业务级 mTLS 实践)│
└─────────────────────────────────────────────────────────────────┘
         ↓ ↑
┌─────────────────────────────────────────────────────────────────┐
│ L2: 端到端 (E2E) — rgs-flash-mock 驱动                             │
│   - smoke-test.sh (12 基础 RPC)                                  │
│   - regression-test-12-partial.sh (12 Partial module)             │
│   - regression-test-30-new-module.sh (30 新 module)               │
│   - regression-test-60-all-modules.sh (本设计书新增, 全 60 module) │
│   - regression-test-plugin-poc.sh (本设计书新增, 1 PoC WASM)      │
└─────────────────────────────────────────────────────────────────┘
         ↓ ↑
┌─────────────────────────────────────────────────────────────────┐
│ L3: 性能 / 混沌 (Drill)                                          │
│   - cluster-ops/tests-disabled/ (per RGS-INC-002 v0.1)         │
│   - LCM drill_lcm_001-010 + drill_chaos + drill_nfr + drill_risk  │
│   - 8/27 INC 复盘: 临时禁用, 需 saga 编译死锁修复后重启用         │
└─────────────────────────────────────────────────────────────────┘
```

## 2. 测试范围矩阵 (60 module × 4 用例类)

per 2026-09-05 06:43 JST 拍板 (depth=opt3):

| 用例类 | 每 module 数量 | 总计 (60 module) | 覆盖目标 |
|---|---|---|---|
| **业务路径 (HAPPY PATH)** | 6-8 | ~400 | 正常 RPC 调用 + 响应字段验证 |
| **错误码 (ERROR CODE)** | 2-4 | ~150 | 业务错误码 1:N 映射 (0=ok, 401/403/500/1001/...) |
| **边界值 (BOUNDARY)** | 4-5 | ~250 | 空 / 极大 / 极小 / 特殊字符 / 负数 |
| **异常路径 (EXCEPTION)** | 2-3 | ~150 | 网络中断 / 超时 / 重入 / 幂等性 |
| **总计** | **15-20** | **~950** | 4 类全覆盖 |

**60 module 来源**:
- 42 module × RGS 7 域 1:1 映射 (per 协议号映射 addendum §5)
- 18 跨域抽象 module (saga / outbox / event-bus / plugin-registry / app-deployment / RBAC / mTLS / audit_log / replay / 等) — **本设计书新增** — 5 域 + 平台层各加 3 个
  - player: profile-sync / session / inventory
  - economy: trade / ledger / saga
  - match: matchmaking-v2 / spectator / replay
  - social: guild / party / chat
  - admin: audit-log / RBAC / gm-handler
  - shared-platform: outbox / mTLS / RBAC
  - cluster-ops: realm-lifecycle / app-deployment / plugin-registry
  - function-plane: registry / gateway / wasm-host

**实际总计**: 42 + 18 = **60 module**

## 3. 业务路径用例模板 (HAPPY PATH, 6-8/module)

每个 module 6-8 用例模板 (per 闪烁之光 5 layer + RGS 7 layer 映射):

```yaml
# 模板: 1 module 1 RPC 1 用例
- module: {module_name}              # e.g. "card.combat"
  protocol_id: {20000-29999}         # 闪烁之光协议号
  rgs_rpc: {RGS gRPC method}         # e.g. "CombatService.PrepareCombat"
  rgs_backend: {svc-name:port}       # e.g. "match-service:50053"

  test_id: HP-{module_code}-{rpc_seq}  # e.g. "HP-COMBAT-001"
  test_name: "{RPC 中文名} 业务路径"
  description: "{正常调用场景描述}"
  preconditions:
    - player 域已注册
    - session 有效
    - {module} 域 service 在线
  steps:
    1. gRPC call {module}.{rpc_name}({request_fields})
    2. 等待响应
  expected:
    - response.code == 0
    - response.msg == "ok"
    - {response_fields} 字段非空且符合 schema
    - 域内 outbox 表新增 1 行 (业务事件)
  evidence:
    - rgs-flash-mock: mock_data/{module}.json rpcs.{code}.mock_response
    - rgs-testkit: per-module test
```

## 4. 错误码用例模板 (ERROR CODE, 2-4/module)

per 8/27 11:06 JST 错误码 REDACTED filter + 9/4 REQ/BDD 文档错误码映射:

```yaml
- test_id: EC-{module_code}-{rpc_seq}
  test_name: "{RPC} 错误码覆盖"
  description: "覆盖该 RPC 可能的 N 个业务错误码"
  preconditions: 域 service 在线
  steps:
    1. 用不同错误输入调 {rpc}
  expected_matrix:
    - input: "正常输入"      → code: 0     (OK)
    - input: "空字段"        → code: 1001  (PARAM_INVALID)
    - input: "权限不足"      → code: 1003  (PERM_DENIED)
    - input: "资源不存在"    → code: 1004  (NOT_FOUND)
    - input: "服务端内部错"  → code: 500   (INTERNAL)
```

**通用错误码 (per 闪烁之光 proto_common)**:
- 0: OK
- 1001-1099: 参数 / 校验错
- 1100-1199: 鉴权 / 权限
- 2000-2099: 资源不存在 / 已存在
- 3000-3099: 状态机非法迁移
- 4000-4099: 业务锁 / 配额
- 5000-5099: 服务端内部错
- 9000-9099: 上游依赖故障

## 5. 边界值用例模板 (BOUNDARY, 4-5/module)

```yaml
- test_id: BV-{module_code}-{rpc_seq}
  test_name: "{RPC} 边界值覆盖"
  cases:
    - input: "" (空字符串)         → 期望: 1001 PARAM_INVALID 或 0 (协议允许)
    - input: "a" * 4096 (超长)     → 期望: 1001 或 1002 (字段超长)
    - input: 0 / -1 / i64::MAX    → 期望: 0 (边界正常) 或 1001
    - input: 特殊字符 \n\r\t\"\\'  → 期望: 0 (转义后) 或 1001
    - input: 重复 id (幂等性测试)   → 期望: 0 (二次返回相同结果)
    - input: 并发同 id (竞态)       → 期望: 0 (锁) 或 1003 (状态冲突)
```

## 6. 异常路径用例模板 (EXCEPTION, 2-3/module)

per 8/27 55.26 fail-closed 精神 + 8/27 11:06 JST 凭据硬 ban:

```yaml
- test_id: EX-{module_code}-{rpc_seq}
  test_name: "{RPC} 异常路径"
  cases:
    - scenario: 网络中断 (client -> server 1s 后断)
      expected: client 端 retry, server 端事务回滚, 重试幂等成功
    - scenario: 超时 (server 处理 > 5s)
      expected: client 端 deadline exceeded, server 端继续但写 dead-letter
    - scenario: 重入 (同一 id 连续 3 次, 间隔 100ms)
      expected: 第一次成功, 后续幂等返回相同结果
    - scenario: mTLS 证书失效 (RGS_TLS_DIR/ca.pem 过期)
      expected: 5xx fail-closed, 详细 trace 包含 cert expiry timestamp
    - scenario: 凭据泄漏尝试 (env var 出现在 log)
      expected: log 被 REDACTED filter 替换, 不出现明文
```

## 7. 跨域 saga / plugin / app 运维 专项用例

### 7.1 跨域 saga (per 8/27 economy::saga)

```yaml
- test_id: SAGA-001
  test_name: "经济域 Reserve 流程跨域 saga 端到端"
  modules: [economy::saga_orchestrator, player, match]
  steps:
    1. economy 收 Reserve 请求
    2. reserve_handler 改 accounts + reservations 表
    3. confirm_handler 改 transaction_ledger 表
    4. outbox 发 1 条 to player service
  expected: 4 步全成功, outbox 表 status=Sent, ledger 一致
```

### 7.2 plugin 集群 (per RGS-PLUGIN-APP-ARCH-2026-09-05 v0.1 §2.2)

```yaml
- test_id: PLUGIN-001
  test_name: "PoC plugin 抽卡概率 hot-swap 不重启 app"
  preconditions:
    - card-service running v0.1 (含 native fallback)
    - function-plane mock 运行, 1 个 plugin "draw_card_probability" v0.1.0 Active
  steps:
    1. POST /ops/functions/draw_card_probability/register v0.2.0 (新 WASM)
    2. 状态 Draft → Active (灰度 10% cards)
    3. 监控 5min, error rate < 1%
    4. set_old_status(Archived)
  expected:
    - card-service 进程不重启
    - 抽卡结果符合新概率 (e.g. SSR 概率 5% → 6%)
    - 100% 旧 fallback 链仍能跑 (plugin 故障不挂 app)
```

### 7.3 app 独立更新 (per ARCH §3.4)

```yaml
- test_id: APP-DEPLOY-001
  test_name: "player-service v0.2 灰度发布 10% → 100%"
  preconditions:
    - k3s namespace 4 个 player-service Pod
  steps:
    1. kubectl set image player-service=v0.2 (1 Pod)
    2. 监控 5min, latency p99 < 100ms, error rate < 0.5%
    3. 扩到 2 Pod, 监控 5min
    4. 全量 4 Pod
  expected:
    - 旧 3 个 Pod 仍服务, traffic 切到新 1 Pod
    - 跨 Pod function-plane 调用 OK (双 registry 模式)
    - k8s rolling update 不中断业务
```

### 7.4 ops 运维 UI (per ARCH §2.3)

```yaml
- test_id: OPS-UI-001
  test_name: "gm-backend /ops/functions/register RBAC 拦截"
  preconditions: GM 域 Lead 已登录
  steps:
    1. POST /ops/functions/card.draw_card_probability/register (域 Lead)
    2. POST /ops/functions/economy.price_curve/register (同域 Lead 越权)
  expected:
    - card.* 通过 (域 Lead 责任域)
    - economy.* 返回 403 PERM_DENIED (域 Lead 不能 register 跨域)
```

## 8. 工具项目 rgs-flash-mock 集成测试

### 8.1 现状盘点 (per 9/4 v0.3 设计)

- 42 module fixture (combat.json / partner.json / ...) — `tools/rgs-flash-mock/mock_data/`
- mock src: gap_matrix.rs (222 行) + handlers.rs (178 行) + main.rs (70 行) + config.rs (112 行)
- scripts: smoke-test.sh / regression-test-12-partial.sh / regression-test-30-new-module.sh

### 8.2 本设计书新增

- `regression-test-60-all-modules.sh` (本设计书新增, 全 60 module)
- `regression-test-plugin-poc.sh` (本设计书新增, 1 PoC WASM)
- `regression-test-error-codes.sh` (本设计书新增, 通用错误码 1:N 映射)
- `regression-test-boundary.sh` (本设计书新增, 边界值)
- `regression-test-exception.sh` (本设计书新增, 异常路径)

### 8.3 k3s 部署 (per ARCH §2.3 复用 gm-backend)

- `tools/rgs-flash-mock/k3s/30-rgs-flash-mock-deployment.yaml` (现状)
- `tools/rgs-flash-mock/k3s/31-rgs-flash-mock-service.yaml` (现状)
- **本设计书新增**: rgs-flash-mock 与 function-plane PoC 联合部署 yaml

## 9. 工具 crate rgs-testkit 复用 (per 9/1 8/27 JST 派工约束)

- NoOp* 替身 (跨域场景)
- fixture 加载 (json/yaml)
- HTTP client (调用 gm-backend ops)
- gRPC client (调用 5 域 + function-plane)
- tracing capture (验证 REDACTED filter)

## 10. DoD 与交付

per AGENTS.md §2.1 + 9/2 10:18 JST D2 拍板:

| 测试类型 | DoD | 交付 |
|---|---|---|
| L1 (UT) | `cargo check --tests` 0 error (限时 60s) | 19 crate 全部通过 |
| L1.1 (Lib) | `cargo test --lib` 全过 (限时 120s) | 565+ tests passed |
| L1.2 (E2E) | `cargo test --test '*' -- --test-threads=1` 0 fail | 跨域 saga + mTLS 全过 |
| L2 (mock) | 5 回归脚本 PASS | smoke + 12-partial + 30-new + **60-all + plugin-poc (新增)** |
| L3 (drill) | tests-disabled/ 重新启用 (待 INC-002 saga fix) | LCM drill 全过 |

## 11. 派生约束对齐

- L1 派生约束 (per AGENTS.md §2.1)
- 5 域独立 Lead (per 2026-08-21 JST)
- 凭据永不打印 (per 8/27 11:06 JST + REDACTED filter)
- 缺标比错标 (per 8/26 JST)
- 不追溯改写 (per 8/27 JST + 8/26 JST DTL-036)
- mavis 默认代签 Ulysses (per 8/27 19:39/20:56/21:59 JST)

## 12. 修订历史

| 版本 | 日期 | 修订人 | 审批 | 摘要 |
|---|---|---|---|---|
| v0.1 | 2026-09-05 06:50 JST | Ulysses — Mavis 接手 | ⏳ 待 DDD Review 二审 | 起草, 60 module × 4 类 ~950 用例 + plugin 架构部分 |
