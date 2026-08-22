# verify-A 工程 55 P0+收尾 code review 交叉审核

**审核对象**：git log 7deff16^..5ace5ad (13 commits: 11 L4 任务 + 2 merge)
**审核子代理**：verify-A code-review-adversarial
**审核时间**：2026-08-22
**commit 基线**：5ace5ad
**审核范围**：55.12 / 55.13 / 55.14 / 55.15 / 55.16 / 55.17 / 55.18 / 55.20 / 55.21+22 / 55.23 / 55.24

> 实际范围说明：任务书标注 8c1dbfd..5ace5ad 12 commits，但 8c1dbfd 是 55.18，不是范围起点。实际工程 55 任务起点是 7deff16 (55.15)，终点 5ace5ad，共 13 commits (11 L4 + 2 merge)。本报告以 13 commits 为准。

---

## 1. 严重度统计

| 严重度 | 数量 | 标识 |
|--------|------|------|
| CRITICAL | 4 | AC-1, AC-2, AC-3, AC-4 |
| HIGH | 7 | AH-1 ~ AH-7 |
| MEDIUM | 11 | AM-1 ~ AM-11 |
| LOW | 8 | AL-1 ~ AL-8 |

---

## 2. CRITICAL Issues

### AC-1. 55.21+22 mTLS / outbox relay 静默降级 - fail-closed 全部失效

- 位置：`crates/{admin,cluster-ops,economy,match,player,social}-service/src/main.rs` 6 个文件
- 类别：业务逻辑 / 安全 / 错误处理
- 问题：

55.18 故意把 `RpcChannelConfig::default().require_tls = true` 设为 fail-closed（见 `crates/shared-platform/src/channel.rs:71-73`）。但 55.21 在 6 域 `main.rs` 都写了同一段 dev/test fallback。

5 域 + cluster-ops × 2 fallback = 6 个 CRITICAL 静默降级路径。

- 影响：
1. **生产风险**：k8s Secret 挂载失败、cert-manager 配错、PEM 路径拼错 → 服务在生产**静默运行 insecure gRPC**。55.18 的 fail-closed 防线被 main.rs 整个绕开。**违反 CH4 + DEC-015 P1 审计建议**。
2. **数据丢失**：NATS 故障时 outbox 行持续累积（无 alert 集成、无 retry 周期、仅 warn log），需要人工 recovery。**违反 CH1 事务性消息 at-least-once 承诺**。
3. **观察性差**：仅靠 `tracing::warn!` 而非 `tracing::error!` + metric（如 `mTLS_bypassed_total` 已在 55.18 设计但本 fallback 不调它），sre 团队无法监控。

- 修复建议：
1. 删除 fallback，**强制 fail-closed**：cert 加载失败 → `std::process::exit(1)`（与 55.15 改 DB pool 失败同模式）。NATS 失败也 exit(1) 或退避重试 + metric。
2. 若必须保留 dev fallback，强制读取 `RGS_ALLOW_INSECURE=true` env（编译时常量不可），并在生产 deployment 显式不注入。
3. 把 `mTLS_bypassed_total` 在 5 域 main.rs 静默降级时也调 +1（与 55.18 `build_insecure_channel` 同语义），并通过 `/metrics` 暴露。

---

### AC-2. 55.13 PgAuditLogRepository::append_atomic 丢弃 SELECT 结果 - hash 链完整性防御失效

- 位置：`crates/admin-service/src/repository.rs:224-256`
- 类别：业务逻辑 / 静默错误
- 问题：

```rust
// crates/admin-service/src/repository.rs:224-257 (55.13)
async fn append_atomic(
    &self,
    tx: &mut Transaction<'_, Postgres>,
    entry: &AuditLogEntry,
) -> Result<AuditLogEntry> {
    let latest_row = sqlx::query(
        "SELECT id, actor_id, action, target, payload, prev_hash, hash, created_at \
         FROM audit_log ORDER BY created_at DESC LIMIT 1 FOR UPDATE",
    ).fetch_optional(&mut **tx).await?;
    // 调用方负责校验 entry.prev_hash 与 latest_row.hash 一致
    let _ = latest_row; // 锁生效即满足；调用方已知 prev_hash。
    sqlx::query("INSERT INTO audit_log ...").execute(&mut **tx).await?;
    Ok(entry.clone())
}
```

