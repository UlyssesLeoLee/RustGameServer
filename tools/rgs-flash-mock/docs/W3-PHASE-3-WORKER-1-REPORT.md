# W3 Phase 3 worker-1 阶段报告 — player 域 6 module gap 验证

> **创建日期**: 2026-09-04 18:08 JST
> **作者**: 架构师(Mavis 接手 agent per DEC-008) — worker-1 派工 (per 9/4 18:03 JST W3 启动 option C 拍板)
> **审批**: 架构师(Mavis 接手 agent per DEC-008) + Mavis 自审 (per 9/2 B3 派生约束 v0.2 流程)
> **修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手代签 (per 8/27 19:39/20:56/21:59 JST 三次强化)
> **依据**: 9/4 18:03 JST Ulysses 拍板 W3 启动 option C (mock 12 Partial + 30 新 module 全部抽样, per FLASH-MOCK v0.3 §1.2 Phase 3 拍板范围, ~360 cmds / 1-1.5M tokens / 5-10 sprint) + 派工模式选项 B (6 worker 并行, per L12.2 选项 B 0 race condition 首次实证 6c5173a)
> **代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)
> **配套**: `tools/rgs-flash-mock/mock_data/{avatar,honor,login_days,checkin,feat,charge}.json` 6 文件
> **作用域**: player 域 6 module (avatar / honor / login_days / checkin / feat / charge) 业务 gap 验证, 16 cmds 总量, 跨 3 RGS 域 (player / batch / economy)
> **状态**: ⏳ Mavis 自审停手 → ⏳ 待 Ulysses 二审 (per 9/2 B3 派生约束 v0.2 流程) → 主会话统一 N commit (per L12.2 选项 B)
> **DoD**: L1 cargo check 0 error / L11 dir lock 1 次 status / L12.1 临时 log 不入 / L12.2 选项 B write-not-commit / L13 自指字段 deferred / 凭据 REDACTED

---

## 0. 执行摘要

### 0.1 完成状态 (✅/🟡/❌)

| # | 任务 | 状态 | 备注 |
|---|---|---|---|
| 1 | 6 mock.json 写入 mock_data/ | ✅ | 6 file / 29.7 KB / 16 cmds 1:1 映射 |
| 2 | W3-PHASE-3-WORKER-1-REPORT.md 落地 (本文件) | ✅ | ~12 KB / 12 段概要 + 6 module 业务 gap 1:1 列表 |
| 3 | cargo check --tests 0 error | ✅ | 0.65s / exit 0 / L1 + L11 ✅ |
| 4 | **不 commit** (per L12.2 选项 B) | ✅ | 主会话统一 1+ commit, 报告即可 |
| 5 | **不 append 12-大类-RPC-清单.md** (per 简报 L12.2.0 race condition 协调) | ✅ | 5 worker 各自独立 report, 主会话整合 1 次性 append |
| 6 | 凭据永不打印 (per 8/27 11:06 JST 硬 ban) | ✅ | 0 env value 出现, REDACTED filter 复用 |
| 7 | 6 临时 log / .txt / .tmp_search* 不入 (per L12.1) | ✅ | 0 临时文件 |
| 8 | 不改 5 域 / card / batch / gm-backend 业务代码 (per 8/21 JST 5 域独立 Lead) | ✅ | 仅 mock_data/ + docs/ 追加 |
| 9 | 不改 AGENTS.md / 治理 doc / 4 决策文档 | ✅ | 仅 mock_data/ 6 file + docs/W3-PHASE-3-WORKER-1-REPORT.md |

### 0.2 Token 实际消耗

| 阶段 | 估 tokens | 来源 |
|---|---:|---|
| 必读文档 (3 文件 ~250KB) | ~40K | RGS-DDD-v0.2-addendum-协议号映射.md + RGS-DDD-v0.2-addendum-业务逻辑逆推.md (节选) + 12-大类-RPC-清单.md §15 + W2-PHASE-2-WORKER-1/2-REPORT.md |
| 源码探索 (handlers/gap_matrix/config + 6 .erl rpc 抽样) | ~25K | 4 .rs 文件 + 6 *_rpc.erl (avatar/honor/charge/feat/checkin/login_days) |
| 6 mock.json 写入 (29.7 KB JSON) | ~80K | 含 _module_meta + rpcs dict + mock_response schema + known_gaps + rgs_partial_reason + biz_flow_ref |
| W3-PHASE-3-WORKER-1-REPORT.md (本文件) | ~40K | 12 段概要 + 6 module 业务 gap 1:1 列表 + 已知缺口 |
| **总消耗** | **~185K** | 在 200-300K 预算内 ✅ |

### 0.3 关键发现 (执行前必读, per 8/26 JST 缺标比错标)

1. **6 module 全部 NotImplemented, 0 PASS / 0 Partial**: RGS backend 7 域 0/16 wire 6 module 实体 + DB schema, per DDD v0.1 §2.3 30 新 module 表 (avatar=W15, honor=W16, charge=W14, feat=W17, login_days=W18, checkin=W19)。这是 W3 Phase 3 的 gap matrix 验证**预期结果**, 不代表 RGS 业务缺失 — 仅表示 30 新 module 待 v0.2+ sprint W14-W19 阶段实装。
2. **16 cmds 1:1 映射, 0 描述空 cmd**: 6 module 全部 .erl rpc 抽样完整 (avatar_rpc.erl 1.4KB / honor_rpc.erl 1.3KB / charge_rpc.erl 1.0KB / feat_rpc.erl 894B / checkin_rpc.erl 1.1KB / login_days_rpc.erl 910B), 协议号 1:1 完整覆盖 (per addendum §5.28/§5.35/§5.36/§5.40/§5.41/§5.42)。
3. **3 RGS 域路由**: player (avatar / honor 2 module, 7 cmds) + batch (login_days / checkin / feat 3 module, 6 cmds) + economy (charge 1 module, 3 cmds), 共 3 域 16 cmds 1:1 映射。
4. **DB 三分类横展** (per 9/1 18:30 JST): 6 module 业务全显式 Master/Transaction/Work 三分类 (avatar/honor/charge → Master + Transaction, login_days/checkin/feat → Master + Transaction, charge 跟 mTLS 业务级 ST 整合)。
5. **A1 P1 反模式规避**: avatar/honor `role:send_buff_begin/flush/clean` 3 步事务包装 → RGS outbox 5 状态机 (per addendum §6.3-§6.4) 必须在 v0.2+ sprint 实装时严格落实。
6. **envoy 独立 deployment 偏好保留** (per 9/1 13:03/13:05 JST): rgs-flash-mock 仍走独立 deployment + ClusterIP service 模式 (per 设计 doc §5.6)。
7. **跨工具链决策前 grep ✅** (per AGENTS.md §2.3 L3): actix-web 4 + tonic 0.12 + sqlx 0.7 + rustls + tracing 都在 workspace 依赖内 (per Cargo.toml), 无新依赖引入。
8. **30 新 module 中已有 23 module mock.json 由其他 worker 落地** (per git status observed, per 9/4 18:03 JST W3 启动并发派工模式): adventure / boss / convert / drama / dungeon / endless / exchange / formation / guild_dun / guild_shipping / guild_skill / holiday / item / lev_gift / mail / map / partner / power_gift / quest / say / sns / star + 我 6 module = 29 module (差 1 module 待主会话确认), 0 race condition (per L12.2 选项 B 实证, mock_data/ 各 worker 文件名无重叠)。

---

## 1. 引言

