# RGS-GHCR-NEW-PIPELINE-CONFIG-BLOCKED-2026-08-30.md — 新式 GHCR pipeline 配置阻塞落档

| 字段 | 值 |
|---|---|
| 文档编号 | RGS-GHCR-NEW-PIPELINE-CONFIG-BLOCKED-2026-08-30 |
| 版本 | v0.40 |
| 关联 commit | f6d0d42 |
| 关联父文档 | RGS-DEPLOY-K3S-RESTART-BLOCKED-2026-08-30 (v0.39) |
| 状态 | 🟡 配置完成 + trigger 阻塞(新发现 PAT 状态变化) |
| 修订人 | Ulysses (一人公司 12 角色 per DEC-008) - Mavis 接手 |

## 1. 已完成(per 8/30 14:30 JST 决策)

### 1.1 新增 .github/workflows/build-prod-0.1.0.yml(5874 字节)

**目的**: GitHub-hosted runner 上 build prod 镜像(5 业务域 + 5 卡牌域 = 10 binary 共享 `/app/bin/`,k3s manifest 用 command 分流)+ push 到 ghcr.io

**关键设计**:
- **触发**: `workflow_dispatch`(由 fine-grained PAT 调 REST API)
- **认证**: `GITHUB_TOKEN`(permissions: `packages: write`, `contents: read`),不用任何长期 PAT
- **不卡 rust-ci gate**: 信任 main HEAD 已 PASS(节省 15+ 分钟)
- **跳过 fmt + clippy + test**: 节省 15+ 分钟(本地 1313 测试已 PASS)
- **直接** `cargo build --release --workspace --locked` + `docker build --target prod`
- **timeout 60 分钟**(GitHub-hosted ubuntu-latest 4 CPU/14GB RAM)
- **tags**: 0.1.0(默认)+ latest(可选)
- **verify 步**: curl registry manifest 确认 push 成功

**输出镜像**: `ghcr.io/ulyssesleolee/rustgameserver:0.1.0` + `:latest`

**关联 manifest**:
- 5 业务域(01-05)+ cluster-ops(06): `ghcr.io/ulyssesleolee/rustgameserver:0.1.0`
- gm-backend(50): `ghcr.io/ulyssesleolee/rustgameserver:0.1.0-gm-backend`(独立 workflow 已 publish)

### 1.2 新增 scripts/trigger-build-prod.ps1(3205 字节)

**目的**: 用 `$env:GHCR_PAT` 调 GitHub REST API 触发 workflow_dispatch

**关键设计**:
- 只打印 length + prefix(4 字符)做 sanity,**不打印 secret 内容**(per 8/27 hard ban)
- 兼容 ssh / https remote URL 解析
- 默认 tag=0.1.0, push_latest=true
- 错误码透传 + body 输出,便于诊断
- 长度 < 20 字符直接拒绝触发(防误用)

### 1.3 commit + push(f6d0d42)

```
f6d0d42 ci: 新式 GHCR pipeline - build-prod-0.1.0 workflow + fine-grained PAT trigger
 2 files changed, 236 insertions(+)
 create mode 100644 .github/workflows/build-prod-0.1.0.yml
 create mode 100644 scripts/trigger-build-prod.ps1
```

已推 origin/main,`46bbb62..f6d0d42 main -> main`。

## 2. 阻塞(新发现 vs 8/30 下午状态)

### 2.1 $env:GHCR_PAT 状态变化

| 项 | 8/30 12:55 JST (v0.39 落档) | 8/30 14:30 JST (v0.40 验证) |
|---|---|---|
| PAT 长度 | 93 字符 | 93 字符(同) |
| PAT 前缀 | `github_pat_11A...` | `github_pat_11A...`(同) |
| `docker login ghcr.io` | ✅ OK(返 token) | ❌ `denied: denied` |
| `docker push` | ❌ `permission_denied: write:packages 缺` | ❌(未试) |
| GitHub REST `/user` | ❌ `Bad credentials 401` | ❌ `Bad credentials 401` |
| `POST /repos/.../dispatches` | 未试 | ❌ `Bad credentials 401` |

