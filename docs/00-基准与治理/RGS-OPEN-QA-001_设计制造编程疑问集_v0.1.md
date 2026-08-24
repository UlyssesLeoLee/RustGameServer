# RGS-OPEN-QA-001 设计与制造编程疑问集 v0.1

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-OPEN-QA-001 |
| 版本 | 0.2（Ulysses 24 条全部答复，7 条 🟡 部分答复标注需下游核实，其余 🟢 已答复）|
| 状态 | 🟢 已答复，进入 §6 下游动作执行阶段 |
| 依据 | RGS-REV-003 v0.3 §6 异议登记表 + RGS-REV-011 v0.1 6 项缺口 + 5-DOMAIN-DTL-REVIEW-REPORT §1.1-§1.7 39 checklist × 5 域 + RGS-PLAN-001 v1.0 + RGS-WBS-001 v0.6 L4 进度表 + RGS-TS-001 §6 token-OLU + DEC-005/DEC-008 |
| 范围 | 5 业务域 DTL/SPEC + 跨域一致性 + Saga 系统 + 5 域 Lead 兼任 + PH-1 工程基础 8 个新 L4 任务（WF-1-55.32~41）|
| 不在本疑问范围 | 工具链 Bug（RGS-TS-001 §7 走 G-CODE-06 旁路）/ 业务 GM 工具 UI（走 admin 域 Lead）/ 客户端 SDK（走 player 域 Lead + UE/Unity 引擎集成）|
| 关联 | RGS-OPEN-QA-001 与 RGS-QA-001（实施前 QA 表）正交：QA-001 是**通过条件清单**，OPEN-QA-001 是**待答复疑问池**（疑问闭环后可能产生新 QA-001 行）|
| 责任人 | AI worker（疑问提出方）→ 审核人 Ulysses（一人公司 12 角色 per DEC-008 兼任）|

---

## 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-24 | AI worker（PH-1 启动首批疑问沉淀）| 首次产出：设计 10 条 + 制造编程 10 条 + 治理 4 条 = **24 条疑问**，全部 🟠 待答复 |
| 0.2 | 2026-08-24 | Ulysses（审核人答复，AI worker 落笔）| 24 条全部答复：17 条 🟢 已答复可直接落地，7 条 🟡 部分答复（Q-D-02/Q-D-05/Q-M-02/Q-M-03/Q-M-04/Q-G-03/Q-G-04）因现状已过时或需要补充实测/逐份核验而标注部分答复。历史疑问原文未改动，仅追加答复栏内容（per §末尾说明）。下游动作已在各答复栏标注，尚未执行（新建 DTL/ADR/DEC 等文档需另行确认后进行）。**同批次自查修正 4 处**：Q-D-01/Q-D-02 提议的 DTL-037/038 与既有 `RGS-DTL-037_Economy域`/`RGS-DTL-038_Match域` 撞号，改为 DTL-043/044（并同步修正 Q-M-09 引用）；Q-D-03/Q-D-04 漏查已存在的 `RGS-SPEC-CROSS-001`（错误码字典）/`RGS-SPEC-CROSS-007`（5域RBAC角色矩阵）骨架文档，误提"新建"，改为"填充"，并核实 `RGS-DEC-NOGO-001` 确认两份骨架的 NO-GO 激活条件已解除，可直接升 v0.2 填内容。 |

---

## 1. 文档目的与用法

### 1.1 为什么单独建一张疑问集（不直接写 REV-003 / REV-011）

| 对比 | REV-003 联合评审 | REV-011 6 项缺口 Follow-up | **OPEN-QA-001（本表）** |
|---|---|---|---|
| 时机 | G-CODE Gate 关闭前 | REV-004 5 域 Review 之后 | PH-1 实施过程中持续 |
| 形态 | 一次性签字 | 提案 + 决议 | 开放池，**滚雪球** |
| 责任人 | 多角色联合 | 5 域 Lead + 架构 | AI worker 提问，**审核人答复** |
| 闭环方式 | 签字 + 修订 DTL/ADR | DTL/SPEC 升版 | 答复 → 决策落地 → 关闭或升级 QA-001 / 改 DTL |
| 数量 | 4 大议题（已结构化）| 6 项缺口（已结构化）| **未结构化**，本表做结构化承载 |

### 1.2 三组分类

| 组 | 编号前缀 | 触发场景 | 审核人期望回复时间 |
|---|---|---|---|
| **A 设计** | Q-D-NN | DTL/SPEC/ADR 文档层面含糊、冲突、缺失 | P0 ≤ 1 天 / P1 ≤ 3 天 / P2 ≤ 7 天 |
| **B 制造/编程** | Q-M-NN | 编码/测试/部署/CI 阶段选择路径不清 | P0 ≤ 1 天 / P1 ≤ 3 天 / P2 ≤ 7 天 |
| **C 治理/流程** | Q-G-NN | 一人公司 + DEC + WBS 流程层冲突 | P0 ≤ 1 天 / P1 ≤ 3 天 / P2 ≤ 7 天 |

### 1.3 状态机

```
🟠 待答复 → 🟢 已答复 / 🟡 部分答复 / 🔴 升级（写 QA-001 / 改 DTL / 走 ADR）
```

---

## 2. A 组：设计疑问（10 条）

### Q-D-01 [P0] DTL-019 vs DTL-X 消息分发主表归属

| 字段 | 内容 |
|---|---|
| 关联文档 | 5-DOMAIN-DTL-REVIEW-REPORT.md §1.5 §A.5.1（第 72 行）+ REV-004 附件 A §A.5 + REV-011 §2.4 + DTL-019 §1.2 |
| 现状 | DTL-019 实际是"推送+兑换码"（push_consents + redemption_code_batches/redemption_codes/redemption_records），**不含 messages/message_recipients/conversations 三表**；REV-004 附件 A §A.5 文件指向"DTL-019 消息分发"与源文件标题"DTL-019 消息推送与兑换码运营工具"不一致 |
| 疑问 | 选 A/B/C 哪个？<br>**A** 新建 `RGS-DTL-XXX_消息分发_v0.1.md`（推荐 per REV-011 §2.4 + 提议 WF-1-55.38）<br>**B** DTL-019 v0.2 升版含消息分发（扩大范围）<br>**C** 归跨域 DTL-021~025 |
| 期望答复 | 选项 + 文档 ID 分配 + 是否进 1.0 而非 1.5 |
| 答复栏 | 🟢 选 **A**：新建 `RGS-DTL-043_消息分发_v0.1.md`（⚠️v0.2修正：初版误标 DTL-037，该号已被 `RGS-DTL-037_Economy域_详细设计书.md` 占用；已核实 DTL-043~050 空闲，改用 043）。理由：DTL-019 标题本身就是"推送与兑换码"专用职责，不应稀释为消息分发大杂烩,与 REV-011 §2.4 提议一致。**直接进 1.0**（非 1.5 草案）——这是正式承接一个当前完全缺失的能力域，不是修补已有文档。下游动作：新建 DTL-043，PH-1 内完成首版。 |

---

### Q-D-02 [P0] Player 主表 DDL（players / player_characters / player_inventory）缺失

