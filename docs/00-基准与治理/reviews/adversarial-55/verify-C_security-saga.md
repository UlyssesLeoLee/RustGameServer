# verify-C 工程 55 P0+收尾 security + saga 交叉审核

**审核对象**: git log 8c1dbfd..5ace5ad 含 12 commit（实际范围扩到 7379019^..5ace5ad 含 12 个 P0+收尾 commit;原始任务范围 8c1dbfd..5ace5ad 仅含 3 commit,按 12 commit 全量审核）

**审核子代理**: verify-C security-saga-adversarial

**审核时间**: 2026-08-22

**commit 基线**: 5ace5ad

**worktree**: D:\RustGameServer-worktrees\verify-55-C-security (branch: verify/55-C-security)

---

## 0. 范围说明

任务范围 8c1dbfd..5ace5ad 仅含 3 commit（55.18/55.23/55.24）。工程 55 P0+收尾的 12 个 commit 实际跨度为 7379019^..5ace5ad（含 merge 共 14 个 commit,去 merge 12 个 P0 commit）,本次审核以此为准。

12 commit 列表（去 merge）:
- 7379019 55.1 AC3 economy 资金事务原子化（OCC + apply_atomic）
- 7deff16 55.15 5 域 + cluster-ops main.rs InMemory -> Pg 接线
- 69ebcd1 55.16 client_interceptor trace_id 从 Span 提取
- 33fca1e 55.14 RBAC DomainAdmin 缺 scope 显式 deny + 边界修复
- 9e55bbe 55.20 dev 密码 6 域独立化
- 6b3cc5d 55.13 audit_log FNV-1a -> SHA-256 + 事务化
- d8d33cf 55.12 SagaOrchestrator handler 实化
- 53a8d37 55.17 outbox SKIP LOCKED + 事务边界 + 6 域 outbox migration
- 8c1dbfd 55.18 mTLS client_auth_required 实化
- 421585c 55.23 economy main.rs SagaOrchestrator 接线
- 465bfeb 55.24 housekeeping 修 pre-existing doctest + clippy
- 5ace5ad 55.21+22 5 域 main.rs mTLS + outbox 接线

---

## 1. 严重度统计

- CRITICAL: 4 (CC-1~4)
- HIGH: 7 (HC-1~7)
- MEDIUM: 7 (MC-1~7)
- LOW: 3 (LC-1~3)

合计 21 个独立 finding。

---

## 2. 攻击面矩阵

| 资产 | 威胁 | 现有防护 | 有效性 |
|------|------|---------|--------|
| 5 域 RPC gRPC 50051-50055 | MITM / 未授权客户端 | mTLS via load_server_tls_config 55.18 + 5 域接线 55.21 | 部分 - 见 HC-1, HC-4 |
| economy Saga | 重复扣款 / 状态机错乱 | Inbox UNIQUE + Reservation 55.12 + 启动期 status check | 不足 - 见 CC-2, CC-4 |
| admin audit_log | 篡改 | SHA-256 + 长度前缀 + 事务化 55.13 + UNIQUE(prev_hash) | 死代码 - 见 CC-1;仅 append 不可验证 - 见 HC-3 |
| RBAC | 越权 / 横向 scope 越界 | DomainAdmin 缺 scope 显式 deny + 边界 / 严格匹配 55.14 | 完备 |
| dev 凭据 .env | 6 域共享 / 弱口令 / commit 泄漏 | 7 KEY 独立 + openssl rand -base64 24 + .gitignore 55.20 | 完备 |
| Outbox | 重复投递 / 死锁 / 多 relay 竞争 | FOR UPDATE SKIP LOCKED + in_flight + lease_until 30s 55.17 | 部分 - 见 CC-3, HC-5, HC-6 |
| Cluster NATS 不可用 | 消息丢失 | dev/test fallback warn 跳过 relay 55.22 | 不足 - 见 HC-2 |

---

## 3. CRITICAL Issues