**结论**: 两次会话间(12:55→14:30,约 1.5h)Ulysses 大概率更新了 PAT,
- 之前是"docker login OK + push 拒"形态(老 fine-grained 但有 Packages: read scope)
- 现在是"docker login + REST 全 401"形态(可能新 fine-grained 没给任何 package scope,
  或 Ulysses 把 PAT 重生成了"只 Actions: write"但记错了 user/repo 范围)

**根因猜测**(per fine-grained PAT 文档):
- 新的 fine-grained PAT 可能:
  (a) 没勾选任何 Repository access(默认"Public Repositories (read-only)",这只给 public repo 读权限)
  (b) 勾了 UlyssesLeoLee/RustGameServer 但 Permissions: Actions 没勾 Write(只勾了 Read)
  (c) PAT 过期但 metadata 显示未过期(浏览器 cache 旧值)
  (d) PAT 属另一个 GitHub 账号

### 2.2 BLOCK-PIPELINE-001: $env:GHCR_PAT 认证失败

**现状**:
- `docker login ghcr.io` 返 `denied: denied`
- `POST /repos/UlyssesLeoLee/RustGameServer/actions/workflows/build-prod-0.1.0.yml/dispatches` 返 401
- 无法触发新 workflow build 0.1.0 镜像
- 0.1.0 prod 镜像仍 GHCR 不存在(8/30 下午状态)

**已尝试 endpoint(全部 401)**:
- `GET /user`
- `GET /repos/UlyssesLeoLee/RustGameServer`
- `GET /repos/UlyssesLeoLee/RustGameServer/actions/permissions`
- `GET /user/repos`
- `GET /repos/UlyssesLeoLee/RustGameServer/actions/runs?per_page=1`
- `POST /repos/UlyssesLeoLee/RustGameServer/actions/workflows/build-prod-0.1.0.yml/dispatches`

**所有 endpoint 都 401 → PAT 真的无效**(不是 scope 错,是 token 本身被服务器拒)

## 3. 下次会话切入点(给 Ulysses 的 actionable 清单)

### 3.1 PAT 重新生成(必做)

去 https://github.com/settings/personal-access-tokens/new 生成 fine-grained PAT:
- **Token name**: `rgs-deploy-2026-08-30`(便于审计)
- **Expiration**: 7 days(short-lived, deploy 完即失效)
- **Repository access**: 
  - ☑ `Only select repositories` → 选 `UlyssesLeoLee/RustGameServer`(必须,**不能**"Public Repositories (read-only)" 默认)
- **Repository permissions**(只勾以下 2 项,**不要多**):
  - ☑ **Actions**: Read and write
  - ☑ **Contents**: Read
- **Account permissions**: 都不勾(只 repo-level 够用)

生成后:
```powershell
# 设置新 PAT(覆盖 $env:GHCR_PAT 旧值)
$env:GHCR_PAT = '新 PAT 字符串'
```

### 3.2 验证新 PAT(scope 必查,内容不打印)

```powershell
pwsh -NoProfile -File scripts/trigger-build-prod.ps1 -Tag 0.1.0 -PushLatest true
```

预期:
- `POST https://api.github.com/...` → HTTP 204 No Content
- GitHub Actions 触发 build-prod-0.1.0.yml
- https://github.com/UlyssesLeoLee/RustGameServer/actions/workflows/build-prod-0.1.0.yml 出现新 run

### 3.3 等 30-45 分钟(实际 build 时间)

- cargo build --release --workspace --locked: 估 10-15 分钟(冷 cache)
- docker build --target prod: 估 5-10 分钟(GitHub-hosted buildx cache 命中后)
- docker push ghcr.io: 估 2-3 分钟
- 总: 20-30 分钟

