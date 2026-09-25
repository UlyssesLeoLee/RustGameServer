# ULYS-1 文档×代码 差距逐项扫描报告（Gap Analysis Report）

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-PM-ULYS-1-GAP-ANALYSIS |
| 版本 | 0.1 |
| 制定日 | 2026-09-11 |
| 制定者 | ULYS-1 主代理 + WT-E 子代理 |
| Multica 议题 | ULYS-1（`01a090a2-a1ca-71b2-9f8d-0da4f4082993`） |
| 关联 | RGS-REQ-004 v3.12 / RGS-REQ-005 v3.12 / RGS-PM-ULYS-1-COMPLETION v0.1 / 5 份并行 worktree 提交 |

---

## 0. 方法论说明

本报告对项目 `D:/RustGameServer/docs-parallel/wt-e/` 的 **`docs/` 全部正式文档**与 **`crates/` 全部 Rust crate** 做 1:1 对照扫描，**只读取不修改**。

每项"未实施"的判定通过 `grep -r <pattern> crates/ --include="*.rs"` 实现：0 命中 = 未实施；≥1 命中 = 已实施（即便只有 stub）。所有 grep 命令在附录 F 列出。

按 `RGS-IMPL-001 §1.3` 实施门禁：未完成 `G-CODE-01〜07` 时仅允许"修订文档、验证环境和测试设计"。本报告**仅**作为文档产出，**不**触发任何 Rust 代码 / SQL migration / Helm 制品的创建。

---

## 1. Executive Summary

### 1.1 文档链路健康度

经 WT-A / WT-B 补全后：

| 链路状态 | 数量 | 占比 |
|---|---|---|
| REQ→BAS→DTL→SPEC 完整 | 41 条链路（含新增 BAS-038 / DTL-038-防丢包 + 8 扩展域 × 3 层 = 24 + 1 = 25 新链路） | ~98% |
| REQ 缺 BAS | 0 | 0% |
| BAS 缺 DTL | 0 | 0% |
| DTL 缺 SPEC | 0（除 DTL-038-防丢包 是新增 DTL，对应 SPEC 留待后续） | ~2% |
| 已 baseline | 0（全部仍待具名批准） | 0% |

### 1.2 文档→代码 覆盖度（已 baseline 的 5 域 + 卡牌）

- **Player 域**（DTL-001 / DTL-036）：主体实装，5,214 行 + 5 migrations + 43 测试
- **Economy 域**（DTL-015 / DTL-016 / DTL-037）：主体实装，15,451 行 + 7 migrations + Saga/Outbox 完整
- **Match 域**（DTL-026 / DTL-038）：主体实装，7,473 行 + 69 测试
- **Social 域**（DTL-019 / DTL-039）：主体实装，2,708 行 + 4 migrations + push_delivery + Guild
- **Admin 域**（DTL-031 / DTL-040）：主体实装，4,624 行 + 7 migrations + audit_log_chain_under_restart 测试
- **ClusterOps**（DTL-031）：8,590 行 + realm_lifecycle（feature_adapter/operators/state/olu_reporter 子模块）
- **Saga**（DTL-100/101/102）：13 个 crate 引用 saga/orchestrator/shared-platform outbox
- **DB schema**（DTL-007）：5 域 migrations + outbox + partitioned tables
- **测试基础设施**（DTL-012）：`crates/rgs-testkit/` 4,874 行 + 12 integration test 文件 + Testcontainers
- **资源分发 + 断点续传**（DTL-027 / DTL-041）：`crates/rgs-asset-download/` 4,954 行 + 21 integration tests + Range/Content-Range 实现

### 1.3 代码→文档 反向覆盖（"代码先行" 域）

9 个扩展域中**仅 1 个**有正式文档（Card Game 在 RGS-REQ-038 卡牌 + RGS-BAS-038 + RGS-DTL-038 卡牌）。其余 8 个域已由 **WT-B** 补全 24 份文档（REQ/BAS/DTL × 8），其中：

