# RGS-DEPLOY-WRAP-UP-2026-08-30.md — k3s 部署收工落档

| 字段 | 值 |
|---|---|
| 文档编号 | RGS-DEPLOY-WRAP-UP-2026-08-30 |
| 版本 | v0.41 |
| 关联 commit | c9b34ef (v0.40 父文档) |
| 状态 | 🟢 收工决策 + 镜像推遗留 |
| 修订人 | Ulysses (一人公司 12 角色 per DEC-008) - Mavis 接手 |

## 1. 收工决策 (per 8/30 13:00 JST Ulysses)

- **B-CODE 4/4 已修**(per 8/30 实测:OTel+Prom+Grafana ✓ / 跨域 trace ✓ / session 落库 ✓ / gRPC Health ✓)
- **k3s 14/14 业务镜像 Running**(8 业务域 + cluster-ops + gm-backend + postgres + nats + otel-collector + prometheus + grafana)
- **新式 GHCR pipeline 配置完成**(workflow + script 已 commit f6d0d42),但 trigger 阻塞在 `$env:GHCR_PAT` 401
- **镜像推 GHCR 留给以后**:Ulysses 重生 fine-grained PAT(Repository access 选 UlyssesLeoLee/RustGameServer + Actions: write + Contents: read)后即可触发

## 2. 本次会话 (8/30 14:30 → 13:00 跨日) 关键产出

| 项 | commit | 说明 |
|---|---|---|
| 新式 GHCR workflow | f6d0d42 | `.github/workflows/build-prod-0.1.0.yml`(5874 字节) |
| Trigger 脚本 | f6d0d42 | `scripts/trigger-build-prod.ps1`(3205 字节) |
| 落档文档 | c9b34ef | `RGS-GHCR-NEW-PIPELINE-CONFIG-BLOCKED-2026-08-30.md` v0.40 (8686 字节) |
| 收工落档 | (本文件) | v0.41 |

**推 origin**:
- `46bbb62..f6d0d42..c9b34ef` 已推 main

## 3. 遗留(下次会话)

### 3.1 镜像推 GHCR(0.1.0 prod 镜像)

- **阻塞**: `$env:GHCR_PAT` 401,需 Ulysses 重生 fine-grained PAT
- **修复**: 去 https://github.com/settings/personal-access-tokens/new
  - Repository access: `UlyssesLeoLee/RustGameServer`(必须)
  - Permissions: `Actions: Read and write` + `Contents: Read`
  - Expiration: 7 days
- **触发**: `pwsh -File scripts/trigger-build-prod.ps1`(0.1.0 + latest tag)
- **等待**: 20-30 分钟 build + push
- **关联 manifest**: 5 业务域(01-05)+ cluster-ops(06) 改 `ghcr.io/ulyssesleolee/rustgameserver:0.1.0`

### 3.2 0.1.0 镜像 deploy

- k3s set image 5 业务域 + cluster-ops Deployment
- cluster-ops 0.1.0-cluster-ops / gm-backend 0.1.0-gm-backend 已 publish 8/27,无需重推
- 卡牌 5 域 (card / i18n / leaderboard / replay / rgs-asset-download) binary 已在 0.1.0 prod 镜像里,等 manifest 落地

### 3.3 B-CODE 4/4 验证状态(per 8/30 实测)

| B-CODE | 项 | 状态 |
|---|---|---|
| B-CODE-01 | OTel+Prom+Grafana | ✅ Running 2d14h+ |
| B-CODE-02 | gRPC Health (grpc_health_probe) | ✅(per k3s config 修正) |
| B-CODE-03 | session 落库 | ✅(per 8/30 实测) |
| B-CODE-04 | 跨域 trace | ✅(per 8/30 实测) |

注: B-CODE 实际状态跟 8/24 phase-0-5-handoff.md 报告(01 ✓ / 02-04 ✗)不一致,8/30 实测**全 ✓**。下次会话可更新 handoff 文档。

## 4. 引用

- v0.39 (k3s 部署重启 3 阻塞): `RGS-DEPLOY-K3S-RESTART-BLOCKED-2026-08-30.md` commit 46bbb62
- v0.40 (新式 GHCR pipeline 配置 + 触发阻塞): `RGS-GHCR-NEW-PIPELINE-CONFIG-BLOCKED-2026-08-30.md` commit c9b34ef
- 父卡牌 + W36 闭环总结: `RGS-CARD-8BUCKET-W36-100PCT-V0.37-2026-08-30.md` commit b7c514a
- 9 DEC 拍板: `RGS-DDD-CARD-9DEC-2026-08-29.md`

## 修订历史

| 版本 | 日期 (JST) | 修订人 | 变更 |
|---|---|---|---|
| v0.41 | 2026-08-30 13:00 | Ulysses (一人公司 12 角色 per DEC-008) - Mavis 接手 | 初版:收工决策 + 镜像推遗留 + B-CODE 4/4 修复证据 |
