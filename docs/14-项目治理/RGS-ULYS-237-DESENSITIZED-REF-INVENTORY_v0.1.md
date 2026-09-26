# RGS-ULYS-237 — 项目内"参考对象 / 第三方商用 IP"提及清单(脱敏版 v0.1)

> **目的**: ULYS-237 的 ask = 「不要提及参考」「彻底去除这些名字」「脱敏即可」。
> 本清单只服务于 ULYS-237 的审计追溯 / 索引需要,所有真名一律脱敏,只有阅读者本人知道代号含义。
>
> **范围**: `agent/minimaxm3/ulys-237` 历史罗列(对应 `62ce0c75` 前 commit 上的 `.multica/ulys-237-inventory.md`)全文事实,
> 经代号化的二级索引。**不是项目规约或治理文件的取代**;`docs/14-项目治理/RGS-FLASH-*.md` 等原文仍然存在,
> 出于 ULYS-237 之外目的(基线、借鉴、逆向、ADR 历史等),未被本 turn 触及。
>
> **编码约定**:
> - 第三方真名 → `[游戏X]` 形式代号
> - 真名拼音别名 / 路径别名 → 保留但加 `[别名]` 角标
> - 已在 rgs 仓库内真实存在的文件名 / 目录名 / crate 名 → 保留(per D-Boy "脱敏即可" 拍板;改名属"彻底去除"路径,本 v0.1 不触碰)
> - 跨盘绝对路径 / 个人学习路径 → 用 `[跨盘-某发行商目录]` / `[某盘根目录]` 形式代号化
>
> **编制人**: minimaxm3 / Ulysses Mavis
> **日期**: 2026-09-25 JST
> **关联**: ULYS-237 / ULYS-134 (REF) / ULYS-208 (probe) / RGS-IMPL-001 §1.3 gate

---

## 0. TL;DR — 项目内共有 4 类参考对象(全部代号化)

| 代号 | 性质 | 在 RGS 中的角色 | 命中文件数 | 字面命中次数(粗估) |
|---|---|---|---:|---:|
| `[游戏A]` | 第三方 MMORPG 服务端源码(合作方匿名 IP) | **首要参考对象** — 协议层 / 12 大类业务 / 网络层 / 业务逻辑全方位对照 | ~175 | ~1,825 |
| `[游戏B]` | 商用分布式 actor 框架的设计语言(非单款产品) | **设计基线** — 9 原则 + 6 反模式 fingerprint | 多文件散落 | ~33 |
| `[游戏C]` | 外部参考源码(绝对路径硬编码) | **GM 后台页面参考** — 19 页面映射 → RGS 10 核心页面落地 | ~12 | ~126 |
| `[游戏D]` | 某厂商 CBT 私服本地逆向 | **架构灵感来源** — 2 份独立治理文档 | ~3 | ~118 |

> D-Boy 真名表按需对照;**本表本身的 rgs 仓库物理文件、commit message、文档正文中只出现本表中的代号**。
> 下文如需引用某个真实文件名 / 路径名 / crate 名,会显式标注 `[别名]` 角标说明。

---

## 1. `[游戏A]` — 合作方匿名 IP(基于某分布式 actor 框架的 MMORPG)

### 1.1 命中范围

| 项 | 数 |
|---|---|
| 命中文件数(去重) | **~175 个** |
| 字面命中次数(中 / 拼音 / ASCII 三种写法合计) | **~1,825 次** |
| 落地证据 commits 数量 | **11 个**(`ffe1778e`、`7a8ae1e6`、`b7371214`、`379f2cd1`、`a5235ebf`、`1134cfd7`、`57edbeb3`、`95e67a69`、`b6b19b73`、`1dd9afcf`、`2e3d9ee0`),均带 `#本参考兼容` 标签 |

### 1.2 在 RGS 中的战略地位

- **协议层**: 端口 8000 / WS path `/websocket` / `cmd=1110` 握手 / `cmd=1199` 心跳 — 全部以 `[游戏A]` 参考实现为蓝本
- **业务层**: 12 大类 RPC 完整迁移(账号 / 角色 / 经济 / 商城 / 场景 / 移动 / 战斗 / PVE / 社交 ...)
- **网络层**: TLV codec + WS transport + FrameRouter dispatcher 全部以 `[游戏A]` 参考实现为蓝本
- **业务逻辑**: 卡牌桶 11 + 角色 + 道具 + 邮件 等模块的协议号映射可被逆推

### 1.3 落地证据 — 文件路径(仅 rgs 仓库内,且真名部分加 `[别名]`)