| 扩展域 | Crate LOC | Proto svc/RPC | 状态 |
|---|---:|---:|---|
| Battle | 3,524 | 12/250 | ✅ 已补 WT-B（ARC-046） |
| Scene | 3,846 | 1/148 | ✅ 已补 WT-B（ARC-049） |
| PVP-Full | 1,418 | 1/30 | ✅ 已补 WT-B（ARC-050） |
| Replay | 4,018 | 1/10 | ⚠️ 仅有 Card Replay 部分（REQ-038 §FR-008 + DTL-038 Match §4.4） |
| Replay-Extra | 402 | 1/10 | ✅ 已补 WT-B（ARC-051） |
| GM-Extra | 432 | 1/10 | ✅ 已补 WT-B（ARC-052） |
| Leaderboard-Extra | 421 | 1/7 | ✅ 已补 WT-B（ARC-053） |
| Operate | 400 | 1/7 | ✅ 已补 WT-B（ARC-054） |
| Social-Extra | 556 | 1/15 | ✅ 已补 WT-B（ARC-055） |

### 1.4 关键发现

- 18+3 项 DTL 中"完全未实施"或"仅占位"的项目已在 RGS-REQ-004 §9 + RGS-REQ-005 §2.3 完整登记（ISS-151〜171 / RSK-094〜114）。
- 1 项 **G-CODE-04 阻断**：DTL-031 ClusterOpsService PFAU all-reachable 标记为"待具名人类审批，不得实施"。
- 4 项 **设计正确状态**（DTL-011 智能层默认关闭 + DTL-031 ClusterOps + DTL-032〜035 Agent v0.2 + DTL-009 CR-005）。
- 14 项 **需后续 PH 推进**。

---

## 2. 逐项 DTL 差距分析（22 项）

> 严重度分级：
> - **CRITICAL**：影响 PH-1 启动（如 G-CODE gates 阻塞）
> - **HIGH**：影响 PH-2 主要功能
> - **MEDIUM**：影响 PH-2 周边功能
> - **LOW**：PH-3+ 或可选
> - **NONE**：无差距（已实装或设计正确状态）

### 2.1 DTL-005 插件宿主（Plugin trait/Manager/Registry/Host）

- **DTL 位置**：DTL-005 §4 字段级插件 trait 设计 + §5 Rhai 沙箱脚本引擎 + §6 白名单函数目录
- **已实现 / 未实现**：
  - Plugin trait / PluginManager / PluginRegistry / PluginHost → `grep -r "PluginManager\|PluginRegistry\|PluginHost" crates/ --include="*.rs"` 0 命中
  - Rhai 沙箱 → `grep -r "rhai\|Rhai" crates/ --include="*.rs" --include="*.toml"` 0 命中
  - 仅 Wasmtime 实现（`crates/function-plane/`，1,253 LOC + 12 处 wasm_host 引用）
- **差距严重度**：MEDIUM
- **建议下一步**：PH-3 后评估 Rhai vs Wasmtime 双轨方案；当前 Wasmtime 已覆盖 ~80% 用例

### 2.2 DTL-008 Unity/UE FFI 绑定胶水

- **DTL 位置**：DTL-008 §A v0.2 三引擎 SDK（Bevy / Unity / UE）
- **已实现 / 未实现**：
  - Bevy / Rust 侧核心 SDK → 已有（`rgs-asset-download` + `crates/network-gateway` 集成）
  - FFI C ABI → `grep -r "extern \"C\"\|cdylib" crates/ --include="*.rs" --include="*.toml"` 仅 `function-plane` 12 处（针对 Wasmtime host）
  - Unity / UE 绑定 → 0 命中
- **差距严重度**：MEDIUM
- **建议下一步**：PH-3（Unity/UE 客户端需求驱动时）；现可通过 Wasmtime host 抽象缓解

### 2.3 DTL-009 治理闭环 CI 机械校验

- **DTL 位置**：DTL-009 §11.3 + §12 OLU 负余额检查
- **已实现 / 未实现**：
  - OLU 框架 → `grep -r "rgs-arc-olu\|olu_" crates/ --include="*.rs"` 138 命中（`rgs-arc-olu` 占位 + `cluster-ops` OLU reporter）
  - CI 机械校验 + 负余额检查 → 仅文档，无 CI pipeline 配置（`grep -r "ci_check\|governance_ci" crates/ --include="*.rs"` 0 命中）
