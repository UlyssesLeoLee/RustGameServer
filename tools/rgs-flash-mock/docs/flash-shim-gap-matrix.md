# RGS shim ↔ 闪烁之光 协议缺口矩阵

> schema `rgs.flash.shim-gap-matrix/v1` · 由 `scripts/build_shim_gap_matrix.py` 生成

> 协议源 `D:\RustGameServer\tools\rgs-flash-mock\mock_data\flash_proto_registry.json`

> shim 真 handler `D:\RustGameServer\tools\rgs-shim-rust\src\registry.rs`

> shim stub 表 `D:\RustGameServer\tools\rgs-shim-rust\src\registry_stubs.rs`

## 1. 总账

| 指标 | 值 |
|---|---|
| 闪烁之光真实 protocol code | **1696** |
| 其中 cfg 有声明 | 768 |
| 仅已实现未声明 | 928 |
| shim 真 handler (`map.insert`) | **401** |
| shim stub (`or_insert` + `handle_stub`) | 766 |
| **合计覆盖** | **779 (45.93%)** |
| ├ 真 handler 占比 | 23.53% |
| └ stub 占比 (丢 payload 回 unknown) | 22.41% |
| **缺口 missing** | **917** |
| phantom (shim 有 / 真实协议无) | 14 |

> ⚠️ 旧 `rgs-shim/README.md` 记的 "6/514 = 1.2%" 已过时: 514 是当时
> 只看部分文件得到的口径. 按 Erlang pack/cfg 实证, 真实 code 为 1696 个.

## 2. 按 proto 模块分布

| module | real | stub | missing | 小计 |
|---|---|---|---|---|
| `proto_1` | 0 | 0 | 2 | 2 |
| `proto_10` | 21 | 0 | 28 | 49 |
| `proto_101` | 3 | 0 | 0 | 3 |
| `proto_102` | 2 | 0 | 17 | 19 |
| `proto_103` | 23 | 0 | 3 | 26 |
| `proto_104` | 4 | 0 | 9 | 13 |
| `proto_105` | 8 | 0 | 0 | 8 |
| `proto_108` | 7 | 0 | 0 | 7 |
| `proto_109` | 11 | 0 | 7 | 18 |
| `proto_11` | 3 | 29 | 7 | 39 |
| `proto_110` | 28 | 36 | 19 | 83 |
| `proto_111` | 0 | 5 | 4 | 9 |
| `proto_112` | 0 | 4 | 8 | 12 |
| `proto_113` | 0 | 3 | 0 | 3 |
| `proto_114` | 0 | 0 | 7 | 7 |
| `proto_115` | 0 | 0 | 40 | 40 |
| `proto_116` | 0 | 0 | 6 | 6 |
| `proto_117` | 0 | 0 | 15 | 15 |
| `proto_12` | 0 | 4 | 2 | 6 |
| `proto_127` | 0 | 14 | 11 | 25 |
| `proto_129` | 0 | 5 | 3 | 8 |
| `proto_13` | 16 | 12 | 5 | 33 |
| `proto_130` | 5 | 7 | 35 | 47 |
| `proto_133` | 23 | 0 | 0 | 23 |
| `proto_134` | 12 | 0 | 1 | 13 |
| `proto_135` | 25 | 0 | 27 | 52 |
| `proto_136` | 0 | 4 | 1 | 5 |
| `proto_138` | 0 | 0 | 5 | 5 |
| `proto_141` | 0 | 4 | 1 | 5 |
| `proto_16` | 9 | 26 | 3 | 38 |
| `proto_164` | 3 | 0 | 0 | 3 |
| `proto_165` | 0 | 0 | 2 | 2 |
| `proto_166` | 22 | 0 | 8 | 30 |
| `proto_167` | 0 | 7 | 3 | 10 |
| `proto_168` | 0 | 3 | 0 | 3 |
| `proto_19` | 15 | 0 | 4 | 19 |
| `proto_20` | 0 | 33 | 38 | 71 |
| `proto_200` | 0 | 23 | 17 | 40 |
| `proto_201` | 0 | 0 | 8 | 8 |
| `proto_202` | 0 | 11 | 10 | 21 |
| `proto_203` | 0 | 2 | 0 | 2 |
| `proto_204` | 0 | 0 | 24 | 24 |
| `proto_205` | 0 | 3 | 1 | 4 |
| `proto_206` | 0 | 12 | 7 | 19 |
| `proto_207` | 0 | 4 | 0 | 4 |
| `proto_208` | 0 | 0 | 18 | 18 |
| `proto_209` | 0 | 0 | 7 | 7 |
| `proto_21` | 0 | 22 | 5 | 27 |
| `proto_210` | 0 | 4 | 0 | 4 |
| `proto_211` | 0 | 2 | 0 | 2 |
| `proto_212` | 0 | 2 | 3 | 5 |
| `proto_213` | 0 | 9 | 11 | 20 |
| `proto_214` | 0 | 4 | 10 | 14 |
| `proto_215` | 0 | 5 | 0 | 5 |
| `proto_216` | 0 | 0 | 2 | 2 |
| `proto_217` | 0 | 0 | 6 | 6 |
| `proto_218` | 0 | 0 | 11 | 11 |
| `proto_219` | 0 | 0 | 11 | 11 |
| `proto_22` | 0 | 5 | 1 | 6 |
| `proto_220` | 0 | 0 | 2 | 2 |
| `proto_221` | 0 | 2 | 0 | 2 |
| `proto_222` | 0 | 4 | 7 | 11 |
| `proto_223` | 0 | 0 | 8 | 8 |
| `proto_224` | 0 | 0 | 15 | 15 |
| `proto_23` | 0 | 70 | 34 | 104 |
| `proto_24` | 109 | 0 | 35 | 144 |
| `proto_25` | 50 | 0 | 66 | 116 |
| `proto_26` | 0 | 0 | 122 | 122 |
| `proto_27` | 0 | 0 | 113 | 113 |
| `proto_28` | 0 | 0 | 42 | 42 |

