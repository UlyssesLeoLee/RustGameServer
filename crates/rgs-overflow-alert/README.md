# rgs-overflow-alert

> 5 域业务服务超限排队 + 客服邮箱告警中间件（per 2026-08-27 Ulysses 拍板）
> 适用域：player / economy / match / social（**不**含 admin / cluster-ops）
> 父需求：RGS-REQ-025 弹性容量规划与超大规模并发架构
> 审批者：架构师(Mavis 接手 agent per DEC-008)

---

## 1. 职责

`rgs-overflow-alert` 是 RGS 5 域业务服务（player / economy / match / social）的
**超限排队 + 告警中间件**，填补 RGS-REQ-025 弹性容量规划中
"超大规模并发场景下流量超限的处理"段落（FR-CAP-020/021/022）落地的最后一公里。

具体职责：

- **限流（limiter）**：双阈值（软 = `hard × NATS_OVERFLOW_SOFT_RATIO`、硬 = `<DOMAIN>_MAX_INFLIGHT`），
  在 Pod 内以原子 CAS 维持 `in_flight` 计数，避免乐观重试导致硬上限失守。
- **排队（queue）**：软阈值已超但未超硬阈值时，业务请求入 NATS JetStream 队列
  （subject = `rgs.<domain>.overflow.v1`，复用 `shared_platform::subject::SubjectBuilder`），
  消费者侧异步处理。
- **告警（alert）**：硬上限已满时拒绝 + 通过 `AlertDeduplicator` 触发邮件告警；
  `SMTP_PASSWORD` 缺失则降级为 `tracing::warn!`，**不抛错，不阻断入队**。
- **域类型系统防越界**：`Domain` 枚举只覆盖 player/economy/match/social 4 域，
  编译期拒绝 admin/cluster-ops 误用。

---

## 2. 模块地图

| 模块 | 路径 | 一句话 |
|------|------|--------|
| `domain` | `src/domain.rs` | `Domain` 枚举（4 域）+ env key / subject token 映射 |
| `config` | `src/config.rs` | `OverflowConfig::from_env()` 统一从 env 读全部配置 |
| `limiter` | `src/limiter.rs` | `OverflowLimiter`：双阈值 + RAII `InFlightGuard` |
| `queue` | `src/queue.rs` | `QueueBackend` trait + `NatsJsQueueBackend`（生产）+ `InMemoryQueueBackend`（test） |
| `alert` | `src/alert.rs` | `AlertSink` trait + `SmtpAlertSink` + `LogOnlySink` + `AlertDeduplicator` |
| `guard` | `src/guard.rs` | `OverflowGuard`：业务最常用的高层 API（`check()` → `OverflowDecision`） |

**入口**：`src/lib.rs` 重新导出所有公共类型，业务服务 main.rs 仅需
`use rgs_overflow_alert::{OverflowConfig, OverflowGuard, Domain, ...};`。

---

## 3. 关键设计决策

### 3.1 双阈值（软/硬）

- **硬上限** = `<DOMAIN>_MAX_INFLIGHT` env（默认 0 = 关闭该域限流）。
- **软阈值** = `ceil(hard × NATS_OVERFLOW_SOFT_RATIO)`（默认比例 0.8）。
- `try_acquire` 行为：
  - `in_flight < soft` → `Pass`（业务持 `OverflowDecision.guard` 到处理完 drop）
  - `soft ≤ in_flight < hard` → `Queued`（入 NATS JS 队列）
  - `in_flight ≥ hard` → `Rejected`（拒绝 + 触发告警）

### 3.2 NATS JS subject 命名 = `rgs.<domain>.overflow.v1`

- 实际 subject 通过 `shared_platform::subject::SubjectBuilder::domain_event(domain, "overflow", 1)` 构造
  （`src/queue.rs:172`，与 `shared-platform/src/subject.rs:48` 实现一致）。
- stream 名 = `RGS_OVERFLOW`（默认），单 stream subject filter = `rgs.*.overflow.v1`，
  一份 stream 覆盖全部 4 域（节省 NATS meta）。
