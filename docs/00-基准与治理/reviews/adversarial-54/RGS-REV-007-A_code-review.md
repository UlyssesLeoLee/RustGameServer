# RGS-REV-007-A 工程 53+54 代码对抗性审核报告

**审核对象**：RustGameServer 工程 53 + 54 全部 Rust 代码 + migration + proto
**审核子代理**：code-review-adversarial-001
**审核时间**：2026-08-22
**commit 基线**：`2486aef`（main ahead origin 82 commits）
**审核范围**：crates/ 下 86 .rs + 10 .toml + 7 .sql + 7 .proto

---

## 1. 严重度统计

- CRITICAL：6 个（必须修，否则 56.x 阻塞）
- HIGH：8 个（55.x 必修）
- MEDIUM：10 个（55.x/56.x 处理）
- LOW：7 个（57.x+ 改进）

合计：**31 issues**

---

## 2. CRITICAL Issues

### C1. 5 域 main.rs 全部使用 InMemoryRepository —— 生产数据零持久化

- **位置**：
  - `crates/player-service/src/main.rs:39-46`
  - `crates/economy-service/src/main.rs`（同模式）
  - `crates/match-service/src/main.rs:35-37`
  - `crates/social-service/src/main.rs:34-35`
  - `crates/admin-service/src/main.rs:35-36`
  - `crates/cluster-ops/src/main.rs:35-36`
- **类别**：资源管理 / 业务逻辑
- **问题**：5 域 binary 全部硬编码 `Arc::new(InMemoryXxxRepository::new())`，`DATABASE_URL` 环境变量被 `env::var("DATABASE_URL").context("DATABASE_URL env required")?` 校验但**从未**用于构造 `PgPool`；`db::pool_from_env` 在 player/main.rs:44 被调用后结果被丢弃，`PgPlayerRepository` 等从未 wired。
- **影响**：**生产部署**只要走这条 main 路径，所有玩家 / 账户 / 对局 / 公会 / 审计日志 全部活在进程内存；服务重启 / OOM / k8s 滚动更新 → 数据全丢。`pg_repository.rs` 一行都没被生产调用。
- **修复建议**：
  ```rust
  // main.rs
  let pool = db::pool_from_env().await.context("DB pool init")?;
  run_migrations(&pool).await.context("migrations")?;
  let players: Arc<dyn PlayerRepository> = Arc::new(PgPlayerRepository::new(pool));
  ```

---

### C2. `client_interceptor` 每次请求生成新随机 `trace_id` —— 分布式追踪彻底失效

- **位置**：`crates/shared-platform/src/grpc_tracing.rs:54-67`（`client_interceptor`）
- **类别**：业务逻辑 / 观测
- **问题**：
  ```rust
  pub fn client_interceptor(request: Request<()>) -> Result<Request<()>, Status> {
      let span = Span::current();
      let _ = span; // 占位 — 完整 trace_id 提取需 tracing-opentelemetry 0.25 API 适配
      let trace_id = Uuid::new_v4();  // 每次都新生成
      let span_id = Uuid::new_v4();
      let traceparent = build_traceparent(trace_id, span_id);
      ...
  }
  ```
  每条 gRPC 出栈请求都会**新**生成 trace_id / span_id，**完全切断**上下游 trace 关联。注释自承认是"占位"。生产里 trace 父链是断的，OTel 后端无法做跨服务调用追踪。
- **影响**：跨域调用排障全靠日志，OTel 全链路 0 价值；SLO / 错误归因 / 延迟分布全部失真。
- **修复建议**：用 `tracing_opentelemetry::OpenTelemetrySpanExt` 拿当前 Span 的 OTel context（`span.context().trace_id()` / `span_id()`），或用 `tracing::Span::current().record` 直接桥接。

---

### C3. economy `credit` / `debit` 多步写无事务包裹 —— 资金可凭空增减

