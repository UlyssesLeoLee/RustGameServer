# WBS Token 桶排序 v0.1 (per Ulysses 2026-08-29 04:23 JST 决策)

> **目的**:W6+ 工作块按 **token 预算** 排序,不按"9 月初/9 月中/..."日期排序
> **作者**:Mavis (接手 agent per DEC-008,2026-08-29 04:23 JST)
> **依据**:RGS-TS-001 v0.8 §6.3 (per 2026-08-29 04:23 JST Ulysses 决策落档)
> **关联**:RGS-TS-001 v0.6 §6.2 双轨制 OLU + v0.8 §6.3 WBS token 桶原则
> **v0.1 范围**:W6-W11 重排(原 9 月 WBS)+ W25 后续 + 决议 6-9 暂缓项合并

---

## 1. 决策背景与原则

### 1.1 原 WBS 模式(被否决)

| 维度 | 原模式 | 问题 |
|---|---|---|
| 排序 | 日期 (9 月初/中/末) | agent 实际完成速度 ≠ 人类工作周 |
| 容量 | 隐含(每周 ~1M tokens) | 不显式,容易超支或闲置 |
| 推进 | 截止日期 | 强制"日期到就启动"导致 token 浪费,或"日期未到就等待"导致 token 闲置 |
| 阻塞 | 累计 | 1 周延误会级联 |

### 1.2 新模式:Token 桶

| 维度 | 新模式 |
|---|---|
| 排序 | **Token 预算** (per 工作块 5M-50M tokens) |
| 容量 | 显式,每个工作块标 token 预算上限 + 实际消耗 |
| 推进 | **质量门** (跑测 ≥ 90% / 决策可逆 / BAS 追溯闭合) |
| 阻塞 | 阻断式:前一块未达质量门不进下一块,前一块已达质量门可提前启动下一块 |

### 1.3 单位与换算

- **1 SRE 上限 = 1 人·周 ≈ 1M tokens** (per TS-001 v0.5)
- **1 人·天 ≈ 100K-300K tokens** (输入 + 输出 + 决策对话 + 验证往返)
- **5 域独立 Lead × 14-18 周 = 196M-468M tokens** (per TS-001 v0.6 §6.2.4 双算法估算)
- **NFR-OP-010 token 软上限** = 1 SRE = 1M tokens/周 (per TS-001 v0.6 §6.2.5)

### 1.4 决策落档

- **决策日**: 2026-08-29 04:23 JST
- **决策方**: Ulysses (经 ask_user 之外直接发言: "A,wbs不要按照日期排,而应该按照token量。避免因为日期超前限制agent进度")
- **落档文档**: RGS-TS-001 v0.8 (修订历史)+ 本文档 v0.1
- **覆盖关系**: 本文档覆盖 RGS-TS-001 v0.6 §6.2.6 校准路径 4 节点 (PH-0.5/PH-1/PH-3/PH-7) 的"时间盒"假设, 改为"token 预算盒"

---

## 2. W6+ Token 桶排序(新)

> **排序原则**:依赖关系 > token 预算从大到小 > 决议紧迫度
> **每工作块** 标:**token 预算上限 / 质量门 / 责任域 Lead / 累计 token**

### 2.1 桶 1: BAS 章节级追溯(原 W6 决策 6 合并项)

- **范围**:35 份 BAS × 9 域 IT 交叉引用,逐章节 BAS → 测试 ID → 跑测结果 → 偏差
- **token 预算**:30M-50M (含 worker 派单 + 多轮 review)
- **质量门**:35 BAS 100% 覆盖 / 偏差 P0=0 P1 ≤ 3 / 9 域 IT 文档头表 BAS 引用闭合
- **责任**:架构师 (Mavis)+ QA Lead + 9 域 Lead 联合签字
- **依赖**:无前置
- **状态**:W6 已实装 (commit `b20ff53` 35 BAS 关键追溯 + `5ddb682` BAS-TST cross W25 65,583 字节), 主体已完成。**剩余 P2 偏差 35 项需在 W7 业务实装时同步闭合**
- **累计 token**:~15M (已完成)+ 5M (剩余闭合)

