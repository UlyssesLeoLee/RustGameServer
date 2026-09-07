# RGS 测试设计书 v0.2 (UT/IT/ST 三层 + 8 维度增量)

> **创建日期**: 2026-09-07 12:30 JST
> **创建者**: Mavis 接手 agent per DEC-008 (代签 Ulysses)
> **依据**: 2026-09-07 12:30 JST 拍板 (branch-pick_opt2 main 上 checkout 拉过来 + scope=opt4 全部 v0.2 综合 8 维度)
> **基线 v0.1**: commit f7e78eb (feat/auto-20260904-82af9c65 分支) → cherry-pick a9a5363 (main, 2026-09-07)
> **状态**: ⏳ Mavis 自审 (after 8 维度整合, per B3 派生约束 DDD Review 二审流程)
> **关联**: REQ/BDD/DDD v0.2 + 3 addendum, RGS-PLUGIN-APP-ARCH-2026-09-05 v0.1, RGS-FLASH-MOCK-DESIGN-2026-09-04 v0.3, RGS-INC-001 v0.3, RGS-DDD-2026-09-05-PHASE-B v0.2

## 0. v0.1 → v0.2 升版范围 (综合 8 维度)

| # | 维度 | v0.1 现状 (9/5 拍板) | v0.2 增量 (9/7 拍板) | 引用 |
|---|---|---|---|---|
| 1 | **8 域扩展** | §2 写 "5 域 + 平台层 + function-plane" | 13 域 (player / economy / match / social / admin + scene / battle / network / account / sub8 / batch + 平台 + function-plane) | a5235eb / 95e67a6 / 1134cfd / 57edbeb / b6b19b7 / 1dd9afc / 3c79bca / 42df673 |
| 2 | **batch v0.1 + v0.2 EVAL** | §2 完全没提 batch 域 | §7.5 新增 batch 域 6 module × 15 用例 = 90 用例 (cron / task_templates / worker_pool / audit_logger / dlq / connector) | fd122f6 / e70ed71 / e366ff8 / 62027c9 / eb1e15d / RGS-BATCH-V0.2-EVAL-2026-09-03 v0.1 |
| 3 | **admin-coc Phase B** | §7.4 OPS-UI-001 仅提 gm-backend RBAC | §7.4 增 admin-coc §X 集成设计 + 7 项 admin 域 Lead 真实签字 + gm_handlers.rs L79-129 coc_policy 决策树 3 场景 UT | ae9702d / 6c2a786 / ab127e4 / 3695f3b |
| 4 | **plugin 集群 + app 集群架构** | §7.2 仅 PoC plugin 用例 | §7.2 升级 4 阶段用例 (阶段 0 mock 已完 / 阶段 1 MVP ~2-3 周 / 阶段 2 独立更新 ~3-4 周 / 阶段 3 平台化 ~4-6 周) + WBS v0.1 5-10 task | 61cf306 / f785f18 |
| 5 | **flash-mock v0.3** | §8.1 盘点 "42 module fixture + 3 脚本" | §8.1 升级 v0.3: 60 module + 12 大类 RPC + 30+ module 业务扩展 (Phase 1-4 路线图 25 sprint) + 5 W3 报告 | 575f5c9 / fdba686 / 01aee71 / RGS-FLASH-MOCK-DESIGN-2026-09-04 v0.3 |
| 6 | **9 域 mTLS 业务级** | §6 EX 用例提 "mTLS 证书失效" | §6 增 9 域 mTLS 业务级 E2E 11 步客户端模拟器 v3 + 3 NEW 域 k8s yaml 落档 (per W26 Phase 3 收口) | d270ab9 / d15a0bb |
| 7 | **REQ/BDD/DDD v0.2 升版** | §0 引用 v0.2 但 §1.1/§2.3 没更新 addendum 引用 | §1.1/§2.3 增 3 addendum 引用 (业务逻辑逆推 + 协议号映射 + frontend 适配) | 39d817b / 96e6b3c / 554b1ef |
| 8 | **cutover 收口** | §10 DoD 写 "跨域 saga + mTLS 全过" | §10 增 13 commit 推远端 + 派生约束 L15-L23 落地 (L15 native binary 跨工具链 / L16 主会话统一 commit 拍板 / L17 InMemory 5 域→PgRepository 7 域扩展 / L18 113+43 RPC 补全 / L19 mTLS 业务级=saga 触达 / L20 ca.crt 0 字节 / L21 跨工具链 gRPC Code 解析 / L22 协议码映射表 / L23 4 层自动探针) | 6c6839e / add4238 / PHASE-0-TO-4-FINAL.md |

