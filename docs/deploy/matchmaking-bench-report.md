# match-service 撮合核心函数 Benchmark 报告

> **状态：⏳ 待实跑（PH-1 编码完成后实跑 + 填入实测值）**
>
> per RGS-DTL-026 v0.4 §4.1.3 + RGS-OPEN-QA-001 Q-D-10 + ACTIONS-v0.3 A-10
>
> 跟踪任务：WF-1-55.42（v0.4 占位 + 框架）/ `WF-?-??.??`（PH-1 之后实跑，编号预留）
>
> ⚠️ **本报告当前为占位文档，不构成性能数据**。所有 p50/p95/p99 字段为预填占位值（`-`），待实跑后由实现侧人工整理 criterion HTML 报告填入。引用本报告前必须确认状态从"⏳ 待实跑"切换为"✅ 已实跑"。

---

## 1. 目标

量化 match-service 撮合核心函数（per RGS-DTL-026 §4.2 `matchmaker_tick`）在不同候选数 n 下的端到端延迟分布，为 §4.1.1 `max_candidates_per_tick` 占位（n≤500）→ 实测值切换提供数据依据。

**核心问题**（per RGS-OPEN-QA-001 Q-D-10）：

> 撮合核心函数在 NFR-PT 单局决策 ≤ 100ms 约束下，单轮 `matchmaker_tick` 能承受的 n 上限是多少？占位 n≤500 是否保守？是否需要触发 §4.1.2 降级策略？

**测试目标函数**：`matchmaker_tick(mode, shard_scope, now) -> Vec<ProposedMatch>`，对应 DTL-026 §4.2 既定实现路径（候选扫描 + tolerance 过滤 + `try_compose_teams` 组合搜索）。

---

## 2. 方法

### 2.1 工具

- **criterion.rs 0.5**（Rust 标准 benchmark 框架，per `crates/match-service/benches/matchmaking_bench.rs`）
- HTML 报告：自动生成到 `target/criterion/matchmaking_tick_n<N>/report/`
- Markdown 摘要：本文件（实现侧人工整理）

### 2.2 测试输入

- **n 档位**：`{100, 200, 500, 1000, 2000}` 共 5 档（per DTL-026 §4.1.3）
- **每档 iteration 数**：100（per DTL-026 §4.1.3）
- **候选分布**：`composite_rating` 服从高斯分布 N(1500, 200²)，模拟真实玩家评分
- **等待时长分布**：所有候选 `enqueued_at = 0`，固定 `now_ms = 1_000_000`，使 `tolerance()` 取 `initial_tolerance = 50`（最严格条件，撮合难度最高）
- **种子**：`0xDEAD_BEEF_CAFE_F00D`（固定，保证可复现）

### 2.3 执行命令

```bash
cd D:\RustGameServer-worktrees\WF-1-55-42
cargo bench -p match-service --bench matchmaking_bench
```

### 2.4 编译配置

- `opt-level = 3` + LTO = "thin" + `codegen-units = 1`（per workspace `[profile.release]`）
- `cargo bench` 默认 release profile
- 单线程执行（`--jobs 1`）避免并行噪声
- 冷启动 + criterion 热身 3s 后采样

### 2.5 软硬件环境（实跑时填入）

| 项 | 实测值 |
|---|---|
| CPU 型号 | _（待实跑）_ |
| 物理核数 | _（待实跑）_ |
| 内存大小 | _（待实跑）_ |
| OS | _（待实跑）_ |
| Rust 工具链 | rustc 1.98.0（已确认） |
| criterion 版本 | 0.5.x（已确认） |
| 实跑日期 | _（待实跑）_ |
| 实跑人 | _（待实跑）_ |

---

## 3. 结果

> ⚠️ **以下所有数值均为占位"-"，待 PH-1 之后实跑 `cargo bench` 后由实现侧人工填入。**

### 3.1 延迟分布（端到端 `matchmaker_tick`）