| 字段 | 内容 |
|---|---|
| 关联文档 | 5-DOMAIN-DTL-REVIEW-REPORT.md §1.2 §A.2.1/A.2.2（第 44-45 行）+ DTL-018 §2 + DTL-036 §6 待补齐项第 1 条（第 58 行）+ REV-011 §2.3 |
| 现状 | DTL-018 §2 含 5 张表（account_identity_links / identity_binding_audit_logs / compliance_profiles / identity_verification_vault / minor_restriction_audit_logs），**均非 player 主表**；`players` / `player_characters` / `player_inventory` 走 DTL-036 §6 第 1 条占位未启动；PH-1 编码（WF-1-54.6 等）需要这 3 张表 DDL 才能建 migration |
| 疑问 | (1) 这 3 张表归 DTL-018 v0.3（升版）还是新建 DTL-037/038/039 单独定义？<br>(2) `players.metadata` JSONB 是否含 `equipment_json` 这类反范式字段，还是 inventory 独立表外键？<br>(3) `player_characters.stats` 用 `JSONB` 还是 `JSONB + 字段化`（HP/ATK/DEF 拆 4 列）？ |
| 期望答复 | DDL 归属 + 字段级反范式决策 |
| 答复栏 | 🟡 **已核实代码现状**：`crates/player-service/migrations/0001_init.sql` 已建 `players`/`player_sessions` 表，但 `player_characters`/`player_inventory` 尚未创建——先代码后文档已经倒挂，需要补文档偿还技术债。(1) 新建 **DTL-044**（⚠️v0.2修正：初版误标 DTL-038，该号已被 `RGS-DTL-038_Match域_详细设计书.md` 占用，改用 044，与 Q-D-01 的 DTL-043 相邻顺延）（`players`/`player_characters`/`player_inventory`）单独定义，不并入 DTL-018（DTL-018 专注身份合规，职责不同）；同时反向为已有 `players`/`player_sessions` 补 DDL 文档说明。(2) `metadata` JSONB **不含** `equipment_json` 等反范式字段，inventory 走独立表 + 外键（背包物品有独立生命周期，需要行级操作与索引，塞进 JSONB 会让经济域对账困难）。(3) `player_characters.stats` 用 **JSONB + 字段化混合**：HP/ATK/DEF 等高频/需索引属性拆列，扩展性低频属性放 JSONB。下游动作：新建 DTL-044 + 反向补 `0001_init.sql` 已有表的文档说明 + 补 `player_characters`/`player_inventory` migration。 |

---

### Q-D-03 [P1] 4 位错误码命名空间在 5 域是否重叠

| 字段 | 内容 |
|---|---|
| 关联文档 | 5-DOMAIN-DTL-REVIEW-REPORT.md §1.7 §A.7.1（第 93 行）+ RGS-SPEC-CROSS-002 §2 + 7 份 DTL §3 错误码节 |
| 现状 | 7 份 DTL 5 份内嵌错误码 + DTL-019 §3 `DeliveryResultCode` 4 项 + DTL-031 §7.2 5 项（gRPC 标准枚举 ALREADY_EXISTS/ABORTED/...）；**未在 REV-004 附件 A 或专门一致性文档中定义 4 位错误码命名空间分配** |
| 疑问 | (1) 4 位错误码分配规则：前 2 位 = 域（01=player/02=economy/03=match/04=social/05=admin）+ 后 2 位 = 子类？<br>(2) 跨域公共错误（如 INTERNAL / UNAVAILABLE / DEADLINE_EXCEEDED）放哪里？<br>(3) DTL-031 §7.2 用了 gRPC 标准枚举，是否要再编码成 4 位？还是双层（gRPC status + 域内 code）？ |
| 期望答复 | 命名空间分配表 + 跨域公共码位置 |
| 答复栏 | 🟢 **⚠️v0.2修正**：初版答复漏查了 `RGS-SPEC-CROSS-001_错误码字典_v0.1.md`——它就是本题要找的"专门一致性文档"，已经存在（骨架状态，§4"待 NO-GO 解除后填充"），且**已预先规划了域×千位段编码**（§3 player=1001-1999 / §4 economy=2001-2999 / §5 match=3001-3999 / §6 social=4001-4999 / §7 admin=5001-5999 / §8 cluster-ops=6001-6999），与我初版提议的"2位域前缀+2位子类"是不同编码方案,应**采纳已有骨架的千位段方案**，不再另提新方案。(1) 命名空间＝骨架已定的千位段（见上）；子类段落在千位内部再细分（如 1001-1099 校验类/1100-1199 状态冲突类等），具体由 AI worker 按各域现有错误清点后填入 §3-§8。(2) 跨域公共错误：CROSS-001 §2 就是"通用错误码（0001-0999）"，不占用各域千位段,与 gRPC 标准 status 并存。(3) 双层：gRPC status（传输层）+ 域内 4 位 code（业务语义层，CROSS-001 §9 已规划"错误码↔gRPC status映射矩阵"章节）。**激活条件已满足**：已核实 `RGS-DEC-NOGO-001_一人公司NO-GO解除决议_v0.1.md`——NO-GO 已解除，G-CODE-03/G-CODE-06 均 🟢 Closed，CROSS-001 §5 三个门槛全部达成，**可直接升 v0.2 填实际编号表**，不再是待定项。下游动作：填充 CROSS-001 v0.2（不新建/不扩展 002，002 是 Proto 风格指南，职责不同）。 |

---

### Q-D-04 [P1] 5 域 RBAC 资源命名空间

| 字段 | 内容 |
|---|---|
| 关联文档 | 5-DOMAIN-DTL-REVIEW-REPORT.md §1.7 §A.7.4（第 96 行）+ RGS-SPEC-CROSS-007 §1 5 域 RBAC 角色矩阵 + DTL-031 §7.2 PERMISSION_DENIED 错误 |
| 现状 | DTL-031 §7.2 PERMISSION_DENIED 含"RBAC/审批依据不足"但未指定 5 域资源名；DTL-016 §2.2 引"工单处理执行权收口于既有 AdminService"但未列资源枚举 |
| 疑问 | (1) RBAC 资源粒度：<br>  - 行级（每玩家档案）？<br>  - 域级（player.* / economy.*）？<br>  - 动作级（player.read / player.write / player.audit）？<br>(2) GM / SRE / PM / 业务方的角色枚举在哪里定义（admin 域 DTL-031 §7 还是单独一份 RGS-RBAC-001）？<br>(3) 与 OPA / Casbin 集成是否在 PH-2 之后？ |
| 期望答复 | 资源粒度 + 角色枚举文档 ID + 与 OPA 时序 |
| 答复栏 | 🟢 **⚠️v0.2修正**：初版答复"新建 RGS-RBAC-001"是重复造轮子——本题"关联文档"字段本身已引用 `RGS-SPEC-CROSS-007_5域RBAC角色矩阵_v0.1.md`，该文档**已经存在**（骨架状态，§4"待 NO-GO 解除后填充"，与 CROSS-001 同一批建的占位文档，同样卡在 NO-GO/G-CODE-06/G-CODE-03 三个激活条件）,且其 §4 章节就叫"权限粒度规范（resource/action/scope）"——正是本题要答的问题,不需要另建文档。(1) 采用**动作级**（`player.read`/`player.write`/`player.audit`）为基础粒度，域级（`player.*`）作为聚合别名；**不做行级**（行级约束放业务逻辑层，塞进 RBAC 资源模型会导致枚举爆炸）——此结论填入 CROSS-007 §4，不是新文档。(2) 角色枚举**填入 CROSS-007**（不塞进 DTL-031 §7，也不新建 RGS-RBAC-001）：CROSS-007 §1"业务角色清单"+§2"admin域RBAC角色定义"+§3"5域→admin域角色映射矩阵"章节已经预留好位置。(3) 与 OPA/Casbin 集成 **PH-2 之后**：PH-1 先用枚举 + 中间件校验实现 fail-closed（phase-0-5 已验证过）。**激活条件已满足**（与 Q-D-03 共用同一份 `RGS-DEC-NOGO-001` 核实结论）。下游动作：填充 CROSS-007 v0.2。 |

---

### Q-D-05 [P1] PFAU R1 ~13 分钟 vs DTL-031 §4.3 待验证参数 300s/120s 数值差异

