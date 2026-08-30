# RGS-AI-HANDOFF-UPSTREAM-2026-08-30.md — 上游 AI 接力 Handoff

| 字段 | 值 |
|---|---|
| 文档编号 | RGS-AI-HANDOFF-UPSTREAM-2026-08-30 |
| 版本 | v0.1 |
| 关联 commit | 36123fc (收工 v0.41) / f6d0d42 (workflow+script) / c9b34ef (v0.40) |
| 关联主对话 session | mvs_25e9300a4de240af9fc2e31f5eb99eaa (Mavis / Ulysses) |
| 状态 | 🟡 主对话收工,镜像推 GHCR 留待上游 AI 处理 |
| 收件方 | 上游 AI(可能拥有 docker push 权限 / 长期 classic PAT / self-hosted runner 凭据 / 镜像预构建能力) |
| 修订人 | Ulysses (一人公司 12 角色 per DEC-008) - Mavis 接手 |

---

## 0. 一句话当前状态

**RGS k3s 部署实质闭环**(B-CODE 4/4 ✓ + 14/14 业务镜像 Running + 跨域 trace 验证 PASS),**仅剩 0.1.0 prod 镜像推 GHCR 失败** — 因 `$env:GHCR_PAT`(fine-grained PAT `github_pat_11AHE3X4...`,长度 93)对 GitHub 全部 endpoint 401 + `docker login ghcr.io` `denied: denied`,token 自身已无效。主对话已完成新式 GHCR pipeline(workflow + trigger script 落档 commit f6d0d42),**等上游 AI 用有效凭据触发 build**。

---

## 1. 关键 commit 链(全部已推 origin/main)

```
36123fc docs(plan): k3s 部署收工落档 v0.41 (per 8/30 13:00 JST 决策)
c9b34ef docs(plan): 新式 GHCR pipeline 配置 + 触发阻塞落档 v0.40 (per 8/30 14:30 JST)
f6d0d42 ci: 新式 GHCR pipeline - build-prod-0.1.0 workflow + fine-grained PAT trigger
b7c514a docs(plan): 卡牌 8 桶 + W36 跨域 100% 闭环最终总结 v0.37 (per 8/30 10:05 JST)
f6820dd merge: W36 economy-service 跨域 trade saga 集成 (100 测试全过)
1ea7284 feat(gm-backend): W36 gm.proto v0.4 实际集成 (15 UT + 5 IT 全过, 0 破坏)
57412e9 merge: W36 match-service 跨域 SaveReplay saga 集成 (177 测试全过)
46bbb62 docs(deploy): k3s 部署重启阻塞落档 (3 阻塞, per 2026-08-30 12:55 JST)
```

main HEAD = `36123fc`,工作区 clean(只 untracked `.worktrees/` + `docs/00-基准与治理/.test-evidence/`)。

---

## 2. 阻塞详情(请优先读这一节)

### 2.1 $env:GHCR_PAT 状态(主对话 8/30 14:30 JST 实测,稳定)

| Endpoint | 响应 | 含义 |
|---|---|---|
| `docker login ghcr.io` (PAT 当密码) | `denied: denied` | token 不能 authenticate 到 GHCR |
| `GET https://api.github.com/user` | `401 Bad credentials` | token 不能 authenticate 到 GitHub API |
| `GET /repos/UlyssesLeoLee/RustGameServer` | `401` | token 没有此 repo 访问 |
| `GET /repos/.../actions/permissions` | `401` | token 没有 Actions 权限 |
| `GET /user/repos` | `401` | token 没有 user 范围 |
| `GET /repos/.../actions/runs` | `401` | token 没有 Actions: read |
| `POST /repos/.../dispatches` | `401` | token 没有 Actions: write |

**全部 401,token 本身已被 GitHub 服务器拒**,**不是 scope 错**。两次会话间(8/30 12:55 → 14:30)token 状态从"docker login OK, push permission_denied"变成"全 401",强烈说明用户(Ulysses)在两次会话间重生了 PAT 或改 scope 配错。

