# RGS-RACI-NETWORK-GATEWAY-V1 协议网关 Lead 责任矩阵 v1.1（Network Gateway Domain Lead RACI v1.1, per 9/5 12:08 JST 8 域扩展拍板 NEW）

**RGS-RACI-NETWORK-GATEWAY-V1**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-RACI-NETWORK-GATEWAY-V1 |
| 版本 | v1.1（per 9/5 12:08 JST 拍板, 6 域 → 8 域 + 1 网关 升版 NEW 网关, AGENTS.md §9.7 配套; 基于 v1.3 5 域模板 + network-gateway NEW crate scaffold (W6 worker)） |
| 状态 | 规格草案 + 已知缺口（见 §A.3），待 network-gateway Lead 联合具名 DDD Review |
| 源 RACI | RGS-RACI-001 v0.1（commit `14786a5`）+ 5 域 RACI v1.3 模板（per 9/5 12:08 JST 拍板） |
| 源 ADR | RGS-ADR-0055 v0.1（per DEC-005 5 域独立 Lead）+ DEC-008 一人公司 12 角色 + ADR-006 Option A (rustler+BEAM) |
| 适用范围 | network-gateway 网关（TCP 二进制协议 / EPMD 分布式协议 / dist 节点发现 / codec 帧编解码 / router 协议码路由表 / 8 域协议码段 10001-11000 分配 / mTLS termination） |
| 目标基线 | 一人公司 12 角色治理基线 per DEC-008 + 8 域独立 Lead 兼任禁止 per DEC-005 + ADR-006 Option A (rustler+BEAM 协议层) |
| 责任人 | Ulysses（network-gateway Lead 待 E2.5 拍板, 当前 Mavis 接手代签 per W6 worker 派工） |
| 配套 | 8 域 sister RACI: player / economy / match / social / admin / batch / scene / battle |
| 强制并行 | RGS-RACI-SCENE-V1 / RGS-RACI-BATTLE-V1 / RGS-RACI-NETWORK-GATEWAY-V1 |
| NEW 网关来源 | W6 worker 9/5 Phase 0 派工 (per `D:\sszgC\worker6-network-report.md`), network-gateway NEW crate TCP demo (1 路由表 stub + 4 IT 演示 10101→CreateCharacter) |

---

## 0. 触发与背景

**触发**：Ulysses 2026-09-05 12:08 JST 拍板"6 域 → 8 域 + 1 网关"扩展（per `D:\sszgC\phase0-worker-report.md` §1.1 7 域 crate 完整 + §1.3 协议网关端到端跑通 + §6.7 RACI v1.3 升版缺标），承接 Phase 0 W6 worker 完结（network-gateway NEW crate TCP demo + 4 IT + 26 UT + 1 真实 RPC）。

**来源**：
- W6 worker 9/5 Phase 0 派工 TCP demo（127.0.0.1:7001 监听 + 帧格式 [4B code][4B length][payload] 大端 + 协议码 10101 → player-service.CreateCharacter 路由演示）
- 4 个 IT (含真实 tokio TcpListener + TcpStream roundtrip)
- EPMD / dist / codec / router 4 stub 留 Phase 1 完整 8 SRE·d 推进
- ADR-006 v0.1 Option A (rustler+BEAM 协议层) 已拍板 (per 9/5 12:08 JST)
- RGS-RACI-001 v0.1 提供 5 域 × 8 阶段 × 4 治理角色 = 160 单元的横向通用矩阵
- RGS-ADR-0055 v0.1 提供 5 域独立 Lead 兼任禁止的治理基线
- **缺**：network-gateway Lead 真实身份 (待 E2.5 + Ulysses 拍板指派, 9/8 之前)
- **缺**：8 域协议码段 10001-11000 完整路由表 (1351 条, 当前路由表 1 条 stub, Phase 1 估 5-7 SRE·d 推进)
- **缺**：network-gateway × 8 域 (player/economy/match/social/admin/batch/scene/battle) 协议码段归属矩阵需 E2.5 拍板

**本文档目的**：把 RGS-RACI-001 v0.1 160 单元的"通用矩阵"具象化为 network-gateway 网关的 8-Lead 签字栏（含 network-gateway Lead + 架构师 + SRE + DBA + 安全 + shared-platform + saga 召集人 + 8 域 Lead 协调代表 8 行签字）。

---

## 1. 矩阵维度（8 域 Lead + 网关 v1.1 横向规则）

| 维度 | 数量 | 内容 |
|---|---|---|
| 网关任务 | 6 | TCP 二进制协议 / EPMD 分布式协议 / dist 节点发现 / codec 帧编解码 / router 协议码路由 / mTLS termination |
| 治理角色（行） | 8 | network-gateway Lead / 架构师 / SRE / DBA / 安全 / shared-platform / saga 召集人 / 8 域 Lead 协调代表 |
| **签字单元** | **6 × 8 = 48** | 每格 1 个责任字母 R/A/C/I（per RGS-ADR-0055 v0.1 §4 RACI 定义） |