- **位置**：`crates/economy-service/src/service.rs:73-114`（credit）、`116-162`（debit）、`174-188`（freeze_account）
- **类别**：业务逻辑
- **问题**：
  ```rust
  // credit
  account.credit(amount);
  let updated = self.accounts.update_with_version(&account).await?;  // step 1
  let mut entry = TransactionLedger::new(...);
  entry.status = TransactionStatus::Confirmed;
  self.ledger.save(&entry).await?;  // step 2：非原子
  Ok(entry)
  ```
  `update_with_version` 与 `ledger.save` 是**两个独立 SQL**。若 step1 成功 step2 失败（网络抖动 / DB 临时故障 / OOM）：
  - credit: 余额已加但账目漏记 → 凭空生钱
  - debit: 余额已减但账目漏记 → 钱被销毁
  - freeze_account: status 已写但 version 不匹配 → OCC 静默成功
- **影响**：资金账目**永久**不一致，且无 audit log 记录（账目表本身就是审计源）。玩家一旦发现可走客服退款路径，运营侧直接亏损。
- **修复建议**：用 `sqlx::Acquire::begin` 开事务；或定义 `apply_with_ledger_atomic()` repository 方法接收 `&mut Transaction`；参考 RGS-SPEC-CROSS-005 事务性消息应同步纳入 outbox。

---

### C4. `SagaOrchestrator::ReserveHandler / ConfirmHandler` 全部 no-op —— saga 业务逻辑空壳

- **位置**：`crates/economy-service/src/saga_orchestrator.rs:144-193`（`ReserveHandler::execute` / `compensate`、`ConfirmHandler::execute` / `compensate`）
- **类别**：业务逻辑
- **问题**：
  ```rust
  async fn execute(&self, saga: &mut Saga) -> Result<()> {
      if let Some(step) = saga.current() {
          if let Some(resource_id) = step.resource_id {
              let r = Reservation::new(saga.id, resource_id, 100, Currency::Gold);
              tracing::info!(target: "saga", saga_id = %saga.id, reservation_id = %r.id, "ReserveHandler executed");
              // r 从未被持久化到 reservations 表
          }
      }
      Ok(())
  }
  ```
  `Reservation::new` 构造完直接 drop；`ConfirmHandler` 只打 log；`compensate` 全 no-op。Orchestrator 框架在跑，但**实际业务等于零**。
- **影响**：转账 / 商城 / 每日奖励 三个 saga type 全部跑通也只更新 `sagas` 表，不动账户、不动预留表、不发 outbox 消息。"saga 完成"≠"业务完成"。
- **修复建议**：handler 必须 `self.reservations.save(&r).await?` + 调 `update_with_version` 扣减可用余额；或把 ReserveHandler 通过 `Arc<dyn ReservationRepository>` 注入。

---

### C5. `audit_log` hash 链读-改-写无并发保护 —— 审计完整性被破坏

- **位置**：`crates/admin-service/src/service.rs:119-132`（`audit_log`）、`crates/admin-service/src/repository.rs:202-210`（`PgAuditLogRepository::latest`）
- **类别**：并发安全
- **问题**：
  ```rust
  async fn audit_log(...) -> Result<AuditLogEntry> {
      let prev = self.audit.latest().await?;                  // 读
      let prev_hash = prev.map(|e| e.hash).unwrap_or_else(...);
      let entry = AuditLogEntry::new(actor_id, ..., prev_hash);
      self.audit.append(&entry).await?;                      // 写
  }
  ```
  这是典型的 check-then-act 竞态。两个并发 audit 调用都读到 `prev_hash=H_n`，都生成 `H_{n+1}`，两条 entry 的 `prev_hash` 都是 `H_n`。**DB 层 `audit_log` 表对 `prev_hash` 无 UNIQUE 约束**（migration 只对 `hash` 加 UNIQUE），所以两条都能写入。Hash 链分叉 → 篡改检测算法永远察觉不到部分篡改。
