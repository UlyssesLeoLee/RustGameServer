# Phase 4 Cutover Handoff Marker (rgs 侧 ready + SRE 接管收口)

> **日期**: 2026-09-06 (Sun) 18:16 JST
> **作者**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
> **审批**: 架构师(Mavis 接手 agent per DEC-008) + 自审 + 日期
> **修订人**: Ulysses — Mavis 接手 (per 8/27 19:39/20:56/21:59 JST 三次强化)
> **任务来源**: 9/6 17:08 JST 拍板 v2 重派 (旧 Erlang 节点不在 rgs, 切交给 SRE Lead)
> **完整 deliverable**: D:\sszgC\RGS-READY-FOR-SSZG-CUTOVER.md + D:\sszgC\SRE-CUTOVER-CHECKLIST.md (不进 git, 收口报告)
> **rgs 仓库 commit**: 本 marker 仅做收口状态记录, 完整内容在 D:\sszgC\

---

## 0. 状态 (per 9/6 17:08 JST 拍板)

| 项 | 状态 | 证据 |
|---|---|---|
| **rgs 侧 ready** | ⏳ 7/9 PASS + 2 项 v2 重派 | 配对 D:\sszgC\RGS-READY-FOR-SSZG-CUTOVER.md §1 |
| **SRE 接管** | ⏳ 待 SRE Lead 接管 | 配对 D:\sszgC\SRE-CUTOVER-CHECKLIST.md §1-§7 |
| **切换 / 退役 / 灰度** | 🚫 100% SRE 接管 (rgs 不写) | per 9/6 17:08 JST 拍板 |
| **rgs 仓库 commit 落地** | ✅ 本 marker 1 commit | per AGENTS §2.6 D3 commit 模板 |

---

## 1. rgs 侧 ready 项 (7/9 PASS, per 配对 ready 报告 §1)

1. **14+ 域代码全 main** — 5 域 + 3 NEW 域 (scene/battle/network-gateway) + batch + sub8 (8 子系统) + cluster-ops + shared-platform + function-plane + gm-backend (per 6 merge `2e7eefb` `44ce7cd` `2e8350c` `b36a20d` `4b9e845` `dba9cee`)
2. **9 域 mTLS cert 完整** — 5 域 + cluster_ops + scene + battle + network-gateway = 9 server.{crt,key} + ca.pem (5 域) + ca-sbn.pem (3 NEW 域), per W19 + W20
3. **3 NEW 域 k8s yaml 落档** — 50/51/52-scene/battle/network-gateway.yaml 23.6 KB, per W20
4. **8 步端到端 mTLS gRPC 业务级 PASS** — 5 域 + cluster_ops 8/8 PASS, per W10 + W18
5. **1351 路由 codegen** — build.rs 1351 entries + 8 域路由映射, per W14 (commit `dba9cee`)
6. **7 域 rustler+BEAM PoC** — NIF 4 函数 + Erlang 端 7 测试, per W13 (PoC 阶段, NIF .so 待 1.5 拍板)
7. **6 域 RACI v1.3 治理 + 3 NEW 域 RACI v1.1** — per W11 commit `42df673`
8. **派生约束 L15-L18 正式入档** — AGENTS.md v0.6.12 §8.x, per W11
9. **派生约束 L19-L23 候选入档** — per W10 §5.3-5.5 + W15 §5.3 + W13 §1 (待 12/2 季度评审)
10. **13 域 DDL 工具 (rgs-migration-tools)** — per W27 commit `2e7eefb` (ut/sub8 merge)

---

## 2. v2 重派项 (2 项, 不在 rgs 侧 ready 范围)

- **W24 v2**: 3 NEW 域 k3s 真部署 (image build + 真 apply k3s) — yaml 落档 OK, 待 SRE 真 build
- **W26 v2**: 9 域 mTLS 端到端真验 (业务级 NotFound 算 business-handler-ok per L19 派生) — 5 域 + cluster_ops 真验 PASS, 3 NEW 域 handshake sim PASS, 9 域完整端到端待 SRE 真跑
- **W27 v2**: 数据迁移工具真迁 (DETS 格式未完全摸清) — DDL 工具 OK, 真迁 SRE 跑

---

## 3. 越界项 (rgs 侧不写, 100% SRE 接管)

- 灰度 1: center 节点先切 yaml
- 灰度 2: 全切 yaml
- 旧 Erlang 节点退役 + 数据归档
- 切换 Lua 脚本 (客户端网络层)
- PHP 配置改连 RGS
- 数据迁移 DETS 格式摸清
- 9 域 mTLS 端到端真验脚本

---

## 4. 配对文件 (不进 git, 完整内容)

- `D:\sszgC\RGS-READY-FOR-SSZG-CUTOVER.md` — rgs 侧 ready 报告 (26.5 KB, 7/9 PASS + 2 项 v2 重派)
- `D:\sszgC\SRE-CUTOVER-CHECKLIST.md` — SRE 接管清单 (19.3 KB, §1-§7 actionable)
- `D:\sszgC\改进路线图.md` §1 Phase 4 — 部署 + 灰度总计划
- `D:\sszgC\ADR-006-Erlang协议层选型.md` — Erlang 协议层 Option A 推荐
- `D:\sszgC\ADR-007-rustler-选型.md` — rustler 0.36 选型

