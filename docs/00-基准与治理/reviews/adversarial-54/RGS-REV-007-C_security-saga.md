# RGS-REV-007-C 工程 53+54 安全 + Saga 对抗性审核报告

**审核对象**：密码学 / Saga 状态机 / Outbox race / RBAC / mTLS / SQL 注入 / 密钥
**审核子代理**：security-saga-adversarial-001
**审核时间**：2026-08-22
**commit 基线**：`e320c69`

---

## 1. 严重度统计

| 等级 | 计数 | 描述 |
|------|------|------|
| **CRITICAL** | **1** | 密码学：audit_log hash 链使用 FNV-1a 64-bit 非加密学摘要 |
| **HIGH** | **6** | 多副本 outbox_relay 抢占无 SKIP LOCKED；outbox 写入无事务边界；audit_log read-to-write 竞争；mTLS `client_auth_required` 未审计可见；Inbox handler 字符串无白名单；saga `Aborted` 死状态 |
| **MEDIUM** | **7** | 状态机非法转移检测缺失；orchestrator 关键路径 `unwrap()`；`authenticate` 明文等值比较；作用域匹配 `starts_with` 边界宽松；dev 密码 `ulysses_local` 多域共享；`compute_hash` 字段分隔符可碰撞；`*.env.bak` 未排除 |
| **LOW** | **4** | 文档与代码不一致（entity.rs 注释 "sha256" 实际 FNV-1a）；MIGRATION_TEMPLATE 注释与现状脱节；rbac `*` 匹配粒度粗；跨域 RPC 客户端 mTLS 启用仅靠 `use_tls: bool` 标志 |

合计：**18 个独立 issue**。

---

## 2. 攻击面矩阵

| 资产 | 威胁 | 现有防护 | 防护有效性 |
|------|------|---------|-----------|
| 5 域 gRPC RPC 链路（player/economy/match/social/admin） | 中间人 / 凭证窃取 | mTLS（rustls）+ CA bundle + `domain_name` | **部分**：CA 加载 + 客户端 cert 已实化（`tls.rs:44-66`），但 `ServerTlsConfig::client_auth_required(true)` 在审计代码中未出现，存在 fallback 明文路径（`channel.rs:60-78`，`use_tls=false` 不报错） |
| economy 域 Saga 提交 | 重复扣款 / 半完成 | Inbox `(command_id, handler)` UNIQUE + Reservation 模式 | **部分**：约束在 SQL 层有效（`0002_saga_init.sql:49`），但 `handler` 字符串无白名单，且 `InboxRepository::append` 用 `ON CONFLICT DO NOTHING` 静默吞掉冲突（`inbox.rs:122-137`），业务方可能误判 "未处理" 为 "已处理" |
| admin 域 audit_log | 篡改历史 / 抵赖 | hash 链（FNV-1a 64-bit）+ UPDATE/DELETE 触发器禁 | **不足**：FNV-1a 64-bit 是非加密学摘要，攻击者可在多项式时间内构造 hash 碰撞（生日攻击 ~2^32）。触发器保护有效（`0001_init.sql:42-50`），但 hash 强度不构成密码学屏障 |
| 6 域 DB 凭据 | 凭据泄漏 | `.env` 入 `.gitignore`（line 7）+ `.env.example` 占位 `CHANGE_ME_*` | **完备**：`git check-ignore -v .env` → `.gitignore:7:.env`；`.env` 未在 `git ls-files` 中；`ulysses_local` 仅用于本地 dev k3s |
| RBAC 角色检查 | 权限提升 / 越权 | `SimpleAuthorizer` + `DomainAdmin scope` 限定（`rbac.rs:155-194`） | **部分**：`DomainAdmin` 走 `resource.starts_with(scope)`（line 160），`scope="player"` 也会匹配 `player_secret/1` 之类的资源；`*:*` 通配符会绕过特定 perm 检查；`domain_scope = None` 时 DomainAdmin 等同 SuperAdmin（**关键绕过**） |
| outbox 跨域消息 | 重复投递 / 丢失 | `OutboxRelay.tick()` + `retry_count` + DLQ | **不足**：多副本 relay 部署时无 `FOR UPDATE SKIP LOCKED`，会同时拉取同一条 entry 并重复 publish 到 NATS（`outbox_relay.rs:60-98`，`outbox.rs:200-209`） |
| Saga 状态机 | 非法转移 / 状态错乱 | 状态枚举 + 字段更新 | **不足**：`complete()`/`fail()`/`compensate()`/`start()` 均为 pub，无前置状态检查（除 `execute` 的 `Pending` 检查外，`saga.rs:55`），`resume()` 无前置检查可从 `Compensating` 回到 `Running`；`Aborted` 变体定义但无任何写入路径（死状态） |

---

## 3. CRITICAL Issues

### C1. audit_log hash chain 使用 FNV-1a 64-bit 非加密学哈希