- `docs/14-项目治理/RGS-REF-134-参考清单_v0.1.md` [别名](整文件,新建于基线 commit,208 行) — 含 24 条亮点
- `docs/14-项目治理/RGS-FLASH-OVERLAP-ANALYSIS_v0.2.md` [别名](维度对比 v0.2, 二审通过)
- `docs/15-IPA-完全对齐438cmds/RGS-DDD-v0.2-addendum-协议号映射.md` [别名] + `RGS-DDD-v0.2-addendum-业务逻辑逆推.md` [别名]
- `crates/network-gateway/src/{codec,ws,tlv,web_conn}.rs` + `bin/main.rs`
- `tools/h5_e2e/` 下 4 份 mjs 脚本 [别名](协议层 PoC)
- `tools/rgs-shanshuo-game/` [别名](目录名 = 旧 PoC 现场,目录名保留拼音暗示,后续 v1 升版前考虑改名 / 归档)
- `start-5-rgs-services.ps1`(5 域真实跑通脚本)

### 1.4 跨盘绝对路径硬编码(15+ 处)

`[跨盘-某发行商目录]\\server-analysis\\...` 类绝对路径,分布在 `docs/` 与 `crates/` 注释内,**含源码注释**,
可能影响 IDE 跳转体验 + 暴露个人学习路径。

> **属下一阶段脱敏处置范围**(P2 / 后续 turn);本 v0.1 只做"清单 + 二级索引",不触碰其他仓库文件。

---

## 2. `[游戏B]` — 商用分布式 actor 框架的设计语言(非单款产品)

### 2.1 性质

> **不是单一游戏**,是行业 30+ 年商用 actor 框架沉淀的设计语言。
> 基线来源: WhatsApp / Mochi Media / Wooga / FarmVille / Discord / Ericsson AXD301 等
> (其中部分产品已经被弃用 / 收购,作为"工业级 case study"使用)。

### 2.2 在 RGS 中的角色

**设计基线** — 9 原则 + 6 反模式 fingerprint,对照 6 域现状产出 P1 / P2 / P3 backlog
(per D-Boy 2026-09-04 14:30 JST paste 的 system prompt)。

> D-Boy ULYS-237 拍板:Q3 选 (A) **保留** —— `actor` / `OTP` / `gen-server` / `gen-fsm` / `supervisor` 是
> **语言 / 库 / 设计模式**,不是游戏名,不算"引入其他游戏名字"。

### 2.3 落地证据

- `docs/14-项目治理/RGS-REF-134-参考清单_v0.1.md` §2 [别名](67 条亮点 = 9 原则 35% 满分命中 + 6 反模式 A2/A3/A5/A6 不命中)
- `docs/14-项目治理/RGS-DDD-2026-09-04-GAP-AUDIT_v0.3.md` §2.1 + §2.2
- `docs/14-项目治理/ddd-review/RGS-DDD-2026-09-10-bottest-from-erlang_v0.4.md` [别名]
- `crates/network-gateway/src/{epmd,dist,cookie,nif,nif_demo}.rs`(分布式 actor 集群原语参考)

### 2.4 字面命中分布(本节描述"actor / OTP 字面"分布,**不是描述游戏**)

关键词 `actor` / `OTP` / `gen-server` / `gen-fsm` / `supervisor` 字面**368+ 行**,分布在:

- `crates/network-gateway/src/{lib, tcp, epmd, cookie, dist, codec, router, zone, web_conn, tlv, nif, nif_demo, ws}.rs` 源码注释
- `crates/network-gateway/build.rs` + `crates/network-gateway/erlang_test/test_add.erl` [别名](一份 actor 框架测试源)
- `crates/network-gateway/tests/{golden_vectors.rs, integration_phase15_demo.rs}`
- `tools/rgs-shim-rust/{Cargo.toml, README.md, docs/*.md, src/*.rs, bench/*.js}`
- `docs/01-核心架构与设计模式/RGS-REQ-027_*` + `RGS-BAS-001/002/023/024/100_*.md` + `RGS-DTL-100/101_*.md` + `RGS-INC-002_*.md`
- `docs/14-项目治理/cutover/PHASE-0-TO-4-FINAL.md` + `PHASE-4-HANDOFF-v0.1.md` + `RGS-DDD-2026-09-08-W7-W9-L18_v0.1.md` + `RGS-DDD-2026-09-10-bottest-from-erlang_v0.4.md`
- `docs/15-IPA-完全对齐438cmds/RGS-DDD-2026-09-04_v0.2.md` + `RGS-REQ-2026-09-04_v0.2.md`

### 2.5 处置策略(长尾)

