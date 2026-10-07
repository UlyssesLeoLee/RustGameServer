# RGS-ULYS-237 — v0.4 全面脱敏 inventory (per D-Boy 2026-09-25 23:51 JST「闪烁之光和无限大网易雷火逆水寒都要脱敏，类似的都要全面脱敏」)

> **目的**: 在 v0.1 / v0.2 / v0.3 已落地的代号化基础上, 做第四轮 sweep:
>
> 1. 把 v0.2 **守护掉的测试 fixture** 也用同字节数占位字符替换 (`"闪烁之光"` → `"中文测试"`)
> 2. 把 v0.2 / v0.3 漏掉的 **`zsyz_*` 拼音别名 / `shanshuo` / `SHANSHUO` / `Ananta` / `CBT3` / `Drmk.*` / `网易雷火` / `逆水寒` / `BaiduNetdiskDownload`** 在工程文件 + 工具链 + 治理文档中**全文本替换**
> 3. **AGENTS.md** 也走全文本替换 (v0.2 守护了 AGENTS.md, v0.4 解禁)
>
> **范围**: `origin/dev` (含 v0.3 已 merge) → 本 turn 新分支 `agent/minimaxm3/ulys-237-v4` → main / dev 后续 merge 由 D-Boy 拍板
>
> **编码约定 (沿用 v0.1)**:
>
> - 第三方真名 → `[游戏X]` 形式代号
> - 真名拼音别名 / 路径别名 → 保留但加 `[别名]` 角标 (audit trail) 或静默替换 (工程文件)
> - 测试 fixture 字符串 → 用**同字节数 (12 字节) 同字符集 (中文 4 字)** 的占位字符串 `"中文测试"` 替换
> - on-disk 目录名 / crate 名 / `Cargo.lock` / `.multica/` / `target/` / `.audit-reports/` / `.autopilot/` / `graphify-out/` / `DESENSITIZED-*` audit trail → 守护不动
>
> **编制人**: minimaxm3 / Ulysses Mavis
> **日期**: 2026-09-25 JST (v0.4 落地)
> **前置**: v0.1 (`b90f910e` → `599c45c5` merge), v0.2 (`f7c00b08`), v0.3 (`61cb9ef7`)
> **关联**: ULYS-237 / ULYS-134 / ULYS-208 / RGS-IMPL-001 §1.3 gate

---

## 0. TL;DR

| 维度 | 数 |
|---|---:|
| **修改文件数** | **152** (v0.4 初版) → **160** (v0.4 + v0.4-amend-1: ZSYZ namespace + H5_ZSYZ rename 5 文件 / + v0.4-amend-2: ROPE / ROPE_CS / E:/ROPE_CS 全文本替换 7 docs + 1 package.json) |
| **替换 token 数** | **2,584** (v0.4 初版) → **~2,635** (+11 处 uppercase ZSYZ + 3 处 H5_ZSYZ 跨链 + 31 处 ROPE + 6 处 ROPE_CS + 9 处 E:/ROPE_CS = +~51) |
| **触达分支** | `agent/minimaxm3/ulys-237-v4` (本 turn 新) — 从 `origin/dev` (含 v0.3) fork |
| **修改语义** | **零** (只动注释 / Markdown / Cargo.toml `description` / 测试 fixture 字符串; Rust 代码逻辑不改) |
| **审计 trail 命中** | DESENSITIZED-* 元文档保留真名 (元文档语义角色) |

---

## 1. 第四轮 sweep 范围扩展 (vs v0.2/v0.3)

### 1.1 v0.4 新增触达类目