| 字段 | 内容 |
|---|---|
| 关联文档 | DTL-031 §4.3（第 191 行）+ handoff §4.3 R1 估算 + RGS-REV-003 §2.3 + RGS-ADR-0052 §4 |
| 现状 | DTL-031 §4.3 明确"300 秒观察窗口和 120 秒超时均为**待验证规划参数**，不是已承诺的 p99/SLA"；handoff §4.3 R1 估算 ~13 分钟（13min = 780s ≠ 300s+120s）|
| 疑问 | (1) R1 13min 的来源是 handoff 哪一段？是否含跨域 trace + admin 域 bcast + 5 域 ack 等待？<br>(2) 13min 是否在 NFR-AV 99.9% / RTO < 5min 约束下冲突？（13min > 5min RTO 上限）<br>(3) DTL-031 §4.3 "300s+120s" 需不需要在 PH-1 启动前冻结为承诺值？ |
| 期望答复 | 13min 公式拆解 + 与 NFR-AV 关系 + 是否需要 DTL 升版冻结 |
| 答复栏 | 🟡 **本次未重新展开 handoff §4.3 原始逐段计算**（不在本轮核验范围），以下为基于现有材料的判断，公式拆解需下游任务补齐。(1) 13min（780s）明显大于 300s+120s=420s，推断 R1 是**端到端最坏情况估计**，很可能包含 300s 观察窗 + 120s 超时之外的环节（admin 域广播确认 + 5 域 ack 等待 + 人工介入缓冲）；两者不是同一层面的数字，不矛盾但需要在 DTL 里显式写清楚二者关系,不能并列不加说明。(2) **字面确实与 NFR-AV RTO<5min 冲突**（13min > 5min），不应回避。倾向方案：**RTO 分级**而非强行压缩 R1——PFAU 这类需人工兜底的跨域联动故障走单独 RTO 分级（如 15min），不能与自动化可恢复路径共用同一个 5min 口径，业界常见做法是分级 SLA。(3) 冻结前必须先完成 (1)(2) 核验，否则冻结一组自相矛盾的参数没有意义。下游动作：升级为新 **RGS-DEC-NNN**（引用 Q-D-05），先做 RTO 分级论证 + 13min 公式拆解，再冻结 300s/120s。 |

---

### Q-D-06 [P1] Active-Active 假设 vs DAU 100k / QPS 10k 容量基线

| 字段 | 内容 |
|---|---|
| 关联文档 | ADR-0052 Active-Active + all-reachable PFAU + REV-011 §2.1 风险段 + NFR-OP-010（待核验）|
| 现状 | ADR-0052 设 ClusterOpsService Active-Active multi-leader（all-reachable PFAU 拓扑），流量分摊到 2 副本；NFR-OP-010 假设 DAU 100k / QPS 10k 单副本容量 |
| 疑问 | (1) Active-Active 后单副本容量是 50k DAU / 5k QPS 还是 100k / 10k（双副本共 200k / 20k）？<br>(2) PFAU 13min R1 假设在 Active-Active 下是否变成 2 副本分别跑导致状态机冲突？<br>(3) REV-011 §2.1 风险段"若 Active-Active 拆分流量，DAU 100k 需重算"是否已重算？ |
| 期望答复 | 单/双副本容量公式 + PFAU 状态机冲突验证 |
| 答复栏 | 🟢 (1) **双副本共享总容量**（100k DAU/10k QPS 是总量，不是每副本各自再担 100k/10k）。若目标总容量为 100k/10k，单副本设计容量约 **50-70k DAU / 5-7k QPS**（留故障切换后单副本临时扛全量的缓冲，不能卡在刚好 50%）。(2) **会冲突**：all-reachable PFAU 拓扑下两副本各自独立跑状态机存在冲突风险，必须有仲裁机制（leader lease / 分布式锁 / CRDT 式收敛），这是 ADR-0052 需要补充的实现细节，不能假设 multi-leader 天然无冲突。(3) **需要重算**，且应与 (1)(2) 一并在 ADR-0052 补一次修订，不要散落多文档。下游动作：ADR-0052 修订版 + 容量重算（可并入 RGS-TS-001 或新建 RGS-CAP-001）。 |

---

### Q-D-07 [P1] DTL-026 §7 Glicko-2 评分算法的 Rust crate 选型

| 字段 | 内容 |
|---|---|
| 关联文档 | DTL-026 §7.1（第 295-302 行）+ RGS-TS-001 §3.5 + REV-011 §2.6（5 域监控含 DTL-026 rating）|
| 现状 | DTL-026 §7.1 选 Glicko-2（v0.3 修正 volatility 持久化），但未指定 Rust crate；社区有 `glicko-rs` / `glicko2` / 自实现三种路径 |
| 疑问 | (1) 用 `glicko-rs` 0.4.x（功能全但依赖较多）还是自实现（约 200 行 Rust）？<br>(2) 是否需要维护 `volatility` 持久化（v0.3 修正）跨版本兼容？<br>(3) `rating_deviation` 与 `volatility` 的 decay 公式参数（τ=0.5 等）走 DTL §7.1 默认还是需调优？ |
| 期望答复 | crate 选型 + 是否写 RGS-CRATE-EVAL-001（crate 选型评估单）|
| 答复栏 | 🟢 **已核实**：workspace 内无 glicko crate 依赖，match-service 无既有实现，属绿地决策。(1) **自实现**（约 200 行），不用 `glicko-rs`：理由——外部 crate 更新不活跃/依赖偏多；项目本身需要自定义 volatility 持久化（DTL-026 §7.1 v0.3 修正），第三方内部状态模型未必与我们持久化 schema 对齐；算法公式明确、可测试性强，自实现更可控。(2) volatility 持久化**需要**跨版本兼容：把 `rating`/`rd`/`volatility` 三元组作为持久化契约的一部分，任何算法参数调整都要走数据迁移评估。(3) τ 等 decay 参数先用 DTL §7.1 默认值起步，PH-1 不做调优（运营阶段的事）。**不需要**单独写 RGS-CRATE-EVAL-001（自实现不涉及第三方 crate 选型评估）。下游动作：DTL-026 §7.1 补充"自实现"决策说明。 |

---

### Q-D-08 [P1] 4 渠道抽象（站内信 / 邮件 / 推送 / 短信）缺失

| 字段 | 内容 |
|---|---|
| 关联文档 | 5-DOMAIN-DTL-REVIEW-REPORT.md §1.5 §A.5.2（第 73 行）+ DTL-019 §3 §1.2 + RGS-TS-001 §4 消息/推送选型 |
| 现状 | DTL-019 §3 `PushDeliveryRequest` proto 字段 5 项（122-128 行），**仅 1 渠道（推送）**；DTL-019 §1.2 明确"不覆盖 APNs/FCM 第三方网关适配层"；4 渠道抽象（站内信/邮件/推送/短信）未在 DTL 层抽象 |
| 疑问 | (1) 4 渠道抽象放在 DTL-019 v0.2 / 新 DTL-X / 跨域 DTL-021~025 哪里？<br>(2) 第三方网关（APNs/FCM/SMTP/SMS）适配层是否 PH-1 范围？<br>(3) 渠道失败重试策略（push 不重试 vs 邮件 3 次重试）放在哪一层？ |
| 期望答复 | 抽象归属 + 第三方网关时序 + 重试策略文档 ID |
| 答复栏 | 🟢 (1) 放 **DTL-019 v0.2**（不新建独立文档，也不放跨域 DTL-021~025）——DTL-019 已是推送渠道归属文档,扩展为 4 渠道是同一职责的自然延伸。**与 Q-D-01 不冲突**：Q-D-01 拆走的是"站内信"三张关系表（messages/message_recipients/conversations 这个业务对象），本题是"多渠道下发能力"这个技术抽象层，二者可以在同一 DTL-019 v0.2 的不同章节共存。(2) 第三方网关适配层（APNs/FCM/SMTP/SMS）**不在 PH-1 范围**，PH-1 先定义抽象接口/枚举，用 mock/stub 网关跑通链路。(3) 重试策略放 DTL-019 v0.2 本身（渠道能力定义的一部分）：push 不重试（用户体验优先），邮件/短信 3 次指数退避重试（到达率优先）。下游动作：DTL-019 升版 v0.2（与 Q-D-01 分离决策一并处理）。 |

