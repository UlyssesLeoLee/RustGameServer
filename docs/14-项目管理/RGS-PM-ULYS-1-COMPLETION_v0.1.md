# ULYS-1 完成报告（Parallel Completion Report）

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-PM-ULYS-1-COMPLETION |
| 版本 | 0.1 |
| 制定日 | 2026-09-11 |
| 制定者 | ULYS-1 主代理 + 5 个并行子代理 |
| Multica 议题 | ULYS-1（`01a090a2-a1ca-71b2-9f8d-0da4f4082993`） |
| 关联 | RGS-REV-001 / RGS-REV-002 + 9/4 改进路线图 §2 + 5 份工作树提交 |

---

## 1. 议题原始需求

> **ULYS-1** 标题：**查找是否有没有实施的设计，以及是否存在没有推进到详细设计的需求或者基本设计**（中文）

任务由发起人 Say世意（D-Boy, lidian727@gmail.com）2026-09-11 14:17 留言触发：

> **"并行完成所有未完成内容，根据需要开 worktree 和子任务"**

## 2. 工作分解（5 个并行 worktree + 子任务）

| WT | 分支 | 路径 | 主要产物 | 状态 |
|---|---|---|---|---|
| A | `agent/ulys-1/wt-a-bas38` | `docs-parallel/wt-a/` | RGS-BAS-038 + RGS-DTL-038-防丢包 + 登记册 | ✅ 完成并提交 |
| B | `agent/ulys-1/wt-b-extreq` | `docs-parallel/wt-b/` | 8 个扩展域 REQ/BAS/DTL 三层文档 | ⚠️ 部分（仅 REQ-040 战斗域） |
| C | `agent/ulys-1/wt-c-testdocs` | `docs-parallel/wt-c/` | 7 份 DTL 测试设计补全（238 TC, +1716 行） | ✅ 完成并提交 |
| D | `agent/ulys-1/wt-d-trace` | `docs-parallel/wt-d/` | RGS-REQ-004 v3.11→v3.12 / RGS-REQ-005 v3.11→v3.12 + 完成报告 | ✅ 完成并提交 |
| E | `agent/ulys-1/wt-e-gapspec` | `docs-parallel/wt-e/` | ULYS-1 文档×代码差距逐项扫描报告 | ⚠️ 待编排（dispatch 受限） |

> WT-B 仅写出一份 REQ-040 战斗域（290 行），其余 7 个扩展域的 21 份文档（7×3）尚需后续补充。
> WT-E 未能在并发窗口内启动（max_concurrent_children=3 + 子代理超时），已在本报告中由主代理 §4 给出等价的差距摘要。
> WT-A / WT-C / WT-D 均已成功提交。

## 3. 已完成工作（commit SHA + 行数）

### 3.1 WT-A：REQ-038 防丢包 BAS+DTL 补全

- **commit**：`4e7c50c`
- **新增文件**：
  - `docs/02-运维安全与网络/RGS-BAS-038_核心传输防丢包强化与周边协议选型_基本设计书.md`（451 行）
  - `docs/02-运维安全与网络/RGS-DTL-038_核心传输防丢包强化_详细设计书.md`（407 行）
- **修改文件**：`docs/document-registry.toml`（+25 行）
- **合计**：858 行新增；3 文件修改
- **关键设计**：ARC-047 FEC over QUIC Datagram 路径 + 自研单包级 XOR parity 编码器 + 新建 `rgs-fec` crate 判定（不汇入 `rgs-common`，per RGS-IMPL-001 §2 Q-102）+ 与既有 `network-gateway` / `rgs-session-state` / `player_db.outbox` 的对接点 + 25% 冗余度决策表
- **实施门禁**：rgs-fec 的 Rust 代码 / `outbox_datagram_fec` migration / 与 `network-gateway` 的集成代码，**均**需等待 `G-CODE-06` 批准后才可编码（per RGS-IMPL-001 §1.3）

### 3.2 WT-C：7 份 DTL 测试设计补全