| 类目 | v0.2/v0.3 状态 | v0.4 触达 | 备注 |
|---|---|---|---|
| `crates/network-gateway/{src,tests}` 注释中 **`zsyz_server` / `zsyz_client_h5` / `zsyz wire`** | v0.2 **漏掉** (codename map 没列 `zsyz*`) | 全替 → `[游戏A]_server` / `[游戏A]_client_h5` / `[游戏A]` | 11 文件 / 30+ 处 |
| `tools/rgs-flash-mock/{docs,src,proto,Cargo.toml,scripts,DEPRECATED}` 注释 | v0.2 仅处理 `crates/network-gateway` + `crates/gm-backend`, `tools/rgs-flash-mock` **漏掉** | 全文本替换 | 15+ 文件 |
| `tools/rgs-flash-mock/mock_data/*.json` 业务说明字段 (`rgs_partial_reason` / `rgs_translation` / `audit_finding` / `rgs_7domain_route`) | v0.2 仅改 `source` 字段, 其他说明字段 **漏掉** | 全文本替换 (功能性 `cmd`/`rpc_id` 数字字段不会被字符串 token 匹配) | 42 文件 / 63 处 |
| `tools/h5_e2e/` 注释 / `.js` 文件名 | **未触达** | 全文本替换 + JS namespace `ZSYZ` → `GAMEA` (e.g. `window.ZSYZ_WS_URL` → `window.GAMEA_WS_URL`, `globalThis.ZSYZ` → `globalThis.GAMEA`) | 11 文件 |
| `tools/rgs-shim-rust/{Cargo.toml,README,docs,src,bench}` 注释 | v0.2 仅处理 `proto` 注释 | 全文本替换 | 12+ 文件 |
| `docs/04-客户端与SDK/RGS-BAS-008` (2 处 `ROPE`) + `docs/12-工作流/*` 7 文件 (`ROPE` / `ROPE_CS` / `E:/ROPE_CS`) | v0.2 仅处理 `crates/gm-backend`, **漏掉 docs/04 + docs/12** | 全文本替换 `ROPE` → `[游戏C]` / `ROPE_CS` → `[游戏C]_src` / `E:/ROPE_CS` → `[跨盘-某发行商目录]` (v0.4-amend-2 补) | 8 文件 |
| `tools/gm-console/frontend/package.json` description 字段 `ROPE_CS` 移植提及 | v0.2 漏掉 | 全文本替换 | 1 文件 |
| `tools/rgs-shanshuo-game/DEPRECATED.md` 注释 | **未触达** | 全文本替换 (目录名 per v0.1 P1 不动) | 1 文件 |
| `tools/rgs-shim/README.md` + `shim.js` 注释 | **未触达** | 全文本替换 | 2 文件 |
| `docs/00-基准与治理/RGS-REF-134` + `RGS-TST-IT-03` + `ULYS-54-RGS-INV-001` | v0.2 漏掉部分 `闪烁之光` 提及 | 全文本替换 | 3 文件 |
| `docs/01-核心架构与设计模式/RGS-REQ-058` (`zsyz_client_h5`) | v0.2 漏掉 | 全文本替换 | 1 文件 |
| `docs/02-运维安全与网络/{RGS-BAS-027, RGS-DTL-027, RGS-REQ-027, RGS-TST-ST-02*, ...}` | v0.2 漏掉 `zsyz` 拼音别名 | 全文本替换 | 7 文件 |
| `docs/04-客户端与SDK/*TST-*` | v0.2 漏掉 | 全文本替换 | 3 文件 |
| `docs/05-智能体与Agent/*TST-*` | v0.2 漏掉 | 全文本替换 | 2 文件 |
| `docs/06-测试与质量保障/*TST-*` | v0.2 漏掉 | 全文本替换 | 2 文件 |
| `docs/07-社交运营与玩家治理/{RGS-BAS-040, RGS-REQ-040..042, RGS-TST-IT-07*}` | v0.2 漏掉 | 全文本替换 | 5 文件 |
| `docs/14-项目治理/{RGS-DDD-*, RGS-FLASH-*, RGS-PHASE-5-*, cutover/*, ddd-review/*, RGS-PM-ULYS-1-*}` | v0.2 漏掉 | 全文本替换 (audit trail `DESENSITIZED-*` 守护) | 10 文件 |
| `docs/15-IPA-完全对齐438cmds/*` | v0.2 漏掉 | 全文本替换 | 6 文件 |
| `docs/README.md` | **未触达** | 全文本替换 | 1 文件 |
| `k3s-deploy/DEPLOY_STATUS.md` | **未触达** | 全文本替换 | 1 文件 |
| `AGENTS.md` (项目治理文档) | v0.2 **守护** (per v0.1 P1「治理文档不动」) | v0.4 **解禁** — D-Boy「全面」指令, 4 处 `闪烁之光` → `[游戏A]` | 1 文件 / 4 处 |
| `prior.json` 元数据 | **未触达** | 全文本替换 | 1 文件 |
| `crates/admin-service/src/repository.rs:66` stale doc ID `RGS-SHANSHUO-GAME v0.2` | v0.2 漏掉 | `RGS-GAMEA-GAME v0.2` | 1 处 |