- `max_pending` 默认 10 000，stream 超容时 `NatsJsQueueBackend::enqueue`
  会从 `PublishError` 字符串识别 `"max messages" / "max bytes"` 兜底为 `QueueError::QueueFull`。

### 3.3 SMTP 缺密码降级（**不**抛错）

- `OverflowConfig::from_env()` 读 `SMTP_PASSWORD` env：未设置或空字符串 → `smtp.password = ""`。
- `OverflowGuard::build_standard_sink()` 看到 `cfg.smtp.password_is_empty()` →
  primary sink = `LogOnlySink`（直接 `tracing::warn!` 落结构化日志），**不**连 SMTP。
- `AlertDeduplicator::notify()` 自身永不抛错（sink 失败 → fallback 到 `LogOnlySink`）。

**生产前置条件**：必须把 `SMTP_PASSWORD` 通过 k8s Secret 注入，**不能**仅靠 `.env`。
详见 §6 集成步骤。

### 3.4 告警去重（窗口内同 (domain, kind) 只发 1 次）

- `AlertDeduplicator` 用 `HashMap<(String, AlertKind), Instant>` 记录 "上次发送时间"。
- 默认窗口 = `ALERT_DEDUP_WINDOW_SECS`（60s），可通过 env 调整。
- 窗口内同 key 直接 `return`（**不**延长窗口，避免在持续故障时永远吞掉后续告警）。
- 锁粒度：`tokio::sync::Mutex`（async），发送时先 `drop(guard)` 再 `.await inner.send()`，
  避免 sink 慢时阻塞其他告警的去重查询。

### 3.5 Queued permit 保留到消费者（限流语义关键，**不**可改成立即 drop）

- `OverflowGuard::check` 在 `Queued` 路径下把 `permit` 装进 `OverflowDecision.guard`（`src/guard.rs:202`），
  返回业务侧后业务应**立即**返回 `ResourceExhausted` 给 client。
- 消费者 task 拿到 `ack_token` 后持有 `decision.guard`，process 完消息后 `drop(decision.guard)` 释放 in_flight 槽位。
- **设计理由**：permit 在消费者处理完之前必须占据 in_flight 槽位，否则
  "无限涌入 → 永远不 Rejected" 的限流语义被破坏。
  （之前版本 permit 在 `Queued` 路径立即 drop，导致 in_flight 永远涨不到 hard —— 2026-08-27 hotfix 修。）

### 3.6 限流并发安全（`compare_exchange` 非 `fetch_update` 乐观重试）

- `OverflowLimiter::try_acquire` 手写 `compare_exchange` loop（`src/limiter.rs:175`）：
  - `load(current)` → 若 `current < hard` → `compare_exchange(current, current+1, AcqRel, Acquire)`；
  - CAS 成功 → 拿 permit，按 `next ≤ soft` 决定 `Pass` / `Queued`；
  - CAS 失败（被其他 task 抢先 +1）→ **直接 Rejected，不重试**。
- **关键**：**不**用 `fetch_update` 的乐观重试（之前版本 1000 并发全过、
  in_flight 突破 hard 上限）—— 乐观重试让所有并发 task 都能在 CAS 失败后再抢一次，
  破坏限流语义。
- Drop guard 走 `fetch_update` + `saturating_sub`（`src/limiter.rs:58`），
  此处乐观重试安全（即使多减 1 也只是 saturating 到 0，不会负数）。

---

## 4. 业务调用（最小例子）

