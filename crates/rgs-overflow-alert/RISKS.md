# rgs-overflow-alert — 风险登记册（v0.1）

> 与 `README.md` 配套：每条 RISK 对应一个明确的 owner / 状态字段。
> 状态：`open`（未缓解）/ `mitigated`（已通过代码 / 配置缓解）/
> `accepted`（已接受残留风险，业务侧走流程签收）。
> 修订者 / 审批者：架构师(Mavis 接手 agent per DEC-008)。

---

## RISK-1: SMTP_PASSWORD 缺失 → 告警只落日志，运维不会收到邮件

- **风险**: 生产环境 `SMTP_PASSWORD` 缺失 / 为空时，`OverflowGuard::build_standard_sink`
  把 primary sink 替换为 `LogOnlySink`，告警仅以 `tracing::warn!` 形式落结构化日志，
  客服邮箱**不**会收到任何邮件。
- **触发条件**:
  1. `.env` / `values.yaml` 未配置 `SMTP_PASSWORD` env（dev 默认行为）；
  2. k8s Secret 注入失败（Secret 名称错误 / 挂载路径错误）；
  3. SMTP_PASSWORD 配了但 `SMTP_HOST` / `SMTP_USER` 错误导致 `SmtpAlertSink::new` 失败，
     触发 `build_standard_sink` 的 secondary fallback（仍降级到 LogOnlySink）。
- **影响**:
  1. 硬上限 Rejected / 软阈值 SoftCapSurge 告警**全部丢失**，
     业务超限不会被运维及时发现，可能在生产事故后 5-30 分钟才被动感知；
  2. 告警去重窗口内同 `(domain, kind)` 只发 1 次，如果 sink 一直没起来，
     第一条日志之外**后续告警都被吞**（RISK-6 协同问题）。
- **缓解**（当前已做）:
  - `LogOnlySink` 落结构化日志（target = `rgs-overflow-alert`），含 domain / kind / in_flight /
    queue_pending / pod / service / reject_count_5min 全字段，可被 Loki / ES / Datadog 检索；
  - `tracing::warn!` 而非 `error!`（避免被 oncall 当成 ERROR 噪声）。
- **建议下一步**:
  1. 在 `crates/player-service` 启动 health check 中加 `SMTP_PASSWORD` 非空断言（fail-fast）；
  2. CI / ArgoCD 同步校验 k8s Secret `rgs-smtp-credentials` 存在；
  3. 加 Prometheus 指标 `rgs_overflow_alert_sink_kind{primary=logonly|smtp}`，
     持续为 1 → 告警到 ops 频道。
- **owner**: SRE Lead（k8s Secret 注入 + 健康检查）+ player/economy/match/social 4 域 Lead（CI 校验）
- **状态**: open（代码层已降级不抛错，但生产前 SMTP 接入是 hard gate）

---

## RISK-2: 5 域 RPC 风格差异 → Queued 时业务返回 ResourceExhausted 还是 await permit 未统一约定

- **风险**: `OverflowGuard::check` 返回 `OverflowStatus::Queued` 后，**不**强制业务行为 —
  4 域业务 service 可能各自决定 "直接 ResourceExhausted" 或 "await permit 后再处理"。
  没有统一约定会让 client 端重试策略、消费者侧 ack 超时配置难以标准化。
- **触发条件**:
  1. 业务 service 选 `await permit`（同步阻塞等 in_flight 槽位），消费者 task 在哪一层做 ack 语义不清晰；
  2. 业务 service 选 `ResourceExhausted` + client 重试，client 重试间隔是否小于消费者 drain 时间窗不确定；
  3. 跨域（player → economy）调用走 gRPC 时，Queued 决策是否在 player 域判定还是 economy 域判定未统一。
- **影响**:
  1. 4 域行为不一致 → 5 域 SLA 无法对齐；
  2. 端到端 P99 latency 在 4 域下不可预测；
  3. SLO 看板按域分桶后数据失真。
- **缓解**（当前已做）:
  - `OverflowDecision.guard` 在 `Queued` 路径**保留** permit（`src/guard.rs:202`），
    消费者侧处理完才 drop — 无论业务选哪种路径，限流槽位语义都不破；
  - `OverflowDecision.ack_token` 透传到消费侧（`Option<AckToken>`，仅 Queued 时 Some），
    业务有充分信息自己决定。