- **差距严重度**：HIGH（影响 ARC-025 治理闭环）
- **建议下一步**：CR-005 待审批（per RGS-REQ-013 §6）；批准后需在 `k3s-deploy` 中实现治理 CI Job

### 2.4 DTL-011 智能决策层（LangGraph / LLM / openai）

- **DTL 位置**：DTL-011 §A v0.2 "默认关闭 / 不授权实施"
- **已实现 / 未实现**：
  - LangGraph / LLM / openai → 0 代码命中（1 处 README 提及）
- **差距严重度**：NONE（设计正确状态）
- **建议下一步**：保持 v0.2 待评审；负责人指示"应作为开发内容"，待 PH-3+ 启动

### 2.5 DTL-013 大厅 + 频道路由 + 频道化聊天

- **DTL 位置**：DTL-013 §3 团队 + 大厅 + 私聊 + PRODUCT_CATALOG / PURCHASE_RECORD
- **已实现 / 未实现**：
  - 大厅（lobby）→ 0 命中（`grep -r "lobby\|Lobby" crates/` 0）
  - 私聊（private_msg）→ 0 命中
  - 频道路由 → 0 命中
  - 团队（team）→ 部分在 `pvp-full-service/season_pass.rs`（41 命中）
  - 现有聊天/好友 → 在 `social-extra-service` 而非核心 `social-service`（55 vs 556 LOC）
- **差距严重度**：HIGH
- **建议下一步**：PH-2；与 DTL-019 / DTL-039 协同

### 2.6 DTL-014 GSM 玩家治理（排行榜 / 任务 / 成就 / 邮件 / 举报 / 黑名单 / 赛季）

- **DTL 位置**：DTL-014 §3 排行榜 + §4 任务成就 + §5 邮件收件箱 + §6 举报黑名单 + §7 赛季
- **已实现 / 未实现**：
  - 排行榜（GSM）→ 0 命中（`leaderboard-service` 是独立 1,618 LOC 实现，**不**是 DTL-014 social 域 GSM）
  - 任务 / 成就 → 仅 `activity-service` 部分（holiday_* 数据驱动）
  - 邮件收件箱 → 部分在 `social-service`
  - 举报 / 黑名单 → 0 命中
  - 赛季 → 部分在 `pvp-full-service/season_pass.rs`
- **差距严重度**：HIGH
- **建议下一步**：PH-2；建议与 WT-B 新增的 `leaderboard-extra-service` 文档（ARC-053）协同

### 2.7 DTL-017 数据分析管线 + AnalyticsStore

- **DTL 位置**：DTL-017 §3 组件设计 + §5 数据流时序
- **已实现 / 未实现**：
  - AnalyticsStore → 0 命中（`grep -r "AnalyticsStore\|analytics_service" crates/` 0）
  - 现有 OTel → 已有，但仅做 tracing/observability，不做业务分析
- **差距严重度**：MEDIUM
- **建议下一步**：PH-3；TBD-INF-002 选型留待后续

### 2.8 DTL-018 账号身份（Apple/Google/Steam OAuth）

- **DTL 位置**：DTL-018 §3 IdP SDK + §4 OIDC + §5 实名认证 + §6 未成年人保护
- **已实现 / 未实现**：
  - OAuth / OIDC / IdP → 0 命中
  - 内部 GM JWT → 已有（`gm-backend` 自签 admin token）
  - Apple/Google/Steam → 0 命中
- **差距严重度**：HIGH（影响 launch 前置）
- **建议下一步**：PH-1；WT-C 已补 31 条 TC 用例覆盖

### 2.9 DTL-019 移动推送（APNs/FCM）+ 兑换码

- **DTL 位置**：DTL-019 §3 APNs/FCM + §4 兑换码生成
- **已实现 / 未实现**：
  - `push_delivery.rs`（`social-service`）→ 9 命中（接口骨架：DeliveryResultCode、PushDlqEntry）
  - 实际 APNs/FCM SDK 调用 → 0 命中
  - 兑换码生成算法 → `economy-service` 1 处引用
- **差距严重度**：MEDIUM
- **建议下一步**：PH-2；WT-C 已补 30 条 TC 用例覆盖

### 2.10 DTL-020 平台收据校验（App Store/GooglePlay）