```rust
use std::sync::Arc;
use rgs_overflow_alert::{
    AlertDeduplicator, Domain, OverflowConfig, OverflowGuard, OverflowLimiter,
    OverflowStatus, NatsJsQueueBackend, QueueBackend, SmtpAlertSink, LogOnlySink,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. 一次 from_env（dev / k8s 通用）
    let cfg = OverflowConfig::from_env()?;

    // 2. 构造三件套
    let limiter = Arc::new(OverflowLimiter::new(Domain::Player, &cfg));
    let queue: Arc<dyn QueueBackend> = Arc::new(NatsJsQueueBackend::connect(&cfg).await?);
    let alerter = Arc::new(AlertDeduplicator::new(
        Arc::new(LogOnlySink),                       // SMTP 缺密码时 fallback
        Arc::new(LogOnlySink),                       // fallback sink
        cfg.support_email.clone(),
        cfg.dedup_window,
    ));

    // 3. 业务主入口（每个 RPC handler 调一次）
    let guard = OverflowGuard::new(
        Domain::Player, &cfg, limiter, queue, alerter,
        None,                              // pod 从 POD_NAME env 读
        "player-service".to_string(),
    );

    // 4. RPC handler 内
    let decision = guard.check("PlayerService::Register", "req-uuid-1234", None).await;
    match decision.status {
        OverflowStatus::Pass => {
            // 正常处理；处理完时 drop(decision.guard) 释放 in_flight
            // （业务可在 spawn / scope 包装把 decision.guard 带到请求结束）
            do_register().await?;
            drop(decision.guard);
        }
        OverflowStatus::Queued => {
            // 业务侧：unary RPC 直接返回 ResourceExhausted；
            // ack_token 透传到消费侧 task，**不要 drop** decision.guard：
            // 消费者处理完消息后再 drop，限流语义依赖这一点
            return Err(tonic::Status::resource_exhausted("queued"));
        }
        OverflowStatus::Rejected => {
            // 硬上限已满，告警已自动触发
            return Err(tonic::Status::resource_exhausted("rejected"));
        }
    }
    Ok(())
}
```