- **建议下一步**:
  1. 在 `RGS-DTL-022/023` 或新增 `RGS-DTL-027` 加 §"超限处理约定" 章节，规定：
     - unary RPC → `ResourceExhausted` + gRPC trailers 带 `retry-after-ms`；
     - streaming RPC（server-streaming / bidi）→ 业务侧 spawn task 等 permit；
  2. 4 域 RACI Lead 各自 sign-off（Ulysses 2026-08-21 已确立"5 域独立 Lead"原则）。
- **owner**: 5 域 Lead 联合（player / economy / match / social）+ 架构师（统一约定）
- **状态**: open

---

## RISK-3: NATS stream create 多副本并发启动的竞态

- **风险**: 业务服务多个副本同时启动时，`NatsJsQueueBackend::connect` 调
  `js_ctx.get_or_create_stream(stream_cfg)` 可能在 NATS 侧产生
  "stream 已存在 / 配置不兼容" 的瞬时报错，影响 Pod readiness probe。
- **触发条件**:
  1. k8s HPA 扩容时 N 副本同时启动；
  2. STS rolling update 时新旧副本并存；
  3. NATS controller 还没初始化 stream（首次部署）。
- **影响**:
  1. 启动失败 → 业务 service 退出 / CrashLoopBackOff；
  2. 即使 stream 已存在但 `max_messages` 改了，async-nats 0.42 的 `get_or_create_stream` 会
     返回 "stream config mismatch" 错误（实测 0.42 行为，理论依赖文档 §3.2 不一致场景）。
- **缓解**（当前已做）:
  - 依赖 async-nats 0.42 的 `get_or_create_stream` 幂等性（stream 名相同 + 配置兼容时**不报错**，
    `src/queue.rs:139-142`）；
  - 失败 → `QueueError::StreamConfig` 错误返回，main.rs 可选择退出 / 重试（业务侧策略）。
- **建议下一步**:
  1. 在 staging 实测 10 副本并发启动（k3s / `kubectl scale --replicas=10`）；
  2. 加 startup retry：业务 main.rs 在收到 `StreamConfig` 错误时**不退**，
     改 1s/2s/4s/8s backoff 重试 5 次（避免被 k8s 重启）；
  3. 在 helm pre-install hook 中加一次性 Job 显式 `nats stream add RGS_OVERFLOW`，
     业务 service 启动时 stream 已就绪。
- **owner**: cluster-ops Lead（k8s 部署协调）+ SRE Lead（启动 retry 策略）
- **状态**: open（未实测多副本并发）

---

## RISK-4: 每域 hard_cap 合理值需要压测给 → 本次全部设 0（不启用），等 Ulysses 拍板

- **风险**: v0.1 release 中 4 域 `<DOMAIN>_MAX_INFLIGHT` 全部默认 0（关闭限流）。
  0 = `OverflowLimiter::new` 不分配 `counter`（`src/limiter.rs:90-93`），
  所有 `try_acquire` 直接 `Pass` —— 中间件不生效。
- **触发条件**:
  1. 业务 service 上线时未在 helm values.yaml 显式设置 `*_MAX_INFLIGHT`（env 默认 0）；
  2. 设置了但值远低于实际承载能力（硬上限过早触发 → 误拒）；
  3. 设置了但值远高于实际承载能力（硬上限失效 → 雪崩时全 Pod 一起挂）。
- **影响**:
  1. 上线初期所有"超限"路径不会真正触发 → 告警 / 排队 / Rejected 都打不出来，
     一旦真实流量超阈值，**第一次会直接走到 Rejected 路径**，运维没有任何提前预警；
  2. 误配 hard_cap = 1 等极小值会导致 99% 请求被 Rejected（player 域尤其严重）。
- **缓解**（当前已做）:
  - v0.1 release 阶段就关掉 hard_cap（业务未压测前不"猜值"），
    README §5 + §6 步骤 3 明确写"v0.1 保持 0，等压测给值"；
  - 限流关闭时中间件不报错，告警降级到日志（仍可观察 in_flight / reject_count_5min 指标）。
