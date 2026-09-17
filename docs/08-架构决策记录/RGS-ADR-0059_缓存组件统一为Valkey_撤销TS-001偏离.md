# RGS-ADR-0059: 缓存组件统一为 Valkey，撤销 RGS-TS-001 §3.5.1 对 Redis 的偏离

| 项目 | 内容 |
|---|---|
| 决策编号 | RGS-ADR-0059 |
| 标题 | 缓存组件统一为 Valkey（撤销 RGS-TS-001 §3.5.1 自 v0.4 起对 Redis 的改选） |
| 状态 | **待具名人类审批**（per DEC-008 一人公司兼任；本文为候选提案，由 worker (ULYS-54) 起草） |
| 制定日期 | 2026-09-15 JST |
| 制定人 | worker (ULYS-54 agent) |
| 主对应方针 | ARC-014（未证明需要不引入）、RGS-ADR-0008（中间件导入判定基准） |
| 关联上游决议 | RGS-REQ-005 附件 D §4 OSS 许可盘点 L452（缓存基础设施 = **Valkey**） |
| 关联下游文档 | RGS-TS-001 §3.5.1 + §5.1；RGS-OPS-001 §1.3/§2.3/§2.5；RGS-BAS-001 §9.1.1；RGS-DTL-006 §4；RGS-ADR-0052 §5.2.2；RGS-CAP-001 §3.3；RGS-BDD-v0.2-addendum-frontend 适配层 §1.2/§7.2 |
| 关联调查 | `docs/00-基准与治理/ULYS-54-RGS-INV-001_缓存选型与设计偏离调查报告_v0.1.md`（§3 根因追溯 + §5.1 P0 建议） |

> **状态说明**：本文是候选 ADR，用以弥补 RGS-ADR-0008 §4「驳回须写明未满足哪一条」的反向义务——TS-001 §3.5.1 自 v0.4 (2026-08-21) 起改选 Redis 7.2+ 时未触发闸门、未复审 REQ-005 §4 上游登记。本 ADR 把偏离正式归档为**驳回 Redis + 恢复 Valkey** 的单点决定。具名人类审批通过前，本文不构成生产基线变更；TS-001 / OPS-001 / BAS-001 等下游文档的具体文字修改在审批通过前不执行。

---

## 1. 背景（Context）

RGS 的缓存组件在两份独立登记册上产生了"双轨"决议：

- **上游登记册**：RGS-REQ-005 附件 D §4 OSS 许可盘点 L452（2026-08-19）把 Valkey 登记为「缓存基础设施」合规组件，备注「Redis 2024 年许可变更后的分支」——即选 Valkey 是为了规避 Redis 上游许可变更。
- **下游选型报告**：RGS-TS-001 §3.5.1（v0.4 起，2026-08-21）改选 Redis 7.2+，理由 3 条（ARC-013 背压 / 自托管许可 / Lua 脚本），且 v0.5/v0.6/v0.7 三次升版未回头补救。

此后所有下游文档（BAS-001、DTL-006、OPS-001、ADR-0052、CAP-001、REQ-100、DDD-2026-09-04）依次跟随 Redis，再无人复审上游决议。

代码层至今**零 Redis 客户端依赖**：workspace `Cargo.toml` 与 32 个 crate `Cargo.toml` 全部无 `redis`/`fred`/`deadpool-redis`/`bb8-redis`/`valkey` 依赖；唯一含「Redis 缓存」字样的是 `crates/i18n-service/src/service.rs` 的占位 `Arc<Mutex<BTreeMap>>`（注释为「W36+ 替换为 redis-rs 真实 client」，未实装）。即偏离纯粹是"纸面偏离"，尚未产生代码侧影响；但 RGS-OPS-001 §2.3 已固化 `image: redis:7-alpine`，一旦开发者按此启动会引入 Redis 客户端依赖，替换成本随之上升。

ULYS-54 调查报告（`ULYS-54-RGS-INV-001` v0.1）§3.3 逐条反验 TS-001 §3.5.1 的 3 个理由：

| TS-001 理由 | 反验结论 |
|---|---|
| 「满足 ARC-013 背压设置位置」 | ARC-013 限定「背压 / 死锁防止」；限流 / 排行榜属 ARC-012「缓存 / 临时状态适用边界」——**理由张冠李戴** |
| 「License 切换为 RSAL/SSPL 后仍可自托管」 | 自托管 ≠ 无变更负担；RSAL/SSPL 变更影响的是 Redis 8.x+，7.2 仍可保留旧 BSD——**理由选择性忽略上游合规诉求** |
| 「Lua 脚本支持复杂原子操作」 | Valkey 7.x 完整继承 Redis 7.x Lua 语义 + `FUNCTION` + `EVALSHA`——**理由不构成选型差异** |

3 条理由中 2 条无效、1 条选择性忽略上游合规诉求。**RGS-ADR-0008 §2 闸门条件 ①「既有组件（Valkey）无法承担该职责」不成立**：Valkey 是 Redis 7.2 的直接 fork，RESP 协议字节级兼容、客户端 API 兼容、`FUNCTION` / `EVALSHA` / `CLUSTER` / `Redlock` 全套继承。