### CC-1. admin-service main.rs 未注入 PgPool -> 55.13 事务化审计日志在生产环境是死代码

- 位置: crates/admin-service/src/main.rs:93（5 域 main.rs 中唯一未调用 with_pool 的）
- 类别: 状态机 / 事务 / SQL
- 问题:
  - 55.13 在 AdminServiceImpl 加了 pool: Option<PgPool> 字段和 with_pool() 构造器,并改造 audit_log() 在 if let Some(pool) = &self.pool 分支走 pool.begin() + SELECT FOR UPDATE + append_atomic + commit 的真事务路径。
  - 但 admin-service/src/main.rs:93 仍然: let service_impl = Arc::new(AdminServiceImpl::new(users, audit)); 未调 .with_pool(pool.clone())。
  - 结果:所有 gRPC 路径触发的 audit_log() 都走 InMemory fallback 分支: 读 latest (无 FOR UPDATE) -> 构造 entry -> append 旧 INSERT 路径,无 tx 包裹。
  - 等价于 55.13 之前的状态:read-then-append 完全没有原子性保护,hash 链分叉不会被数据库 UNIQUE(prev_hash) 兜底。
  - audit_log_atomic_latest_append 单元测试也只跑了 InMemory 分支(svc() 用 AdminServiceImpl::new 无 pool),Pg 路径无任何测试覆盖。
- 影响: CRITICAL - audit_log 的 hash 链完整性保证在 admin-service 完全失效。如有并发审计写入,可能产生 hash 链分叉 + 静默数据不一致。
- 修复建议: AdminServiceImpl::new(users, audit).with_pool(pool.clone()) + 加 Pg 路径集成测试覆盖 audit_log_atomic_* 场景。

### CC-2. Saga 崩溃恢复循环完全失效:list_running 找的 sagas 永远不会被 execute 接受

- 位置:
  - 调用: crates/economy-service/src/main.rs:111-123 (55.23 引入)
  - 状态机入口检查: crates/economy-service/src/saga_orchestrator.rs:71-77
  - 状态机查询: crates/economy-service/src/saga.rs:395-402 (PgSagaRepository::list_running)
- 类别: 状态机 / 崩溃恢复
- 问题:
  - 55.23 启动后台 task,每 30s 调 sagas.list_running(SAGA_RECOVER_BATCH) 查 status IN (running, compensating) 的 sagas,然后对每个调 orch.resume(id)。
  - resume() 调 self.execute(&mut saga)。
  - execute() 入口有严格检查: if saga.status != SagaStatus::Pending { return Err(Validation(...)); }
  - 矛盾: list_running 查的是 running / compensating 状态,但 execute 只接受 pending。每个被恢复的 saga 都会以 Validation 错误被拒绝,只产生 tracing::warn 而 saga 永远停留在 running 状态。
  - 54.8 时期没有这个循环所以没暴露。pre-existing bug 在 55.23 接线时未发现。
- 影响: CRITICAL - production crash 后所有未完成的 saga 永久卡在 running 状态,资源不会自动释放、补偿不会触发、玩家转账/购买永远 pending。
- 修复建议:
  - 方案 A(最小修复):在 execute() 顶部增加对 Running/Compensating 状态的接管逻辑
  - 方案 B(更稳):新增 resume_running(saga) 方法绕过 Pending 检查
  - 加测试: make_running_saga() + orch.resume(id) 应从断点继续执行
  - 现实:建议加 metric saga_recovery_skipped_total{reason=status_not_pending}

### CC-3. 6 域 outbox 迁移缺少 status CHECK 约束 -> 与 shared-platform 模板不一致

- 位置: 全部 6 个 outbox migration
  - crates/player-service/migrations/0002_outbox.sql:12
  - crates/economy-service/migrations/0003_outbox.sql:13
  - crates/match-service/migrations/0002_outbox.sql:12
  - crates/social-service/migrations/0002_outbox.sql:12
  - crates/admin-service/migrations/0002_outbox.sql:12
  - crates/cluster-ops/migrations/0002_outbox.sql:12