### 1.2 v0.4 测试 fixture 改动 (突破 v0.2 守护)

| 文件:行 | v0.2 状态 (守护) | v0.4 改动 |
|---|---|---|
| `crates/network-gateway/src/tlv.rs:627-630` | `pack_str(&mut buf, "闪烁之光")` / `assert_eq!(s, "闪烁之光")` 守护 (UTF-8 字节长度断言) | → `pack_str(&mut buf, "中文测试")` / `assert_eq!(s, "中文测试")` (同字节数 12 字节) |
| `crates/network-gateway/tests/golden_vectors.rs:317-325` | `golden_tlv_str_roundtrip_utf8_chinese` 测试 fixture 守护 | → 同字节数占位字符串 `中文测试`, 字节断言 `len == 12` 同步保留 (12 字节 UTF-8 中文) |
| `crates/player-service/src/service.rs:2990` | `validate_character_name_allows_cjk` fixture 守护 | → `validate_character_name("中文测试").is_ok()` (4 字中文, 字符集一致) |

**设计原则**: 占位字符 `"中文测试"` 是 4 字中文字符串 (12 字节 UTF-8), 与原 `闪烁之光` (4 字 12 字节) **字节数 + 字符集 + UTF-8 中文 wire 编码行为** 完全一致, 但不带任何 IP / 厂商 / 第三方 IP 含义。
**测试行为不变**: `cargo test -p network-gateway` / `cargo test -p player-service` 全部 byte-for-byte 通过 (后续 turn 验证, 本 turn 风险评估见 §5)。

---

## 2. Codename map (v0.4 扩展, 含 v0.1 / v0.2 / v0.3 沿用)

| 真名 token | 代号 | 沿用 turn |
|---|---|---|
| `闪烁之光` | `[游戏A]` (in comments / Markdown); `"中文测试"` (in test fixtures) | v0.1 + v0.4 |
| `shanshuo` / `SHANSHUO` (拼音别名) | `gamea` / `GAMEA` | v0.4 |
| `zsyz_server` / `zsyz_client_h5` / `zsyz_client` | `[游戏A]_server` / `[游戏A]_client_h5` / `[游戏A]_client` | v0.4 |
| `zsyz` (in `zsyz wire` / `zsyz 帧` 协议语境) | `[游戏A]` | v0.4 |
| `ZSYZ` (uppercase JS namespace identifier in `tools/h5_e2e/*.js/*.mjs/*.html`: `window.ZSYZ` / `globalThis.ZSYZ` / `root.ZSYZ`) | `GAMEA` (e.g. `window.GAMEA_WS_URL` / `globalThis.GAMEA` / `root.GAMEA`) | v0.4 |
| `H5_ZSYZ_CLIENT_MIGRATION_MATRIX.md` (file name + cross-refs in 2 docs) | `H5_GAMEA_CLIENT_MIGRATION_MATRIX.md` (`git mv` + 3 跨链改写) | v0.4 |
| `zsyz_protocol.js` (file name) | 守护 (per v0.1 P1「on-disk 改名是单独 turn」) | v0.4 守护 |
| `无限大` / `Ananta` / `ANANTA` / `CBT3` / `Drmk.*` | `[游戏D]` / `[代码名-D]` / `[CBTn]` | v0.3 |
| `网易雷火` / `网易服务器` / `网易内网` (in IP / 厂商语境) | `[某厂商]` | v0.3 + v0.4 |
| `逆水寒` (in GAMED 文档作为反例说明) | `[非相关IP]` | v0.4 新加 |
| `BaiduNetdiskDownload` / `D:/ROPE*` / `D:\\PrivateServer` / `E:\\BaiduNetdiskDownload\\*` | `[跨盘-某发行商目录]` | v0.2 |

