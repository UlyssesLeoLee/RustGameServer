# RGS-TST-08-06 — axum-test vs wiremock 工具决策草案

> **目的**:7 域(5 域 + cluster-ops + admin)用 `wiremock 0.6` HTTP mock server,8 域 gm-backend 用 `axum-test 16` in-process router,工具栈不统一
> **关联**:TBD-08-06 v0.2 处置 + RGS-UT-08 工具栈治理
> **作者**:Mavis(接手 agent per DEC-008,2026-08-28 ut 实施 v0.2)
> **状态**:🟡 OPEN — 待 Ulysses 决策

---

## 0. 现状调研

| 域 | 测试工具 | 版本 | 用法 |
|---|---|---|---|
| player-service | wiremock | 0.6 | mock 5 域 admin RPC |
| economy-service | wiremock | 0.6 | mock outbox NATS |
| match-service | wiremock | 0.6 | mock 撮合 HTTP |
| social-service | wiremock | 0.6 | mock 好友/消息 HTTP |
| admin-service | wiremock | 0.6 | mock GM endpoint |
| cluster-ops | wiremock | 0.6 | mock 跨域 HTTP |
| **gm-backend (8 域)** | **axum-test** | **16** | **in-process axum Router,无 HTTP** |
| **rgs-testkit** | wiremock | 1 | **gRPC mock 实际走 mockito (HTTP server)** |

**8 域用 axum-test 的原因**:gm-backend 内部直接是 axum Router,axum-test 16 走 in-process tower::Service::call,无需起真 HTTP server,**更快更稳定**。

**7 域用 wiremock 的原因**:5 域 + cluster-ops + admin 测试需要 mock 上下游 HTTP admin/RPC,wiremock 起真 HTTP server 接 axum client。

## 1. 决策方案对比

| 方案 | 工作量 | 风险 | 收益 | 推荐? |
|---|---|---|---|---|
| **方案 A: 统一 axum-test (推荐 v0.3 渐进)** | 5 域 + cluster-ops + admin 7 域 wiremock 测试代码改写 ~50-100 文件,~ 1 周 | 5 域 outbox 异步 + 多端口场景 wiremock 表现更好,改 axum-test 可能漏异步 case | in-process 跑测快 5-10x,无端口冲突 | ⚠️ 中期 |
| **方案 B: 统一 wiremock** | gm-backend 12 测试改 wiremock,~ 1 天 | axum-test 的 in-process 优势失去,gm-backend 跑测变慢 | 7 域 wiremock 代码可复用 mock server | ❌ 倒退 |
| **方案 C: 保留现状(per TBD-08-06 决策)** | 0 | 工具栈分裂,接手 agent 需学 2 套 | gm-backend 8 域是后加,历史决策可理解 | ✅ **短期推荐** |
| **方案 D: 双工具并存(per 当前实装状态)** | 0 | 0 | 短期可接受,长期需统一 | ✅ 即采用 |

## 2. 推荐路径

### 阶段 1(v0.2 当前):方案 D(双工具并存)
- gm-backend 8 域用 axum-test 16(已实装,优势明显)
- 7 域用 wiremock 0.6(历史稳定)
- **现状**:已完成,无需变更
- 接受理由:8 域是新增(2026-08-27),gm-backend 自带 axum Router,axum-test 是 in-process 最佳实践

### 阶段 2(v0.3 中期):方案 A 试点
- 选 1 个 5 域(推荐 player-service)做 axum-test 试点
- 对比 wiremock 跑测时间 + 稳定性
- 若收益明显,扩展到其余 4 域 + cluster-ops + admin
- 关键迁移难点:5 域 outbox 异步 + 跨域 RPC 链(可能保留 wiremock 给特定场景)

### 阶段 3(v1.0 长期):统一 axum-test
- 全部 8 域用 axum-test
- wiremock 仅用于第三方 HTTP(S3 / 外部 API)mock

## 3. 决策项

| 决策点 | 选项 | 推荐 |
|---|---|---|
| 立即决策 | 方案 A / 方案 B / 方案 C / 方案 D | **方案 D(双工具并存)** |
| v0.3 试点 | 哪个 5 域先迁移 | **player-service**(最小依赖)|
| 评估指标 | 跑测时间 / CI 稳定性 / 接手 agent 学习成本 | 三者综合 |

## 4. 待 Ulysses 决策

- [ ] 是否同意方案 D(双工具并存)v0.2 接受现状
- [ ] v0.3 试点从哪个 5 域开始
- [ ] 评估指标是否需要量化阈值(跑测时间 < X 秒 / 失败率 < Y%)

---

**作者**:Mavis(接手 agent per DEC-008,2026-08-28 09:30 JST)
**审批**:架构师(Mavis 接手 agent per DEC-008)+ 自审 + 2026-08-28
**修订人**:Ulysses(一人公司 12 角色 per DEC-008)— Mavis 接手