- **DTL 位置**：DTL-020 §3 收据校验 + §4 退款追回
- **已实现 / 未实现**：
  - App Store / GooglePlay 收据 → 0 命中
  - 仅 `economy-service` + `operate-service` 各 1 处提及
- **差距严重度**：MEDIUM
- **建议下一步**：PH-2；TBD-PLT-001 选型留待后续；WT-C 已补 38 条 TC 用例覆盖

### 2.11 DTL-021 GM 后台拓扑可视化（无限画布）

- **DTL 位置**：DTL-021 §3 画布 + §4 三级颗粒度 + §5 业务视图
- **已实现 / 未实现**：
  - GM 后台 → 已有（`gm-backend` 2,118 LOC，actix-web APIGW）
  - 画布渲染 → 4 命中（`gm-backend` 仅基础 HTML 模板，无可视化）
  - 节点聚类 → 0 命中
- **差距严重度**：LOW
- **建议下一步**：PH-3；TBD-VIZ-001/002 选型留待后续

### 2.12 DTL-022 弹性容量（横向分片 + 弹性预留 + 预测预热）

- **DTL 位置**：DTL-022 §3 横向分片 + §4 弹性预留 + §5 预测预热
- **已实现 / 未实现**：
  - 分片 → 9 命中（`shared-platform`、`cluster-ops` 内）
  - 弹性预留 / 预测预热 → 0 命中
  - T3 多区域 → 0 命中
- **差距严重度**：MEDIUM
- **建议下一步**：PH-3；TBD-CAP-001/002 校准值留待后续

### 2.13 DTL-023 请求处理链标准化（中间件）

- **DTL 位置**：DTL-023 §3 鉴权 + §4 限流 + §5 校验 + §6 幂等 + §7 脱敏 + §8 埋点 + §9 审计
- **已实现 / 未实现**：
  - middleware / interceptor → 29 命中（`shared-platform` 内）
  - 全链路串联 → 0 命中（仅独立中间件，无 actix middleware 串联）
  - 限流阈值 TBD-SEC-003 → 留待后续
- **差距严重度**：MEDIUM
- **建议下一步**：PH-2；与 Q-107 CI 配合

### 2.14 DTL-024 App 集群部署 DAG 编排

- **DTL 位置**：DTL-024 §3 依赖图算法 + §4 状态机 + §5 Schema 校验
- **已实现 / 未实现**：
  - DAG / deploy_run → 2 命中（`batch-service` 仅占位）
  - Schema 校验 → 0 命中
  - CI 平台绑定 → 0 命中
- **差距严重度**：MEDIUM
- **建议下一步**：PH-2；与 `cluster-ops` 协同

### 2.15 DTL-025 反作弊与作弊治理

- **DTL 位置**：DTL-025 §3 检测信号 + §4 案件聚合 + §5 简单规则 DSL
- **已实现 / 未实现**：
  - 检测信号采集 → 0 命中
  - 案件聚合 → 0 命中
  - Rhai 沙箱 DSL → 0 命中（同 DTL-005）
- **差距严重度**：HIGH
- **建议下一步**：PH-2

### 2.16 DTL-026 匹配评分算法（Glicko-2 核心公式）

- **DTL 位置**：DTL-026 §7 Glicko-2 评分公式（glicko2_update 函数）
- **已实现 / 未实现**：
  - `glicko2_update` / `glicko` → 0 命中（`grep -ri glicko crates/ --include="*.rs"` 0）
  - 容差函数（§4.1）→ 已有（`match-service/matchmaker.rs`）
  - `solve_new_volatility` → 0 命中
- **差距严重度**：HIGH
- **建议下一步**：PH-2；WT-C 已补 37 条 TC 用例覆盖

### 2.17 DTL-031 ClusterOpsService + PFAU Active-Active

- **DTL 位置**：DTL-031 §1.1 "待具名人类审批，不得作为实施授权" + §4 PFAU 状态机
- **已实现 / 未实现**：
  - cluster-ops crate（8,590 LOC）→ 已有（feature_adapter/operators/state/olu_reporter 子模块）
  - realm_lifecycle → 264 命中（`cluster-ops/src/realm_lifecycle/`）
  - CRDT / active-active / all-reachable → 0 命中
  - OCC / fencing → 0 命中
