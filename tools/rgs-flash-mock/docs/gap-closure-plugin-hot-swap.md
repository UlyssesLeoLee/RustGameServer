# plugin hot-swap 缺口闭环报告 (rgs-flash-mock)

> 缺口主题: README.md「已知缺口」第 3 条 —— plugin 集群 draw_card_probability PoC 仅 v0.1, 缺 hot-swap E2E 实战验证
> 交付版本: v0.1 (本报告)
> 归属: mock 项目 `tools/rgs-flash-mock/` (per 2026-09-04 17:47 JST「所有开发过程中的测试脚本和测试数据都应归入 mock 项目」)

## 1. 缺口来源 (git 实证)

| 事实 | 证据 (file:line) |
|---|---|
| README「已知缺口」章节 | `tools/rgs-flash-mock/README.md:103` |
| 第 3 条原文: 「**plugin 集群** — draw_card_probability PoC 仅 v0.1, 缺 hot-swap E2E 实战验证」 | `tools/rgs-flash-mock/README.md:107` |
| 权威用例定义 (plugin 集群 7 用例) | `docs/15-IPA-完全对齐438cmds/RGS-TEST-CASES-2026-09-07_v0.2.md:303-315` |
| 跨语言 dispatch helper (5 个子命令) | `tools/rgs-flash-mock/scripts/_lib_mock_switch_rgs.py:1-165` |
| ACI registry (5 plugin × 12 module + module_count_*) | `tools/rgs-flash-mock/.aci.json:148-199` |
| cluster registry (enabled/mode/trace 模板/module_count) | `tools/rgs-flash-mock/.mock-cluster.json:1-24` |
| 既有 PoC 回归脚本 (echo 式) | `tools/rgs-flash-mock/scripts/regression-test-plugin-poc.sh:1-80` |

### 1.1 7 个 PLUGIN 用例 (逐条引用 TEST-CASES §3.2)

| 用例 | 定义 (TEST-CASES §3.2) | 本地可观测性 |
|---|---|---|
| PLUGIN-001 | PoC 抽卡 hot-swap (per 主设计书 §7.2 + RGS-PLUGIN-APP-ARCH §3.1) — L306 | **可观测 (config-binding 级)** |
| PLUGIN-002 | 阶段 1 MVP: PG registry 持久化 + 9 域共享 (per 9/5 ARCH §4.2) — L307-308 | 不可观测 (无 PG / 无 9 域 runtime) |
| PLUGIN-003 | 阶段 2: 每 app 独立更新 + 双 registry 模式 (per 9/5 ARCH §4.3) — L309-310 | 不可观测 (mock 只有单 registry) |
| PLUGIN-004 | WASM 资源限 fuel cap fail-closed — L311 | 不可观测 (无 WASM runtime) |
| PLUGIN-005 | WASM 内存限 memory_mib cap fail-closed — L312 | 不可观测 (无 WASM runtime) |
| PLUGIN-006 | plugin Paused → app 走 native fallback — L313 | **可观测 (config-binding 级)** |
| PLUGIN-007 | 各 app plugin 与 registry 一致性 — L314 | **可观测 (config-binding 级)** |

### 1.2 RGS-PLUGIN-APP-ARCH 定位结论 (未定位到, 不编造)

- 仓库级 grep `RGS-PLUGIN-APP-ARCH|plugin_switch` + `docs/` 级 grep `PLUGIN-APP-ARCH` + `docs/**/*PLUGIN-APP-ARCH*` 文件名匹配 —— **当前树中不存在同名文档**。
- `docs/` 下有 10 个文件以**引用**形式提及 `RGS-PLUGIN-APP-ARCH` (§3.1 / §4.2 / §4.3 段号), 但无承载其正文的同名文件。
- 因此本交付**只按 TEST-CASES §3.2 L303-315 的用例定义实现**, 不复述、不推测该架构文档正文。
  依据: AGENTS.md §1.1 (引用必须 git 实证) + 「缺标比错标安全」。

## 2. 交付物

| 文件 | 类型 | 说明 |
|---|---|---|
| `tools/rgs-flash-mock/mock_data/plugin_hot_swap.json` | 新增 | 场景式 fixture, 7 个 PLUGIN 用例全覆盖 (3 个可执行 + 4 个 known gap) |
| `tools/rgs-flash-mock/scripts/regression-test-plugin-hot-swap.sh` | 新增 | 真实断言回归脚本 (71 条断言), 0 依赖 cargo |
| `tools/rgs-flash-mock/docs/gap-closure-plugin-hot-swap.md` | 新增 | 本报告 |