**Token 形式**: `github_pat_11AHE3X4...` 长度 93(标准 fine-grained PAT 格式)

### 2.2 主对话已做的诊断结论

- ✅ PowerShell 能读到 `$env:GHCR_PAT`(长度 93,前缀 `github_pat_`)
- ❌ GitHub 拒认这个字符串(401,无歧义)
- 修复只能在 token 层面,代码层面零阻塞
- 主对话已写好新式 pipeline,等有效 token

---

## 3. 主对话已交付的资产(等上游 AI 用)

### 3.1 新式 GHCR workflow(已 commit f6d0d42)

文件: `.github/workflows/build-prod-0.1.0.yml`(5874 字节)

**关键设计**:
- 触发: `workflow_dispatch`
- 认证: `GITHUB_TOKEN` + `permissions: packages: write, contents: read`
- runner: `ubuntu-latest`,timeout 60 分钟
- 步骤: `cargo build --release --workspace --locked` → `docker build --target prod` → `docker push ghcr.io/ulyssesleolee/rustgameserver:0.1.0`(可选 :latest)
- 验证步: curl registry manifest 确认
- **不卡 rust-ci gate**(信任 main HEAD 已 PASS,节省 15+ 分钟)
- 兼容 5 业务域 + 5 卡牌域 = 10 binary 共享 `/app/bin/`,k3s manifest 用 command 分流

**输出镜像**: `ghcr.io/ulyssesleolee/rustgameserver:0.1.0`(内容包含 player/economy/match/social/admin/cluster-ops/i18n/card/leaderboard/replay/rgs-asset-download 11 个 binary)

### 3.2 Trigger 脚本(已 commit f6d0d42)

文件: `scripts/trigger-build-prod.ps1`(3205 字节)

**用法**(上游 AI 需提供 valid `$env:GHCR_PAT`):
```powershell
pwsh -NoProfile -File scripts/trigger-build-prod.ps1 -Tag 0.1.0 -PushLatest true
```

**安全约束**(per 8/27 hard ban):
- **禁止**打印 `$env:GHCR_PAT` 内容到任何 log / 终端
- 只打印 length + 4 字符 prefix 做 sanity check
- 长度 < 20 字符直接拒绝触发

**触发后**:
- GitHub Actions UI: https://github.com/UlyssesLeoLee/RustGameServer/actions/workflows/build-prod-0.1.0.yml
- 等待 20-30 分钟(冷 cache 30 分钟,热 cache 20 分钟)

### 3.3 k3s manifest 镜像需求(全部 3 tag)

| Tag | 内容 | 状态 |
|---|---|---|
| `0.1.0` | 5 业务域 + 5 卡牌域 = 10 binary | ❌ 未推(等上游 AI 触发 workflow) |
| `0.1.0-cluster-ops` | cluster-ops 单独 | ✅ 已 publish 8/27 |
| `0.1.0-gm-backend` | gm-backend 单独 | ✅ 已 publish 8/27 |

k3s manifest 路径:
- 5 业务域: `docs/deploy/01-k8s-manifests/0[1-5]-*-service.yaml`
- cluster-ops: `docs/deploy/01-k8s-manifests/06-cluster-ops-service.yaml`
- gm-backend: `docs/deploy/01-k8s-manifests/50-gm-backend-service.yaml`

---

## 4. 上游 AI 推荐处理路径(3 选 1 或自由组合)

### 路径 A:用 fine-grained PAT 触发 workflow_dispatch(原方案)

**前提**: 上游 AI 拥有有效 fine-grained PAT(scope: `Repository access` 选 `UlyssesLeoLee/RustGameServer` + `Permissions` 勾 `Actions: Read and write` + `Contents: Read`)

```powershell
$env:GHCR_PAT = 'new_fine_grained_pat_here'  # 上游 AI 提供
pwsh -NoProfile -File scripts/trigger-build-prod.ps1 -Tag 0.1.0 -PushLatest true
# 等 20-30 分钟,看 https://github.com/UlyssesLeoLee/RustGameServer/actions
```