- **commit**：`728e967`
- **修改文件**（7 份）：DTL-015（trade Saga）+ DTL-016（对账/工单）+ DTL-018（身份/IdP）+ DTL-019（推送/兑换码）+ DTL-020（收据校验）+ DTL-026（匹配/Glicko-2）+ DTL-031（ClusterOps）
- **合计**：+1716 行；238 TC 用例
- **TC 分布**：
  - DTL-015: 29 TC（trade Saga 端到端 + 补偿 + 防条件反转 + DLQ）
  - DTL-016: 38 TC（对账 + 工单 + RSK-SUP-002 防条件反转 + 重试）
  - DTL-018: 31 TC（IdP/OAuth/OIDC + 实名 + 未成年 + 合规）
  - DTL-019: 30 TC（APNs/FCM 推送 + 兑换码生成 + DLQ）
  - DTL-020: 38 TC（App Store/Google Play 收据 + 退款追回 + RSK-PLT-001）
  - DTL-026: 37 TC（Glicko-2 评分公式 + 评分周期 + 容差函数）
  - DTL-031: 35 TC（PFAU Active-Active + OCC + 状态机 + DAG + 演练）
- **TC-ID 命名约定**：`TC-{DTL-NNN}-{SUBSYSTEM}-{NNN}`（符合项目既有规范）
- **覆盖层**：UT（单元）+ IT（集成 Testcontainers）+ ST（系统），均引用既有 `rgs-testkit` 夹具（`PlayerFixture` / `EconomyFixture` / `SagaFixture` / `MatchFixture` / `SocialFixture` / `AdminFixture`）

### 3.3 WT-D：可追溯性矩阵 + 风险管理表同步

- **修改文件**：
  - `docs/00-基准与治理/RGS-REQ-004_附件C_可追溯性矩阵.md`（v3.11 → v3.12；+32 行：§7 ID 归属登记 + §7.1 主编号 + §9 新增 21 项差距登记附录 + §4.1 课题行新增 8 个扩展域）
  - `docs/00-基准与治理/RGS-REQ-005_附件D_问题风险管理表.md`（v3.11 → v3.12；+47 行：§1.3 ISS-151〜171 + §2.3 RSK-094〜122）
- **新增文件**：本完成报告 `RGS-PM-ULYS-1-COMPLETION_v0.1.md`
- **关键映射**：
  - ARC-048（Battle）/ ARC-049（Scene）/ ARC-050（PVP-Full）/ ARC-052（Replay-X）/ ARC-057（GM-X）/ ARC-059（LB-X）/ ARC-060（OPR）/ ARC-061（Social-X）— 8 个新 ARC 编号已登记
  - 21 项未实施 DTL 全部登记为 ISS-151〜171 + RSK-094〜114
  - 8 个扩展域"代码先行 / 文档滞后"登记为 RSK-115〜122

### 3.4 WT-B：8 个扩展域（部分完成）

- **已完成**：
  - `docs/07-社交运营与玩家治理/RGS-REQ-040_战斗域_需求定义书.md`（290 行）
- **待补**（后续 dispatch）：
  - Battle: BAS-040 + DTL-047
  - Scene: REQ-041 + BAS-041 + DTL-048
  - PVP-Full: REQ-042 + BAS-042 + DTL-049
  - Replay-X: REQ-043 + BAS-043 + DTL-050
  - GM-X: REQ-044 + BAS-044 + DTL-051
  - LB-X: REQ-045 + BAS-045 + DTL-052
  - OPR: REQ-046 + BAS-046 + DTL-053
  - Social-X: REQ-047 + BAS-047 + DTL-054（if not covered by DTL-013）
- **合计**：21 份文档 × ~250 行 = ~5250 行

### 3.5 WT-E：差距扫描报告（待编排）

- 由主代理在本报告中以摘要形式覆盖，详见 §4 差距摘要。完整报告（1500-2500 行）待后续在 wt-e 工作树单独 dispatch。

## 4. ULYS-1 议题最终结论

### 4.1 A. 没有推进到基本设计的需求