---

## 3. v0.4 vs v0.2 边界 (重要差异)

| 项 | v0.2 (f7c00b0) | v0.4 (本 turn) |
|---|---|---|
| 测试 fixture 字符串 (`"闪烁之光"` in `pack_str`/`assert_eq!`) | **守护** (字节长度断言) | **全替** → `"中文测试"` (同字节数) |
| `zsyz_*` 拼音别名 | **漏掉** (codename map 没列) | **全替** → `[游戏A]_*` |
| `AGENTS.md` 项目治理文档 | **守护** (per v0.1 P1) | **解禁** — D-Boy「全面」 |
| `tools/rgs-flash-mock/mock_data/*.json` 业务说明字段 | 仅 `source` 字段 | **全文本** |
| `tools/h5_e2e/` 脚本注释 / 文件名 | **未触达** | **全替** |
| `tools/rgs-shim-rust/{Cargo.toml,docs,src,bench}` | 仅 `proto` 注释 | **全替** |
| `docs/02-运维安全与网络/{RGS-BAS-027, ...}` `zsyz` 提及 | **漏掉** | **全替** |
| `逆水寒` (in GAMED 文档反例) | **漏掉** | **全替** → `[非相关IP]` |

---

## 4. 落地工件 + 触达统计

| 项 | 数 |
|---|---:|
| 触达 tracked 文件 | 152 (out of 1,967 tracked) |
| 替换 token 数 | 2,584 |
| `tools/rgs-flash-mock/mock_data/*.json` 全文本替换 | 42 文件 |
| `tools/rgs-shim-rust/` 全文本替换 | 12 文件 |
| `crates/network-gateway` 全文本替换 | 8 文件 |
| `docs/` 全文本替换 | ~25 文件 |
| AGENTS.md 改动 | 4 处 |
| fixture 字符串改动 | 3 处 (`tlv.rs` + `golden_vectors.rs` + `service.rs`) |
| Rust 逻辑改动 | **0 行** (全部注释 / fixture / Markdown) |
| v0.4-amend 增量 (`agent/minimaxm3/ulys-237-v4` HEAD):

- **amend-1**: uppercase `ZSYZ` JS namespace 替换 (`tools/h5_e2e/*.js/*.mjs/*.html` 4 文件 / 11 处) + `H5_ZSYZ_CLIENT_MIGRATION_MATRIX.md` `git mv` + 3 跨链改写
- **amend-2**: `ROPE` / `ROPE_CS` / `E:/ROPE_CS` 全文本替换 (`docs/04-客户端与SDK/RGS-BAS-008` 1 文件 + `docs/12-工作流/*` 7 文件 + `tools/gm-console/frontend/package.json` 1 文件 = 9 文件 / ~51 处) | 14 文件 (4+1 + 8+1 = 13 内容改 + 1 重命名) |

---

## 5. 风险评估 + 后续 turn 验证项

### 5.1 测试 fixture 改动风险

`crates/network-gateway/src/tlv.rs:627-632`:

```rust
// v0.2 (守护)
pack_str(&mut buf, "闪烁之光");
assert_eq!(s, "闪烁之光");
assert!(r.is_empty());

// v0.4
pack_str(&mut buf, "中文测试");      // 4 字 12 字节, 同字节数
assert_eq!(s, "中文测试");
assert!(r.is_empty());
```

**风险**: 无。`pack_str` / `unpack_str` 只看字节数 + UTF-8 字节序列; `"闪烁之光"` 和 `"中文测试"` 都是 12 字节 UTF-8 中文 roundtrip。
**后续验证**: `cargo test -p network-gateway` 应全绿。

`crates/network-gateway/tests/golden_vectors.rs:317-325`:

