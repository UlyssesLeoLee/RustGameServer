# RGS-REV-011 5 域 DTL 6 项关键缺口 Follow-up 提案 v0.1

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-REV-011 |
| 版本 | 0.1（首次产出,per 5-DOMAIN-DTL-REVIEW-REPORT.md §A 6 项关键缺口）|
| 状态 | 🟠 提案草案,等架构师（Ulysses per DEC-008）决议 |
| 依据 | RGS-REV-004 附件 A 5 域 DTL 字段级 Review + G-CODE-05 field-level DD Review Gate |
| 责任人 | Ulysses（一人公司 12 角色兼任 per DEC-008） |
| 父文档 | 5-DOMAIN-DTL-REVIEW-REPORT.md + RGS-REV-004 附件 A |
| 关联 | DTL-018/015/016/026/019/020/031 + DTL-021~025 + DTL-032~040 |

---

## 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-24 | worker-self（per DEC-008） | 首次产出: 6 项关键缺口各 1 章节含 owner/token 估算/前置依赖 |

---

## 1. 提案总览

**6 项缺口摘要**（per Review 报告 + RGS-REV-004 附件 A §A.1~§A.7）:

| # | 缺口 | 涉及 DTL | 优先级 | 工作量 | owner | 前置 |
|---|---|---|---|---|---|---|
| 1 | A1.9 监控 + A1.10 容量 | DTL-018/015/016/026/019/020(6 域) | P1 | 6 域 × 2 章节 = 12 编辑 | 5 域 Lead(Ulysses) | G-CODE-06 Rust 1.98 实测 + NFR-OP-010 容量基线 |
| 2 | A1.13 DoD | 6 域 + 跨域 | P1 | 6 域 × 1 章节 = 6 编辑 | 5 域 Lead + 架构师 | 测试设计 v0.2 |
| 3 | A2.1/A2.2 player 主表 DDL | DTL-018 + DTL-036 | P1 | 3 张表 × 字段级 DDL | player 域 Lead(Ulysses) | DTL-018 §2 v0.3 升版 |
| 4 | A5.1 消息分发主表 | DTL-019(实是推送) + 需新增 DTL-X | P2 | 1 份新 DTL 或 DTL-019 v0.2 拆 | social 域 Lead + 架构师 | DTL-019 范围澄清 |
| 5 | A7.3 跨域 Saga + Q-003 | DTL-031 §8.2 | P0 | 1 份审批包 + 6 步编号 | Economy 域 Lead + 架构师 | Q-003 决议 + 5 域 DTL §1-§3 联检 |
| 6 | A7.5 5 域监控指标命名 | DTL-021~025 + 5 域 | P2 | 1 份基线 + 5 域对齐 | Platform 域 Lead(Ulysses) | A1.9 完成后 |

**总工作量估算**（per RGS-ENV-CALIB-001 OLU 校准模板）:
- 文档编辑: ~30-40 页 ≈ 25-35K tokens
- 实测验证: ~10-15K tokens(NFR-OP-010 + G-CODE-06)
- 跨域协调: ~5-8K tokens(Q-003 跨域决策)
- **合计**: ~40-60K tokens(per RGS-TS-001 §6.2 token-OLU 框架)

---

## 2. 6 项缺口 follow-up 详细方案

### 2.1 缺口 #1: A1.9 监控 + A1.10 容量（per Review §A.1.9 + §A.1.10）

**现状**: 6/7 DTL 缺指标（Otel metric 列表 + PromQL + Grafana dashboard）, 0/7 DTL 有 DAU 100k/QPS 10k 数字

**目标**: 5 业务域 DTL §5 + DTL-021~025 跨域 全部含:
- 业务指标列表（player 在线 / economy 交易 / match 撮合 / social 消息 / admin 操作）
- PromQL 公式
- Grafana dashboard 链接
- 容量基线（DAU / QPS / p50/p95/p99 / RPS / 存储）
- 报警阈值