- **影响**：管理员后台可绕过审计。SEC-100 §7 hash 链抗篡改承诺破功。
- **修复建议**：
  1. 加 `UNIQUE (prev_hash)` 约束（migration 加列），并发竞争时第二个 append 直接 `23505 unique_violation` 报错；
  2. OR `BEGIN; SELECT ... FOR UPDATE; INSERT; COMMIT;` 显式序列化；
  3. OR 用 `pg_advisory_xact_lock(0xAUDIT_LOG_LOCK)`。

---

### C6. RBAC `DomainAdmin` 缺 `domain_scope` 时获得 `*:*` —— 权限提升漏洞

- **位置**：`crates/shared-platform/src/rbac.rs:131-194`（`SimpleAuthorizer::new` + `check`）
- **类别**：安全 / API 设计
- **问题**：
  ```rust
  role_permissions.insert(Role::DomainAdmin, vec!["*:*"]);  // 与 SuperAdmin 同权

  fn check(...) {
      for role in &subject.roles {
          if matches!(role, Role::DomainAdmin) {
              if let Some(scope) = &subject.domain_scope {       // 只在 Some 时校验
                  if !resource.starts_with(scope) { return deny; }
              }
              // 缺 scope：scope 校验整体跳过
          }
          if let Some(perms) = self.role_permissions.get(role) {
              for p in perms {
                  if *p == "*:*" { return Allow; }            // 立即全权
                  ...
              }
          }
      }
  }
  ```
  攻击面：管理员只填 `Role::DomainAdmin` 而**漏填** `domain_scope`（DB schema 允许 NULL：migration `admin-service/0001_init.sql:10`），`Subject` 构造时 `domain_scope: None` → 通过 `*:*` 拿到全域 SuperAdmin 权限。同 `rbac.rs:227-244` 的 `player_admin()` 测试用例明确依赖 `Some("player")`，缺该字段的负面用例不存在。
- **影响**：单点权限提升，绕过 DTL-019 §3.1 + DEC-005 五域边界。
- **修复建议**：
  ```rust
  // 把 "DomainAdmin 但无 scope" 显式 deny
  if matches!(role, Role::DomainAdmin) && subject.domain_scope.is_none() {
      return CheckResult::deny_if("domain_admin requires domain_scope");
  }
  ```
  外加测试 `domain_admin_without_scope_denied()`。

---

## 3. HIGH Issues

### H1. 6 域 migration 全部缺 `outbox` 表 —— 事务性消息表 DTL-100 §5.3 未落地

- **位置**：6 个 `crates/<domain>/migrations/0001_init.sql`（除 economy 0002_saga_init 之外）
- **类别**：业务逻辑 / 资源
- **问题**：`shared-platform/src/outbox.rs:307` 提供 `MIGRATION_TEMPLATE` 模板，但实际 6 域 migration 都没有 `CREATE TABLE outbox`。意味着即使业务方调 `OutboxRepository::append` 也会因 `relation "outbox" does not exist` 立刻崩。
- **影响**：跨域事件（玩家注册 → 通知 / 任务系统 / 排行榜）只能走同步 RPC，破坏 per DTL-100 §5.3 事务性消息 + 55.x 跨域事件链。
- **修复建议**：在每个域的 `0001_init.sql` 末尾追加 `MIGRATION_TEMPLATE` 内容；或在 `db.rs::run_migrations` 后追加 `outbox` 单 DDL。

### H2. `InMemoryAccountRepository::update_with_version` 对不存在账户静默 `Ok`

- **位置**：`crates/economy-service/src/repository.rs:285-302`
- **类别**：错误处理
- **问题**：
  ```rust
  match guard.get(&account.id) {
      Some(existing) if existing.version == account.version => { ... }
      Some(_) => Err(...),
      None => Ok(account.clone()),   // 不存在 = 假装成功
  }
  ```
  Pg 版本 `rows_affected() == 0` 正确返错。InMemory 版本让测试假阳性：所有"用 InMemory 测试通过"不代表 Pg 也会通过。
- **影响**：InMemory 测过、Pg 部署炸。生产事故难复现。
- **修复建议**：`None => Err(crate::Error::NotFound { entity: "Account", id: account.id.to_string() })`。