未改动 (文件所有权外): `_lib_mock_switch_rgs.py` / `.aci.json` / `.mock-cluster.json` / `README.md` / 其它 `mock_data/*.json` / 其它脚本 / `.github/workflows/**` / `Cargo.*`。

## 3. fixture 设计 (`mock_data/plugin_hot_swap.json`)

schema 沿用 `mock_data/batch_health.json` 的场景式约定 (`module` / `version` / `description` / `scenarios[].{scenario,name,domains,expected}`), 额外增加 `executable` / `coverage` / `known_gap` 三个字段以区分「可断言」与「本地不可观测」。

```
scenarios[] = {
  scenario, name, executable(bool), coverage(str),
  domains{},                      # 涉及域/对象的自然语言描述
  expected{},                     # 脚本真正断言的期望值
  known_gap{reason,evidence,closed_by,asserted}   # 仅 executable=false 时存在
}
```

**关键设计: `expected` 全部可被脚本真断言, 且期望值从 fixture 读取** —— 脚本不硬编码计数, 而是 `mt get` fixture 后与工具真实输出比对。因此 fixture 一旦与实际配置漂移, 断言立刻失败 (而不是被静默绕过)。

`expected` 中同时使用**绝对值**与**增量**:
- 绝对值 (`plugins_total=5` / `modules_total=12` / `modules_enabled=12`): 契约快照, 与 `.aci.json` / `.mock-cluster.json` 的 `module_count_*` 三方一致;
- 增量 (`modules_enabled_delta=-1`): 切换语义, 与基线实际值解耦, 避免配置扩容时误报。

hot-swap 核心语义 (fixture `semantics.hot_swap_core`): **切换瞬间在途的调用仍由旧版本完成, 新调用才走新版本**。

## 4. 脚本设计 (`scripts/regression-test-plugin-hot-swap.sh`)

### 4.1 隔离策略

- 源配置 (`--aci-config` / `--cluster-config`, 默认 = tracked 的 `.aci.json` / `.mock-cluster.json`) **只读**;
- 脚本启动后 `cp` 到 `mktemp -d` 临时目录, 所有变更 (关闭 module / 加 module / 改 cluster.enabled / 改 compat 版本) **只发生在副本上**;
- `trap cleanup EXIT` 清理临时目录;
- 脚本末尾 **P8-1 / P8-2 自校验源文件 sha256 未变** —— 「tracked 文件零改动」是脚本自身的断言, 不是口头承诺。

### 4.2 断言矩阵 (共 71 条)