- **位置**：`crates/admin-service/src/entity.rs:148-154`（实现）+ `crates/admin-service/src/entity.rs:96`（注释自相矛盾地标注 "sha256"）+ `crates/admin-service/src/service.rs:127-130`（调用方 read-then-append 两次查询，无事务）
- **类别**：密码学 / 篡改防护 / 事务正确性
- **证据**：

  ```rust
  // entity.rs:148-154
  // 简单 FNV-1a 64-bit hash（生产环境用 sha2::Sha256；这里保持无外部依赖）
  let mut h: u64 = 0xcbf29ce484222325;
  for b in s.bytes() {
      h ^= b as u64;
      h = h.wrapping_mul(0x100000001b3);
  }
  format!("{:016x}", h)
  ```
- **问题**：
  1. FNV-1a 64-bit 摘要空间仅 2^64，**生日攻击 ~2^32 次计算即可找到碰撞**（普通笔记本数小时可生成 5 字节碰撞）。
  2. 非加密学哈希不满足 RGS-SEC-100 §7 "密码学 hash 链" 要求；GDPR / 等保 2.0 / SOC2 CC7.1 均要求不可篡改审计（cryptographic non-repudiation）。
  3. 字段拼接用 `|` 分隔但无长度前缀（`write!(s, "{}|{}|{}|{}|{}|{}", actor_id, action, target, payload, prev_hash, created_at_ms)`），`action="a" target="b|0"` 与 `action="a|" target="b"` 会产生相同输入到 hasher，**碰撞构造进一步简化**。
  4. 业务注释（`entity.rs:96`）写 "hash = sha256(...)" 与实际实现 FNV-1a 不一致——**自相矛盾**，未来维护者会按注释误判安全等级。
- **影响**：
  - 攻击者可低成本伪造历史审计条目，且能让伪造条目与原条目具有相同 hash（链不断），事故追溯失效。
  - 触发器禁 UPDATE/DELETE 只能防 SQL 修改；FNV 弱碰撞让 "INSERT 新条目覆盖事件" 仍能达成。
- **修复建议**：

  ```rust
  // Cargo.toml 加依赖
  // sha2 = "0.10"
  // hex = "0.4"

  use sha2::{Sha256, Digest};

  fn compute_hash(
      actor_id: Uuid,
      action: &str,
      target: &str,
      payload: &str,
      prev_hash: &str,
      created_at: DateTime<Utc>,
  ) -> String {
      let mut hasher = Sha256::new();
      // 显式长度前缀防 separator 碰撞
      hasher.update(actor_id.as_bytes());
      hasher.update((action.len() as u64).to_le_bytes());
      hasher.update(action.as_bytes());
      hasher.update((target.len() as u64).to_le_bytes());
      hasher.update(target.as_bytes());
      hasher.update((payload.len() as u64).to_le_bytes());
      hasher.update(payload.as_bytes());
      hasher.update(prev_hash.as_bytes());
      hasher.update(created_at.timestamp_millis().to_le_bytes());
      hex::encode(hasher.finalize())
  }
  ```
- **附加修复**：必须**在事务内**读 `latest()` + `append()`（见 H3），否则 hash 链本身在并发下断裂。
- **测试**：加 `collision_resistance_test`（构造 1000 随机输入，验证 hash 全不同）。
- **依赖变更**：`Cargo.toml` 加 `sha2 = "0.10"` + `hex = "0.4"`。

---

## 4. HIGH Issues

### H1. 多副本 outbox_relay 抢占无 `SELECT ... FOR UPDATE SKIP LOCKED`

- **位置**：`crates/shared-platform/src/outbox.rs:200-209`（`list_pending`）+ `crates/shared-platform/src/outbox_relay.rs:60-98`（`tick`）
- **证据**：

  ```rust
  // outbox.rs:200-209 —— 朴素 SELECT
  async fn list_pending(&self, limit: i64) -> Result<Vec<OutboxEntry>> {
      let rows = sqlx::query(
          "SELECT id, subject, payload, command_id, saga_id, status, retry_count, last_error, created_at, sent_at \
           FROM outbox WHERE status = ''pending'' ORDER BY created_at LIMIT $1",
      )
      ...
  }
  ```
- **攻击 / 故障场景**：
  - 部署 2 副本 outbox_relay（k3s Deployment replicas=2）→ 两个进程同时 `list_pending` 拉到相同 entry → 同一 `command_id` 被 publish 到 NATS 两次 → consumer 端若未严格去重则双倍执行（Saga 重复扣款）。
  - K8s rolling restart 期间短暂双活也会触发。
- **修复建议**：

  ```rust
  async fn list_pending(&self, limit: i64) -> Result<Vec<OutboxEntry>> {
      let mut tx = self.pool.begin().await?;
      let rows = sqlx::query(
          "SELECT id, subject, payload, command_id, saga_id, status, retry_count, last_error, created_at, sent_at \
           FROM outbox WHERE status = ''pending'' \
           ORDER BY created_at \
           LIMIT $1 FOR UPDATE SKIP LOCKED"
      )
      .bind(limit)
      .fetch_all(&mut *tx)
      .await?;
      tx.commit().await?;
      Ok(rows.into_iter().map(row_to_entry).collect())
  }
  ```
  注意：单纯 `SKIP LOCKED` 仍可能在 publish 成功后 mark_sent 之前崩溃导致重复；推荐引入 `in_flight` 状态或 lease 机制。
