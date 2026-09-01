# AGENTS.md — AI Agent 协作规则 (RustGameServer)

> **创建日期**: 2026-08-31 21:50 JST
> **创建者**: Mavis 接手 agent per DEC-008 (代签 Ulysses)
> **依据**: RGS-OPEN-QA-2026-08-31-test-summary v0.2 (commit `8da6695`) + 8/26-8/27 JST 派生约束
> **作用域**: 所有 AI agent (Mavis / 上游 AI / 下游 AI) 在本仓库工作时的强约束
> **优先级**: 仓库级 `AGENTS.md` 优先于任务级 prompt 简报

---

## 0. 仓库元信息

- **项目**: RustGameServer (分布式游戏服务器 Rust + gRPC)
- **架构**: **6 域** (player / economy / match / social / admin / **batch**) + 平台层 + 工具 crate, 6 域独立 Lead (per 2026-08-21 JST + 2026-09-01 18:00-19:24 JST batch 域扩展, per §7)
- **基线 commit**: 46dd2a0 (831) → 305f2cb (8/31 19:48 JST) → f5c0359 → 8da6695 → fd122f6 (REQ) → e366ff8 (BASIC) → 62027c9 (DETAILED) → e70ed71 (PLAN, 2026-09-01 19:24 JST batch 4 件套)
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

### 6.2 临时越界记录 (per 2026-09-01 09:45 JST Ulysses 决策 opt3 + 追认)

**背景**: 9/1 09:00-10:00 JST k3s 部署恢复期, postgres Deployment 缺 + initdb.sql 缺 CREATE USER, 5 域 svc 全部 CrashLoopBackOff 13 次 (DB pool timeout)。

**Mavis 临时越界** (v0.3 §7.5 ❌ 改 yaml 边界外):
- 改 `docs/deploy/01-k8s-manifests/22-postgres-configmap.yaml` initdb.sql: 加 `CREATE USER + GRANT` (6 域 user, password `ulysses_local`)
- 改 `crates/player-service/migrations/0004_player_characters_inventory.sql`: 拆 cross-table FK (line 93 forward ref) → DO 块 + ALTER TABLE 在表建好后加

**Ulysses 追认**: per ask_user opt3 决策 ("Mavis 改 22-postgres-configmap.yaml initdb.sql + apply, Mavis 临时越界, 你追认")

**保留派生约束**:
- 临时越界仅限部署恢复紧急路径, **不允许扩展到日常 commit / feature dev**
- 越界后必须 24h 内 commit + 在 PR 描述 + 修订历史写明 "临时越界 + Ulysses 追认"
- 不追溯改写历史文档中的"审批者=—" (per 8/27 19:39 JST 决策)

**AGENTS.md 后续工作**:
- v0.4 正式纳入"部署恢复期临时越界许可"流程 (Mavis 上报 + Ulysses 决策 + 24h 内 commit 三件套)
- v0.4 追加 m4 forward ref FK 案例到 §2 Worker 工作流规则 (新派生约束 L7)

### 6.3 PT 派工派生约束 (per 9/1 14:15-15:10 JST 8 worker 派工, commit ffbfb19)

**背景**: 9/1 14:15 JST Ulysses 决策"5 平台层 (130 .rs) + 3 工具 (92 .rs) 拆 worker 派工", 派工 5 平台 (1 crate 1 worker) + 3 工具 (按业务相关性合并, 3 crate 1 worker) = 8 worker. 8 worker 25 min 全交付 (vs 8/31 5 worker 0 产出 4h), 验证 v3 hotfix 模板化复制有效.

**新增派生约束 L11/L12** (per DDD Review v0.1 §5.11/§5.12):

#### L11: PT 派工 cargo build dir lock 防御

- **教训**: 8 worker 同时跑 `cargo check --tests`, 8 cargo 进程并发竞争 target/ build dir lock, 多个 worker 报告"等待多轮编译"
- **强约束**: **PT 派工简报** 必须明文 "DoD = cargo check --tests 1 次拿 status, 修到 0 error, 不要 polling 多轮编译", 避免 8 cargo 进程互锁
- **依据**: 9/1 14:15-15:10 JST 8 worker 25 min 完工 (vs 8/31 5 worker 0 产出 4h), 8 worker 报告都提到"等待多轮编译"但 0 死锁
- **检查工具**: worker 报告里 grep `Waiting for|build dir lock` 应该 < 5 次
- **配套**: 简报明文 "30 min 必须出 commit, 失败也没关系, 占位 commit 也行", 避免 worker 长时间 polling 编译