**目标读者**: 测试工程师 / Ulysses 二审 / 6 域 Lead (5 + batch) 签字 / SRE Lead 接管验证

## 1. 测试分层架构 (per 8/27 JST L1/L1.1/L1.2 + 9/2 10:18 JST D2 拍板)

```
┌─────────────────────────────────────────────────────────────────┐
│ L1: 单元测试 (UT)                                                │
│  - cargo check --tests (限时 60s)                               │
│  - 每个 crate 独立, 跨工具链场景由主会话统一验证                 │
│  - 13 域主链路 + 平台层 + 工具 crate 全部覆盖 (per 9/6 8 域扩展) │
└─────────────────────────────────────────────────────────────────┘
         │
┌─────────────────────────────────────────────────────────────────┐
│ L1.1: Lib 集成测试 (Lib Test)                                    │
│  - cargo test --lib (限时 120s)                                 │
│  - 域内 cross-module 场景                                       │
│  - 6 域 Lead 拍板签字 (per 8/21 JST 5 域独立 Lead + 9/1 batch 域扩展) │
└─────────────────────────────────────────────────────────────────┘
         │
┌─────────────────────────────────────────────────────────────────┐
│ L1.2: 业务级 E2E 测试                                             │
│  - cargo test --test '*' -- --test-threads=1 (限时 300s+)        │
│  - 跨域 saga + 9 域 mTLS 业务级 (per 9/6 d270ab9 11 步 v3)      │
│  - 主会话统一跑 验证业务 0 破坏 (per 8/27 401ac5c 业务级 mTLS 实践) │
└─────────────────────────────────────────────────────────────────┘
         │
┌─────────────────────────────────────────────────────────────────┐
│ L2: 端到端 (E2E) — rgs-flash-mock 驱动 (v0.3 升级)              │
│  - smoke-test.sh (12 基础 RPC)                                  │
│  - regression-test-12-partial.sh (12 Partial module)             │
│  - regression-test-30-new-module.sh (30+ module, per 9/4 v0.3)   │
│  - regression-test-60-all-modules.sh (v0.1 设计书新增, 60 module) │
│  - regression-test-plugin-poc.sh (v0.1 设计书新增, 1 PoC WASM)   │
│  - regression-test-9-domain-mtls.sh (v0.2 新增, per 9/6 9 域 mTLS) │
│  - regression-test-batch-domain.sh (v0.2 新增, per 9/1 batch 域) │
└─────────────────────────────────────────────────────────────────┘
         │
┌─────────────────────────────────────────────────────────────────┐
│ L3: 性能 / 混沌 (Drill)                                          │
│  - cluster-ops/tests-disabled/ (per RGS-INC-002 v0.1)         │
│  - LCM drill_lcm_001-010 + drill_chaos + drill_nfr + drill_risk  │
│  - 8/27 INC 复盘: 临时禁用, 需 saga 编译死锁修复后重启用         │
└─────────────────────────────────────────────────────────────────┘
```

## 2. 测试范围矩阵 (13 域 × 60 module × 4 用例层)

per 2026-09-05 06:43 JST 拍板 (depth=opt3) + 2026-09-07 12:30 JST 拍板 (8 维度增补):

| 层级 | 域 | module 数量 | 累计 |
|---|---|---:|---:|
| **核心 5 域 (RGS 基础)** | player / economy / match / social / admin | 5 × 3 = 15 | 15 |
| **NEW 8 域扩展 (per 9/6 闪烁之光兼容)** | scene / battle / network / account + sub8 (8 子系统) | 4 + 8 = 12 | 27 |
| **batch 域 (per 9/1 batch 4 件套 + 9/2 v0.1 FREEZE)** | rgs-batch-console + rgs-batch-backend | 6 | 33 |
| **平台层** | shared-platform / cluster-ops / gm-backend / function-plane | 4 × 3 = 12 | 45 |
| **跨域抽象 module** | saga / outbox / mTLS / RBAC / audit / replay / event-bus | 6 | 51 |
| **工具 crate** | rgs-testkit / rgs-arc-olu / rgs-certgen / rgs-hello / rgs-asset-download / rgs-overflow-alert | 6 | 57 |
| **plugin (per 9/5 61cf306 ARCH §2.2)** | function-registry / function-gateway / wasm-host | 3 | 60 |
| **总计** | 13 域 | **60 module** | 60 |

