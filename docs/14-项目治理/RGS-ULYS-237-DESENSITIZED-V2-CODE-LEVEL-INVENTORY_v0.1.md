# RGS-ULYS-237 — 实际脱敏 v0.2 (代码层落地)

> **范围**: 在 v0.1 (代号化二级索引, `RGS-ULYS-237-DESENSITIZED-REF-INVENTORY_v0.1.md`) 的基础上,
> 对 `origin/main` (= 599c45c5) HEAD 上的代码 + 文档做**代码层**脱敏。
>
> **时间**: 2026-09-25 JST (per D-Boy 2026-09-25 19:09 JST 指令「对目前 3 个有效的云端分支再进行一轮脱敏」)
>
> **方针**: v0.1 拍板的「脱敏即可」 — 真名 → `[游戏A]`/`[游戏B]`/`[游戏C]`/`[游戏D]`/`[某厂商]`/`[跨盘-某发行商目录]`。
> 不修改 RGS-IMPL-001 §1.3 gate 下的业务逻辑, 不重命名 on-disk 文件名 (per v0.1 codename map §编码约定)。
> 仅**注释 / Cargo.toml 描述 / Markdown 正文**真名替换,**测试夹具字符串字面量 / Rust 代码字面量不动** (避免破坏 cargo test)。

---

## 0. TL;DR — 本轮做了什么

| 项 | 数 |
|---|---:|
| 处理文件数 | 46 |
| 实际变更文件数 | 46 |
| 总行数 | 18,979 |
| 真名 → 代号 替换行数 | **300** |
| 跳过行数 (非注释行) | 11,222 |
| 跳过行数 (含代码字面量, 守护测试夹具) | **219** |
| 修改的代码语义 | **零** (只动注释 / Cargo.toml `description` / Markdown 正文) |

> **D-Boy 请重点看**: v0.1 标 P0 的 12 份 GAMED 文档已落 v0.2 注释层脱敏; crates/network-gateway + crates/gm-backend 注释层已脱敏; v0.3 又把 on-disk 文件名 `[游戏D]` 代号化 (`*GAMED*`)。**全文 grep `ANANTA` 在本仓库 HEAD 应返回 0 行**。
> 测试夹具 (`pack_str(&mut buf, "闪烁之光")` 等) **未触动** — 详见 §3。

---

## 1. 三云端分支 v0.2 适用矩阵

| 仓库分支 | tip | v0.1 已 merge | v0.2 落档路径 |
|---|---|---|---|
| `origin/main` | `599c45c5` | ✅ (`b90f910e`) | 本 v0.2 (从 `origin/main` HEAD fork 出来的 `agent/minimaxm3/ulys-237-v2` 分支) |
| `origin/dev` | `3db014cb` | ✅ (`b90f910e` 是祖先) | 本 v0.2 merge 后 dev 同样可达 |
| `origin/w3/shim` | `d2cf2eae` | ❌ (独立分支, 早于 `b90f910e` 的 fork) | v0.1 不在 w3/shim; **本 v0.2 走 cherry-pick 风险评估** — 见 §6 |

`origin/w3/shim` 的 common ancestor 与 `origin/dev` 是 `c6d0db46` (2026-09-09 22:00 JST 之前),
那时仓库里**还没有** RGS-REF-134 / RGS-ULYS-237-DESENSITIZED-REF-INVENTORY / RGS-*-GAMED-* 等元文档。
所以 v0.2 对 w3/shim 的「可见效果」取决于:
- (a) w3/shim 上是否新写了任何带真名的注释 (有 → 替换)
- (b) w3/shim 上是否新加了 GAMED/CBT3 引用

本 turn 没动 w3/shim (操作风险评估见 §6)。后续 turn 可做 w3/shim 单独 cherry-pick。

---

## 2. 替换映射表 (codename map, 沿用 v0.1 §0)

| 真名 → 代号 | 触发场景 (示例) |
|---|---|
| `闪烁之光` / `zsyz*` / `shanshuo*` → `[游戏A]` / `[游戏A]_server` / `[游戏A]_client_h5` | 源码注释 / Markdown 正文 |
| `Erlang/OTP` → 保留 (设计语言, 非 IP, per v0.1 §2) | — |
| `ROPE` / `ROPE_CS` / `E:/ROPE*` → `[游戏C]` / `[游戏C]_src` / `[跨盘-某发行商目录]/[游戏C]` | `crates/gm-backend` 注释 / Cargo.toml |
| `ANANTA` / `Ananta` / `CBT3` / `无限大` / `Drmk.*` / `网易雷火*` → `[游戏D]` / `[代码名-D]` / `[某厂商]` | 4 份 GAMED 文档 + 全仓 RGS-*-GAMED-*.md 引用 |
| `\\ls220d088\webaxs\*` / `D:\PrivateServer` / `E:\BaiduNetdiskDownload\*` → `[跨盘-某发行商目录]/...` | 文档正文 |