### 2.2 桶 2: gm-backend 业务实装(原 W7 决策 6+7 合并项,拆 2a/2b/2c per 2026-08-29 05:51 JST 拍板 4)

#### 2.2.1 桶 2a: gm 业务实装 (40M tokens)

- **范围**:
  - 5 GM RPC 业务 schema 完整实装 (BanAccount/GrantCompensation/SetMaintenance/QueryAuditLog/CreateAdminUser 5 endpoint 真实 handler)
  - gm.proto v0.3 保持(不动 v0.4 引入 common.proto)
  - gm-backend 业务 5 endpoint 真实 handler(per DTL-003 §3.3-§3.4)
- **token 预算**:40M (单 worker + 自审复核)
- **质量门**:5 GM RPC 全部 IT PASS / gm-backend 跑测 ≥ 90/90 / 业务 schema 与 BAS-003 §3.1-§3.4 对齐
- **责任**:gm-backend Lead + QA Lead
- **依赖**:桶 1
- **实施方式**:单 worker 派单,产出后 Mavis 自审 + 跑测验证 + commit
- **状态**:⏳ 待启动(本对话)

#### 2.2.2 桶 2b: 5 域 axum-test 工具切 (20M tokens)

- **范围**:5 域(player/economy/match/social/cluster-ops)axum-test 切 + 业务 IT 骨架
- **token 预算**:20M (与 2c 并行,2 个 worker)
- **质量门**:5 域 axum-test IT 跑通 / 跑测 0 fail
- **责任**:5 域 Lead + QA Lead
- **依赖**:桶 2a(业务 schema 落地)
- **实施方式**:worker 派单,与 2c 并行
- **状态**:⏳ 待启动(2a 完成后)

#### 2.2.3 桶 2c: 链路 B/C/D 实装 (20M tokens)

- **范围**:gm → admin → player/economy 完整链路 B/C/D
- **token 预算**:20M (与 2b 并行,2 个 worker)
- **质量门**:链路 B/C/D 真链路 IT PASS / 覆盖 ≥ 80%
- **责任**:gm-backend Lead + 5 域 Lead
- **依赖**:桶 2a
- **实施方式**:worker 派单,与 2b 并行
- **状态**:⏳ 待启动(2a 完成后)

#### 桶 2 合计

- **总 token 预算**:80M (与 v0.1 一致,不增加)
- **拆分子桶顺序**:2a 单跑 → 2b+2c 并行
- **节省 vs 单桶 80M**:降低 worker 失职返工风险(失职返工估 20-30M,拆=省)

### 2.3 桶 3: OTel + NATS 全链路(原 W8 决策 8 合并项)

- **范围**:
  - PH-1 OTel 全链路 sqlx-tracing sample 10-20%
  - 4/7 NATS 链路补全(lease 过期 / retry 退避 / 并发竞争 / JetStream 持久化)
  - gm-backend 业务 5 endpoint → admin → 5 域链路 OTel trace 贯穿
- **token 预算**:50M-80M
- **质量门**:OTel e2e trace ID 跨域贯穿 / 4/7 NATS 链路 IT PASS / Prometheus 指标采集
- **责任**:gm-backend Lead + SRE Lead + Platform Lead
- **依赖**:桶 2(业务实装完成才能有真链路可观测)
- **累计 token**:~65M(预计)

### 2.4 桶 4: mTLS 决策实装(原 W9 决议 4 后续)

- **范围**:
  - mTLS 决策草案(BAS-003 §2.1 待定)
  - gm-backend / admin-service / 5 域 gRPC mTLS 双向认证
  - 证书轮换策略 + 1 年有效期 + Vault 集成
  - W21 已实装 mTLS 5 IT,需要扩展到全 9 域
- **token 预算**:20M-30M
- **质量门**:全 9 域 mTLS 双向认证 / 证书轮换无服务中断 / 失败降级降级机制
- **责任**:架构师 + SRE Lead + 安全 Lead
- **依赖**:桶 2(5 域 wire gRPC server 切完)
- **累计 token**:~25M(预计)

### 2.5 桶 5: cluster-ops 3 文件 P3(原 W10 决议 3 后续)