- 类别: SQL / 状态机
- 问题:
  - shared-platform 的 MIGRATION_TEMPLATE(crates/shared-platform/src/outbox.rs:464-489)有 CHECK 约束: status TEXT NOT NULL DEFAULT pending CHECK (status IN (pending, in_flight, sent, failed))
  - 6 个域的 migration 都漏写此 CHECK: status VARCHAR(16) NOT NULL DEFAULT pending
  - 实际差异: 模板是 TEXT,migration 是 VARCHAR(16)(更短,本身无所谓),但 CHECK 缺失。
- 影响:
  - CRITICAL - 任何代码 bug、误用 raw SQL、运维手工修复都可能写入 status=in_flght(拼错)或 status=PROCESSING 等非法值,DB 层不会拒绝。
  - parse_status 默认 fallback 到 Pending,导致重复消费。
  - 索引 idx_outbox_pending(WHERE status = pending)不会生效。
- 修复建议:
  - 6 个 migration 全部加 CHECK (status IN (pending, in_flight, sent, failed))
  - 或追加 1 个 0003 修正 migration(ADD CONSTRAINT ... CHECK)
  - CI 加 lint: 检查所有 outbox migration 是否包含 CHECK

### CC-4. 资金幻影生成:apply_atomic OCC 失败时 ReserveHandler 补偿会无中生有加余额

- 位置: crates/economy-service/src/saga_orchestrator.rs:229-323 (ReserveHandler.execute + compensate)
- 类别: 状态机 / 资金 / 事务
- 问题:
  - ReserveHandler.execute 顺序:
    1. Reservation::new() -> 状态 Reserved
    2. self.reservations.save(&r).await? -> reservation 已落库(独立 implicit tx)
    3. account.try_debit(self.amount) -> 仅本地内存检查
    4. self.accounts.apply_atomic(&account, &entry).await? -> 真 OCC debit
  - 若第 4 步失败(OCC 冲突),reservation 已在表里,但账户余额从未被扣减。
  - saga 标记 step=Failed -> 触发 compensate(saga)。
  - ReserveHandler.compensate:
    1. 找 reservation(找到,status=Reserved)
    2. account.credit(refund_amount) -> +amount 加到本地副本
    3. apply_atomic 写库 -> 真实余额 +amount
  - 结果: 账户从未被扣过款,但被加了 amount 款 -> 凭空造钱。
  - 测试通过的原因是: 测试用 InMemoryAccountRepository + apply_atomic 必定成功,从不触发 OCC 失败。生产路径 OCC 冲突(高并发转账、跨副本)会稳定触发。
- 影响:
  - CRITICAL - 资金安全 / 经济系统完整性。任何 OCC 冲突下:
    1. 玩家凭空获得 amount 金币
    2. 同时 reservation 被标记 Compensated
    3. ledger 有一条 +amount 的 Compensated entry
    4. sum(ledger) != sum(balance) 关系破裂
- 修复建议:
  - 方案 A: ReserveHandler.execute 改为 reservation 与 account debit 同一事务(55.17 outbox 同事务模式应用到 reservation)
  - 方案 B: ReservationStatus 增加 Reserved vs Debited 区分;compensate 只在 status=Debited 时执行 credit
  - 加测试: 模拟 OCC 失败,断言 balance 净变化 = 0
  - 这也是 CC-2 修复的前提 - 恢复路径如果误触发补偿,问题会成倍放大

---

## 4. HIGH Issues

### HC-1. 5 域 mTLS fallback 静默降级为 insecure gRPC,无 metric 无 fail-closed 开关