---

### Q-D-09 [P1] 跨域事件订阅契约缺失

| 字段 | 内容 |
|---|---|
| 关联文档 | 5-DOMAIN-DTL-REVIEW-REPORT.md §1.5 §A.5.4（第 75 行）+ DTL-019 §4.1/§4.3 + DTL-020 §4.4 + DTL-026 §3 事件 + RGS-SPEC-CROSS-003 跨域事件 Schema 字典 |
| 现状 | DTL-019/020/026 跨域引用隐含（§3 事件线 / §4.1 推送 / §4.4 退款）；**未显式声明事件订阅关系与契约**（谁发布 → 谁订阅 → NATS subject 命名）|
| 疑问 | (1) NATS subject 命名规则（per RGS-SPEC-CROSS-003）是否需要按"域.事件.动作"形式（如 `economy.trade.completed` / `player.session.created`）？<br>(2) 跨域事件是否要求至少一次 + 幂等消费，还是精确一次？<br>(3) DTL-026 §3 `MatchRatingChanged` 是否会同时触发 player 域 `PlayerRatingUpdated` 缓存失效？ |
| 期望答复 | NATS subject 命名 + 投递语义 + 跨域事件清单 |
| 答复栏 | 🟢 **已核实代码现状**：`async-nats` 已在 5 域生产代码使用，`shared-platform/src/{producer,consumer,messaging,outbox_relay}.rs` 已实现发布/消费基础设施。(1) 采用 **"域.对象.动作"命名**（`economy.trade.completed`/`player.session.created`），域名用现有短名（player/economy/match/social/admin/cluster_ops）；实际分隔符按 `producer.rs` 现有实现为准,不推翻已落地代码。(2) **至少一次 + 幂等消费**：JetStream 天然 at-least-once,精确一次代价过高；Outbox 幂等 check migration 已在 5 域落地（`000X_outbox_check_idempotent.sql`），消费侧用事件 ID 做幂等键即可，这是既有能力的确认而非新增工作。(3) **是**，`MatchRatingChanged` 应触发 player 域 `PlayerRatingUpdated` 缓存失效，这类订阅关系需要显式登记。下游动作：补全 RGS-SPEC-CROSS-003 跨域事件 Schema 字典（列出已知发布者→订阅者关系，含此例）。⚠️ 见 RGS-OPEN-QA-001-ACTIONS-v0.3.md §5 修正 #7：应沿用 RGS-SPEC-CROSS-003 v0.1 §2.2 已有的 `rgs.events.<domain>.<aggregate>.<action>.<version>` 命名（如 `rgs.events.economy.wallet.committed.v1`），不要新造"域.对象.动作"方案。 |

---

### Q-D-10 [P2] DTL-026 §4.1 撮合复杂度 O(n²) 与 NFR-PT 100ms 性能预算的实测量化

| 字段 | 内容 |
|---|---|
| 关联文档 | DTL-026 §4.1 + §4.2 + §5 OCC（第 175-251 行）+ NFR-PT 单局决策 ≤ 100ms + 5-DOMAIN-DTL-REVIEW-REPORT §1.4 §A.4.3（第 64 行）|
| 现状 | DTL-026 §4.2 撮合复杂度 O(n²) 候选筛选，n 大时存在性能风险；§4.1 容差函数为**初始提案**未调优；DTL-026 §4.1 标"容差参数终值"为不覆盖项 |
| 疑问 | (1) n 上限 = 多少时保证 100ms？需不需要 PH-1 启动后跑 benchmark（per WF-1-55.X 提议）？<br>(2) 容差函数参数（score_distance / latency_budget / skill_bracket）走 DTL §4.1 默认还是 SPEC 单独调优？<br>(3) 如果 n > 上限，是降级（拆分撮合轮）还是熔断（返回 RetryAfter）？ |
| 期望答复 | n 上限 + 调优文档 ID + 降级/熔断策略 |
| 答复栏 | 🟢 (1) 需要 PH-1 启动后跑 **benchmark** 才能给出可信 n 上限，不应拍脑袋定数字；临时占位上限 **n≤500**（100ms/500²≈0.4μs/pair，对纯内存候选筛选是宽松量级，仅供 benchmark 前占位）。(2) 容差函数参数走 DTL §4.1 默认值起步，不单独开 SPEC，调优结果反哺回 DTL 升版即可。(3) **降级优先于熔断**：n 超上限先拆分撮合轮（分桶降低单轮 n），降级后仍超预算才熔断返回 RetryAfter——直接熔断对玩家体验不友好，应作最后手段。下游动作：PH-1 内新增 benchmark 子任务 + DTL-026 §4.1 补 n 上限与降级策略。 |

---

## 3. B 组：制造/编程疑问（10 条）

### Q-M-01 [P0] Saga 步骤编号 1.0~6.0 在 DTL-015/016 哪个章节加

| 字段 | 内容 |
|---|---|
| 关联文档 | RGS-REV-005 附件 B Saga 6 场景 + DTL-015 §3.1 + DTL-016 §3.3 + DTL-031 §8.2 + RGS-IMPL-100 v0.1 + REV-011 §2.5（WF-1-55.39 提议）|
| 现状 | DTL-015 §3.1 "执行_atomic_transfer" 隐含 4 步（269 行）；DTL-016 §3.3 同样 1 步；REV-005 附件 B 已演练 6 场景；**两份 DTL 均未编号步骤**；Q-003 审批（per WF-1-55.40 RGS-DEC-Q003 v0.1）未完成 |
| 疑问 | (1) 步骤编号放 DTL §3.1/§3.3 内部还是新增 §3.4 "Saga 步骤编号映射"？<br>(2) 6 场景步骤编号（per REV-005 附件 B）是 1.0~6.0 还是 1.x 嵌套？<br>(3) Q-003 审批包（per WF-1-55.40 RGS-DEC-Q003 v0.1）与 DTL 步骤编号**先后顺序**：先 DTL 升版还是先 RGS-DEC-Q003？ |
| 期望答复 | 章节位置 + 编号格式 + Q-003 / DTL 升版先后 |
| 答复栏 | 🟢 (1) 新增 **§3.4「Saga 步骤编号映射」**（不塞进已有 §3.1/§3.3 内部）——编号映射是跨多步骤的横切说明，单独成节更清晰，未来新增场景只改 §3.4 不动正文。(2) 采用 **1.0~6.0**（对应 REV-005 附件 B 6 个场景，每场景一个整数段），场景内部子步骤用 1.1/1.2/1.3 嵌套，两者不矛盾：整数段=场景，小数段=场景内步骤。(3) **先 DTL 升版，后 RGS-DEC-Q003**：DTL 编号是纯文档结构化工作不涉及决策取舍，DEC-Q003 审批包依赖编号后的 DTL 作引用基础，顺序反过来会让 DEC-Q003 引用一个不稳定的编号。下游动作：DTL-015 §3.4 + DTL-016 §3.4 新增，随后完成 RGS-DEC-Q003。 |

---

### Q-M-02 [P0] PgTestDatabase fixture 的 `#[sqlx::test]` 强约束在 5 域统一应用范围