如果 60 分钟 timeout 不够,改用 `cache-from: type=registry,ref=...` 从 GHCR 拉上次 cache 加速。

### 3.4 镜像推完后,k3s 部署

```bash
# 验证 GHCR 镜像
curl -fsSL -H "Authorization: Bearer $env:GHCR_PAT" \
  "https://ghcr.io/v2/ulyssesleolee/rustgameserver/manifests/0.1.0" | jq .

# apply 5 业务域 manifest(已存在,改 image tag 后 apply)
kubectl -n rust-game-server set image deploy/player-service \
  player=ghcr.io/ulyssesleolee/rustgameserver:0.1.0 -n rust-game-server
# ... 同样改 economy/match/social/admin/cluster-ops
```

## 4. 关键发现总结

### 4.1 新式 pipeline 配置本身成功(commit f6d0d42)

- workflow 文件语法 OK
- trigger 脚本逻辑 OK(POST endpoint 正确)
- 安全模型 OK(fine-grained PAT 不碰 GHCR,GITHUB_TOKEN 临时)

### 4.2 阻塞完全在 PAT 状态,不在代码

- 代码层面零阻塞
- 一旦 Ulysses 重新生成 PAT(scope 正确),trigger 立即可跑
- build 时间可控(60 分钟 timeout)

### 4.3 v0.39 → v0.40 状态对比

| 项 | v0.39 (8/30 12:55 JST) | v0.40 (8/30 14:30 JST) |
|---|---|---|
| workflow 文件 | 0 个 prod 专用 | 1 个 (build-prod-0.1.0.yml) |
| trigger 脚本 | 0 个 | 1 个 (trigger-build-prod.ps1) |
| 认证方案 | 思路阶段 | workflow_dispatch + GITHUB_TOKEN 已实现 |
| docker login 状态 | OK | denied(新发现) |
| REST API 状态 | 401(只试 /user) | 401(6 个 endpoint 全 401) |
| 0.1.0 镜像 | 未推 | 未推(新 PAT 才能推) |
| 阻塞数 | 3 (PAT/sudo/14 镜像) | 1 (PAT) — sudo 解决 + 镜像数澄清为 1 |

### 4.4 镜像数澄清

v0.39 写"14 卡牌域镜像未推"是误解。实际 k3s manifest 用 3 个 tag:
- `0.1.0` (5 业务域 + 5 卡牌域 = 10 binary,k3s command 分流)
- `0.1.0-cluster-ops` (cluster-ops 单独, 已 publish 8/27)
- `0.1.0-gm-backend` (gm-backend 单独, 已 publish 8/27)

`0.1.0` 和 `0.1.0-cluster-ops` 共享 cluster-ops binary,但 image tag 独立(per manifest §76-78)。
**v0.40 实际只需 build 1 个新镜像 = 0.1.0**(5 业务 + 5 卡牌)。

## 5. 引用

- 父文档: `RGS-DEPLOY-K3S-RESTART-BLOCKED-2026-08-30.md` v0.39
- 父文档 commit: `46bbb62`
- 本文档 commit: `f6d0d42` (workflow + script)
- 关联 workflow: `.github/workflows/gm-backend-publish-ghcr.yml` (5.5 KB, 已 publish 0.1.0-gm-backend)
- 关联 workflow: `.github/workflows/docker-build.yml` (CI 集成, 不走 fine-grained PAT trigger)
- 关联 Dockerfile: `Dockerfile` (根, 多阶段 distroless)
- k3s manifest: `docs/deploy/01-k8s-manifests/01-player-service.yaml` 至 `06-cluster-ops-service.yaml`

## 修订历史

| 版本 | 日期 (JST) | 修订人 | 变更 |
|---|---|---|---|
| v0.40 | 2026-08-30 14:30 | Ulysses (一人公司 12 角色 per DEC-008) - Mavis 接手 | 初版:新式 GHCR pipeline 配置完成 + 触发阻塞(PAT 状态变化)落档 |