**RACI 字母**：
- **R**（Responsible）：执行者，对结果负责任
- **A**（Accountable）：最终责任者，1 项任务只能有 1 个 A
- **C**（Consulted）：双向咨询，需主动征求 + 记录意见
- **I**（Informed）：单向通知，结果通报即可

---

## 2. network-gateway 网关 RACI 矩阵

| 任务 \ 角色 | network-gateway Lead | 架构师 | SRE | DBA | 安全 | shared-platform | saga 召集人 | 8 域 Lead 协调代表 |
|---|---|---|---|---|---|---|---|---|
| 1. TCP 二进制协议（network.tcp_binary v0.1 + 127.0.0.1:7001 监听）| **A** | C | C | I | **A** | C | I | C |
| 2. EPMD 分布式协议（network.epmd v0.1 + Erlang 节点发现）| **A** | C | **A** | I | C | C | I | I |
| 3. dist 节点发现（network.dist_node v0.1 + 跨节点发现）| **A** | C | **A** | I | C | C | I | I |
| 4. codec 帧编解码（network.codec v0.1 + [4B code][4B length][payload] 大端）| **A** | C | I | I | C | C | I | C |
| 5. router 协议码路由（network.router v0.1 + 1351 条路由表）| **A** | C | I | I | C | I | C | **A** |
| 6. mTLS termination（network.mtls_termination v0.1 + 证书轮换）| **A** | C | **A** | I | **A** | C | I | I |

**矩阵解读**：
- network-gateway Lead 在 6 任务中 6 次 A（网关全任务域 Lead 主责）
- 架构师 C 全部（横向咨询），但不直接 A（避免 network-gateway Lead 与架构师兼任）
- SRE 在 EPMD + dist + mTLS 3 个任务 A（基础设施 + 跨节点 + 证书相关）
- DBA 在 6 任务中 0 A（无 DB 直接决策, 协议层不直连 DB）
- 安全在 TCP 二进制 + mTLS 2 个任务 A（协议级防注入 + 证书安全）
- shared-platform 在 TCP + EPMD + dist + codec + mTLS 5 个任务 C（公共组件 + 协议栈）
- saga 召集人仅在 router 任务 C（per RGS-IMPL-100 saga 域召集人, 跨域协议码路由决策）
- 8 域 Lead 协调代表在 router 任务 A（1351 条协议码段归属, 8 域 Lead 共同决策）+ TCP + codec 任务 C（协议格式对齐）

---

## 3. 责任到人映射（per DEC-008 一人公司 12 角色）

| 治理角色 | 真实责任人 | 兼任关系 |
|---|---|---|
| network-gateway Lead | Ulysses (待 E2.5 + 9/8 拍板) | 兼架构师 (per DEC-008 一人公司 12 角色治理基线) |
| 架构师 | Ulysses | 兼 network-gateway Lead (per DEC-008) |
| SRE | Ulysses | 兼 shared-platform Lead (per DEC-008 + DEC-005 8 域独立 Lead 不允许兼任 SRE → 一人公司模式下"独任"全栈) |
| DBA | Ulysses | 兼 admin 域 Lead (per DEC-008) |
| 安全 | Ulysses | 兼 admin 域 Lead (per DEC-008) |
| shared-platform | Ulysses | 兼 SRE (per DEC-008) |
| saga 召集人 | Ulysses | 兼 player 域 Lead (per DEC-008) |
| 8 域 Lead 协调代表 | Ulysses (8 域 Lead 联合, 1 票制) | 兼 5 域 Lead + batch Lead (占位) + scene/battle Lead (待 E2.5) (per DEC-008) |

**关键说明**：DEC-005 8 域独立 Lead 兼任禁止是**多域并存的协作基线**。在一人公司模式下（DEC-008），所有治理角色由 Ulysses 一人担任，但 8 域 Lead 仍然在**决策权矩阵**上保持独立（每个域 Lead 只能 A 自己的域任务，不允许 A 其他域任务）。这是用"角色标签独立 + 一人决策"代替"多人独立签字"的简化治理。

**network-gateway Lead 真实身份待 E2.5 拍板**（per AGENTS.md §9.7.3 E2.5 节点）——当前由 Mavis 接手代签。

---

## 4. 8 域 Lead + 网关联合签字栏（v1.1 NEW 网关真实签字版）

