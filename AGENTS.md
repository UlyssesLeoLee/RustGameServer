# AGENTS.md — AI Agent 协作规则 (RustGameServer)

> **创建日期**: 2026-08-31 21:50 JST
> **创建者**: Mavis 接手 agent per DEC-008 (代签 Ulysses)
> **依据**: RGS-OPEN-QA-2026-08-31-test-summary v0.2 (commit `8da6695`) + 8/26-8/27 JST 派生约束
> **作用域**: 所有 AI agent (Mavis / 上游 AI / 下游 AI) 在本仓库工作时的强约束
> **优先级**: 仓库级 `AGENTS.md` 优先于任务级 prompt 简报

---

## 0. 仓库元信息

- **项目**: RustGameServer (分布式游戏服务器 Rust + gRPC)
- **架构**: 5 域 (player / economy / match / social / admin) + 平台层 + 工具 crate, 5 域独立 Lead (per 2026-08-21 JST)
- **基线 commit**: 46dd2a0 (831) → 305f2cb (8/31 19:48 JST) → f5c0359 → 8da6695
- **代签规则**: 修订人 = Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手;审批 = 架构师(Mavis 接手 agent per DEC-008) (per 2026-08-27 19:39/20:56/21:59 JST 三次强化)

---

## 1. 强约束 (Hard Constraints, per 8/26-8/27 JST 派生)

### 1.1 文档治理 (per 8/26 JST)

| 约束 | 说明 | 引用 |
|---|---|---|
| **缺标比错标安全** | 拿不准时显式列"已知缺口", 不假装覆盖 | RGS-OPEN-QA-2026-08-27-k3s-deploy v0.4 §0 |
| **引用必须 git 实证** | 所有 commit SHA / file:line / 时间戳必须可独立 `git log` / `Read` 验证 | RGS-OPEN-QA-2026-08-27-k3s-deploy v0.4 §0 |
| **禁回溯叙事** | 禁止"per X 历史形态""per X 升版前/后""原本是"等无 git 历史证据的回溯叙事 | per 8/26 JST DTL-036 v1.4 hotfix 复盘 |
| **代签规则已反转** | Mavis 默认代签 Ulysses (per 8/27 19:39/20:56/21:59 JST 三次强化), 修订历史"审批者"列可填 Mavis 真实责任, 但禁止编造无证据叙事 | 8/27 21:59 JST Ulysses 第三次强化 |

### 1.2 环境变量安全 (per 8/27 11:06 JST hard ban)

| 约束 | 说明 |
|---|---|
| **禁止打印 env 值** | `Get-ChildItem env:` 表格、`echo $VAR`、`$env:X expand`、`cat .env` 等所有可能泄露 secret 的操作禁止 |
| **只可 invoke** | `$env:VAR` 引用后直接 pipe (如 `$env:UbuntuPW \| wsl -e bash -c '...'`), 或传给程序参数 |
| **反例** | `Get-ChildItem env: \| Format-Table` 输出 `<value>` 即违规 |

### 1.3 Windows Shell 约束 (per 当前环境 win32)

| 约束 | 说明 |
|---|---|
| **PowerShell only** | 不混 bash/sh, 不在脚本中用 `&&` `ls -la` `head` `tail` `grep` `wc` |
| **正确 cmdlet** | `Get-ChildItem` / `Select-Object` / `Select-String` / `Measure-Object` |
| **PSNativeCommandUseErrorActionPreference** | 调原生程序时显式设置 `= $true` 或检查 `$LASTEXITCODE` |
| **Get-Content 编码** | 读 UTF-8 必须 `-Encoding UTF8` |
| **失败快速** | 多语句脚本起 `$ErrorActionPreference = 'Stop'` |

---

## 2. Worker 工作流规则 (per 8/31 测试阶段 4 阶段迭代教训)

### 2.1 L1 + L2 合并: Cargo 编译/测试策略