| 段 | 断言 ID | 内容 | 对应用例 |
|---|---|---|---|
| 1 基线 | P0-1..P0-2 | read-plugins exit=0 且输出是合法 JSON | PLUGIN-007 |
| 1 基线 | P1-1..P1-4 | plugins_total / plugins_enabled / modules_total / modules_enabled == fixture | PLUGIN-001 |
| 1 基线 | P1-5..P1-6 | is-enabled exit=0, stdout=`CLUSTER_ENABLED=true` | PLUGIN-001 |
| 1 基线 | P1-7..P1-8 | get-mode exit=0, stdout 含 `CLUSTER_MODE=` | PLUGIN-001 |
| 1 基线 | P1-9..P1-11 | trace exit=0, 输出非空, **逐个 plugin 名**都出现在 trace 中 | PLUGIN-001/007 |
| 1 基线 | P1-12..P1-13 | validate-compat exit=0, stdout 含 `ACI_COMPAT=OK` | PLUGIN-007 |
| 2 翻转 | P2-1..P2-4 | 副本 `enabled=false` → is-enabled exit **1** + stdout `CLUSTER_ENABLED=false` + trace 含 `cluster.enabled=false` | PLUGIN-001 |
| 2 翻转 | P2-5..P2-6 | 恢复后 exit 回到 **0**; 且翻转前后 exit **严格不等** | PLUGIN-001 |
| 3 fail-closed | P3-1..P3-2 | 副本 compat 版本改坏 → validate-compat exit=1 + stdout 含 `ACI_COMPAT=FAIL` | PLUGIN-007 |
| 4 切走 | P4-1..P4-6 | 关 `admin::card` → modules_enabled **-1** / module enabled=false / plugin 仍 enabled / cluster 仍 enabled / trace cluster token 不变 | PLUGIN-001 |
| 4 切回 | P4-7..P4-10 | 恢复副本 → read-plugins 与基线**深度相等** + delta=0 + module 恢复 enabled | PLUGIN-001 |
| 4 回滚 | P4-11..P4-17 | 叠加 v2 module 后 modules_total=基线+1; 回滚后深度相等 + delta=0 + v2 module 消失 | PLUGIN-001 |
| 5 in-flight | P5-1..P5-2 | 切换前捕获绑定: module_enabled=true, mode=offline | PLUGIN-001 |
| 5 in-flight | P5-3 | **在途调用仍按切换前捕获的绑定完成 (旧版本)** | PLUGIN-001 |
| 5 in-flight | P5-4..P5-6 | 负控: 新调用按新配置重新解析 → 与 in-flight 绑定不同 + module_enabled=false | PLUGIN-001 |
| 6 fallback | P6-1..P6-7 | Paused 后 module **仍在 registry** + enabled=false + plugin 仍 enabled + 其余 plugin 不受影响 + cluster enabled/mode 不变 | PLUGIN-006 |
| 7 一致性 | P7-1..P7-12 | `.aci.json` ↔ `.mock-cluster.json` ↔ `read-plugins` 三方: plugin/module id 集合、per-plugin 计数、`module_count_total/enabled`、trace per-plugin 计数、trace 尾部 `=12/12` | PLUGIN-007 |
| 8 自校验 | P8-1..P8-2 | 源 `.aci.json` / `.mock-cluster.json` sha256 未变 | 隔离 |
| 8 gap | — | fixture 中 `executable=false` 的用例以 `[GAP]` 打印, **不计入失败** | PLUGIN-002/003/004/005 |

### 4.3 in-flight 语义的断言方式 (含负控, 非恒真)

1. 切换**前**: 用当前 (aci, cluster) 快照解析一次调用 `admin::card` 的绑定 → 存入 ledger (`inflight_before.json`): `module_enabled=true, mode=offline`。
2. 执行切换 (关掉 `admin::card`)。
3. **P5-3**: ledger 里的 `module_enabled` 仍为 `true` → 在途调用由旧版本完成 (ledger 不被重新解析)。
4. **P5-4 (负控)**: 对**同一调用**用切换后的配置重新解析, `module_enabled=false` ≠ ledger 的 `true` → 证明 (a) 配置确实切了, (b) in-flight 的旧版本不是「因为配置没变」造成的假象。
5. **P5-5 / P5-6**: 新调用重新绑定 (`new_call != in_flight_before`) 且 `module_enabled=false` → 新调用走新版本。

即: 若把 resolver 改成「无视配置恒定返回旧值」, P5-4/P5-5 会失败; 若把 in-flight 改成「每次都重新解析」, P5-3 会失败。两个方向都可证伪。

### 4.4 退出码语义

| 退出码 | 含义 |
|---|---|
| 0 | 全部断言通过 (known gap 不计失败) |
| 1 | 存在断言失败 |
| 2 | 环境/前置缺失 (python / fixture / 被测配置缺失, 未知参数) |

## 5. 怎么验证

### 5.1 前置

- `python` 3.13.7 在 PATH (脚本用 `python` 或 `python3` 探测)
- Git Bash 5.2 (`C:\Program Files\Git\bin\bash.exe`), 依赖 `mktemp` / `sha256sum` / `awk`
- **不需要 cargo**

### 5.2 语法检查

```bash
cd tools/rgs-flash-mock
bash -n scripts/regression-test-plugin-hot-swap.sh   # rc=0
```

### 5.3 正常跑 (CI 接入形态)

```bash
cd tools/rgs-flash-mock
bash scripts/regression-test-plugin-hot-swap.sh; echo "rc=$?"
```

实测 (摘录, 完整 71 条见 §5.5):

```
=== 汇总 ===
  PASS: 71  FAIL: 0  KNOWN-GAP: 4
  结果: ✅ 全部断言通过 (known gap 未计入失败)
rc=0
```

### 5.4 可证伪验收 (破坏 → 恢复)

