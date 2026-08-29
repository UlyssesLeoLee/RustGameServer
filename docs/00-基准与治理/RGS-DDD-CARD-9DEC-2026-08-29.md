# 卡牌游戏 DDD Review 9 DEC 拍板表 (per 2026-08-29 15:55 JST)

> **目的**:对 RGS-DTL-038 v0.1 §9.2 列出的 9 个 DEC 提供拍板推荐 + 拍板位 + 影响,给 DDD Review 一次拍板
> **作者**:Mavis (接手 agent per DEC-008)
> **上游**: RGS-DTL-038 §9.2 + RGS-REQ-038 + RGS-BAS-038 (待写)
> **拍板方式**:每个 DEC 1 行推荐 + 1 拍板位 (✅ 推荐 / ⚠ 调整 / ❌ 否决)
> **状态**:草案,待 DDD Review

---

## 拍板位图例

- **✅ = 直接拍板推荐项**
- **⚠ = 接受但需调整** (请在备注列写明)
- **❌ = 否决推荐,选其他** (请在备注列写明新选)

---

## DEC-038-01 卡组归属

**上下文**: RGS-REQ-038 §FR-002 卡组管理 RPC
**候选**:
- A. player-service v2 内置(已选,推荐)
- B. 新 card-service v1 内
- C. 独立 deck-service

**拍板位**: ☐ ✅  ☐ ⚠  ☐ ❌

**影响**:
- ✅ 选 → 桶 11 已实装, 0 返工
- ❌ 否 → 桶 11 需重做 (15-18M 重投入)

**备注**: (待 DDD Review 填)

---

## DEC-038-02 leaderboard 域

**上下文**: RGS-REQ-038 §FR-007 排行榜 4 类
**候选**:
- A. 新 leaderboard-service(已选,推荐)
- B. match-service 子模块
- C. 复用 shared-platform Redis 缓存

**拍板位**: ☐ ✅  ☐ ⚠  ☐ ❌

**影响**:
- ✅ 选 → 桶 12 已实装 27 测试, 0 返工
- ❌ 否 → 桶 12 需重做 (10-12M 重投入)

**备注**: (待 DDD Review 填)

---

## DEC-038-03 replay 存储

**上下文**: RGS-REQ-038 §FR-008 战斗回放
**候选**:
- A. cluster-ops 对象存储(推荐)
- B. 新 replay-service (PostgreSQL + S3)
- C. 外部 S3-兼容 (MinIO)

**拍板位**: ☐ ✅  ☐ ⚠  ☐ ❌

**影响**:
- ✅ 选 → 桶 13 推 W34+ 1 worker 1 桶, 估 15M
- B 选 → 桶 13 变 2 桶, 估 25-30M (+10M)
- C 选 → 引入 MinIO 外部依赖, 需 8 域 RACI 拍板

**备注**: (待 DDD Review 填)

---

## DEC-038-04 trade 域归属

**上下文**: RGS-REQ-038 §FR-009 卡牌交易
**候选**:
- A. economy-service v2 内置(推荐)
- B. 新 trade-service
- C. 复用现有 inbox 协议

**拍板位**: ☐ ✅  ☐ ⚠  ☐ ❌

**影响**:
- ✅ 选 → 桶 14 估 25M, 在 economy-service v2 范围内
- B 选 → 桶 14 变 2 桶 (trade 新域 + gm), 估 35-40M (+10-15M)
- C 选 → 公开拍卖缺独立撮合, 业务不完整

**备注**: (待 DDD Review 填)

---

## DEC-038-05 i18n 模式