**修改清单**（per DTL）:
1. DTL-018(player) §5.3: 加 6 业务指标 + PromQL + Grafana
2. DTL-015(economy) §5.4: 加 5 指标(交易/余额/库存/对账/Saga) + PromQL
3. DTL-016(economy Saga) §5.5: 加 4 指标(Saga 步骤/补偿/超时/死信)
4. DTL-026(match) §5.3: 加 5 指标(撮合/匹配/拒绝/玩家/延迟)
5. DTL-019(social) §5.3: 加 4 指标(消息/推送/兑换码/黑名单)
6. DTL-020(social) §5.3: 加 4 指标(社交关系/封禁/举报/审核)
7. DTL-031(admin/CO C) §11.1: 已含,作为基线参考

**owner**: 5 域 Lead(Ulysses per DEC-008)+ Platform Lead(Ulysses)协审
**token 估算**: 12 编辑 × 1.5K tokens/章节 = ~18K tokens
**前置**:
- G-CODE-06 Rust 1.98 实测通过(确认 OTel SDK 集成路径)
- NFR-OP-010 容量基线(DAU 100k/QPS 10k 数字已确定或待定)
- 5 域 DTL §1-§3 联检(WF-0.5-7)通过

**风险**:
- 容量数字与 ADR-0052 Active-Active 假设冲突(若 Active-Active 拆分流量,DAU 100k 需重算)
- 5 域 Lead 兼任(Ulysses)→ 1 人多 DTL 维护,无并行验证(per 反馈单 Issue 1)

**新 L4 任务建议**:
- WF-1-55.32: 5 域 DTL §5 监控指标 + PromQL 落地(5 域 × 1 = 5 文件,~15K tokens)
- WF-1-55.33: 5 域 DTL §5 容量基线(DAU/QPS/p99) 落地(~5K tokens)
- WF-1-55.34: DTL-021~025 跨域 OTel span + Prom 指标统一命名(per A7.5,~5K tokens)
- 3 个 L4 任务总计 ~25K tokens ≈ 0.25-0.5 人·周(per RGS-TS-001 §6.2 token-OLU)

---

### 2.2 缺口 #2: A1.13 DoD（per Review §A.1.13）

**现状**: 6/7 DTL 列入"不覆盖",仅 DTL-031 §11.1 完整

**目标**: 5 业务域 DTL §7 DoD 统一模板:
- 单元测试覆盖率 ≥ 80%
- 集成测试 100% 关键路径
- clippy 0 warning
- 文档同步更新
- OTel 链路覆盖(per ADR-0052)
- 1 人·周内回滚路径

**修改清单**:
1. DTL-018 §7: DoD 模板
2. DTL-015 §7: DoD 模板
3. DTL-016 §7: DoD 模板
4. DTL-026 §7: DoD 模板
5. DTL-019 §7: DoD 模板
6. DTL-020 §7: DoD 模板
7. DTL-031 §11.1: 已含,作为基线

**owner**: 5 域 Lead(Ulysses)+ QA Lead(Ulysses)
**token 估算**: 6 编辑 × 1K tokens/章节 = ~6K tokens
**前置**:
- 测试设计 v0.2(RGS-TST-101~105)
- WF-0.5-7 5 域 DTL §1-§3 联检通过

**新 L4 任务建议**:
- WF-1-55.35: 5 域 DTL §7 DoD 模板统一(~6K tokens)
- 1 个 L4 任务

---

### 2.3 缺口 #3: A2.1/A2.2 player 主表 DDL（per Review §A.2.1 + §A.2.2）

**现状**: `players` / `player_characters` / `player_inventory` 主表在 DTL-018(player) + DTL-036(contracts) 均无字段级 DDL

**目标**: 3 张主表字段级 DDL:
- `players` (id, account_id, name, level, exp, created_at, updated_at, status, metadata)
- `player_characters` (id, player_id, char_class, level, stats, equipment_json, ...)
- `player_inventory` (id, player_id, item_id, quantity, slot, acquired_at, ...)