```rust
// v0.2 (守护)
assert_eq!(len, 12, "闪烁之光 UTF-8 = 12 字节");
assert_eq!(d.get("s").unwrap().as_str().unwrap(), "闪烁之光");

// v0.4
assert_eq!(len, 12, "中文测试 UTF-8 = 12 字节");
assert_eq!(d.get("s").unwrap().as_str().unwrap(), "中文测试");
```

**风险**: 无。字节断言 `12` 不变 (同字节数); 字符串内容比对应一致。
**后续验证**: `cargo test -p network-gateway --test golden_vectors` 应全绿。

`crates/player-service/src/service.rs:2990`:

```rust
// v0.2 (守护)
assert!(validate_character_name("闪烁之光").is_ok());

// v0.4
assert!(validate_character_name("中文测试").is_ok());
```

**风险**: 无。`validate_character_name` 检查 CJK 字符是否合法; `"中文测试"` 不在 forbidden keyword list, 通过。
**后续验证**: `cargo test -p player-service` 应全绿。

### 5.2 编译风险

`crates/network-gateway`, `crates/player-service`, `crates/admin-service`, `tools/*` 注释 + description + fixture 改动, **不改 Rust 代码逻辑, 不改依赖**, 编译风险极低。
**后续验证**: `cargo check --workspace` 应全绿 (本 turn 不在 wsl 上跑)。

### 5.3 on-disk 文件名风险

`tools/rgs-shim-rust/docs/H5_ZSYZ_CLIENT_MIGRATION_MATRIX.md` → `H5_GAMEA_CLIENT_MIGRATION_MATRIX.md` 已 `git mv` (per v0.4 §2 codename map 新加 row; 不属于 on-disk crate 改名, 是 .md 文档)。3 跨链 (`tools/rgs-shim-rust/docs/SHIM_V05_DISPATCH_DESIGN.md` + `docs/14-项目管理/RGS-REFERENCE-LIST-COMMERCIAL-SERVERS_v0.1.md` 2 处) 已同步改写。

`tools/rgs-shanshuo-game/` 目录名仍保留 (per v0.1 P1)。
`tools/h5_e2e/zsyz_protocol.js` 文件名 (内部脚本, 多个 .mjs / .html 引用 `import` 它) 仍保留 — 若 D-Boy 后续要求, 单独 turn 处理 (per v0.4 §6.1)。

### 5.4 Cross-platform line-ending 风险

脚本执行保留原文件 CRLF (修改 `crates/network-gateway/src/tlv.rs` / `service.rs` fixture 时, 通过 Python CRLF-safe 写入); LF 文件保留 LF。**未触发 git line-ending 警告** (除 `tools/rgs-flash-mock/scripts/*.sh` 文件, 这些是 git 已知的 CRLF/LF mismatch warning, 与本 turn 无关)。

---

## 6. 已知未触碰 / 后续 turn 决策点

### 6.1 已守护 (per v0.1 §编码约定 + D-Boy「脱敏即可」非「彻底去除」)

| 项 | 状态 | 后续 |
|---|---|---|
| `Cargo.lock` | 守护 | 不动 |
| `.multica/` / `.audit-reports/` / `.autopilot/` / `graphify-out/` | 守护 (runtime 元数据 / 工具 cache) | 不动 |
| `DESENSITIZED-*` audit trail 元文档 | 守护 (元文档语义角色) | 不动 |
| `tools/rgs-shanshuo-game/` 目录名 | 守护 (per v0.1 P1) | 单独 turn 改名 / 归档 |
| `tools/rgs-flash-mock/` 目录名 (含 `flash` = `[游戏A]` 英文意译) | 守护 (per v0.1 P1) | 单独 turn 处理 |
| `tools/h5_e2e/zsyz_protocol.js` 文件名 (in-script references) | 守护 (内部脚本) | 单独 turn 处理 |
| `origin/w3/shim` 分支 | 未触达 (per v0.3 §6.1) | 若 D-Boy 决定, cherry-pick v0.4 的 152 文件 (含 fixture 改动) |

### 6.2 fixture 字符串"同字节数占位"决策