`let _ = latest_row;` **完全丢弃 SELECT 结果**，没有任何校验。

- 影响：
1. 任何调用方（除 service.rs:142-170 路径外）传一个 prev_hash 错误的 `entry` 进来，**INSERT 仍然成功**。hash 链会**分叉**，但代码不报警。
2. `service.rs:147-163` 的 PG 路径已经自己先 SELECT 取 prev_hash 再 INSERT — 等于 1 个事务 2 次 SELECT，浪费 I/O。
3. 注释 调用方已知 prev_hash 把校验责任推给所有调用方 — 任何新 caller 引入就可能引入 hash 链完整性漏洞。
4. 触发器 `audit_log_no_update` 只防 UPDATE/DELETE，**不防 INSERT 错 prev_hash**。

- 修复建议：
1. **把 SELECT 留在 service.rs**，删除 PgAuditLogRepository.append_atomic 的 SELECT（仅做 INSERT）。
2. **或**：append_atomic 内部做 `assert_eq!(entry.prev_hash, latest_row.hash)`，错就 panic / 返回 Err。
3. 加测试：传 wrong prev_hash 必须返 Err。

---

### AC-3. 55.13 service.rs audit_log 重复 prev_hash 计算逻辑 - PG / InMemory 路径分叉

- 位置：`crates/admin-service/src/service.rs:140-170`
- 类别：业务逻辑 / API 设计
- 问题：service.rs 的 audit_log 方法在 PG 路径和 InMemory 路径都做 取 latest → 算 prev_hash → 构造 entry，逻辑完全一致。

- 影响：
1. PG 路径下 `append_atomic` 又做了一次 SELECT（见 AC-2），2 次 SELECT 1 次 INSERT。
2. InMemory 路径调 `append`（旧 trait 方法），PG 路径调 `append_atomic`（新方法），接口分叉。
3. 两条路径若以后 bug 修复只修一边，分叉更严重。

- 修复建议：抽出 `AuditLogService` wrapper 统一管 SELECT，PG / InMemory trait 只暴露底层 INSERT。

---

### AC-4. 55.12 ReserveHandler 失败时静默吞掉 reservation 清理错误 - dangling reservation 永久存在

- 位置：`crates/economy-service/src/saga_orchestrator.rs:240`
- 类别：错误处理 / 资源管理
- 问题：

```rust
// saga_orchestrator.rs:238-246 (55.12)
if !account.try_debit(self.amount) {
    // 清理 dangling reservation
    let _ = self.reservations.delete_by_id(r.id).await;
    return Err(Error::InsufficientFunds { ... });
}
```

`let _ = ...` 完全吞掉 `delete_by_id` 的 Result。

- 影响：
1. 测试 `reserve_handler_rejects_insufficient_funds` 验证 `reservations.len() == 0`，但生产 PG 实现下若 delete 因 FK / 锁 / 网络抖动失败，reservation 行**永久留存**。
2. reservation 表上无 TTL 清理任务（grep `WHERE status=...` 无 cron 类 job），孤儿累积**无上限**。
3. 同一个 Saga 重试时 `list_by_saga` 仍能找到孤儿 reservation，导致 `compensate()` 找错对象。

- 修复建议：
1. 至少 `tracing::warn!` 留审计日志。
2. 加 retry 1-2 次（指数退避）。
3. 加 migration 周期 job（先 mark 后 delete）。
4. 测试覆盖 PG 路径：sqlx mock 验证 delete 失败不阻塞 Err 返回。

---

## 3. HIGH Issues

### AH-1. 55.12 SagaOrchestrator::execute 关键路径用 `current_mut().unwrap()` 三处

- 位置: crates/economy-service/src/saga_orchestrator.rs:95, 100, 110
- 类别: 错误处理 / panic 风险
- 问题: saga.current() 在第 84 行已验证返回 Some, 但 current_mut() 在第 95/100/110 仍 unwrap().
- 影响:
1. 任何 current() / current_mut() 的内部状态被外部代码改后(如并发 resume), unwrap panic, 整个 orchestrator task 死掉, saga 永远卡在 Running 状态.
2. 多副本崩溃恢复(55.23 saga_resume_loop 调 orch.resume)若两个 resume 同时跑同一 saga, 竞争导致 current_mut() 错位 -> panic.
- 修复建议: `let cur = saga.current_mut().ok_or_else(|| Error::Validation(...))?;` 把 Option 显式传 Err.