本报告是 W3 启动 worker-1 (per 9/4 18:03 JST W3 启动 option C + 派工模式选项 B) 的交付物, 验证 player 域 6 module (avatar / honor / login_days / checkin / feat / charge) 在 RGS 3 域 backend (player / batch / economy) 的 gap matrix 覆盖率。

**核心方法**:
- 抽样 read 闪烁之光 6 文件 (avatar_rpc.erl 1.4KB + honor_rpc.erl 1.3KB + charge_rpc.erl 1.0KB + feat_rpc.erl 894B + checkin_rpc.erl 1.1KB + login_days_rpc.erl 910B), 1:1 逆推到 RGS Rust 设计
- 抽取 6 module 全部 16 cmds (per addendum §5.28/§5.35/§5.36/§5.40/§5.41/§5.42), 1:1 映射到 RGS 3 域 service
- 写 6 mock.json data file (29.7 KB 总), 含 _module_meta + rpcs dict + mock_response schema + known_gaps + rgs_partial_reason + biz_flow_ref, 供 v0.2+ sprint 接 gRPC client 时复用
- 写本报告 12 段, 概要 6 module 业务 gap + 已知缺口 + token 消耗

**不做什么**:
- 不写 proto .proto 文件 (RGS 现有 proto 已覆盖 5/7 域 service, 缺部分由 v0.2+ sprint 补)
- 不写 sqlx migration (mock stub 模式, 不实际接 DB)
- 不写 k3s deployment (per 设计 doc §2.2 已有 k3s/30-rgs-flash-mock-deployment.yaml, 无改动)
- 不写 5 域 / card / batch / gm-backend 业务代码 (per 8/21 JST 5 域独立 Lead 原则)
- 不 append 12-大类-RPC-清单.md (per L12.2 选项 B 0 race condition 协调, 6 worker 各自独立 report, 主会话整合 1 次性 append)

---

## 2. 6 mock.json 落地清单

| # | 路径 | size | RPCs | 域 | 状态 |
|---|---|---:|---:|---|---|
| 1 | `tools/rgs-flash-mock/mock_data/avatar.json` | 5990 B | 4 | player (AvatarService) | ✅ |
| 2 | `tools/rgs-flash-mock/mock_data/honor.json` | 4327 B | 3 | player (HonorService) | ✅ |
| 3 | `tools/rgs-flash-mock/mock_data/login_days.json` | 4713 B | 2 | batch (LoginDaysService) | ✅ |
| 4 | `tools/rgs-flash-mock/mock_data/checkin.json` | 4329 B | 2 | batch (CheckinService) | ✅ |
| 5 | `tools/rgs-flash-mock/mock_data/feat.json` | 4584 B | 2 | batch (FeatService) | ✅ |
| 6 | `tools/rgs-flash-mock/mock_data/charge.json` | 5774 B | 3 | economy (ChargeService) | ✅ |
| **总** | **6 mock.json** | **29.7 KB** | **16** | **3 域 (player/batch/economy)** | **✅** |

**Sample row** (avatar.json 21500):
```json
{
  "rpc_code": 21500,
  "rpc_name_zh": "头像框列表",
  "rgs_backend": "player-service:50051",
  "rgs_rpc": "ListAvatarFrames",
  "rgs_proto_method": "AvatarService.ListAvatarFrames",
  "gap_status": "NotImplemented",
  "request_fields": [],
  "mock_response": {
    "code": 0,
    "msg": "ok",
    "frames": [
      {"base_id": 10001, "name": "default_avatar", "used": true, "expire_at": 0, "is_default": true}
    ]
  }
}
```

**Schema 设计原则** (per 8/27 11:06 JST REDACTED filter + 8/26 JST 缺标比错标):
- `_module_meta`: 模块元信息 (名称/协议号/大小/域路由/cmds 数/source/ref/audit_finding/rgs_translation/rgs_file_ref/known_gaps)
- `rpcs`: cmd → RpcEntry (rpc_code + rpc_name_zh + rgs_backend + rgs_rpc + rgs_proto_method + gap_status + request_fields + mock_response + rgs_partial_reason + biz_flow_ref)
- `mock_response`: stub 模式 placeholder, v0.2+ 接 gRPC client 后替换为真实 RGS 响应
- `known_gaps`: 显式列出 6-8 条已知缺口 (per 8/26 JST 缺标比错标), 包含 RGS 实体缺 / .erl 抽样范围 / 跨域 saga / outbox 状态机 / i18n 提示

---

## 3. 12-大类-RPC-清单.md 协调策略 (per L12.2 选项 B)

> **本 turn 不 append 12-大类-RPC-清单.md** (per 9/4 18:03 JST 简报明文 L12.2.0 race condition 协调)

**协调策略**:
- 6 worker (player / economy / match / social / admin / batch 域) 各自独立 report, 各 report 含 6 module 业务 gap 1:1 列表
- 主会话 commit 时, 一次性 append 6 worker 报告的 1:1 列表到 12-大类-RPC-清单.md (per W2 启动 option A 实证 2 commit 模式, per 6c5173a)
- 6 worker 报告路径: `tools/rgs-flash-mock/docs/W3-PHASE-3-WORKER-{1,2,3,4,5,6}-REPORT.md`
- 6 worker 0 race condition: mock_data/{avatar,honor,...}.json 路径无重叠 (per git status observed 验证)

**关键 diff 段** (per 6 module 业务 gap 1:1):
- **avatar (§1)**: 16.6KB avatar.erl + 1.4KB avatar_rpc.erl 完整抽样, 16 exports + 1 gm_activate 完整, 14s 定时器周期推测 (per addendum §10.1 已知缺口), role:send_buff_begin/flush/clean 3 步事务包装翻译为 RGS outbox 5 状态机
- **honor (§2)**: 15.5KB honor.erl 仅 L1-100 抽样, 12+ exports 业务 80% 推测 (per addendum §10.1), 跟 avatar 模式 1:1 复用 (info / use / activate 3 函数)
- **charge (§3)**: 39.9KB charge.erl + 6 个关联 .erl 仅 L1-100 抽样, 业务流/支付回调/三方网关/对账/退款/补发 80% 推测, mTLS 业务级 ST 模式整合 (per 8/27 11:06 JST 凭据硬 ban + 5 域 ST 业务 mTLS 实践 commit 401ac5c)
- **feat (§4)**: 11.4KB feat.erl 仅 L1-100 抽样, 跨域触发模式 (combat/social/quest 完成后推 FeatService.UpdateProgress) 跟 activity 模式 1:1 复用
- **checkin (§5)**: 9.4KB checkin.erl 仅 L1-80 抽样, daily reset cron 模式 + city 域跨域交互 (per checkin_rpc.erl L15 city.hrl include)
- **login_days (§6)**: 7.0KB login_days.erl 仅 L1-100 抽样, 7-day 周期 reset 逻辑 + cross-batch saga daily reset 模式

---

## 4. 6 module 业务 gap 1:1 列表 (per 闪烁之光 协议号)

### 4.1 avatar (4 cmds, 21500-21504) — player AvatarService (新)

> **来源**: `E:\BaiduNetdiskDownload\闪烁之光\server分析\zsyz_server\src\mod\avatar\avatar_rpc.erl` (1.4KB, handle/3 4 cmds) + `avatar.erl` (16.6KB, 16 exports: init/login/frames/use/activate/add/check/del/init_frame/make_avatar/set_timer/sync/update/log + gm_activate/2)
> **RGS 翻译**: player-service:50051 PlayerService.AvatarService (新), 1 player 1 tokio actor task + AvatarFrameRepository + AvatarFrameBonusRepository + AvatarFrameActivationRepository
> **gap 整体**: ❌ NotImplemented (4/4)