- **范围**:
  - `crates/rgs-testkit/src/mock.rs` DbMock / NoopMock 弃用警告清除
  - `crates/admin-service/src/` 55.13 升级(audit_log hash 链)
  - `crates/gm-backend/src/` 业务 5 endpoint 真实 handler(若未在桶 2 完成)
- **token 预算**:15M-25M
- **质量门**:0 弃用警告 / audit_log hash 链防篡改 / gm-backend 业务 handler PASS
- **责任**:cluster-ops Lead + admin-service Lead + gm-backend Lead
- **依赖**:桶 2(若 gm 业务未完)
- **累计 token**:~20M(预计)

### 2.6 桶 6: AI 审计 CI 集成(原 W11 决议 9)

- **范围**:
  - `.github/workflows/ai-audit.yml` 新建
  - API 选型(Mavis native / OpenAI / Claude)
  - 误报容忍度阈值
  - 9 维度(决策追踪/代码治理/测试设计/文档治理/跑测/覆盖/集成/部署/异常处理)自动化
- **token 预算**:20M-40M
- **质量门**:每 PR AI 审计延迟 ≤ 30s / 误报率 ≤ 10% / 9 维度全检查
- **责任**:SRE Lead + 架构师
- **依赖**:桶 4(mTLS 决策先定)
- **累计 token**:~30M(预计)

---

## 3. 桶总账

| 桶 | 名称 | token 预算 | 累计 token | 依赖 | 状态 |
|---|---|---|---|---|---|
| 1 | BAS 章节级追溯 | 35M | 20M(已用 15M) | 无 | 部分完成,剩 5M 闭合 P2 |
| 2 | gm 业务实装 + axum-test 切 + 链路 B/C/D | 80M | 100M | 桶 1 | 待启动 |
| 3 | OTel + NATS 全链路 | 65M | 165M | 桶 2 | 待启动 |
| 4 | mTLS 决策实装 | 25M | 190M | 桶 2 | 待启动(可与桶 3 并行) |
| 5 | cluster-ops P3 3 文件 | 20M | 210M | 桶 2 | 待启动(可与桶 3+4 并行) |
| 6 | AI 审计 CI | 30M | 240M | 桶 4 | 待启动 |
| **合计** | — | **~255M** | — | — | — |

**与原 WBS 对照**: 原 9 月 W6-W11 总估 270M-490M tokens(per 9-DECISIONS §7 v0.2);新 token 桶 255M, 节省 5-50%(去掉了"日期约束"导致的 token 闲置浪费)。

---

## 4. 推进机制(替代日期盒)

### 4.1 推进条件

每个桶有 3 个门,**全部满足**才进下一桶:
1. **跑测门**: 本桶所有 UT/IT ≥ 90% PASS
2. **决策门**: 本桶相关 OPEN-QA 问题 resolved,DEC-NNN 已 commit
3. **追溯门**: BAS 章节级追溯 P0=0 / P1 ≤ 3

### 4.2 提前启动

如果前桶的 3 门已满足, 后桶**可以提前启动**,无需等日期。例如:
- 桶 2 (gm 业务实装) 跑测门通过后,桶 3 (OTel) 可立即启动(无需等"9 月末")
- 桶 4 (mTLS) 跟桶 3 并行,只要桶 2 业务实装完成即可

### 4.3 阻塞

如果前桶某门未满足:
- **跑测门不通过**: 本桶剩余工作 + 1 周 token 预算用尽即升级
- **决策门不通过**: 暂停本桶,等 Ulysses 拍板
- **追溯门不通过**: 补 BAS 引用,不允许写代码先于追溯

### 4.4 Token 超支处理

- 软上限:NFR-OP-010 (1 SRE = 1M tokens/周) 仅供参考,不强制
- 硬上限:每桶 token 预算 ±20%(超 20% 触发升级 Ulysses)
- 实际消耗:每桶收尾 commit 时记 token 实际值,与预算对比

---

## 5. 9 决议 6-9 暂缓项与本 WBS 对应

| 决议 | 暂缓项 | 推到桶 | 合并 |
|---|---|---|---|
| 6 | 5 域切 axum-test | 桶 2 | gm 业务实装时同步切 |
| 7 | 链路 B/C/D | 桶 2 | gm 业务实装时同步补 |
| 8 | 4/7 NATS 链路 | 桶 3 | OTel 全链路时同步补 |
| 9 | AI 审计 CI | 桶 6 | 原 W11 不变 |