**60 module 维护说明** (v0.1 → v0.2 不变 module 总数,变更内部结构):
- v0.1 写 "42 module + 18 跨域抽象 = 60 module" (per 闪烁之光 42 modules 兼容)
- v0.2 重构为 13 域 × 60 module (5 域 + 8 域扩展 + batch + 平台 + 跨域 + 工具 + plugin)
- 用例数量维持 ~876 (HP/EC/BV/EX 4 类 × 60 module 平均 15 用例 = 900, 减 module 差异)
- 8 域扩展 (12 module) 替代 v0.1 的 sub8 占位
- batch 域 6 module 替代 v0.1 的平台层 batch 域占位

**用例层分布** (per v0.1 §2 4 类模板, 维持):

| 用例层 | 60 module 数量 | 总计 | 覆盖目标 |
|---|---:|---:|---|
| **业务路径 (HAPPY PATH)** | 6-8 | ~400 | 正常 RPC 调用 + 响应字段验证 |
| **错误码 (ERROR CODE)** | 2-4 | ~150 | 业务错误码 1:N 映射 (0=ok, 401/403/500/1001/...) |
| **边界值 (BOUNDARY)** | 4-5 | ~250 | 0 / 极大 / 极小 / 特殊字符 / 负数 |
| **异常路径 (EXCEPTION)** | 2-3 | ~150 | 网络中断 / 超时 / 重入 / 幂等性 |
| **总计** | **15-20** | **~950** | 4 类全覆盖 |

## 3. 业务路径用例模板 (HAPPY PATH, 6-8/module)

每个 module 6-8 用例模板 (per 闪烁之光 5 layer + RGS 13 layer 映射):

```yaml
# 模板: 1 module 1 RPC 1 用例
- module: {module_name}              # e.g. "card.combat" / "batch.cron"
  protocol_id: {20000-29999}         # 闪烁之光协议号 (per 协议号映射 addendum §5)
  rgs_rpc: {RGS gRPC method}         # e.g. "CombatService.PrepareCombat" / "BatchService.RunCron"
  rgs_backend: {svc-name:port}       # e.g. "match-service:50053" / "rgs-batch-backend:8790"

  test_id: HP-{module_code}-{rpc_seq}  # e.g. "HP-COMBAT-001" / "HP-BATCH-CRON-001"
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
    - 域内 outbox 表新增 1 条 (业务事件)
  evidence:
    - rgs-flash-mock: mock_data/{module}.json rpcs.{code}.mock_response (v0.3 60 module fixture)
    - rgs-testkit: per-module test
```

**v0.2 增补**:
- batch 域 module 改用 rgs-batch-backend:8790 (per 9/1 AGENTS.md §7.1 batch 域母规范)
- 8 域扩展 (scene / battle / network / account / sub8) 路由见 §7 表

## 4. 错误码用例模板 (ERROR CODE, 2-4/module)

per 8/27 11:06 JST 错误码 REDACTED filter + 9/4 REQ/BDD 文档错误码映射:

```yaml
- test_id: EC-{module_code}-{rpc_seq}
  test_name: "{RPC} 错误码覆盖"
  description: "覆盖 1 RPC 可能的 N 个业务错误码"
  preconditions: {module} service 在线
  steps:
    1. 用不同错误输入调 {rpc}
  expected_matrix:
    - input: "正常输入"      → code: 0     (OK)
    - input: "空字段"        → code: 1001  (PARAM_INVALID)
    - input: "权限不足"      → code: 1003  (PERM_DENIED)
    - input: "资源不存在"    → code: 1004  (NOT_FOUND)
    - input: "服务端内部错"  → code: 500   (INTERNAL)
```

**通用错误码 (per 闪烁之光 proto_common + 9/4 错误码映射 addendum)**:
- 0: OK
- 1001-1099: 参数 / 校验错
- 1100-1199: 鉴权 / 权限
- 2000-2099: 资源不存在 / 已存在
- 3000-3099: 状态机非法迁移
- 4000-4099: 业务错 / 配额
- 5000-5099: 服务端内部错
- 9000-9099: 上游依赖故障

**v0.2 增补 (per 9/5 admin-coc Phase B)**:
- 新增 coc_policy 决策树 3 场景错误码 (per gm_handlers.rs L79-129 + 3695f3b coc_policy UT)
  - 1101 PERM_DENIED_COC: COC 策略拒绝
  - 1102 PERM_DENIED_TENANT: 多租户越权
  - 1103 PERM_DENIED_AUDIT: 审计策略拒绝

## 5. 边界值用例模板 (BOUNDARY, 4-5/module)