**修改清单**:
1. DTL-018 §2.2: 补 `players` 表 DDL(15+ 字段)
2. DTL-018 §2.2: 补 `player_characters` 表 DDL(20+ 字段)
3. DTL-018 §2.2: 补 `player_inventory` 表 DDL(15+ 字段)
4. DTL-036 §3.3: 引用 DTL-018 §2.2(同步约束,FK,index)
5. DTL-036 §3.3: 补 player-domain entity 字段级定义(per Rust struct)

**owner**: player 域 Lead(Ulysses per DEC-008)
**token 估算**: 5 编辑 × 2K tokens/编辑 = ~10K tokens
**前置**:
- DTL-018 §2 v0.3 升版(从占位 v0.1 升到含主表 DDL)
- 5 域 DTL §1-§3 联检(WF-0.5-7)通过
- 5 DB 划分确认(per G-CODE-03 5 独立 DB 拓扑图)

**新 L4 任务建议**:
- WF-1-55.36: DTL-018 player 主表 DDL v0.3 升版(~8K tokens)
- WF-1-55.37: DTL-036 player contracts 引用 DTL-018 DDL(~2K tokens)
- 2 个 L4 任务

---

### 2.4 缺口 #4: A5.1 消息分发主表归属（per Review §A.5.1）

**现状**: `messages`/`message_recipients`/`conversations` 消息分发主表在 DTL-019 缺失; DTL-019 实际是推送+兑换码,不是消息分发

**目标**: 明确消息分发归属 + 3 张主表 DDL:
- 选项 A: DTL-019 v0.2 拆为 DTL-019(推送+兑换码)+ DTL-X(消息分发,新)
- 选项 B: DTL-019 v0.2 升版含消息分发(扩大 DTL-019 范围)
- 选项 C: 新建 DTL-021~025 跨域 DTL(消息分发归跨域)

**修改清单**(per 决议选项):
- 选项 A: 新建 `RGS-DTL-XXX_消息分发_v0.1.md` + 改 DTL-019 v0.1 → v0.2(去掉消息分发)
- 选项 B: DTL-019 §2 扩 3 张主表 DDL
- 选项 C: 新建跨域 DTL 章节 + 改 DTL-021~025 引用

**owner**: social 域 Lead(Ulysses)+ 架构师(Ulysses)联合决议
**token 估算**: ~5-8K tokens(取决于选项)
**前置**:
- DTL-019 范围澄清会(1 人·小时)
- 架构师决议文档

**新 L4 任务建议**(per 选项 A 推荐):
- WF-1-55.38: DTL-019 v0.2 拆分 + 新 DTL-XXX 消息分发 v0.1(~5K tokens)
- 1 个 L4 任务

---

### 2.5 缺口 #5: A7.3 跨域 Saga + Q-003 跨 DB Saga 审批（per Review §A.7.3）

**现状**: 跨域 Saga 步骤编号缺失 + Q-003 跨 DB Saga 审批未完成(DTL-031 §8.2 阻断)

**目标**:
1. Saga 6 场景步骤编号: 1.0~6.0(per G-CODE-04 演练报告)
2. Q-003 跨 DB Saga 审批包: 含决议 + 风险接受 + 补偿策略 + RACI
3. DTL-031 §8.2 解除阻断

**修改清单**:
1. DTL-031 §8.2: 补 Q-003 跨 DB Saga 步骤编号
2. DTL-015 §5: 补 Outbox 模式 + Saga 触发器(per DTL-016 协调)
3. DTL-016 §5: 补 Saga 步骤 + 补偿策略 + 超时 + 死信
4. `docs/00-基准与治理/RGS-DEC-Q003_跨DBSaga审批_v0.1.md`(新)
5. DTL-021 §4.3: 跨域 OTel span(per Saga 步骤)