#### L12: PT 派工临时 log 不入 commit 防御

- **教训**: 8 worker 在 worktree 根写 .log / .txt 临时文件 (cargo-check.log / commit-msg.log / COMMIT_MSG_TMP.txt / .tmp_search_backup), 未跟踪但污染 worktree
- **强约束**: **PT 派工简报** 必须明文 "临时 log / .txt / .tmp_search* 不入 commit, 主会话 merge 后清理", 避免 8 worktree 7 个临时文件残留
- **依据**: 9/1 14:15-15:10 JST 8 worker 临时文件: shared-platform 1 / cluster-ops 1 / rgs-testkit 2 / leaderboard-overflow-asset 5, 都没入 commit, 但 7 worktree 根污染
- **检查工具**: `git status` 在 merge 前应该 0 untracked (除了 .gitignore 没列的临时 log)
- **主会话清理**: merge 前 `git worktree remove --force` + `git worktree prune` 批量清理 8 worktree

#### L9 流程化: 临时越界 (Mavis) + 追认 (Ulysses) 三件套

- **背景**: 9/1 部署恢复期, Mavis 改 yaml (22-postgres-configmap + m4) 越 v0.3 §7.5 边界, Ulysses 决策 opt3 追认
- **完整流程** (per AGENTS.md v0.2 §6.2):
  1. **Mavis 上报**: ask_user 给 Ulysses 选项 (不能只问"可以吗", per 9/1 14:58 JST Ulysses 反馈)
  2. **Ulysses 决策**: opt3 (Mavis 改 + 你追认) / opt1 (SRE 介入, Mavis 退出) / opt2 (Mavis apply + 临时越界)
  3. **Mavis 改**: 改 yaml + apply (具体改法由 Mavis 直接落地, 不需要再问)
  4. **24h 内 commit + 修订历史写明**: "临时越界 + Ulysses 追认" 三行齐全
  5. **AGENTS.md 同步**: v0.x 加 §6.2 临时越界记录, 记入派生约束
- **不允许扩展到**: 日常 commit / feature dev / 业务实装 (仅限部署恢复紧急路径)
- **追溯改写**: 不追溯改写历史文档"审批者=—" (per 8/27 19:39 JST 决策)

#### PT 派工简报标准模板 (per v0.3 §6.1 + L11 + L12)

```markdown
# 任务简报 - <category> 域 <task>

## 工作环境
- worktree: D:/rgs-<prefix>-<scope>
- 分支: <prefix>/<scope> (基线 <baseline>)
- 负责 crate: crates/<scope> (其他 crate 不动)

## 必做
1. 读 <briefing> + AGENTS.md §6 模板 + DDD Review §<N>
2. 探索: Get-ChildItem crates/<scope> -Recurse -Filter *.rs
3. 写 UT 优先 (沿用 InMemory mock 风格, rgs-testkit 禁 InMemory, 用 NoOp)
4. 写 IT 优先 (跨模块场景)
5. **验证**: cd D:/rgs-<prefix>-<scope>; cargo check -p <scope> --tests 2>&1 | tail -20
   (1 次拿 status, **不要 polling 多轮**, per L11)
6. 0 error → git add + commit (代签格式 per 8/27 JST)
7. **(可选)** git push origin <prefix>/<scope>

## DoD
- ✅ cargo check --tests 60s 内通过
- ✅ commit 1+ 段带代签 (代签/审批/修订人 三行齐全)
- ✅ 1 域 1 worker 不交叉改 (5 域独立 Lead 原则)
- ✅ **不动** AGENTS.md / DDD Review / OPEN-QA / manifests (主会话负责)
- ✅ **临时 log / .txt / .tmp_search* 不入 commit** (per L12)
- ✅ **30 min 必须出 commit**, 失败也没关系, 占位 commit 也行

## 卡住的应对
- cargo check 超 60s → 接受 warning, 先 commit 占位
- 找不到合适 mock → 复用 src/ 已有 InMemory*Repository
- **不要 Start-Sleep 轮询等编译** (per L11)
- 单 commit 跨多个 crate → 不允许
```

### 6.4 模板 (per ST worker / 主会话 ST)

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

## 7. batch 域派生约束 (per 2026-09-01 18:00-19:24 JST)