| n 档位 | 样本数 | p50 (ms) | p95 (ms) | p99 (ms) | mean (ms) | std dev (ms) | 配对成功数 (mean) |
|---|---|---|---|---|---|---|---|
| 100 | 100 | - | - | - | - | - | - |
| 200 | 100 | - | - | - | - | - | - |
| 500 | 100 | - | - | - | - | - | - |
| 1000 | 100 | - | - | - | - | - | - |
| 2000 | 100 | - | - | - | - | - | - |

### 3.2 NFR-PT 合规性

| n 档位 | p99 实测 (ms) | 100ms 阈值 | 结论 |
|---|---|---|---|
| 100 | - | ✅ pass / ❌ fail | _（待实跑）_ |
| 200 | - | ✅ pass / ❌ fail | _（待实跑）_ |
| 500 | - | ✅ pass / ❌ fail | _（待实跑）_（**占位 n≤500 的硬性断言**） |
| 1000 | - | ✅ pass / ❌ fail（仅记录） | _（待实跑）_ |
| 2000 | - | ✅ pass / ❌ fail（仅记录） | _（待实跑）_ |

### 3.3 复杂度验证（O(n²) 期望）

| n 档位 | 期望 ratio（vs n=100 基线） | 实测 ratio | 偏差 |
|---|---|---|---|
| 200 | 4× | - | - |
| 500 | 25× | - | - |
| 1000 | 100× | - | - |
| 2000 | 400× | - | - |

偏差应 < 2×（cache miss + L2/L3 抖动导致常数项变化）。偏差 > 2× 提示算法存在非 O(n²) 隐藏开销或 stand-in 失真，需排查。

---

## 4. 结论

> 待实跑后填入。三种典型结果路径已在 DTL-026 v0.4 §4.1.3 第 6 款约定，此处仅占位骨架。

### 4.1 n ≤ 500 满足 NFR-PT p99 < 100ms（最可能）

- **占位 n≤500 保留**：`config/match-service.toml` 字段 `matchmaking.max_candidates_per_tick = 500` 不变。
- **§4.1.2 降级策略**保持"超限才触发"。
- **v0.5 升版时机**：若实测 n=1000 也满足 p99 < 100ms，**主动**升版 DTL-026 至 v0.5 把占位扩到 1000（per §4.1.1 修订历史跟踪约定）。
- **v0.5 升版判定标准**：
  - n=1000 p99 < 100ms **且** 复杂度验证 ratio 偏差 < 2×。
  - 实跑人签字 + Ulysses（per DEC-008 派生）复核。
  - 修订历史注明"per benchmark 报告 YYYY-MM-DD 实测 n=1000"。

### 4.2 n ≤ 500 不满足 NFR-PT p99 < 100ms（次可能）

- **占位 n≤500 缩到实测上限**：例如实测 n=300 满足、n=500 不满足，则 `max_candidates_per_tick = 300`。
- **§4.1.2 降级策略**调整为"默认路径"：`max_candidates_per_tick` 与桶大小 `n'` 同值时无降级，n > `n'` 时拆分撮合轮触发（per §4.1.3 第 6 款第 2 项）。
- **触发预案**：若实测 n=100 也超 100ms，提示 `try_compose_teams` 内部策略有问题，需另行排查（不在本任务边界）。
- **v0.5 升版内容**：占位缩值 + §4.1.2 第一步降级路径重写为"默认路径"语义。

### 4.3 n > 1000 仍满足 NFR-PT（意外结果）

- **占位 n≤500 大幅扩到实测值**：例如实测 n=2000 满足 p99 < 100ms，则 `max_candidates_per_tick = 2000`。
- **§4.1.2 桶大小 `n'` 同步扩到 2000**；n > 2000 触发拆分撮合轮。
- **复杂度验证**应严格检查：n=2000 p99 < 100ms 意味着 0.4μs/pair（100ms / 2000²），与现代 x86_64 / ARM64 单核 L1 cache 命中 + 单条 SIMD 比较的极限一致；该结果若出现，提示 stand-in 简化过度（真实 `try_compose_teams` 实际开销会更大），**应在 v0.5 升版前用更接近真实实现的 stand-in 重测**。