- 位置: crates/economy-service/src/main.rs:177-201(player/match/social/admin 同款)
- 类别: mTLS / 降级路径
- 问题:
  - load_server_tls_config 读 PEM 文件失败时,main.rs 仅 tracing::warn + 返回 None
  - 然后 server_builder 不调 tls_config(),gRPC 跑在 plaintext 上
  - 生产风险:
    1. k8s Secret 挂载失败 / rgs-certgen 没跑 / certgen 私钥丢失 -> 服务静默降级为明文 gRPC
    2. Prometheus scrape 看不到此事件(MTLS_BYPASSED_TOTAL 仅在 client 侧 increment,server 侧无对应计数器)
    3. 攻击者在内网即可冒充任意客户端调用
  - 5 域都同模板,每个域独立风险 = 5 x HC-1
- 影响: HIGH - 任何 cert 部署失误直接关闭 mTLS,且无告警无审计
- 修复建议:
  - 加环境变量 RGS_REQUIRE_TLS=true(默认 true),false 时才允许 fallback 并记录 metric
  - 加 server 侧 INSECURE_GRPC_FALLBACK_TOTAL 计数器
  - 加 tracing::error! 而非 warn!,触发 Prometheus alert
  - 单元测试覆盖: 模拟 RGS_TLS_DIR 不存在时 main 启动应 fail

### HC-2. outbox relay NATS fallback 静默累积 outbox 行,无 DLQ 表无 metric

- 位置: 5 域 main.rs(player/match/social/admin/economy)的 NATS 接线段(同模板)
- 类别: 消息 / 数据丢失风险
- 问题: NATS 不可用时仅 warn,relay 没启动,outbox 表只增不删。manual recovery required 没有具体步骤文档,没有积压阈值告警机制。
- 影响: HIGH - NATS 短暂抖动后服务看似运行正常,但消息持续累积到磁盘 100%
- 修复建议: 增加 outbox_pending_count gauge(按 5 域 label);增加 periodic NATS connect 复检;增加 DLQ 表

### HC-3. audit_log 仅有 append,无 hash chain verifier 函数 -> 篡改事后不可检测

- 位置: crates/admin-service/src/entity.rs, crates/admin-service/src/service.rs
- 类别: 密码学 / 审计完整性
- 问题:
  - 55.13 重写 compute_hash(FNV-1a -> SHA-256 + 长度前缀),但只提供 append 路径,没有 verify_chain(entries) -> Result<()> 函数
  - 当前任何代码都无从校验 prev_hash 是否等于上一条 hash,或重新计算的 hash 是否等于存储的 hash
  - DB 触发器 audit_log_no_modify 阻止 UPDATE/DELETE,但不阻止 INSERT 篡改(DBA 用 superuser 直接 INSERT 假条目)
  - 没有定时任务重算整链 + 校验
- 影响: HIGH - 审计链的事后验证能力缺失。RGS-SEC-100 §7 要求 hash 链防篡改,但仅依赖 hash 链本身不验证 = 装饰
- 修复建议:
  - 加 AuditLogEntry::verify_chain(entries) -> Result<(), ChainError>
  - 加 AdminService::verify_audit_log() 公开方法
  - 考虑加 signed_hash(用 KMS 私钥对每条 hash 签名)

### HC-4. tonic 0.12 client_auth_optional 默认值隐式信任 - 未来升级可能静默取消 mTLS

- 位置: crates/shared-platform/src/tls.rs:113-118 (load_server_tls_config)
- 类别: mTLS / 依赖默认行为
- 问题: 注释说默认 required,但代码没有显式 .client_auth_optional(false)。如果未来 tonic 升级到 0.13+ 改了默认,或有人误加 .client_auth_optional(true),会静默取消 client cert 校验。
- 影响: HIGH - 长期演进风险,未来 1 次依赖升级即可让 mTLS 失效
- 修复建议: 显式调 .client_auth_optional(false);加单元测试 assert!(cfg.client_auth_optional() == false)

### HC-5. outbox list_pending lease 时长 SQL 内硬编码 30s,不可配置

- 位置: crates/shared-platform/src/outbox.rs:300-302
- 类别: outbox / 可配置性
- 问题: UPDATE outbox SET status = in_flight, lease_until = NOW() + INTERVAL 30 seconds WHERE id = ANY()
  - 30s 适合默认,但以下场景需要更长/更短: NATS 慢 + batch 大;高频低延迟;多 DC / 跨云网络抖动