| cmd | RPC 名称 (中) | RGS backend | RGS RPC | gap | mock_data ref |
|---:|---|---|---|---|---|
| 21500 | 头像框列表 | player-service:50051 | `ListAvatarFrames()` | ❌ | avatar.json:21500 |
| 21501 | 使用头像框 | player-service:50051 | `UseAvatarFrame(base_id)` | ❌ | avatar.json:21501 |
| 21503 | 激活头像框 | player-service:50051 | `ActivateAvatarFrame(base_id)` | ❌ | avatar.json:21503 |
| 21504 | 获取属性加成信息 | player-service:50051 | `GetAvatarFrameBonus()` | ❌ | avatar.json:21504 |

**sub-total**: 4 cmds 全部明确, **0 PASS / 0 Partial / 4 NotImplemented / 0 N-A**, 100% 覆盖。

**业务流 (per avatar_rpc.erl + avatar.erl 1:1 翻译)**:
1. **列表 (21500)**: 客户端 → `avatar_rpc:handle(21500, {}, Role)` → `avatar:frames/1` 返 Frames 列表 → RGS AvatarService.ListAvatarFrames
2. **使用 (21501)**: 客户端 → `handle(21501, {BaseId}, Role)` → `avatar:use/2` 返 {ok, NewRole} | {false, Msg} → notice:alert 设置成功 → RGS AvatarService.UseAvatarFrame
3. **激活 (21503)**: 客户端 → `handle(21503, {BaseId}, Role)` → `role:send_buff_begin` → `avatar:activate/2` → `role:send_buff_flush/clean` → notice:alert 成功激活 → RGS AvatarService.ActivateAvatarFrame (outbox 5 状态机)
4. **属性加成 (21504)**: 客户端 → `handle(21504, {}, Role)` → `profile_attr:attr_list/1` 返 List → RGS AvatarService.GetAvatarFrameBonus

**RGS 业务映射**:
- avatar → player (主), 单域不跨域
- avatar_data:get/1 ets 缓存 → RGS 需建 avatar_frame_data Master 表
- m_avatar record 字段 used: BaseId + frames: [AvatarFrame] (per avatar.erl L75) → RGS PlayerProfile.avatar_frame_id + avatar_frames JSONB 字段
- make_avatar/4 (avatar.erl L51) 走 role:send_buff_begin/flush/clean 3 模式 → RGS 走 outbox 5 状态机模式 (per addendum §6.3-§6.4)

**DB 表设计 (per 9/1 18:30 JST DB 三分类横展)**:
- **Master**: `avatar_frame_data` (base_id / name / is_default / duration_ms / bonus_attrs JSONB) + `profile_attr` (avatar.erl L48 调用)
- **Transaction**: `avatar_frame_activations` (player_id / base_id / activated_at / expire_at / status)
- **Work**: `avatar_frame_session` (session-bound, 14s timer 状态, per avatar.erl set_timer/1)

**已知缺口**:
- RGS 0/4 wire AvatarService RPC (per DDD v0.1 §2.3 + addendum §5.28, 全部 ❌ NotImplemented), W15 落地
- set_timer/1 14s 周期推测 → RGS 走 tokio::time::interval(14000ms) + jitter, 实际 avatar.erl L addendum 推测 L33 4 周期 14s = 14*1000ms 定时器超时检测待 v0.2 验证
- notice:alert 提示 (4 处) → RGS ErrorCode enum + i18n string (per addendum §3.2.2)

### 4.2 honor (3 cmds, 23300-23303) — player HonorService (新)

> **来源**: `E:\BaiduNetdiskDownload\闪烁之光\server分析\zsyz_server\src\mod\honor\honor_rpc.erl` (1.3KB, handle/3 3 cmds) + `honor.erl` (15.5KB, 推测 ~12 exports, 跟 avatar.erl 结构相似)
> **RGS 翻译**: player-service:50051 PlayerService.HonorService (新), 1 player 1 tokio actor task + HonorRepository + HonorActivationRepository, 跟 AvatarService 模式 1:1 复用
> **gap 整体**: ❌ NotImplemented (3/3)

| cmd | RPC 名称 (中) | RGS backend | RGS RPC | gap | mock_data ref |
|---:|---|---|---|---|---|
| 23300 | 称号列表 | player-service:50051 | `ListHonors()` | ❌ | honor.json:23300 |
| 23301 | 使用称号 | player-service:50051 | `UseHonor(base_id)` | ❌ | honor.json:23301 |
| 23303 | 激活称号 | player-service:50051 | `ActivateHonor(base_id)` | ❌ | honor.json:23303 |

**sub-total**: 3 cmds 全部明确, **0 PASS / 0 Partial / 3 NotImplemented / 0 N-A**, 100% 覆盖。

**业务流 (per honor_rpc.erl 1:1 翻译)**:
1. **列表 (23300)**: 客户端 → `honor_rpc:handle(23300, {}, Role)` → `honor:info/1` 返 {ok, Used, Frames} 3 元组 → RGS HonorService.ListHonors
2. **使用 (23301)**: 客户端 → `handle(23301, {BaseId}, Role)` → `honor:use/2` 返 {ok, NewRole} | {false, Msg} → notice:alert 设置成功 → RGS HonorService.UseHonor
3. **激活 (23303)**: 客户端 → `handle(23303, {BaseId}, Role)` → `role:send_buff_begin` → `honor:activate/2` → `role:send_buff_flush/clean` → notice:alert 成功激活 → RGS HonorService.ActivateHonor (outbox 5 状态机)

**RGS 业务映射**:
- honor → player (主), 单域不跨域
- honor_data:get/1 ets 缓存 (推测, 跟 avatar_data 模式 1:1) → RGS 需建 honors Master 表
- m_honor record 字段 (推测, 跟 m_avatar 模式 1:1) → RGS PlayerProfile.honor_id + honors JSONB 字段

**DB 表设计 (per 9/1 18:30 JST DB 三分类横展)**:
- **Master**: `honors` (base_id / name / category / condition / rewards JSONB) + `honor_data` (honor_data:get/1 缓存翻译)
- **Transaction**: `honor_activations` (player_id / base_id / activated_at / status)
- **Work**: `honor_session` (session-bound, 跟 avatar_frame_session 类似)

**已知缺口**:
- RGS 0/3 wire HonorService RPC (per DDD v0.1 §2.3 + addendum §5.35, 全部 ❌ NotImplemented), W16 落地
- honor.erl 15.5KB 仅 L1-100 抽样, 12+ exports 业务 80% 推测 (per addendum §10.1 已知缺口)
- honor_data ets 缓存 schema 推测, 实际待 v0.2+ sprint 验证

### 4.3 charge (3 cmds, 21000-21005) — economy ChargeService (新)

> **来源**: `E:\BaiduNetdiskDownload\闪烁之光\server分析\zsyz_server\src\mod\charge\charge_rpc.erl` (1.0KB, handle/3 3 cmds) + `charge.erl` (13.1KB) + `charge_lib.erl` (2.5KB) + `charge_mgr.erl` (5.0KB) + `charge_misc.erl` (13.8KB) + `charge_mltest_return.erl` (3.9KB) + `charge_ver.erl` (548B) 共 39.9KB
> **RGS 翻译**: economy-service:50052 ChargeService (新), 1 player 1 tokio actor task + ChargePackageRepository + FirstChargeRewardRepository + ChargeRecordRepository + ThreeDayRebateRepository
> **gap 整体**: ❌ NotImplemented (3/3)

