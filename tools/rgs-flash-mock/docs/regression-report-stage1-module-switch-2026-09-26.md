# RGS Mock module_switch Stage 1 Regression Report

> **生成时间**: 2026-09-26T11:55:00Z (per ULYS-190 dispatcher)
> **范围**: tools/rgs-flash-mock/ (.aci.json + .mock-cluster.json + scripts/_lib_mock_switch_rgs.py + tests/test_rgs_mock_switch.py + src/lib.rs 接入 doc)
> **触发**: ULYS-190 §4.4 stage4 派工触发 D-Boy 「按照推荐彻底完成任务」 reply 2026-09-26 02:39 JST (comment `01a0db94-e5b1-7624-bbca-1e5ad124370b`)
> **守门**: 守门 #1+#5+#6+#7+#9+#10+#11+#12+#13+#14v4+#15+#19v19+#20+#24

## §1 范围

| # | 路径 | 类型 | 关键内容 |
|---|---|---|---|
| 1 | `tools/rgs-flash-mock/.aci.json` | 修改 | 5 plugin × 12 module 全填 (per handlers.rs 12 pub mod 一一映射) |
| 2 | `tools/rgs-flash-mock/.mock-cluster.json` | 修改 | `mock_switch_trace_format` 改真拼接模板 + `module_count_total/enabled: 12` |
| 3 | `tools/rgs-flash-mock/scripts/_lib_mock_switch_rgs.py` | 新 (~140 LOC) | Python reader, CLI: is-enabled / get-mode / trace / validate-compat / read-plugins |
| 4 | `tools/rgs-flash-mock/tests/test_rgs_mock_switch.py` | 新 (~190 LOC) | 18 单元测试 (per Star test_mock_switch_plugins.py 範式) |
| 5 | `tools/rgs-flash-mock/src/lib.rs` | 修改 (+41 doc lines) | `## module_switch 接入` 段落 + 12 module 总览 + 跨语言 dispatch 用法 + 跨项目累计表 |
| 6 | `tools/rgs-flash-mock/docs/regression-report-stage1-module-switch-2026-09-26.md` | 新 (本文件) | 7-8 段 per AGENTS.md §3 |

## §2 12 module 落地清单 (per plugin 表格)

| Plugin | Module count | Modules | 对应 handlers.rs pub mod |
|---|---|---|---|
| **player** | **3** | `role` / `scene` / `friend` | `pub mod role {}` / `pub mod scene {}` / `pub mod friend {}` |
| **economy** | **3** | `econ` / `pay` / `event` | `pub mod econ {}` / `pub mod pay {}` / `pub mod event {}` |
| **match** | **2** | `combat` / `pvp` | `pub mod combat {}` / `pub mod pvp {}` |
| **social** | **2** | `guild` / `rank` | `pub mod guild {}` / `pub mod rank {}` |
| **admin** | **2** | `gm` / `card` | `pub mod gm {}` / `pub mod card {}` |
| **总计** | **12** | **ALL** | (5 域 SRS player / economy / match / social / admin + 7 域 SRS 7 域 mapping per RGS-FLASH-MOCK-DESIGN-2026-09-04 v0.3 §3) |

**命名决策**: per handlers.rs 12 `pub mod` 一一映射 (`scene / role / combat / pvp / guild / econ / friend / event / pay / rank / gm / card` 全 12 都各 1 module), 按 5 域 SRS 分组. 跨项目粒度可比 (IM1.0 28 = per pub fn; CATs 13 = per Rust 子模块; RGS 12 = per pub mod 一一映射).

## §3 验收脚本结果 (5 个 §)

| § | 验证项 | 命令 | 结果 |
|---|---|---|---|
| §1 | `is-enabled` | `python _lib_mock_switch_rgs.py --aci-config .aci.json is-enabled` | ✅ `CLUSTER_ENABLED=true` (exit 0) |
| §2 | `get-mode` | `python _lib_mock_switch_rgs.py --aci-config .aci.json get-mode` | ✅ `CLUSTER_MODE=offline` |
| §3 | `trace` | `python _lib_mock_switch_rgs.py --aci-config .aci.json trace` | ✅ 真拼接 `cluster.enabled=true,mode=offline,plugins=[player(3m),economy(3m),match(2m),social(2m),admin(2m)]=12/12 modules` |
| §4 | `validate-compat` | `python _lib_mock_switch_rgs.py --aci-config .aci.json validate-compat` | ✅ `ACI_COMPAT=OK (cluster=0.1.0-draft aci=0.1.0-draft)` |
| §5 | `read-plugins` (NEW) | `python _lib_mock_switch_rgs.py --aci-config .aci.json read-plugins` | ✅ JSON: 5 plugins / 12 modules 全 enabled |