| 字段 | 内容 |
|---|---|
| 关联文档 | RGS-REV-009 CR-3 + WF-1-55.27 f6a6f3f commit + RGS-IMPL-001 §3 测试约定 + REV-011 §2.5 |
| 现状 | WF-1-55.27 引入 `PgTestDatabase` fixture + `#[sqlx::test]` 强约束（仅 economy-service --lib 50/50 pass）；其他 4 域（player/match/social/admin）尚未统一 |
| 疑问 | (1) `#[sqlx::test]` 强约束是**全 5 域统一**还是仅 **economy 域硬约束**？<br>(2) 单元测试（无 DB）是否允许 `#[tokio::test]` 不用 `#[sqlx::test]`？<br>(3) 集成测试（IT）是否走 `tests/` 目录独立 `#[sqlx::test]`？ |
| 期望答复 | 5 域统一范围 + 单元/集成测试规则 |
| 答复栏 | 🟡 **已核实代码现状（现状描述已过时）**：`rgs-testkit` 现已支持 `FixtureBuilder` + 5 域（commit 7592b66/71a47b0/e280967 已合并），但实际只有 **economy-service** 在自己 `tests/` 目录接入了 `rgs-testkit`（`integration_outbox.rs`）；player/match/social/admin 4 域尚未在各自 Cargo.toml dev-dependencies 接入——基础设施已就绪，采纳是待补的执行工作，不是待决策的范围问题。(1) **全 5 域统一**（非仅 economy 硬约束），否则会出现"economy 严格、其余域宽松"的质量洼地。(2) 无 DB 的纯单元测试**允许** `#[tokio::test]`，不强制 `#[sqlx::test]`。(3) 集成测试**走 `tests/` 目录独立 `#[sqlx::test]`**（`integration_outbox.rs` 即样板，直接复制推广）。下游动作：4 域各补 rgs-testkit dev-dependency + 同款集成测试骨架（约 4×0.5 人天）。 |

---

### Q-M-03 [P0] OTel SDK 集成路径在 5 域独立 binary 下的 trace_id 跨进程传播

| 字段 | 内容 |
|---|---|
| 关联文档 | RGS-TS-001 §6 OTel 选型 + RGS-INC-001 v0.2 §4 + RGS-GOBS-100 v0.1 + RGS-SPEC-CROSS-006 trace_id 命名规范 + REV-011 §2.6 |
| 现状 | 5 域独立 binary（player / economy / match / social / admin）+ 1 cluster-ops；OTel SDK 跨进程 trace_id 传播需要 W3C TraceContext `traceparent` header 透传；gRPC metadata 透传已有，但 NATS message header 需手动加 |
| 疑问 | (1) NATS message header 是否加 `traceparent`（OpenTelemetry 规范）？需要 NATS SDK 升级到支持 header 的版本？<br>(2) Postgres query span 是否开启（sqlx-tracing 集成）？开启后 p50/p99 延迟影响？<br>(3) 5 域 binary 各自上报到 OTel Collector，还是走 cluster-ops 统一 OTLP 出口？ |
| 期望答复 | NATS header 规则 + sqlx-tracing 开关 + OTLP 出口拓扑 |
| 答复栏 | 🟡 **已核实**：workspace `Cargo.toml` 注释标注"opentelemetry 启用待 53.12 OTel SDK 接入（54.13）"，说明 OTel 启用本身还在依赖一个待办任务，本答复给出方向，最终落地要看该任务状态。(1) NATS message header **加 `traceparent`**：`async-nats` 0.42（已在用）原生支持 header（NATS 2.2+ 特性），**不需要升级依赖版本**，只需应用层在 `producer.rs`/`consumer.rs` 手动注入/提取 W3C TraceContext。(2) sqlx query span **开启但用采样率控制**（建议 PH-1 先 10-20% 采样，验证延迟影响可接受后再提高），避免可观测性拖累 p99。(3) 5 域**各自直接上报 OTLP**（不经 cluster-ops 中转），中转会引入单点瓶颈和额外一跳延迟。下游动作：核实 53.12/54.13 任务状态；若未完成需先落地才能实施本题决策。 |

---

### Q-M-04 [P0] mTLS 兼容修复 RGS-OPS-101 的 grpc_health_probe 在 5 域 manifest 配置一致性

| 字段 | 内容 |
|---|---|
| 关联文档 | RGS-OPS-101 v0.1（commit 66ff53b，26 文件 / 685 行）+ 5 域 k8s manifest（per phase-0-5 step 1+5）+ 6 域 mTLS Secret（per phase-0-5 step 4 50-secret-*）|
| 现状 | RGS-OPS-101 修复了 grpc_health_probe 与 mTLS 的兼容问题；phase-0-5 step 1+5 推送了 6 域 manifest（01-player-service.yaml ~ 06-cluster-ops-service.yaml）；但**5 域 manifest 的 liveness/readiness probe 配置是否一致**（同一个 helm 模板 vs 各自手写）？ |
| 疑问 | (1) 5 域 manifest 走 Helm template 共享 probe 配置，还是各自 YAML 手写？<br>(2) RGS-OPS-101 修复后是否需要在 5 域 manifest 同步加 `--tls` / `--tls-ca-cert` 参数？<br>(3) probe 失败重试次数 / 超时阈值在 5 域是否一致？ |
| 期望答复 | probe 配置模板化 vs 分散决策 + RGS-OPS-101 修复同步范围 + 阈值统一性 |
| 答复栏 | 🟡 **已抽查核实**：`01-player-service.yaml` 与 `02-economy-service.yaml` 的 probe 段结构完全一致（同一套 `grpc_health_probe -tls -tls-client-cert/-tls-client-key/-tls-ca-cert` 参数，仅 `-tls-server-name` 按域替换），说明现状是**手写但保持一致模板**（非 Helm）。**仅抽查 2/6 份**，其余 4 份未逐一核对,不能断言全部一致。(1) 现状**手写非 Helm**；PH-1 暂不引入 Helm（为 6 个近乎相同的 manifest 引入工具链复杂度收益不明显），但要求任何一份 probe 段修改必须同步到其余 5 份，用脚本/CI 校验而非人工记忆。(2) `-tls` 系列参数**已同步**（抽查的 2 份均有），无需额外动作。(3) **阈值一致性未完全核实**，需要补一次全 6 份 diff。下游动作：写 CI/pre-commit 脚本 diff 6 份 manifest 的 probe 段，并补全 6 份核对（本次仅验证 2 份）。 |

---

### Q-M-05 [P1] mTLS Secret 在 6 域的命名空间分配

| 字段 | 内容 |
|---|---|
| 关联文档 | phase-0-5 step 4（commit 765930a）+ `docs/deploy/01-k8s-manifests/50-secret-*.yaml`（player/economy/match/social/admin/cluster-ops 6 份）|
| 现状 | 6 域 mTLS Secret 已创建（50-secret-player-tls.yaml ~ 50-secret-cluster-ops-tls.yaml）；Secret type = `kubernetes.io/tls`；命名约定 `<domain>-tls` |
| 疑问 | (1) 6 域 Secret 是放在同一 namespace（如 `rgs-system`）还是各自 namespace（如 `rgs-player` / `rgs-economy`）？<br>(2) CA 证书（50-secret-ca.yaml）单例还是按域分？<br>(3) Secret 轮转（rotation）策略：手工 kubectl apply 还是 cert-manager 自动？ |
| 期望答复 | namespace 分配 + CA 拓扑 + 轮转自动化范围 |
| 答复栏 | 🟢 **已核实**：`50-secret-player-tls.yaml` 的 `namespace: rgs`（单一共享 namespace），`50-secret-ca.yaml` 仅 1 份。(1) **同一 namespace（`rgs`）**，与 `deploy_dev_k3s.ps1` 单一 namespace 部署现状一致，保持现状——拆分按域 namespace 会增加 NetworkPolicy/RBAC 复杂度，PH-1 阶段收益不明显。(2) **CA 证书单例**（已验证只有 1 份 `50-secret-ca.yaml`），6 域共用一条信任链，符合当前单集群内部信任域拓扑。(3) 轮转策略 **PH-1 手工 `kubectl apply`**（未发现 cert-manager 部署迹象），自动化轮转是 PH-2 增强项，不阻塞上线，但需要运维手册记录手工轮转 SOP 与证书有效期提醒。下游动作：补充证书轮转 SOP（可并入现有运维文档）。 |

---

### Q-M-06 [P1] RGS-IMPL-100 Saga 实施规范的 Rust crate 选型（outbox / async-trait / tokio）