- 影响: HIGH - 调优死代码,操作风险
- 修复建议: lease 作为参数传到 PgOutboxRepository(用 INTERVAL 1 second *  绑定);RelayConfig.lease: Duration 字段

### HC-6. OutboxRepository::append 接受 PgExecutor 但 InMemory 忽略 -> 测试与生产行为分叉

- 位置: crates/shared-platform/src/outbox.rs:386-394 (InMemoryOutboxRepository::append)
- 类别: 测试一致性 / 类型安全
- 问题: 显式忽略 _executor: E。InMemory 路径不验证业务事务回滚时 outbox 也回滚。生产路径同事务保证 0 覆盖。
- 影响: HIGH - 同事务保证无测试守护;业务代码若忘记把 outbox append 放进业务 tx,InMemory 测试通过,但生产路径下消息会发出去但业务回滚
- 修复建议: 加 InMemory OutboxRepository 的 tx 概念;或加集成测试 sqlx::test 真 PG

### HC-7. Reservation::save 用 ON CONFLICT (id) DO UPDATE SET status - 只更新 status 字段

- 位置: crates/economy-service/src/reservation.rs:187-206
- 类别: 数据完整性
- 问题: 若 id 已存在,只更新 status,其他字段(amount / currency / expires_at)保持旧值。缺少 CHECK 约束 amount > 0、currency IN (...)
- 影响: HIGH - 隐性数据漂移风险
- 修复建议: 改为 ON CONFLICT (id) DO UPDATE SET status, amount, currency, expires_at;加 CHECK 约束

---

## 5. MEDIUM Issues

### MC-1. Saga / Step 状态机无非法转移检测

- 位置: crates/economy-service/src/saga.rs:199-232 (Saga state methods)
- 类别: 状态机
- 问题: start/complete/compensate/fail 都是 pub 且无状态前置检查。任何调用方可以 Completed -> start() 重新 Running。
- 影响: MEDIUM - 业务层误用导致状态污染
- 修复建议: 状态转换函数加 Result<()> 返回 + 状态前置检查;加 can_transition 白名单

### MC-2. SagaStatus::Aborted 死状态

- 位置: crates/economy-service/src/saga.rs:50,302,312
- 类别: 状态机 / 死代码
- 问题: Aborted 在 enum 中,有 saga_status_to_str / parse_saga_status 支持序列化,但没有任何代码路径写入(grep 全无业务写入)
- 影响: MEDIUM - 死代码 + 误导
- 修复建议: 选项 A 移除 Aborted variant;选项 B 加 fn abort() 方法

### MC-3. Reservation 5 分钟过期 - 但无 scheduled task 标记 Expired

- 位置: crates/economy-service/src/reservation.rs:51-89
- 类别: 资源泄漏
- 问题: expires_at = created_at + 5min,有 is_expired() 方法,但没有 background task 扫描并 mark_expired。saga 崩溃后 reservation 永远停在 Reserved 状态。
- 影响: MEDIUM - 监控/告警基于 status=reserved 会漏报
- 修复建议: 加 reservation_reaper background task;或在 saga 恢复循环中加 reservation expiry 检查

### MC-4. ReserveHandler execute 失败时残留 reservation 风险

- 位置: crates/economy-service/src/saga_orchestrator.rs:229-269
- 类别: 资源泄漏
- 问题: try_debit 失败路径有 cleanup delete_by_id,但 apply_atomic 失败路径无 cleanup
- 影响: MEDIUM - 与 CC-4 复合放大;每 OCC 冲突产生 1 个 dangling reservation
- 修复建议: apply_atomic 失败时也 delete_by_id

### MC-5. mTLS bypass 计数仅 client 端;server 端 insecure 降级无指标