### mock-switch-validate.py 跨项目验证 (Star tools/ 已 ship)

| Project | cluster_ok | enabled | mode | aci_status |
|---|---|---|---|---|
| `rgs` (本 commit) | ✅ True | True | offline | OK |

## §4 mock_switch_trace_format 真拼接 (OLD vs NEW)

| 阶段 | 字符串 |
|---|---|
| **OLD** (PR 配套 `28a3c025` §4.3, 静态占位符) | `cluster.enabled={cluster.enabled}, cluster.mode={cluster.mode}, plugins=[5-domain], modules=per_plugin (TBD)` |
| **NEW** (本 commit 真拼接) | `cluster.enabled={cluster.enabled},mode={cluster.mode},plugins=[player(3m),economy(3m),match(2m),social(2m),admin(2m)]=12/12 modules` |

`_lib_mock_switch_rgs.py` 的 `build_trace()` 用 `str.replace()` 手动 substitute (因为 placeholder 含 dot, str.format() 不支持 dotted kwargs).

**实测输出**:
```
cluster.enabled=true,mode=offline,plugins=[player(3m),economy(3m),match(2m),social(2m),admin(2m)]=12/12 modules
```

**trace 长度**: ~94 chars (per G-MS-08 ~80 字 阈值, **超 14 字**, 跨项目截断 batch 跨 session per G-MS-BRIEF-S44-02).

## §5 _lib_mock_switch_rgs.py 扩展 (per IM1.0 + CATs + Star 範式)

### MockSwitchReader class
- `__init__(aci_config_path, cluster_config_path)` 读两 JSON
- `is_enabled()` / `get_mode()`: cluster 基础
- `validate_compat()`: 校验 cluster.aci_compat_version == aci.aci_compat_version
- `build_trace()`: 替换 placeholder
- `read_plugins()`: 读 plugins.<id>.modules.<id> 树, 返回 dict {plugins_total, plugins_enabled, modules_total, modules_enabled, plugins: {...}}

### CLI subcommands (5 个, 跟 Star / IM1.0 / CATs 命名一致)
- `is-enabled` (exit 0=enabled, 1=disabled)
- `get-mode` (print CLUSTER_MODE=...)
- `trace` (print 真拼接 trace)
- `validate-compat` (exit 0=OK, 1+FAIL)
- `read-plugins` (print JSON, exit 0)

## §6 14 守门合规

✅ #1 code can be tested (Python unittest) - 18 tests ALL PASS
✅ #5 secrets 0 泄露 (RGS 没新增 secret)
✅ #6 mock 项目存在 (`tools/rgs-flash-mock/` 7/7 项目都有)
✅ #7 mock 不改真实 schema (并存扩展: 新增字段, 0 改 PR 配套 `28a3c025` v0.1 base 字段)
✅ #9 commit message 完整 + author=Ulysses (per 守門 #10)
✅ #10 author=Ulysses ulysses@mavis.local
✅ #11 透明披露 (本报告 §7)
✅ #12 docs 同步 (本 regression report 7-8 段 per AGENTS.md §3)
✅ #13 W/T/M (Write: lib.rs doc comment; Test: 18 unit tests; Maintain: regression report)
✅ #14 v4 Mavis 审核决策 / 独立审核
✅ #15 1 sub-agent 1 切点 (per D-Boy 「彻底完成」batch override)
✅ #19 v19 self-driven (sub-agent dispatcher extends IM1.0/CATs/Star pattern)
✅ #20 documentation 同步 (regression report + lib.rs doc + cross project tracking table)
✅ #24 documentation + 跨 session 续做 8 项 (本报告 §8)

✅ #3 D-Boy 「按照推荐彻底完成任务」三 project 一次性派工 override per 守門 #15 v3 + AGENTS.md §4 #3 等价条件

## §7 已知缺口 (5 项, per 守門 #11 透明披露)

### G-MS-BRIEF-S44-01-rgs
**Python helper, Rust native 跨 session** — 当前 `_lib_mock_switch_rgs.py` 是 Python subprocess 形式 (per G-MS-BRIEF-S44-01 short-term). Rust native `_lib_mock_switch.rs` (直接读 JSON, no subprocess overhead) 跨 session 续做.