### H3. social-service `create_guild` / `join_guild` / `dissolve_guild` 多步非原子

- **位置**：`crates/social-service/src/service.rs:52-153`
- **类别**：业务逻辑
- **问题**：
  - `create_guild`: save guild → save leader member（两步独立，step2 失败 → guild 存在无 leader）
  - `join_guild`: save member → save guild with member_count+1（step2 失败 → 实际人数与 counter 不一致）
  - `dissolve_guild`: 逐个 delete member（任一失败 → 半完成）+ delete guild（不原子）
- **影响**：公会 / 成员 / 计数器三方数据漂移；UX 表现"显示 5 人但只 3 人查到"。
- **修复建议**：包 `sqlx::Acquire::begin()` 事务；`dissolve_guild` 改 `DELETE FROM guild_members WHERE guild_id = $1` 单 SQL。

### H4. `metrics()` 全局 lazy init `expect("metrics init")` —— 服务启动脆弱

- **位置**：`crates/shared-platform/src/metrics.rs:122-124`
- **类别**：错误处理
- **问题**：
  ```rust
  pub fn metrics() -> &'static Metrics {
      METRICS.get_or_init(|| Metrics::new().expect("metrics init"))
  }
  ```
  任意一个 `register_*_with_registry!` 失败（重复注册 / 资源耗尽 / OOM）→ `panic!` 进程崩。`/metrics` scrape 是 SRE 救命的，不能让监控系统拖死业务。
- **影响**：第一次 `record_http_request` 调用 panic；测试套件共享全局 → 跨测 `register_counter_vec_with_registry!` 重复注册会 panic 整个 cargo test 进程。
- **修复建议**：返回 `Result<&'static Metrics, MetricsError>`，调用方决策；或用 `OnceLock<Option<Metrics>>` + 内部 `eprintln!` 降级。

### H5. `init_tracing` 与 `init_json_logging` 互斥约束无强制 —— 全局 subscriber race

- **位置**：`crates/shared-platform/src/tracing_init.rs:11-13`、`crates/shared-platform/src/json_logging.rs:15-18`
- **类别**：并发安全 / 资源
- **问题**：注释明确"二选一不可同时调用"，但代码没有任何 mutual exclusion 机制：
  - 各自有独立 `OnceLock<()>`，分别 set 不会冲突；
  - 但 `tracing::subscriber::set_global_default` 第二次调用会 `Err(AlreadyInitialized)`，原 `try_init` 把 error 转成 `SubscriberInit` 抛出，**调用方拿到错误**。
  - 没有 `init_tracing_called` 与 `init_json_logging_called` 的双向检查。
- **影响**：55.x 多 module 各自 init 的概率上升时无清晰报错。
- **修复建议**：定义 `static SUBSCRIBER_INIT: OnceLock<&'static str>` 标记"已 init 哪种"，第二次调用显式 `Err(AlreadyInitialized { existing: ... })`。

### H6. 6 域 `main.rs` 全用 `tracing_subscriber::fmt()` 直连 —— shared-platform observability 是死代码

- **位置**：6 个 `crates/<domain>/src/main.rs`（第 20-28 行附近）
- **类别**：资源 / 工程
- **问题**：所有 main binary 都直接 `use tracing_subscriber::fmt; ... .init();`，**没有**任何一处调 `shared_platform::tracing_init::init_tracing` / `init_json_logging` / `init_metrics`。`OtelConfig` / `JsonLogError` / `Metrics` 全部只是 lib crate 内部能编译过；运行期从未被触发。
- **影响**：OTel / JSON 日志 / Prometheus endpoint 在 55.x SRE 接入时需要改 6 个 main.rs；但更糟的是当前根本没接，运维只看到默认 fmt 文本日志。
- **修复建议**：55.x 在 main.rs 顶部引入 `shared_platform::tracing_init::init_tracing(...)` 统一替换 `fmt()..init()`。

### H7. `SagaOrchestrator::reservations` 字段标 `#[allow(dead_code)]` —— 不完整实现的遮羞布