## 3. 按 shim 分组 (仅 stub 表有分组标记)

| group | real | stub | missing | 小计 |
|---|---|---|---|---|
| `admin-gm` | 0 | 4 | 0 | 4 |
| `battle` | 0 | 88 | 0 | 88 |
| `guild` | 0 | 36 | 0 | 36 |
| `guild-shop` | 0 | 23 | 0 | 23 |
| `mail` | 0 | 70 | 0 | 70 |
| `maincity` | 0 | 48 | 0 | 48 |
| `partner` | 0 | 77 | 0 | 77 |
| `social` | 0 | 23 | 0 | 23 |
| `unknown` | 0 | 11 | 0 | 11 |

## 4. Phantom cmd (shim 有, 真实协议注册表查不到)

| code | handler | name |
|---|---|---|
| 13040 | `handle_stub` | stub-13040 |
| 16730 | `handle_stub` | stub-16730 |
| 16731 | `handle_stub` | stub-16731 |
| 21220 | `handle_stub` | stub-21220 |
| 23205 | `handle_stub` | stub-23205 |
| 23820 | `handle_stub` | stub-23820 |
| 23821 | `handle_stub` | stub-23821 |
| 24404 | `handlers::handle_welfare_24404` | welfare-24404 (w5-2 simple real) |
| 25102 | `handlers::handle_daily_quest_flag` | daily_quest_flag |
| 30001 | `handle_stub` | stub-30001 |
| 30002 | `handle_stub` | stub-30002 |
| 30100 | `handle_stub` | stub-30100 |
| 30101 | `handle_stub` | stub-30101 |
| 30102 | `handle_stub` | stub-30102 |

> 这些多为 shim 自造测试 cmd (如 heartbeat / role_list),
> **不能计入业务覆盖**.

## 5. 补齐入口

每条 missing 条目在 `mock_data/flash_gap_matrix.json` 的 `entries` 里,
已带 `req` / `reply` 字段规格 (类型 / 字段名 / 中文说明), 可直接照着写 handler.