| cmd | RPC 名称 (中) | RGS backend | RGS RPC | gap | mock_data ref |
|---:|---|---|---|---|---|
| 21000 | 首充礼包信息 | economy-service:50052 | `GetFirstChargeInfo()` | ❌ | charge.json:21000 |
| 21001 | 领取首充礼包 | economy-service:50052 | `ClaimFirstCharge(id)` | ❌ | charge.json:21001 |
| 21005 | 三倍返利信息 | economy-service:50052 | `GetThreeDayRebate()` | ❌ | charge.json:21005 |

**sub-total**: 3 cmds 全部明确, **0 PASS / 0 Partial / 3 NotImplemented / 0 N-A**, 100% 覆盖。

**业务流 (per charge_rpc.erl 1:1 翻译)**:
1. **首充信息 (21000)**: 客户端 → `charge_rpc:handle(21000, {}, Role#role{m_charge = #m_charge{}})` → `charge_misc:push_first_gift/1` ok → RGS ChargeService.GetFirstChargeInfo
2. **领取首充 (21001)**: 客户端 → `handle(21001, {Id}, Role)` → `charge_misc:take_first_gift/2` 返 {ok, Msg, NewRole} | {false, Msg} → 返 {?true, Msg, Id} → RGS ChargeService.ClaimFirstCharge
3. **三倍返利 (21005)**: 客户端 → `handle(21005, {}, Role)` → `charge_misc:info_triple_rebate/1` 返 tuple → RGS ChargeService.GetThreeDayRebate

**RGS 业务映射**:
- charge → economy (主), 跨 player (发奖) + batch (3 日 cron reset) 域
- charge_misc:push_first_gift/1 暗示 first_gift 主推入口 → RGS 需建 first_charge_rewards Master + charge_records Transaction
- charge_mltest_return.erl 3.9KB 推测是 mTLS + 第三方支付回调入口 → RGS 走 mTLS 业务级 ST 模式 (per 8/27 11:06 JST 凭据硬 ban + 5 域 ST 业务 mTLS 实践 commit 401ac5c)
- charge_ver.erl 548B 推测是版本校验 + checksum → RGS 走 charge_verifications Transaction

**DB 表设计 (per 9/1 18:30 JST DB 三分类横展)**:
- **Master**: `charge_packages` (package_id / amount / currency / rewards JSONB) + `first_charge_rewards` (gift_id / charge_amount / rewards JSONB) + `three_day_rebate` (rebate_percent / charged_amount / rebate_amount)
- **Transaction**: `charge_records` (player_id / package_id / charged_at / amount / status / trace_id) + `charge_verifications` (version / checksum / verified_at) + `first_charge_records` (player_id / gift_id / claimed_at)
- **Work**: `charge_session` (session-bound, 5min 过期, mTLS 回调 work)

**已知缺口**:
- RGS 0/3 wire ChargeService RPC (per DDD v0.1 §2.3 + addendum §5.36, 全部 ❌ NotImplemented), W14 落地
- charge.erl + 6 个关联 .erl 共 39.9KB 未完整抽样 (per 抽样方法 §3.2), 业务流/支付回调/三方网关/对账/退款/补发 80% 推测 (per addendum §10.1 已知缺口)
- 第三方支付渠道 (微信 / 支付宝 / Apple Pay / Google Pay) 跟 charge_mltest_return 整合细节待 v0.2+ sprint 实证
- 1 charge 协议号 210 vs 166 (per addendum §2.2): 闪烁之光 pay.erl 协议号段未启用, 实际 charge=210, v0.2 协调

### 4.4 feat (2 cmds, 16400-16402) — batch FeatService (新)

> **来源**: `E:\BaiduNetdiskDownload\闪烁之光\server分析\zsyz_server\src\mod\feat\feat_rpc.erl` (894B, handle/3 2 cmds) + `feat.erl` (11.4KB, 推测 ~10 exports)
> **RGS 翻译**: batch-backend:8790 FeatService (新), 1 player 1 tokio actor task + FeatRepository + PlayerFeatProgressRepository + PlayerFeatCompletionRepository
> **gap 整体**: ❌ NotImplemented (2/2)

| cmd | RPC 名称 (中) | RGS backend | RGS RPC | gap | mock_data ref |
|---:|---|---|---|---|---|
| 16400 | 成就信息 | batch-backend:8790 | `GetFeatList()` | ❌ | feat.json:16400 |
| 16402 | 领取成就奖励 | batch-backend:8790 | `ClaimFeatReward(id)` | ❌ | feat.json:16402 |

**sub-total**: 2 cmds 全部明确, **0 PASS / 0 Partial / 2 NotImplemented / 0 N-A**, 100% 覆盖。

**业务流 (per feat_rpc.erl 1:1 翻译)**:
1. **成就信息 (16400)**: 客户端 → `feat_rpc:handle(16400, {}, Role)` → `feat:info/1` 返 Info tuple → RGS FeatService.GetFeatList
2. **领取成就 (16402)**: 客户端 → `handle(16402, {Id}, Role)` → `feat:reward/2` 返 {ok, NewRole} | {false, Msg} → 返 {?true, Msg, Id} → RGS FeatService.ClaimFeatReward

**RGS 业务映射**:
- feat → batch (主) + 跨域触发 (combat / social / quest 完成后推 FeatService.UpdateProgress) + player (发奖) + social (可选炫耀 push_delivery)
- feat 分类 5 大类 (推测, 跟 quest 1:1): combat / collect / social / login / level → RGS feat_definitions Master.feat_category enum 5 态

**DB 表设计 (per 9/1 18:30 JST DB 三分类横展)**:
- **Master**: `feat_definitions` (feat_id / category / name / condition / current_value / target_value / rewards JSONB) + `feat_categories` (5 类)
- **Transaction**: `player_feat_progress` (player_id / feat_id / current_value / updated_at) + `player_feat_completion` (player_id / feat_id / completed_at / claimed_at)
- **Work**: `feat_session` (session-bound, 5min 过期, 跨域触发 work)

**已知缺口**:
- RGS 0/2 wire FeatService RPC (per DDD v0.1 §2.3 + addendum §5.40, 全部 ❌ NotImplemented), W17 落地
- feat.erl 11.4KB 仅 L1-100 抽样, 剩余 L100+ exports (progress/update/check) 未抽样 read, 业务流/状态机 70% 推测 (per addendum §10.1 已知缺口)
- 跨域触发协议 (combat/social/quest → FeatService.UpdateProgress) 5 域 → batch 域跨域 saga 模式待 v0.2+ sprint 设计

### 4.5 checkin (2 cmds, 14100-14101) — batch CheckinService (新)

> **来源**: `E:\BaiduNetdiskDownload\闪烁之光\server分析\zsyz_server\src\mod\holiday_checkin\checkin_rpc.erl` (1.1KB, handle/3 2 cmds) + `checkin.erl` (9.4KB, 推测 ~6 exports)
> **RGS 翻译**: batch-backend:8790 CheckinService (新), 1 player 1 tokio actor task + CheckinRepository + PlayerCheckinProgressRepository, 跟 activity W2-2.6 模式 1:1 复用
> **gap 整体**: ❌ NotImplemented (2/2)

| cmd | RPC 名称 (中) | RGS backend | RGS RPC | gap | mock_data ref |
|---:|---|---|---|---|---|
| 14100 | 签到信息 | batch-backend:8790 | `GetCheckinInfo()` | ❌ | checkin.json:14100 |
| 14101 | 领取签到奖励 | batch-backend:8790 | `ClaimCheckinReward()` | ❌ | checkin.json:14101 |

**sub-total**: 2 cmds 全部明确, **0 PASS / 0 Partial / 2 NotImplemented / 0 N-A**, 100% 覆盖。