- **位置**：`crates/economy-service/src/saga_orchestrator.rs:35-36`
- **类别**：API 设计 / 业务逻辑
- **问题**：
  ```rust
  /// Reservation 仓储（保留供后续 step handler 内部使用；54.8 编排器自身不直接访问）
  #[allow(dead_code)]
  reservations: Arc<dyn ReservationRepository>,
  ```
  注释自承"54.8 编排器自身不直接访问"，但 `new()` 强制要求传 `Arc<dyn ReservationRepository>`（行 42-44）。这迫使 `economy-service/src/main.rs` 必须实例化一个毫无用处的 reservation repo。**这是 C4 的姐妹问题**：要么真用（注入到 handler），要么删字段 + 删构造函数参数。
- **影响**：API 表面失真；55.x review 时易被误判"已经接好"而漏看。
- **修复建议**：
  - 选项 A（推荐）：把 `reservations: Arc<dyn ReservationRepository>` 移到 `ReserveHandler` 自己的 `new()`，Orchestrator 只持 `Vec<Arc<dyn SagaStepHandler>>`；
  - 选项 B：Orchestrator 在 step handler 接口暴露 `set_reservations(...)`。

### H8. `PlayerService::register` 唯一昵称 check-then-insert 竞态

- **位置**：`crates/player-service/src/service.rs:70-84`（`register`）
- **类别**：并发安全
- **问题**：
  ```rust
  if self.players.find_by_name(&name).await?.is_some() { return Err(NicknameTaken); }
  let player = Player::new(name);
  self.players.save(&player).await?;   // DB unique violation 兜不住
  ```
  两个并发同昵称请求都通过 find_by_name，都尝试 INSERT。Pg 版本会因 `name UNIQUE` 触发 unique_violation 但被 `?` 透传成 `Error::Database(_)`，不是 `Error::NicknameTaken`，客户端拿到 `gRPC Internal` 而不是 `AlreadyExists`。
- **影响**：客户端体验是"内部错误"而非"昵称已占用"，但更糟是**未处理时可能暴露 SQL 错误细节**。
- **修复建议**：在 service 层 match `sqlx::Error::Database(e)` if `e.is_unique_violation()` → 翻译成 `NicknameTaken`；或在 repo `save` 内置 upsert 错误码翻译。

---

## 4. MEDIUM Issues

### M1. 各域 enum 解析 `parse_*` 静默 fallback 到默认 variant

- **位置**：
  - `crates/economy-service/src/repository.rs:372-378`（parse_currency → Gold）
  - `crates/economy-service/src/repository.rs:388-394`（parse_account_status → Active）
  - `crates/economy-service/src/repository.rs:406-414`（parse_transaction_kind → Compensation）
  - `crates/economy-service/src/repository.rs:425-432`（parse_transaction_status → Pending）
  - `crates/economy-service/src/reservation.rs:122-128`（parse_currency → Gold）
  - `crates/economy-service/src/reservation.rs:139-145`（parse_status → Reserved）
  - `crates/economy-service/src/inbox.rs:89-94`（parse_status → Processed）
  - `crates/economy-service/src/saga.rs:287-293`（parse_saga_type → DailyReward）
  - `crates/economy-service/src/saga.rs:306-315`（parse_saga_status → Failed）
  - `crates/player-service/src/repository.rs:399-406`（parse_status → Active）
  - `crates/match-service/src/repository.rs:334-340`（parse_mode → BattleRoyale）
  - `crates/match-service/src/repository.rs:352-358`（parse_status → Cancelled）
  - `crates/admin-service/src/repository.rs:338-345`（parse_role → Support）
  - `crates/shared-platform/src/outbox.rs:152-158`（parse_status → Pending）
- **问题**：DB 出现新 enum 值或手动 SQL 误写时，解析器**静默**降级到"看起来正常"的 variant，调用方完全无感。审计 / 调试时困难。
- **修复建议**：在 dev / staging 环境用 `tracing::warn!` 记录 unknown value；production 用 `Result::Err(Error::Database(...))` 上抛。