| 字段 | 内容 |
|---|---|
| 关联文档 | RGS-IMPL-100 v0.1 + RGS-TS-001 §3 主要技术选型 + DTL-016 §3.3 |
| 现状 | RGS-IMPL-100 v0.1 给出 Saga 实施规范但未指定具体 crate；outbox 模式有 `sqlx-outbox` / `outbox-pattern-rs` / 自实现三种；async-trait 在 Rust 1.75+ 已稳定（但 RUST toolchain 1.98）|
| 疑问 | (1) outbox crate 选 `sqlx-outbox` 0.x 还是自实现（建 `outbox_events` 表 + 后台 poller）？<br>(2) Saga 步骤的 `async fn` trait 是否用 `async-trait` 还是 native AFIT（Rust 1.75+ stable）？<br>(3) 死信队列（DLQ）落库 vs 落 NATS JetStream？ |
| 期望答复 | crate 选型 + AFIT 决策 + DLQ 位置 |
| 答复栏 | 🟢 **已核实代码现状**：5 域均已用自建 `outbox_events` 表 + `000X_outbox.sql`/`000X_outbox_check_idempotent.sql` migration,workspace 内**无 `sqlx-outbox` 依赖**——outbox 早已是自实现且已落地跑通（economy-service 集成测试通过）。(1) **自实现**（确认既定路径，不需再评估 `sqlx-outbox`）。(2) **native AFIT**（Rust 1.75+ 已稳定，项目 toolchain 1.98 完全支持），不用 `async-trait` 做 Saga step trait——`async-trait` 目前只是其他位置（如 mock 层）的既有依赖，Saga trait 本身应用原生 `async fn in trait` 减少 `Box<dyn Future>` 分配开销。(3) **DLQ 落库**（不落 JetStream）：便于人工查询/重放，与现有 outbox 落库习惯一致，避免给 NATS 叠加另一套语义。下游动作：RGS-IMPL-100 补充 crate 选型确认段落（代码已是现状，无需变更）。 |

---

### Q-M-07 [P1] ReserveHandler OCC 修复后（WF-1-55.27）的回归测试覆盖

| 字段 | 内容 |
|---|---|
| 关联文档 | RGS-REV-009 CR-1 + WF-1-55.27 commit c96efe8（reservation.rs + saga_orchestrator.rs +159/-4）+ 50/50 cargo test pass |
| 现状 | WF-1-55.27 修了 ReserveHandler OCC cleanup + reservation release 失败路径；CR-1 验收测试 50/50 pass（per WBS-001 v0.5 §4）；CR-2/3 也合并 |
| 疑问 | (1) 50/50 是单元测试还是含集成？需要补 `tests/it_*.rs` 端到端？<br>(2) reservation release 失败路径的混沌测试（chaos test）覆盖：DB 突然断开 / Reservation row 被外部 DELETE / 死锁，3 个场景是否要进 IT？<br>(3) 修复后 OTel span 是否完整（reservation.create → saga.step → reservation.release / cleanup）？ |
| 期望答复 | 50/50 范围 + 混沌测试场景 + OTel span 完整度 |
| 答复栏 | 🟢 (1) 50/50 目前是**单元测试范围**（`--lib`），需要补 `tests/it_*.rs` 端到端集成测试覆盖 reservation 完整生命周期（create→conflict→release/cleanup）——明确缺口，不应视为已覆盖。(2) 3 个混沌场景**都应进 IT**，但优先级不同：DB 突然断开 / 死锁是生产真实会发生的场景（**P1，PH-1 内完成**）；"row 被外部 DELETE"更多是防御性测试（**P2，可推后**）。(3) OTel span 完整度需**实测验证**（不能只凭代码审查判断"应该完整"），建议在 IT 测试里加断言校验 span 树结构（`reservation.create → saga.step → reservation.release/cleanup` 三层嵌套）。下游动作：新增子任务：reservation 集成测试 + 混沌测试（DB断开/死锁先做）+ span 断言。 |

---

### Q-M-08 [P2] fail-closed 验证脚本在新增业务域后是否需要重跑

| 字段 | 内容 |
|---|---|
| 关联文档 | phase-0-5 step 4（commit 765930a）+ RGS-SEC-100 v0.1 GM 审计与 Saga 安全 + 07-no-go-checklist_business v0.3 |
| 现状 | phase-0-5 step 4 跑了 fail-closed 验证（5 域 mTLS + Secret + RBAC fail-closed）；4 B-CODE 全部 Closed |
| 疑问 | (1) PH-1 期间新增域（5 → 6 域或后续 7 域）是否需要重跑 fail-closed？<br>(2) fail-closed 验证脚本是 `scripts/verify-fail-closed.ps1`（建议命名）一次性还是纳入 CI？<br>(3) RBAC 资源新增时 fail-closed 行为是否仍生效（默认拒绝 vs 默认放行）？ |
| 期望答复 | 重跑触发条件 + 脚本命名 + 默认拒绝决策 |
| 答复栏 | 🟢 (1) **需要**：任何新增域触发一次全量 fail-closed 重跑，不能假设新域自动继承现有域的安全基线。(2) 建议**纳入 CI**（不是一次性脚本），命名 `scripts/verify_fail_closed.ps1`（下划线风格，与项目现有 `deploy_dev_k3s.ps1` 命名习惯一致），在每次 manifest/RBAC 变更的 PR 上跑，不仅限新增域时。(3) **默认拒绝**——fail-closed 语义本身就是默认拒绝,新增 RBAC 资源在显式授权前一律不可访问,这是既定原则不应动摇。下游动作：把 fail-closed 验证脚本化并接入 CI（当前是手工一次性验证，需固化）。 |

---

### Q-M-09 [P2] DTL-019 v0.2 拆分后 v0.2/v0.3 版本号语义

| 字段 | 内容 |
|---|---|
| 关联文档 | REV-011 §2.4 选项 A 提议（WF-1-55.38）+ DTL-019 当前 v0.1 + DTL 升版规范（per DTL 头表）|
| 现状 | 若选选项 A：DTL-019 v0.1 → v0.2（去掉消息分发）+ 新建 DTL-X v0.1（消息分发）；版本号语义是按"内容变更程度"还是"修订次数"？ |
| 疑问 | (1) 拆出去 + 加内容是否升 minor（v0.1 → v0.2）？<br>(2) DTL-X 新建 v0.1 是首版还是草案？<br>(3) 跨文档引用（SPEC-DTL-019 / DTL-021~025）版本号同步如何保证？ |
| 期望答复 | 版本号语义规范 + 跨文档引用同步机制 |
| 答复栏 | 🟢 (1) 拆分（去掉消息分发）与加内容（Q-D-08 渠道抽象）若同批完成，**合并算一次 v0.1→v0.2**，不需要为拆分和加内容分别升两次版。(2) 新建 DTL-043（消息分发，⚠️v0.2修正：初版误标 037，见 Q-D-01 修正说明）**从 v0.1 起步**（首版，非草案）——按 Q-D-01"直接进 1.0 状态"的决策，"状态标记"（1.0/1.5）与"版本号"（v0.1）是两个独立维度，不要混淆。(3) 跨文档引用同步**不追求实时自动化**（成本过高），约定"任何 DTL 升版时必须 grep 全仓库该 DTL 编号引用，逐一更新版本号标注"作为升版 checklist 固定一步。下游动作：DTL 升版规范补充"引用同步 checklist"一条。 |

---

### Q-M-10 [P2] NATS / Redis Stream 选型对 Outbox 模式 schema 的影响

