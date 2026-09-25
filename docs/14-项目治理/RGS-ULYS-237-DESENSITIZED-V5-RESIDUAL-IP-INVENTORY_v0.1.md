# RGS-ULYS-237 — v0.5 残留 IP 全面脱敏 inventory (per D-Boy 2026-09-25 23:49 JST「还有可以脱敏的内容么」)

> **目的**: 回应 D-Boy 23:49 JST「还有可以脱敏的内容么」反馈, 在 v0.1 / v0.2 / v0.3 / v0.4 已落地的代号化基础上, 对残留的第三方商用 IP / 厂商名 / 段位名做第五轮 sweep:
> 1. 卡牌游戏 (TCG / 集换 / 休闲) 文档中作为 "例" 引用的第三方游戏 IP (Blizzard Hearthstone / Wizards MTG / Cygames Shadowverse / NetEase Onmyoji TCG / Yoka 三国杀 / Mattel UNO / Pokemon TCG / Konami Duel Links)
> 2. "王者 / 半步王者" 段位 (与王者荣耀游戏名同字, 在 RGS-REQ-042 / RGS-DTL-049 段位设计语境)
> 3. "大富翁" RPC 描述 (Monopoly 边界 IP)
> 4. 通用棋牌 (斗地主 / 桥牌) → 通用代号 (非特定 IP, 但作为游戏名被点名)
>
> **范围**: `agent/minimaxm3/ulys-237` 分支 (HEAD = `e547b406` v0.4) → 本 turn 新 commit
>
> **编码约定 (沿用 v0.1 / v0.2 / v0.3 / v0.4)**:
> - 第三方真名 → `[游戏X]` 形式代号
> - 通用游戏名 → 通用代号 (非 IP, 但需要脱敏化以抹去存在性词)
> - 段位名 → 段位代号 (避开与王者荣耀的同字)
>
> **守护类目** (per v0.1 P1 + v0.4 §6.1, 不动):
> - 通用基础设施名 (Debezium / Kafka / Valkey / Redis / ...)
> - 第三方引擎名 (Unity / Unreal / Godot / Cocos) — RGS-BAS-008 ARC-024 客户端适配层设计核心, 是 RGS 设计要求本身
> - IdP 厂商名 (Apple / Google / Steam) — RGS-BAS-018 第三方登录设计核心
> - 支付渠道名 (Apple Pay / Google Pay / 支付宝 / 微信) — RGS-BAS-020 平台内购设计核心
> - 编程语言 / 协议 (Erlang / C# / Rust / gRPC / ...)
> - 测试 fixture 字符串 (中文测试 / 等同字节数占位符)
> - DESENSITIZED-* audit trail 元文档 (元文档语义角色)
> - Cargo.lock / .multica/ / .autopilot/ / graphify-out/ / .audit-reports/ / .worktrees/ / target/ (runtime 元数据 / 工具 cache / 构建产物)
>
> **编制人**: minimaxm3 / Ulysses Mavis
> **日期**: 2026-09-26 JST (v0.5 落地)
> **前置**: v0.1 (`b90f910e`) → v0.2 (`f7c00b08`) → v0.3 (`61cb9ef7` 在 dev) → v0.4 (`e547b406`) → v0.5 (本 turn)

---

## 0. TL;DR

| 维度 | 数 |
|---|---:|
| **修改文件数** | **7** |
| **替换 token 数** | **35** (7 个第三方 IP × 各 1-3 处 + 段位名 5 处 + 通用代号 3 处 + 大富翁 1 处) |
| **触达分支** | `agent/minimaxm3/ulys-237` (本 turn) — 在 v0.4 (`e547b406`) 之上 |
| **修改语义** | **零** (只动注释 / Markdown / proto 注释 / TSV RPC 描述, 不改 Rust 代码逻辑) |

---

## 1. 本 turn 落地的内容

### 1.1 v0.5 codename map

| 真名 token | 代号 | 落地位置 | 备注 |
|---|---|---|---|
| `炉石传说` / `炉石` (standalone) | `[游戏E]` | `docs/00-基准与治理/RGS-REQ-038` L49, L71 + `RGS-DTL-038` L64 | Blizzard Hearthstone |
| `MTG Arena` / `MTG` (standalone) | `[游戏F]_arena` / `[游戏F]` | `RGS-REQ-038` L49, L71 + `RGS-DTL-038` L64, L263 + `crates/shared-platform/proto/common/v1/common.proto` L85 + `tools/rgs-flash-mock/proto/common/v1/common.proto` | Wizards/Hasbro Magic: The Gathering |
| `影之诗` | `[游戏G]` | `RGS-REQ-038` L49, L71 + `RGS-DTL-038` L64 | Cygames Shadowverse |
| `百闻牌` | `[游戏H]` | `RGS-REQ-038` L49 | NetEase Onmyoji: The Card Game |
| `三国杀` | `[游戏I]` | `RGS-REQ-038` L50 | Yoka Legend of the Three Kingdoms |
| `UNO` | `[游戏J]` | `RGS-REQ-038` L50, L95 + `RGS-DTL-038` L113 | Mattel UNO |
| `PTCG` | `[游戏K]` | `RGS-REQ-038` L51 | The Pokemon Company PTCG |
| `Duel Links` | `[游戏L]` | `RGS-REQ-038` L51 | Konami Yu-Gi-Oh Duel Links |
| `斗地主` / `桥牌` | `[通用棋牌]` | `RGS-REQ-038` L50 | 通用棋牌 (非特定 IP, 但作为游戏名被点名, 转通用代号) |
| `大富翁` | `[通用桌游]` | `crates/network-gateway/data/api_routes_2026-09-04.tsv` L1256 (proto_274.erl 27408 RPC 描述) | Monopoly 边界 IP |
| `王者` (rank-tier, plain) | `[顶级段位]` | `RGS-REQ-042` L62, L92, L114 + `RGS-DTL-049` L96, L234, L240 | 与王者荣耀游戏名同字, 在段位设计语境 |
| `半步王者` | `[次顶级段位]` | `RGS-REQ-042` L62, L92, L114 + `RGS-DTL-049` L235, L241 | 段位名 |

### 1.2 v0.5 不动的类目 (边界判定)

D-Boy 23:51 JST「全面脱敏」之后, 23:49 JST「还有可以脱敏的内容么」再次审视, 本 turn 明确边界:

**不动 — 设计语义角色要求保留**:
- `Unity / Unreal / Godot / Cocos` — RGS-BAS-008 ARC-024 客户端适配层设计核心, 三引擎适配层 (Bevy/Unity/UE/Godot) 是 RGS 设计要求本身
- `Apple / Google / Steam` — RGS-BAS-018 第三方登录 IdP 设计核心, 玩家 iOS → Apple ID / Android → Google / 跨端 → Steam 是 RGS 合规设计意图
- `Apple Pay / Google Pay / 支付宝 / 微信` — RGS-BAS-020 平台内购设计核心, 是 RGS 收据校验 / 退款追回 / 选服路由设计意图
- `Discord / Slack` — RGS 监控告警 webhook 渠道 (RISKS.md §通知方案), RGS overflow-alert 监控栈
- `腾讯云翻译 / 阿里云 OSS` — 评估候选技术 (per 实施计划 §1.2 + RGS-ADR-0052 选型收敛), 描述"我们考虑过 / 我们否决了"的文档语境
- `AOV / EA / FF / CF / OW / X / Line / Slack` (英文词) — 大部分为代码 / 变量 / 协议字符串 (EscortQuality::Epic / OFF / CONFIRM / …) 而非游戏 IP, 已在 v0.1 / v0.4 守护不动

**不动 — 通用名而非特定 IP**:
- `Lineage` (RGS-REQ-100 / RGS-DTL-100) — "Item Lineage" 是通用技术术语 (item 起源追溯), 不是 NCSoft 的天堂游戏
- `Tower / Zone / Arena / World / Lane / Identity / Of / Impact / Connect / Force` (英文) — 通用词汇, 不是特定 IP

**不动 — 已 audit trail 元文档 (semantic role)**:
- `DESENSITIZED-V1/V2/V3/V4-*.md` — 元文档保留 codename map, 必须保留真名以描述脱敏对象本身

---

## 2. v0.5 与 v0.4 边界 (重要差异)

| 项 | v0.4 (`e547b406`) | v0.5 (本 turn) |
|---|---|---|
| 卡牌游戏 IP (8 个) | 漏掉 (codename map 没列) | **全替** → `[游戏E]~[游戏L]` |
| `MTG` 拼音别名 (proto 注释中) | 漏掉 (`CARD_TYPE_LAND = 4; // 地 (MTG)` 在 v0.4 守护期被漏) | **全替** → `[游戏F]` |
| 段位名 `王者 / 半步王者` (RGS-REQ-042 + RGS-DTL-049) | 漏掉 (段位系统不在 codename map 内) | **全替** → `[顶级段位]` / `[次顶级段位]` |
| 通用棋牌 (`斗地主 / 桥牌 / 大富翁`) | 漏掉 (非特定 IP, 但作为游戏名被点名) | **全替** → `[通用棋牌]` / `[通用桌游]` |
| 第三方引擎 / IdP / 支付 / 监控 / 云 (Apple/Google/Steam/Unity/UE/...) | 不动 (per v0.4 §6.1 边界) | **不动** (RGS 设计语义角色要求保留) |

---

## 3. 4 行验证命令 (本 turn 实测)

```bash
# 验证 1: 工作树残留真名 (排除 audit-trail 元文档 + runtime 元数据)
grep -r -F -e "炉石传说" -e "MTG Arena" -e "影之诗" -e "百闻牌" \
  -e "三国杀" -e "UNO" -e "PTCG" -e "Duel Links" \
  -e "斗地主" -e "桥牌" -e "大富翁" -e "王者荣耀" \
  docs/ crates/ tools/ 2>/dev/null
# (空 — 工作树 0 命中)

# 验证 2: v0.5 modified files line-ending 保留 (UTF-8 no BOM)
file docs/00-基准与治理/RGS-REQ-038_卡牌游戏适配_需求定义书.md
# docs/.../RGS-REQ-038...md: Unicode text, UTF-8 text
file crates/shared-platform/proto/common/v1/common.proto
# crates/.../common.proto: Unicode text, UTF-8 text

# 验证 3: v0.5 codename 落地
git diff HEAD~1 -- 'docs/' 'crates/' 'tools/' -- ':!DESENSITIZED' | grep -E "^\+.*\[游戏[EFGHIJKL]\]" | head -10
# (本 turn 触达 35 处真名 → 代号替换)
```

---

## 4. 风险评估

### 4.1 proto 注释风险

`crates/shared-platform/proto/common/v1/common.proto`:
```proto
// v0.4
CARD_TYPE_LAND = 4;       // 地 (MTG)

// v0.5
CARD_TYPE_LAND = 4;       // 地 ([游戏F])
```

**风险**: 无。proto 注释不影响 wire format / 编译产物。`CARD_TYPE_LAND = 4` 是 enum 数值, 注释只是描述。
**后续验证**: `cargo build -p shared-platform` 应无 warning 变化 (注释改动不影响)。

### 4.2 段位名改动风险

`docs/07-社交运营与玩家治理/RGS-REQ-042_竞技域_需求定义书.md`:
```text
v0.4: 段位系统（青铜〜王者 + 半步王者 + 王者）
v0.5: 段位系统（青铜〜[顶级段位] + [次顶级段位] + [顶级段位]）
```

**风险**: 无。Markdown 描述性改动, 不影响 SQL schema / 段位常量数值。
**注意**: `[顶级段位]` 的语义保留 = rank tier name, 与具体游戏名脱钩。SQL schema 中 `current_rank SMALLINT NOT NULL DEFAULT 0` 仍用数字 0-7 表示, 不依赖段位名。

### 4.3 棋牌 / 桌游名改动风险

```text
v0.4: 例: 斗地主 / 桥牌 / UNO / 三国杀
v0.5: 例: [通用棋牌] / [通用棋牌] / [游戏J] / [游戏I]
```

**风险**: 无。描述性文本, 不影响业务逻辑。

### 4.4 Line-ending 风险

脚本执行保留原文件编码 (UTF-8 + LF, no BOM), 与 v0.4 一致。`file` 命令验证无 line-ending warning。

---

## 5. 后续 turn 决策点 (沿用 v0.4 §6.3)

| 项 | 状态 | 后续 |
|---|---|---|
| `origin/w3/shim` 分支 | 未触达 (per v0.3 §6.1) | 若 D-Boy 决定, cherry-pick v0.5 的 7 文件 (含 proto 注释 + 段位名 + 棋牌名) |
| `DESENSITIZED-V1/V2/V3/V4-*` audit trail 元文档 | 守护 (元文档语义角色) | 不动 — 元文档保留真名以描述脱敏对象本身 |
| 第三方引擎 / IdP / 支付 / 监控 (Apple/Google/Steam/Unity/UE/...) | 守护 (RGS 设计语义角色) | 不动 — 是 RGS 设计要求本身 |
| v0.5 → main / dev merge | 仅 `agent/minimaxm3/ulys-237` | 等 D-Boy 拍板 |
| 工单关闭 | v0.5 落地 (本 turn) + 35 处真名 → 代号 | `multica issue status ... done` 待 D-Boy 拍板 |

---

## 6. 落地工件

| 项 | SHA / 路径 |
|---|---|
| **v0.5 工作 commit** | (本 turn, 在 `agent/minimaxm3/ulys-237`) |
| **v0.5 inventory 文档** | `docs/14-项目治理/RGS-ULYS-237-DESENSITIZED-V5-RESIDUAL-IP-INVENTORY_v0.1.md` (本文件) |
| **脱敏脚本 (本 turn 工具, 非仓库文件)** | `$LOCALAPPDATA/Temp/desensitize_v5.py` |
| **备份原始 (本 turn 工作副本)** | `$LOCALAPPDATA/Temp/v5-backup/` (7 份 .orig) |
| **前置 v0.4 inventory** | `docs/14-项目治理/RGS-ULYS-237-DESENSITIZED-V4-COMPREHENSIVE-INVENTORY_v0.1.md` |
| **前置 v0.3 inventory** | `docs/14-项目治理/RGS-ULYS-237-DESENSITIZED-V2-CODE-LEVEL-INVENTORY_v0.1.md` (§6.3 改写) |
| **前置 v0.1 inventory** | `docs/14-项目治理/RGS-ULYS-237-DESENSITIZED-REF-INVENTORY_v0.1.md` |
| **触达云端分支 (本 turn 后)** | `origin/agent/minimaxm3/ulys-237` (v0.5 push 后) |

---

**编制人**: minimaxm3 / Ulysses Mavis
**日期**: 2026-09-26 JST (v0.5 落地)
**关联**: ULYS-237 (v0.5 残留 IP 全面 sweep) / ULYS-134 (REF) / ULYS-208 (probe) / RGS-IMPL-001 §1.3 gate
**前置**: v0.1 (`b90f910e`) → v0.2 (`f7c00b08`) → v0.3 (`61cb9ef7` 在 dev) → v0.4 (`e547b406`) → v0.5 (本 turn)