- **建议下一步**:
  1. 5 域压测（k6 / vegeta）分别打 baseline 2x / 5x / 10x 流量，观测：
     - 4 域 service P99 latency 在哪个 in_flight 拐点开始劣化；
     - DB connection pool / Redis / NATS 哪个先成为瓶颈；
  2. Ulysses 拍板后由 SRE Lead + 4 域 Lead 联合 sign-off 每域 hard_cap 值，
     写入 `RGS-REQ-025` 附录 + 4 域 helm values.yaml 锁定；
  3. 加 Prometheus 指标 `rgs_overflow_limiter_enabled{domain=...}` 持续为 0 → 告警。
- **owner**: 架构师（Ulysses 拍板）+ 4 域 Lead（压测值 sign-off）+ SRE Lead（values 锁定）
- **状态**: accepted（短期方案，长期必须拍板）

---

## RISK-5: `InFlightGuard` 持有到消费者处理完 → 消费者 bug 不 drop guard 导致硬上限假死

- **风险**: `OverflowDecision.guard` 在 `Queued` 路径下保留到消费者处理完才 drop
  （`src/guard.rs:202`，README §3.5 关键设计）。
  消费者 task 任何 bug（panic / early return / 未消费 ack_token）→ guard 不 drop
  → in_flight 永远 +1 → 后续所有请求全部 Rejected。
- **触发条件**:
  1. 消费者代码 `tokio::spawn(async move { do_work().await; })` 但**忘带** `decision.guard` 进 spawn；
  2. 消费者 task 在 await 中 panic，guard 在 panic 边界 drop 但**已被消费**（OK），
     但 panic 前可能已经 +1 多个 permit 未归还；
  3. ack_token 丢失（序列化失败 / DB 写入失败）→ 消费者未真正开始但已 +1；
  4. 长事务中消费者 await 外部依赖超时（DB / RPC），guard 仍未 drop。
- **影响**:
  1. 单一消费者 bug → in_flight 泄漏 → 整域不可用（硬上限被无效占用）；
  2. 假死期间所有新请求 Rejected，告警被去重窗口吞（RISK-6 协同）；
  3. 排查时需要去 5 域各 Pod 看 in_flight 计数（无现成 dashboard）。
- **缓解**（当前已做）:
  - `InFlightGuard::drop` 走 `fetch_update` + `saturating_sub`（`src/limiter.rs:55-65`），
    即使 panic unwind 时 drop 也能正确释放；
  - `drop(g)` 先于 `inner.send().await`（`src/alert.rs:252`）防止 sink 慢阻塞。
- **建议下一步**:
  1. 消费者侧提供 `consume_queued(ack_token, decision_guard) -> Result<()>` helper，
     强制业务**显式**把 decision_guard 传入 spawn（类型系统保证）；
  2. 加监控指标 `rgs_overflow_in_flight_aged{domain=...}`（in_flight > 60s 未释放告警）；
  3. 单元测试覆盖 "消费者 panic" 场景（`tokio::spawn` + panic 后断言 in_flight 归 0）。
- **owner**: cluster-ops Lead（消费者 sidecar）+ 4 域 Lead（业务侧消费者实现）
- **状态**: mitigated（RAII 已确保 panic 时 drop 正确，但"消费者忘带 guard"是业务 bug 不可强制）

---

## RISK-6: 告警去重 60s 窗口内同 key 只发 1 次 → 重大事件可能被吞

- **风险**: `AlertDeduplicator` 用 `HashMap<(String, AlertKind), Instant>` 在
  `ALERT_DEDUP_WINDOW_SECS`（默认 60s）窗口内对同 `(domain, kind)` **只发 1 次**
  （`src/alert.rs:243-249`）。窗口内后续告警**不**延后窗口，仅 `return` 跳过。
- **触发条件**:
  1. 域硬上限持续超限 60s 以上 → 只有首条告警发出，后续全部被吞；
  2. RISK-1 的 SMTP 不可用场景叠加：SMTP 失败 1 次后整个告警路径走 fallback，
     60s 内同 key 都走 fallback = 同一日志行被重复 1000 次（log spam）；
  3. 告警系统（Grafana / OpsGenie）误判"已告警"（仅看首条），不升级。