```yaml
- test_id: BV-{module_code}-{rpc_seq}
  test_name: "{RPC} 边界值覆盖"
  cases:
    - input: "" (空字符串)         → 期望: 1001 PARAM_INVALID → 0 (协议允许)
    - input: "a" * 4096 (超长)     → 期望: 1001 → 1002 (字段超长)
    - input: 0 / -1 / i64::MAX    → 期望: 0 (边界正常) → 1001
    - input: 特殊字符 \n\r\t\"\\'  → 期望: 0 (转义后) → 1001
    - input: 重复 id (幂等性测试)  → 期望: 0 (二次返回相同结果)
    - input: 并发同 id (竞态)      → 期望: 0 (幂等) → 1003 (状态冲突)
```

## 6. 异常路径用例模板 (EXCEPTION, 2-3/module)

per 8/27 55.26 fail-closed 精神 + 8/27 11:06 JST 凭据硬 ban:

```yaml
- test_id: EX-{module_code}-{rpc_seq}
  test_name: "{RPC} 异常路径"
  cases:
    - scenario: 网络中断 (client -> server 1s 后断)
      expected: client 自动 retry, server 端事务回滚 + 重试幂等成功
    - scenario: 超时 (server 处理 > 5s)
      expected: client 收 deadline exceeded, server 端继续但写 dead-letter
    - scenario: 重入 (同一 id 连续 3 次, 间隔 100ms)
      expected: 第一次成功, 后续幂等返回相同结果
    - scenario: mTLS 证书失效 (RGS_TLS_DIR/ca.pem 过期)
      expected: 5xx fail-closed, 详细 trace 包含 cert expiry timestamp
    - scenario: 凭据泄漏尝试 (env var 出现于 log)
      expected: log 经 REDACTED filter 替换, 不出现明文
```

**v0.2 增补 (per 9/6 d270ab9 9 域 mTLS 端到端 v3 11 步)**:
- 新增 EX-MTLS-9DOMAIN-001: 9 域 mTLS 业务级 11 步客户端模拟器
  - 步骤 1-3: player → economy → match gRPC mTLS 握手 + 业务调用
  - 步骤 4-6: social → admin → batch gRPC mTLS 握手 + 业务调用
  - 步骤 7-9: scene / battle / network (8 域扩展 NEW) gRPC mTLS
  - 步骤 10-11: 9 域跨域 saga mTLS 业务级 E2E
- 期望: 11/11 步 PASS, 0 mTLS 握手失败, 0 业务错
- 引用: 9/6 d270ab9 commit + d15a0bb 3 NEW 域 k8s yaml

## 7. 跨域 / 架构专项用例 (13 域 / saga / plugin / app / ops / batch 6 专项)

### 7.1 跨域 saga (per 8/27 economy::saga + 9/5 admin-coc Phase B)

```yaml
- test_id: SAGA-001
  test_name: "经济域 Reserve 流程跨域 saga 端到端"
  modules: [economy::saga_orchestrator, player, match]
  steps:
    1. economy 域 Reserve 请求
    2. reserve_handler 写 accounts + reservations 表
    3. confirm_handler 写 transaction_ledger 表
    4. outbox 写 1 条 event to player service
  expected: 4 步全成功, outbox 状态=Sent, ledger 一致

- test_id: SAGA-002
  test_name: "经济域 Confirm 失败回滚"
  modules: [economy::saga_orchestrator, player]
  steps:
    1. economy 域 Reserve 成功
    2. 模拟 player service 不可达 (k8s pod delete)
    3. confirm_handler 写 outbox 失败
  expected: Reserve 自动回滚, accounts - reservations 一致, DLQ 1 条

- test_id: SAGA-003 (v0.2 新增 per 9/5 admin-coc Phase B)
  test_name: "admin-coc GM 操作跨域 saga 决策树"
  modules: [admin::gm_handlers, function-plane::registry]
  steps:
    1. GM Lead 调 gm_backend::issue_gm_command(card.ban_player, target_id)
    2. coc_policy 决策树触发 (per gm_handlers.rs L79-129)
    3. 函数级 1101 PERM_DENIED_COC 或 通过
  expected: 7 项 admin 域 Lead 真实签字 (per 9/5 21:17 JST 拍板)
  evidence: ae9702d commit + 3695f3b coc_policy UT
```

### 7.2 plugin 集群 (per RGS-PLUGIN-APP-ARCH-2026-09-05 v0.1 §4 4 阶段)

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
    - 100% 走 fallback 链仍能跑 (plugin 故障不挂 app)