## 2. 决定（Decision）

**统一缓存组件为 Valkey，撤销 RGS-TS-001 §3.5.1 自 v0.4 (2026-08-21) 起的 Redis 决议。** 具体内容：

1. **缓存产品**：Valkey 7.2+（per REQ-005 §4 上游登记 + REQ-100 §7 BR-111「禁止 Redis Enterprise / 云专有 / 商业 SaaS」+ REQ-100 §7「评估纯开源替代方案」）。Valkey 是 Linux Foundation 治理下的 BSD-3 项目，原生 Redis 7.2 字节级 fork。
2. **协议层**：保持 RESP3（Valkey 与 Redis 共享协议层），客户端 API 零迁移成本——已存在的 `redis-rs`/任意 RESP 客户端可直接对接 Valkey 服务器（字节级兼容，已被 Valkey 项目官方文档与多家大型生产部署证实）。
3. **算法与抽象**：RGS-DTL-006 §4 的 Lua 限流、ADR-0052 §5.2.2 的分布式锁、CAP-001 §3.3 的容量基线、BDD-v0.2 §1.2/§7.2 的适配层——全部按「缓存 = Valkey」对齐，命名从 `redis` 改为 `valkey`（`infra.kind=valkey` 或抽象为 `cache.kind`，推荐后者以满足 NFR-MI-005 可替换空间）。
4. **下游级联**（本 ADR 审批通过后执行）：
   - **RGS-TS-001 §3.5.1**：状态从「【已决】Redis 7.2+」改为「【已决】Valkey 7.2+ (per REQ-005 §4 上游登记, Redis 2024 许可变更后的合规 fork)」
   - **RGS-TS-001 §5.1 已决选型表**：「Redis 7.2+」→「Valkey 7.2+」
   - **RGS-TS-001 修订历史**：新增 v0.9 条目记录本次统一决议
   - **RGS-OPS-001 §1.3/§2.3/§2.5**：`image: redis:7-alpine` → `image: valkey/valkey:7-alpine`（Valkey 官方 Docker Hub 镜像，兼容 redis-cli），`REDIS_URL` → `VALKEY_URL`（环境变量命名统一；协议字节兼容，旧客户端无需改）
   - **RGS-BAS-001 §9.1.1**：错误日志 `infra.kind` 由 `redis` 改 `valkey` 或抽象为 `cache.kind`
   - **RGS-DTL-006 §4**：Lua 限流算法命名由「Redis INCR+EXPIRE」改「Valkey INCR+EXPIRE」；语义不变
   - **RGS-ADR-0052 §5.2.2**：分布式锁由「Redis Redlock」改「Valkey Redlock」；可选 fallback 仍为 NATS KV lock
   - **RGS-CAP-001 §3.3**：QPS 引用源由「Redis Redlock 单实例 50k」改「Valkey Redlock 单实例 50k」（量级一致，注释替换即可）
   - **RGS-BDD-v0.2 §1.2 + §7.2**：消解内部矛盾——§1.2「不引入 Redis」与 §7.2「v0.3+ 评估」冲突；统一为「不引入（per ADR-0059，缓存统一 Valkey）」
   - **代码层**：`crates/i18n-service/src/service.rs` 注释「W36+ 替换为 redis-rs 真实 client」改为「W36+ 替换为 valkey 真实 client」（或保持客户端名 `redis-rs` 不变，因其兼容 Valkey 协议）
5. **不引入新客户端 crate**：保留现状（无任何 Redis/Valkey 客户端依赖），未来实装时按 NFR-MI-005「可替换」原则选用 `redis-rs`（已支持 Valkey）或 `valkey-rs`。
6. **撤销决议**：本 ADR 通过后，TS-001 §3.5.1 的 Redis 决议**不再生效**；附件 D §3 决议登记补行「ADR-0059 缓存统一 Valkey, 撤销 TS-001 §3.5.1」。

## 3. 曾考虑并否决的方案（Alternatives Considered）

### 3.1 维持 Redis 7.2+ 不变

否决理由：

- 直接违反 RGS-REQ-005 §4 上游登记——上游登记具有合规意义，TS-001 §3.5.1 改选时未复审此行；
- RGS-ADR-0008 §2 闸门条件 ①「既有组件（Valkey）无法承担该职责」不成立——Valkey 是 Redis 7.2 fork，字节级兼容；
- TS-001 §3.5.1 给出的 3 个理由中 2 条无效、1 条选择性忽略上游合规诉求（详见 §1）；
- 维持 Redis 8.x+ 将面临 RSAL/SSPL 双重许可的持续合规审查负担；维持 Redis 7.2 钻的是版本空子，不是合规论证。

### 3.2 同时引入 Redis 与 Valkey（按业务域切分）

否决理由：

