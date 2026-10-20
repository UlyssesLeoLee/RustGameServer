# 跨域故事链 + 跨域事件链闭环 (GAP-002)

> 缺口来源: `tools/rgs-flash-mock/README.md:105` (gap_matrix 22 RPC stub 缺 E2E 故事链) + `tools/rgs-flash-mock/README.md:106` (saga_abstract 仅 mock, 缺 e2e cross-domain event verification)
> 归属: `src/saga_verify.rs` + `src/lib.rs` (挂 `pub mod saga_verify;`) + `tests/saga_story_chain.rs` + `scripts/regression-test-cross-domain-event.sh`
> 代签: Mavis 默认代签 Ulysses (per 8/27 19:39/20:56/21:59 JST 三次强化)

## 1. 模型

| 概念 | 说明 |
|---|---|
| `StoryStep` | 一步 = 域 + `gap_matrix` **真实 rpc_code** + 期望 `RpcStatus` + 本步发出的跨域事件 + 本步消费的事件 |
| `CrossDomainEvent` | `(from_domain, event_name, to_domain)` |
| `StoryChain::cross_domain_default()` | 5 域链 player → economy → match → social → admin |
| `StoryChain::verify(&GapMatrix)` | 逐步核对状态 + 校验事件链, 返回 `StoryChainReport` |

默认链的 5 步 rpc_code 与期望状态全部取自 `src/gap_matrix.rs` 的 22 个 stub (不另造数据源):

| 步骤 | 域 | rpc_code / rpc | 期望状态 |
|---|---|---|---|
| player.session.open | player | 201 GetPlayerProfile | Partial |
| economy.reserve | economy | 602 CreateAuction | Pass |
| match.start | match | 301 StartCombat | Pass |
| social.notify | social | 702 SendMessage | Partial |
| admin.audit | admin | 1101 BanAccount | Pass |

事件链: `player.profile.loaded` → `economy.reservation.confirmed` → `match.combat.started` → `social.message.delivered`, 每条被下一步消费。

## 2. 校验 4 件事 (各自独立可测)

1. **步骤状态**: rpc_code 在 gap_matrix 里不存在, 或实际 `RpcStatus` 与期望不符 → 该步失败。
2. **悬空事件**: 某步产生的事件没有任何下游步骤消费。
3. **顺序错乱**: 某步消费的事件由**更后面**的步骤产生。
4. **域归属不符**: 消费方域 != 事件的 `to_domain`。

失败定位到**具体步骤**: 报告给出失败步名 + rpc_code + 期望 vs 实际 (`first_failure` / `failed_rpc_codes`), 不是笼统 bool。

## 3. 怎么验证

```bash
cd tools/rgs-flash-mock
bash scripts/regression-test-cross-domain-event.sh
# 或直接: cargo test --test saga_story_chain --locked
```

集成测试 7 条 + `src/saga_verify.rs` 内单元测试 4 条, 覆盖:
- 默认链全绿 / 覆盖 ≥3 个域
- 注入状态偏差 → 报出 `match.start` (rpc 301)
- gap_matrix 无此 code → 报出 `gap_matrix 无 rpc_code 99999`
- 悬空事件 / 顺序错乱 / 域归属不符 各检出 1 条

判据: 失败路径全部由**显式注入的期望值偏差**驱动, 不用随机数 / sleep / 并发, 每次运行结果一致。

## 4. 可证伪实测

变异方式: 把 `match.start` 的期望状态从 `Pass` 改成 `Partial` → 该测试必须红; 恢复 → 绿。

## 5. 剩余缺口 (已知, 本 lane 不覆盖)

- **ST 阶段真实域未接**: 故事链当前只跑在 mock 层 (gap_matrix stub), k3s 真实域的 per-case 接入仍缺 (README.md:105 后半句)。
- saga_abstract 3 module 的**真实**跨域事件验证需要真实 NATS / 域服务, 本地无集群; 本 lane 只固化事件链的一致性判定口径。
- `src/lib.rs` 的 CYPHER HEADER 由 `tools/cypher-header-gen/cypher_header.py` 生成 (已随本次改动更新); 注意该生成器会重写全仓 180 个过期 header, 跨 lane 使用时必须限定 `--only` 范围。
