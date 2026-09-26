# ULYS-94 子代理 brief — WSL Postgres + k3s + 12 fixture + e2e-smoke 12 探针

> 创建时间: 2026-09-20 14:35 JST
> 创建者: Hermes Agent (MinimaxM3, c557dae5), per D-Boy 05:17 拍板授权
> 父 issue: ULYS-112 (coordination)
> 目标 issue: ULYS-94 (01a0b7b2-fd50-7037-b43a-8f648bb0e2d6)
> 关联: ULYS-95 (admin-service migration 0005 fix 已合 dev), ULYS-111 (e2e-smoke 5 FAIL fix 已合 dev)
> 任务状态: in_progress (前序 GPT5.6terra task 05:29 cancelled due to 0 output after 3h)

## 上下文

ULYS-81 全量回归发现两类基础设施级阻断项:

1. WSL Postgres 冷启动 (历史 9/16 同样问题)
2. WSL k3s 集群启动

ULYS-94 任务范围: 验证 Step 1+2+3 全过。

**前序状态 (2026-09-20 14:30 JST 实测)**:

- WSL Ubuntu: 运行中 (1 day+ uptime)
- postgresql: `active`
- k3s: `activating` (需 step 3 验证是否已就绪)
- 代码状态: ULYS-90/91/92/93 fixes (player/economy/network-gateway regression bugs) 已合 dev; ULYS-95 admin-service migration 0005 fix 已合 dev; ULYS-111 e2e-smoke 5 FAIL fixes 已合 dev (commit 8b44bf2 + 6bd9f65)

## 任务范围 (4 步)

### Step 1: WSL Ubuntu 实例 + postgresql + k3s 启动验证

```bash
wsl -d Ubuntu -e bash -c 'uptime; systemctl is-active postgresql k3s; sudo -u postgres psql -c "SELECT 1" -tA'
```

预期: uptime > 1 day; postgresql `active`; k3s `active` 或 `activating`(秒级就绪); PG SELECT 返回 1。

**若 postgresql/k3s 仍是 inactive**: 启动之:

```bash
wsl -d Ubuntu -e bash -c 'sudo systemctl start postgresql; sudo systemctl start k3s'
```

### Step 2: 跑 12 个 DB fixture 集成测试

```bash
cd D:/RustGameServer  # 在你分配的 worktree 里
cargo test -p player-service  --test integration_player_basic  --no-fail-fast -j 4
cargo test -p economy-service --test integration_economy_basic --no-fail-fast -j 4
cargo test -p match-service    --test integration_match_basic    --no-fail-fast -j 4
cargo test -p social-service   --test integration_social_basic   --no-fail-fast -j 4
cargo test -p admin-service    --test integration_admin_basic    --no-fail-fast -j 4
```

**单 crate 模式** (per memory): 禁止 `cargo test --workspace`。每个 crate 单独跑。

**共享 cargo target dir 缓存污染预防** (per memory, ULYS-100 incident):
开工前先实测一次 cargo, 若 rustc 报 method takes X args but Y supplied + defined here metrics.rs:NNN + on-disk line ≠ NNN → 必为共享缓存污染, 立即:

```bash
rm -f /e/DevCache/cargo/target/debug/deps/libshared_platform-*.{rlib,rmeta}
rm -rf /e/DevCache/cargo/target/debug/incremental/shared_platform-*
# 然后重跑
```

预期: 12 个 FAIL → 全部 PASS。

### Step 3: k3s 5 域 Pod + e2e-smoke 12 探针

```bash
wsl -d Ubuntu -e bash -c 'sudo systemctl status k3s'  # 确认 ready
wsl -d Ubuntu -e bash -c 'sudo kubectl --kubeconfig=/etc/rancher/k3s/k3s.yaml apply -k /mnt/d/RustGameServer/docs/deploy/01-k8s-manifests/'
# 等所有 5 域 service Pod Ready (player / economy / match / social / admin)
wsl -d Ubuntu -e bash -c 'sudo kubectl --kubeconfig=/etc/rancher/k3s/k3s.yaml -n rust-game-server get pods -o wide'
pwsh -File D:/RustGameServer/scripts/e2e-smoke.ps1 -Json
```

预期: 5 域 Pod Ready; 12 探针全过 (per ULYS-111 fixes 已合 dev, 应能 12/12)。

### Step 4: 若发现新 bug

若 k3s / e2e-smoke 暴露新问题, **暂停 e2e-smoke**, 另开 issue 修复, 标 `in_progress` 后回这里继续。**禁止在当前 issue 直接修新 bug** (避免 scope creep)。

## 输出 (本 issue 收口时贴的 comment)

格式 (per ULYS-94 brief 原 spec):

```markdown
## ULYS-94 完成报告 (<JST 时间>)

### 完成情况
| Step | 状态 | 备注 |
|---|---|---|
| Step 1: WSL Ubuntu + Postgres + k3s | ✅/❌ | ... |
| Step 2: 12 DB fixture | ✅/❌ (X/12 PASS) | ... |
| Step 3: k3s 5 域 Pod + e2e-smoke 12 探针 | ✅/❌ (Y/12 PASS) | ... |
| Step 4: 新 bug | (N/A or 链接到新 issue) | ... |

### 关键 commit
- (任何本次新增的 commit, 链接到 D:/RustGameServer 分支 + SHA)

### e2e-smoke 日志路径
- D:/RustGameServer/logs/e2e-smoke-<timestamp>.log
- WSL: /var/log/e2e-smoke.log
```

## 风险与约束 (per memory)

1. **单 crate 模式**: 禁止 `cargo test --workspace`。
2. **共享 cargo target dir 污染**: 开工前实测一次。
3. **worktree 复用**: 你被分配的 worktree 路径, 在 brief 后面的运行参数里给出 (本父会话不预设, 由平台 daemon 决定)。
4. **commit hygiene**: 排除 `.multica/`、AGENTS.md 的 `<!-- BEGIN MULTICA-RUNTIME -->` 块、`tmp_debug*.py`。`git add <具体文件>` 而非 `-A`。
5. **不要 PR**: ULYS-94 不是新功能, 是基础设施恢复。commit 直接落在你被分配的分支, 不开 PR (父会话负责 cherry-pick 到 dev)。
6. **不要 close issue**: 父会话在验证 commit 后 close ULYS-94。

## 完成定义 (DoD)

- [ ] Step 1 完成 (WSL/k3s/PG 全 active)
- [ ] Step 2 12/12 DB fixture PASS
- [ ] Step 3 e2e-smoke 12/12 PASS
- [ ] 评论贴完成报告
- [ ] 父会话验证 → close issue

## 不要做

- 不要碰 ULYS-87 / ULYS-88 / ULYS-95 / ULYS-111 (都已合 dev, 不要重复)
- 不要碰 ULYS-98 / ULYS-110 / ULYS-81 (协调性 issue, 等 D-Boy)
- 不要碰 ULYS-103 (已合 dev, 父会话刚推 done)
- 不要 PR
- 不要 close ULYS-94 本身 (留给父会话)