- **唯一例外**：REQ-038 核心传输防丢包 → ✅ WT-A 已补全（BAS-038 + DTL-038-防丢包）
- **结论**：当前**不存在**其他"未推进到 BAS 的 REQ"。

### 4.2 B. 有基本设计但未推进到详细设计

- **结论**：**不存在**。全部 27 份 BAS + 8 份 BAS-031〜037 + BAS-100 + 新增 BAS-038 都有匹配 DTL。

### 4.3 C. 有详细设计但未实施 → 21 项登记

按 RGS-REQ-004 §9 + RGS-REQ-005 §2.3 完整登记如下：

| # | DTL | 项目 | 阻断条件 | 推荐 PH |
|---|---|---|---|---|
| 1 | DTL-005 | 插件宿主（Plugin trait/Registry/Host） | 当前无 Rust 代码 | PH-3+（FE 平台需求驱动） |
| 2 | DTL-011 | 智能决策层（LangGraph/LLM） | 设计明确"默认关闭" | 已在 DTL-011 §A v0.2 自陈"留待后续" |
| 3 | DTL-018 | 第三方 IdP（Apple/Google/Steam OAuth） | 全仓 0 命中 | PH-1（依赖 launch 前置） |
| 4 | DTL-025 | 反作弊 DSL | 全仓 0 命中 | PH-2 |
| 5 | DTL-026 | Glicko-2 评分公式 | match-service 内未实装 | PH-2（match 域内） |
| 6 | DTL-032〜035 | Agent 运行时 5 份 DTL | v0.2 待评审 | PH-3+ |
| 7 | DTL-042 | 服务器全生命周期 6 阶段 | operators 骨架已有，编排器/Saga 未实装 | PH-2 |
| 8 | DTL-008 | Unity/UE FFI 绑定胶水 | 核心 SDK 完成 | PH-3 |
| 9 | DTL-009 | 治理闭环 CI 校验 | CR-005 待审批 | PH-3 |
| 10 | DTL-013 | 大厅 + 频道路由 + 私聊 | 全仓 grep "lobby"/"private_msg" = 0 | PH-2 |
| 11 | DTL-014 | GSM 玩家治理（举报/黑名单/赛季） | 部分实现 | PH-2 |
| 12 | DTL-017 | AnalyticsStore 选型 | 全仓 0 命中 | PH-3 |
| 13 | DTL-019 | APNs/FCM SDK + 兑换码生成 | 接口骨架 | PH-2 |
| 14 | DTL-020 | 平台收据 SDK | 仅占位 | PH-2 |
| 15 | DTL-021 | GM 拓扑画布 | TBD-VIZ-001/002 | PH-3 |
| 16 | DTL-022 | T3 多区域 + 预测预热 | TBD-CAP-001/002 | PH-3 |
| 17 | DTL-023 | 前后处理管道串联 | 中间件未完整 | PH-2 |
| 18 | DTL-024 | 集群部署 DAG | schema 校验未实装 | PH-2 |
| 19 | DTL-031 | ClusterOpsService PFAU all-reachable | "待具名人类审批，不得实施" | 待 G-CODE-02 批准 |
| 20 | DTL-013 | team/PRODUCT_CATALOG/PURCHASE_RECORD DDL | 仅 spec | PH-2 |
| 21 | DTL-005 | 沙箱脚本引擎 Rhai 选型 | TBD-PLG-001 | PH-3 |

### 4.4 D. "代码先行 / 文档滞后" 的扩展域 → 8 项登记