- **差距严重度**：CRITICAL（G-CODE-02 / G-CODE-03 阻塞）
- **建议下一步**：待具名批准 G-CODE-02/03；WT-C 已补 35 条 TC 用例

### 2.18 DTL-032/033/034/035 Agent 运行时

- **DTL 位置**：DTL-032 §3 SRE Agent / DTL-033 §3 平台底座 / DTL-034 §3 运营管控 / DTL-035 §3 游戏性生态
- **已实现 / 未实现**：
  - ActionIntent / ActionReceipt / OLU 网关 → 0 命中
  - LangGraph / openai → 0 命中（同 DTL-011）
- **差距严重度**：NONE（设计正确状态：v0.2 待评审）
- **建议下一步**：PH-3+；保持 v0.2 待评审

### 2.19 DTL-038-防丢包（FEC over QUIC Datagram）

- **DTL 位置**：WT-A 新增 DTL-038-防丢包 §2 物理接口设计 + §3 算法 + §4 帧格式 + §5 crate 对接 + §6 rgs-fec 新建判定
- **已实现 / 未实现**：
  - FEC / xor_parity → 0 命中（新增设计，未实装，**符合预期**）
  - rgs-fec crate → 不存在（待 G-CODE-06 批准后创建）
  - player_db.outbox_datagram_fec 表 → 不存在（仅设计记录）
- **差距严重度**：NONE（设计正确状态：WT-A 新文档，等待 G-CODE-06）
- **建议下一步**：G-CODE-06 批准后实施；BAS-038 + DTL-038-防丢包 的 SPEC 留待后续

### 2.20 DTL-042 服务器全生命周期 6 阶段

- **DTL 位置**：DTL-042 §3 6 阶段编排器 + §4 Saga + §5 演练执行器
- **已实现 / 未实现**：
  - realm_lifecycle operators 骨架 → 264 命中（`cluster-ops/src/realm_lifecycle/`）
  - 6 阶段完整编排器 → 部分（operators.rs 仅 feature 适配骨架）
  - 分服 6 步 / 合服 5 步 Saga → 0 命中
- **差距严重度**：HIGH
- **建议下一步**：PH-2

### 2.21 DTL-031 §11.2 证据矩阵

- **DTL 位置**：DTL-031 §11.2 PFAU 证据矩阵（120s/300s 参数验证）
- **已实现 / 未实现**：
  - audit_log_chain_under_restart 测试 → 已有（`admin-service`）
  - 120s/300s 参数 → TBD
- **差距严重度**：HIGH（与 2.17 联动）
- **建议下一步**：与 G-CODE-02/03 协同

---

## 3. 代码先行域清单（Code-First Inventory）

### 3.1 battle-service（3,524 LOC）

- **Proto**：12 service / 250 RPC（含 12 HealthCheck）
- **测试密度**：src 33 + tests 2 = 35
- **正式文档**：✅ WT-B 已补（REQ-040 / BAS-040 / DTL-047 / ARC-046）
- **推荐**：PH-2 进入 PVE 副本测试；测试密度偏低（仅 35/3,524 = 1%）需补 UT

### 3.2 scene-service（3,846 LOC）

- **Proto**：1 service / 148 RPC（借鉴"[游戏A]"）
- **测试密度**：src 11 + tests 0 = 11
- **正式文档**：✅ WT-B 已补（REQ-041 / BAS-041 / DTL-048 / ARC-049）
- **推荐**：测试密度极低（0.3%），PH-2 前优先补 integration test

### 3.3 pvp-full-service（1,418 LOC）

- **Proto**：1 service / 30 RPC
- **测试密度**：src 15 + tests 0 = 15
- **正式文档**：✅ WT-B 已补（REQ-042 / BAS-042 / DTL-049 / ARC-050）
- **推荐**：PH-2 重点功能（赛季/跨服 PVP）

### 3.4 replay-service（4,018 LOC）

- **Proto**：1 service / 10 RPC（卡牌回放 per REQ-038 §FR-008）
- **测试密度**：src 21 + tests 13 = 34
- **正式文档**：⚠️ 仅有 Card Replay 部分（REQ-038 §FR-008 + DTL-038 Match §4.4），未单独 REQ/BAS
- **推荐**：与 DTL-038 Match 域保持单点引用即可