- **影响**:
  1. 持续性严重事件（数据库雪崩 / NATS 死锁）只发 1 次告警，运维可能误判"已恢复"；
  2. 告警"升级"语义（first → ack → escalation）缺失；
  3. 与 RISK-1 协同：SMTP 不可用 + 持续超限 = 仅 1 条 log 噪音 + 1 封丢失败邮件。
- **缓解**（当前已做）:
  - **不**延长窗口（"窗口内后续不重置 last" 避免持续故障时永远吞告警，
    `src/alert.rs:246-247` 注释显式说明）；
  - 邮件正文带 `first_at` / `last_at` 字段（`src/alert.rs:62-63`），即使只发 1 封也能看出持续时间；
  - LogOnlySink 每次都 `tracing::warn!` 完整字段（log 端**不**去重）。
- **建议下一步**:
  1. 引入 escalation 机制：
     - 首次发邮件；
     - 持续 5min 超限 → page（PagerDuty / OpsGenie）；
     - 持续 30min → 升 P1 拉群；
  2. 拆 dedup key 为 `(domain, kind, severity)`，分级告警；
  3. 与 RISK-1 协同方案：SMTP 不可用时 fallback 到 **webhook**（Slack / Discord）而非纯 log。
- **owner**: SRE Lead（告警通道多通道化）+ 架构师（escalation 策略 sign-off）
- **状态**: open（已部分缓解，但 escalation 机制未实化）

---

## RISK-7: `AlertDeduplicator` 用 `tokio::sync::Mutex` → 高并发下 `notify()` 串行化

- **风险**: `AlertDeduplicator::notify` 在 `state: Arc<Mutex<HashMap<(String, AlertKind), Instant>>>`
  上 `lock().await`（`src/alert.rs:243`），查询去重后 `drop(g)` 再 `await inner.send()`。
  锁本身不会跨 await 持有（lock 只保护 HashMap 操作，send 已在锁外），
  但**所有 notify() 调用在 Mutex 队列上串行化** —— 4 域 × 4 kind = 最多 16 个独立 key，
  高并发时等于单 key throughput 受限。
- **触发条件**:
  1. 4 域同时硬上限 Rejected，每域 1k QPS 入告警；
  2. 测试场景下 1000 并发 `d.notify(...)`（`tests/integration_overflow.rs` 的 e2e 用例）；
  3. RISK-6 的去重窗口 + 大量误触发叠加。
- **影响**:
  1. 4 域告警走同一把 Mutex 排队，tail latency 抖动（P99 在 sink 慢时不可预测）；
  2. 与 RISK-6 协同：去重窗口 60s + Mutex 串行 = 同一分钟内 1k 次告警可能只真正发送 1 次，
     其余 999 次 await lock 时被 RISK-6 吞掉；
  3. 集成测试 1000 并发场景下 `d.notify` 串行化是性能瓶颈（未实测）。
- **缓解**（当前已做）:
  - 锁粒度 = 整个 HashMap（不分 shard）；
  - drop(g) 在 `inner.send().await` 之前（`src/alert.rs:252`），sink 慢不阻塞其他查询；
  - 锁操作（`get` / `insert`）是 O(1) 摊销，O(n) 退化场景（碰撞）极少见。
- **建议下一步**:
  1. 实测 4 域 × 1k QPS 下 `d.notify` P99 latency（k6 / vegeta 推告警事件）；
  2. 如果 P99 > 10ms 阈值，改 `dashmap::DashMap` 或 `parking_lot::Mutex`（同步锁但短持有）；
  3. 或拆为 per-`(domain, kind)` 锁（16 个独立 Mutex，吞吐近似 16x），
     但需要 lock 顺序约定避免死锁。
- **owner**: 架构师（性能基线 + 拍板改造方案）
- **状态**: mitigated（短锁 + drop-before-send 已消除 sink 慢阻塞；高并发下未压测）

---

## 修订历史

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 |
|------|--------|--------|--------|----------|
| 0.1 | 2026-08-27 | 架构师(Mavis 接手 agent per DEC-008) | 架构师(Mavis 接手 agent per DEC-008) | 初版（7 条 RISK，覆盖 SMTP 降级 / RPC 风格 / NATS 并发 / hard_cap 拍板 / permit 持有 / 告警去重 / Mutex 串行化） |