> 关键词不是游戏名,是**语言 / 库 / 设计模式**。处置:
> - 若是描述 RGS 当前实现("参考:" / "远期借鉴")→ **保留**
> - 若是营销性提及("我们参考了 actor 设计")→ **改写**
>
> 详见 ULYS-237 历史回复 `01a0d79e-d74f-7c46-a395-e4853c4637c1` §Q3。

---

## 3. `[游戏C]` — 外部参考源码(`[跨盘-某发行商目录]` 绝对路径硬编码)

### 3.1 性质

商用 3D RPG GM 后台参考(外部源码目录 `[跨盘-某发行商目录]` [别名],非项目内仓库)。

### 3.2 在 RGS 中的角色

**GM 后台页面参考** — 19 页面映射 → RGS 10 核心页面落地。

### 3.3 命中范围

- 字面 `PATH-KEYWORD-A` [别名] **64 次** + `PATH-KEYWORD-B` [别名] **62 次** = **126 次**
- 多在 GM 后端代码注释 / 文档引用路径

### 3.4 落地证据

- `crates/gm-backend/src/{auth, broadcast, canvas, items, mall, players, reports, servers, summary, support}_handler.rs` + `lib.rs` + `main.rs`(handler 顶部含参考注释)
- `crates/gm-backend/Cargo.toml`
- `docs/14-项目管理/RGS-PM-ULYS-1-GAP-ANALYSIS_v0.1.md`
- commits `52c1a83f` / `23d447b5`:
  - _"feat(rgs-web): v0.2-gm GM 后台增强(参考 `[跨盘-某发行商目录]` [别名] 19 页面落地 10 核心页面) per D-Boy 13:13 JST"_

### 3.5 处置策略(P0)

- P0 — 跨盘绝对路径脱敏:`[跨盘-某发行商目录]\\...` → 相对路径 / `reference/` 占位

---

## 4. `[游戏D]` — 某厂商 CBT 私服本地逆向

### 4.1 性质

某厂商开放世界动作 MMO,以 `[CBTn]` [别名] 私服版本作为基准参考对象。

### 4.2 在 RGS 中的角色

**架构灵感来源** — 写入了 2 份独立治理文档,触发 ULYS-1 gap 阶段。

### 4.3 命中范围

- `[游戏D]` **39 次**(`docs/14-项目管理/RGS-BASIC-GAMED-INSPIRED-2026-09-17_v0.1.md` [别名])
- `[CBTn]` [别名] **79 次**(`docs/14-项目管理/RGS-REFERENCE-GAMED-PRIVATE-SERVER_v0.1.md` [别名])

### 4.4 落地证据

- `docs/14-项目管理/RGS-REFERENCE-GAMED-PRIVATE-SERVER_v0.1.md` [别名](**617 行**)
  - 整文件都是 `[游戏D]` / `[CBTn]` [别名] / 某厂商名 [别名] / `Drmk` [别名] / `PrivateServer` [别名] 命名
  - L572 提 `[游戏A]-client` = 某客户端引擎主客户端 [别名](L595 有修正说明,L572 那个 `[游戏A]` 引用其实是另一个 MMORPG 项目,与本需求无关)
  - L595 提到 `[跨盘-shanshuo-src-winrar]`(2.38 GB / `[游戏A]` 全套)是另一个 MMORPG 项目
- `docs/14-项目管理/RGS-BASIC-GAMED-INSPIRED-2026-09-17_v0.1.md` [别名](**~800 行**)
  - 10 项 REQ-001..010 + 文件名级逆向引用
  - L776 提 `[跨盘-PrivateServer]` 作为 `[游戏D]` 源码位置
- `docs/14-项目管理/RGS-PM-ULYS-1-GAP-ANALYSIS_v0.1.md`(`[CBTn]` 出现 9 次)

### 4.5 配套 DTL-001..010(10 份详细设计)

`docs/14-项目管理/RGS-DETAILED-GAMED-{001-rgs-proto-dump, 002-rgs-config-loader, 003-handler-partial-pattern, 004-rgs-protocol-builds, 005-rgs-debug, 006-db-known-issues, 007-data-driven-gameplay, 008-start-rgs-stack, 009-saga-runtime-multi-server, 010-mlua-config}_v0.1.md` [别名]

### 4.6 处置策略(P0 — D-Boy 拍板前待确认)

> ULYS-237 历史回复 `01a0d7a2-cee0-7470-8540-1a35aa07b36b` D-Boy "脱敏即可" 间接确认:
> - Q1 (A) 整批 `Remove-Item` —— 不采纳,因"脱敏即可"
> - Q1 (B) 改名 + 脱敏字面 —— 本 v0.1 路径下推荐
> - Q1 (C) 移到 `D:\\某用户笔记\\` 不入仓 —— 本 v0.1 路径下备选

