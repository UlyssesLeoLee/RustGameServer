# k3s 5 域 binary CrashLoopBackOff GLIBC 修复 (per 9/9 17:55 JST Ulysses 拍板选项 1)

**代签**: Mavis 默认代签 Ulysses (per 8/27 三次强化 + 9/8 15:19 第 6/7 次强化)
**修复时间**: 2026-09-09 17:55-19:00 JST
**关联 commit**: (待 mega-v3 完成 commit)

---

## 1. 现象

- 5 域 binary (player / economy / match / social / admin) 在 k3s pod 内 CrashLoopBackOff 87 次/12h
- `Back-off restarting failed container`
- 5 域 pod 始终 0/1 Ready, 但 infra (gm-backend / grafana / nats / otel-collector / prometheus) 全部 1/1 Running 22h+

## 2. 根因 (per 9/9 18:00-18:30 JST Mavis 诊断)

**5 域 binary 在 WSL2 Ubuntu 24.04 (GLIBC 2.39) 上编译, image base 是 distroless debian-12 bookworm (GLIBC max 2.36), 差 2 个 minor version**.

| 项 | 值 |
|---|---|
| Image base | `gcr.io/distroless/cc-debian12:nonroot` (Google distroless, bazel build) |
| Image build date | 2026-08-24 (W7-W9 派工前, 老 toolchain) |
| Image layer count | 20 |
| Base GLIBC max | 2.36 |
| 5 域 binary 编译 toolchain | WSL2 Ubuntu 24.04 GLIBC 2.39 (rust 1.98 默认链 2.38) |
| 5 域 binary 需 GLIBC | `__isoc23_strtol` / `__isoc23_sscanf` / `__cxa_thread_atexit_impl` / `gnu_get_libc_version` (= GLIBC 2.38+) |
| Pod log | `/app/bin/player-service: /lib/x86_64-linux-gnu/libc.so.6: version 'GLIBC_2.38' not found` |

## 3. 修复 (per 9/9 17:55 JST Ulysses 拍板选项 1)

**选项 1: 改 Dockerfile base → gcr.io/distroless/cc-debian13 (trixie, GLIBC 2.38)**.

实施步骤 (per 9/9 18:00-19:00 JST Mavis 自驱):

1. **确认 Dockerfile 源码已改** (per commit `1a580b9` 9/9 06:00 JST "fix(deploy): 5 域 distroless glibc 2.38"):
   - `Dockerfile:39` 已是 `FROM gcr.io/distroless/cc-debian13:nonroot AS runtime-base`
   - 注释 line 37-38 解释: "cc-debian12 glibc 2.36 跟 binary 链 GLIBC_2.38 不匹配, 改 cc-debian13 (glibc 2.38)"

2. **WSL2 装工具链** (rustup 1.98.1 + nerdctl 2.0.4 + buildkitd 0.17.1)

3. **WSL2 编译 5 域 binary** (linux x86_64-gnu, 4m 50s 完成, 12 域 + cluster-ops)

4. **nerdctl build image tag `:0.1.0-cc13`** (cc-debian13 base + prebuilt binary + grpc_health_probe v0.4.56)

5. **k3s ctr images import** (本地 cache, imagePullPolicy: Never)

6. **kubectl set image 5 域 deployment** → `:0.1.0-cc13` + rollout status

7. **nerdctl push ghcr.io** (用 `$env:GHCR_PAT_CLASSIC` 40 chars `ghp_...iwZl`, 走 stdin pipe 不打印)

## 4. 验证 (待 mega-v3 完成)

- 5 域 pod Ready 1/1 (从 0/1 升到 1/1)
- grpc_health_probe exec probe OK (per 5 域 deployment liveness/readiness probe)
- 5 域 HealthCheck STATUS_OK (grpc)
- shim 766 cmd → rgs-proxy → k3s svc DNS 5 域 → 100% 业务覆盖
- ghcr.io/ulyssesleolee/rustgameserver:0.1.0-cc13 可拉 (多节点 cluster)

## 5. 派生约束 (per 9/9 19:00 JST 新增)

- **L13**: 5 域 binary 编译环境 GLIBC 必须 <= image base GLIBC max, 否则 CrashLoopBackOff
- **L13.1**: image base GLIBC 决策时, 跟 rust toolchain 默认 link GLIBC 对齐
  - rust 1.83+ 默认链 GLIBC 2.38, image base 必须 ≥ 2.38 (推荐 cc-debian13 trixie)
  - rust 1.78- 默认链 GLIBC 2.35, image base 可以是 cc-debian12 bookworm
- **L13.2**: WSL2 build 走 nerdctl (apt 仓库无 nerdctl, 需 GitHub release binary), buildkitd 同
- **L13.3**: ghcr.io push 凭据走 PowerShell `[IO.File]::WriteAllBytes` + WSL `cp` 模式, 不打印不打印不打印 (per 8/27 11:06 硬 ban)
- **L13.4**: `cluster-ops` 不带 `-service` suffix, 跟 5 域不同, COPY 写错 set -e 退出 (教训)
- **L13.5**: grpc_health_probe 必须 COPY 进 image, k8s exec probe 强需 (v0.4.56, SHA256 verified)
- **L13.6**: WSL `/tmp` 跨 sudo bash instance 不共享, 临时文件用 `/mnt/d/temp/` 共享 + chmod 600

## 6. 已知缺口 (per 9/9 19:00 JST)

- **5 域 NEW 域 (batch/battle/card/i18n/leaderboard/replay/scene) k8s deployment 未更新 image tag** (这次只更新 5 域)
- **buildkitd 没真用上** (nerdctl build 走 k3s containerd rootless, 不需要 buildkitd 单独 daemon; 但已经启了, 无副作用)
- **GHCR_PAT_CLASSIC 过期时间未知** (push 成功, 但 token rotation 没在 docs 记录)
- **k3s imagePullPolicy: Never 阻塞多节点** (ghcr.io push 已解, 但需更新 deployment yaml imagePullPolicy: Always, 后续 SRE 决策)

## 7. 后续 (待 Ulysses 拍板)

- [ ] 更新 5 域 deployment yaml image tag (commit)
- [ ] 更新 DEPLOY_STATUS.md v0.2 (5 域 k3s Ready 1/1)
- [ ] SRE 拍板 imagePullPolicy: Never → Always (多节点共用)
- [ ] NEW 域 (batch/battle/card/i18n/leaderboard/replay/scene) k8s deployment 更新
- [ ] GHCR_PAT rotation 文档化