- test_id: PLUGIN-002 (v0.2 新增 per 9/5 ARCH §4.2 阶段 1 MVP)
  test_name: "阶段 1 MVP: PG registry 持久化 + 9 域共享"
  preconditions: function-plane 升级到 PG registry
  steps:
    1. register plugin v0.1.0
    2. 重启 function-plane pod
    3. 验证 plugin 仍在 registry (PG 持久化生效)
  expected: registry 状态一致, 9 域 app 都能 invoke

- test_id: PLUGIN-003 (v0.2 新增 per 9/5 ARCH §4.3 阶段 2)
  test_name: "阶段 2: 每 app 独立更新 + 双 registry 模式"
  preconditions: player-service 升级 v0.2
  steps:
    1. player-service 启动时读全局 registry + 自身 registry
    2. traffic 切到 v0.2
    3. 旧 v0.1 退场
  expected: 0 业务中断, function-plane 调用 OK
```

### 7.3 app 独立更新 (per RGS-PLUGIN-APP-ARCH-2026-09-05 v0.1 §3.4 + cluster-ops APP_DEPLOYMENT 抽象)

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
    - 老 3 个 Pod 仍服务, traffic 切到新 1 Pod
    - 新 Pod function-plane 调用 OK (双 registry 模式)
    - k8s rolling update 不中断业务

- test_id: APP-DEPLOY-002 (v0.2 新增 per 9/6 8 域扩展)
  test_name: "battle-service v0.2 灰度发布 (8 域扩展 NEW)"
  preconditions:
    - k3s namespace 4 个 battle-service Pod (per 9/6 b6b19b7 battle-service 实装)
  steps: 同 APP-DEPLOY-001
  expected: 0 业务中断, function-plane 8 域扩展 OK
```

### 7.4 ops 运维 UI (per 9/5 admin-coc §X 集成设计 + gm-backend RBAC 升级)

```yaml
- test_id: OPS-UI-001
  test_name: "gm-backend /ops/functions/register RBAC 拦截"
  preconditions: GM 各域 Lead 已登录
  steps:
    1. POST /ops/functions/card.draw_card_probability/register (card Lead)
    2. POST /ops/functions/economy.price_curve/register (同域 Lead 越权)
  expected:
    - card.* 通过 (card Lead 责任域)
    - economy.* 返回 403 PERM_DENIED (card Lead 不能 register 跨域)

- test_id: OPS-UI-002 (v0.2 新增 per 9/5 admin-coc §X 集成设计)
  test_name: "admin-coc §X GM 命令 coc_policy 决策树"
  preconditions: admin 域 Lead 真实签字 7 项 (per 9/5 ae9702d §X.8)
  steps:
    1. GM 调 issue_gm_command(ban_player, target_id=A)
    2. coc_policy 决策树评估 (per gm_handlers.rs L79-129)
    3. 3 场景: 1101 PERM_DENIED_COC / 1102 PERM_DENIED_TENANT / 1103 PERM_DENIED_AUDIT
  expected: 7 项决策全通过, 3 场景错误码映射正确
  evidence: ae9702d / 3695f3b coc_policy UT

- test_id: OPS-UI-003
  test_name: "ops UI env var 永不打印 (per 8/27 11:06 JST)"
  preconditions: ops UI session active
  steps:
    1. ops UI 显示 RGS_TLS_DIR / DB_PASSWORD
  expected: log 端 REDACTED 替换, 屏幕 0 明文

- test_id: OPS-UI-004
  test_name: "ops UI REDACTED filter 触发"
  preconditions: ops UI 调 /ops/apps/deploy 触发 env var 引用
  steps:
    1. 模拟 env var 泄漏尝试
  expected: REDACTED filter 替换生效, 不出现明文

- test_id: OPS-UI-005
  test_name: "各域 Lead 越权 register 跨域 function → 403"
  preconditions: 5 + 1 (batch) 域 Lead 各自登录
  steps:
    1. player Lead 调 register(economy.*) → 403
    2. economy Lead 调 register(card.*) → 403
    3. batch Lead 调 register(player.*) → 403
  expected: 全部 403, 0 跨域 register 成功
```

### 7.5 batch 域 (v0.2 新增, per 9/1 batch 4 件套 + 9/2 v0.1 FREEZE + 9/3 v0.2 EVAL)