- 位置: crates/shared-platform/src/channel.rs:77-87 (MTLS_BYPASSED_TOTAL);5 域 main.rs
- 类别: 监控
- 问题: client 端有 mtls_bypassed_total Prometheus 计数器,server 端 insecure 降级只有 warn。不对称
- 影响: MEDIUM - 监控盲点
- 修复建议: 在 shared-platform 加 INSECURE_SERVER_TOTAL 原子计数

### MC-6. audit_log Pg 路径:SELECT FOR UPDATE 锁住 latest,但 append_atomic + commit 间无错误聚合

- 位置: crates/admin-service/src/service.rs:147-163
- 类别: 事务一致性
- 问题: append_atomic 成功但 commit 失败 -> 偶发链跳号现象(hash 链仍合法但中间缺号)
- 影响: MEDIUM - 无 metric 难以追踪
- 修复建议: 加 audit_log_commit_failures_total Prometheus 计数器

### MC-7. 5 域 main.rs tracing subscriber 初始化后调 init() 多次 = 多次输出

- 位置: 5 域 main.rs
- 类别: 可观测性
- 问题: 若未来有集成测试启动多个 service,init 多次会 warn
- 影响: LOW - 当前生产无影响;测试扩展时是隐患
- 修复建议: 用 try_init() + 静默 ignore 重复 init

---

## 6. LOW Issues

### LC-1. 测试代码使用 unwrap() 较多
- 影响: LOW;建议 clippy 加 #![cfg_attr(test, allow(clippy::unwrap_used))]

### LC-2. SHA-256 hash 链无全局 sequence 字段
- 影响: LOW;当前 schema 防御足够;加 seq BIGSERIAL 列是过度设计

### LC-3. AdminServiceImpl audit_log fallback 路径无 tracing 日志
- 影响: LOW;调试时不便
- 修复: if self.pool.is_none() { tracing::debug!(target: admin-service, audit_log using non-atomic fallback (pool=None)); }

---

## 7. Saga 状态机审计

- 状态转移是否完备: N (Aborted 死状态 MC-2;缺 Pending -> Aborted 显式转移)
- 非法转移检测: N (所有 start/complete/compensate/fail 均为 pub,无前置状态检查 MC-1)
- 幂等保证: 部分 (IdempotencyKey 在 ledger;但 reservation.save 幂等性弱 HC-7)
- 补偿顺序合理性: Y (55.12 修复了 pre-existing 空集 bug;反向 iter().rev() 顺序)
- resume 入口: N - 完全失效 (CC-2,list_running 找 Running/Compensating 但 execute 要求 Pending)
- 并发安全: 部分 (多副本 saga 调度无显式 leader 选举;apply_atomic 内 OCC 校验保证 55.1)

---

## 8. Outbox race condition 审计

- FOR UPDATE SKIP LOCKED: Y (crates/shared-platform/src/outbox.rs:278-289 正确使用)
- 重复投递去重: 依赖业务方 / inbox UNIQUE (relay 端不做去重,consumer 靠 command_id 幂等)
- 事务原子性: 部分 (Pg 路径有 executor: PgExecutor 支持同事务,但 InMemory 路径忽略 HC-6)
- 崩溃恢复: Y (lease 过期后另一副本可重试)
- 多副本 relay 抢占: Y (SKIP LOCKED + lease 双重保护)
- in_flight lease: Y 但硬编码 30s(HC-5)

---

## 9. mTLS 5 域接线审计

| 域 | tls_config 调 | fallback insecure | 启动期验证 | 文件行号 |
|----|--------------|-------------------|-----------|---------|
| player | OK | warn + silent | 无 metric | crates/player-service/src/main.rs:117-125 |
| economy | OK | warn + silent | 无 metric | crates/economy-service/src/main.rs:193-201 |
| match | OK | warn + silent | 无 metric | crates/match-service/src/main.rs:116-124 |
| social | OK | warn + silent | 无 metric | crates/social-service/src/main.rs:116-124 |
| admin | OK | warn + silent | 无 metric | crates/admin-service/src/main.rs:115-123 |