### 4.4 工程推荐 n 上限（最终结论栏）

| 场景 | 工程推荐 n 上限 | 降级路径 |
|---|---|---|
| _（待实跑）_ | _（待实跑）_ | _（待实跑）_ |

---

## 5. 后续动作

> 待实跑后按 §4 结论分情况执行。

### 5.1 实跑完成后

1. **填表**：把 §3 所有 `-` 字段替换为实测值，移除"⏳ 待实跑"状态标记，改"✅ 已实跑"。
2. **复现性验证**：相同 seed + 相同硬件跑第二遍，p99 偏差应 < 5%。
3. **criterion HTML 报告归档**：把 `target/criterion/matchmaking_tick_n*/report/index.html` 复制到 `docs/deploy/criterion-snapshots/matchmaking-bench-YYYY-MM-DD.html`，作为审计追溯证据。
4. **DTL-026 升版**：按 §4 结论触发 v0.5 升版流程（提交 v0.5 草案 + Ulysses 复核 + commit）。

### 5.2 实跑后异常处理

- **panic / 段错误**：criterion 自身 panic 会导致 bench 整体失败，提示 stand-in 有内存安全问题；不修改 stand-in 实现（PH-1 替换为真实实现），仅记录 panic 信息到本报告 §6 异常栏。
- **p99 超出 1s**：criterion 默认 10s measurement 预算可能不够，需手动调大 `measurement_time` 重跑。
- **n=2000 OOM**：criterion 单次迭代内存约 n × sizeof(QueueEntry) ≈ 2000 × 64B = 128KB，远低于默认 1GB 限制；若 OOM 出现，提示生成函数有内存泄漏，需排查 `generate_candidates`。

### 5.3 与 DTL-026 §4.1.3 任务边界的合规

- 本报告**仅**在 PH-1 L4 任务（`WF-?-??.??` 编号预留）实跑后由实现侧填入。
- WF-1-55.42（本任务）**不**实跑 `cargo bench`，**不**修改 stand-in 实现，**不**填本报告数据。
- 任何对本报告的"填值"动作必须发生在 PH-1 之后，否则报告无效。

---

## 6. 异常栏（实跑时记录）

| 日期 | 异常类型 | 处置 |
|---|---|---|
| _（待实跑）_ | _（待实跑）_ | _（待实跑）_ |

---

## 7. 追溯性

| 来源 | 章节 |
|---|---|
| RGS-DTL-026 v0.4 §4.1.1（n≤500 占位） | §4.1.1 引用 + §5 后续动作 |
| RGS-DTL-026 v0.4 §4.1.2（降级策略） | §4.2/§4.3 结论分支 |
| RGS-DTL-026 v0.4 §4.1.3（benchmark 子任务） | 全文契约 |
| RGS-DTL-026 v0.4 §4.2（撮合算法） | §1 目标函数 |
| RGS-OPEN-QA-001 Q-D-10 | §1 核心问题 + §2.1 工具选型 |
| RGS-OPEN-QA-001-ACTIONS-v0.3 A-10 | 任务来源 |
| NFR-PT 单局决策 ≤ 100ms | §2.4 编译配置 + §3.2 NFR-PT 合规性 + §4 结论 |
| WBS WF-1-55.42 | 任务边界（v0.4 占位 + 框架，不实跑） |
| `crates/match-service/benches/matchmaking_bench.rs` | 实测代码入口 |
| `crates/match-service/Cargo.toml` | criterion 0.5 dev-dep + [[bench]] target |

---

## 8. 修订历史

| 版本 | 日期 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-25 | Ulysses（per DEC-008 派生子代理 WF-1-55.42） | 初版占位报告：目标/方法/结论/后续动作骨架齐全，所有数值字段标 `-` 待实跑；不构成性能数据；状态 ⏳ 待实跑 |