### M2. `player_service::list_paginated` COUNT + SELECT 非事务

- **位置**：`crates/player-service/src/repository.rs:164-188`
- **问题**：先 `SELECT COUNT(*)` 再 `SELECT ... OFFSET LIMIT`，两条独立查询，期间 INSERT 增删行会**让 total 与 items 不一致**（total=10 但 items 跨两页错位）。
- **修复建议**：`sqlx::Acquire::begin()` + REPEATABLE READ；或用 window function `COUNT(*) OVER ()` 单 SQL。

### M3. `rgs-testkit::mock::NoopMock` 全部 no-op —— 测基础设施空壳

- **位置**：`crates/rgs-testkit/src/mock.rs:36-55`
- **问题**：`DbMock::mock_url()` 返回固定字符串，`GrpcMock::serve()` / `NatsMock::publish()` 直接 `Ok(())`。任何引用 `NoopMock` 的测试都在测"空"。55.x 集成测试想用 testcontainers + mockito + async-nats-mock 时发现接口面是对的但实现空。
- **修复建议**：把 `NoopMock` 改名 `PlaceholderMock` 标明 status；55.x 接入 `testcontainers` + `mockito` + `async-nats-mock` 实现真实行为。

### M4. `rgs-certgen` 生成私钥写盘 + 错误被 `let _ =` 吞

- **位置**：`crates/rgs-certgen/src/main.rs:64-67`
- **问题**：
  ```rust
  for domain in &cli.domains {
      let _ = generate_server_cert(&cli.output, domain, &ca_cert, &ca_key, cli.validity_days)?;
      println!("[rgs-certgen] 服务证书已生成: {}.crt.pem", domain);
  }
  ```
  `generate_server_cert` 内部 `fs::write` 失败 → `?` 上抛外层 `main`；但**这条** `let _ = ...` 屏蔽了返回值路径——外层 `?` 拿不到内部 err。注释自承"53.11 占位 self-signed"。
- **修复建议**：去掉 `let _ =`；私钥文件加 0600 权限（`std::os::unix::fs::PermissionsExt`）。

### M5. `outbox_relay` 公开测试不构造真实 producer

- **位置**：`crates/shared-platform/src/outbox_relay.rs:155-167`
- **问题**：`relay_tick_empty` 注释自承"用 None jetstream 不可行；跳过构造，仅测空 list"——只测了 `list_pending=[]` 的空 happy path。max_retries 触发 giveup 的分支、partial batch 失败的分支均无测试。
- **修复建议**：用 `Producer::new` 接一个测试用 jetstream（可 mock）或 trait-ify producer 让 `tick()` 单测可注入 fake。

### M6. economy `display_name` 用 `format!("{:?}-{:?}", ...)` debug 输出

- **位置**：`crates/economy-service/src/service.rs:261`
- **问题**：`display_name: format!("{:?}-{:?}", account.currency, account.player_id)` —— `{:?}` 是 Rust debug 输出，格式不稳定（`Debug` trait 改字段顺序会断 client）。
- **修复建议**：`format!("{}:{}", account.currency.as_str(), account.player_id)`；为 `Currency` 加 `as_str()` 方法。

### M7. `tracing::info!` 等日志无结构化字段对生产不友好

- **位置**：`crates/economy-service/src/saga_orchestrator.rs:163`（仅字符串）、各 service.rs
- **问题**：saga_orchestrator 的 `tracing::info!` 已经用结构化字段（good），但很多 domain 还在 `println!`（rgs-certgen）或 `tracing::info!("xxxxx")` 字符串拼接。ELK / Loki 索引困难。
- **修复建议**：55.x 全量替换为 `tracing::info!(field = %value, "msg")`。

### M8. `InboxRepository::append` `ON CONFLICT (command_id, handler) DO NOTHING` 静默吞重复