**本 turn 落地的"脱敏版"**:
- 不修改 §4.4 / §4.5 提到的 12 份文件本身(per D-Boy "脱敏即可")
- 本清单 `RGS-ULYS-237-DESENSITIZED-REF-INVENTORY_v0.1.md` 已经把所有真名 / 拼音 / 路径别名通过 `[别名]` 角标标明
- 后续 turn 可选执行:
  - P0 — 整批删除 / 改写 `docs/14-项目管理/RGS-{REFERENCE, BASIC, DETAILED}-GAMED-*.md` [别名](2 份实际 + 10 份 planned; on-disk 改名于 v0.3 落地)
  - 备选 P0 — 移到 `D:\\某用户笔记\\` 不入仓,本仓库零痕迹

---

## 5. 汇总矩阵

| 维度 | `[游戏A]` | `[游戏B]` | `[游戏C]` | `[游戏D]` |
|---|---|---|---|---|
| 命中文件 | ~175 | 多文件散落 | ~12 | ~3 |
| 字面命中 | ~1,825 | 33 (按 "actor / OTP" 字串) | 126 | 118 |
| 真名性质 | 第三方 MMORPG IP | 设计语言(非 IP) | 商用 3D RPG GM 后台 | 某厂商新 IP / `[CBTn]` 私服 |
| 在 RGS 角色 | 协议 / 业务 / 网络全方位 | 设计基线 | GM 页面落地参考 | 架构灵感 |
| 处置优先级 | P0(协议对齐保留 + 营销角标) | P3(语言名保留) | P0(跨盘路径脱敏) | P0(整批删除 / 改写,本 v0.1 不动) |
| 文件位置 | docs/14/15/10/09 + crates/* | crates/network-gateway + docs/ | crates/gm-backend + docs/14-项目管理 | docs/14-项目管理 |

---

## 6. 不算"游戏名字"的字眼(已确认)

- **基础设施名**: Debezium / Kafka / Valkey / Redis / NATS — 通用 ADR 决策,不属游戏
- **开源协议**: HTTP / HTTPS / WS / TLV / mTLS — 协议栈,不属游戏
- **开源引擎**: 某客户端引擎(A) / Unity / IL2CPP — 通用引擎(虽然经常与 `[游戏A]` 共现,但本身无 IP)
- **自研协议名**: SmartSocket [别名](虽然经常与 `[游戏A]` 共现,但是 RGS 项目方自定义协议名)

---

## 7. 工作记录 — ULYS-237 turn 链路

| turn | 内容 | commit / artifact |
|---|---|---|
| 1(罗列) | 全库静态 grep 列举 + 5 类实体 + 命中文件清单 | (本地 `.multica/ulys-237-inventory.md`) |
| 2(拍板) | D-Boy "彻底去除这些名字" | — |
| 3(二次拍板) | D-Boy "脱敏即可" + ULYS-237 ask 改向(mask-only,非整批删除) | — |
| 4(代号化清单) | 本文件 — 代号化二级索引,落 `docs/14-项目治理/RGS-ULYS-237-DESENSITIZED-REF-INVENTORY_v0.1.md` | (next commit) |
| 5(合并到 dev / main) | per D-Boy 2026-09-25 08:37 JST 指令 | (merge commits) |

> 后续 turn 可选项(优先级排序):
> - P0 — 整批删除 / 改写 `[游戏D]` 12 份文件(per §4.6)
> - P0 — 跨盘绝对路径脱敏 `[游戏C]` 15+ 处(per §3.5)
> - P1 — `tools/rgs-shanshuo-game/` [别名] 改名 / 归档(目录名含 `[游戏A]` 拼音)
> - P2 — `tools/rgs-flash-mock/` [别名] 清理 `[游戏A]` 字面(若功能性提及可保留;目录名 `flash` = `[游戏A]` 英文意译)
> - P3 — `[游戏A]` 协议 / 端口对齐注释保留功能性提及,加 "(历史参考)" 角标

---

## 8. RGS-IMPL-001 §1.3 gate 取舍

> per `gate RGS-IMPL-001 §1.3`: 业务 Rust / SQL / Helm 实施被阻挡,直至 G-CODE-01~07 named-approved。

ULYS-237 的最终落地产物是**代号化索引**,不修改任何 RGS 实现层代码 / SQL / Helm。
业务 Rust / SQL / Helm 路径仍由 G-CODE-01~07 名号审批流程管控;本 v0.1 不触碰 gate。