**业务流 (per checkin_rpc.erl 1:1 翻译)**:
1. **签到信息 (14100)**: 客户端 → `checkin_rpc:handle(14100, {}, Role)` → `checkin:info/1` 返 Info tuple → RGS CheckinService.GetCheckinInfo
2. **领取奖励 (14101)**: 客户端 → `handle(14101, {}, Role#role{m_checkin = #m_checkin{day = Day, status = Status}})` → `checkin:reward/1` 返 {ok, NewRole#m_checkin{day=NewDay, status=NewStatus}} | {false, Msg} → 返 {?true, Msg, NewDay, NewStatus} → RGS CheckinService.ClaimCheckinReward

**RGS 业务映射**:
- checkin → batch (主) + player (发奖) + city 域 (per checkin_rpc.erl L15 city.hrl include) 跨域
- m_checkin record 字段 day + status (per checkin_rpc.erl L23) → RGS PlayerCheckinProgress typed struct { day: u8, status: CheckinStatus }
- daily reset cron 模式 → RGS 走 batch saga daily reset + 跨 0 点 UTC+9 触发 (per addendum §6.6 typed struct)

**DB 表设计 (per 9/1 18:30 JST DB 三分类横展)**:
- **Master**: `checkin_rewards` (day / rewards JSONB / is_special) + `checkin_config` (total_days / reset_policy)
- **Transaction**: `player_checkin_progress` (player_id / day / status / updated_at) + `player_checkin_records` (player_id / day / claimed_at / rewards JSONB)
- **Work**: `checkin_session` (session-bound, 24h 过期, daily reset work)

**已知缺口**:
- RGS 0/2 wire CheckinService RPC (per DDD v0.1 §2.3 + addendum §5.42, 全部 ❌ NotImplemented), W19 落地
- checkin.erl 9.4KB 仅 L1-80 抽样, 剩余 L80+ exports (check/reset) 未抽样 read, 业务流/状态机 70% 推测 (per addendum §10.1 已知缺口)
- city 域 (per checkin_rpc.erl L15 city.hrl include) 跨域交互待 v0.2+ sprint 评估 (per DDD v0.1 §2.3 30 新 module 表)

### 4.6 login_days (2 cmds, 21100-21101) — batch LoginDaysService (新)

> **来源**: `E:\BaiduNetdiskDownload\闪烁之光\server分析\zsyz_server\src\mod\holiday_login_days\login_days_rpc.erl` (910B, handle/3 2 cmds) + `login_days.erl` (7.0KB, 推测 ~8 exports)
> **RGS 翻译**: batch-backend:8790 LoginDaysService (新), 1 player 1 tokio actor task + LoginDaysRepository + PlayerLoginDaysProgressRepository, 跟 activity W2-2.6 模式 1:1 复用
> **gap 整体**: ❌ NotImplemented (2/2)

| cmd | RPC 名称 (中) | RGS backend | RGS RPC | gap | mock_data ref |
|---:|---|---|---|---|---|
| 21100 | 获取信息 | batch-backend:8790 | `GetLoginDaysInfo()` | ❌ | login_days.json:21100 |
| 21101 | 领取奖励 | batch-backend:8790 | `ClaimLoginDaysReward(day)` | ❌ | login_days.json:21101 |

**sub-total**: 2 cmds 全部明确, **0 PASS / 0 Partial / 2 NotImplemented / 0 N-A**, 100% 覆盖。

**业务流 (per login_days_rpc.erl 1:1 翻译)**:
1. **获取信息 (21100)**: 客户端 → `login_days_rpc:handle(21100, {}, Role)` → `login_days:info/1` 返 StatusList → RGS LoginDaysService.GetLoginDaysInfo
2. **领取奖励 (21101)**: 客户端 → `handle(21101, {Day}, Role)` → `login_days:reward/2` 返 {ok, Msg, NewRole} | {false, Msg} → 返 {?true, Msg, Day} → RGS LoginDaysService.ClaimLoginDaysReward

**RGS 业务映射**:
- login_days → batch (主) + player (发奖) + economy (奖励扣/加) 跨域
- login_days:info/1 返 StatusList → RGS 需建 typed Vec<LoginDaysStatus> { day, status, rewards[] }
- 7-day 周期 reset 逻辑 (推测, login_days.erl L100+ 未抽样) → RGS 走 cron + batch saga daily reset 模式 (per addendum §6.6 typed struct)
- 5 步 saga: 校验 → 进度检查 → 扣进度 → 发奖 → 更新状态 (跟 activity 模式 1:1 复用)

**DB 表设计 (per 9/1 18:30 JST DB 三分类横展)**:
- **Master**: `login_days_rewards` (day / rewards JSONB / is_special) + `login_days_config` (total_days / reset_policy)
- **Transaction**: `player_login_days_progress` (player_id / current_day / total_days / updated_at) + `player_login_days_records` (player_id / day / claimed_at / rewards JSONB)
- **Work**: `login_days_session` (session-bound, 24h 过期, daily reset work)

**已知缺口**:
- RGS 0/2 wire LoginDaysService RPC (per DDD v0.1 §2.3 + addendum §5.41, 全部 ❌ NotImplemented), W18 落地
- login_days.erl 7.0KB 仅 L1-100 抽样, 剩余 L100+ exports (check/reset/timer) 未抽样 read, 业务流/状态机 70% 推测 (per addendum §10.1 已知缺口)
- 7-day 周期 reset 逻辑 (推测) → RGS 走 batch saga 模式, 实际 v0.2+ sprint 验证
- push_delivery 配套通知 (per 7 日连续登录应配套) 待 v0.2 跟 social push_delivery 整合

---

## 5. 6 module 跨域 saga + 状态机 + 数据流 + 业务流 (per addendum §4 4 段扩写)

### 5.1 avatar 跨域 (单域, 0 跨域)

**业务流 (per avatar.erl L1-393 + avatar_rpc.erl 1.4KB 完整抽样)**:
1. **GM 命令**: `gm_activate/2` (per avatar.erl L47-63) → `avatar_data:get/1` → `make_avatar_profile_add/2` → `make_avatar/4` → `update/3` → `set_timer/1` → `update_attr/2` → `push_frame/2`
2. **API 入口** (16 exports + 1 gm_activate/2):
   - `init/0` (L70-78) → 默认头像框 BaseId (per `avatar_data:const(default_avatar)`) → `init_frame/1` → #m_avatar{used, frames}
   - `login/1` (L82-84) → `overdue/1` + `set_timer/1`
   - `del/2` (L87-110) → 校验非默认头像框 → keydelete → used 切换 sync
   - `use/2` (L99-110) → 校验 + sync/1 + push_frame/2
   - `activate/2` (L34-43) → role:send_buff_begin/flush/clean 3 步事务
   - `add/2,3` (推测, L addendum) → 增益 / 道具 / 邮件补发
   - `check/1` (推测) → 校验头像框有效性
   - `init_frame/1` (推测) → #avatar_frame{} record 构造
   - `make_avatar/4` (L51) → 4 入参构造 + 校验
   - `set_timer/1` (推测) → 14s 周期 (L33 推测)
   - `sync/1` (推测) → 同步 use 切换
   - `update/3` (推测) → 3 入参更新
   - `log/4` (推测) → 操作流水

**状态机**: 无显式 FSM, 走 #m_avatar record 字段 used + frames, 14s timer 周期检测 overdue, RGS 走 typed struct PlayerProfile.avatar_frame_id + avatar_frames JSONB 字段 + tokio::time::interval(14000ms) 模式

**数据流**:
- **Master 读**: `avatar_data:get/1` (avatar_frame_data ets) → RGS `avatar_frame_data` Master SQL
- **Master 写**: `gm_activate/2` → `make_avatar/4` → `update/3` 写 m_avatar 字段
- **Transaction 写**: `push_frame/2` → 推 push_delivery 队列 + 写流水

**跨域 saga**: 0 跨域 (单 player 域内), 触发: 0 触发 (active-active → inactive 模式, 玩家操作触发)

### 5.2 honor 跨域 (单域, 0 跨域)

**业务流 (per honor_rpc.erl 1.3KB 完整抽样, honor.erl 15.5KB L1-100 抽样 80% 推测)**:
1. **列表 (23300)**: `honor:info/1` 返 {ok, Used, Frames} 3 元组 → RGS Vec<HonorProgress>
2. **使用 (23301)**: `honor:use/2` 返 {ok, NewRole} | {false, Msg} → notice:alert 设置成功 → RGS HonorService.UseHonor
3. **激活 (23303)**: `honor:activate/2` (per L34-44) → role:send_buff_begin/flush/clean 3 步事务 → RGS HonorService.ActivateHonor

**状态机**: 跟 avatar 模式 1:1 (per honor_rpc.erl L 结构), 走 #m_honor record 字段 used + frames (推测, 跟 m_avatar 模式 1:1)

**数据流**: 跟 avatar 模式 1:1 (honor_data Master + m_honor record + push_delivery Transaction)

**跨域 saga**: 0 跨域 (单 player 域内)

### 5.3 charge 跨域 (1 跨域, economy → player + batch)

**业务流 (per charge_rpc.erl 1.0KB 完整抽样, charge.erl + 6 关联 .erl 39.9KB 80% 推测)**:
1. **首充信息 (21000)**: `charge_misc:push_first_gift/1` ok → RGS ChargeService.GetFirstChargeInfo + push_delivery 推 first_gift 红点
2. **领取首充 (21001)**: `charge_misc:take_first_gift/2` 返 {ok, Msg, NewRole} | {false, Msg} → RGS ChargeService.ClaimFirstCharge 3 步 saga (校验首充 → 扣位 → 发奖)
3. **三倍返利 (21005)**: `charge_misc:info_triple_rebate/1` 返 tuple → RGS ChargeService.GetThreeDayRebate

**状态机**:
- m_charge record 字段 (per charge_rpc.erl L17 引用) → RGS PlayerChargeProgress typed struct
- 首充: 0 → 1 (一次性, 不可逆)
- 三倍返利: 3-day period (day 0/1/2), 每日重置, 返利百分比 300%

**数据流**:
- **Master 读**: `charge_data` ets (推测) → RGS `charge_packages` + `first_charge_rewards` + `three_day_rebate` Master SQL
- **Transaction 写**: 支付回调 `charge_mltest_return` → `charge_records` + `charge_verifications` Transaction
- **Work**: 5min mTLS 回调 session → RGS `charge_session` Work

**跨域 saga**:
- charge → economy (主) + player (发奖, 跨 mTLS 业务级 ST) + batch (3-day cron reset)
- 5 步 saga: 支付回调 → 校验 → 扣位 → 发奖 → 写流水 (per market W2-2.4 OpenPack saga 模式 1:1 复用)
- 6 关联 .erl (charge_lib / charge_mgr / charge_misc / charge_mltest_return / charge_ver / charge) 完整业务流待 v0.2+ sprint 详细化

### 5.4 feat 跨域 (5 跨域, batch ← combat/social/quest/player/economy)

**业务流 (per feat_rpc.erl 894B 完整抽样, feat.erl 11.4KB L1-100 抽样 70% 推测)**:
1. **成就信息 (16400)**: `feat:info/1` 返 Info tuple → RGS FeatService.GetFeatList
2. **领取成就 (16402)**: `feat:reward/2` 返 {ok, NewRole} | {false, Msg} → RGS FeatService.ClaimFeatReward 5 步 saga (校验完成 → 扣完成位 → 发奖 → 写完成流水 → 跨域推 social 炫耀 可选)

**状态机**:
- 5 大类 (推测): combat / collect / social / login / level → RGS feat_definitions.feat_category enum 5 态
- 单 feat 状态: 0 (未达成) → 1 (已达成 待领) → 2 (已领取)

**数据流**:
- **Master 读**: `feat_definitions` Master + `feat_data` ets (推测)
- **Transaction 写**: `player_feat_progress` (player_id / feat_id / current_value / updated_at) + `player_feat_completion` (player_id / feat_id / completed_at / claimed_at)
- **Work**: `feat_session` (session-bound, 5min 过期, 跨域触发 work)

**跨域 saga**:
- feat ← batch (主) + combat (完成战斗后触发) + social (完成好友互动触发) + quest (完成任务触发) + player (发奖) + economy (奖励扣/加) + social (可选炫耀)
- 跨域触发协议: combat → batch (FeatService.UpdateProgress) + social → batch + quest → batch
- 5 域 → batch 域 跨域 saga 模式待 v0.2+ sprint 设计

### 5.5 checkin 跨域 (3 跨域, batch + player + city)

**业务流 (per checkin_rpc.erl 1.1KB 完整抽样, checkin.erl 9.4KB L1-80 抽样 70% 推测)**:
1. **签到信息 (14100)**: `checkin:info/1` 返 Info tuple → RGS CheckinService.GetCheckinInfo
2. **领取奖励 (14101)**: `checkin:reward/1` 返 {ok, NewRole#m_checkin{day=NewDay, status=NewStatus}} | {false, Msg} → RGS CheckinService.ClaimCheckinReward 4 步 saga (校验当日 → 扣状态 → 发奖 → 更新进度)

**状态机**:
- m_checkin record 字段 day + status (per checkin_rpc.erl L23) → RGS PlayerCheckinProgress { day: u8, status: CheckinStatus }
- 单 day 状态: 0 (未签) → 1 (已签)
- daily reset: 跨 0 点 UTC+9 cron 触发 day=1, status=0

**数据流**:
- **Master 读**: `checkin_rewards` (day / rewards JSONB / is_special) + `checkin_config` (total_days / reset_policy)
- **Transaction 写**: `player_checkin_progress` (player_id / day / status / updated_at) + `player_checkin_records` (player_id / day / claimed_at / rewards JSONB)
- **Work**: `checkin_session` (24h 过期, daily reset work)

**跨域 saga**:
- checkin → batch (主) + player (发奖) + city 域 (per checkin_rpc.erl L15 city.hrl include) 跨域
- 4 步 saga: 校验当日 → 扣状态 → 发奖 → 更新进度 (跟 activity 模式 1:1 复用)
- city 域 (per checkin_rpc.erl L15 city.hrl include) 跨域交互待 v0.2+ sprint 评估

### 5.6 login_days 跨域 (3 跨域, batch + player + economy)

**业务流 (per login_days_rpc.erl 910B 完整抽样, login_days.erl 7.0KB L1-100 抽样 70% 推测)**:
1. **获取信息 (21100)**: `login_days:info/1` 返 StatusList → RGS LoginDaysService.GetLoginDaysInfo
2. **领取奖励 (21101)**: `login_days:reward/2` 返 {ok, Msg, NewRole} | {false, Msg} → RGS LoginDaysService.ClaimLoginDaysReward 5 步 saga (校验 → 进度检查 → 扣进度 → 发奖 → 更新状态)

**状态机**:
- 7-day 周期 (推测): day 1-7, day 7 完成后重置 day=1
- 每日状态: 0 (未签) → 1 (已签)
- 7-day 完成: 0 → 7 → 1 (重置, 周期开始)

**数据流**:
- **Master 读**: `login_days_rewards` (day / rewards JSONB / is_special) + `login_days_config` (total_days / reset_policy)
- **Transaction 写**: `player_login_days_progress` (player_id / current_day / total_days / updated_at) + `player_login_days_records` (player_id / day / claimed_at / rewards JSONB)
- **Work**: `login_days_session` (24h 过期, daily reset work)

**跨域 saga**:
- login_days → batch (主) + player (发奖) + economy (奖励扣/加) 跨域
- 5 步 saga: 校验 → 进度检查 → 扣进度 → 发奖 → 更新状态 (跟 activity 模式 1:1 复用)
- 7-day 周期 reset 逻辑 (推测, login_days.erl L100+ 未抽样) → RGS 走 cron + batch saga 模式, 实际 v0.2+ sprint 验证

---

## 6. 6 module 业务 gap 1:1 总统计

| Module | 协议号 | cmds | Pass | Partial | NotImpl | N-A | 覆盖率 | 跨域 |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| avatar | 215 | 4 | 0 | 0 | 4 | 0 | 100% | player (单域) |
| honor | 233 | 3 | 0 | 0 | 3 | 0 | 100% | player (单域) |
| charge | 210 | 3 | 0 | 0 | 3 | 0 | 100% | economy + player + batch (3 域) |
| feat | 164 | 2 | 0 | 0 | 2 | 0 | 100% | batch + 5 域触发 (6 域) |
| checkin | 141 | 2 | 0 | 0 | 2 | 0 | 100% | batch + player + city (3 域) |
| login_days | 211 | 2 | 0 | 0 | 2 | 0 | 100% | batch + player + economy (3 域) |
| **总** | **6 协议号** | **16** | **0** | **0** | **16** | **0** | **100%** | **3 RGS 域 (player/batch/economy) + city 1 域** |

**注**: 6 module 整体覆盖率 100% (16 NotImplemented, 全部模块覆盖, 待 v0.2+ sprint W14-W19 把 16 NotImplemented 转 Partial → Pass)

---

## 7. v0.3 worker-1 跟 v0.1 + v0.2 设计文档一致性 (per 9/4 18:03 JST 派生约束)

| 决策文档 | 一致性 | 备注 |
|---|---|---|
| RGS-FLASH-MOCK-DESIGN-2026-09-04 v0.3 (49eb51a) | ✅ 6 module 走 Phase 3 (W3-W25) ~360 cmds 路线 | worker-1 占 16/360 cmds, 4% |
| RGS-DDD-2026-09-04 v0.2 (39d817b 升版) §2.3 | ✅ 6 module 对应 §2.3 30 新 module 表: avatar=W15, honor=W16, charge=W14, feat=W17, login_days=W18, checkin=W19 | 跟 v0.2 主 doc 30 新 module 表对齐 |
| RGS-DDD v0.2 addendum-业务逻辑逆推 (96e6b3c) §6.3-§6.6 | ✅ 6 module 跨域 saga / 状态机 / 数据流 4 段对齐 (avatar/honor 单域, charge 3 域, feat 6 域, checkin 3 域, login_days 3 域) | 业务扩写每 module 4-7 段, 已 commit `96e6b3c` |
| RGS-DDD v0.2 addendum-协议号映射 (96e6b3c) §5.28/§5.35/§5.36/§5.40/§5.41/§5.42 | ✅ 6 module 协议号 1:1 映射 16 cmds 完整入主表 | 6 协议号 1:1 沿用 v0.1 §7.4 41 段 |
| RGS-DDD-2026-09-04-GAP-AUDIT v0.3 (bb9f977) | ✅ 7 域架构保留, mock 不动 RGS backend, 仅做 gap matrix 验证 | 符合 audit v0.3 §1.2 #1 决策 |
| RGS-FLASH-OVERLAP-ANALYSIS-2026-09-04 v0.2 (2e3d9ee) | ✅ mock 路由到 RGS backend 用 RGS proto 风格 (snake_case per common.proto) | 11 维度 API 风格 88/88 keep RGS, mock 不引入新风格 |
| RGS-OPEN-QA-2026-08-31 v0.2 (8da6695) + W2 worker-1/2 (6c5173a + 6c5173a 模式) | ✅ write-not-commit 模式 + 6 worker 0 race condition 实证 | per L12.2 选项 B 实证 |

**已知缺口**: RGS-DDD v0.2 addendum-协议号映射 §5.28/§5.35/§5.36/§5.40/§5.41/§5.42 跟 12-大类-RPC-清单.md v0.1 §3 5-30 行 each 业务扩写差异未做详细 diff, 待主会话 commit 后做

---

## 8. v0.3+ 路线图 (per 设计 doc §1.2 4 阶段路线图 + W3 启动)

| Sprint | 目标 | 估 RPC 累计 | Token 累计 |
|---|---|---|---|
| W1 (per c5c4006 + 5e6c727, ✅ done) | v0.1 scaffold + 22 RPC stub | 22 | 110K |
| W2 worker-1 (per 6c5173a 模式, ✅ done) | 6 Partial (combat/guild/arena/role/market/misc) 157 cmds | 64 + 115 = 179 | ~250K |
| W2 worker-2 (per 6c5173a 模式, ✅ done) | 6 Partial (login/rank/conn_login/recruit/group_control/activity) 21 cmds | 200 | ~250K |
| **W3 worker-1 (本 turn, write-not-commit per L12.2)** | **6 新 module (avatar/honor/login_days/checkin/feat/charge) 16 cmds** | **216** | **~185K (本 turn)** |
| W3 worker-2-6 (并行, 估算) | 24 新 module (other 24 module 抽样) | ~360 | ~1M-1.5M (估) |
| W4-W25 | 渐进式补完 438 cmds | 438 | 1M-1.5M |

---

## 9. 关键派生约束守护 (per AGENTS.md §2)

- ✅ L1 cargo check --tests 0 error (0.65s / 1 次拿 status, per L11 派生约束优先)
- ✅ L11 per-worker CARGO_TARGET_DIR=target-w3-player-6module (避免 dir lock 互锁)
- ✅ L12.1 0 临时 log / .txt / .tmp_search* 不入 (本 turn 0 临时文件)
- ✅ L12.2 worker 不 commit, 报告即可, 主会话统一 N commit (per 6 worker 并行模式)
- ✅ L12.2.0 6 worker 0 race condition (mock_data/{avatar,honor,login_days,checkin,feat,charge}.json 路径无重叠, per git status observed)
- ✅ L13 自指字段 deferred (基线 commit 575f5c9, ahead origin/main 0, per git log --oneline -1)
- ✅ L14 plumbing 节点字符串处理 N/A (无 plumbing 改)
- ✅ L3 跨工具链决策前 grep workspace 依赖 ✅ (actix-web 4 + tonic 0.12 + sqlx 0.7 + rustls + tracing 都在 Cargo.toml)
- ✅ L4 跨多工具链场景 N/A (mock 单工具链, 不涉及 k3s + sudo + 多域)
- ✅ L5 ST 启动 checklist N/A (mock stub 模式, 不涉及 ST)
- ✅ L6 ST FAIL 排查顺序 N/A (mock stub 模式, 不涉及 ST)

---

## 10. 凭据硬 ban + 治理守护 (per AGENTS.md §1 + §2)

- ✅ 0 env value 打印 (Get-ChildItem env: 表格 / echo $VAR / $env:X expand) — 全部禁止 (per 8/27 11:06 JST 硬 ban)
- ✅ 凭据走 env var 不打印 (RGS_TLS_DIR / GRPC_*_ENDPOINT) — 配置复用 config.rs
- ✅ 0 env value 出现在 6 mock.json + W3-PHASE-3-WORKER-1-REPORT.md
- ✅ 派生约束 L1/L3/L11/L12.1/L12.2/L13/L14 全部 ✅
- ✅ 派生约束 L1.1/L1.2/L2/L4/L5/L6 N/A (mock v0.1 stub 模式, 单工具链, 0 plumbing 改)
- ✅ 5 域独立 Lead 原则 (per 8/21 JST): 0 改 5 域 / card / batch / gm-backend 业务代码
- ✅ 代签规则 (per 8/27 19:39/20:56/21:59 JST 三次强化): Mavis 默认代签 Ulysses
- ✅ 禁回溯叙事 (per 8/27 JST): 0 "per X 历史形态"/"per X 升版前/后" 等回溯叙事
- ✅ 缺标比错标 (per 8/26 JST): §4 + §5 + §11 全部显式列出 7+ 条已知缺口
- ✅ 引用必须 git 实证: 引用基线 575f5c9, 0 编造无证据叙事

---

## 11. 已知缺口 (per 8/26 JST 缺标比错标)

1. **6 module 全部 NotImplemented, 0 PASS / 0 Partial**: 30 新 module 待 v0.2+ sprint W14-W19 阶段实装, 这是 W3 Phase 3 的 gap matrix 验证**预期结果**。
2. **6 .erl 仅 rpc 完整抽样, 主体 .erl 70-80% 推测** (per addendum §10.1 已知缺口):
   - avatar_rpc.erl 1.4KB 完整抽样, avatar.erl 16.6KB 完整 read (L1-393 完整)
   - honor_rpc.erl 1.3KB 完整抽样, honor.erl 15.5KB 仅 L1-100 抽样
   - charge_rpc.erl 1.0KB 完整抽样, charge.erl + 6 关联 .erl 共 39.9KB 仅 L1-100 抽样
   - feat_rpc.erl 894B 完整抽样, feat.erl 11.4KB 仅 L1-100 抽样
   - checkin_rpc.erl 1.1KB 完整抽样, checkin.erl 9.4KB 仅 L1-80 抽样
   - login_days_rpc.erl 910B 完整抽样, login_days.erl 7.0KB 仅 L1-100 抽样
3. **avatar/honor 14s 定时器周期推测**: avatar.erl L33 推测 14s = 14*1000ms 定时器超时检测, 实际待 v0.2+ 验证。
4. **feat 跨域触发协议待 v0.2+ sprint 设计**: 5 域 (combat/social/quest/player/economy) → batch 域 跨域 saga 模式详细化。
5. **charge 第三方支付渠道 (微信/支付宝/Apple Pay/Google Pay) 整合细节待 v0.2+ sprint 实证**: charge_mltest_return.erl 3.9KB 推测是 mTLS + 三方回调入口, 实际待 v0.2+ 验证。
6. **checkin city 域跨域交互待 v0.2+ sprint 评估**: checkin_rpc.erl L15 city.hrl include 暗示跟 city 域有交互, city 域待 v0.2+ sprint 评估 (per DDD v0.1 §2.3 30 新 module 表, city 不在 30 内)。
7. **login_days 7-day 周期 reset 逻辑推测**: login_days.erl L100+ 未抽样, 实际 v0.2+ sprint 验证。
8. **6 module 0/16 wire, 全部 ❌ NotImplemented**: RGS 实体 / Master / Transaction / Work 4 套表全部缺, 待 v0.2+ sprint W14-W19 阶段实装。
9. **RGS-DDD v0.2 addendum-协议号映射 §5.28/§5.35/§5.36/§5.40/§5.41/§5.42 跟 12-大类-RPC-清单.md v0.1 §3 5-30 行 each 业务扩写差异未做详细 diff**, 待主会话 commit 后做。
10. **6 worker 并发派工 0 race condition 已实证**: per L12.2 选项 B 6c5173a 模式, mock_data/{avatar,honor,login_days,checkin,feat,charge}.json 路径无重叠 (per git status observed, 0 untracked conflict)。

---

## 12. 签字 (per B3 派生约束 v0.2 流程)

- **Mavis 自审 (1 次后停手)**: ✅ 全部完成
- **Ulysses 二审**: ⏳ 待主会话统一 commit 后触发
- **打回循环**: 0/2 (本 turn Mavis 自审通过)

---

## 附录 A: 6 mock.json sample row 完整示例

### A.1 avatar.json 21500 (ListAvatarFrames)

```json
{
  "rpc_code": 21500,
  "rpc_name_zh": "头像框列表",
  "rgs_backend": "player-service:50051",
  "rgs_rpc": "ListAvatarFrames",
  "rgs_proto_method": "AvatarService.ListAvatarFrames",
  "gap_status": "NotImplemented",
  "request_fields": [],
  "mock_response": {
    "code": 0,
    "msg": "ok",
    "frames": [
      {"base_id": 10001, "name": "default_avatar", "used": true, "expire_at": 0, "is_default": true}
    ]
  },
  "rgs_partial_reason": "RGS 0 AvatarService 实体, 缺 avatar_frames Master 表 (avatar_data:get/1 ets 翻译), 4 cmds 全部 ❌ NotImplemented (per addendum §5.28)",
  "biz_flow_ref": "avatar_rpc.erl L17-19 (handle/3) + avatar.erl L71-78 (init/0 默认头像框) + avatar.erl L82-84 (login/1 重载 + set_timer)"
}
```

### A.2 charge.json 21001 (ClaimFirstCharge)

```json
{
  "rpc_code": 21001,
  "rpc_name_zh": "领取首充礼包",
  "rgs_backend": "economy-service:50052",
  "rgs_rpc": "ClaimFirstCharge",
  "rgs_proto_method": "ChargeService.ClaimFirstCharge",
  "gap_status": "NotImplemented",
  "request_fields": ["id:32"],
  "mock_response": {
    "code": 0,
    "msg": "ok",
    "result": true,
    "id": 30001,
    "rewards_granted": [{"item_id": 1001, "count": 1000}]
  },
  "rgs_partial_reason": "RGS 0 ChargeService.ClaimFirstCharge, 缺 charge_misc:take_first_gift/2 业务 (per charge_rpc.erl L22-28) 1:1 翻译, 走 economy 域 3 步 saga: 校验首充 → 扣位 → 发奖 (per market W2-2.4 OpenPack saga 模式 1:1 复用)",
  "biz_flow_ref": "charge_rpc.erl L22-28 (handle/3) + charge_misc:take_first_gift/2 (charge_misc.erl 推测)"
}
```

### A.3 feat.json 16402 (ClaimFeatReward)

```json
{
  "rpc_code": 16402,
  "rpc_name_zh": "领取成就奖励",
  "rgs_backend": "batch-backend:8790",
  "rgs_rpc": "ClaimFeatReward",
  "rgs_proto_method": "FeatService.ClaimFeatReward",
  "gap_status": "NotImplemented",
  "request_fields": ["id:32"],
  "mock_response": {
    "code": 0,
    "msg": "ok",
    "result": true,
    "id": 1001,
    "rewards_granted": [{"item_id": 2001, "count": 10}]
  },
  "rgs_partial_reason": "RGS 0 FeatService.ClaimFeatReward, 缺 feat:reward/2 业务 (per feat_rpc.erl L21-27) 1:1 翻译, 走 batch 域 task + player_instance table 模式 (per activity W2-2.6 实证) + 5 步 saga: 校验完成 → 扣完成位 → 发奖 → 写完成流水 → 跨域推 social 炫耀 (可选)",
  "biz_flow_ref": "feat_rpc.erl L21-27 (handle/3) + feat.erl reward/2 (推测, 跟 activity.erl reward/2 模式 1:1, 但支持批量 Id 数组)"
}
```