| 域 | 现有 crate | LOC | RPC | 当前文档 | 待补文档 |
|---|---|---|---|---|---|
| Battle | `battle-service` | 2,875 | 241 | 无 | REQ-040 ✅ / BAS-040 / DTL-047 |
| Scene | `scene-service` | 3,823 | 148 | 无 | REQ-041 / BAS-041 / DTL-048 |
| PVP-Full | `pvp-full-service` | 1,402 | 151 | 无 | REQ-042 / BAS-042 / DTL-049 |
| Replay-X | `replay-extra-service` | 386 | 199 | 无 | REQ-043 / BAS-043 / DTL-050 |
| GM-X | `gm-extra-service` | 416 | 37 | 无 | REQ-044 / BAS-044 / DTL-051 |
| LB-X | `leaderboard-extra-service` | 405 | — | 无 | REQ-045 / BAS-045 / DTL-052 |
| OPR | `operate-service` | 384 | 47 | 无 | REQ-046 / BAS-046 / DTL-053 |
| Social-X | `social-extra-service` | — | — | 部分在 DTL-013 | REQ-047 / BAS-047 / DTL-054 |

**判定（per WT-D §4.1 课题行 P-BATTLE 等）**：建议**合并到 RGS-REQ-038 卡牌游戏适配 + DTL-038 卡牌游戏适配下作为子域**；若属"通用服务器能力"则补齐三层文档。

## 5. 阻塞项 / 待办

### 5.1 文档侧（待续）

| 项 | 责任 worktree | 优先级 |
|---|---|---|
| 8 个扩展域剩余 21 份文档 | WT-B（续） | 中 |
| ULYS-1 完整差距扫描报告 | WT-E（dispatch 续） | 中 |
| BAS-038 / DTL-038-防丢包 的 SPEC | 后续 dispatch | 低（待 BAS+DTL 评审通过） |

### 5.2 实施侧（被 IMPL-001 §1.3 阻塞）

| Gate | 内容 | 阻塞 |
|---|---|---|
| G-CODE-01 | — | 待具名批准 |
| G-CODE-02 | DTL-031 控制面边界 / Cargo 位置 / PFAU 状态 | 待具名批准 |
| G-CODE-03 | ADR-0052 Active-Active / all-reachable | 待具名批准 |
| G-CODE-04 | Saga/Outbox/补偿 | 架构 + DBA + 经济域 Lead 具名批准 |
| G-CODE-05 | 五域 App/DB/插件宿主依赖矩阵 | 五域 Lead DD Review 签署 |
| G-CODE-06 | Rust 1.98 stable GA + CI bootstrap | 截至 2026-09-11 stable 为 1.97.1 |
| G-CODE-07 | rgs-testkit OLU 重算 + QA/SRE 签署 | 待具名批准 |

## 6. 致发起人

5 份 worktree 中，**3 份完整提交**（WT-A / WT-C / WT-D），**2 份部分完成**（WT-B 仅战斗域 REQ，WT-E 因 max_concurrent_children=3 + 超时尚未启动）。已完成部分合计：

- **3 commits / 5 文件** 新增 + 2 文件大幅修改
- **+约 2600 行** 正式文档（不计算差异更新）
- **238 TC** 测试用例
- **21 项未实施 DTL** 完整登记
- **8 个扩展域** ARC 编号 + 风险条目登记

后续如需补完 WT-B / WT-E，请回复"继续"或单独指派。**没有写一行 Rust 代码或 SQL migration**（per RGS-IMPL-001 §1.3 实施门禁）。

---

## 附：commits 列表

| 分支 | SHA | 说明 |
|---|---|---|
| `agent/ulys-1/wt-a-bas38` | `4e7c50c` | docs(bas-dtl-038): 补全 REQ-038 核心传输防丢包 BAS+DTL (ULYS-1) v0.1 |
| `agent/ulys-1/wt-c-testdocs` | `728e967` | docs(test-design): 补全 7 份 DTL 文档 §6/§7 测试设计条目 (ULYS-1) v0.1→v0.2 |
| `agent/ulys-1/wt-d-trace` | （本次提交） | docs(traceability+risk): RGS-REQ-004/005 v3.11→v3.12 同步 ULYS-1 审计 |
| `agent/ulys-1/wt-b-extreq` | （未提交） | 8 扩展域仅 REQ-040 战斗域完成，其余 21 份待续 |
| `agent/ulys-1/wt-e-gapspec` | （未启动） | 完整差距扫描报告待续 |