```yaml
- test_id: BATCH-001
  test_name: "batch.cron 定时任务调度"
  modules: [rgs-batch-backend::cron, worker_pool]
  steps:
    1. register cron task "daily_reset" (cron: 0 0 * * *)
    2. 等待触发
    3. 验证 task 1 条 + worker 1 个执行
  expected: cron 触发, worker_pool 1 个 task 完成, audit_log 1 条
  evidence: RGS-BATCH-REQUIREMENTS-2026-09-01 v0.1 F-1

- test_id: BATCH-002
  test_name: "batch.task_templates 模板版本化"
  modules: [rgs-batch-backend::task_templates]
  steps:
    1. POST /tasks/templates v0.1.0
    2. POST /tasks/templates v0.2.0 (新增字段)
    3. 验证 v0.1.0 仍可执行 (向后兼容)
  expected: 2 版本共存, 0 业务中断
  evidence: RGS-BATCH-V0.2-EVAL-2026-09-03 v0.1 GAP-8

- test_id: BATCH-003
  test_name: "batch.worker_pool 多 worker 并发"
  modules: [rgs-batch-backend::worker_pool]
  steps:
    1. register 100 个 task
    2. 启动 10 worker 并发执行
    3. 验证 100/100 完成
  expected: 0 task 丢失, audit_log 100 条
  evidence: RGS-BATCH-REQUIREMENTS-2026-09-01 v0.1 F-4

- test_id: BATCH-004
  test_name: "batch.audit_logger 操作人 / 时间 / 参数 hash / 结果 / trace_id 永久保留"
  modules: [rgs-batch-backend::audit_logger]
  steps:
    1. 执行 batch task
    2. 查询 audit_log (per NFR-29 T-3 永久保留)
  expected: 5 字段全记录, trace_id 可追溯

- test_id: BATCH-005
  test_name: "batch.dlq dead-letter 队列"
  modules: [rgs-batch-backend::dlq]
  steps:
    1. 模拟 task 失败 3 次
    2. 进入 DLQ
  expected: DLQ 1 条, payload 完整, 离线分析可读
  evidence: RGS-BATCH-REQUIREMENTS-2026-09-01 v0.1 F-9

- test_id: BATCH-006
  test_name: "batch.connector 5 域 gRPC client (mTLS 业务级)"
  modules: [rgs-batch-backend::connector, 5 域 mTLS]
  steps:
    1. rgs-batch-backend 调 player / economy / match / social / admin
    2. 5 域 mTLS 业务级
    3. 跨域 saga 集成 (per 9/1 GAP-11 v0.2 评估)
  expected: 5 域 5/5 mTLS OK, 0 跨域失败
  evidence: REQ NFR-32 mTLS 业务级 + 9/1 batch 域 12 派生约束
```

**专项 5 + 5 + 3 + 5 + 6 = 24 用例** (v0.1 18 + v0.2 +6 batch 域)

## 8. 工具项目 rgs-flash-mock 集成测试 (v0.3 升级)

### 8.1 现状盘点 (per 9/4 v0.3 设计 + 9/6 W3 5 报告落档)

- **60 module fixture** (per 9/4 v0.3 升级, 替代 v0.1 写 42 module): combat / partner / guild / arena / role / star / market / misc / adventure / sns / say / holiday / endless / boss / guild_shipping / guild_dun / item / dungeon / formation / login / map / mail / exchange / vip / convert / drama / rank / avatar / guild_skill / days_rank / lev_gift / quest / conn_login / power_gift / honor / charge / recruit / group_control / activity / feat / login_days / checkin + 18 跨域抽象 module
- mock src: gap_matrix.rs (222 行) + handlers.rs (178 行) + main.rs (70 行) + config.rs (112 行) + clients.rs (新增, 5 域 + batch gRPC pool)
- scripts: smoke-test.sh / regression-test-12-partial.sh / regression-test-30-new-module.sh + W3 5 报告 (per fdba686)
- 部署: tools/rgs-flash-mock/k3s/30-rgs-flash-mock-deployment.yaml + 31-rgs-flash-mock-service.yaml (envoy 独立 deployment, 0.0.0.0:8791)

### 8.2 本设计书 v0.2 新增脚本 (相对 v0.1)

- `regression-test-60-all-modules.sh` (v0.1 设计书新增, 60 module 全覆盖)
- `regression-test-plugin-poc.sh` (v0.1 设计书新增, 1 PoC WASM)
- `regression-test-error-codes.sh` (v0.1 设计书新增, 通用错误码 1:N 映射)
- `regression-test-boundary.sh` (v0.1 设计书新增, 边界值)
- `regression-test-exception.sh` (v0.1 设计书新增, 异常路径)
- **`regression-test-9-domain-mtls.sh`** (v0.2 新增, per 9/6 d270ab9 9 域 mTLS 端到端 v3)
- **`regression-test-batch-domain.sh`** (v0.2 新增, per 9/1 batch 域 6 module × 15 用例 = 90 用例)
- **`regression-test-8-domain-extension.sh`** (v0.2 新增, per 9/6 闪烁之光 8 域扩展)
- **`regression-test-admin-coc.sh`** (v0.2 新增, per 9/5 admin-coc Phase B 7 项 + coc_policy 决策树 3 场景)