- 同一职责两套实现违反 ARC-014「同一类目只选一个」+ TS-001 §1.5「同一类目只选一个」原则；
- 引入新运维实体（两套部署、两套镜像、两套监控、两套升级），违反 RGS-ADR-0008 §2 条件 ③（OLU 估算）；
- 客户端 API 完全兼容，业务域切分没有技术必要性。

### 3.3 切到 KeyDB / DragonflyDB（其他 Redis 兼容分叉）

否决理由：

- TS-001 §3.5.1 备选已否决 KeyDB（与 Redis 7 兼容性未保证）、DragonflyDB（尚不成熟）；本 ADR 不引入新分叉评估，遵从上游登记；
- KeyDB 已加入 MariaDB 商业版，许可复杂度高于 Valkey（Valkey 是 Linux Foundation 治理下的纯 BSD-3）；
- DragonflyDB 在生产环境中成熟度不足（per TS-001 §3.5.1 备选论证）。

## 4. 结果与代价（Consequences）

**获得**：

- 与上游登记（RGS-REQ-005 §4）的合规决议一致，消除"双轨"治理漏洞；
- 缓存组件对齐「纯开源 BSD-3」许可（Valkey Linux Foundation 治理），消除 RSAL/SSPL 变更带来的持续合规审查负担；
- ARC-014 / RGS-ADR-0008 闸门得到执行——单点改选须经 ADR 评审的先例成立；
- 客户端 API 字节兼容，代码层零迁移成本（截至当前无任何 Redis/Valkey 客户端依赖被引入）；
- NFR-MI-005「缓存可替换空间」从纸面落到抽象层（`infra.kind=valkey` 或 `cache.kind`）。

**付出**：

- 下游 7 份文档的字面更新（命名一致性，无技术影响）；
- RGS-ADR-0008 §3.1 「逐案裁量无成文依据」的反对观点不再成立——本 ADR 把「缓存选型」从「裁量」升为「ADR 登记」，回归 ARC-014 闸门路径；
- 若未来缓存需求变化（如需要 Redis 8.x 独有特性），需新立 ADR 论证「既有 Valkey 无法承担」。

**已知张力**：

- **RGS-BDD-v0.2 适配层 §1.2/§7.2 内部矛盾**——§1.2「不引入 Kafka/Redis」与 §7.2「v0.3+ 评估 Redis 备份」冲突。本决议统一为「不引入（per ADR-0059，缓存统一 Valkey）」，但 v0.3+ 的"备份存储"诉求若重新出现，须在 ADR 中独立论证。
- **RGS-ADR-0052 §5.2.2 Redis Redlock**——本决议统一为 Valkey Redlock，但保留 NATS KV lock 作为可选 fallback（与 §2 决定 4 一致）。

## 5. 关联

- **依赖本决策**：ARC-014（未证明需要不引入）、RGS-ADR-0008（中间件导入判定基准）、RGS-REQ-005 附件 D §4 OSS 许可盘点、RGS-REQ-100 §7 BR-111（禁止 Redis Enterprise / 云专有 / 商业 SaaS / 闭源事务协调器）
- **本决策撤销的偏离**：RGS-TS-001 §3.5.1（v0.4 起至 v0.7 持续偏离）
- **本决策触发的下游级联**：RGS-TS-001 §3.5.1/§5.1/修订历史；RGS-OPS-001 §1.3/§2.3/§2.5；RGS-BAS-001 §9.1.1；RGS-DTL-006 §4；RGS-ADR-0052 §5.2.2；RGS-CAP-001 §3.3；RGS-BDD-v0.2 §1.2/§7.2
- **冲突 / 延伸事项**：见 §6 后续工作项；不与本 ADR 直接冲突

## 6. 后续工作项（按优先级）

| 优先级 | 工作项 | 备注 |
|---|---|---|
| **P0** | 本 ADR 具名人类审批（Ulysses 一人公司 12 角色 per DEC-008） | 审批通过后执行 §2 决定 4 的下游级联 |
| **P1** | RGS-TS-001 v0.9 升版（更新 §3.5.1 + §5.1 + 修订历史） | 候选操作者：架构师；本 ADR 通过后即可起草 |
| **P1** | RGS-OPS-001 v_next 升版（更新 §1.3/§2.3/§2.5 的镜像与 URL 命名） | 候选操作者：架构师 / SRE Lead |
| **P2** | BDD v0.2 §1.2/§7.2 内部矛盾消解 | 已在 §2 决定 4 中列入；BDD 维护者 follow-up |
| **P2** | 缓存客户端依赖登记（CLAUDE.md / AGENTS.md / Cargo.lock 后续 PR） | 当前无依赖；引入时按 NFR-MI-005 抽象层 |

## 7. 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-09-15 JST | worker (ULYS-54 agent) | 初版制定。撤销 TS-001 §3.5.1 Redis 偏离；统一缓存为 Valkey；下游级联清单 7 项；后续工作项 5 项 |

> **下次评审**：随 ULYS-54 处置决议同步更新（批准 / 修订 / 驳回）。