- **阻塞阶段**：55.x 强阻塞（k3s 多副本部署即触发）。

### H2. outbox `append` 未绑定业务事务（transactional outbox 失效）

- **位置**：`crates/shared-platform/src/outbox.rs:178-198` + 文档承诺 `crates/shared-platform/src/outbox.rs:6-7` + `outbox.rs:118-119`
- **证据**：

  ```rust
  // outbox.rs:178-198
  async fn append(&self, entry: &OutboxEntry) -> Result<()> {
      sqlx::query("INSERT INTO outbox ... VALUES (...)")
          .bind(...)
          .execute(&self.pool)   // ← 直接用 pool，没有 tx 参数
          .await?;
      Ok(())
  }
  ```
- **问题**：trait 签名 `async fn append(&self, entry: &OutboxEntry)` 不接受 `&mut PgTransaction`，调用方**无法**在业务写 DB 同事务内追加 outbox 记录。文档却承诺 "业务写 DB + 写 outbox 表必须在同一事务"（`outbox.rs:6-7`）。如果调用方先 `pool.begin()` 写业务表，再 `outbox.append()` 用 pool 直连，**两个操作分属不同事务**，业务 commit 后 outbox 可能因 DB 故障丢失。
- **攻击 / 故障场景**：
  - economy 转账业务 commit（账户已扣款）→ outbox INSERT 因网络抖动失败 → 转账完成但消息未发出 → consumer 永远不知道要扣 B 账户 → 钱凭空消失。
- **修复建议**：

  ```rust
  #[async_trait]
  pub trait OutboxRepository: Send + Sync {
      async fn append<''e, E: sqlx::PgExecutor<''e>>(
          &self,
          entry: &OutboxEntry,
          executor: E,
      ) -> Result<()>;
  }

  // 业务方用法
  let mut tx = pool.begin().await?;
  accounts.save(&account, &mut *tx).await?;
  outbox.append(&entry, &mut *tx).await?;   // 同一事务
  tx.commit().await?;
  ```

### H3. audit_log 写入 read-to-write 竞争，hash 链可断裂

- **位置**：`crates/admin-service/src/service.rs:119-132`（`audit_log` 方法）
- **证据**：

  ```rust
  // service.rs:127-130
  async fn audit_log(&self, actor_id, action, target, payload) -> Result<AuditLogEntry> {
      let prev = self.audit.latest().await?;     // ← query 1（无锁）
      let prev_hash = prev.map(|e| e.hash).unwrap_or_else(|| "0".repeat(64));
      let entry = AuditLogEntry::new(actor_id, action, target, payload, prev_hash);
      self.audit.append(&entry).await?;           // ← query 2（不同事务）
      Ok(entry)
  }
  ```
- **问题**：
  1. `latest()` 和 `append()` 不在同一事务，无 `SELECT ... FOR UPDATE`。
  2. 两个并发 `audit_log()` 调用可能都看到同一 `prev_hash` → 两条新 entry 的 `prev_hash` 相同 → 链分叉 → hash 校验路径失败（虽然 admin-service 目前**没有 hash 链验证函数**——见下文 §11）。
- **修复建议**：

  ```rust
  async fn audit_log(&self, ...) -> Result<AuditLogEntry> {
      let mut tx = self.audit_pool.begin().await?;
      let prev = sqlx::query("SELECT ... FROM audit_log ORDER BY created_at DESC LIMIT 1 FOR UPDATE")
          .fetch_optional(&mut *tx).await?;
      let prev_hash = prev.map(|e| e.hash).unwrap_or_else(|| "0".repeat(64));
      let entry = AuditLogEntry::new(actor_id, action, target, payload, prev_hash);
      sqlx::query("INSERT INTO audit_log ...").bind(...).execute(&mut *tx).await?;
      tx.commit().await?;
      Ok(entry)
  }
  ```
  （即使升级到 SHA-256，此 race 仍让 hash 链断裂。）

### H4. mTLS `ServerTlsConfig::client_auth_required(true)` 在审计代码中未出现

- **位置**：grep 范围 `crates/` → "No matches found"
- **证据**：审计范围 `crates/shared-platform/src/{tls,channel,client}.rs` 仅含：
  - `load_client_tls`（`tls.rs:44-66`）：客户端加载自己的 cert + CA。
  - `load_server_identity`（`tls.rs:68-79`）：仅加载服务端 cert + key，**未配置 client CA 也不强制校验客户端证书**。
  - `build_channel`（`channel.rs:60-78`）：根据 `cfg.tls: Option<...>` 决定是否启 TLS，**`None` 时静默走明文 HTTP/2**。
  - `build_service_channel`（`client.rs:60-81`）：注释明确 "54.9 范围只演示结构；mTLS 证书路径待 55.x 配置"。