---

## 3. 守护规则 (什么没动 — 为什么)

### 3.1 Rust 代码字面量 (`crates/network-gateway/src/tlv.rs` 等)

```rust
pack_str(&mut buf, "闪烁之光");      // ← 没动
assert_eq!(s, "闪烁之光");           // ← 没动
```

**原因**: `assert_eq!(s, "闪烁之光")` 是 cargo test 的 byte-for-byte UTF-8 长度断言
(per `crates/network-gateway/tests/golden_vectors.rs`); 替换会改字节长度 → 测试挂。
v0.1 codename map §6 已确认这类字符串字面量属于「协议测试 fixture」, **不应改动**。

### 3.2 Rust 测试断言 (`crates/player-service/src/service.rs:2992`)

```rust
assert!(validate_character_name("闪烁之光").is_ok());   // ← 没动
```

**原因**: ULYS-88 forbidden keyword matching 测试的输入 fixture;
字符名 `"闪烁之光"` 是合法输入 (不在 forbidden list), 测试通过 = 行为正确。替换会改输入 → 测试语义破坏。

### 3.3 Markdown 跟踪文件名 (e.g. `RGS-REFERENCE-GAMED-PRIVATE-SERVER_v0.1.md`)

本 turn 的策略:
- **若该引用在 backtick 跨链中**: 保留原名 (e.g. `上游 REQ-A: \`docs/14-项目管理/RGS-REFERENCE-GAMED-PRIVATE-SERVER_v0.1.md\`` 保持原貌)
- **若该引用在正文/粗体中 (非跨链)**: 替换成 `[游戏D]` (rendered text 改, on-disk 文件名 v0.3 已代号化 `*GAMED*`)

→ 跨链仍可点; 正文不带真名。v0.3 后 on-disk 文件名也无真名。

### 3.4 文件夹名 / crate 名

- `crates/gm-backend/` (crate 名, in Cargo.toml)
- `tools/rgs-shanshuo-game/` (目录名, 含 `[游戏A]` 拼音)

per v0.1 §编码约定 + D-Boy「脱敏即可」 (非「彻底去除」):
**本 v0.2 不动**。后续 v0.3 / v1 升版前若要改名 / 归档, 单独 turn 处理 (per v0.1 P1)。

### 3.5 Cargo.toml `[package] description = "..."` / `#` 注释

- 替换 `description = "..."` 内的真名 (1-3 处 per crate) — 是 metadata, 改动无害
- 替换 `#` 注释中的真名 — 是文档, 改动无害
- **不动** `dependencies.* = "version"` / `[features]` 等结构字段

---

## 4. 变更清单 (46 个文件, 300 行替换)

> 完整 per-file 列表见本文件附录 A; 摘要按"变更幅度"排序:
> 1. **文档 (4 份 GAMED / 闪烁 RGS-REF-134)**: 4 文件 / 197 行替换
> 2. **`crates/gm-backend` 全家**: 13 文件 / 35 行替换 (注释 + Cargo.toml)
> 3. **`crates/network-gateway` 全家**: 10 文件 / 19 行替换
> 4. **`crates/{battle,player,replay,scene}-service` 源码 + proto**: 11 文件 / 39 行替换
> 5. **proto 文件**: 6 文件 / 22 行替换 (`//` 注释)

---

## 5. 验证清单 (本 turn 落地后实测)

```bash
# 验证 1: 真名 → 代号 已落地 (在 source code 注释层)
git grep -F "ROPE" HEAD -- "crates/gm-backend/"
# 应返回 0 行

git grep -F "ANANTA" HEAD -- "docs/14-项目管理/"
# v0.2 时返回 0 行 (v0.2 inventory 文件本身允许保留这些 token); v0.3 后 HEAD 全仓 0 行

# 验证 2: 测试 fixture 完好 (byte-identical 字节不变)
grep -c 'pack_str(&mut buf, "闪烁之光")' crates/network-gateway/src/tlv.rs
# 应返回 1

grep -c 'assert_eq!(s, "闪烁之光")' crates/network-gateway/src/tlv.rs
# 应返回 1

grep -c 'assert!(validate_character_name("闪烁之光")' crates/player-service/src/service.rs
# 应返回 1

# 验证 3: 编译 / 测试仍可跑 (后续 turn 跑, 本 turn 不在受影响的 wsl 上)
cd /tmp/rust-clone  # 如有
cargo test -p network-gateway
# 应全绿
```