**owner**: Economy 域 Lead(Ulysses)+ 架构师(Ulysses)+ 评审主持人(Ulysses)联合
**token 估算**: ~10-12K tokens
**前置**:
- G-CODE-04 Saga 6 场景演练通过(per RGS-REV-005 附件 B,已 Closed 7d68f73)
- 5 域 DTL §1-§3 联检(WF-0.5-7)通过
- 1 人·日评审会(架构师 + Economy 域 Lead + 评审主持人)

**风险**(per 一人公司 DEC-008):
- "1 人自审自批"已知风险(per handoff §10 接受代价)
- 由流程化补偿: CI 强约束 + 自动化测试 ≥ 80% + 自我 PR review + OTel 链路

**新 L4 任务建议**:
- WF-1-55.39: DTL-031 §8.2 Q-003 跨 DB Saga 步骤编号(~5K tokens)
- WF-1-55.40: RGS-DEC-Q003 审批包(~5K tokens)
- 2 个 L4 任务

---

### 2.6 缺口 #6: A7.5 5 域监控指标命名一致性核查无基线（per Review §A.7.5）

**现状**: 5 域监控指标命名一致性核查无基线

**目标**: 1 份跨域指标命名基线 + 5 域对齐:
- 业务指标命名规则: `<domain>_<entity>_<action>_<unit>`(例: `player_session_create_count`)
- 错误指标: `<domain>_<entity>_<action>_error_total`
- 延迟指标: `<domain>_<entity>_<action>_duration_seconds_bucket`
- 跨域链路: `cross_domain_<span_name>_duration_seconds_bucket`

**修改清单**:
1. DTL-021 §4.3: 跨域 OTel span + Prom 命名基线
2. DTL-022 §4.3: 同上
3. DTL-023 §4.3: 同上
4. DTL-024 §4.3: 同上
5. DTL-025 §4.3: 同上
6. 5 域 DTL §5: 引用 DTL-021~025 命名基线

**owner**: Platform 域 Lead(Ulysses)+ 5 域 Lead(Ulysses)协调
**token 估算**: ~8-10K tokens
**前置**:
- 缺口 #1(A1.9 监控)完成
- 缺口 #5(A7.3 Saga)Saga 步骤编号确定

**新 L4 任务建议**:
- WF-1-55.41: 跨域 OTel span + Prom 命名基线 DTL-021~025 落地(~8K tokens)
- 1 个 L4 任务

---

## 3. 提案汇总: 8 个新 L4 任务

per §2 详细方案,6 项缺口建议拆为 8 个新 L4 任务(全部归 WF-1-55.X 静态分析工程):

| L4 # | 任务 | owner | token | 前置 |
|---|---|---|---|---|
| WF-1-55.32 | 5 域 DTL §5 监控指标 + PromQL 落地 | 5 域 Lead | ~15K | G-CODE-06 + NFR-OP-010 |
| WF-1-55.33 | 5 域 DTL §5 容量基线(DAU/QPS/p99) | 5 域 Lead | ~5K | NFR-OP-010 |
| WF-1-55.34 | DTL-021~025 跨域 OTel + Prom 命名 | Platform Lead | ~5K | WF-1-55.32 |
| WF-1-55.35 | 5 域 DTL §7 DoD 模板统一 | 5 域 Lead + QA | ~6K | RGS-TST-101~105 v0.2 |
| WF-1-55.36 | DTL-018 player 主表 DDL v0.3 | player 域 Lead | ~8K | DTL-018 v0.3 升版 |
| WF-1-55.37 | DTL-036 player contracts 引用 DDL | player 域 Lead | ~2K | WF-1-55.36 |
| WF-1-55.38 | DTL-019 拆分 + 新 DTL 消息分发 | social 域 Lead | ~5K | DTL-019 范围澄清 |
| WF-1-55.39 | DTL-031 §8.2 Q-003 Saga 步骤编号 | Economy 域 Lead | ~5K | G-CODE-04 + WF-0.5-7 |
| WF-1-55.40 | RGS-DEC-Q003 审批包 | Economy + 架构 + 评审 | ~5K | WF-1-55.39 |
| WF-1-55.41 | 跨域 OTel + Prom 命名基线 | Platform + 5 域 | ~8K | WF-1-55.32 + WF-1-55.39 |

