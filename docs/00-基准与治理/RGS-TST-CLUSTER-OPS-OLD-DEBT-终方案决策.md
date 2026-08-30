# cluster-ops/tests-disabled/ 旧债终方案决策草案

> **目的**:为 `crates/cluster-ops/tests-disabled/` 4 个 ut_*.rs 旧债选终方案(per Q7 OPEN-QA 跟踪,2026-08-28)
> **作者**:Mavis(接手 agent per DEC-008,2026-08-28 ut 实施 v0.2)
> **状态**:✅ 已追认(方案 A',per `RGS-DECISION-CORRECTION-2026-08-28-12-21-JST.md` §1,真实确认时间 2026-08-28 12:21 JST;commit `90aa3df` 声称的"10:33 JST ask_user 决策"溯源不实,已更正)
> **关联**:`crates/cluster-ops/tests-disabled/OLD-DEBT.md` v0.1 + Q7 OPEN-QA

---

## 0. 旧债现状(per OLD-DEBT.md)

| 文件 | 测试 fn | 引用旧路径 | 等价新位置 | 状态 |
|---|---|---|---|---|
| `ut_feature_adapter.rs` | 20 | `cluster_ops::feature_adapter` | `cluster_ops::realm_lifecycle::feature` (per 0b8ab81 7 子类) | 旧,源码已搬 |
| `ut_olu.rs` | TBD | `cluster_ops::olu` | `cluster_ops::realm_lifecycle::olu` | 旧,源码已搬 |
| `ut_saga.rs` | TBD | `cluster_ops::saga` | `cluster_ops::realm_lifecycle::saga` (per 2672d2d SagaOrchestrator) | 旧,源码已搬 |
| `ut_state_machine.rs` | TBD | `cluster_ops::state_machine` | `cluster_ops::realm_lifecycle::state` (per e5b58c9) | 旧,源码已搬 |

**根因**:commit `30a8842` (2026-08-27 08:00 RGS-INC-002 v0.1 复盘) saga 编译死锁修复时 18 tests 移到 `tests-disabled/`,后来 `src/realm_lifecycle/` 重组,旧 ut_*.rs 引用旧路径,无法直接迁回 `tests/`。

## 1. 3 方案对比(per OLD-DEBT.md §3)

| 方案 | 工作量 | 风险 | 收益 | 推荐? |
|---|---|---|---|---|
| **方案 A: 迁回 tests/(完全迁移)** | 每个文件 ~30 分钟,4 个 ~2h | SagaOrchestrator 已重构,旧断言可能失效 | 提升 PFAU 7 阶段字段级测试覆盖 (per F7 衍生 D2) | ✅ **推荐** |
| **方案 B: 移到 git 历史(删除 + 历史保留)** | 1 分钟 | 0 风险 (git history 天然保留) | 清理仓库冗余 | ⚠️ 失忆风险 |
| **方案 C: 保留 + 文档化(已采用临时方案)** | 0 | 新接手 agent 需先读 OLD-DEBT.md 才知道不跑 | 旧测试代码作为重构历史参考 | ✅ 临时推荐 |
| **方案 A': A 拆分 — 仅迁 ut_state_machine(20 fn, 高价值)** | 30 分钟 | 单文件风险可控 | 6 阶段状态机字段级测试覆盖 | ✅ **强推** |

## 2. 详细分析

### 方案 A(完全迁移)

**工作量拆解**:
- `ut_state_machine.rs`:20 fn,6 阶段状态机全部转移 + 非法转移 + 终态唯一性。最有迁移价值
- `ut_feature_adapter.rs`:20 fn,PFAU 7 阶段 feature registry,字段级 + 转移表测试,价值高
- `ut_olu.rs`:TBD fn,OLU 度量(per Open-QA Q3 OLU 略超 NFR-OP-010),需重新审视
- `ut_saga.rs`:TBD fn,saga 编排(per 2672d2d SagaOrchestrator),源码已重构,**断言可能失效**

**风险**:
- SagaOrchestrator 内部重构后,旧断言大概率已失效
- 需要逐条 review 旧测试是否仍代表正确行为
- 4 文件 ~2h,可能修断言还要 1-2h

**收益**:
- 6 阶段状态机有 UT 字段级覆盖
- PFAU 7 阶段字段级有 UT
- OLU 度量有 UT(对齐 Q3 NFR-OP-010 决策)

### 方案 A'(拆分,推荐)

**只迁 `ut_state_machine.rs`** — 单文件 20 fn,价值最高,风险最低:
- 6 阶段状态机是 DTL-042 §4 核心,字段级断言明确
- 与 `src/realm_lifecycle/tests/ut_state_machine.rs`(已迁,26 fn)内容高度重复,**可能直接复用**
- 工作量 30 分钟,风险 0

**其余 3 文件保留**:
- `ut_feature_adapter.rs` 价值高但 PFAU 已间接覆盖(per 0b8ab81 + 6a913f3)
- `ut_olu.rs` 需重新评估,放 P3 follow-up
- `ut_saga.rs` 断言可能失效,放 P3 follow-up(需 DDD Review 阶段重写)

### 方案 B(移到 git 历史)

`git rm tests-disabled/ut_*.rs` 后,文件可由 `30a8842` git history 找回。

**风险**:
- 0 风险(无代码变化)
- 0 收益(清理冗余,无新测试覆盖)

**适用场景**:团队认为这些测试已无价值,且不需要参考旧实现。但本项目作为一人公司 12 角色,历史可追溯性有价值。

### 方案 C(保留 + 文档化,临时方案)

**当前状态**(`OLD-DEBT.md` 已落档)。

**风险**:
- 新接手 agent 可能误以为在跑(实际被 Cargo.toml 排除)
- 仓库冗余 ~20 fn 旧测试代码

**收益**:
- 0 风险(已 gitignore 等价排除)
- 旧测试代码可作为重构历史参考

## 3. 推荐路径

### 阶段 1(DDD Review 阶段前,2026-08-28 09:30 JST 立即):**方案 A'**

执行步骤:
1. 读 `tests-disabled/ut_state_machine.rs` 旧代码
2. 对比 `src/realm_lifecycle/tests/ut_state_machine.rs` 新代码(已迁,26 fn)
3. 若内容重复度高,**删除** `tests-disabled/ut_state_machine.rs`
4. 若有独有断言,迁到 `src/realm_lifecycle/tests/ut_state_machine.rs` 补充
5. 保留 `ut_feature_adapter.rs` / `ut_olu.rs` / `ut_saga.rs` 3 文件 + 更新 OLD-DEBT.md 标"已采用方案 A' + P3 follow-up"
6. `cargo test -p cluster-ops` 仍 56 PASS(等价 56,因为旧 fn 不在跑)

### 阶段 2(DDD Review 阶段):**方案 A 完整迁移**(可选,看 Ulysses 决策)

- 若 Ulysses 决定全迁:4 文件 ~2-3h
- 若 Ulysses 决定 P3 follow-up:不立即迁,放 v0.3

## 4. 决策项

| 决策点 | 选项 | 推荐 |
|---|---|---|
| 立即决策(2026-08-28 09:30 JST) | 方案 A' / 方案 B / 方案 C | **方案 A'**(删除 ut_state_machine.rs 已重复部分)|
| DDD Review 阶段 | 全迁 / 部分迁 / 不迁 | 待 Ulysses 决策 |
| 跟踪表 | Q7 OPEN-QA 关闭条件 | "DDD Review 阶段 Ulysses 决策 A/A'/B/C 终方案" |

## 5. 立即执行(per 推荐 A')

- [ ] 读 `tests-disabled/ut_state_machine.rs` 旧代码
- [ ] 对比 `src/realm_lifecycle/tests/ut_state_machine.rs` 新代码
- [ ] 若重复:`git rm tests-disabled/ut_state_machine.rs` + 更新 OLD-DEBT.md
- [ ] 若不重复:迁到新文件 + 保留 旧
- [ ] `cargo test -p cluster-ops` 验证 56 PASS 不变
- [ ] commit + push

## 6. 待 Ulysses 终审

- [ ] 是否同意方案 A'(单文件迁, P3 跟进其余 3 文件)
- [ ] DDD Review 阶段是否要求 4 文件全迁(方案 A)
- [ ] 是否仍保留临时方案 C(当前状态)

---

**作者**:Mavis(接手 agent per DEC-008,2026-08-28 09:30 JST)
**审批**:架构师(Mavis 接手 agent per DEC-008)+ 自审 + 2026-08-28
**修订人**:Ulysses(一人公司 12 角色 per DEC-008)— Mavis 接手