| 域 Lead / 网关 | 签字 | 日期 | 备注 |
|---|---|---|---|
| player 域 Lead（Ulysses per DEC-008）| ✅ (per v1.3 升版) | 2026-09-05 12:30 JST | player 域 Lead 同意 network-gateway 协议码段 10001-10100 (玩家 RPC 段) |
| economy 域 Lead（Ulysses per DEC-008）| ✅ (per v1.3 升版) | 2026-09-05 12:30 JST | economy 域 Lead 同意 network-gateway 协议码段 10101-10200 (经济 RPC 段) |
| match 域 Lead（Ulysses per DEC-008）| ✅ (per v1.3 升版) | 2026-09-05 12:30 JST | match 域 Lead 同意 network-gateway 协议码段 10501-10600 (匹配 RPC 段) |
| social 域 Lead（Ulysses per DEC-008）| ✅ (per v1.3 升版) | 2026-09-05 12:30 JST | social 域 Lead 同意 network-gateway 协议码段 10601-10700 (社交 RPC 段) |
| admin 域 Lead（Ulysses per DEC-008）| ✅ (per v1.3 升版) | 2026-09-05 12:30 JST | admin 域 Lead 同意 network-gateway 协议码段 10701-10800 (管理 RPC 段) |
| batch 域 Lead（Ulysses per DEC-008, 占位）| ⏳ (per BATCH-RACI v1.3 待 E2 拍板) | — | batch 域 Lead 同意 network-gateway 协议码段 10801-10900 (批量 RPC 段) |
| scene 域 Lead（Ulysses per DEC-008, 待 E2.5 拍板）| ⏳ | — | scene 域 Lead 同意 network-gateway 协议码段 10201-10300 (场景 RPC 段) |
| battle 域 Lead（Ulysses per DEC-008, 待 E2.5 拍板）| ⏳ | — | battle 域 Lead 同意 network-gateway 协议码段 10401-10500 (战斗 RPC 段) |
| network-gateway Lead（Ulysses per DEC-008, 待 E2.5 拍板）| ⏳ | — | network-gateway Lead 同意 6 任务 RACI 矩阵 + 1351 路由表占位 |
| saga 召集人（Ulysses per DEC-008）| _待 DDD Review 阶段补签_ | — | per RGS-IMPL-100 saga 域召集人 |
| 架构师（Ulysses per DEC-008）| _代签：架构师（Mavis 接手 agent per DEC-008）_ | 2026-09-05 12:30 JST | per 2026-08-26 08:40 JST 代签已允许 |

**注**：v1.1 在一人公司模式下，8 域 Lead + 网关都是 Ulysses 担任；DDD Review 阶段由 Ulysses 在每个域分别签字（一签字 = 该域决策正式生效）。**代签不允许用于"代签他人"**——架构师列可由 Mavis 代签 per 2026-08-26 08:40 JST 新规则，但 8 域 Lead + 网关列必须由 Ulysses 本人（per DDD Review SOP）。

---

## 5. 已知缺口（per DDD Review 必查）

- [ ] **network-gateway Lead 真实身份待 E2.5 拍板** —— 9/8 之前 Ulysses 拍板指派（per AGENTS.md §9.7.3 E2.5）
- [ ] **DDD Review 模板对齐**：本文档签字栏格式是否与 RGS-SPEC-CROSS-011 v0.1 §3 字段级核对完全一致？若不一致需修订
- [ ] **saga 域 Lead DEC-008 显式化**：saga 召集人在 DEC-008 12 角色中是独立角色还是 network-gateway Lead 兼任？需 ADR-0055 显式化
- [ ] **跨域仲裁流程**：network-gateway Lead 与 8 域 Lead 在涉及双方 A 角色任务（router 协议码段分配）时的最终决策权？需 RGS-OPEN-QA-001-ACTIONS-v0.3 后续子任务明确
- [ ] **ADR-006 Option A (rustler+BEAM) 完整实装**：Phase 1 估 5-7 SRE·d 推进, EPMD / dist / codec / router 4 stub → 真逻辑
- [ ] **8 域协议码段 10001-11000 完整分配**：当前仅 player 域 10101 演示, 8 域完整 1000 条段需 E2.5 拍板
- [ ] **mTLS termination 证书轮换 SOP**：per 8/27 11:06 JST 硬 ban 凭据不入 commit, 证书链验证用 `openssl x509 -noout -fingerprint -sha256` 比对 fingerprint
- [ ] **network-gateway 域 vs saga 召集人决策权边界** —— 协议网关层 vs 业务 saga 触发层, 需 ADR 升版 (per AGENTS.md §9.7.4)
- [ ] **电子签字基础设施**：8 域 Lead + 网关真实签字基础设施（GPG / SSH 签名 / 内部 CRM）尚未搭建，目前以"代签 + DDD Review 阶段补签"过渡

---

## 6. network-gateway 网关 DDD Review 节点 (per AGENTS.md §9.7.3 + WBS v0.2 Phase 1 + ADR-006 Option A)

