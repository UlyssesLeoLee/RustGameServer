# RGS-GHCR-PUSH-RESOLVED-V0.42-2026-08-31.md — GHCR 0.1.0 镜像推送阻塞已解除

| 字段 | 值 |
|---|---|
| 文档编号 | RGS-GHCR-PUSH-RESOLVED-V0.42-2026-08-31 |
| 版本 | v0.42 |
| 关联 commit | bfb16b0 (workflow tags 输出格式修复) |
| 关联父文档 | docs/deploy/RGS-AI-HANDOFF-UPSTREAM-2026-08-30.md (v0.1) / RGS-GHCR-NEW-PIPELINE-CONFIG-BLOCKED-2026-08-30.md (v0.40) |
| 状态 | 🟢 GHCR push 阻塞已解除,k3s rollout 待后续(集群当前不可连) |
| 修订人 | 上游 AI 接力(Claude Code / Mavis 账号) |

## 1. 一句话结论

Handoff 中记录的 `$env:GHCR_PAT` 401 状态,本次接力时重新实测**已恢复为 200**(GitHub 侧 token 已被重新签发/修复,非本次改动)。真正卡住 build 的是 `build-prod-0.1.0.yml` workflow 里 **`Resolve tags` 步骤的输出格式 bug**:把 `--tag X --tag Y` CLI flag 字符串整体塞进 `docker/build-push-action@v5` 的 `tags:` 输入,而该 action 期望的是换行分隔的 `image:tag` 列表,导致 buildx 报 `invalid reference format`(run 33336778830 失败,9m8s)。

## 2. 处理过程

1. 用 `$env:GHCR_PAT` 实测 GitHub API(`/user` `/repos/...` `/actions/permissions` `/actions/runs`)全部 200,`docker login ghcr.io` 也 `Login Succeeded` — 确认 token 层面已无阻塞。
2. 用已登录的 `gh` CLI(scope: repo + workflow)触发 `build-prod-0.1.0` workflow_dispatch(run 33336778830) → cargo build 成功,docker buildx push 失败(`invalid reference format`)。
3. 定位到 `.github/workflows/build-prod-0.1.0.yml` 第 99-107 行 `Resolve tags` 步骤的 bug,改为 `GITHUB_OUTPUT` heredoc 多行输出(commit bfb16b0),push 到 main。
4. 重新触发(run 33337236838),29m41s 后成功:
   - `pushing manifest for ghcr.io/ulyssesleolee/rustgameserver:0.1.0@sha256:24da076e3e6d2ebb58a9d129e307bcedd0c161040fff4c0f052ecb534374fddf` ✅
   - `pushing manifest for ghcr.io/ulyssesleolee/rustgameserver:latest@sha256:...` ✅
5. `docker manifest inspect ghcr.io/ulyssesleolee/rustgameserver:0.1.0`(本地,用 `docker login` 会话,非 REST API)确认两个 tag 均可正常拉取(linux/amd64 manifest 存在)。

**附注**: workflow 内 `verify push + summarize` job 的 curl 校验步骤(用 `Authorization: Bearer $GITHUB_TOKEN` 直接查 `ghcr.io/v2/.../manifests/...`)返回 403 — 这是该 job 自身对 GHCR REST 包元数据 API 权限不足(`GITHUB_TOKEN` 的 `packages: write` 不等于包管理 API 需要的 `read:packages` classic scope),**不代表镜像未推送成功**,已用 `docker manifest inspect` 交叉验证。此 curl 校验步骤本身仍是已知缺陷,待后续修复(非阻塞,不影响镜像可用性)。

## 3. 未完成事项

k3s 集群当前**不可连接**(`kubectl` 报 `dial tcp 127.0.0.1:52551: connectex`),推测是 WSL2/Docker Desktop 里的 k3s 未启动。**未尝试强制启动**,因已知 WSL2/k3s 存在 HPA minReplicas 强启动风暴的历史问题(per 之前 session 记录),贸然操作有风险。

因此 handoff 第 5 节的 `kubectl set image` 5 业务域 + cluster-ops rollout 步骤**留待下次集群可用时执行**:

```bash
kubectl -n rust-game-server set image deploy/player-service player=ghcr.io/ulyssesleolee/rustgameserver:0.1.0
# 同样改 economy / match / social / admin / cluster-ops
kubectl -n rust-game-server rollout status deploy/player-service --timeout=5m
```

## 4. 关键引用

- `docs/deploy/RGS-AI-HANDOFF-UPSTREAM-2026-08-30.md`(v0.1,原始 handoff)
- `docs/00-基准与治理/RGS-GHCR-NEW-PIPELINE-CONFIG-BLOCKED-2026-08-30.md`(v0.40)
- `.github/workflows/build-prod-0.1.0.yml`(commit bfb16b0 修复)
- GHCR 镜像: `ghcr.io/ulyssesleolee/rustgameserver:0.1.0` / `:latest`,digest `sha256:24da076e3e6d2ebb58a9d129e307bcedd0c161040fff4c0f052ecb534374fddf`
- 成功 run: https://github.com/UlyssesLeoLee/RustGameServer/actions/runs/33337236838

## 修订历史

| 版本 | 日期 (JST) | 修订人 | 变更 |
|---|---|---|---|
| v0.42 | 2026-08-31 | 上游 AI 接力 | 初版:GHCR push 阻塞根因(workflow tags 格式 bug)+ 修复 + 验证闭环;k3s rollout 因集群不可连留待后续 |