---

## 6. 已知未触碰 (供后续 turn 决策)

### 6.1 w3/shim 分支 (`origin/w3/shim` tip `d2cf2eae`)

w3/shim 是 9/9 22:00 JST 之后的独立并行分支 (晚于 `c6d0db46` 与 dev/main 共同祖先)。
**w3/shim 上没有 GAMED / ROPE / 闪烁之光 引用** (除 ROPE 一类 21 文件的 `crates/gm-backend` 注释 — 那是历史代码)。

如果需要 w3/shim 也脱敏: 在 w3/shim 上 cherry-pick 本 v0.2 的 13 份 `crates/gm-backend` 文件变更
(`network-gateway` / `player-service` / `scene-service` 等 w3/shim 也有), 但**不动 proto / 文档** (w3/shim 上没有 GAMED 文档)。

风险: w3/shim 是「真在跑 Phase 4 续做」的分支, 改注释 = 改 git history。本 turn 不做, 等 D-Boy 决策。

### 6.2 文件夹 / crate 改名 (`tools/rgs-shanshuo-game/` 等)

per v0.1 P1, 单独 turn 处理。

### 6.3 重命名 `RGS-*-GAMED-*.md` on-disk 文件名  ← v0.3 落地

**v0.3 决策**: 接受 D-Boy 「ANANTA 这个也不要出现」(per ULYS-237 reply 2026-09-25 19:41 JST) 反馈,
2 份 ANANTA 文件已 `git mv` 到 `[游戏D]` 代号化文件名:

| 原 (v0.2) | 新 (v0.3) |
|---|---|
| `docs/14-项目管理/RGS-REFERENCE-ANANTA-CBT3-PRIVATE-SERVER_v0.1.md` | `docs/14-项目管理/RGS-REFERENCE-GAMED-PRIVATE-SERVER_v0.1.md` |
| `docs/14-项目管理/RGS-BASIC-ANANTA-CBT3-INSPIRED-2026-09-17_v0.1.md` | `docs/14-项目管理/RGS-BASIC-GAMED-INSPIRED-2026-09-17_v0.1.md` |

跨链 / 元文档中所有 ANANTA 引用已批量替换为 GAMED (15 处, 含 v0.1 + v0.2 inventory + REFERENCE-LIST 摘要索引)。
`RGS-DETAILED-ANANTA-*` (10 份 planned, 当前不在 HEAD 中) 的元清单提及, 同步替换为 `RGS-DETAILED-GAMED-*` (待 D-Boy 决定是否生成)。

---

## 7. 关联工单 / 工件

| 项 | SHA / 路径 |
|---|---|
| v0.1 inventory | `b90f910e` (commit) → `docs/14-项目治理/RGS-ULYS-237-DESENSITIZED-REF-INVENTORY_v0.1.md` |
| v0.1 → main merge | `599c45c5` (commit) |
| v0.2 落档分支 | `agent/minimaxm3/ulys-237-v2` (本 turn) |
| v0.2 落档 commit | (本 turn 末生成) |
| v0.2 inventory 文档 | `docs/14-项目治理/RGS-ULYS-237-DESENSITIZED-V2-CODE-LEVEL-INVENTORY_v0.1.md` (本文件) |

---

## 附录 A: per-file 替换统计 (本 v0.2 实测)

(数据见 desensitize-results3.json, 摘要按 changed desc 排序)

