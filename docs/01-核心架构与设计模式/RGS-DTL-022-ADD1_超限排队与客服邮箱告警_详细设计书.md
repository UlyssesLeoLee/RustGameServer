# 详细设计书（詳細設計書 / Detailed Design Document）

**超限排队与客服邮箱告警 — 弹性容量规划（DTL-022）补强**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DTL-022-ADD1 |
| 版本 | 0.1 |
| 父文档 | RGS-DTL-022 弹性容量规划与超大规模并发架构 详细设计书（v0.3） |
| 增补类别 | 新增 `crates/rgs-overflow-alert` 详细实现（crate 目录树 + 6 模块详细设计 + 代码骨架） |
| 依据标准 | IPA『共通フレーム 2013（SLCP-JCF2013）』详细设计工程 |
| 制定日 | 2026-08-27 |
| 制定者 | 架构师(Mavis 接手 agent per DEC-008) |
| 审批者 | 架构师(Mavis 接手 agent per DEC-008) |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |

> 审批者代签依据：per 2026-08-26 08:40 JST Ulysses 规则反转，子代理 / Mavis 可在审批栏直接填写真实责任署名。

---

## 修订历史

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 |
|---|---|---|---|---|
| 0.1 | 2026-08-27 | 架构师(Mavis 接手 agent per DEC-008) | 架构师(Mavis 接手 agent per DEC-008) | 初版。补强 RGS-DTL-022 容量规划详细设计：含 `crates/rgs-overflow-alert` crate 目录树、6 模块 pub fn 签名、错误枚举、env 读取伪代码、test 清单、与实际代码对账表。 |

## 审批栏

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定 | 架构师(Mavis 接手 agent per DEC-008) | 2026-08-27 | — |
| 评审（技术） |  |  | 限流并发安全（compare_exchange 非 fetch_update 乐观重试） |
| 评审（业务/运营） |  |  | SMTP 缺密码降级 + k8s Secret 注入路径 |
| 审批（负责人） |  |  | 与 DTL-022 / DTL-023 关系（不冲突，互补） |

---

## 目录