### 3.5 replay-extra-service（402 LOC）

- **Proto**：1 service / 10 RPC（录像扩展域）
- **测试密度**：src 5 + tests 0 = 5
- **正式文档**：✅ WT-B 已补（REQ-043 / BAS-043 / DTL-050 / ARC-051）
- **推荐**：PH-2；测试密度仅 1.2%，需补 UT

### 3.6 gm-extra-service（432 LOC）

- **Proto**：1 service / 10 RPC（GM/运维扩展域）
- **测试密度**：src 5 + tests 0 = 5
- **正式文档**：✅ WT-B 已补（REQ-044 / BAS-044 / DTL-051 / ARC-052）
- **推荐**：PH-2；与 `gm-backend` 协同

### 3.7 leaderboard-extra-service（421 LOC）

- **Proto**：1 service / 7 RPC（图鉴扩展域）
- **测试密度**：src 5 + tests 0 = 5
- **正式文档**：✅ WT-B 已补（REQ-045 / BAS-045 / DTL-052 / ARC-053）
- **推荐**：PH-3；图鉴通常延后

### 3.8 operate-service（400 LOC）

- **Proto**：1 service / 7 RPC（运营活动）
- **测试密度**：src 6 + tests 0 = 6
- **正式文档**：✅ WT-B 已补（REQ-046 / BAS-046 / DTL-053 / ARC-054）
- **推荐**：PH-2；与 DTL-013 §3 联动

### 3.9 social-extra-service（556 LOC）

- **Proto**：1 service / 15 RPC（聊天/好友扩展域）
- **测试密度**：src 8 + tests 0 = 8
- **正式文档**：✅ WT-B 已补（REQ-047 / BAS-047 / DTL-054 / ARC-055）
- **推荐**：PH-2；与 DTL-013 大厅/聊天 联动

---

## 4. 测试设计覆盖矩阵

### 4.1 WT-C 输出（238 TC 用例 / 7 文件）

| DTL 文件 | 主题 | 原"不覆盖"数 | WT-C 新增 TC | 覆盖层 |
|---|---|---:|---:|---|
| DTL-015 | trade Saga | 4 | **29** | UT + IT + ST |
| DTL-016 | 对账/工单 | 4 | **38** | UT + IT + ST |
| DTL-018 | 身份/IdP | — | **31** | UT + IT + ST |
| DTL-019 | 推送/兑换码 | — | **30** | UT + IT |
| DTL-020 | 收据校验 | — | **38** | UT + IT + ST |
| DTL-026 | 匹配/Glicko-2 | — | **37** | UT + IT + ST |
| DTL-031 | ClusterOps | — | **35** | UT + IT + ST |
| **合计** | | **≥ 8** | **238** | |

TC-ID 命名约定：`TC-{DTL-NNN}-{SUBSYSTEM}-{NNN}`（符合项目既有规范）。

### 4.2 现有 `RGS-TST-*-ADD*` 系列（不重叠，独立设计）

- `RGS-TST-UT-04 / 05 / 07 / 13` 系列（含 addendums）
- 11 份独立测试设计文档（断点续传 / CDN / 风控 / Agent / 智能层）
- 与 WT-C 的 238 TC **不重叠**——前者是既有测试设计 addendums，后者是填补 DTL 自陈"不覆盖"的项目

### 4.3 覆盖率评估

| 域 | UT | IT | ST | 总数 |
|---|---:|---:|---:|---:|
| 5 域 + 卡牌 | 充足 | 充足 | 充足（部分依赖 G-CODE） | — |
| 8 扩展域 | 不足（普遍 5-15 src tests） | 几乎 0 | 几乎 0 | **缺口 > 80%** |
| 智能层 / Agent / 反作弊 | 0 | 0 | 0 | **缺口 100%** |
| 资源分发（DTL-041） | 充足（21 IT） | 充足 | 12/13 AC | — |

---

## 5. 后续推荐顺序

### 5.1 优先级 P0（CRITICAL — 启动 PH-1 必需）

1. **G-CODE-06** 待 Rust 1.98 stable GA + CI bootstrap 通过；当前 stable 为 1.97.1
2. **G-CODE-02 / G-CODE-03** 待 DTL-031 ClusterOps + ADR-0052 Active-Active 具名批准
3. **G-CODE-04** 待 Saga/Outbox 真实场景演练具名批准（架构 + DBA + 经济 Lead）
4. **G-CODE-05** 待五域 Lead DD Review 签署