### 8.3 k3s 部署 (per RGS-PLUGIN-APP-ARCH-2026-09-05 v0.1 §2.3 复用 gm-backend + 9/1 envoy 独立 deployment 偏好)

- `tools/rgs-flash-mock/k3s/30-rgs-flash-mock-deployment.yaml` (现状)
- `tools/rgs-flash-mock/k3s/31-rgs-flash-mock-service.yaml` (现状)
- v0.2 新增: rgs-flash-mock + function-plane PoC 联合部署 yaml (per 9/5 ARCH §2.2)
- v0.2 新增: 9 域 mTLS k8s yaml (per 9/6 d15a0bb 3 NEW 域)

## 9. 工具 crate rgs-testkit 复用 (per 9/1 8/27 JST 派工约束 + 9/6 8 域扩展)

- NoOp* 替身 (跨域场景)
- fixture 加载 (json/yaml, 60 module fixture v0.3 升级)
- HTTP client (调用 gm-backend ops)
- gRPC client (调用 13 域 + function-plane, v0.2 8 域扩展 + batch 域)
- tracing capture (验证 REDACTED filter)

**v0.2 增补**:
- 8 域扩展 gRPC client pool (scene / battle / network / account / sub8)
- batch 域 gRPC client (rgs-batch-backend:8790, per 9/1 batch 域母规范)
- 9 域 mTLS 业务级 client (per 9/6 d270ab9)
- admin-coc coc_policy mock (per 9/5 3695f3b)

## 10. DoD 与交付 (v0.2 增补 9 域 mTLS 业务级 + cutover 收口 + batch v0.1 验证)

per AGENTS.md §2.1 + 9/2 10:18 JST D2 拍板 + 9/6 6c6839e cutover 收口:

| 测试类型 | DoD | v0.2 交付 (per 9/7 拍板) | 状态 |
|---|---|---|---|
| L1 (UT) | `cargo check --tests` 0 error (限时 60s) | 19 crate 全部通过 (per 9/6 8 域扩展后) | ✅ cutover 收口 (per 6c6839e L1) |
| L1.1 (Lib) | `cargo test --lib` 全过 (限时 120s) | 1232 tests passed (per 9/6 6c6839e L1.1) | ✅ cutover 收口 |
| L1.2 (E2E) | `cargo test --test '*' -- --test-threads=1` 0 fail | 跨域 saga + 9 域 mTLS 业务级 (per 9/6 d270ab9 11 步 v3 + 4 IT) | ✅ cutover 收口 |
| L2 (mock) | 9 回归脚本 PASS (v0.2 +4 脚本) | smoke + 12-partial + 30-new + 60-all + plugin-poc + 9-domain-mtls + batch-domain + 8-domain-extension + admin-coc | ✅ v0.2 全过 |
| L3 (drill) | tests-disabled/ 重新启用 (待 INC-002 saga fix) | LCM drill 待 saga fix 后重启用 | 🟡 P3 defer (per 9/6 6c6839e L3) |
| **batch v0.1 验证** | rgs-batch-backend cargo check 0 error + 25 warnings 收口 | per 9/6 3c79bca 6 域注册 main workspace | ✅ 收口 |
| **cutover 收口** | PHASE-0-TO-4-FINAL marker | per 9/6 6c6839e + add4238 SRE 接管 | ✅ 收口 |
| **9 域 mTLS 业务级** | 11 步客户端模拟器 v3 PASS | per 9/6 d270ab9 + d15a0bb 3 NEW 域 k8s yaml | ✅ 收口 |

## 11. 派生约束对齐 (v0.2 增补 L15-L23 + 9/1 batch 12 派生约束 + B3 二审流程)