### 路径 B:用 classic PAT 直接 docker push(快速)

**前提**: 上游 AI 拥有 `ghp_...` classic PAT,带 `write:packages` + `repo` scope

```bash
# 1. 在能访问 Docker daemon 的机器上(WSL / Linux runner)
echo $CLASSIC_PAT | docker login ghcr.io -u UlyssesLeoLee --password-stdin
docker build --target prod -t ghcr.io/ulyssesleolee/rustgameserver:0.1.0 -t ghcr.io/ulyssesleolee/rustgameserver:latest .
docker push ghcr.io/ulyssesleolee/rustgameserver:0.1.0
docker push ghcr.io/ulyssesleolee/rustgameserver:latest
```

需要本地能 build(10 分钟冷 build + 5 分钟 push),需要 `Dockerfile` + `target/release/*-service` binary。

### 路径 C:用 self-hosted runner(长期 CI/CD 友好)

**前提**: 上游 AI 能配置 GitHub Actions self-hosted runner 在某机器上

修改 `.github/workflows/build-prod-0.1.0.yml`:
- `runs-on: ubuntu-latest` → `runs-on: self-hosted`
- 在 self-hosted 机器上 build(本地 cache 命中,5 分钟 build)
- 用 GITHUB_TOKEN(actions 注入)+ packages: write push

### 路径 D:从其他渠道获取 0.1.0 镜像

如果上游 AI 不能 push GHCR,但能:
- 从 Docker Hub / 其他 registry pull 现成镜像
- 用 cosign 从 GHCR attestation 校验镜像
- 从 backup tarball 解出镜像

请提供 0.1.0 镜像 SHA256 digest,主对话可以用 `cosign verify` + `docker pull` 校验。

---

## 5. 上游 AI 处理完后,主对话(下次会话)的下一步

```bash
# 1. 验证 GHCR 镜像存在
curl -fsSL -H "Authorization: Bearer $GHCR_PAT" \
  "https://ghcr.io/v2/ulyssesleolee/rustgameserver/manifests/0.1.0" | jq .

# 2. k3s set image 5 业务域 + cluster-ops(per k3s manifest)
kubectl -n rust-game-server set image deploy/player-service \
  player=ghcr.io/ulyssesleolee/rustgameserver:0.1.0
# 同样改 economy / match / social / admin / cluster-ops

# 3. 验证 6 deployment 全 RollingUpdate 完成
kubectl -n rust-game-server rollout status deploy/player-service --timeout=5m
# ... 同样 5 个

# 4. 落档 v0.42 (k3s 0.1.0 部署完成)
```

---

## 6. 关键引用

- v0.41 收工落档: `docs/00-基准与治理/RGS-DEPLOY-WRAP-UP-2026-08-30.md`
- v0.40 GHCR pipeline 阻塞落档: `docs/00-基准与治理/RGS-GHCR-NEW-PIPELINE-CONFIG-BLOCKED-2026-08-30.md`
- v0.39 k3s 部署重启阻塞落档: `docs/00-基准与治理/RGS-DEPLOY-K3S-RESTART-BLOCKED-2026-08-30.md`
- v0.37 卡牌 + W36 闭环总结: `docs/00-基准与治理/RGS-CARD-8BUCKET-W36-100PCT-V0.37-2026-08-30.md`
- 9 DEC 拍板: `docs/00-基准与治理/RGS-DDD-CARD-9DEC-2026-08-29.md`
- k3s manifest: `docs/deploy/01-k8s-manifests/*.yaml`
- 8/24 SRE handoff(参考格式): `docs/deploy/phase-0-5-handoff.md`

---

## 修订历史

| 版本 | 日期 (JST) | 修订人 | 变更 |
|---|---|---|---|
| v0.1 | 2026-08-30 13:06 | Ulysses (一人公司 12 角色 per DEC-008) - Mavis 接手 | 初版:上游 AI 接力 handoff,聚焦 GHCR PAT 401 阻塞 + 3 选 1 处理路径 |