**触发**: per Ulysses 2026-09-01 18:00 JST "batch 需要一个专门的管理界面和对其支持的前后端功能, 应该是一个独立的项目, 但可以按照其他功能的方式融入架构, 从需求文档开始设计" + 18:25 JST 范围澄清 "所有内容的批量, 包括但不限于 log/数据整理" + 18:34 JST Q2 拍板 "独立双项目" + 19:00 JST 继续 PLAN 决策。

**4 文档落地 commit**: REQ `fd122f6` + BASIC `e366ff8` + DETAILED `62027c9` + PLAN `e70ed71`(per OLU-WEB 4 件套范式, 2026-09-01 18:00-19:24 JST)。

### 7.1 batch 域项目形态 (per Q2 拍板)

| 项目 | 路径 | 技术栈 | 部署 |
|---|---|---|---|
| **rgs-batch-console** (前端) | `tools/rgs-batch-console/` | Node 22 + 原生 http + 0 依赖 | envoy 独立 deployment + ClusterIP service (per 9/1 13:03/13:05 JST) |
| **rgs-batch-backend** (后端) | `tools/rgs-batch-backend/` | Rust + actix-web 4 + tokio + tonic 0.12 gRPC client + sqlx 0.7 + mTLS 业务级 | envoy 独立 deployment + ClusterIP service |
| **端口** | console 127.0.0.1:8789 (区别 rgs-web 8788), backend ClusterIP 0.0.0.0:8790 | | |

**对标**: rgs-web (Node) + gm-backend (actix-web) 模式, 但**项目命名空间独立** (rgs-batch-*, 不与现有 tools 冲突)。

### 7.2 batch 域核心约束 (12 条派生约束)

per 2026-09-01 18:00-19:24 JST Ulysses 决策 + 5 域独立 Lead 原则 + DB 横展三分类 + env value 硬 ban + envoy 独立 deployment 偏好:

| # | 约束 | 说明 | 引用 |
|---|---|---|---|
| 1 | **5 域独立 Lead → 6 域扩展** | batch 域不与 5 域 Lead 兼任 (per 8/21 JST 拒绝兼任), 扩展 5 域 → 6 域 + 1 batch Lead (per REQ F-16 + R-5) | REQ §0 |
| 2 | **DB 三分类横展** | Master / Transaction / Work 三分清晰 (per 9/1 18:30 JST), 16 张表 schema 划分 `batch_master` / `batch_transaction` / `batch_work` / `batch_transaction_archive` | REQ §4 |
| 3 | **env value 硬 ban** | 凭据走 env var, 永不打印值 (per 8/27 11:06 JST), 包括 BATCH_DB_PASSWORD / GRPC_CERT_PATH_* (5 域) / GRPC_CLIENT_KEY / trace_id | REQ NFR-30 |
| 4 | **mTLS 业务级** | 5 域 gRPC 调用走 mTLS (per 5 域 ST 业务级 mTLS 实践 commit `401ac5c`), 证书复用 8/27 ST 导出 SOP | REQ NFR-32 |
| 5 | **envoy 独立 deployment** | 不选 nginx, 不选 istio sidecar (per 9/1 13:03/13:05 JST), rgs-batch-console / rgs-batch-backend / envoy 三个独立 deployment | REQ IR-8 |
| 6 | **127.0.0.1 only** | rgs-batch-console 监听 127.0.0.1:8789 (per rgs-web 母规范 + NFR-31) | REQ F-12 |
| 7 | **0 依赖 Node** | 沿用 rgs-web 母规范, 不引入 Express / Koa / Fastify / React / Vue / chart.js / d3 / dhtmlx-gantt | BASIC §2.1 |
| 8 | **1 写者约束** | rgs-batch-backend 单进程 + tokio multi-thread runtime (per OLU-WEB §5.1.5), sqlx 默认 1 写者安全 | BASIC §2.2 |
| 9 | **rgs-testkit 禁 InMemory** | 跨工具链决策前先 grep workspace 依赖 (per AGENTS.md §2.3 L3), 用 NoOp + 真实 sqlx + 5 域 gRPC client mock | DETAILED §7.1 |
| 10 | **audit 永久保留** | audit_event T-3 永久保留 (per NFR-29), 操作人 / 时间 / 参数 hash / 结果 / trace_id 全记录 | REQ F-10 |
| 11 | **代签规则** | Mavis 默认代签 Ulysses (per 8/27 19:39/20:56/21:59 JST 三次强化), author=Ulysses / 审批=架构师 / 修订人=Ulysses — Mavis 接手 | REQ §0 |
| 12 | **saga 集成** | v0.1 不集成, saga-runtime 独立 Pod (per RGS-BAS-100 v0.1), v0.2 评估跨域 saga 触发 (per REQ GAP-11) | REQ GAP-11 |