破坏在**临时副本**上进行, tracked 文件全程零改动:

```bash
# 破坏 1: 临时副本 cluster.enabled true -> false
bash scripts/regression-test-plugin-hot-swap.sh \
  --aci-config <tmp>/.aci.json --cluster-config <tmp>/.mock-cluster.json; echo "rc=$?"
# 预期 rc=1 (is-enabled 基线断言 P1-5 失败)

# 破坏 2: 临时副本删掉一个 plugin 条目 (match)
# 预期 rc=1 (plugins_total 4 != fixture 5, P1-1 失败)

# 恢复: 用默认 tracked 配置再跑
bash scripts/regression-test-plugin-hot-swap.sh; echo "rc=$?"
# 预期 rc=0
```

实测输出见 §5.5 (含 `git diff --stat` 为空的证据)。

### 5.5 实测输出 (可证伪验收完整记录)

见本文末尾「附录 A: 实测输出」。

## 6. 剩余缺口 (known gaps)

### 6.1 in-flight 语义的可观测性结论 (重要)

**结论: 运行时级 in-flight 语义在当前 mock 层不可观测。**

- 本脚本断言的是 **config-binding 级** 的 in-flight 语义: 「调用在启动时捕获绑定, 切换后该绑定不被重写; 新调用按新配置重新解析」。
- 真实运行时语义 (WASM 实例在切换瞬间仍在执行, 由旧版本完成, 不被中途杀掉) 需要 **function-plane WASM runtime 在跑**, 才能观测:
  - 在途实例是否被 drain / 等待完成;
  - 新版本是否等旧实例排空后才接流;
  - 旧版本 `Archived` 后是否仍有在途实例引用。
- 这些在本地**没有可观测对象** —— `tools/rgs-flash-mock/scripts/_lib_mock_switch_rgs.py` 只读本地 JSON, 不连任何 runtime。故本报告把 PLUGIN-001 的覆盖范围明确限定为 config-binding 级, 运行时级留作 known gap。
- 结论对主设计书 §7.2 的影响: 运行时级 hot-swap E2E 仍需在集成环境 (function-plane + PoC WASM 起服) 补测, 本交付不替代它。

### 6.2 用例级 known gap

| 用例 | 为什么本地不可观测 | 补齐条件 |
|---|---|---|
| PLUGIN-002 | mock 层无 PG 实例、无 9 域 app runtime; helper 只读本地 JSON | function-plane + PG 集成环境 (ARCH §4.2 阶段 1 MVP) |
| PLUGIN-003 | mock 只有单份 `.aci.json` registry, 无 app-local/global 双 registry 结构, 无 app 独立更新通道 | ARCH §4.3 阶段 2 的双 registry 数据结构落地后 |
| PLUGIN-004 | 无 WASM runtime, 配置无 fuel 字段 | function-plane WASM 起服后按 fuel cap 注入用例 |
| PLUGIN-005 | 无 WASM runtime, 配置无 memory_mib 字段 | function-plane WASM 起服后按 memory_mib 注入用例 |

对应 fixture 条目的 `known_gap.reason` / `known_gap.evidence` / `known_gap.closed_by` 字段, 脚本以 `[GAP]` 逐条打印 (不假绿、不计入失败)。

### 6.3 fixture 漂移风险 (已知, 有意为之)

`expected` 里的绝对计数 (5 plugin / 12 module) 是**契约快照**。若将来 `.aci.json` 增删 module, 需同步更新 fixture, 否则脚本会失败并提示漂移 —— 这是有意的 (比静默通过安全), 但会要求维护者同步改 fixture。

## 7. 对既有 `regression-test-plugin-poc.sh` 的判断

**判断: 它是 echo 式自述脚本, 不构成回归防线。** 逐条实证:

| 行 | 它实际做了什么 | 是否断言 |
|---|---|---|
| L19-33 | 真跑 `cargo check --tests` (唯一有真实退出码的检查) | ✅ 有退出码, 但只查编译, 不查任何 hot-swap 行为 |
| L38-42 | `curl` function-plane `/registry/list`, 失败则打 `⚠️` 后**继续往下跑** | ❌ 失败不终止, 无断言 |
| L46-53 | 纯 `echo` 输出 `function_id` / `version` / `runtime` / `status` / `fuel` / `memory_mib` / `hash_sha256` | ❌ 全部硬编码字符串, 不读取任何真实状态 |
| L57-65 | 纯 `echo` 输出 hot-swap 的 step1-4 与 expected | ❌ 「expected: card-service 进程不重启」等**没有对应任何检查** |
| L69-72 | 纯 `echo` 输出 native fallback / registry Paused / DLQ | ❌ 无检查 |
| L75-80 | 结尾统一打 `✅` | ❌ 无论前面是否失败都会打 ✅ |