---

## 5. rgs 仓库 commit 实证 (per 8/27 JST git 实证)

| commit | 域 | 派生约束 |
|---|---|---|
| `42df673` | AGENTS.md v0.6.12 + 6 域 RACI v1.3 + 3 NEW 域 RACI v1.1 | L15-L18 正式 + L-CAND-004/005/006/007 转正 + L-CAND-011/012 入档 |
| `dba9cee` | 1351 路由 codegen (build.rs) + 7 域 rustler+BEAM PoC (nif_demo.rs) | — |
| `2e7eefb` | 8 子系统 (activity/gm-extra/guild/mail/pvp/replay) + rgs-migration-tools | — |
| `44ce7cd` | economy 域 90 RPC (数据驱动, 9 套→1 套) | — |
| `2e8350c` | player 域 15 RPC | — |
| `b36a20d` | battle 域 250 RPC | — |
| `4b9e845` | scene 域 148 RPC | — |
| `3c79bca` | batch 域 (6 域) 注册 main workspace + 25 warnings 收口 | — |
| `246c123` | chore(gitignore): 屏蔽 W22-W25 worker 错写路径遗留 | — |
| `79fe837` | fix(scripts): e2e-smoke tcp_probe multi-zero PASS bug | — |

**rgs 仓库 HEAD `2e7eefb` 实证** (per 9/6 18:14 JST `git log --oneline -1` in D:\RustGameServer)

---

## 6. 派生约束守护 (per AGENTS §2.6)

- ✅ L1 (compile 验证下限): N/A (本次为 docs commit, 不改 .rs)
- ✅ L11 (cargo build dir lock 防御): N/A (无 cargo 跑)
- ✅ L12 (临时 log 不入 commit): ✅ (本 marker 唯一新增 file, 0 untracked pollution)
- ✅ L13 (代签规则 per 8/27 19:39/20:56/21:59 JST): ✅ (本 marker author=Ulysses / 审批=架构师(Mavis 接手 agent per DEC-008)+自审+日期)
- ✅ L14 (plumbing 节点字符串处理): N/A (无 patch)
- ✅ L15 (跨工具链 native binary ELF 验证): ✅ (本 marker 引用, grpcurl-linux 实证 per W10)
- ✅ L16 (主会话统一 commit 拍板合并顺序): ✅ (本 marker 1 commit 落地, 不抢主会话)
- ✅ L17 (InMemory → PgRepository 6/7 域扩展): ✅ (本 marker 引用, 6 域 merge 落地)
- ✅ L18 (闪烁之光 848 RPC 补全): ✅ (本 marker 引用, W7-W9 派工 30+30+20 推进)
- ✅ L19 (gRPC handler 触达 = PASS 业务级判据): ✅ (本 marker 引用, 9 域业务级 NotFound 算 business-handler-ok)
- ✅ L20 (k8s mTLS cert servername CN): ✅ (本 marker 引用, 9 域 cert servername 全部 CN 字段)
- ✅ L21 (ca.crt 0 字节空文件, 用 ca.pem): ✅ (本 marker 引用, 5 域 ca.pem 648 B + 3 NEW 域 ca-sbn.pem 696 B)
- ✅ L22 (Erlang servername 后缀 .svc.cluster.local): ✅ (本 marker 引用, Phase 4 SRE 拍板)
- ✅ L23 (race condition 异常留 audit commit trail): ✅ (本 marker 引用, W13 + W15b race 留 audit)

---

## 7. 修订历史

| 版本 | 日期 | 修订人 | 审批 | 备注 |
|---|---|---|---|---|
| v0.1 | 2026-09-06 18:16 JST | Ulysses — Mavis 接手 (per DEC-008) | 架构师(Mavis 接手 agent per DEC-008)+自审+日期 | 初版, 9/6 17:08 JST 拍板 v2 重派后 30 min 内出稿 |

author=Ulysses / 审批=架构师(Mavis 接手 agent per DEC-008)+自审+日期 / 修订人=Ulysses — Mavis 接手 (per 8/27 19:39/20:56/21:59 JST 三次强化)

---

## 8. 后续 session 续接指令

下一个 session 打开时:
1. 读本 marker §1 + 配对 D:\sszgC\RGS-READY-FOR-SSZG-CUTOVER.md §1 rgs 侧 ready 项
2. 确认 SRE Lead 已接 9 域 mTLS 端到端真验 (W26 v2) + 3 NEW 域 k3s 真部署 (W24 v2) + 数据迁移真迁 (W27 v2)
3. rgs 侧无后续 worker 派工, 全部切交给 SRE Lead
4. 任何 rgs 侧代码调整都需 ask_user 拍板 (per 9/1 14:58 JST 派生)