---

### AH-2. 55.17 mark_sent / mark_failed 不校验 status - 可能覆写 re-claim 行

- 位置: crates/shared-platform/src/outbox.rs:315-348
- 类别: 业务逻辑 / 并发
- 问题: mark_sent 直接 UPDATE, 没有 `AND status='in_flight' AND lease_until > NOW()` 保护.
- 影响: relay R1 取出 entry 后 publish 期间 lease 过期 -> R2 re-claim -> R1 publish 完成调 mark_sent -> R2 仍在 publish 同一行; 最终重复消费 + retry_count 错误递增.
- 修复建议: mark_sent 加 `AND status='in_flight'`; 或加 `lease_version` int 字段做 CAS.

---

### AH-3. 55.17 outbox migrations 缺 CHECK 约束 - 与 code template 不一致

- 位置: crates/{admin,cluster-ops,match,player,social}-service/migrations/0002_outbox.sql, crates/economy-service/migrations/0003_outbox.sql
- 类别: 业务逻辑 / 防御
- 问题: 6 域 SQL migration 的 `status VARCHAR(16) NOT NULL DEFAULT 'pending'` 完全没有 `CHECK (status IN ('pending','in_flight','sent','failed'))`. 而 `crates/shared-platform/src/outbox.rs:471-484` 的 `MIGRATION_TEMPLATE` 常量**有** CHECK 约束.
- 影响: 6 域实际跑的是无 CHECK 版本, 未来若有人手 UPDATE 把 status 设成 PROCESSING(笔误), DB 不会拦截, relay 永远找不到该行 -> 静默丢消息.
- 修复建议: 6 域补 `0003_outbox_status_check.sql`:
```sql
ALTER TABLE outbox ADD CONSTRAINT chk_outbox_status
    CHECK (status IN ('pending','in_flight','sent','failed'));
```

---

### AH-4. 55.18 mTLS bypass 计数器是 `static AtomicU64` - 多副本无法聚合

- 位置: crates/shared-platform/src/channel.rs:82-87
- 类别: API / 观察性
- 问题: 进程内 counter, 多副本部署每副本独立计数.
- 影响: RGS-REV-007 CH4 要求把 mTLS_bypassed_total 作为监控项 - 进程内 counter 不符合生产监控规范.
- 修复建议: 与 tracing_opentelemetry 集成直接 emit OTel counter; 或改用 `metrics::counter!`.

---

### AH-5. 55.13 audit_log service.rs 事务的读+写顺序 - 同事务内 SELECT FOR UPDATE 触发器行为差异

- 位置: crates/admin-service/src/service.rs:150-160 + crates/admin-service/migrations/0001_init.sql:42-50
- 类别: 业务逻辑 / 集成风险
- 问题: service.rs 在事务内 SELECT ... FOR UPDATE 锁 audit_log latest 行 + INSERT 新行. 触发器只禁 UPDATE/DELETE, 不触发 SELECT FOR UPDATE. 但 outbox 表无 append-only 触发器, 行为不一致.
- 影响: 当前不构成代码 bug, 但应加 ADR 文档说明 audit_log 是 append-only hash chain; outbox 是 status-machine 可 UPDATE.
- 修复建议: 加 ADR + pg_test 验证 SELECT FOR UPDATE 不触发 audit_log_no_modify 触发器.

---

### AH-6. 55.23 Saga 恢复后台任务无界循环 + 无健康检查 + 无 graceful shutdown

- 位置: crates/economy-service/src/main.rs:104-136
- 类别: 资源管理 / 进程生命周期
- 问题: 无限 loop 永远不退出; 没有 tokio::select! 监听 SIGTERM / CancellationToken; DB 连接池耗尽时仍持续重试.
- 影响:
1. **k8s graceful shutdown timeout 默认 30s**, saga recover 任务不响应 SIGTERM -> 主进程退出被 SIGKILL, spawn 任务未完成当前 resume() 调用 -> 资源泄漏.
2. DB 长时间不可用时, list_running 持续返 Err 写 warn log(每 30s 一行) -> 日志风暴.
- 修复建议: 用 tokio::select! 监听 SIGTERM + 加指数退避 + AtomicBool 健康检查.