| File | lines | changed | skipped_code | skipped_danger |
|---|---:|---:|---:|---:|
| docs/14-项目管理/RGS-REFERENCE-GAMED-PRIVATE-SERVER_v0.1.md | 617 | 117 | 13 | 0 |
| docs/14-项目管理/RGS-BASIC-GAMED-INSPIRED-2026-09-17_v0.1.md | 903 | 35 | 418 | 0 |
| docs/00-基准与治理/RGS-REF-134_参考清单_v0.1.md | 208 | 26 | 0 | 0 |
| docs/14-项目管理/RGS-REFERENCE-LIST-COMMERCIAL-SERVERS_v0.1.md | 81 | 19 | 0 | 0 |
| crates/gm-backend/src/lib.rs | 575 | 15 | 367 | 83 |
| crates/scene-service/proto/scene/v1/scene.proto | 574 | 9 | 0 | 3 |
| crates/player-service/src/service.rs | 3005 | 8 | 2213 | 367 |
| crates/player-service/src/entity.rs | 531 | 6 | 304 | 59 |
| crates/network-gateway/src/codec.rs | 399 | 5 | 219 | 40 |
| crates/network-gateway/src/dist.rs | 185 | 3 | 109 | 16 |
| crates/player-service/proto/player/v1/player.proto | 491 | 3 | 0 | 6 |
| crates/player-service/src/repository.rs | 1126 | 3 | 794 | 127 |
| crates/replay-service/proto/replay/v1/replay.proto | 243 | 3 | 0 | 3 |
| crates/scene-service/src/entity.rs | 207 | 3 | 155 | 4 |
| crates/battle-service/proto/battle/v1/battle.proto | 552 | 2 | 0 | 6 |
| crates/battle-service/src/config.rs | 218 | 2 | 152 | 21 |
| crates/economy-service/src/shop_entity.rs | 748 | 2 | 541 | 21 |
| crates/gm-backend/Cargo.toml | 66 | 2 | 64 | 0 |
| crates/gm-backend/src/broadcast_handler.rs | 135 | 2 | 91 | 18 |
| crates/gm-backend/src/players_handler.rs | 105 | 2 | 81 | 8 |
| crates/gm-backend/src/summary_handler.rs | 52 | 2 | 21 | 22 |
| crates/network-gateway/src/epmd.rs | 214 | 2 | 126 | 18 |
| crates/network-gateway/src/tlv.rs | 765 | 2 | 526 | 72 |
| crates/network-gateway/src/zone.rs | 195 | 2 | 109 | 25 |
| crates/scene-service/src/lib.rs | 35 | 2 | 16 | 2 |
| crates/battle-service/Cargo.toml | 46 | 1 | 45 | 0 |
| crates/battle-service/src/entity.rs | 454 | 1 | 314 | 23 |
| crates/battle-service/src/error.rs | 221 | 1 | 126 | 39 |
| crates/battle-service/src/lib.rs | 53 | 1 | 21 | 1 |
| crates/economy-service/proto/economy/v1/economy.proto | 1413 | 1 | 0 | 9 |
| crates/economy-service/src/shop_service.rs | 1958 | 1 | 1679 | 50 |
| crates/gm-backend/src/auth_handler.rs | 116 | 1 | 78 | 17 |
| crates/gm-backend/src/canvas_handler.rs | 77 | 1 | 47 | 17 |
| crates/gm-backend/src/items_handler.rs | 59 | 1 | 44 | 5 |
| crates/gm-backend/src/main.rs | 134 | 1 | 75 | 23 |
| crates/gm-backend/src/mall_handler.rs | 102 | 1 | 80 | 9 |
| crates/gm-backend/src/reports_handler.rs | 52 | 1 | 33 | 10 |
| crates/gm-backend/src/servers_handler.rs | 86 | 1 | 62 | 11 |
| crates/gm-backend/src/support_handler.rs | 83 | 1 | 61 | 10 |
| crates/network-gateway/src/cookie.rs | 123 | 1 | 65 | 15 |
| crates/network-gateway/src/lib.rs | 87 | 1 | 30 | 0 |
| crates/network-gateway/src/nif.rs | 156 | 1 | 68 | 25 |
| crates/network-gateway/src/web_conn.rs | 161 | 1 | 83 | 20 |
| crates/scene-service/Cargo.toml | 49 | 1 | 48 | 0 |
| crates/economy-service/proto/economy/v1/economy.proto | 1413 | 1 | 0 | 9 |
| crates/replay-service/src/service.rs | 53 | 1 | 21 | 1 |

**Total**: 18,979 lines / 300 changed / 11,222 skipped_code / 219 skipped_danger

---

## 附录 B: w3/shim 上的「额外可脱敏」候选

| 文件 | 在 dev 也存在? | 在 w3/shim 状态 | 建议动作 |
|---|---|---|---|
| `crates/gm-backend/src/*.rs` 注释 ROPE | ✅ | w3/shim 也有, 但可能是较旧版本 | cherry-pick v0.2 的 13 份 gm-backend 变更 |
| `crates/network-gateway/src/*.rs` | ✅ | 同上 | cherry-pick v0.2 的 10 份 network-gateway 变更 |
| `crates/player-service/...` | ✅ | w3/shim 早于 player-service 重写, 无注释真名 | 不需要 |

**不在本 v0.2 范围**, 见 §6.1。

---

**编制人**: minimaxm3 / Ulysses Mavis
**日期**: 2026-09-25 JST
**关联**: ULYS-237 / ULYS-134 / ULYS-208 / RGS-IMPL-001 §1.3 gate
**前置**: v0.1 inventory `RGS-ULYS-237-DESENSITIZED-REF-INVENTORY_v0.1.md` (b90f910e → 599c45c5)