**强约束**: **Worker 必须在提交前跑至少一次 `cargo check --tests` (限时 60s 内应出结果) 作为编译验证下限, 不允许跳过验证直接 commit; 但不得要求跑完整 `cargo test`**。

**反面**:
- ❌ 简报里写 "DoD = cargo test 全过" → worker 卡在长编译 polling 循环
- ❌ 简报里写 "DoD = 不跑 cargo, 只写不验" → worker commit 38 编译错误
- ❌ Worker 用 `Start-Sleep + Get-Process cargo` 轮询 → 反 pattern, 浪费轮次

**正面**:
- ✅ 简报里写 "DoD = cargo check --tests 通过" (快, 几秒)
- ✅ Worker 触发编译后直接返回, 等任务完成信号
- ✅ 最终 `cargo test` 由主会话在 worker 全部完成后统一跑

**证据**: 8/31 UT v1 (5 worker polling 失败) + UT v2 (38 编译错误) + UT v3 hotfix (全部 cargo check 0 error)

### 2.2 L3: 跨工具链决策前先查 workspace 依赖

**强约束**: **跨工具链决策 (mock server / testkit / 外部依赖选型) 前必须先 `grep` workspace `Cargo.toml` 确认依赖是否存在 + 阅读相关强约束文档段落 (如 `crates/rgs-testkit/src/lib.rs` §唯一接受的 API), 禁止拍脑袋假设依赖可用**。

**反例**: 8/31 ST 阶段第一轮决策"新 mock server binary", 实际 workspace 无 axum/hyper/warp, 且 `rgs-testkit` 强约束禁 InMemory mock → 浪费 ~20 min 切到 k3s 真实部署

**checklist**:
- 写 `mock server` 决策前: `grep -E 'axum|hyper|warp|actix|rocket' Cargo.toml`
- 写 `InMemory mock` 决策前: `Read crates/rgs-testkit/src/lib.rs` L17-34 (强约束段)
- 写 `k3s/sudo/wsl` 集成决策前: 确认 `wsl --status` + `kubectl get nodes` 可达

### 2.3 L4: 跨多工具链场景先主会话打头阵

**强约束**: **跨多工具链场景 (WSL + sudo + k3s + 多域 + 外部脚本) 不直接派 worker 从 0 探索; 主会话先打头阵跑通 1 条完整链路, 产出可复用模板后, 再派 worker 做模板化复制 (改 probe 列表 / 改域名等参数化工作)**。

**证据**: 8/31 ST 阶段派 5 worker 写 ST 脚本, 5 worker 全部 0 产出 (跟 UT v1 player 同症); 主会话自写 10 脚本后 40 min 跑通

**checklist**:
- 任务涉及 ≥2 个工具链 (e.g. k3s + WSL + sudo + 5 域 binary) → 主会话先跑
- 单工具链任务 (e.g. cargo test / 写脚本 / grep) → 可直接派 worker

### 2.4 L5: ST worktree 启动 checklist

**强约束**: **ST 阶段启动 checklist 新增: 路径选择前先 `grep` k8s secret 位置 + 证书导出 SOP 是否存在; 5 域 mTLS 业务级 ST 需要的证书导出必须列入 ST worktree 初始化步骤, 不能等到写测试时才发现缺失**。

**ST 启动 checklist** (per 8/31 教训):
1. `git worktree add -b st/<feature> <wt> main`
2. `grep -r 'tls\|mTLS' docs/deploy/01-k8s-manifests/` 确认证书位置
3. `kubectl get secret <domain>-tls -n rust-game-server -o yaml > certs/<domain>-tls.yaml` (需集群可达)
4. 验证 e2e-smoke baseline: `pwsh scripts/e2e-smoke.ps1 -Json` (12 probe 应有 ≥10 PASS)
5. 写第 1 个 ST 场景, 跑通后 commit, 再扩展

### 2.5 L6: ST FAIL 排查顺序