**上下文**: RGS-REQ-038 §BR-009 多语言
**候选**:
- A. Redis 缓存 + DB 持久化 + 独立 i18n-service(推荐)
- B. build-time 嵌入
- C. 静态文件 (i18n/*.json)

**拍板位**: ☐ ✅  ☐ ⚠  ☐ ❌

**影响**:
- ✅ 选 → 卡牌游戏 i18n 实装, 估 +5M, 1 个新域
- B 选 → 改文案需重新发版, 不适合赛季活动高频
- C 选 → 0 新域但不支持热更新, 不适合卡牌游戏运营

**备注**: (待 DDD Review 填)

---

## DEC-038-06 抽卡概率公开

**上下文**: RGS-REQ-038 §SR-001 抽卡概率
**候选**:
- A. 强制公开 + drop_table_snapshot(推荐)
- B. 可选 (per 监管)
- C. 关闭

**拍板位**: ☐ ✅  ☐ ⚠  ☐ ❌

**影响**:
- ✅ 选 → 符合中国 / 日本法律, GM 工具可审计, 桶 10 已实现
- B 选 → per 地区配置, 复杂
- C 选 → 合规风险 (中国 / 日本)

**备注**: (待 DDD Review 填)

---

## DEC-038-07 gm.proto v0.4 时机

**上下文**: RGS-REQ-038 §FR-010 GM 工具
**候选**:
- A. 卡牌 8 桶 完成后(桶 14 后,推荐)
- B. 立即 (本周)
- C. 8 桶中段 (桶 10 后)

**拍板位**: ☐ ✅  ☐ ⚠  ☐ ❌

**影响**:
- ✅ 选 → gm.proto v0.4 等卡牌 8 桶后期再升版, 避免反复改, 桶 14 含升版
- B 选 → 解锁 gm 卡牌能力, 不阻塞, 但 gm.proto 改动频繁
- C 选 → 中间点平衡

**备注**: (待 DDD Review 填)

---

## DEC-038-08 8 桶 WBS 排序

**上下文**: RGS-DTL-038 §8 8 桶 WBS
**候选**:
- A. 按 token 桶(桶 7→8→9→10→11→12→13→14,推荐)
- B. 按 5 域 RACI (8 域 Lead 各自领桶)
- C. 按业务关键路径 (catalog → deck → match → trade)

**拍板位**: ☐ ✅  ☐ ⚠  ☐ ❌

**影响**:
- ✅ 选 → 与现有 6 桶排序一致, 可视化好, 8 桶执行中
- B 选 → 责任清晰, 跨域协调多
- C 选 → 业务价值递进, 估时不均

**备注**: (待 DDD Review 填)

---

## DEC-038-09 总 token 追加

**上下文**: RGS-DTL-038 §8.1 8 桶 WBS 估 129M,余额 31M
**候选**:
- A. 追加 98M tokens(推荐)
- B. 拆 2 阶段(Phase 1 = 桶 7-10 60M, Phase 2 = 桶 11-14 69M)
- C. 砍 1 桶(选 1 非关键, e.g. leaderboard,总估 121M,追加 90M)

**拍板位**: ☐ ✅  ☐ ⚠  ☐ ❌

**影响**:
- ✅ 选 → 8 桶全过, 沿用 6 桶节省 88% 经验, 实际估 80-100M
- B 选 → 阶段性, 适合多 token 压力场景
- C 选 → 砍 1 桶节省 8M, 但 leaderboard 是 BR-004 业务要求

**当前追加实际**: 卡牌 6 桶已用 ~32M (3 桶 7/11/12 + 3 桶落档), 剩 5 桶 (8/9/10/13/14) 估 48-76M, **追加 17-45M** 必做(per v0.15 落档文档)

**备注**: (待 DDD Review 填)

---

## DDD Review 拍板汇总

| DEC | 推荐 | 拍板位 |
|---|---|---|
| 01 卡组归属 | A player-service v2 | ☐ |
| 02 leaderboard 域 | A 新 leaderboard-service | ☐ |
| 03 replay 存储 | A cluster-ops 对象存储 | ☐ |
| 04 trade 域归属 | A economy-service v2 | ☐ |
| 05 i18n 模式 | A Redis+DB+i18n-service | ☐ |
| 06 抽卡概率 | A 强制公开 | ☐ |
| 07 gm.proto v0.4 时机 | A 桶 14 后 | ☐ |
| 08 8 桶 WBS 排序 | A token 桶 | ☐ |
| 09 总 token 追加 | A 追加 98M | ☐ |

---

## 拍板影响

- 9 ✅ 全过 → 8 桶继续推 (3 桶 7/11/12 已完成, 5 桶 8/9/10/13/14 推 W34+ 单 worker 单桶)
- 任 1 ❌ → 该桶返工, 估 10-40M 额外 token, 推 W36+
- 8 + 1 调整 → 调整桶按新设计实装, +5-15M 额外 token

---

## 拍板者

| 角色 | 姓名 | 签字 | 日期 | 备注 |
|---|---|---|---|---|
| 制定 | 架构师 (Mavis 接手 agent per DEC-008) | ✓ | 2026-08-29 | 草案 |
| 审批 (技术) | — | ⏳ | — | 待 DDD Review |
| 审批 (业务) | — | ⏳ | — | 待 Ulysses |
| **最终决策** | **Ulysses** | ⏳ | — | 待拍板 9 DEC |

---

## 关联文档

- RGS-REQ-038 v0.1 (本表上游)
- RGS-DTL-038 v0.1 §9.2 (9 DEC 候选详细)
- RGS-BAS-038 (待写, 9 DEC 实装指南)
- RGS-BUCKET-8-9-10-WORKER-FAILED-2026-08-29.md (3 worker 失职落档)
- RGS-PLAN-WBS-token-bucket-v0.5 (6 桶 WBS 历史, v0.6 升档待拍板)