- **问题**：
  - 服务端 `ServerTlsConfig::client_auth_required(true)` 强制要求客户端证书的代码在审计范围内**不存在**。
  - 任何调用方忘记设置 `use_tls: true` 都会回退到明文 gRPC（HTTP/2 over TCP），无任何运行时拒绝。
  - CN/SAN 校验仅在 `ClientTlsConfig::domain_name` 设置，缺失时等于 "accept all"。
- **修复建议**：
  1. 在 `tls.rs` 加 `load_server_tls_config(server_cert, server_key, ca_cert) -> ServerTlsConfig` 函数，其中 `ServerTlsConfig::client_auth_required(true).client_ca_certificate(ca)`。
  2. 在 `build_channel` 中 `cfg.tls = None` 时返回编译期/启动期错误（fail-closed），不要静默回退。
  3. `RpcChannelConfig` 把 `tls: Option<...>` 改成 `tls: TlsConfig`（必填），由 `TlsConfig::Insecure` 显式标记不安全。
- **阻塞阶段**：55.x 强阻塞（生产部署前必须实化）。

### H5. Inbox `handler` 字符串无白名单，可被攻击者污染

- **位置**：`crates/economy-service/src/inbox.rs:46-56`（`InboxEntry::new`）+ `0002_saga_init.sql:42-50`（unique 约束）+ `crates/economy-service/src/inbox.rs:122-137`（`append` 静默吞掉冲突）
- **证据**：

  ```rust
  // inbox.rs:46-56
  pub fn new(command_id: Uuid, handler: String, result: String) -> Self { ... }
  // 无 handler 格式校验、无白名单
  ```
- **问题**：
  1. `handler` 字段类型是 `String`，由调用方传。理论上一个被攻陷的 service 可以用任意 `handler` 名字 `append` 到 inbox，**抢注别的业务 handler 的幂等键组合**。
  2. 实际上 `handler` 是应用层 namespace（"saga.transfer"），但代码未强制。
  3. `ON CONFLICT (command_id, handler) DO NOTHING`（line 126）—— 当幂等冲突时**不返回错误**，业务方难以判断 "插入失败因为已存在" vs "其他原因"。建议改为 `RETURNING` 让调用方显式感知。
- **修复建议**：
  1. `InboxEntry::new` 加 `handler` 白名单校验（`matches! handler { "saga.transfer" | "saga.purchase" | ... }`）。
  2. `append` 改成：

     ```rust
     let result = sqlx::query("INSERT INTO inbox ... ON CONFLICT (command_id, handler) DO NOTHING RETURNING id")
         .bind(...).fetch_optional(&self.pool).await?;
     if result.is_none() {
         return Err(InboxError::Duplicate(command_id, handler));
     }
     ```
- **影响**：当前 6 域均无外部攻击面（handler 来自内部），但**纵深防御**缺失。

### H6. Saga 状态机 `Aborted` 变体定义但无任何转移路径

- **位置**：`crates/economy-service/src/saga.rs:36-51`（`SagaStatus::Aborted` 枚举）+ `crates/economy-service/src/saga.rs:227-231`（`fail` 方法）+ `crates/economy-service/src/saga.rs:306-315`（`parse_saga_status` 把 "aborted" 映射回 Aborted 但 DB CHECK 包含）
- **证据**：

  ```rust
  // saga.rs:36-51
  pub enum SagaStatus {
      Pending, Running, Compensating, Completed, Failed, Aborted,
  }
  // saga.rs:198-232 仅有 start() / complete() / compensate() / fail()
  // Aborted 没有对应方法！
  ```
- **问题**：
  1. DB CHECK 约束接受 `aborted`（`0002_saga_init.sql:12`），Rust 枚举有 `Aborted` 变体，但**没有任何代码路径能把状态设为 Aborted**。
  2. `parse_saga_status` 把 "aborted" 映射到 `Aborted`（line 312）→ 读到 DB 里的 aborted 行能解析成功 → 但代码内**没有方法**把状态写入 Aborted。死状态。
  3. 未来如果 admin 想 "手动中止一个卡住的 Running saga"，会尝试调 `saga.abort()`——不存在该方法，开发者会用 `fail()` 替代，掩盖业务语义。
- **修复建议**：
  1. 短期：移除 `Aborted` 变体 + DB CHECK 约束中的 ''aborted''，保持状态机 5 态（Pending/Running/Compensating/Completed/Failed）。
  2. 中期：若需要 Aborted（运营手动干预），加 `pub fn abort(&mut self)` 方法 + 在 `SagaOrchestrator` 加中止支持。

---

## 5. MEDIUM Issues

### M1. 关键路径 `saga.current_mut().unwrap()` 在 orchestrator 热路径中

- **位置**：`crates/economy-service/src/saga_orchestrator.rs:78, 83, 93`
- **证据**：

  ```rust
  // saga_orchestrator.rs:78
  saga.current_mut().unwrap().mark_running();
  ```
- **问题**：`saga.current_mut()` 返回 `Option<&mut SagaStep>`，但 orchestrator 假设 `current_step` 永远在 `[0, steps.len())` 范围内。如果 `current_step` 因部分写库（崩溃恢复场景下数据 corruption）变成 `steps.len()`，`unwrap()` 直接 **panic**，整个进程崩溃。
- **修复建议**：

  ```rust
  saga.current_mut()
      .ok_or_else(|| Error::Validation(format!("saga {} current_step {} out of bounds", saga.id, saga.current_step)))?
      .mark_running();
  ```