**强约束**: **ST 阶段 FAIL 不能直接归咎测试代码, 先对照 e2e-smoke baseline (12 probe 基线) 排除是否为已知基础设施问题; k3s 容器 HTTP 不响应类问题应先查 pod 重启次数/events (与历史 HPA minReplicas 强启动风暴问题同类特征), 而非默认怀疑 binary 逻辑**。

**ST FAIL 排查 checklist**:
1. `pwsh scripts/e2e-smoke.ps1 -Json` → 12 probe baseline 状态
2. 跟 ST 场景 probe 对照: baseline 7/5 PASS/FAIL → ST FAIL 是否新增
3. 如 baseline 已 FAIL → 跳过测试, 转 k3s 容器诊断
4. k3s 诊断: `kubectl get pods -o jsonpath='{.items[*].status.containerStatuses[*].restartCount}'` + `kubectl describe pod` + `kubectl logs` + `kubectl exec curl localhost:port`
5. 历史经验: HPA minReplicas 强启动风暴会导致 SandboxChanged 风暴, 容器在跑但 HTTP 不响应, 表现与 Q8 一致

---

## 3. 5 域独立 Lead 流程 (per 8/21 JST 决策)

| 域 | Lead 责任 | UT commit | IT commit | RACI 文档 |
|---|---|---|---|---|
| player | Mavis 接手代签 | `3cfeedb` (137 tests) | `bd83fb3` (12 tests) | `RGS-RACI-PLAYER-V1_v1.1.md` |
| economy | Mavis 接手代签 | `1db3249` (~82 tests) | `afd3d65` (20 tests) | `RGS-RACI-ECONOMY-V1_v1.1.md` |
| match | Mavis 接手代签 | `5070547` (28+ tests) | `c70ef64` (7 tests) | `RGS-RACI-MATCH-V1_v1.1.md` |
| social | Mavis 接手代签 | `3e456b4` (47 tests) | `3f41626` (9 tests) | `RGS-RACI-SOCIAL-V1_v1.1.md` |
| admin | Mavis 接手代签 | `04a9838` (13+ tests) | `67f82d6` (11 tests) | `RGS-RACI-ADMIN-V1_v1.1.md` |

**5 域并行工作流** (per 8/31 经验):
1. 每个域独立 worktree (`D:/rgs-ut-<domain>`)
2. 5 worker 并行派工, 每 worker 1 域
3. 域独立校验: `git diff --name-only 46dd2a0..HEAD -- 'crates/*' | grep -v '^ crates/<own-domain>/'`
4. 主会话统一 `cargo check -p <domain>-service --tests` 验证
5. 5 域全部通过后, 5 个 `--no-ff` merge 到 main

---

## 4. 关键决策记录 (per 8/31 OPEN-QA v0.2)

### 4.1 admin 域 Q1-Q2 决策

- **Q1 gm_handlers RBAC**: handler 入口补, 不下沉 trait; IT 为主 (`issue_gm_command_with_rbac` 转正), UT 补 role_matrix
- **Q2 audit_log startup verify**: **增量 verify** (最近 1000 条 / 24h), **非全表**; 真实篡改 fail-closed, infra 失败 warning + 继续

### 4.2 social 域 Q5-Q7 决策

- **Q5 guild capacity 50 vs 64**: 代码现状 50 为准, 不擅自改 64, 转 social Lead 业务确认
- **Q6 leave_guild**: PH-6 社交域下一轮实现, leadership 转移规则 = 加入时间最早剩余成员, 离开后 `player.profile.guild_id` 置空
- **Q7 push_delivery dispatcher**: 走 NATS (不新增 FCM/APNs 直连), retry 复用 economy outbox+saga 模式, 需要 DLQ

### 4.3 ST 阶段 Q8-Q11 决策 (待集群可达后执行)

- **Q8 gm-backend 8081**: 需 k3s 集群诊断 (上游决策见 `RGS-AI-HANDOFF-DOWNSTREAM-2026-08-31.md` §1.1)
- **Q9 prometheus/grafana**: 同 Q8 (§1.2)
- **Q10 mTLS 业务级 ST**: 工具链 = **grpcurl** (§2), 证书导出 + 实际重跑待集群可达
- **Q11 NATS 部署范围**: 一条 `kubectl get pods` 核查 (§1.3)