即: 它宣称覆盖 PLUGIN-001~007, 但实际只有 `cargo check` 一处有真实退出码; 全部 7 个用例的行为断言都是 echo 常量, **在真回归中会恒绿** (假绿)。

本脚本的定位与之互补: 不跑 cargo, 只用已有 helper (`_lib_mock_switch_rgs.py`) 对**真实配置 + 真实工具输出**做 71 条断言, 覆盖 PLUGIN-001/006/007 的可观测部分, 并把不可观测的 4 个用例显式登记为 known gap。

## 8. 集成建议 (给主会话)

```bash
cd tools/rgs-flash-mock
bash scripts/regression-test-plugin-hot-swap.sh
```

- CI 接入: 作为一条独立 job 即可, 无需 cargo / 无需起服 / 无需集群, 纯本地配置级回归, 秒级完成。
- 退出码: 0 = 通过 (可直接作 CI gate), 1 = 断言失败, 2 = 环境缺失。
- 若 CI 需要以 ACI 断言格式输出, 可后续接 `tools/aci-summary/aci_summary.py` (本脚本当前只输出 `[OK]/[FAIL]/[GAP]` 文本, 未接 ACI emitter —— 属已知未做项, 非缺陷)。

## 附录 A: 实测输出

### A.1 四轮可证伪验收 (驱动脚本为临时文件, 不入 commit)

```
$ bash .falsify-hot-swap.sh

##### A. 正常态 (默认 tracked 配置) #####
  PASS: 71  FAIL: 0  KNOWN-GAP: 4
A_rc=0

##### B. 破坏 1: 临时副本 cluster.enabled=true -> false #####
  PASS: 65  FAIL: 6  KNOWN-GAP: 4
  [FAIL] P1-5   is-enabled exit (enabled=true) (expected=[0] actual=[1])
  [FAIL] P1-6   is-enabled stdout (expected=[CLUSTER_ENABLED=true] actual=[CLUSTER_ENABLED=false])
  [FAIL] P2-5   恢复后 is-enabled exit (expected=[0] actual=[1])
  [FAIL] P4-5   切走后 cluster 仍 enabled (app 不重启) (expected=[true] actual=[false])
  [FAIL] P4-6   切走后 trace cluster token 不变 (missing 'cluster.enabled=true' in [cluster.enabled=false,...])
  [FAIL] P6-6   Paused 后 cluster 仍 enabled (expected=[true] actual=[false])
B_rc=1

##### C. 破坏 2: 临时副本删除一个 plugin 条目 (match) #####
  PASS: 60  FAIL: 10  KNOWN-GAP: 4
  [FAIL] P1-1   plugins_total == fixture (expected=[5] actual=[4])
  [FAIL] P1-2   plugins_enabled == fixture (expected=[5] actual=[4])
  [FAIL] P1-3   modules_total == fixture (expected=[12] actual=[10])
  [FAIL] P1-4   modules_enabled == fixture (expected=[12] actual=[10])
  [FAIL] P6-4   Paused 后其余 plugin 不受影响 (expected=[5] actual=[4])
  [FAIL] P7-8   read-plugins modules_total (expected=[12] actual=[10])
  [FAIL] P7-9   read-plugins modules_enabled (expected=[12] actual=[10])
  [FAIL] P7-10  read-plugins plugins_total (expected=[5] actual=[4])
  [FAIL] P7-11  trace per-plugin 计数一致 (expected=[true] actual=[false])
  [FAIL] P7-12  trace 尾部计数一致 (expected=[true] actual=[false])
C_rc=1

##### D. 恢复: 重新用 tracked 配置跑 #####
  PASS: 71  FAIL: 0  KNOWN-GAP: 4
D_rc=0

##### E. tracked 文件 git diff (必须为空) #####
E_git_diff_stat_empty=true
```