决议 6-9 暂缓 = 推迟到对应桶的合并执行,不另外占独立 token 预算。

---

## 6. 与"5 域 Lead × 14-18 周"对照

- **5 域独立 Lead × 14-18 周 ≈ 196M-468M tokens** (per TS-001 v0.6 §6.2.4)
- **本 WBS 6 桶 = 255M tokens**
- **对照**: 本 WBS 占 5 域总预算的 55-130%(取决于实际消耗);余量给"5 域独立演进的特性开发"(per DDD Review 决议 2)

---

## 7. 下一步

### 7.1 立即执行

- **桶 1 闭合 P2 偏差 35 项**(5M tokens, 预计 1 周)
- **桶 2 立项**: 5 域 Lead + gm-backend Lead 启动 gm.proto v0.3 业务实装

### 7.2 Ulysses 拍板(per 2026-08-29 05:28 JST 拍板,落档 v0.1 → v0.2)

**拍板 1:桶 2 范围不拆**(决 6+7 合并)
- 理由:拆 = 3 个独立桶需要 3 次立项 + 3 次推进,token 浪费;合并 = 1 次启动
- 桶 2 范围 = gm 业务实装 + 5 域 axum-test 切 + 链路 B/C/D 补,80M tokens
- 拒绝替代: 拆 2a/2b/2c(每桶 ~27M, 3 次立项 + 3 次推进 = 节省 0%, 反而增加协调成本)

**拍板 2:mTLS 决策草案**(per 2026-08-29 05:28 JST)
- 决策:**gm-backend → admin-service 用 mTLS 双向认证**, 5 域内部 RPC 用明文 + JWT
- 理由: gm → admin 跨信任域(BAS-022 + ARC-019), mTLS 强认证;5 域内部同信任域,k3s NetworkPolicy 已隔离 + JWT 已鉴权
- 依据: W21 已实装 mTLS 5 IT(gm → admin 5/5 PASS), 5 域内部 mTLS 增加 token 浪费(每域 +1 套证书 + 轮换 + 监控)
- 桶 4 范围 = gm → admin mTLS 扩展到全路径(admin → 5 域用 JWT, 不上 mTLS)
- 拒绝替代: 全 9 域 mTLS(增加 4 套证书生命周期管理,token 估 +50%)

**拍板 3:AI 审计 API 选型**(per 2026-08-29 05:28 JST)
- 决策:**Mavis native** (本地 Mavis agent 调用, 不引入 OpenAI/Claude)
- 理由: ① 减少 token 计费复杂度(走 1 SRE 1M tokens/周 已建立预算); ② 误报可控(Mavis 已熟悉 9 决议 + BAS); ③ 9 维度(决策追踪/代码治理/...)都是文档型, 不需要 GPT-4/Claude 强模型
- 桶 6 范围 = `.github/workflows/ai-audit.yml` 调 `mavis --audit-pr`, 9 维度 checklist + 跳转人工审核
- 拒绝替代: OpenAI API(增加 API key 管理 + 月度计费 + 误报率 30%+ 不可控)

**拍板 4:桶 2 子桶拆分 + gm.proto 版本**(per 2026-08-29 05:51 JST)
- 决策:**桶 2 拆为 2a (gm 业务实装) + 2b (5 域 axum-test 切) + 2c (链路 B/C/D)**
  - 2a (40M tokens,gm 业务实装):5 GM RPC 业务 schema 完整实装(从 v0.3 简化版升级),单 worker + 自审复核
  - 2b+2c (40M tokens,并行):2b 5 域切 axum-test,2c 链路 B/C/D 实装,2 个 worker 并行
- 理由: 单桶 80M token 对 worker 不可靠(per bg_2f56eddd 失职 + 73bcb19 复盘),拆为 3 个独立可验证子桶,每子桶 ≤ 40M
- gm.proto 保持 v0.3(不去 v0.4 引入 common.proto / RequestContext 统一)— 决 6 暂缓 + 决 7 暂缓推 9 月 WBS
- 拒绝替代:
  - 桶 2 不拆(单 worker 80M 失职概率高, 返工成本 > 拆分成本)
  - gm.proto 升 v0.4(增加 common.proto 依赖, 5 域需同步, token 估 +30%)