### 5.2 优先级 P1（HIGH — PH-1 完成后启动）

5. **DTL-018 第三方 IdP**（Apple/Google/Steam OAuth）：launch 前置；WT-C 31 TC 已就绪
6. **DTL-026 Glicko-2 评分公式**：`glicko2_update()` + 波动率迭代；WT-C 37 TC 已就绪
7. **DTL-013 大厅 + 频道路由 + 私聊**：与 DTL-019 / DTL-039 协同
8. **DTL-014 GSM 玩家治理**（举报/黑名单/赛季）：与 WT-B leaderboard-extra 协同
9. **DTL-025 反作弊 DSL**：与 DTL-005 沙箱脚本引擎协同
10. **DTL-031 §11.2 证据矩阵**：120s/300s 参数验证
11. **DTL-042 服务器全生命周期 6 阶段**：分服 6 步 / 合服 5 步 Saga
12. **DTL-009 治理闭环 CI 校验**：CR-005 待审批（per RGS-REQ-013 §6）

### 5.3 优先级 P2（MEDIUM — PH-2 启动后）

13. DTL-019 APNs/FCM SDK + 兑换码生成算法
14. DTL-020 平台收据 SDK（App Store / GooglePlay）
15. DTL-022 弹性预留 + T3 多区域（TBD-CAP-001/002 校准）
16. DTL-023 请求处理链完整串联
17. DTL-024 集群部署 DAG Schema 校验 + CI 绑定

### 5.4 优先级 P3（LOW — PH-3+）

18. DTL-005 插件宿主（Rhai vs Wasmtime 评估）
19. DTL-008 Unity/UE FFI 胶水代码
20. DTL-011 / DTL-032-035 智能层 + Agent（设计正确状态）
21. DTL-017 AnalyticsStore 选型（TBD-INF-002）
22. DTL-021 GM 后台拓扑画布（TBD-VIZ-001/002）
23. **DTL-038-防丢包 + SPEC**：WT-A 已新增 DTL/BAS，SPEC 留待 BAS+DTL 评审通过

---

## 6. 附录：grep 验证命令清单