### 4.4 教训决策 (L1-L6, 本节 §2 已落)

---

## 5. ST 阶段 v0.2 commit + DDD Review 一审材料 (per 8/31 19:48 JST)

| 文档 | commit | 关联 |
|---|---|---|
| DDD Review UT+IT | `bd0884f` (docs/ddd-review/) | `RGS-DDD-2026-08-31-UT-IT_v0.1.md` |
| DDD Review ST | `bd0884f` (docs/ddd-review/) | `RGS-DDD-2026-08-31-ST_v0.1.md` |
| OPEN-QA v0.1 | `f5c0359` | 11 P1 + 6 教训汇总 |
| OPEN-QA v0.2 (上游 AI 决策) | `8da6695` | Q1-Q11 + L1-L5 决策 |
| AI Handoff Downstream | `8da6695` (docs/deploy/) | `RGS-AI-HANDOFF-DOWNSTREAM-2026-08-31.md` |
| 5 域 merge | `329d129` `7e76a7b` `73fd9b8` `103481a` `69d8c0a` | ut/<domain> 5 域 |
| ST merge | `305f2cb` | st/mock-server-and-scripts |

---

## 6. 任务级 prompt 简报模板 (per Mavis 8/31 经验)

### 6.1 模板 (per 域 worker)

```markdown
# 任务简报 - <domain> 域 <task>

## 工作环境
- worktree: D:/rgs-<prefix>-<domain>
- 分支: <prefix>/<domain> (基线 46dd2a0)
- 负责 crate: crates/<domain>-service

## 必做
1. 读 <briefing 文件> 完整内容
2. 探索: Get-ChildItem crates/<domain>-service/src -Recurse -Filter *.rs
3. 写 <产出>
4. 验证: cd D:/rgs-<prefix>-<domain>; cargo check -p <domain>-service --tests 2>&1 | tail -20
5. 修到 0 error 后 commit (代签格式 per 8/27 JST)

## DoD
- ✅ 必跑 `cargo check --tests` (限时 60s) - 禁止 `cargo test` 长编译
- ✅ commit 1+ 段带代签
- ✅ 域内不破坏

## 卡住的应对
- cargo check 超 60s → 接受 warning, 先 commit 占位
- 找不到合适 mock → 复用 src/ 已有 InMemory*Repository
- 等编译不要用 Start-Sleep 轮询
```

### 6.2 模板 (per ST worker / 主会话 ST)

```markdown
# ST 任务简报

## 路径
- 走 8/27 JST k3s 真实部署, 复用 scripts/e2e-smoke.ps1 12 probe 框架
- 不写新 mock server binary (per AGENTS.md §2.3 L3)

## 工作流
1. 主会话先跑通 1 个 ST 场景 (per AGENTS.md §2.4 L4 + L4 checklist)
2. 写 N 个 ST 场景 (e2e-smoke 复用 + gm-backend health)
3. 每场景 4 文件: ps1 + mock.json + .log + .md evidence
4. 失败就修脚本, 不改 e2e-smoke.ps1

## DoD
- ✅ N 场景 verdict=PASS
- ✅ 4 文件齐全
- ✅ commit 1+ 段带代签
```

---

## 7. 修订历史

| 版本 | 日期 (JST) | 修订人 | 变更 |
|---|---|---|---|
| v0.1 | 2026-08-31 21:50 | 架构师(Mavis 接手 agent per DEC-008) | 初始创建, 摘录 OPEN-QA v0.2 L1-L5 规则 + 5 域 Lead 流程 + 任务级 prompt 模板 |
| (待续) | — | — | Q1-Q11 业务实现落地后, 追加 "业务级实施跟踪" 段 |

**修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
**审批**: 架构师(Mavis 接手 agent per DEC-008)
**代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)