### 7.3 batch 域已知缺口 (per 2026-09-01 18:30 JST 缺标比错标)

- **GAP-1 ~ GAP-12** (12 缺口) per RGS-BATCH-REQUIREMENTS-2026-09-01 v0.1 §9 (跨 batch DAG / WebSocket / 流式 / mavis cron 告警 / 任务优先级 / AI 协助 SQL / rgs-web 深联动 / 任务模板版本化 / Rollback SQL 验证 / 任务超时 kill / 跨域 saga 触发 / batch 域 Lead RACI 同步)
- **RACI v1.2** 扩展 5 域 → 6 域 (待 DDD Review 阶段, per PLAN A.3)
- **IMPL-PLAN-BATCH-001 v0.1** 起草 (per 5 域 IMPL-PLAN 范式, per PLAN §A.3)
- **RACI-BATCH-V1 v0.1** 独立 RACI 文档 (per 5 域独立 RACI 范式, per PLAN §A.3)
- **v0.2 评估** (per PLAN §A.3): Log 批量 + 数据整理 + DAG + WebSocket + mavis cron 告警 + AI 协作 + rgs-web 深联动 + 证书轮换 + dry-run + 任务超时强制 kill
- **k3s 资源上限 + namespace 隔离策略** (per REQ §10.3 待协调)
- **5 域 binary 未来调外部 LLM 未登记** (v0.1 不集成, v0.2 评估 per OLU-WEB F-25)

### 7.4 batch 域 brief 模板 (per OLU-WEB 范式 + 8/27 JST 代签格式)

```markdown
# 任务简报 - batch 域 <task>

## 工作环境
- worktree: D:/rgs-ut-batch (或 D:/rgs-st-batch)
- 分支: <prefix>/batch (基线 46dd2a0 / fd122f6)
- 负责 crate: tools/rgs-batch-console (前端) 或 tools/rgs-batch-backend (后端)

## 必做
1. 读 <briefing> + AGENTS.md §7 batch 域派生约束 + RGS-BATCH-*-DESIGN-2026-09-01 v0.1
2. 探索: Get-ChildItem tools/rgs-batch-{console,backend} -Recurse
3. 写实现 (沿用 OLU-WEB 范式: rgs-testkit 禁 InMemory, 用 NoOp + 真实 sqlx + 5 域 gRPC client mock)
4. **验证**: cd D:/rgs-<prefix>-batch; cargo check -p rgs-batch-backend --tests 2>&1 | tail -20
   (1 次拿 status, **不要 polling 多轮**, per L11)
5. 0 error → git add + commit (代签格式 per 8/27 JST)
6. **(可选)** git push origin <prefix>/batch

## DoD
- ✅ cargo check --tests 60s 内通过 (1 次拿 status)
- ✅ commit 1+ 段带代签 (代签/审批/修订人 三行齐全)
- ✅ 1 worker 1 域不交叉改 (6 域独立 Lead 原则: 5 域 + batch)
- ✅ **不动** AGENTS.md / RGS-BATCH-* 文档 / 5 域代码 / manifests (主会话负责)
- ✅ **临时 log / .txt / .tmp_search* 不入 commit** (per L12)
- ✅ **30 min 必须出 commit**, 失败也没关系, 占位 commit 也行
- ✅ **凭据永不打印** (per 8/27 11:06 JST 硬 ban + REDACTED filter, per DETAILED §5.1)
- ✅ **DB 表归类 Master/Transaction/Work** (per 9/1 18:30 JST 横展)

## 卡住的应对
- cargo check 超 60s → 接受 warning, 先 commit 占位
- 找不到合适 mock → 复用 src/ 已有 InMemory*Repository 或 rgs-testkit NoOp
- **不要 Start-Sleep 轮询等编译** (per L11)
- 5 域 gRPC 调用失败 → retry 3 次 (指数退避 100/200/400ms) + DLQ, 不静默失败
- 单 commit 跨多个 crate → 不允许
- env value 出现在日志 → 立即用 REDACTED filter (per DETAILED §5.1)
- 任务撤销原子性 → 仅未执行 + 未生效可撤销 (per F-21)
- batch 域 Lead 兼任 5 域 → 立即拒绝, per 8/21 JST 拒绝兼任基线
```