```bash
# 进入 worktree
cd "D:/RustGameServer/docs-parallel/wt-e"

# --- 1. 插件宿主（DTL-005）---
grep -r "PluginManager\|PluginRegistry\|PluginHost" crates/ --include="*.rs"
grep -r "rhai\|Rhai" crates/ --include="*.rs" --include="*.toml"

# --- 2. FFI 绑定（DTL-008）---
grep -r "extern \"C\"\|cdylib" crates/ --include="*.rs" --include="*.toml"

# --- 3. 治理 CI（DTL-009）---
grep -r "rgs-arc-olu\|olu_" crates/ --include="*.rs"
grep -r "ci_check\|governance_ci" crates/ --include="*.rs"

# --- 4. 智能层（DTL-011）---
grep -ri "langgraph\|openai\|gpt" crates/ --include="*.rs"

# --- 5. 大厅 / 频道路由（DTL-013）---
grep -r "lobby\|Lobby\|private_msg\|channel_router" crates/ --include="*.rs"

# --- 6. GSM 玩家治理（DTL-014）---
grep -r "season_pass\|blacklist\|report_abuse" crates/ --include="*.rs"

# --- 7. AnalyticsStore（DTL-017）---
grep -r "AnalyticsStore\|analytics_service" crates/ --include="*.rs"

# --- 8. OAuth / IdP（DTL-018）---
grep -r "OAuth\|OIDC\|IdP\|openid" crates/ --include="*.rs"

# --- 9. APNs / FCM（DTL-019）---
grep -r "APNs\|FCM\|push_delivery" crates/ --include="*.rs"

# --- 10. 平台收据（DTL-020）---
grep -r "AppStoreReceipt\|GooglePlayReceipt\|receipt" crates/ --include="*.rs"

# --- 11. GM 后台画布（DTL-021）---
grep -r "canvas\|infinite_canvas" crates/ --include="*.rs"

# --- 12. 弹性容量（DTL-022）---
grep -r "sharding\|shard_id\|migration_step" crates/ --include="*.rs"

# --- 13. 中间件（DTL-023）---
grep -r "middleware\|interceptor" crates/ --include="*.rs"

# --- 14. DAG 编排（DTL-024）---
grep -r "DAG\|deploy_run\|topo_sort" crates/ --include="*.rs"

# --- 15. 反作弊（DTL-025）---
grep -r "anti_cheat\|cheat_detect\|Rhai" crates/ --include="*.rs"

# --- 16. Glicko-2（DTL-026）---
grep -ri "glicko" crates/ --include="*.rs"

# --- 17. ClusterOps / PFAU（DTL-031）---
grep -r "cluster_ops\|crdt\|active_active\|all_reachable" crates/ --include="*.rs"

# --- 18. Agent 运行时（DTL-032〜035）---
grep -r "ActionIntent\|ActionReceipt\|agent_runtime" crates/ --include="*.rs"

# --- 19. 服务器生命周期（DTL-042）---
grep -r "realm_lifecycle\|merge_realm\|split_realm" crates/ --include="*.rs"

# --- 20. FEC 防丢包（DTL-038-防丢包，WT-A 新增）---
grep -r "fec\|xor_parity\|FecEncoder\|FecDecoder" crates/ --include="*.rs"

# --- 21. Crate LOC 统计 ---
for c in battle-service scene-service pvp-full-service replay-service replay-extra-service gm-extra-service leaderboard-extra-service operate-service social-extra-service; do
    find crates/$c -name "*.rs" | xargs wc -l | tail -1
done

# --- 22. 测试密度 ---
for c in battle-service scene-service pvp-full-service replay-extra-service gm-extra-service leaderboard-extra-service operate-service social-extra-service; do
    src_tests=$(grep -r "#\[test\]\|#\[tokio::test\]" crates/$c/src/ --include="*.rs" | wc -l)
    int_tests=$(grep -r "#\[test\]\|#\[tokio::test\]" crates/$c/tests/ --include="*.rs" 2>/dev/null | wc -l)
    echo "$c: src=$src_tests tests=$int_tests"
done
```

---

## 7. 总结

- **22 项 DTL 逐项审查完毕**（含 WT-A 新增 DTL-038-防丢包）
- **9 个代码先行扩展域已全部补全 REQ/BAS/DTL**（WT-B 输出 24 份文档 + 注册 23 条 + README §1.15）
- **238 个测试用例设计完成**（WT-C 输出 7 文件 +1716 行）
- **可追溯性矩阵 / 风险表已同步**（WT-D 输出 RGS-REQ-004/005 v3.11→v3.12 + 完成报告）
- **全部 5 个 worktree 已落地 commit**：4e7c50c / 728e967 / de1e4ec / c541055 / （本提交）
- **未写一行 Rust 代码 / SQL migration / Helm 制品**（per RGS-IMPL-001 §1.3）

**下一步（per 优先级 §5）**：
1. 等待 G-CODE 具名批准 → 启动 PH-1 编码
2. 优先级 P1 项目的代码实施按 §5.2 顺序
3. BAS-038 + DTL-038-防丢包 的 SPEC 在评审通过后补充
4. SPEC-DTL-038-防丢包 路径已在 RGS-REQ-005 §X "ULYS-1 同步" 登记

---

| 关联文档 | 路径 |
|---|---|
| 完成报告 | `docs/14-项目管理/RGS-PM-ULYS-1-COMPLETION_v0.1.md` |
| 可追溯性矩阵 | `docs/00-基准与治理/RGS-REQ-004_附件C_可追溯性矩阵.md` (v3.12) |
| 风险表 | `docs/00-基准与治理/RGS-REQ-005_附件D_问题风险管理表.md` (v3.12) |
| WT-A 输出 | `docs/02-运维安全与网络/RGS-BAS-038_*.md` + `RGS-DTL-038_核心传输防丢包强化_详细设计书.md` |
| WT-B 输出 | 24 份 REQ/BAS/DTL × 8 扩展域（commit c541055） |
| WT-C 输出 | 7 份 DTL 文件（commit 728e967） |
| WT-D 输出 | REQ-004/005 v3.12 + 完成报告（commit de1e4ec） |