| 字段 | 内容 |
|---|---|
| 关联文档 | RGS-TS-001 §5 消息队列选型（未决）+ RGS-SPEC-CROSS-003 跨域事件 Schema + RGS-IMPL-100 v0.1 §3 |
| 现状 | RGS-TS-001 §5 消息队列候选：NATS JetStream（推荐）/ Redis Stream；Outbox 模式 schema 与消息队列 header 格式相关；DTL-019 §1.2 "不覆盖消息队列选型" |
| 疑问 | (1) NATS JetStream vs Redis Stream 决策何时落地（PH-1 / PH-2）？<br>(2) Outbox 表 schema（payload JSONB / metadata JSONB / trace_id 列）是否随队列选型变化？<br>(3) 若选 NATS，msg header 格式（per RGS-SPEC-CROSS-003）需不需要先冻结？ |
| 期望答复 | 选型时序 + Outbox schema 稳定性 + header 规范冻结 |
| 答复栏 | 🟢 **现状描述已过时**：`docs/deploy/01-k8s-manifests/30-nats-{configmap,networkpolicy,pvc,sa,service,statefulset}.yaml` 已合并部署，`async-nats` 已是 5 域生产代码实际使用的库（`main.rs` 全部导入），`InMemoryNatsMock` 已合并作为测试基础设施——**NATS JetStream 已经落地，不是"未决"**。(1) 选型已定，**Redis Stream 不再需要评估**。(2) Outbox schema **已经稳定**（`000X_outbox.sql` + idempotent check 已在 5 域落地并通过集成测试），不会因队列选型再变化。(3) msg header 格式**仍需在 RGS-SPEC-CROSS-003 补充冻结**（结合 Q-M-03/Q-D-09 的 `traceparent` header 需求一并定义），这是唯一未完成部分。下游动作：RGS-TS-001 §5 状态改为"已决策：NATS JetStream"（去掉"未决"标注）+ RGS-SPEC-CROSS-003 冻结 header 格式。 |

---

## 4. C 组：治理/流程疑问（4 条）

### Q-G-01 [P0] 一人公司 5 域 Lead 兼任与 DEC-005 "不兼任" 原则的兼容性

| 字段 | 内容 |
|---|---|
| 关联文档 | DEC-005（5 域独立 Lead 不兼任）+ DEC-008（一人公司 12 角色兼任）+ RGS-PLAN-001 v1.0 §1.2 + user profile 2026-08-21 偏好 |
| 现状 | DEC-005 拒绝"架构师兼任 player / SRE 兼任 admin"兼任方案；DEC-008 接受"一人公司 = Ulysses 12 角色兼任，已知代价由流程化补偿"；当前 RGS-PLAN-001 v1.0 §1.2 写"5 域独立 Lead 不兼任(per DEC-005); 1 人公司 = Ulysses 12 角色兼任(per DEC-008,已知代价由流程化补偿)" |
| 疑问 | (1) DEC-005 与 DEC-008 是否矛盾？<br>(2) "流程化补偿"具体指哪些流程（per plan §1.2 提到 CI 强约束 + 自动化测试 + 自我 PR review + OTel 链路）？<br>(3) PH-1 实施期间是否需要新增 RACI 矩阵明确每件事 1 主 + 1 复审？ |
| 期望答复 | DEC-005/008 兼容论证 + 流程化补偿清单 + RACI 矩阵 |
| 答复栏 | 🟢 (1) **不矛盾，二者作用在不同层**：DEC-005 是"组织设计原则"（多角色制度下不应让同一自然人同时坐架构师和 player Lead 两把椅子，避免自我审查/自我批准的治理风险）；DEC-008 是"当前实际执行约束"（现实只有 Ulysses 一个自然人，理想的组织设计目前不可得，需要补偿机制）。前者定义理想状态角色应如何分配，后者承认理想状态目前达不到,不是同一命题的两个相反答案。(2) 流程化补偿具体指：**CI 强约束**（阻止不合规代码合并,替代人工跨角色审查）+ **自动化测试**（替代部分人工验收）+ **自我 PR review**（有 diff 留痕可追溯,比完全没有 review 记录强）+ **OTel 链路**（生产问题可追责到代码变更）。这份清单已写在 RGS-PLAN-001 §1.2，需要从"提及"升级为"正式登记"（逐项标注负责的 CI/工具）。(3) **需要新增 RACI 矩阵**：一人公司下"R"永远是 Ulysses/AI worker，但"A"（最终批准）/"C"（咨询）/"I"（知会）仍有区分意义——涉及资金/合规的决策需要"A=Ulysses 本人明确签字"，不能让 AI worker 自我 PR review 顶替，这是防止自我审查失控的实质防线。下游动作：升级为 **ADR**（RGS-ADR-00XX，登记 DEC-005/008 关系论证）+ RGS-PLAN-001 §1.2 补 RACI 简表（按决策类别：代码合并/DTL升版/生产发布/资金相关）。 |

---

### Q-G-02 [P1] RGS-REV-011 8 个新 L4 任务（WF-1-55.32~41）owner 全是 Ulysses 的执行机制

| 字段 | 内容 |
|---|---|
| 关联文档 | REV-011 §3 提案汇总 + RGS-WT-001 v0.2 §11 worktree 隔离 + user profile 2026-08-21 "派 worker 子代理" 偏好 + token-OLU 框架 |
| 现状 | REV-011 §3 提议 8 个 L4 任务（WF-1-55.32~41）owner 全是 Ulysses（player/economy/match/social/admin 域 Lead）+ Platform Lead（Ulysses）；总 ~64K tokens（0.5-1 人·周 AI 协作）|
| 疑问 | (1) 8 个 L4 任务是**串行** Ulysses 自做，还是**并行**派 worker 子代理（per user profile 偏好）？<br>(2) 如果派子代理，子代理在 worktree 内的权限范围（写 DTL / 改 SPEC / 跑 cargo test）？<br>(3) Ulysses "实际签" 与 worker 子代理 "实做" 的责任矩阵如何区分（per DEC-008 一人公司治理）？ |
| 期望答复 | 串行/并行决策 + 子代理权限 + 责任矩阵补充 |
| 答复栏 | 🟢 (1) **并行派 worker 子代理**（采纳已知偏好，user profile 2026-08-21 记录的派发子代理偏好）——串行 8 个任务对一人公司是不必要的时间瓶颈，worktree 隔离机制（RGS-WT-001）本来就是为此设计。但并行度建议控制在 **3-4 个同时进行**（受限于 Ulysses 本人作为最终审核瓶颈的吞吐——审核比生产慢，并行太多只会堆积待审核积压）。(2) 子代理在 worktree 内**可以**写 DTL/改 SPEC/跑 cargo test（本地验证循环需完整），但**不能**自主合并到 main（必须走 PR + 至少一次独立 review 关卡）、不能改 CI 配置本身、不能执行有资金/生产影响的操作——与 phase-0-5 反馈单里观察到的实际执行模式一致。(3) 责任矩阵：**子代理 = R（执行），Ulysses = A（批准/最终合并决策权）**，与 Q-G-01 的 RACI 矩阵是同一份不需另立。下游动作：Q-G-01 的 RACI 矩阵补一行覆盖 WF-1-55.32~41 这 8 个任务。⚠️ 见 RGS-OPEN-QA-001-ACTIONS-v0.3.md §5 修正 #5：REV-011 §3 实际是 10 个任务（WF-1-55.32~41），且 55.32~37 已被 `RGS-WBS-001_瀑布式工作分解结构_v0.3.md` 既有任务占用，新 L4 任务需从 WF-1-55.38 起重新编号。 |

---

### Q-G-03 [P1] token-OLU 框架在 RGS-WT-001 worktree 模式下的分摊