### M2. `SagaOrchestrator::resume` 无前置状态检查

- **位置**：`crates/economy-service/src/saga_orchestrator.rs:130-140`
- **证据**：

  ```rust
  pub async fn resume(&self, saga_id: Uuid) -> Result<()> {
      let mut saga = self.sagas.find_by_id(saga_id).await?.ok_or(...)?;
      self.execute(&mut saga).await    // ← 直接调 execute，无状态检查
  }
  ```
- **问题**：`execute()` 检查 `saga.status != Pending` 则 return Err（line 55-60），但**从 Compensating/Completed/Failed 状态调用 resume 都被静默拒绝**。崩溃恢复的语义应该是：从任何非终态（Running/Compensating）续跑，从终态（Completed/Failed）拒绝。**目前没有 "Running 续跑" 的入口**——因为 `execute()` 只接受 Pending。
- **修复建议**：拆分为 `execute_pending` 与 `resume_in_progress`，后者检查 `status ∈ {Running, Compensating}`。

### M3. `Saga::compensate()` 不触发 handler 实际反向逻辑

- **位置**：`crates/economy-service/src/saga.rs:215-224`
- **问题**：仅把已完成 step 标记为 Compensated，**不调用 handler.compensate()**。如果业务代码绕过 `SagaOrchestrator` 直接调 `saga.compensate()`，反向操作（取消 reservation、退款）**不会发生**，钱被永久扣住。
- **修复建议**：将 `saga.compensate()` 设为 `pub(crate)` 或删除，要求所有补偿走 `SagaOrchestrator::compensate`。

### M4. `authenticate` 用 `password_hash != password_hash` 等值比较

- **位置**：`crates/admin-service/src/service.rs:75`
- **证据**：

  ```rust
  if user.password_hash != password_hash {
      return Err(Error::InvalidCredentials(username));
  }
  ```
- **问题**：
  1. API 契约模糊：调用方传的是 "已 hash 的密码" 还是 "明文密码"？`entity.rs:33` 注释 "密码哈希（argon2id）" 暗示是 hash。
  2. 字符串等值比较不是**常数时间**——`!=` 在第一个不同字节就 short-circuit return true，理论上可侧信道。但实际攻击者需先拿到数据库的 hash 值，且 admin 登录频率低，**实际风险低**，归为 MEDIUM。
- **修复建议**：
  1. API 重命名为 `authenticate(username, plaintext_password)` + 内部用 argon2 verify。
  2. 若坚持接收 hash，用 `subtle::ConstantTimeEq` 做比较。

### M5. RBAC `DomainAdmin` 走 `resource.starts_with(scope)` 边界宽松

- **位置**：`crates/shared-platform/src/rbac.rs:159-167`
- **问题**：
  - `scope="player"` 匹配 `player/123`（OK）也匹配 `player_secret/1`（not OK，如果存在此资源）。
  - 没有"段边界"判断：理想实现应 `resource == scope` 或 `resource.starts_with(&format!("{}/", scope))`。
- **修复建议**：

  ```rust
  if resource != *scope && !resource.starts_with(&format!("{}/", scope)) {
      return CheckResult::deny_if(...);
  }
  ```

### M6. dev 密码 `ulysses_local` 在 6 域 + superuser 全共享

- **位置**：`.env` line 25, 42, 48, 54, 60, 66, 72（全部为 `ulysses_local`）
- **问题**：
  - 虽 `.env` 已 gitignore，但 6 域共享同一密码违反"凭据隔离"（per RGS-BAS-100 §7 独立 schema/user 隔离）。一旦一个域泄漏，所有 6 域 + postgres superuser 全失守。
  - `.env.example` 模板也是 `CHANGE_ME_*` 占位（good），但实际 `.env` 全一致（bad）。
- **修复建议**：
  1. 用 `openssl rand -base64 24` 为每个域生成独立密码，写入 `.env`。
  2. CI/CD 通过 secret manager 注入（k8s Secret），源码仓库不留任何 dev 真实密码。
  3. 文档明确："`ulysses_local` 已被禁止用于非 dev-only / 多域共享"。

### M7. `compute_hash` 字段分隔符 `|` 可被输入数据碰撞

- **位置**：`crates/admin-service/src/entity.rs:137-147`
- **问题**：
  - 无长度前缀。`action="a|" target="b"` 与 `action="a" target="b|"` 拼接结果相同。
  - 即使升级到 SHA-256，**没有长度前缀的拼接仍有理论碰撞风险**（虽然 SHA-256 抗碰撞，但攻击者可构造 `action` 和 `target` 让其内容混淆）。
  - 同字段内含 `|` 也会让拼接结果不一致。
- **修复建议**：使用显式长度前缀（见 C1 修复代码）。

---

## 6. LOW Issues

### L1. 文档与代码不一致：`entity.rs:96` 注释 "sha256" 但实现 FNV-1a