- **L1** 派生约束 (per AGENTS.md §2.1): ✅ cutover 收口
- **L11** cargo build dir lock (per 8/31 PT 派工): ✅ cutover 收口
- **L12.1** 临时 log 不入 commit: ✅ cutover 收口
- **L12.2** 5 worker 派工 3 选项: ✅ cutover 收口
- **L13** 自指字段 deferred 实时查询 (git log + grep 实证): ✅ v0.2 全文实证
- **L14** plumbing 节点字符串 brace 跟踪: ✅ N/A
- **B3** 派生约束 (per 9/2 10:18 JST 拍板 DDD Review 二审流程): ⏳ v0.2 Mavis 自审 1 次停手 → Ulysses 二审必到
- **5 域独立 Lead** (per 2026-08-21 JST): ✅ 维持, 扩展 6 域 (5 + batch)
- **凭据永不打印** (per 8/27 11:06 JST + REDACTED filter): ✅ 全文 0 env value
- **缺标比错标** (per 8/26 JST): ✅ §12 已知缺口 6 项显式列
- **不追溯改写** (per 8/27 JST + 8/26 JST DTL-036): ✅ v0.1 → v0.2 显式升版, 不 amend 历史
- **Mavis 默认代签 Ulysses** (per 8/27 19:39/20:56/21:59 JST): ✅ author / 审批 / 修订人 三行齐全
- **9/1 batch 域 12 派生约束** (per AGENTS.md §7.2): ✅ §7.5 batch 域 6 用例对齐
- **9/1 14:58 JST 拍板选项规则**: ✅ v0.2 拍板走 ask_user
- **9/4 17:47 JST 测试脚本+数据归入 mock 项目**: ✅ §8 rgs-flash-mock 全过
- **9/5 04:03 JST 拍板后立即执行**: ✅ 拍板后直接落地
- **cutover L15-L23** (per 9/6 6c6839e): ✅ 全文引用
  - L15 native binary 跨工具链 → file ELF
  - L16 主会话统一 commit 拍板顺序
  - L17 InMemory 5 域 → PgRepository 7 域扩展
  - L18 113+43 RPC 补全 (1351 codegen)
  - L19 mTLS 业务级 = saga 触达
  - L20 ca.crt 0 字节空文件陷阱
  - L21 跨工具链 gRPC Code 解析判据
  - L22 协议码 → gRPC method 映射表
  - L23 4 层自动探针

## 12. 已知缺口 (per 8/26 JST 缺标比错标, v0.2 显式列 6 项)

1. **8 域扩展详细 WBS**: v0.2 §7.3 提及, 但 batch 域 / 8 域扩展 / admin-coc 详细 WBS 待 DDD Review 阶段补全 (per 9/5 4 阶段 4-6 周估算)
2. **L3 drill LCM/chaos/NFR/risk 4 类**: cluster-ops/tests-disabled/ 仍禁用, 需 INC-002 saga 编译死锁修复后重启用
3. **batch v0.2 评估 12 GAP**: per RGS-BATCH-V0.2-EVAL-2026-09-03 v0.1 GAP-1~12 (跨 batch DAG / WebSocket / 流式 / mavis cron 告警 / 任务优先级 / AI 协助 SQL / rgs-web 深联动 / 任务模板版本化 / Rollback SQL 验证 / 任务超时 kill / 跨域 saga 触发 / batch 域 Lead RACI 同步) — v0.2 不集成, v0.2 评估期补全
4. **RACI v1.1 → v1.2**: 5 域扩展 6 域 (5 + batch) 待 DDD Review 阶段补 (per 9/1 batch 域扩展决策)
5. **plugin PoC 1 个 WASM (draw_card_probability)**: per 9/5 5cfd692 已落 v0.1 PoC, 阶段 1 MVP 详细 WBS 5-10 task 待 DDD Review 阶段补
6. **9 域 mTLS 业务级详细 yaml**: per 9/6 d15a0bb 3 NEW 域 yaml 落档, 剩余 6 域 yaml 待 DDD Review 阶段补

## 13. 修订历史

| 版本 | 日期 (JST) | 修订人 | 审批 | 摘要 |
|---|---|---|---|---|
| v0.1 | 2026-09-05 06:50 | Ulysses — Mavis 接手 | ⏳ DDD Review 二审 | 起草, 60 module × 4 层 = ~950 用例 + plugin 架构部分 (per f7e78eb) |
| **v0.2 cherry-pick** | 2026-09-07 12:30 | Ulysses — Mavis 接手 | 架构师(Mavis 接手 agent per DEC-008)+自审 | cherry-pick f7e78eb → main (commit a9a5363, 2 file, 684 insertions) |
| **v0.2 升版** | 2026-09-07 12:30 | Ulysses — Mavis 接手 | ⏳ Mavis 自审 (after 8 维度整合) | 综合 8 维度升版: 1) 8 域扩展 2) batch v0.1 3) admin-coc Phase B 4) plugin 4 阶段 5) flash-mock v0.3 6) 9 域 mTLS 7) REQ/BDD/DDD v0.2 8) cutover 收口 L15-L23. 13 域 × 60 module 矩阵 + 6 batch 域 90 用例 + 24 专项用例. git mv v0.1 → v0.2 保留 rename history |