统一问题(5 域同模板):HC-1 + HC-4 + HC-5(admin 额外受 CC-1 影响)

---

## 10. dev 密码 6 域独立化审计(55.20)

- 7 KEY 全部独立: OK (脚本 scripts/generate_dev_passwords.ps1:47-55 显式枚举 player/economy/match/social/admin/cluster_ops/postgres_su,共 7 KEY;每个 KEY 独立生成)
- .env 不入 commit: OK (.gitignore:7 屏蔽 .env;git check-ignore -v .env 命中;git ls-files | grep 关键扩展名 返回空)
- 脚本 PS 7+ 强制 + openssl rand -base64 24: OK (脚本首段检查 PSVersionTable.PSVersion.Major -lt 7 -> exit 1;调用 openssl rand -base64 24 产生 24 字节 = 32 字符 base64 熵)
- .env 写盘安全: OK (UTF-8 无 BOM;写后 Set-Acl 收紧到仅当前用户)
- 文档 RGS-SEC-101: OK (313 行覆盖背景/范围/实施/安全/合规/升级路径;明确隔离原则)

总体判定: 55.20 凭据治理实现完备,符合 RGS-SEC-100/RGS-DEC-018 规范。

---

## 11. 修复优先级矩阵

| Issue | 严重度 | 文件 | 估时 | 阻塞 |
|-------|--------|------|------|------|
| CC-1 | CRITICAL | crates/admin-service/src/main.rs:93 | 5 min | 阻断 PR merge(55.13 核心承诺未兑现) |
| CC-2 | CRITICAL | crates/economy-service/src/saga_orchestrator.rs:71-77 + saga.rs:list_running | 1 day | 阻断生产部署(崩溃恢复失效) |
| CC-3 | CRITICAL | 6 个域 migrations | 30 min | 阻断 PR merge(数据完整性) |
| CC-4 | CRITICAL | crates/economy-service/src/saga_orchestrator.rs:229-323 | 2 days | 阻断经济系统上线(资金幻影风险) |
| HC-1 | HIGH | 5 域 main.rs | 4 hour | 阻断生产部署(mTLS 静默降级) |
| HC-2 | HIGH | 5 域 main.rs | 1 day | 不阻断但生产风险(消息丢失) |
| HC-3 | HIGH | admin-service entity.rs + service.rs | 1 day | 阻断审计合规(无 verify 能力) |
| HC-4 | HIGH | shared-platform/tls.rs | 30 min | 不阻断但长期技术债 |
| HC-5 | HIGH | shared-platform/outbox.rs | 1 hour | 不阻断但运维风险 |
| HC-6 | HIGH | shared-platform/outbox.rs + service.rs | 4 hour | 不阻断但测试覆盖盲点 |
| HC-7 | HIGH | economy-service/reservation.rs | 30 min | 不阻断但数据漂移风险 |
| MC-1 | MEDIUM | economy-service/saga.rs | 1 day | 不阻断 |
| MC-2 | MEDIUM | economy-service/saga.rs | 30 min | 不阻断(清理 dead code) |
| MC-3 | MEDIUM | economy-service/reservation.rs | 4 hour | 不阻断(资源泄漏) |
| MC-4 | MEDIUM | economy-service/saga_orchestrator.rs | 30 min | 不阻断(与 CC-4 联动) |
| MC-5 | MEDIUM | shared-platform/channel.rs + 5 域 main.rs | 2 hour | 不阻断(监控盲点) |
| MC-6 | MEDIUM | admin-service/service.rs | 1 day | 不阻断 |
| MC-7 | LOW | 5 域 main.rs | 1 hour | 不阻断 |
| LC-1~3 | LOW | 散落 | 2 hour | 不阻断 |

总估时: 4 CRITICAL 必修约 3.5 天;7 HIGH 约 4 天;7 MEDIUM 约 3.5 天;3 LOW 约 0.5 天。建议优先 CC-1/2/3/4 + HC-1,共 5 个 blocking fix,约 4.5 天。