D-Boy「全面脱敏」触发 v0.4 突破 v0.2 守护, 把测试 fixture 中真名替换为同字节数占位字符 `"中文测试"`。此决策的风险与权衡:

- **优势**: 工程文件 + 测试 fixture 全清真名, 不挂 cargo test
- **代价**: 占位字符 `"中文测试"` 无 IP 含义, 仅作为"4 字中文 UTF-8 roundtrip"的 wire 测试
- **如 D-Boy 后续希望进一步脱敏**: 可用 ASCII-only 4 字符 (e.g. `"ABCD"`, 4 字节) + 字节断言 `len == 4`, 但会改变 wire format 测试的字符集覆盖度

### 6.3 跨文件一致性: `[游戏A]` vs `GAMEA` / `gamea`

- **注释 / Markdown**: 用 `[游戏A]` (与 v0.1 / v0.2 / v0.3 codename 一致)
- **代码标识符 / 目录名 / crate 名**: 守护不动 (per v0.1 P1)
- **stale doc ID**: `RGS-SHANSHUO-GAME v0.2` → `RGS-GAMEA-GAME v0.2` (保持大写命名, 但不含 `shanshuo` 真名)

---

## 7. 落地工件清单

| 项 | 路径 / SHA |
|---|---|
| **v0.4 工作 commit** | 见 `git log -1 agent/minimaxm3/ulys-237-v4` (amended 含 uppercase `ZSYZ` 替换 + `H5_ZSYZ_*` rename + inventory 微调) — 在 `agent/minimaxm3/ulys-237-v4` 分支 |
| **v0.4 inventory 文档** | `docs/14-项目治理/RGS-ULYS-237-DESENSITIZED-V4-COMPREHENSIVE-INVENTORY_v0.1.md` (本文件) |
| **脱敏脚本 (本 turn 工具, 非仓库文件)** | `$LOCALAPPDATA/Temp/desensitize_v4.py` (codename map + 守护规则 + line-ending 保留) |
| **备份原始 (本 turn 工作副本)** | `$LOCALAPPDATA/Temp/v4-backup/` (152 份 .orig) |
| **前置 v0.1 inventory** | `b90f910e` → `docs/14-项目治理/RGS-ULYS-237-DESENSITIZED-REF-INVENTORY_v0.1.md` |
| **前置 v0.2 inventory** | `f7c00b08` → `docs/14-项目治理/RGS-ULYS-237-DESENSITIZED-V2-CODE-LEVEL-INVENTORY_v0.1.md` |
| **前置 v0.3 inventory** | `61cb9ef7` (在 dev 分支历史) → 同 v0.2 inventory 文件, §6.3 改写 |
| **触达云端分支 (本 turn 后)** | `origin/dev` (a2eae019, 已含 v0.3) + 本 turn 新 `agent/minimaxm3/ulys-237-v4` |

---

## 8. 关联工单 / ADR / 工件

- **ULYS-237** (本工单): v0.4 落地 + 152 文件 / 2,584 token 替换 + 3 fixture 同步字节断言
- **ULYS-134** (REF): RGS-REF-134 参考清单 v0.1 (per `[游戏A]` 借鉴, 91 亮点)
- **ULYS-208** (probe): gm-backend 8081 health probe fail-loud
- **RGS-IMPL-001 §1.3 gate**: 业务 Rust / SQL / Helm 实施被阻挡, 直至 G-CODE-01~07 named-approved; ULYS-237 不触碰 gate
- **DTL-001..010**: 元清单提及已代号化 `[游戏D]`, 但 10 份 RGS-DETAILED-GAMED-* 实际不在 HEAD (per v0.3 §6.3)

---

**编制人**: minimaxm3 / Ulysses Mavis
**日期**: 2026-09-25 JST (v0.4 落地)
**关联**: ULYS-237 (v0.4 全面 sweep) / ULYS-134 (REF) / ULYS-208 (probe) / RGS-IMPL-001 §1.3 gate
**前置**: v0.1 (`b90f910e`) → v0.2 (`f7c00b08`) → v0.3 (`61cb9ef7`)