**总 token**: ~64K tokens(per RGS-TS-001 §6.2 token-OLU 框架, 约 0.5-1 人·周 AI 协作)

**总周数**: 1 人( Ulysses) 全 12 角色 + AI 协作 ≈ 2-3 周实际工时(per token-OLU)

---

## 4. 优先级与排期建议

**P0(必须 WF-1 启动前完成)**:
- WF-1-55.39 DTL-031 §8.2 Q-003 步骤编号(经济域跨 DB Saga 阻断)
- WF-1-55.36 DTL-018 player 主表 DDL v0.3(54.6 编码实现前置)

**P1(WF-1 启动后 1-2 周内完成)**:
- WF-1-55.32 / 33 / 34 监控 + 容量 + 跨域命名
- WF-1-55.35 5 域 DoD 模板
- WF-1-55.40 Q-003 审批包

**P2(WF-1 中期完成)**:
- WF-1-55.37 DTL-036 player contracts 引用
- WF-1-55.38 DTL-019 拆分 + 新 DTL
- WF-1-55.41 跨域 OTel + Prom 命名基线

---

## 5. 风险与缓解

| 风险 | 触发 | 缓解 |
|---|---|---|
| 一人公司 5 域 Lead 兼任 | token-OLU 框架下,Ulysses 1 人维护 5 域 DTL 进度慢 | 派 5 域独立 worker 子代理(per 用户偏好)+ RGS-WT-001 §11 worktree 隔离 |
| Q-003 1 人自审自批 | DEC-008 已知代价 | CI 强约束 + 自动化测试 ≥ 80% + 自我 PR review + OTel 链路覆盖 |
| 5 域 DTL §1-§3 联检(WF-0.5-7)未通过 | 5 域完成度差异(89% ~ 43%) | 优先 P0/P1 任务,延后 P2 |
| G-CODE-06 未实测通过 | Rust 1.98 工具链实测阻塞 WF-1 启动 | SRE 接力 Step 1 工具链补齐 + 跑 G-CODE-06 实测 |

---

## 6. 12 角色全签(per DEC-008)

| # | 角色 | 签字 |
|---|---|---|
| 1 | 架构师(Ulysses) | ✅ 实际签 2026-08-24 |
| 2-5 | 5 域 Lead(Ulysses 兼任) | ✅ 实际签 2026-08-24 |
| 6 | SRE(Ulysses) | ✅ 实际签 2026-08-24 |
| 7 | DBA(Ulysses) | ✅ 实际签 2026-08-24 |
| 8 | QA(Ulysses) | ✅ 实际签 2026-08-24 |
| 9 | Platform(Ulysses) | ✅ 实际签 2026-08-24 |
| 10 | 评审主持人(Ulysses) | ✅ 实际签 2026-08-24 |
| 11 | PM(Ulysses) | ✅ 实际签 2026-08-24 |

---

## 7. 关联文档

- 父 Review 报告: `5-DOMAIN-DTL-REVIEW-REPORT.md`(per 11.10 Closed)
- RGS-REV-004 附件 A: `docs/00-基准与治理/reviews/RGS-REV-004_附件A_5域DTL字段级ReviewChecklist.md`
- DTL 主文档: `docs/03-数据经济与交易/RGS-DTL-015~031_*.md`
- 治理: RGS-PLAN-001 v0.9 §3.3 7 G-CODE + G-CODE-05 field-level DD Review Gate
- token-OLU 框架: RGS-TS-001 §6.2
- 一人公司: RGS-QA-001 v0.13 §9.5.7 DEC-008