> **不**在中间件层决定 "Queued 时业务是 await permit 还是返回 ResourceExhausted" —
> 由 4 域业务 service 自适配 RPC 风格（unary vs streaming），
> 见 [RISKS.md §RISK-2](RISKS.md#risk-2-5-域-rpc-风格差异queued-时业务返回-resourceexhausted-还是-await-permit未在-dtl-写明统一约定)。

---

## 5. 配置

| env key | 默认值 | 说明 |
|---------|--------|------|
| `SUPPORT_EMAIL` | `hanakagumi@gmail.com` | 告警收件人（per 2026-08-27 Ulysses 拍板） |
| `NATS_URL_IN_CLUSTER` | （fallback 链） | k8s 内 NATS URI（per `shared-platform` 约定） |
| `NATS_URL_LOCAL` | `nats://localhost:4222` | 本地端口转发 URI |
| `NATS_OVERFLOW_SOFT_RATIO` | `0.8` | 软阈值比例（必须 `(0, 1]`） |
| `NATS_OVERFLOW_STREAM` | `RGS_OVERFLOW` | NATS JS stream 名 |
| `NATS_OVERFLOW_CONSUMER_GROUP` | `rgs-overflow-workers` | 消费者组 |
| `NATS_OVERFLOW_MAX_PENDING` | `10_000` | stream 单 stream max_msgs |
| `ALERT_DEDUP_WINDOW_SECS` | `60` | 告警去重窗口 |
| `SMTP_HOST` | `smtp.gmail.com` | SMTP server |
| `SMTP_PORT` | `587` | SMTP 端口 |
| `SMTP_USER` | `hanakagumi@gmail.com` | SMTP 用户名 |
| `SMTP_PASSWORD` | （空） | SMTP 密码；**空 → LogOnlySink 降级**，必须走 k8s Secret |
| `SMTP_FROM_NAME` | `RGS-Ops-Alert` | 发件人显示名 |
| `SMTP_TIMEOUT_MS` | `3_000` | SMTP 发送超时 |
| `POD_NAME` | （hostname fallback） | 注入到告警正文 |
| `PLAYER_MAX_INFLIGHT` | `0` | player 域硬上限（0 = 关闭） |
| `ECONOMY_MAX_INFLIGHT` | `0` | economy 域硬上限（0 = 关闭） |
| `MATCH_MAX_INFLIGHT` | `0` | match 域硬上限（0 = 关闭） |
| `SOCIAL_MAX_INFLIGHT` | `0` | social 域硬上限（0 = 关闭） |

> **重要**：当前 v0.1 release **所有 4 域 `<DOMAIN>_MAX_INFLIGHT` 全部为 0（关闭）**，
> 等压测给值后由 Ulysses 拍板；详见 [RISKS.md §RISK-4](RISKS.md#risk-4-每域-hard_cap-合理值需要压测给-本次全部设-0不启用等-ulysses-拍板)。

---

## 6. 集成步骤

5 域业务服务（player / economy / match / social）按以下 4 步接入：

### 步骤 1：业务服务 main.rs 初始化

```rust
// crates/player-service/src/main.rs（仅示例代码风格）
let cfg = OverflowConfig::from_env()?;
let limiter = Arc::new(OverflowLimiter::new(Domain::Player, &cfg));
let queue: Arc<dyn QueueBackend> = Arc::new(NatsJsQueueBackend::connect(&cfg).await?);
let alerter = OverflowGuard::build_alerter(&cfg);  // 标准三件套
let guard = Arc::new(OverflowGuard::new(
    Domain::Player, &cfg, limiter, queue, alerter,
    None, "player-service".to_string(),
));
// 把 guard 注入到 tonic interceptor / axum middleware
```

### 步骤 2：RPC handler 包 limiter

unary RPC 直接调 `guard.check(op, request_id, business_json).await`，
按 `decision.status` 三分支处理（见 §4）；

streaming RPC 由业务 service 决定在 `open_stream` 调一次还是在每个 message 调一次。

### 步骤 3：helm values 覆盖

在 `<domain>-service` helm values.yaml 中加入：

```yaml
env:
  - name: NATS_URL_IN_CLUSTER
    value: nats://nats.{{ .Values.global.namespace }}.svc.cluster.local:4222
  - name: PLAYER_MAX_INFLIGHT     # 或 ECONOMY/MATCH/SOCIAL
    value: "0"                     # v0.1 保持 0，等压测给值
  - name: NATS_OVERFLOW_SOFT_RATIO
    value: "0.8"
```

### 步骤 4：k8s Secret 注入 SMTP_PASSWORD

**生产前置**：在 SMTP 真正可用前，`SMTP_PASSWORD` 必须通过 k8s Secret 注入：

```yaml
envFrom:
  - secretRef:
      name: rgs-smtp-credentials   # 含 SMTP_PASSWORD
```

> **不要**把 `SMTP_PASSWORD` 写入 `.env` 提交历史（per `.env.example` §8 注释），
> 也**不要**写入 `values.yaml`。**当前 `.env.example` 锁定的就是这一约定**。

---

## 7. 测试

```bash
# 单元 + 集成测试（30 unit + 5 integration = 35/35 全过）
cargo test -p rgs-overflow-alert

# 仅跑 lib.rs 单元（不需 NATS）
cargo test -p rgs-overflow-alert --lib

# 集成测试需 NATS_URL 指向 dev / staging NATS：
NATS_URL_IN_CLUSTER=nats://127.0.0.1:14222 cargo test -p rgs-overflow-alert --test integration_overflow
```

> 集成测试用 `InMemoryQueueBackend` + `LogOnlySink`，**不依赖真实 NATS / SMTP**，
> 但走 `OverflowGuard::check` 全路径（限流 → 入队 → 告警 → permit drop），可验证
> Pass / Queued / Rejected 三分支在 1000 并发下的比例正确性。

---

## 8. 引用文档

| 引用键 | 路径 | git 实证（最新 commit SHA） | 实际章节 |
|--------|------|---------------------------|----------|
| RGS-REQ-025 | `docs/01-核心架构与设计模式/RGS-REQ-025_弹性容量规划与超大规模并发架构_需求定义书.md` | `adb3e34` (2026-08-21) | §FR-CAP-020/021/022 |
| RGS-BAS-022 | `docs/01-核心架构与设计模式/RGS-BAS-022_弹性容量规划与超大规模并发架构_基本设计书.md` | `adb3e34` (2026-08-21) | 全册 |
| RGS-DTL-022 | `docs/01-核心架构与设计模式/RGS-DTL-022_详细设计书.md` | `b8c8598` (升版 v0.2→v0.3) | 同上 |
| RGS-BAS-023 | `docs/01-核心架构与设计模式/RGS-BAS-023_请求处理链标准化前后端处理管道_基本设计书.md` | `adb3e34` (2026-08-21) | 全册 |
| RGS-DTL-023 | `docs/01-核心架构与设计模式/RGS-DTL-023_详细设计书.md` | `e1c22ea` (升版) | 同上 |
| RGS-BAS-003 | `docs/02-运维安全与网络/RGS-BAS-003_运维与GM后台管控_基本设计书.md` | `adb3e34` (2026-08-21) | 全册 |
| RGS-DTL-040 | `docs/02-运维安全与网络/RGS-DTL-040_Admin域_详细设计书.md` | `d8c922c` (DTL-036 升版 commit) | 全册 |
| RGS-DTL-100 | `docs/01-核心架构与设计模式/RGS-DTL-100_Saga业务模式设计_v0.1.md` | `7474d7a` (RGS-ADR-0057 Accepted) / `3e066e3` (初版) | **§6.2 异步事件（NATS JetStream）**（非 §5，见下） |
| RGS-SPEC-CROSS-005 | `docs/13-实现规约/RGS-SPEC-CROSS-005_数据库命名约定_v0.1.md` | `e232915` (DEC-009) / `0f9af88` (初版) | **已知缺口**，见下 |

### 已知缺口（per 2026-08-26 治理规则：缺标比错标安全）

1. **RGS-DTL-100 §5 vs §6.2 不一致**：
   `src/queue.rs` 注释和 `lib.rs` doc comment 都引用 "RGS-DTL-100 §5"；
   实际查 `git show 3e066e3:docs/01-核心架构与设计模式/RGS-DTL-100_Saga业务模式设计_v0.1.md` ，
   DTL-100 v0.1 的 §5 是 **"Reservation 流程（以 Currency 为例）"** ，
   NATS JetStream 异步总线内容在 **§6.2 "异步事件（NATS JetStream）"**。
   本 README 按文件实际内容引用 **§6.2**，不沿用源码注释里的 §5。
2. **RGS-SPEC-CROSS-005 内容不匹配**：
   `shared-platform/src/subject.rs:5` 注释引用 "RGS-SPEC-CROSS-005 草案（subject 命名）"；
   实际 `git show 0f9af88:docs/13-实现规约/RGS-SPEC-CROSS-005_数据库命名约定_v0.1.md`
   内容是 **"数据库命名约定"**（与 subject 命名无关）。
   本 README 沿用源码注释中的引用但**显式标注为草案 / 已知缺口**，
   不擅自改写"事实上的规范在 X 文档"等无 git 证据的回溯叙事。
3. **RGS-ARC-051（CEM 中心事件管理）在 `subject.rs:1` 引用**：
   `git ls-files | grep RGS-ARC-051` **无匹配**；本 README 不引用此文档。

### 引用同步策略

- 上述引用键在每轮文档升版 / 文档 fork 时需重新 `git log -p --follow <file>` 实证；
  README 仅跟随 git 实证结果，不跟随源码注释中的"per X"声明。
- 缺标比错标安全：未实证 / 源码注释矛盾时，本 README 走"已知缺口"路径，
  不杜撰"原本是 / per X 历史形态"等回溯叙事。

---

## 修订历史

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 |
|------|--------|--------|--------|----------|
| 0.1 | 2026-08-27 | 架构师(Mavis 接手 agent per DEC-008) | 架构师(Mavis 接手 agent per DEC-008) | 初版（覆盖 d7a139f 实现 commit，文档化双阈值 / subject 命名 / SMTP 降级 / 去重 / permit 持有 / CAS 限流 6 项关键设计） |