---

### AH-7. 55.18 client.rs `build_secure_channel` 默认占位证书路径在生产 100% 失败 - builder API 不强制注入

- 位置: crates/shared-platform/src/client.rs:124-132 (default_client_tls_input)
- 类别: API 设计
- 问题: build_secure_channel(service, host) 默认用 /etc/rgs/certs/... 占位路径 - 若 caller 忘记调 build_secure_channel_with_tls 注入真实路径, **生产必失败(FileNotFound)**.
- 影响: API 错误诱导; 与 55.21 main.rs 静默降级组合(双向失守).
- 修复建议: 删除 default_client_tls_input 和 build_secure_channel(service, host), **只留 build_secure_channel_with_tls**.

---

## 4. MEDIUM Issues

### AM-1. 55.17 MIGRATION_TEMPLATE 与实际 6 域 migration 状态机描述不一致

- 位置: crates/shared-platform/src/outbox.rs:464-489 (template) vs 6 域 .sql 文件
- 类别: 文档 / API
- 问题: template 含 CHECK (status IN ...), 6 域 migration 没有. 详见 AH-3.
- 修复建议: 统一 template, 或 6 域补 migration.

---

### AM-2. 55.17 InMemory outbox 性能 - list_pending 全表扫描

- 位置: crates/shared-platform/src/outbox.rs:402-435
- 类别: 性能 (仅测试路径)
- 问题: guard.values().filter(...).collect() + sort_by_key + truncate 三步 O(n log n). 生产路径不会用 InMemory, 仅测试.
- 修复建议: 文档加注释仅供测试.

---

### AM-3. 55.17 mark_giveup 不带 retry_count 上限校验

- 位置: crates/shared-platform/src/outbox.rs:340-348
- 类别: API 设计
- 问题: mark_giveup 直接 UPDATE, 不检查 retry_count >= max_retries. relay 在 outbox_relay.rs:80-81 检查, 但这是应用层检查, DB 层无防御.
- 修复建议: DB 加 CHECK (retry_count <= 1000).

---

### AM-4. 55.16 current_trace_ids OTel API 假设 - SpanId 8 字节零填充

- 位置: crates/shared-platform/src/grpc_tracing.rs:75-95
- 类别: API / 互操作
- 问题: OTel SpanId 永远 8 字节, UUID 16 字节, 这是技术债. 未来如果上游服务用 UUID SpanId(不常见但可能)会错位.
- 修复建议: traceparent header 改用 00-{32 hex trace_id}-{16 hex span_id}-01.

---

### AM-5. 55.13 with_pool 注入 PgPool 但 AdminService trait 不暴露

- 位置: crates/admin-service/src/service.rs:62-70
- 类别: API 设计
- 问题: AdminServiceImpl 内部用 pool: Option<PgPool> 字段, trait AdminService 不知道 pool 存在. 生产必须 main.rs 调 with_pool 才生效.
- 修复建议: 强制构造时传 pool; 或 trait 加 fn pool().

---

### AM-6. 55.12 Saga serialize_steps 静默丢弃序列化错误

- 位置: crates/economy-service/src/saga.rs:271-277
- 类别: 错误处理
- 问题: serde_json::to_value(steps).unwrap_or(serde_json::json!([])); from_value(value).unwrap_or_default() 静默吞错.
- 修复建议: 返 Result<Value>, 让 caller 上抛.

---

### AM-7. 55.16 OTel subscriber 桥接假设 - 测试无 OTel 时的 fallback

- 位置: crates/shared-platform/src/grpc_tracing.rs:88-95
- 类别: 观察性
- 问题: sc.is_valid() 仅在 OTel subscriber 启用时为 true. 生产若忘 init, 所有 client->server 追踪都断.
- 修复建议: 启动时检查 OTel subscriber; 未注册则 exit(1).

---

### AM-8. 55.14 RBAC 多角色顺序敏感 - SuperAdmin 在 DomainAdmin 之前可绕过 scope