- **位置**：`crates/economy-service/src/inbox.rs:122-137`
- **问题**：`append` 用 `ON CONFLICT DO NOTHING` 但不返回"是否真的插入"。`inbox` 表的幂等语义本意如此，但上层需要 `fetch_by_command` 才能确认；两次 round-trip 浪费。
- **修复建议**：`INSERT ... ON CONFLICT ... RETURNING id` 拿返回值判断。

### M9. `retry::rand_u64` 用 `SystemTime` 哈希 —— 抖动质量差

- **位置**：`crates/shared-platform/src/retry.rs:74-82`
- **问题**：注释自承"per RGS-AI 不引入新 dep 原则"。但 `DefaultHasher::new()` + `SystemTime::now().hash()` 在低并发场景下两线程同一纳秒调会得到**完全相同**的 jitter → thundering herd 雪崩风险。
- **修复建议**：用 `std::cell::RefCell` + thread-local 状态，或 `parking_lot::Mutex<SmallRng>`；OR 直接引入 `rand` crate（5KB 编译代价）。

### M10. `OutboxRelay::tick` 单 publish 失败即跳过余下 entry 的部分错误吞没

- **位置**：`crates/shared-platform/src/outbox_relay.rs:60-98`
- **问题**：`for entry in pending` 内若 `mark_sent` / `mark_failed` DB 失败（不是 publish 失败），整个循环直接 `?` 上抛，本批剩余 entry 全部不被处理，下个 tick 再来。
- **修复建议**：循环内对每个 entry 的 DB 标记错误降级为 `tracing::error!` 继续下一个；只在 publish 决策层（拿到 `Ok(())` vs `Err(_)`）逻辑严格。

---

## 5. LOW Issues

| # | 描述 | 位置 |
|---|------|------|
| L1 | `rbac.rs` `Role::Player` 校验中 `*p == "player:self"` 冗余（permission_matches 已通配符验证） | `crates/shared-platform/src/rbac.rs:178` |
| L2 | `#![allow(clippy::all)]` 在 `shared-platform/src/proto.rs` 全量抑制（含 correct lints） | `crates/shared-platform/src/proto.rs:6` |
| L3 | 6 域 `lib.rs` / `error.rs` 全部 `#![allow(clippy::result_large_err)]` 顶层开，掩盖了具体函数尺寸 | 各 `crates/*/src/error.rs:9` |
| L4 | `rgs-testkit::fixture::EconomyFixture.currency` 字段类型是 `i64` 但 economy 域 `currency` 是 enum `Gold/Diamond/Token`，命名/语义错位 | `crates/rgs-testkit/src/fixture.rs:21` |
| L5 | 6 域 `health_check()` 一律 `Ok(true)`，永远不查 DB / NATS 状态 | 6 个 `service.rs:health_check()` |
| L6 | `outbox.rs::parse_status` 未知字符串默认 `Pending`，掩盖失败 | `crates/shared-platform/src/outbox.rs:152-158` |
| L7 | `rgs-certgen` 输出 CA + server 私钥到磁盘文件，dev 用 ok 但缺 chmod 600 + 路径警示 | `crates/rgs-certgen/src/main.rs:93-94, 124-127` |

---

## 6. 修复优先级矩阵