- **位置**：`crates/admin-service/src/entity.rs:96`
- **修复**：与 C1 一并修复——升级到 SHA-256 并修正注释。

### L2. `outbox.rs:307` 注释要求 "各域 migrations 应包含本表" 但 6 域 migrations 文件 1 字节

- **位置**：`crates/shared-platform/src/outbox.rs:306-327` + `crates/*/migrations/0001_init.sql` 各 1 字节（空文件）
- **问题**：MIGRATION_TEMPLATE 是常量字符串，**从未在域 migration 中应用**。如需 outbox 表，需要每个域单独建。
- **修复**：
  1. 在 `economy-service/migrations/0003_outbox.sql` 等具体域 migration 中粘贴模板 SQL。
  2. 或在 `shared-platform` 提供 `setup_outbox_migration()` 函数 + 文档化。

### L3. `rbac.rs:permission_matches` 通配符仅支持 `*` 整段，粒度粗

- **位置**：`crates/shared-platform/src/rbac.rs:197-211`
- **问题**：`*` 只能匹配整段（resource 或 action），不能匹配部分（如 `player.*` 不支持）。
- **修复建议**：用 `matcher` crate 或自实现 glob 匹配，按需扩展。

### L4. `build_service_channel(use_tls: bool)` 显式 bool 标志易被误传

- **位置**：`crates/shared-platform/src/client.rs:60-81`
- **问题**：`use_tls=false` 是合法调用，无任何告警。生产环境若误传 `false`，整个 mTLS 失效。
- **修复建议**：
  1. 拆分为 `build_insecure_channel`（显式命名）+ `build_secure_channel`（默认入口），前者打 `warn!` 日志并加 metric。
  2. 编译期 feature flag：`#[cfg(not(feature = "insecure"))]` 让 `build_insecure_channel` 在生产编译中消失。

---

## 7. Saga 状态机审计

```
[Pending] → start() → [Running] → advance() + complete() → [Completed]
                ↓ step fail                ↑ all completed
                ↓
         [Compensating] → reverse completed steps → [Failed]
                ↓ (no path)
            [Aborted]   ← DEAD STATE（无任何代码路径写入）
```

| 检查项 | 结论 | 证据 |
|--------|------|------|
| 状态转移是否完备 | **N** | `Aborted` 是死状态（见 H6） |
| 非法转移检测 | **N** | `complete()`/`fail()`/`compensate()`/`start()` 均为 `pub`（`saga.rs:199/208/215/227`），无前置状态检查。`start()` 可在 `Compensating` 状态被调用并把状态改回 `Running` |
| 幂等保证 | **部分 Y** | `sagas.command_id` 有 UNIQUE INDEX（`0002_saga_init.sql:21`），但 `find_by_command_id` 仅用于查询，不强制业务方先 check 再 execute；inbox 也有 UNIQUE（`0002_saga_init.sql:49`），但 `append` 用 `ON CONFLICT DO NOTHING` 静默吞冲突（见 H5） |
| 补偿顺序合理性 | **N** | `Saga::compensate` 反向遍历已 Completed step 标记（`saga.rs:215-224`），**不调用 handler.compensate()**。实际反向逻辑在 `SagaOrchestrator::compensate`（`saga_orchestrator.rs:106-127`），但顺序遍历的是 `saga.steps.iter().rev().filter(|s| s.status == Completed)`——已与 `saga.compensate()` 重复执行 |
| resume 入口 | **N** | `resume()` 无前置状态检查（见 M2） |
| 并发安全 | **N** | `save()` 用 `INSERT ... ON CONFLICT (id) DO UPDATE`（`saga.rs:371-378`），无 `SELECT ... FOR UPDATE`，两个 orchestrator 同时 resume 同一 saga 会丢失更新 |

**关键缺失**：

1. 状态机非法转移检测函数不存在——`Saga` 结构体本身应提供 `transition_to(new_status) -> Result<()>` 方法，封装合法转移矩阵。
2. saga 表的 `command_id` UNIQUE INDEX 在并发下不会触发 UPDATE——因为 `save` 用 `ON CONFLICT (id) DO UPDATE`，而并发 resume 是相同 `id`（同 saga），按 PK 走 update，**两个事务都做 read-modify-write 后写回，last-write-wins，丢失一个事务的 step 推进**。

**修复建议**（Saga 状态机强化）：

```rust
// saga.rs 加
impl Saga {
    pub fn transition_to(&mut self, new: SagaStatus) -> Result<()> {
        let allowed = match (self.status, new) {
            (SagaStatus::Pending, SagaStatus::Running) => true,
            (SagaStatus::Running, SagaStatus::Compensating) => true,
            (SagaStatus::Running, SagaStatus::Completed) => true,
            (SagaStatus::Compensating, SagaStatus::Failed) => true,
            _ => false,
        };
        if !allowed {
            return Err(Error::Validation(format!(
                "illegal saga transition {:?} -> {:?}", self.status, new)));
        }
        self.status = new;
        self.updated_at = Utc::now();
        Ok(())
    }
}

// PgRepository::save 改用乐观并发
async fn save(&self, entity: &Saga) -> Result<Saga> {
    let result = sqlx::query(
        "UPDATE sagas SET current_step=$2, steps=$3, status=$4, updated_at=$5, completed_at=$6 \
         WHERE id=$1 AND updated_at=$7"
    ).bind(entity.id).bind(entity.current_step as i32).bind(steps_json)
     .bind(saga_status_to_str(entity.status)).bind(entity.updated_at)
     .bind(entity.completed_at).bind(entity.original_updated_at)
     .execute(&self.pool).await?;
    if result.rows_affected() == 0 {
        return Err(Error::Conflict(format!("saga {} concurrent update", entity.id)));
    }
    Ok(entity.clone())
}
```