- 位置: crates/shared-platform/src/rbac.rs:165-211
- 类别: 业务逻辑
- 问题: 若 subject.roles = vec![Role::SuperAdmin, Role::DomainAdmin], 第一轮 SuperAdmin 命中 *:* -> Allow, 绕开 DomainAdmin scope 检查.
- 修复建议: scope 检查放最后(任何 role 都要 scope 通过).

---

### AM-9. 55.20 generate_dev_passwords.ps1 ACL 设置 try/catch 静默失败

- 位置: scripts/generate_dev_passwords.ps1:98-107
- 类别: 错误处理
- 问题: try/catch 仅 Write-Warning, dev 环境 .env 权限失败时仅 warning, 密码以明文保留, ACL 是 644(其他用户可读).
- 修复建议: 失败时 Write-Error + exit 1.

---

### AM-10. 55.15 player-service main.rs 之前用 warn 路径, 55.15 改为 exit(1) - 改动未在 commit message 说明

- 位置: crates/player-service/src/main.rs:34-58 (7deff16 diff)
- 类别: 沟通
- 问题: commit 7deff16 把 player-service 改为 fail-fast, 但 commit message 仅说切到 PgRepository. 行为变更(warn->exit)没在 message 单独标注.
- 修复建议: commit message 显式列 breaking 行为变更.

---

### AM-11. 55.18 build_insecure_channel 在 client.rs 暴露 - 但 main.rs 没用

- 位置: crates/shared-platform/src/client.rs:96-122
- 类别: API 设计
- 问题: build_insecure_channel 是公共 API, 可被 5 域 main.rs 调用 - 但 5 域都走 load_server_tls_config(服务端 API), 不走 client builder.
- 修复建议: 把 build_insecure_channel 标 cfg(test) 或 cfg(feature=test-util).

---

## 5. LOW Issues

| ID | 描述 | 位置 |
|----|------|------|
| AL-1 | default_client_tls_input 占位路径 /etc/rgs/certs/... 硬编码 | shared-platform/src/client.rs:128-130 |
| AL-2 | json_logging doctest 用 unwrap() - 文档不应示范 panic | shared-platform/src/json_logging.rs:13 |
| AL-3 | rbac.rs rgs-testkit/src/mock.rs 文档注释 indent 修复 1 行 | rgs-testkit/src/mock.rs:14 |
| AL-4 | SagaStatus 序列化(snake_case)但 PgRepository 持久化时用 saga_status_to_str 自定义映射 - 双重真理源 | economy-service/src/saga.rs:295-303 |
| AL-5 | mTLS_bypassed_total 测试用 before/after 模式 - 测试运行顺序影响 counter 值 | shared-platform/src/channel.rs:175-192 |
| AL-6 | OutboxStatus::as_str() 与 parse_status() 重复字符串表 | shared-platform/src/outbox.rs:67-74, 207-213 |
| AL-7 | OutboxRelay 改泛型后 outbox_relay_tick_empty 测试只验证空 list, 没真正测 relay | shared-platform/src/outbox_relay.rs:174-184 |
| AL-8 | let _nats_keepalive = nats_client; 注释 需 owner 存在以维持连接 与 async_nats::Client 实际 API 行为是否一致未验证 | 6 域 main.rs spawn block |

---

## 6. 修复优先级矩阵

| Issue | 严重度 | 文件 | 估时(人·时) | 阻塞阶段 |
|-------|--------|------|---------------|----------|
| AC-1 | CRITICAL | 6 域 main.rs | 4-6 | **部署前必须修复** |
| AC-2 | CRITICAL | admin/repository.rs | 2-3 | 部署前必须修复 |
| AC-3 | CRITICAL | admin/service.rs | 2 | 部署前必须修复 |
| AC-4 | CRITICAL | economy/saga_orchestrator.rs:240 | 1-2 | 部署前必须修复 |
| AH-1 | HIGH | economy/saga_orchestrator.rs:95/100/110 | 1 | 部署前应修复 |
| AH-2 | HIGH | shared-platform/outbox.rs:315-348 | 2-3 | 部署前应修复 |
| AH-3 | HIGH | 6 域 migrations | 1 | 部署前应修复 |
| AH-4 | HIGH | shared-platform/channel.rs:82-87 | 2-3 | 部署前应修复 |
| AH-5 | HIGH | docs ADR | 1 | 可后续 |
| AH-6 | HIGH | economy/main.rs:104-136 | 2-3 | 部署前应修复 |
| AH-7 | HIGH | shared-platform/client.rs:124-132 | 1 | 部署前应修复 |
| AM-1 ~ AM-11 | MEDIUM | - | 各 0.5-2 | 1 周内 |
| AL-1 ~ AL-8 | LOW | - | 各 0.25-0.5 | 2 周内 |