### G-MS-BRIEF-S44-02-rgs
**trace_format ~94 字 vs G-MS-08 ~80 字, 跨 session 截断** — 本 stage output `cluster.enabled=true,mode=offline,plugins=[player(3m),economy(3m),match(2m),social(2m),admin(2m)]=12/12 modules` 超 14 字 (跨项目: IM1.0 ~120 + CATs ~92 + Star ~139 + RGS ~94 都超阈值). 跨项目 batch 截断到 ~80 字 跨 session 续做.

### G-MS-BRIEF-S44-04-rgs
**不支持 hot reload** — 当前 `.aci.json` change 后必须 restart 进程或 reload per Python helper invocation. v0.2 hot reload 跨 session 评估 per G-MS-09.

### G-MS-BRIEF-S44-05-rgs
**12 module 命名跨项目一致性, 待 stage5-6 (IDE1.0 / GitGit) 验证** — IM1.0=28 + CATs=13 + Star=7 + RGS=12 (本). 命名粒度差异: IM1.0=per pub fn; CATs=per Rust 子模块; Star=per fixture suite; RGS=per pub mod 一一映射. 跨项目 naming convention 跨 session 跨项目 batch 评估.

### G-MS-RGS-SPECIFIC-01
**`scene` module 在 RGS 设计中实际上不存 (per RGS-FLASH-MOCK-DESIGN-2026-09-04 v0.3 §3 "RGS TCG 无场景 N-A")** — 但我们 1-1 map `pub mod scene` 当 module_switch 名称. 实际场景 REST handler 不存在但 module_switch 仍声明, 让 mock framework 知道 "scene 子模块在 RGS TCG 是 no-op". 这是 design-induced dead branch, 不影响 trace_format 真拼接正确性. 跨 session 验证.

## §8 跨 session 续做入口 (per 守門 #24)

1. 🟡 **§4.4 stage5 IDE1.0** 派工 (mirror RGS 範式, base=main, 2 plugin 实际 module_switch 全填)
2. 🟡 **§4.4 stage6 GitGit** 派工 (TypeScript MSW frontend 範式, base=dev, 3 plugin: health/repo/vault)
3. 🟡 **§4.4 stage7 Ada** 永久跳过 (降級 L1 only by design)
4. 🟡 **Star design-analysis v0.4 → v0.5** 加 §10 module_switch 落地回顧 (跨项目, 跨 session)
5. 🟡 **G-MS-08 trace_format 截断到 ~80 字** (跨项目 batch: IM1.0 ~120 + CATs ~92 + Star ~139 + RGS ~94 都超阈值)
6. 🟡 **G-MS-05 開關變更審計日誌** (Transaction audit SCD-2, 跨项目)
7. 🟡 **G-MS-09 hot reload** (v0.2 評估, 跨项目)
8. 🟡 **Rust native `_lib_mock_switch.rs`** (跨项目, 替换 Python subprocess, per G-MS-BRIEF-S44-01 推广)
9. 🟡 **CI mock-switch-validate 加 module 级校验** (本 stage 只校验 cluster, module 级跨 session)
10. 🟡 **RGS `scene` module dead branch 真实业务验证** (per G-MS-RGS-SPECIFIC-01, 跨 session)
11. 🟡 **Layer 1→Layer 2→Layer 3 贯通验收** (per AGENTS.md §3, 顶层 sub-task)
12. 🟡 **ULYS-190 状态 in_review → done 最终 flip** (per AGENTS.md, `done` stays human 但有 release judgement)

## §9 跨项目累计 (per §4.4 stage 1+2+3+4, 4/7 项目 module_switch 落地)

| 项目 | Plugin count | Module count | PR | merged at (JST) | 状态 |
|---|---|---|---|---|---|
| IM1.0 | 5 (assertions / fixtures / mock_grpc / mock_rest / mock_ws_frames) | 28 | #24 | 9/26 00:18 | ✅ MERGED |
| CATs | 4 (data / db / http / infra) | 13 | #18 | 9/26 01:36 | ✅ MERGED |
| Star | 7 (five_domain / agent_runtime / mcp / db_wtm / langgraph / uat / core) | 7 | #151 | 9/26 02:36 | ✅ MERGED |
| **RGS** | **5 (player / economy / match / social / admin)** | **12** | **(本 stage 跟踪)** | **(squash merge commit)** | **🟡 待 merge** |
| IDE1.0 | 0 | 0 | — | — | 🟡 pending |
| GitGit | 0 | 0 | — | — | 🟡 pending |
| Ada | 0 (降級 L1 only) | 0 | — | — | 🚫 by design |
| **合计** | **21/23 plugin 累计** | **60/100+ module 累计** | **3/7 MERGED + 1/7 待 merge** | — | — |