---

## 8. Outbox race condition 审计

| 检查项 | 结论 | 证据 |
|--------|------|------|
| `SELECT ... FOR UPDATE SKIP LOCKED` | **N** | grep `SKIP LOCKED` 在 `crates/` 全部 0 匹配 |
| 重复投递去重 | **依赖业务方** | `outbox_relay.rs:67` 成功后 `mark_sent` 仅改 DB 状态，**consumer 端必须依赖 inbox 唯一约束去重**；`outbox.rs:5-9` 注释明确 "consumer 端靠 envelope.command_id 去重" |
| 事务原子性（outbox INSERT + 业务写） | **N** | `append` 不接受 tx 参数（见 H2） |
| 崩溃恢复（relay publish 后 mark_sent 前崩溃） | **Y（重发）** | 至少一次投递语义保留，consumer 端用 inbox 去重；这是设计正确但有副作用——relay 必须在 publish 成功**之后**才 mark_sent，目前实现 OK（`outbox_relay.rs:67-68`） |
| 多副本 relay 抢占 | **N** | 见 H1 |
| max_retries 后转 DLQ | **Y** | `outbox_relay.rs:71-74` 超 max_retries → `mark_giveup` → status=''failed'' |

**关键缺失**：状态机增加 `in_flight` 状态（介于 pending 和 sent 之间），relay 取到 entry 后立刻 `UPDATE ... SET status=''in_flight'', lease_until=now()+30s`，publish 成功 mark_sent，失败 mark_failed（保留 in_flight 状态等 lease 过期后被另一副本重试）。

---

## 9. RBAC 矩阵

| 角色 | scope | 权限 | 资源 | 操作 | 决策 | 绕过风险 | 备注 |
|------|-------|------|------|------|------|---------|------|
| SuperAdmin | None | `*:*` | player/123 | ban | ALLOW | N | `rbac.rs:172-173` 直接 return Allow |
| DomainAdmin(player) | "player" | `*:*` | player/123 | ban | ALLOW | **Y**（`starts_with` 边界宽松，见 M5） | 资源 `player_secret/1` 也会通过 scope 检查 |
| DomainAdmin(player) | "player" | `*:*` | economy/1 | grant | DENY | N | scope mismatch，正确 deny |
| DomainAdmin(player) | None | `*:*` | player/123 | ban | **ALLOW** | **Y（关键）** | `subject.domain_scope = None` 时**不进入** scope check 分支（`rbac.rs:159`），但后续 `*:*` 仍然 Allow——**DomainAdmin 在缺 scope 时等同 SuperAdmin**！需修复 |
| Auditor | None | `*:read` | player/1 | read | ALLOW | N | `permission_matches("*:read", "player:read")` = true（`rbac.rs:298-302`） |
| Auditor | None | `*:read` | player/1 | ban | DENY | N | 权限不匹配 |
| Support | None | `player:read` | player/1 | read | ALLOW | N | 显式匹配 |
| Support | None | `player:read` | guild/1 | read | ALLOW | N | `["player:read", "guild:read"]` 都有 |
| Player | None | `player:self` | `subject.id` | self-access | ALLOW | N | `rbac.rs:178-183` self check |
| Player | None | `player:self` | other-player | self-access | DENY | N | 显式 self check |
| Player | None | `player:self` | `subject.id` 但 perm="player:read" | read | **DENY** | N | `player:self` 不匹配 `player:read`（粒度只匹配 self 操作） |
| 任意角色 | None | `*` 单段通配 | player:read | read | DENY | N | `permission_matches` 长度必须一致，`*` 单独无效 |

**关键绕过**：

1. **DomainAdmin domain_scope = None 等同 SuperAdmin**（`rbac.rs:155-167` 中 `if let Some(scope) = ...` 在 None 时不进入 deny 分支）——应改为 `if role == DomainAdmin && subject.domain_scope.is_none() { return DENY }`。
2. **scope 用 starts_with 边界宽松**（M5）。
3. `permission_matches` 通配符仅整段（`*:read` 不能写成 `player:read*`），粒度粗但不会误放行。

---

## 10. 修复优先级矩阵