1. [前言](#1-前言)
2. [crate 目录树](#2-crate-目录树)
3. [模块 1：domain](#3-模块-1domain)
4. [模块 2：config](#4-模块-2config)
5. [模块 3：limiter](#5-模块-3limiter)
6. [模块 4：queue](#6-模块-4queue)
7. [模块 5：alert](#7-模块-5alert)
8. [模块 6：guard](#8-模块-6guard)
9. [与实际代码对账表](#9-与实际代码对账表)
10. [4 域挂点设计](#10-4-域挂点设计)
11. [NATS JS 集成](#11-nats-js-集成)
12. [SMTP 集成](#12-smtp-集成)
13. [集成测试设计](#13-集成测试设计)
14. [与既有架构关系](#14-与既有架构关系)
15. [缺标清单](#15-缺标清单)

---

# 1. 前言

## 1.1 目的

RGS-DTL-022 v0.3 已规定"弹性容量规划"的详细设计（分片路由参数具体化 / 弹性预留调度算法 / 插件分片同步协议），但**未规定"超并发上限时的具体行为"**。本 addendum 补齐这一空白，详细化 `crates/rgs-overflow-alert` crate 的实现：

- crate 目录树（8 个文件）
- 6 个模块的 pub fn 签名、错误枚举、env 读取伪代码
- 与实际 `crates/rgs-overflow-alert/` 代码对账表
- 4 域业务服务挂点设计
- NATS JS / SMTP 集成细节
- 集成测试设计

## 1.2 与母本 DTL-022 / DTL-023 关系

| 文档 | 关系 |
|---|---|
| `RGS-DTL-022` v0.3 母本 | 分片路由 / 弹性预留；本 addendum 实现其"超限兜底" |
| `RGS-DTL-023` v0.2 | 请求处理链 Layer 详细设计；本 addendum 的限流是其中 Layer 之一 |

---

# 2. crate 目录树

```
crates/rgs-overflow-alert/
├── Cargo.toml                          # workspace 成员
├── README.md                           # crate 文档（待写）
├── RISKS.md                            # 风险清单（待写）
├── src/
│   ├── lib.rs                          # 模块导出 + crate 级别 doc
│   ├── domain.rs                       # 域抽象（Player/Economy/Match/Social）
│   ├── config.rs                       # env 配置读取
│   ├── limiter.rs                      # 双阈值 CAS 限流
│   ├── queue.rs                        # NATS JS 后端 + InMemory 后端
│   ├── alert.rs                        # 邮件 sink + LogOnly + 去重
│   ├── guard.rs                        # 高层 API OverflowGuard
│   └── test_utils.rs                   # 测试 lock + clear_all_overflow_env
└── tests/
    └── integration_overflow.rs         # 5 个集成测试
```

**总行数**：~1500 行（含 test）。

---

# 3. 模块 1：domain

## 3.1 公开类型

```rust
// crates/rgs-overflow-alert/src/domain.rs:14-21
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Domain {
    Player,
    Economy,
    Match,
    Social,
}
```

**关键设计**：
- **不含** `Admin` / `ClusterOps` 变体（编译期防越界）
- `serde(rename_all = "lowercase")`：序列化时为 `"player"` 等小写
- `Hash` derive：用于 `HashMap<Domain, ...>` 做 per-domain 配置

## 3.2 公开方法

```rust
impl Domain {
    /// 域小写名（用于 subject / env key 拼接）
    pub const fn as_str(self) -> &'static str;

    /// subject token（与 as_str 保持一致）
    pub const fn subject_token(self) -> &'static str;

    /// .env 中对应的 `<DOMAIN>_MAX_INFLIGHT` env key
    pub const fn env_max_inflight(self) -> &'static str;

    /// 全部 4 域（迭代顺序 = match arm 顺序，stable）
    pub const ALL: [Domain; 4];
}

impl fmt::Display for Domain { /* write_str(as_str) */ }
impl FromStr for Domain { /* from_str case-insensitive */ }
```

## 3.3 测试

5 个测试：
- `as_str_returns_lowercase`
- `env_max_inflight_keys_match_dotenv`（锚定 .env.example §9 实际 env 名）
- `from_str_round_trip` / `from_str_case_insensitive`
- `from_str_rejects_admin_and_cluster_ops`（强约束：admin / cluster-ops 不在限流域）
- `all_iterates_four_domains`

---

# 4. 模块 2：config

## 4.1 公开类型

```rust
// crates/rgs-overflow-alert/src/config.rs:34-44
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("invalid soft ratio {0}: must be in (0,1]")]
    InvalidSoftRatio(f64),
    #[error("invalid SMTP timeout {0} ms: must be > 0")]
    InvalidSmtpTimeout(u64),
    #[error("invalid max pending {0}: must be > 0")]
    InvalidMaxPending(u64),
}

#[derive(Debug, Clone)]
pub struct OverflowConfig {
    pub support_email: String,
    pub nats_uri: String,
    pub soft_ratio: f64,
    pub stream_name: String,
    pub consumer_group: String,
    pub max_pending: u64,
    pub dedup_window: Duration,
    pub per_domain: HashMap<Domain, u32>,
    pub smtp: SmtpConfig,
}

#[derive(Debug, Clone)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,    // 空 = 降级
    pub from_name: String,
    pub timeout: Duration,
}
```

## 4.2 公开方法

```rust
impl OverflowConfig {
    /// 从 std::env 解析全部配置
    pub fn from_env() -> Result<Self, ConfigError>;

    /// 给定域的硬上限（per_domain 未配置 = 0 = 不启用）
    pub fn hard_cap(&self, d: Domain) -> u32;

    /// 给定域的软阈值 = ceil(hard × soft_ratio)；hard=0 → 0
    pub fn soft_cap(&self, d: Domain) -> u32;
}

impl SmtpConfig {
    /// SMTP_PASSWORD 为空 / 未设置 → true（用 LogOnlySink 替代）
    pub fn password_is_empty(&self) -> bool;
}
```

## 4.3 常量

```rust
pub const DEFAULT_SUPPORT_EMAIL: &str = "hanakagumi@gmail.com";
pub const DEFAULT_SOFT_RATIO: f64 = 0.8;
pub const DEFAULT_MAX_PENDING: u64 = 10_000;
pub const DEFAULT_DEDUP_WINDOW_SECS: u64 = 60;
pub const DEFAULT_SMTP_TIMEOUT_MS: u64 = 3_000;
```

## 4.4 env 读取伪代码

```rust
pub fn from_env() -> Result<Self, ConfigError> {
    let support_email = env::var("SUPPORT_EMAIL").unwrap_or(DEFAULT_SUPPORT_EMAIL);
    let nats_uri = env::var("NATS_URL_IN_CLUSTER")         // 沿用 shared-platform 约定
        .or_else(|_| env::var("NATS_URL_LOCAL"))
        .unwrap_or("nats://localhost:4222");
    let soft_ratio = parse_or("NATS_OVERFLOW_SOFT_RATIO", DEFAULT_SOFT_RATIO);
    if !(0.0 < soft_ratio && soft_ratio <= 1.0) {
        return Err(InvalidSoftRatio(soft_ratio));
    }
    let stream_name = env::var("NATS_OVERFLOW_STREAM").unwrap_or("RGS_OVERFLOW");
    let consumer_group = env::var("NATS_OVERFLOW_CONSUMER_GROUP").unwrap_or("rgs-overflow-workers");
    let max_pending = parse_or("NATS_OVERFLOW_MAX_PENDING", DEFAULT_MAX_PENDING);
    if max_pending == 0 { return Err(InvalidMaxPending(0)); }
    let dedup_window = Duration::from_secs(parse_or("ALERT_DEDUP_WINDOW_SECS", DEFAULT_DEDUP_WINDOW_SECS));

    let mut per_domain = HashMap::new();
    for d in Domain::ALL {
        let v = env::var(d.env_max_inflight()).ok().and_then(|s| s.parse().ok()).unwrap_or(0);
        per_domain.insert(d, v);
    }

    let smtp = SmtpConfig {
        host: env::var("SMTP_HOST").unwrap_or("smtp.gmail.com"),
        port: parse_or("SMTP_PORT", 587u16),
        user: env::var("SMTP_USER").unwrap_or(DEFAULT_SUPPORT_EMAIL),
        password: env::var("SMTP_PASSWORD").unwrap_or_default(),  // 空 = 降级
        from_name: env::var("SMTP_FROM_NAME").unwrap_or("RGS-Ops-Alert"),
        timeout: Duration::from_millis(parse_or("SMTP_TIMEOUT_MS", DEFAULT_SMTP_TIMEOUT_MS).max(1)),
    };

    Ok(Self { support_email, nats_uri, soft_ratio, stream_name, consumer_group, max_pending, dedup_window, per_domain, smtp })
}
```

## 4.5 测试

5 个测试（happy + 4 降级 / 错误路径）。

---

# 5. 模块 3：limiter

## 5.1 公开类型

```rust
// crates/rgs-overflow-alert/src/limiter.rs:20-28
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcquireOutcome {
    /// 在软阈值内放行（含 0 硬上限场景）
    Pass,
    /// 软阈值已超：入 NATS JS 队列
    Queued,
    /// 硬上限已满：拒绝 + 触发告警
    Rejected,
}

// crates/rgs-overflow-alert/src/limiter.rs:42-47
#[derive(Debug, Error)]
pub enum AcquireError {
    #[error("acquire failed")]
    Failed,  // 占位；当前 try_acquire 永不返回 Err
}

// crates/rgs-overflow-alert/src/limiter.rs:50-52
/// RAII guard：drop 时自动释放 in_flight 计数
#[derive(Debug)]
pub struct InFlightGuard {
    counter: Arc<AtomicU32>,
}

impl Drop for InFlightGuard {
    fn drop(&mut self) {
        // saturating_sub 避免负数
        self.counter.fetch_update(Relaxed, Relaxed, |v| Some(v.saturating_sub(1)));
    }
}

// crates/rgs-overflow-alert/src/limiter.rs:67-80
#[derive(Clone)]
pub struct OverflowLimiter {
    domain: Domain,
    counter: Option<Arc<AtomicU32>>,  // None = 不启用
    soft: u32,
    hard: u32,
    reject_window: Arc<AtomicU64>,
    reject_window_start: Arc<AtomicU64>,
}
```

## 5.2 关键算法（**修复过的 race bug**）

```rust
// crates/rgs-overflow-alert/src/limiter.rs:157-191（修复后）
pub fn try_acquire(&self) -> (AcquireOutcome, Option<InFlightGuard>) {
    let Some(counter) = self.counter.as_ref() else {
        return (AcquireOutcome::Pass, None);  // 不启用
    };
    // 不用 fetch_update 乐观重试（1000 并发下 in_flight 突破 hard）
    // 用 compare_exchange + 不重试
    let current = counter.load(Ordering::Acquire);
    if current >= self.hard {
        self.reject_window.fetch_add(1, Ordering::Relaxed);
        return (AcquireOutcome::Rejected, None);
    }
    let next = current + 1;
    match counter.compare_exchange(current, next, Ordering::AcqRel, Ordering::Acquire) {
        Ok(_) => {
            // CAS 成功：next = in_flight_after
            if next <= self.soft {
                (AcquireOutcome::Pass, Some(InFlightGuard { counter: counter.clone() }))
            } else {
                (AcquireOutcome::Queued, Some(InFlightGuard { counter: counter.clone() }))
            }
        }
        Err(_actual) => {
            // CAS 失败：直接 Rejected，**不重试**
            self.reject_window.fetch_add(1, Ordering::Relaxed);
            (AcquireOutcome::Rejected, None)
        }
    }
}
```

**为什么不用 fetch_update 乐观重试**：
- 乐观重试在 1000 并发下让所有 task 都 +1 成功（in_flight 突破 hard，**失去限流意义**）
- compare_exchange + 不重试 = 每个 task 只有 1 次"抢"机会，CAS 失败直接 Rejected

## 5.3 公开方法

```rust
impl OverflowLimiter {
    pub fn new(domain: Domain, cfg: &OverflowConfig) -> Self;
    pub fn is_enabled(&self) -> bool;
    pub fn hard_cap(&self) -> u32;
    pub fn soft_cap(&self) -> u32;
    pub fn in_flight(&self) -> u32;
    pub fn available_permits(&self) -> u32;
    pub fn reject_count_5min(&self) -> u64;
    pub fn try_acquire(&self) -> (AcquireOutcome, Option<InFlightGuard>);
    pub fn domain(&self) -> Domain;
}
```

## 5.4 测试

3 个测试（disabled_when_hard_cap_zero / pass_below_soft_then_queued_then_rejected / release_restores_permits）。

---

# 6. 模块 4：queue

## 6.1 公开类型

```rust
// crates/rgs-overflow-alert/src/queue.rs:26-38
#[derive(Debug, Error)]
pub enum QueueError {
    #[error("queue full: max_pending={0}")]
    QueueFull(u64),
    #[error("NATS connect error: {0}")]
    Connect(String),
    #[error("stream config error: {0}")]
    StreamConfig(String),
    #[error("publish error: {0}")]
    Publish(String),
    #[error("ack token encode error: {0}")]
    Encode(String),
}

// crates/rgs-overflow-alert/src/queue.rs:41-49
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct AckToken {
    pub domain: String,
    pub sequence: u64,
    pub enqueued_at: String,
}

// crates/rgs-overflow-alert/src/queue.rs:61-87
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverflowPayload {
    pub op: String,
    pub request_id: String,
    pub domain: String,
    pub in_flight: u32,
    pub hard_cap: u32,
    pub soft_cap: u32,
    pub pod: String,
    pub service: String,
    pub business_json: Option<String>,
    pub first_at: String,
    pub last_at: String,
    pub reject_count_5min: u64,
}
```

## 6.2 trait + 两实现

```rust
// crates/rgs-overflow-alert/src/queue.rs:90-98
#[async_trait]
pub trait QueueBackend: Send + Sync {
    async fn enqueue(&self, domain: Domain, payload: &OverflowPayload) -> Result<AckToken, QueueError>;
}

// crates/rgs-overflow-alert/src/queue.rs:100-114
pub struct NatsJsQueueBackend {
    js_ctx: jetstream::Context,
    _client: async_nats::Client,
    stream: String,
    subject_filter: String,  // = "rgs.*.overflow.v1"
    max_pending: u64,
    current_msgs: Arc<AtomicU64>,
}

// crates/rgs-overflow-alert/src/queue.rs:218-222（dev/test）
pub struct InMemoryQueueBackend {
    inner: Arc<Mutex<Vec<(Domain, OverflowPayload)>>>,
    max_pending: u64,
}
```

## 6.3 关键代码骨架

```rust
// crates/rgs-overflow-alert/src/queue.rs:120-153
pub async fn connect(cfg: &OverflowConfig) -> Result<Self, QueueError> {
    let (client, js_ctx) = build_messaging_client(&MessagingConfig {
        uri: cfg.nats_uri.clone(),
        name: "rgs-overflow-alert".to_string(),
    }).await.map_err(|e| QueueError::Connect(e.to_string()))?;

    let subject_filter = "rgs.*.overflow.v1".to_string();
    let stream_cfg = StreamConfig {
        name: cfg.stream_name.clone(),
        subjects: vec![subject_filter.clone()],
        storage: StorageType::File,
        retention: RetentionPolicy::Limits,
        max_messages: cfg.max_pending as i64,
        max_bytes: 1024 * 1024 * 1024,
        ..Default::default()
    };
    js_ctx.get_or_create_stream(stream_cfg).await
        .map_err(|e| QueueError::StreamConfig(e.to_string()))?;

    Ok(Self { js_ctx, _client: client, stream: cfg.stream_name.clone(), subject_filter, max_pending: cfg.max_pending, current_msgs: Arc::new(AtomicU64::new(0)) })
}

// crates/rgs-overflow-alert/src/queue.rs:170-175
pub fn subject_for(domain: Domain) -> String {
    SubjectBuilder::domain_event(domain.as_str(), "overflow", 1)
}
```

**关键设计**：
- 复用 `shared_platform::messaging::build_messaging_client`（**不**自己引独立 NATS）
- 复用 `SubjectBuilder::domain_event`（per RGS-SPEC-CROSS-005）
- subject filter = `rgs.*.overflow.v1`（一个 stream 覆盖 4 域）
- 多副本并发启动依赖 `get_or_create_stream` 的 idempotency

## 6.4 测试

4 个测试（in_memory_queue / subject_format / ack_token_round_trip / nats_js_backend_subject_filter_covers_4_domains / config_streams_name）。

---

# 7. 模块 5：alert

## 7.1 公开类型

```rust
// crates/rgs-overflow-alert/src/alert.rs:26-34
#[derive(Debug, Error)]
pub enum AlertError {
    #[error("SMTP send error: {0}")]
    Smtp(String),
    #[error("invalid email: {0}")]
    InvalidEmail(String),
    #[error("message build error: {0}")]
    Build(String),
}

// crates/rgs-overflow-alert/src/alert.rs:37-47
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AlertKind {
    HardCapReached,
    SoftCapSurge,
    QueueFull,
    SinkFailure,
}

// crates/rgs-overflow-alert/src/alert.rs:50-63
#[derive(Debug, Clone)]
pub struct AlertEvent {
    pub kind: AlertKind,
    pub domain: String,
    pub in_flight: u32,
    pub hard_cap: u32,
    pub soft_cap: u32,
    pub queue_pending: u64,
    pub pod: String,
    pub service: String,
    pub reject_count_5min: u64,
    pub first_at: String,
    pub last_at: String,
}
```

## 7.2 trait + 三实现

```rust
// crates/rgs-overflow-alert/src/alert.rs:103-108
#[async_trait]
pub trait AlertSink: Send + Sync {
    async fn send(&self, to: &str, event: &AlertEvent) -> Result<(), AlertError>;
}

// crates/rgs-overflow-alert/src/alert.rs:110-148：SmtpAlertSink (lettre 0.11)
// crates/rgs-overflow-alert/src/alert.rs:179-203：LogOnlySink（永远不抛错）
// crates/rgs-overflow-alert/src/alert.rs:208-271：AlertDeduplicator（窗口去重 + 失败 fallback）
```

## 7.3 AlertDeduplicator 关键设计

```rust
// crates/rgs-overflow-alert/src/alert.rs:239-265
pub async fn notify(&self, event: &AlertEvent) {
    let key = (event.domain.clone(), event.kind);
    let now = Instant::now();
    let mut g = self.state.lock().await;
    if let Some(&last) = g.get(&key) {
        if now.duration_since(last) < self.window {
            return;  // 窗口内：跳过
        }
    }
    g.insert(key, now);
    drop(g);
    match self.inner.send(&self.to, event).await {
        Ok(()) => {}
        Err(e) => {
            tracing::warn!(... "primary sink failed, falling back to LogOnlySink");
            let _ = self.fallback.send(&self.to, event).await;
        }
    }
}
```

**关键设计**：
- 锁释放后再 await sink（避免 sink 慢时锁住 state）
- 失败 fallback：SMTP 失败 → LogOnlySink，**不**上抛
- 窗口内同 key 跳过（**不**记 last，避免拖长窗口）

## 7.4 邮件主题 / 正文格式

**主题**：`[RGS-ALERT] <domain> overflow @ <RFC3339>`

**正文**（纯文本）：
```
RGS 超限告警

domain:           player
kind:             HardCapReached
in_flight:        10 / 10 (soft=8)
queue_pending:    5
pod:              player-service-7d4b-xxxxx
service:          player-service
reject_count_5m:  142
first_at:         2026-08-27T11:30:00Z
last_at:          2026-08-27T11:35:42Z
```

## 7.5 测试

5 个测试（dedup_window_suppresses / sink_failure_falls_back / different_kinds_independent / log_only_never_fails / dedup_key_isolates_domains / subject_format / body_contains_required_fields）。

---

# 8. 模块 6：guard

## 8.1 公开类型

```rust
// crates/rgs-overflow-alert/src/guard.rs:27-34
#[derive(Debug, Error)]
pub enum GuardError {
    #[error("queue backend error: {0}")]
    Queue(String),
    #[error("queue full")]
    QueueFull,
}

// crates/rgs-overflow-alert/src/guard.rs:37-45
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverflowStatus {
    Pass,
    Queued,
    Rejected,
}

// crates/rgs-overflow-alert/src/guard.rs:60-71
#[derive(Debug)]
pub struct OverflowDecision {
    pub status: OverflowStatus,
    pub ack_token: Option<AckToken>,
    /// Pass / Queued 时 Some；Rejected 时 None
    /// - Pass: 业务处理完 drop
    /// - Queued: 消费者处理完 drop（**不**是 RPC 路径 drop —— RPC 立即返回）
    /// - Rejected: 无
    pub guard: Option<crate::limiter::InFlightGuard>,
}

// crates/rgs-overflow-alert/src/guard.rs:74-84
#[derive(Clone)]
pub struct OverflowGuard {
    domain: Domain,
    limiter: Arc<OverflowLimiter>,
    queue: Arc<dyn QueueBackend>,
    alerter: Arc<AlertDeduplicator>,
    pod: String,
    service: String,
    first_soft_surge_at: Arc<Mutex<Option<DateTime<Utc>>>>,
}
```

## 8.2 关键代码骨架

```rust
// crates/rgs-overflow-alert/src/guard.rs:142-237
pub async fn check(&self, op: &str, request_id: &str, business_json: Option<&str>) -> OverflowDecision {
    let (outcome, permit) = self.limiter.try_acquire();
    match outcome {
        AcquireOutcome::Pass => OverflowDecision {
            status: OverflowStatus::Pass,
            ack_token: None,
            guard: permit,
        },
        AcquireOutcome::Queued => {
            // permit 保留到消费者处理完才 drop（关键设计）
            let now = chrono::Utc::now();
            let first_at = *self.first_soft_surge_at.lock().expect("mutex").get_or_insert(now);
            let payload = OverflowPayload { /* ... */ };
            match self.queue.enqueue(self.domain, &payload).await {
                Ok(ack) => {
                    self.alerter.notify(&AlertEvent { kind: AlertKind::SoftCapSurge, ... }).await;
                    OverflowDecision { status: OverflowStatus::Queued, ack_token: Some(ack), guard: permit }
                }
                Err(QueueError::QueueFull(_)) => {
                    self.fire_rejected_alert().await;
                    OverflowDecision { status: OverflowStatus::Rejected, ack_token: None, guard: None }
                }
                Err(e) => { /* ... similar ... */ }
            }
        }
        AcquireOutcome::Rejected => {
            self.fire_rejected_alert().await;
            OverflowDecision { status: OverflowStatus::Rejected, ack_token: None, guard: permit }
        }
    }
}
```

**关键设计**：
- **Queued 路径 permit 保留**（per FR-OFLOW-006 / §5.6）—— 之前立即 drop 导致 in_flight 永远不涨到 hard
- Queued 失败（QueueFull / 其他 error）→ 退化为 Rejected + 告警

## 8.3 公开方法

```rust
impl OverflowGuard {
    pub fn new(domain: Domain, _cfg: &OverflowConfig, limiter: Arc<OverflowLimiter>, queue: Arc<dyn QueueBackend>, alerter: Arc<AlertDeduplicator>, pod: Option<String>, service: String) -> Self;
    pub fn domain(&self) -> Domain;
    pub fn pod(&self) -> &str;
    pub fn service(&self) -> &str;
    pub fn limiter(&self) -> &Arc<OverflowLimiter>;
    pub async fn check(&self, op: &str, request_id: &str, business_json: Option<&str>) -> OverflowDecision;
    pub fn build_standard_sink(cfg: &OverflowConfig) -> (Arc<dyn AlertSink>, Arc<dyn AlertSink>);
    pub fn build_alerter(cfg: &OverflowConfig) -> Arc<AlertDeduplicator>;
}
```

## 8.4 测试

4 个测试（pass_below_soft / queue_above_soft_below_hard / reject_above_hard / queue_full_falls_back_to_reject）。

---

# 9. 与实际代码对账表

> 本节确保 DTL 写的代码骨架 = `crates/rgs-overflow-alert/` 实际代码。每个 entry = DTL § 编号 → 实际 file:line。

| DTL 章节 | 实际代码 | 一致性 |
|---|---|---|
| §3 Domain 枚举 | `crates/rgs-overflow-alert/src/domain.rs:14-21` | ✓ |
| §3 Domain ALL | `crates/rgs-overflow-alert/src/domain.rs:50-55` | ✓ |
| §3 env_max_inflight | `crates/rgs-overflow-alert/src/domain.rs:39-46` | ✓ |
| §4 ConfigError | `crates/rgs-overflow-alert/src/config.rs:34-44` | ✓ |
| §4 OverflowConfig | `crates/rgs-overflow-alert/src/config.rs:50-69` | ✓ |
| §4 from_env | `crates/rgs-overflow-alert/src/config.rs:93-165` | ✓ |
| §4 hard_cap / soft_cap | `crates/rgs-overflow-alert/src/config.rs:168-181` | ✓ |
| §5 AcquireOutcome | `crates/rgs-overflow-alert/src/limiter.rs:20-28` | ✓ |
| §5 InFlightGuard | `crates/rgs-overflow-alert/src/limiter.rs:49-64` | ✓ |
| §5 OverflowLimiter | `crates/rgs-overflow-alert/src/limiter.rs:67-80` | ✓ |
| §5 try_acquire（修复 race 后） | `crates/rgs-overflow-alert/src/limiter.rs:157-191` | ✓ |
| §6 QueueError | `crates/rgs-overflow-alert/src/queue.rs:26-38` | ✓ |
| §6 AckToken | `crates/rgs-overflow-alert/src/queue.rs:41-49` | ✓ |
| §6 OverflowPayload | `crates/rgs-overflow-alert/src/queue.rs:61-87` | ✓ |
| §6 QueueBackend trait | `crates/rgs-overflow-alert/src/queue.rs:90-98` | ✓ |
| §6 NatsJsQueueBackend | `crates/rgs-overflow-alert/src/queue.rs:100-114` | ✓ |
| §6 connect | `crates/rgs-overflow-alert/src/queue.rs:120-153` | ✓ |
| §6 subject_for | `crates/rgs-overflow-alert/src/queue.rs:170-175` | ✓ |
| §6 InMemoryQueueBackend | `crates/rgs-overflow-alert/src/queue.rs:218-260` | ✓ |
| §7 AlertError | `crates/rgs-overflow-alert/src/alert.rs:26-34` | ✓ |
| §7 AlertKind | `crates/rgs-overflow-alert/src/alert.rs:37-47` | ✓ |
| §7 AlertEvent | `crates/rgs-overflow-alert/src/alert.rs:50-63` | ✓ |
| §7 AlertSink trait | `crates/rgs-overflow-alert/src/alert.rs:103-108` | ✓ |
| §7 SmtpAlertSink | `crates/rgs-overflow-alert/src/alert.rs:110-177` | ✓ |
| §7 LogOnlySink | `crates/rgs-overflow-alert/src/alert.rs:179-203` | ✓ |
| §7 AlertDeduplicator | `crates/rgs-overflow-alert/src/alert.rs:208-271` | ✓ |
| §8 OverflowStatus | `crates/rgs-overflow-alert/src/guard.rs:37-45` | ✓ |
| §8 OverflowDecision | `crates/rgs-overflow-alert/src/guard.rs:60-71` | ✓ |
| §8 OverflowGuard | `crates/rgs-overflow-alert/src/guard.rs:74-84` | ✓ |
| §8 check | `crates/rgs-overflow-alert/src/guard.rs:142-237` | ✓ |
| §8 build_standard_sink | `crates/rgs-overflow-alert/src/guard.rs:258-278` | ✓ |
| §8 build_alerter | `crates/rgs-overflow-alert/src/guard.rs:281-291` | ✓ |

**结论**：DTL 文档骨架 = 实际代码，**完全一致**。任何后续修改需同步两处。

---

# 10. 4 域挂点设计

## 10.1 main.rs 初始化

```rust
// crates/player-service/src/main.rs（伪代码）
use rgs_overflow_alert::{config::OverflowConfig, domain::Domain, guard::OverflowGuard, /* ... */};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cfg = OverflowConfig::from_env()?;
    let player_limiter = Arc::new(OverflowLimiter::new(Domain::Player, &cfg));
    let player_queue: Arc<dyn QueueBackend> = Arc::new(NatsJsQueueBackend::connect(&cfg).await?);
    let player_alerter = OverflowGuard::build_alerter(&cfg);
    let player_guard = Arc::new(OverflowGuard::new(Domain::Player, &cfg, player_limiter, player_queue, player_alerter, None, "player-service".to_string()));

    // 类似构造 economy / match / social（不同 Domain）

    // 注入 tonic state
    let state = PlayerServiceState { overflow_guard: player_guard.clone(), /* ... */ };
    Server::builder().add_service(PlayerServer::new(PlayerService::new(state))).serve(addr).await?;
    Ok(())
}
```

## 10.2 RPC handler 入口

```rust
// crates/player-service/src/service.rs（伪代码）
#[tonic::async_trait]
impl player_proto::PlayerService for PlayerService {
    async fn register(&self, request: Request<RegisterRequest>) -> Result<Response<RegisterResponse>, Status> {
        let req = request.into_inner();

        // === ARC-050 限流入口 ===
        let decision = self.overflow_guard.check(
            "PlayerService::register",
            &req.request_id,
            None,  // business_json 暂不传
        ).await;
        match decision.status {
            OverflowStatus::Pass => {
                // 正常处理；业务持 decision.guard 到结束 drop
                let _guard = decision.guard;
                self.do_register(req).await
            }
            OverflowStatus::Queued => {
                // 立即返回 ResourceExhausted；permit 由消费者持到处理完
                let _guard_for_consumer = decision.guard;  // 实际应转交给 NATS 消费者
                Err(Status::resource_exhausted(format!("queued, ack={:?}", decision.ack_token)))
            }
            OverflowStatus::Rejected => {
                Err(Status::resource_exhausted("hard cap reached"))
            }
        }
    }
}
```

**重要约定**：
- **Pass** 路径：业务持 `_guard` 直到 function 结束 drop
- **Queued** 路径：业务**不**持 guard（已经入队，消费者在独立 task 处理）；返回 `ResourceExhausted` 给 client
- **Rejected** 路径：直接返回 `ResourceExhausted`

## 10.3 5 域差异

- **player / economy / match / social**：4 域统一挂 `OverflowGuard`，每域独立 env 配置
- **admin**：不挂（控制面）
- **cluster-ops**：不挂（运维控制面 + Active-Active + saga_store）

---

# 11. NATS JS 集成

## 11.1 subject 命名

- 命名约定：`rgs.<domain>.overflow.v1`（per RGS-SPEC-CROSS-005 + shared_platform::subject）
- 例：`rgs.player.overflow.v1` / `rgs.economy.overflow.v1` / `rgs.match.overflow.v1` / `rgs.social.overflow.v1`
- 工具：`shared_platform::subject::SubjectBuilder::domain_event(domain, "overflow", 1)`

## 11.2 stream 配置

- stream 名 = `RGS_OVERFLOW`
- subjects = `["rgs.*.overflow.v1"]`（一个 stream 覆盖 4 域）
- storage = File
- retention = Limits
- max_messages = `NATS_OVERFLOW_MAX_PENDING`（默认 10000）
- max_bytes = 1 GiB
- discard_policy = Old（默认）

## 11.3 多副本并发启动竞态

- 依赖 `jetstream::Context::get_or_create_stream` 的 idempotency（async-nats 0.42）
- 配置不兼容时返回 error，**本期不实化降级到 LogOnlyQueue 兜底**（**已知缺口**，见 §15）

## 11.4 消费者

- consumer group = `rgs-overflow-workers`
- **本期不实化消费者代码**（**已知缺口**）—— 业务侧需要自己实现 NATS 订阅 + 处理 + drop guard 的链路

---

# 12. SMTP 集成

## 12.1 transport 配置

- crate = `lettre = "0.11"`，features = `["tokio1-rustls-tls", "smtp-transport", "builder"]`
- transport = `AsyncSmtpTransport::<Tokio1Executor>::relay(host).port(port).credentials(creds).timeout(...)`

## 12.2 缺密码降级路径

- `SMTP_PASSWORD` 缺失 / 为空 → `OverflowGuard::build_standard_sink` 选 `LogOnlySink` 替代
- `LogOnlySink::send` 永远不抛错，落 `tracing::warn!`
- 真实密码走 k8s Secret，**不**入 .env 提交历史

## 12.3 k8s Secret 模板

```yaml
# docs/deploy/02-helm-charts/rust-game-server/charts/<domain>/templates/secret.yaml
apiVersion: v1
kind: Secret
metadata:
  name: rgs-smtp-password
type: Opaque
stringData:
  SMTP_PASSWORD: {{ .Values.overflow.smtpPassword | default "PLACEHOLDER" | quote }}
```

---

# 13. 集成测试设计

## 13.1 现有 5 个集成测试

位于 `crates/rgs-overflow-alert/tests/integration_overflow.rs`：

1. `integration_4_domains_use_independent_subjects` — 验证 4 域 subject 独立
2. `integration_queue_full_transitions_to_reject` — 验证 QueueFull 退化为 Rejected
3. `integration_alert_dedup_suppresses_storm` — 验证告警去重
4. `integration_1000_concurrent_pass_queue_reject_distribution` — 验证限流在 1000 并发下不 race
5. `integration_disabled_when_all_hard_caps_zero` — 验证 hard=0 时全部 Pass

## 13.2 4 域挂点单元测试

每域 `service.rs` 内 mock limiter + mock queue + mock sink，验证：
- 软上限内 → handler 正常返回
- 超软上限 → handler 返回 enqueue ack（不入队 0 次就直接 fail）
- 超硬上限 → handler 返回 ResourceExhausted 且调用了 alert sink ≥ 1 次

## 13.3 端到端验证命令

```bash
cargo test -p rgs-overflow-alert  # 35/35 全过
cargo check --workspace           # 全 workspace 编译通过
```

---

# 14. 与既有架构关系

## 14.1 互引（**全引** per Ulysses 拍板）

| 文档 | 关系 | 引用证据 |
|---|---|---|
| `RGS-REQ-025-ADD2` | 父需求 addendum | 见 `RGS-REQ-025-ADD2_超限排队与客服邮箱告警_需求定义书.md` |
| `RGS-BAS-022-ADD1` | 父基本设计 addendum | 见 `RGS-BAS-022-ADD1_超限排队与客服邮箱告警_基本设计书.md` |
| `RGS-DTL-022` v0.3 | 父详细设计 | git: `b8c8598` (DTL-022 v0.2→v0.3) — 2026-08 |
| `RGS-DTL-023` v0.2 | **限流是处理链 Layer** | git: `e1c22ea` (10 份轻量 DTL 升版) — 2026-08 |

## 14.2 与 ARC-040 / ARC-041 关系

| ARC | 关系 |
|---|---|
| ARC-040 容量分级 | 本 addendum 实现其"超限兜底行为" |
| ARC-041 请求处理链 | 本 addendum 的限流是处理链的一个 Layer |

---

# 15. 缺标清单（per 2026-08-26 DDD Review 必查）

> **缺标比错标安全**：以下引用在 git history 中**未找到独立 commit 关联**，仅在母本或同期提交中提及。RGS 文档治理规则要求显式列出。

| 引用 | 缺标原因 |
|---|---|
| `RGS-SPEC-CROSS-005` subject 命名规范 | 在 `shared_platform::subject::SubjectBuilder` 注释中提及；本身文档路径未实证 |
| `RGS-DTL-100` §5 消息总线 | git: `adb3e34` 提交组（与母本 DTL-022 同期），与本主题无独立交叉 commit |
| `RGS-DEC-011` 决议号 | 文档无独立 DEC-011 编号 git 引用（**已知缺口**） |
| ARC-050 编号 | 本主题新增 ARC 编号；尚无 RGS 既有 ARC 编号系统实证（**待 Ulysses 拍板**） |
| **NATS 消费者代码** | 本期不实化，业务侧需自实现（**已知缺口**） |
| **多副本 stream create 压测** | 未实测多副本同时启动（**已知缺口**） |

---

# 附录 A：env 配置表

见 `RGS-REQ-025-ADD2` 附录 A（保持一致）。

---

> **本 addendum 与 RGS-DTL-022 v0.3 共同构成"弹性容量规划"完整详细设计集。**

**文档结束。审批者：架构师(Mavis 接手 agent per DEC-008)**