**总估时**: CRITICAL+HIGH 修复约 20-30 人·时(含测试 + 文档 + 6 域同步更新).

---

## 7. 关键交叉发现(跨 commit)

### 7.1 fail-closed 防线在 main.rs 端全失守(AC-1 + AH-7 + AM-11 联合)

工程 55 的 fail-closed 防线由 3 层组成:
1. **L1**: 55.18 `RpcChannelConfig::default().require_tls = true`(OK)
2. **L2**: 55.18 `build_insecure_channel` 显式 + 计数(OK)
3. **L3**: 55.21 5 域 main.rs **强制 mTLS 加载 + 失败 exit**(**缺失**, 用 warn + 降级代替)

55.18 单独看是合规的, 55.21 单独看也写了 mTLS 加载 逻辑, 但**两层接合点**(main.rs 失败时怎么办)选择了 静默降级 而非 fail-closed. **这是 55.x 最大的安全漏洞**.

**对应**: RGS-REV-007 §3.5 CH4 fail-closed 建议未在 55.21 落实.

### 7.2 测试覆盖率与代码复杂度不匹配(AH-1 + AC-2 + AC-4 联合)

55.12 SagaOrchestrator 加 7 个测试, 但**生产代码用 `unwrap()` 的 3 处关键路径无对应测试**(saga.current_mut() 边界场景).
55.13 audit_log 加 1 个并发测试(20 个并发 audit_log), 但**PG 路径下 prev_hash 错误传参的 negative test 没有**.
55.17 outbox 加 4 个测试, 但**PG 路径下多副本竞争 + lease 过期 + 双 relay 标 sent** 的 negative test 没有.

**结论**: 单测覆盖 happy path, 缺少 boundary / negative / concurrency tests.

### 7.3 fallback to insecure 模式被复制 6 次(AC-1 + AM-10 联合)

5 域 + cluster-ops main.rs 复制 6 份相同的 `let tls_config = match load_server_tls_config(...) { Ok => Some, Err => None }` 模式, 加 6 份相同的 `let nats_uri = env::var... match build_messaging_client` 模式. **这是 refactor 机会**: 提取 `init_secure_grpc_server` + `init_outbox_relay` helper 到 shared-platform, 避免 6 份代码各 fix 各的.

---

## 8. 审计员签注

<审计员>: verify-A
<签名>: <占位 - 等待用户正式签字>

**审核范围声明**: 本报告仅基于 commit 5ace5ad 仓库快照 + git diff 7deff16^..5ace5ad 静态阅读源码 + grep 关键模式 (unwrap / expect / panic / TODO / FIXME / unsafe / #![allow]).
**审核方法**:
1. `git log --stat` 看 13 commits 文件清单
2. `git show` 完整 diff 11 个 L4 任务
3. read 关键文件: saga_orchestrator.rs / outbox.rs / outbox_relay.rs / channel.rs / tls.rs / client.rs / rbac.rs / entity.rs / service.rs (admin) / main.rs x 6
4. grep 验证代码风格 (unwrap/expect/panic/unsafe/allow)
**未验证项**:
- 未跑 `cargo test` (CI 验)
- 未跑 `cargo clippy`
- 未跑 integration test 连真 PG / NATS
- 未追踪 54.x 历史 bug 与 55.x 修复的对应关系(仅靠 commit message 自我报告)
- 未验证 `rgs-certgen` 与 55.18 mTLS config 的实际证书生成兼容性

**核心理由**: 任务书标注 git log 8c1dbfd..5ace5ad 12 commits, 但 8c1dbfd 是 55.18(不是 55 范围起点), 实际范围应为 7deff16^..5ace5ad(13 commits: 11 L4 + 2 merge). **已在报告第 1 节顶部说明**.

---

**报告结束** - 请用户根据优先级矩阵决定修复顺序.
