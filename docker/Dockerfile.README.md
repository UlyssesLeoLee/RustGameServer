# Dockerfile（per WBS v0.3 §2A.5 WF-1-53.13）

## 多阶段构建 + 3 个 target

| Target | 基础镜像 | 用途 | 典型 tag |
|---|---|---|---|
| `dev` | rust:1.98-slim | dev 热重载 + 工具链（cargo / clippy / rustfmt） | `dev` / `dev-{git-sha}` |
| `staging` | distroless cc-debian12:nonroot | staging 部署 | `0.1.0-staging` |
| `prod`（默认） | distroless cc-debian12:nonroot | prod 部署 | `0.1.0` / `latest` |

## 构建命令

```bash
# dev
docker build --target dev -t ghcr.io/ulyssesleolee/rustgameserver:dev .

# staging
docker build --target staging -t ghcr.io/ulyssesleolee/rustgameserver:0.1.0-staging .

# prod（默认）
docker build -t ghcr.io/ulyssesleolee/rustgameserver:0.1.0 .
```

## 启用

- 53.13 写好 Dockerfile
- docker-build.yml 启用待 53.7 注释解除（per commit 621aa0c）
- 实际镜像推送待 WF-1-57.8 cosign keyless 签名

## 53.13 范围

- ✅ 多阶段构建（rust builder + distroless runtime）
- ✅ cargo-chef 缓存（加速二次构建）
- ✅ 3 个 target：dev / staging / prod
- ✅ nonroot 用户（per distroless 默认）
- ✅ workspace profile.release（lto + strip + codegen-units）
- ⚠ docker build / push 启用待 53.7 + 57.8
- ⚠ 多 service 镜像（player-service / economy-service 等）拆分待 54.x 业务实施