| 字段 | 内容 |
|---|---|
| 关联文档 | RGS-TS-001 §6.2 token-OLU 框架 + RGS-WT-001 v0.2 §11 + user profile 2026-08-21 |
| 现状 | token-OLU 框架：1 人·天 ≈ 100K-300K tokens；1 SRE 上限 = 1 人·周 ≈ 1M tokens；5 域独立 Lead × 14-18 周 = 80-120M tokens（待 SRE Lead + PM 校准）|
| 疑问 | (1) token 是否按 worktree 独立计数（每个 worktree 1M tokens 独立）还是共享计数（总池 5M tokens 跨 worktree）？<br>(2) 跨 worktree 的决策对话（不在 worktree 内的会话）是否计入 token？<br>(3) "AI 上下文窗口" 与 "单次会话成本" 哪个是硬约束？ |
| 期望答复 | worktree 独立/共享 + 跨会话决策计入 + 硬约束优先级 |
| 答复栏 | 🟡 (1) **共享总池计数，但每个 worktree 设软上限告警**（不做硬隔离）：完全独立计数会导致 8 个任务各自不知道全局预算消耗到哪（信息孤岛）；共享总池 + 单任务软上限（如预估 8K token，超 150% 触发告警但不强制中断）更符合总预算约束下动态调配的实际管理需求。(2) **跨 worktree 决策对话计入**（比如本次这种"审核 24 个问题"的主对话，就是跨 worktree 协调成本，理应计入，不能假装免费——这是一人公司里 Ulysses 扮演"管理层"角色消耗的真实成本）。(3) 硬约束优先级：**AI 上下文窗口 > 单次会话成本**。上下文窗口是物理约束（超限直接失败/被压缩丢信息，不可逆）；会话成本是经济约束（可通过预算调整容忍，比如多花 20% token 换正确性），应优先保护不可逆约束。下游动作：RGS-TS-001 §6.2 补充"worktree 共享池 + 软上限告警"具体阈值参数（本答复给出原则，具体数字待 PH-1 首轮实测后校准）。⚠️ 见 RGS-OPEN-QA-001-ACTIONS-v0.3.md §5 修正 #8：RGS-TS-001 v0.6 §6.2 已经是双轨制（人·天/周 + token/周）框架，本动作只是补一段参数，不是重新定义框架。 |

---

### Q-G-04 [P2] WBS done 100% 与 B-CODE log 实质完成的关系

| 字段 | 内容 |
|---|---|
| 关联文档 | WBS-001 v0.6 §8 WBS 状态维护 SOP（per phase-0-5 反馈单 Issue 4/5）+ phase-0-5 反馈单 + 07-no-go-checklist_business v0.3 |
| 现状 | phase-0-5 反馈单 Issue 4 描述："已合并进 main"但"任务实质未完成"是反模式；WBS-001 v0.6 §8.3 明令禁止"合并 ≠ 任务完成"；当前 B-CODE log 重写（11 份）算"实质完成"还是"形式完成"？|
| 疑问 | (1) B-CODE log 重写 11 份算 done 100% 还是仅 partial？<br>(2) "实际跑通"vs"文档通过"的判定边界在哪里？<br>(3) PH-1 期间是否需要新增 B-CODE / C-CODE log 模板（per Issue 5 anti-pattern）？ |
| 期望答复 | B-CODE log 重写完成度 + 边界判定 + 新 log 模板时序 |
| 答复栏 | 🟡 (1) 现有 11 份 B-CODE log 重写**需逐份核实**，不能一刀切判定 done——一刀切正是本疑问要防止的反模式本身。按 WBS-001 §8.3 SOP 判定标准（是否有可运行验证 + 测试通过记录）对每份 log 打标签：已按 SOP 验证=完全 done / 仅文档重写未附验证证据=partial 需补验证。(2) "实际跑通" vs "文档通过"边界：以是否存在**可重复执行的自动化验证**（cargo test / CI pipeline 记录 / 集成测试日志）为唯一硬指标，纯文字"已完成"不算数——与本次审核 phase-0-5 反馈单的方法一致（不轻信自陈，回 git/CI 记录交叉验证）。(3) **需要新增** B-CODE/C-CODE log 模板（per Issue 5 反模式），强制包含"验证证据"字段（commit hash / 测试输出摘要 / CI run 链接）,不允许只填"已完成"了事。下游动作：WBS-001 §8 补新模板 + 对现有 11 份 log 做一次逐份核验（可作独立 L4 任务）。⚠️ 见 RGS-OPEN-QA-001-ACTIONS-v0.3.md §5 修正 #6："11 份"实际构成是 7 份 G-CODE + 4 份 B-CODE = 11 份，逐份核验需按此口径分类，不是清一色 B-CODE。 |

---

## 5. 汇总与优先级

| 优先级 | 数量 | 占比 | 期望回复时间 |
|---|---|---|---|
| **P0** | 7 | 29% | ≤ 1 天 |
| **P1** | 13 | 54% | ≤ 3 天 |
| **P2** | 4 | 17% | ≤ 7 天 |
| **合计** | **24** | 100% | — |

### 5.1 类别分布

| 类别 | 数量 | 关键 P0 |
|---|---|---|
| **A 设计** | 10 | Q-D-01（消息分发）/ Q-D-02（player 主表）|
| **B 制造/编程** | 10 | Q-M-01（Saga 步骤）/ Q-M-02（#[sqlx::test]）/ Q-M-03（OTel trace）/ Q-M-04（probe mTLS）|
| **C 治理/流程** | 4 | Q-G-01（DEC-005/008 兼容）|

### 5.2 关联文档命中数（Top 5）

| 文档 | 命中条数 |
|---|---|
| 5-DOMAIN-DTL-REVIEW-REPORT.md | 7 |
| DTL-019 / DTL-031 / DTL-026 / DTL-018 | 6 |
| RGS-REV-011 v0.1 | 4 |
| RGS-PLAN-001 v1.0 | 3 |
| RGS-TS-001 §6 + DEC-005/008 | 3 |

---

## 6. 流程：审核人答复后动作

| 审核人答复类型 | AI worker 后续动作 |
|---|---|
| **直接决策**（如 Q-D-01 选 A）| 立即写 DTL 升版 / 新建 DTL / 改 SPEC（per RGS-IMPL-001 §2 边界）|
| **指定下游文档**（如 Q-M-01 先 RGS-DEC-Q003）| 先做下游文档，本疑问挂起直到下游落地 |
| **暂不答复**（P2 延后）| 关闭本条，移到 `RGS-OPEN-QA-001-deferred.md`（未来 v0.2 重新激活）|
| **升级为 ADR**（如 Q-G-01）| 新建 RGS-ADR-NNN，按 ADR 模板走 5 状态（Proposed/Accepted/Superseded/Deprecated/Rejected）|
| **升级为新决策**（如 Q-D-05 PFAU 13min 冲突）| 新建 RGS-DEC-NNN v0.1，引用本疑问 ID |

---

## 7. 关联文档

- **基础评审**：RGS-REV-003 v0.3 联合评审 / RGS-REV-004 附件 A 5 域 DTL checklist / RGS-REV-005 附件 B Saga 演练 / RGS-REV-009 工程 55 CR-1/2/3 / RGS-REV-011 v0.1 6 项缺口
- **设计真源**：5-DOMAIN-DTL-REVIEW-REPORT.md / 7 份 DTL（018/015/016/026/019/020/031）/ 5 份 SPEC / 12 份 ADR
- **实施规范**：RGS-IMPL-001 / 002-006 / 100
- **计划/进度**：RGS-PLAN-001 v1.0 / RGS-WBS-001 v0.6 L4 进度表 / RGS-INC-001 v0.2 / RGS-INC-002 v0.1
- **技术选型**：RGS-TS-001 §6 OTel / §6.2 token-OLU / §7 工具链 Bug
- **治理**：DEC-001~008（已知）/ DEC-NOGO-001 / RGS-QA-001 v0.13

---

> **本疑问集不替代 RGS-QA-001 实施前 QA 表的具名签字 / RGS-PLAN-001 v1.0 12 类签字 / 各 DTL/SPEC/ADR 的版本演进。** 审核人答复后由 AI worker 落地到对应文档，本表 v0.1 → v0.2 仅追加"已答复 + 已落地"段，不修改历史疑问。