| Issue | 严重度 | 修复位置 | 预计工时 | 阻塞阶段 |
|-------|--------|----------|----------|----------|
| C1 | CRITICAL | 6 个 `crates/<domain>/src/main.rs` | 2 人·天 | 55.x |
| C2 | CRITICAL | `shared-platform/src/grpc_tracing.rs:54-67` | 0.5 人·天 | 55.x |
| C3 | CRITICAL | `economy-service/src/service.rs:73-188` | 1 人·天 | 55.x |
| C4 | CRITICAL | `economy-service/src/saga_orchestrator.rs:144-193` | 2 人·天 | 56.x |
| C5 | CRITICAL | `admin-service/src/service.rs:119-132` + migration | 0.5 人·天 | 55.x |
| C6 | CRITICAL | `shared-platform/src/rbac.rs:131-194` | 0.3 人·天 | 55.x |
| H1 | HIGH | 6 个 `migrations/0001_init.sql` | 0.5 人·天 | 55.x |
| H2 | HIGH | `economy-service/src/repository.rs:285-302` | 0.1 人·天 | 55.x |
| H3 | HIGH | `social-service/src/service.rs:52-153` | 1 人·天 | 55.x |
| H4 | HIGH | `shared-platform/src/metrics.rs:122-124` | 0.2 人·天 | 56.x |
| H5 | HIGH | `tracing_init.rs` + `json_logging.rs` | 0.3 人·天 | 56.x |
| H6 | HIGH | 6 个 `main.rs` | 1 人·天 | 55.x |
| H7 | HIGH | `economy-service/src/saga_orchestrator.rs:35-50` | 0.3 人·天 | 55.x |
| H8 | HIGH | `player-service/src/service.rs:70-84` | 0.2 人·天 | 55.x |
| M1-M10 | MEDIUM | 见各 issue | 总计 ~3 人·天 | 55.x / 56.x |
| L1-L7 | LOW | 见各 issue | 总计 ~1 人·天 | 57.x+ |

**总工时估算**：CRITICAL 6.3 人·天 + HIGH 3.6 人·天 + MEDIUM 3.0 人·天 + LOW 1.0 人·天 = **约 14 人·天**（AI 协作场景按 token 折算 ≈ 2M-3M tokens）。

---

## 7. 审计员签注

<审计员>：code-review-adversarial-001
<签名>：code-review-adversarial-001
<审计时间>：2026-08-22
<commit>：2486aef
<范围>：86 .rs + 10 .toml + 7 .sql + 7 .proto

**核心结论**：
1. 工程 53+54 的**框架/抽象**层（trait / service / repository / gRPC skeleton / migrations）已经搭起来，命名、模块切分、错误模型大体符合 DTL-015/016/018/019/020/026/100 的契约。
2. 但**实化**层大面积空壳：
   - 6 域 main.rs 全部走 InMemory（Pg 代码写好但没接）
   - saga handlers 全部 no-op
   - outbox 表全部不建
   - audit_log hash 链并发可破
   - rbac DomainAdmin scope 漏洞
   - 分布式追踪是假的
3. 这些不是性能/风格问题，是**功能正确性**问题；如果按当前 binary 部署到 staging，1) 服务重启数据全丢；2) 转账业务实际不扣款；3) 审计日志并发后被分叉；4) 域管理员误填即可拿全权；5) OTel 看板全空。

**建议执行顺序**：
1. **55.x 必修**：C1 + C2 + C3 + C5 + C6 + H1 + H2 + H6 + H8（10 个，工时 ~5.5 人·天），覆盖最致命的功能正确性 + 安全 + 观测问题。
2. **55.x 可选 / 56.x 必修**：C4 + H3 + H4 + H5 + H7（5 个，工时 ~3.5 人·天），覆盖业务实现完整度 + 全局状态。
3. **57.x+ 改进**：M + L（17 个，工时 ~4 人·天），覆盖代码质量 + 测试完整性。

**未审计但建议关注**：
- 6 域 proto 全部只有 `HealthCheck + GetXxx` 两个 RPC，55.x 需扩展 `List / Create / Update / Delete` 全套；当前 service 层的 `register / credit / debit / create_match` 等都未暴露给 gRPC client，**等于不可用**。
- `cluster_ops/src/lib.rs` 未读；建议补审 55.x 上线前。
- `rgs-hello` crate 未审（应为 hello-world 模板，可豁免）。
- `scripts/` 目录未审（部署 / 迁移脚本可能含独立风险）。

**审计环境说明**：
- 由于 verifier 角色限定，本次审计**仅审核 + 报告**，未修改任何项目代码；落盘路径 `D:\RustGameServer\docs\00-基准与治理\reviews\adversarial-54\RGS-REV-007-A_code-review.md`。
- 6 域 `db.rs` 未逐个深读（与各自 `repository.rs` 重复面较多）；如有需要 55.x 重新启动审计。