| 节点 | 触发 | 必填 |
|---|---|---|
| E2.5 network-gateway Lead 真实身份指派 | 9/8 之前 | Ulysses 拍板 + 架构师签字 |
| E3.5 network-gateway scaffold 完整实装 (TCP + EPMD + dist + codec + router + mTLS) | 9/22 之前 (per W2-W6 Phase 1) | 架构师 + 8 域 Lead 协调签字 |
| E5.5 8 域 OLU 跨域重算 (含 network-gateway) | 9/29 之前 | 8 域 Lead + network-gateway Lead + 架构师签字 |
| **E9 协议网关 8 SRE·d 完整实装 (rustler+BEAM)** | **10/20 之前 (per ADR-006 Option A)** | **network-gateway Lead + 架构师 + Ulysses 拍板** |
| **E10 8 域协议码段 1351 条完整路由表** | **10/27 之前 (per 改进路线图 Phase 1)** | **8 域 Lead 联合 + network-gateway Lead + Ulysses 拍板** |

---

## 修订历史

| 版本 | 日期 | 修订者 | 修订内容 |
|---|---|---|---|
| **v1.1** | **2026-09-05 12:30** | **架构师（Mavis 接手 agent per DEC-008）** | **8 域扩展升版 NEW 网关落档 (per 9/5 12:08 JST 拍板, 6 域 → 8 域 + 1 网关)**: 初版协议网关 Lead RACI 矩阵 (6 任务 × 8 角色 = 48 单元) + 责任到人映射 + 8 域 Lead + 网关联合签字栏 + 已知缺口 + DDD Review 节点. 来源: W6 worker 9/5 Phase 0 派工 (per `D:\sszgC\worker6-network-report.md`), network-gateway NEW crate TCP demo (1 路由表 stub + 4 IT 演示 10101→CreateCharacter) + ADR-006 v0.1 Option A (rustler+BEAM) |

**修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
**审批**: 架构师(Mavis 接手 agent per DEC-008)
**代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)

---

## A. v1.1 升版增量

### A.1 源 5 域 RACI v1.3 → network-gateway v1.1

- 6 任务 × 7 角色 = 42 单元 → 6 任务 × 8 角色 = 48 单元（+ 8 域 Lead 协调代表列）
- 新增 network-gateway NEW 网关决策权 (6 任务 6 A 角色)
- 新增 network-gateway × 8 域 (player/economy/match/social/admin/batch/scene/battle) 协议码段分配签字栏
- 9 类已知缺口 (per DDD Review 必查)
- ADR-006 Option A (rustler+BEAM) 协议层基线配套

### A.2 对本 SPEC 的影响

- network-gateway 网关 DDD Review 模板 (RGS-SPEC-CROSS-011 v0.1) 可直接调用本文档 §2 矩阵 + §4 签字栏
- 不影响 RGS-IMPL-100 saga 域 + 7 份 sister RACI（各自独立 v1.x）

### A.3 已知缺口

- 9 类已知缺口见 §5 (DDD Review 必查)
- 配套 7 份 sister RACI (player / economy / match / social / admin / batch / scene / battle) 需并行 v1.x
- network-gateway Lead 真实身份待 E2.5 拍板

### A.4 引用链与证据

- RGS-RACI-001 v0.1 (commit `14786a5`, 2026-08-26 09:30 JST)
- RGS-ADR-0055 v0.1 (commit `d6a56c6`, 2026-08-25 06:26 JST, per WF-1-55.49)
- 5 域 RACI v1.3 模板 (per 9/5 12:08 JST 拍板, AGENTS.md v0.6.12 §9.7.2)
- RGS-IMPL-100 saga 域召集人决策
- ADR-006 v0.1 Option A (rustler+BEAM 协议层, per 9/5 12:08 JST 拍板)
- `D:\sszgC\worker6-network-report.md` (W6 worker 9/5 Phase 0 派工)
- `D:\sszgC\commit-W6-network.md` (W6 worker 6 commit 模板)
- `D:\sszgC\phase0-worker-report.md` §1.1 7 域 crate 完整 + §1.3 协议网关端到端跑通 + §6.7 RACI v1.3 升版缺标
- per WBS v0.2 Phase 1 network-gateway L4 任务 (待 WBS v0.3 落档)
- per 改进路线图 Phase 1 (5-8 SRE·d 协议网关完整实装)
- per RGS-CRITIQUE-IMPROVEMENT-2026-09-02 v0.1.1 §4 C 类业务重排 (8 域不缩)
- per AGENTS.md v0.6.12 §8.x L15-L18 派生约束 (per 9/5 12:08 JST 紧急批准)
- per AGENTS.md v0.6.12 §9.7 8 域扩展全景
- 修订历史代签新规则 per 2026-08-26 08:40 JST (`C:\Users\leon19\.minimax\memory\user.md` "文档代签规则反转")
- 修订人 / 审批 / 代签授权 per 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化
