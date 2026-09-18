## ULYS-56 完成报告 — ADR-0061 候选已立 (待具名人类审批)

### 交付

- **commit**: `ecbabc8 docs(ULYS-56): ADR-0061 CDC/Outbox 偏离候选 — 自研 Outbox 取代 Debezium CDC`
- **branch**: `agent/minimaxm3/ulys-56`
- **文件**: `docs/08-架构决策记录/RGS-ADR-0061_CDC_Outbox偏离参考设计_自研Outbox取代DebeziumCDC.md` (254 行 / ~28KB)
- **状态**: **待具名人类审批** (per DEC-008 一人公司兼任)

### 验收标准覆盖 (per 工单 §验收标准)

| # | 验收项 | 覆盖位置 | 备注 |
|---|---|---|---|
| 1 | ADR-0061 §1-§7 完整结构 | 全文档 | 与 ADR-0059 / ADR-0060 同构骨架 |
| 2 | 自研 Outbox 捕获事件族范围 | §1.3.3 | 6 域 (admin/economy/match/player/social + cluster-ops) outbox 表清单 |
| 3 | 与 ULYS-54.A ADR-0060 联动 | §4 已知张力 + §5 关联 | Debezium→NATS connector 缺失论证 + 协同事件传播栈 |
| 5 | Debezium 合规性评估 | §1.4 | 主项目 Apache-2.0 vs Confluent 商业产品区分 |
| 6 | INV-001 v0.2 联动 | §6 P3 | ULYS-56 不修改 INV-001, 由 ULYS-54 协调者执行 |

### 关键证据

- **代码层**: `crates/shared-platform/src/outbox.rs` (4 状态机: Pending/InFlight/Sent/Failed + 30s lease + FOR UPDATE SKIP LOCKED) + `outbox_relay.rs`
- **6 域实装**: admin-service / cluster-ops / economy-service / match-service / player-service / social-service 各自 `main.rs` 启动 outbox relay 后台轮询
- **6 份 migration**: `0003_outbox.sql` + `0004_outbox_check_idempotent.sql` 等
- **零 Debezium**: `Cargo.toml` workspace + 32 个 crate `Cargo.toml` + `Cargo.lock` + `docs/deploy/` 全 grep 零命中

### 关键论证 (vs 工单 §偏离事实表)

- **状态机**: 工单描述「5 状态机」实际为 **4 状态机** (Pending/InFlight/Sent/Failed + `lease_until` 时间戳), 已订正并附 `outbox.rs` L52-75 行号引用
- **捕获范围**: 工单描述「5 域 + cluster_ops + shared_platform」实际为 **5 域 + cluster-ops = 6 服务** (shared_platform 是库 crate 无独立 outbox 表), 已订正并附 `crates/admin-service/migrations/0003_outbox.sql` 等行号引用
- **BR-111 边界澄清**: Debezium 主项目 Apache-2.0 与 BR-111「禁止 Redis Enterprise / 云专有 / 商业 SaaS / 闭源事务协调器」**不冲突**; RGS 拒绝 Debezium 的真实理由是 **OLU 估算** + **设计替代性** (事务内强制 outbox 写入已实现「事务一致性 + 事件传播」), 非 BR-111 合规

### 与 ADR-0059 / ADR-0060 的同构处置 (三面治理漏洞闭合)

| 维度 | 缓存 (ADR-0059) | 事件总线 (ADR-0060) | CDC (ADR-0061) |
|---|---|---|---|
| 偏离事实 | Redis 7.2+ vs 上游 Valkey | NATS JetStream vs 参考 Kafka | 自研 Outbox vs 参考 Debezium |
| 闸门触发 | ❌ 未触发 | ❌ 未触发 | ❌ 未触发 |
| ADR 候选 | ✅ 已立 | ✅ 已立 | ✅ **本工单已立** |

### 后续工作项 (P0-P3 共 11 项)

- **P0**: 具名人类审批
- **P1** (5 项): REQ-005 §3/§4 + REQ-100 §7 BR-111 备注 + BAS-001 §4.7/§5.8 + DTL-100 §5.3 补注
- **P2** (4 项): ADR-0015 决策链补注 + Outbox 监控指标 + Poison event DLQ + Schema evolution
- **P3** (1 项): ULYS-54 INV-001 v0.2+ §4.4 联动 (ULYS-54 协调者执行, 非 ULYS-56 范围)

### 关联工单状态

- **ULYS-55 (ULYS-54.A)**: ADR-0060 NATS vs Kafka 候选已立, 无依赖可并行 ✅
- **ULYS-56 (ULYS-54.B)**: ADR-0061 自研 Outbox vs Debezium CDC 候选已立 ✅ (本工单)
- **ULYS-54 父工单**: 关闭前 INV-001 v0.2+ 需追加三面治理漏洞闭合标注 (属协调者范围)

### 仓库状态

```
## agent/minimaxm3/ulys-56
 M AGENTS.md                                ← 平台 runtime block, 未触碰
?? .multica/                                ← 平台运行时, 未触碰
ecbabc8 docs(ULYS-56): ADR-0061 ...        ← 本工单交付
5cc57c4 chore(agent): baseline
```