| Issue | 严重度 | 文件 | 估时 (人·天) | 阻塞阶段 |
|-------|--------|------|-------------|----------|
| **C1** audit_log FNV-1a → SHA-256 | CRITICAL | admin-service/entity.rs, service.rs, repository.rs | 0.5（含测试） | 55.x 强阻塞（合规审计） |
| **H1** outbox_relay SKIP LOCKED | HIGH | shared-platform/outbox.rs, outbox_relay.rs | 1.0（schema 迁移 + relay 重构） | 55.x（多副本部署） |
| **H2** outbox 事务边界 | HIGH | shared-platform/outbox.rs, 5 域 repository.rs | 1.5（trait 变更影响 5 域） | 55.x |
| **H3** audit_log read-then-append race | HIGH | admin-service/service.rs | 0.5 | 55.x |
| **H4** mTLS client_auth_required 不可见 | HIGH | shared-platform/tls.rs, channel.rs | 1.0（设计 + 实现 + 测试） | 55.x 强阻塞 |
| **H5** Inbox handler 白名单 | HIGH | economy-service/inbox.rs, 6 域调用方 | 0.5 | 56.x |
| **H6** Saga Aborted 死状态 | HIGH | economy-service/saga.rs, 0002_saga_init.sql | 0.3 | 56.x |
| **M1** orchestrator unwrap → Result | MEDIUM | economy-service/saga_orchestrator.rs | 0.2 | 55.x |
| **M2** resume 状态检查 | MEDIUM | economy-service/saga_orchestrator.rs | 0.3 | 55.x |
| **M3** saga.compensate 不触发 handler | MEDIUM | economy-service/saga.rs | 0.2 | 55.x |
| **M4** authenticate 等值比较 | MEDIUM | admin-service/service.rs | 0.3 | 56.x |
| **M5** RBAC scope 边界 | MEDIUM | shared-platform/rbac.rs | 0.2 | 55.x |
| **M6** dev 密码共享 | MEDIUM | .env（dev only），文档 | 0.5 | 56.x（生产前） |
| **M7** compute_hash 分隔符 | MEDIUM | admin-service/entity.rs（C1 修复时一并） | 0.0（与 C1 合并） | 55.x |
| **L1** 注释不一致 | LOW | admin-service/entity.rs（C1 修复时一并） | 0.0 | 55.x |
| **L2** outbox migration 模板 | LOW | 5 域 migrations | 0.5 | 56.x |
| **L3** RBAC 通配符粒度 | LOW | shared-platform/rbac.rs | 0.3 | backlog |
| **L4** use_tls bool 标志 | LOW | shared-platform/client.rs | 0.3 | 55.x |

**总估时**：8.1 人·天（含测试与文档更新）。

**Token 等价（per RGS-TS-001 v0.4 §6.2 草案，1 人·天 ≈ 100K-300K tokens）**：约 0.8M-2.4M tokens。

---

## 11. 审计员签注

<审计员>：security-saga-adversarial-001
<签名>：⟪adversarial-54/security-saga-001@2026-08-22⟫

**审核范围独立判断**：
- 本审计仅基于 commit `e320c69` 静态阅读源码 + git 历史，**未执行动态 fuzzing / 渗透测试**。
- 6 域 repository.rs 的 SQL 注入风险通过 `grep "format!\(.*SELECT|FROM|WHERE"` 全仓 0 匹配 + 逐个 `sqlx::query().bind()` 模式确认 = **0 注入点**。`cluster-ops/repository.rs:302` 的 `format!("{}|{}", key, scope_value)` 仅用于构造应用层 key（参数化绑定），不拼接到 SQL。
- mTLS 实化（54.9）当前为"演示结构"，**生产部署前必须实化 `ServerTlsConfig::client_auth_required(true)`**（H4）。
- `.env` 文件已被 `.gitignore` 屏蔽（`git check-ignore -v .env` → `.gitignore:7:.env` 验证）—— 密钥管理整体合格，仅 dev 密码共享需治理（M6）。
- 密码学侧 **C1（FNV-1a） 是唯一 CRITICAL**，必须在 55.x 修复并补 SHA-256 collision resistance 测试。
- Saga 状态机 + Outbox 抢占 + audit_log 竞争是 **HIGH 三大并发陷阱**，建议合并到 55.x 单一 saga-security hardening milestone。

**未验证项**（环境受限）：
- 各域 migrations 实际在 PG 18.6 的执行结果（仅审计了 SQL 文件，未跑 migration）。
- NFR-OP-010（2 SRE ≤ 20 人·天/周）下 8.1 人·天的修复量会触发资源冲突（per RGS-PM-008），需与 SRE Lead 协调——**审计员不裁决资源分配**。
- 生产 k8s Secret 注入路径（per RGS-SEC-100 §7）当前依赖 .env，**非理想**，但属于运维层而非代码层。
- audit_log hash chain **当前没有任何验证函数**（grep `verify_hash_chain` 全仓 0 匹配）——即使升级到 SHA-256，没有独立 verifier 也只是"自我声明的完整性"。

**审计员立场声明**：本报告所有发现均经源码 grep / read 验证（行号已标注），非"听上去有风险"的推测。但**未在动态环境复现**，建议 security-saga 专项 follow-up 加 fuzzing / property-based testing（如 `proptest` + `arbitrary` for Saga status transition）作为 55.x 准入门槛。