结论: 两种破坏都被检出 (rc=1), 恢复正常 (rc=0), tracked 配置零改动。

### A.2 正常态关键断言 (节选)

```
[1/8] 前置 + 基线读取 (PLUGIN-007 契约快照)
  [OK]   P0-1                     read-plugins exit code (= 0)
  [OK]   P0-2                     read-plugins 输出是合法 JSON
  [OK]   P1-1                     plugins_total == fixture (= 5)
  [OK]   P1-3                     modules_total == fixture (= 12)
  [OK]   P1-5                     is-enabled exit (enabled=true) (= 0)
  [OK]   P1-6                     is-enabled stdout (= CLUSTER_ENABLED=true)
  [OK]   P1-11-1                  trace 含 plugin [admin] (contains 'admin')
  [OK]   P1-13                    validate-compat stdout (contains 'ACI_COMPAT=OK')

[2/8] cluster.enabled 严格翻转 (PLUGIN-001 cluster_enabled_flip)
  [OK]   P2-1                     enabled=false 时 is-enabled exit (= 1)
  [OK]   P2-5                     恢复后 is-enabled exit (= 0)
  [OK]   P2-6                     exit code 严格翻转 0→1 (!= 0)

[4/8] hot-swap 三段: 切走 / 切回 / 回滚 (PLUGIN-001)
  [OK]   P4-2                     切走后 modules_enabled delta (= -1)
  [OK]   P4-8                     切回后与基线深度相等 (= true)
  [OK]   P4-12                    v2 态 modules_total = 基线+1 (= 13)
  [OK]   P4-14                    回滚后与基线深度相等 (= true)
  [OK]   P4-17                    回滚后 v2 module 消失 (= true)

[5/8] in-flight 语义: 在途调用走旧版本 / 新调用走新版本 (PLUGIN-001)
  [OK]   P5-3                     in-flight 绑定保持旧版本 (= true)
  [OK]   P5-4                     in-flight 绑定 vs 新解析 不同 (= true)
  [OK]   P5-6                     新调用 module_enabled (= false)

[6/8] native fallback: plugin Paused 时 app 仍可路由 (PLUGIN-006)
  [OK]   P6-1                     Paused 后 module 仍在 registry (= true)
  [OK]   P6-2                     Paused 后 module enabled=false (= false)
  [OK]   P6-7                     Paused 后 cluster mode 不变 (= true)

[7/8] 三方一致性 (.aci.json ↔ .mock-cluster.json ↔ read-plugins) (PLUGIN-007)
  [OK]   P7-1                     plugin id 集合一致 (= true)
  [OK]   P7-11                    trace per-plugin 计数一致 (= true)
  [OK]   P7-12                    trace 尾部计数一致 (= true)

[8/8] tracked 文件零改动自校验 + known gap 登记
  [OK]   P8-1                     源 .aci.json sha256 未变 (= e268dbf1...)
  [OK]   P8-2                     源 .mock-cluster.json sha256 未变 (= f0b0c74c...)
  [GAP]  PLUGIN-002               known gap (not-observable-locally): ...
  [GAP]  PLUGIN-003               known gap (not-observable-locally): ...
  [GAP]  PLUGIN-004               known gap (not-observable-locally): ...
  [GAP]  PLUGIN-005               known gap (not-observable-locally): ...

=== 汇总 ===
  PASS: 71  FAIL: 0  KNOWN-GAP: 4
  结果: ✅ 全部断言通过 (known gap 未计入失败)
```

### A.3 运行环境

| 项 | 值 |
|---|---|
| bash | `C:\Program Files\Git\bin\bash.exe` (Git Bash 5.2) |
| python | 3.13.7 (PATH 中 `python`) |
| 单轮耗时 | 约 2-3 min (Windows; 约 70 次 python 进程启动。Linux CI 上同量级断言应显著更快) |
| 依赖 | `mktemp` / `sha256sum` / `awk` (Git Bash 自带) |
| 不需要 | cargo / k3s / 起服 / 网络 |

## 附录 B: 交付元信息

- Base: `dev` @ `4f773122` (ci(mock): 补上 rgs-flash-mock 的 CI 覆盖)
- Branch: `agent/mavis/mock-plugin`
- Worktree: `D:\RustGameServer-worktrees\mock-plugin`
- 提交: 1 个 commit (per AGENTS.md L16)