### 7.5 batch 域与已有架构的集成关系 (5 不破坏 + 4 复用 + 3 引用)

**5 不破坏** (per BASIC §6.2):
- ❌ 不破坏 5 域架构: rgs-batch-backend 作为 gRPC 客户端调用 5 域, **不修改** 5 域代码
- ❌ 不破坏 rgs-web: rgs-batch-console 独立 Node 项目, **不嵌入** rgs-web
- ❌ 不破坏 shared-platform: 复用现有 crate, **不修改** shared-platform 代码
- ❌ 不破坏 function-plane: v0.1 不集成 (saga-runtime 独立 Pod per RGS-BAS-100), **不修改** function-plane 代码
- ❌ 不破坏 gm-backend: rgs-batch-console 跟 gm-console 形态不同, 但都是 envoy 独立 deployment

**4 复用** (per BASIC §6.2):
- ✅ rgs-web 母规范 5 份: 0 依赖 + 127.0.0.1 only + 30s 轮询 + JSON 响应
- ✅ rgs-web OLU-WEB 4 份: data/ 目录 + lockfile + token-estimate + ai-ledger.jsonl
- ✅ gm-backend 范式: actix-web + mTLS + 8443 HTTPS APIGW
- ✅ 5 域 ST 业务级 mTLS 实践 (commit `401ac5c`): 证书 + 双向认证 + 8/27 ST 导出 SOP

**3 引用** (per BASIC §6.2):
- 🔗 shared-platform 20 模块: outbox + tracing + span_helpers + retry + dlq + grpc_tracing + rbac + tls + ...
- 🔗 5 域 gRPC client: player / economy / match / social / admin 50051-50055 (k8s targetPort)
- 🔗 saga-runtime 独立 Pod (per RGS-BAS-100 v0.1, v0.2 集成)

### 7.6 实施 WBS 入口 (per PLAN v0.1)

38 L4 任务 (W1 6 + W2 7 + W3 8 + W4 7 + W5 5 + W6 5), 54 人·天 / 9.65M tokens, 6 周落地. 完整 WBS 任务表 per RGS-BATCH-PLAN-2026-09-01 v0.1 §3 (commit `e70ed71`).

---

## 8. 修订历史

| 版本 | 日期 (JST) | 修订人 | 变更 |
|---|---|---|---|
| v0.1 | 2026-08-31 21:50 | 架构师(Mavis 接手 agent per DEC-008) | 初始创建, 摘录 OPEN-QA v0.2 L1-L5 规则 + 5 域 Lead 流程 + 任务级 prompt 模板 |
| (待续) | — | — | Q1-Q11 业务实现落地后, 追加 "业务级实施跟踪" 段 |
| v0.2 | 2026-09-01 10:00 | 架构师(Mavis 接手 agent per DEC-008) | 9/1 k3s 部署恢复期: 加 §6.2 临时越界记录 (Ulysses opt3 追认), 22-postgres-configmap initdb.sql + m4 forward ref FK 两处临时越界 |
| v0.3 | 2026-09-01 16:00 | 架构师(Mavis 接手 agent per DEC-008) | 9/1 PT 派工 8 worker 完结 (commit ffbfb19) + 5 域 ST 业务级 mTLS 全完成 (commit 401ac5c): 加 L9/L11/L12 派生约束 (临时越界流程化 + cargo build dir lock 防御 + 临时 log 不入 commit 防御) |
| v0.4 | 2026-09-01 19:24 | 架构师(Mavis 接手 agent per DEC-008) | 9/1 batch 域 4 件套落地 (REQ `fd122f6` + BASIC `e366ff8` + DETAILED `62027c9` + PLAN `e70ed71`, 2576 行 / 165.2 KB / 23 处自审 fix): §0 元信息更新 5 域→6 域 + §7 新增 batch 域派生约束 (12 条约束 + 已知缺口 + brief 模板 + 5 不破坏 + 4 复用 + 3 引用 + 38 L4 任务入口) |

**修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
**审批**: 架构师(Mavis 接手 agent per DEC-008)
**代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)