---

## 12. 关键代码引用

- compute_hash SHA-256 + 长度前缀: crates/admin-service/src/entity.rs:127-148
- append_atomic 事务化: crates/admin-service/src/repository.rs:179-206
- UNIQUE(prev_hash) migration: crates/admin-service/migrations/0002_audit_prev_hash_unique.sql
- AdminServiceImpl pool 字段: crates/admin-service/src/service.rs:50-69
- 5 域 main.rs 缺 with_pool 调用: crates/admin-service/src/main.rs:93
- SagaOrchestrator execute Pending check: crates/economy-service/src/saga_orchestrator.rs:71-77
- SagaOrchestrator resume: crates/economy-service/src/saga_orchestrator.rs:149-160
- Recovery loop 永远失败: crates/economy-service/src/main.rs:111-123
- ReserveHandler execute 顺序: crates/economy-service/src/saga_orchestrator.rs:229-269
- ReserveHandler compensate 幻影加款: crates/economy-service/src/saga_orchestrator.rs:272-323
- ConfirmHandler execute + compensate: crates/economy-service/src/saga_orchestrator.rs:350-428
- outbox list_pending SKIP LOCKED: crates/shared-platform/src/outbox.rs:273-313
- outbox lease 30s 硬编码: crates/shared-platform/src/outbox.rs:299-302
- outbox InMemory 忽略 executor: crates/shared-platform/src/outbox.rs:386-394
- load_server_tls_config 隐式 required: crates/shared-platform/src/tls.rs:92-118
- 5 域 mTLS fallback insecure: 5 个 main.rs 同模板
- DomainAdmin 缺 scope 显式 deny: crates/shared-platform/src/rbac.rs:172-185
- resource_in_scope 边界: crates/shared-platform/src/rbac.rs:222-227
- generate_dev_passwords.ps1 7 KEY: scripts/generate_dev_passwords.ps1:47-55
- .gitignore .env 屏蔽: .gitignore:7

---

## 13. 审计员签注

审计员: verify-C
签名: verify-C / 2026-08-22 / D:\RustGameServer\docs\00-基准与治理\reviews\adversarial-55\verify-C_security-saga.md

审核方法:
- 静态代码 review:12 commit diff 全部审过(5.7MB 文本)
- 关键文件全文阅读:entity.rs / repository.rs / service.rs / saga.rs / saga_orchestrator.rs / reservation.rs / outbox.rs / outbox_relay.rs / tls.rs / rbac.rs / channel.rs / grpc_tracing.rs / generate_dev_passwords.ps1 / 6 个 main.rs / 6 个 outbox migration / admin audit_log migrations
- 横向交叉:5 域 main.rs 模板一致性 + 6 域 outbox migration 一致性
- grep 验证:unsafe / panic / unwrap / Aborted / status check / list_pending / execute 等
- git check-ignore 验证 .env 屏蔽
- git ls-files 验证无 tracked 秘密文件

未验证项:
- 未跑 dynamic fuzzing / property-based test(建议 proptest 覆盖 saga execute 状态机)
- 未连真 PG 实例跑 sqlx::test 验证 Pg 路径(特别是 CC-1/CC-4 的真实事务行为)
- 未连真 NATS 实例验证 relay failover 行为
- 未分析 RGS-SEC-100 §7 backup encryption 范围(55.x 不在范围)
- 未审 cluster-ops main.rs(55.21+22 仅覆盖 5 域,cluster-ops 接线状态待确认)

待 follow-up:
- 若 CC-2 修复后,验证多副本同时 recovery 是否产生重复补偿(需要 saga-level 抢占锁)
- 验证 tonic 0.12 -> 0.13 升级时 client_auth_optional 默认是否变化
- 长期:考虑将 shared-platform 的 MIGRATION_TEMPLATE 改为 CI 可检查的 reference migration(避免模板与实际 migration 分叉,CC-3 根因)