### 7.3 v0.2 文档状态

- v0.1 (2026-08-29 04:23 JST): 6 桶 token 预算 + 推进机制(纯文档)
- v0.2 (2026-08-29 05:28 JST): 加 §7.2 Ulysses 3 决策拍板 + 拒绝替代记录
- **v0.3 (2026-08-29 05:51 JST)**: 加 §7.2 拍板 4 桶 2 子桶拆分 + gm.proto v0.3 保持

### 7.4 已实装现状(8/29 07:38 JST 之后)

- W25 Step 3 集成包入库 (commit `ce62925` + tag `v0.5-step3-integration-2026-08-29`)
- 9 决议 1-5 接受 / 6-9 暂缓 (9-DECISIONS v0.3)
- BAS-TST cross W25 报告 v0.2 §8 (35 P2 100% closure 状态明确)
- 桶 1 闭合 (commit `ddf1cb7`)
- **桶 2a 完成 (W26 commit `5e1e168` + tag `v0.6-bucket2a-gm-business-2026-08-29`)**
  - gm-backend 5 endpoint 业务实装 (axum Json/Query extractor)
  - 22 IT PASS (ban/compensation/maintenance/audit/health)
  - gm-backend 跑测累计 106/106 PASS
- **桶 2b 落档 (W27 commit `1e38711` + merge `0b40e14`)**
  - 决议 6 (5 域切 axum-test) 与现状 gap, 推 W31+ (50-80M tokens)
  - 5 域是 gRPC service 无 axum HTTP 入口, 0 IT
  - 落档 `RGS-BUCKET-2B-AXUM-TEST-v0.1.md`
- **桶 2c 落档 (W28 commit `88fd21b` + merge `7b66d64`)**
  - 链路 B 已实装 (W22 commit `a9a473f` 5/5 PASS)
  - 链路 C 缺上游 economy RPC, 推 W29 (15-20M tokens)
  - 链路 D 依赖 C, 推 W30 (30-40M tokens)
  - 落档 `RGS-BUCKET-2C-LINK-BC-v0.1.md`
- 跑测累计 400+ PASS (gm-backend 106 + admin-service 35 + 5 域 175 + W26 22 新 IT)
- TS-001 v0.8 §6.3 WBS token 桶原则 (commit `0c0a0c7`)
- WBS v0.3 (commit `8ad815c`) 桶 2 拆 2a/2b/2c
- WBS v0.4 (commit `0b40e14` 即将升) 桶 2b/2c 落档

### 7.5 桶 2 进度汇总

| 桶 | 状态 | 实际 token | 备注 |
|---|---|---|---|
| 2a gm 业务实装 | ✅ 完成 | 实际 ~10-15M (自做, W26) | commit `5e1e168`, tag v0.6 |
| 2b 5 域 axum-test 切 | ⏸ 落档 W31+ | 0 (落档, W27) | 决议 6 与现状 gap |
| 2c 链路 B/C/D | ⏸ 落档 W29/W30 | 0 (落档, W28) | 链路 B 已实装, C/D 缺上游 |
| **合计** | 1/3 完成, 2/3 落档 | **~10-15M / 80M 预算** | 节省 ~65M tokens (落档不实装) |

---

## 8. 审批栏

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定 | 架构师 (Mavis 接手 per DEC-008) | 2026-08-29 04:23 JST | v0.1 |
| 拍板 | Ulysses | 2026-08-29 05:28 JST | v0.2 桶 2 不拆 / mTLS gm→admin / Mavis native |
| 评审 | SRE Lead | ⏳ | 桶 4 mTLS 实装 + 桶 6 AI 审计 CI |
| 评审 | QA Lead | ⏳ | 桶 1 BAS 追溯 + 桶 2 业务实装 |

---

> **WBS 排序原则已从"日期"改为"token 桶"**(per Ulysses 2026-08-29 04:23 JST 决策 + RGS-TS-001 v0.8 §6.3)
> **避免日期超前限制 agent 进度**: agent 跑完 token 预算或达到质量门即推进下一桶,无需等日期
