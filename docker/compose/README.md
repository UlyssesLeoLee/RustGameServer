# docker-compose dev（per WBS v0.3 §2A.5 WF-1-53.8）

## 启动

```bash
# 1. 复制环境变量模板
cp docker/compose/.env.compose.example docker/compose/.env

# 2. 编辑 .env 把所有 CHANGE_ME_* 替换为实际密码
# 53.8 接受：dev 用 ulysses_local（与 .env + k3s DevValues 一致）

# 3. 启动
cd docker/compose
docker compose --profile dev up -d

# 4. 看日志
docker compose --profile dev logs -f
```

## 停止

```bash
cd docker/compose
docker compose --profile dev down
# 加 -v 删数据卷
docker compose --profile dev down -v
```

## 6 独立 DB（per ARC-008 5 独立 DB + cluster_ops_db 第 6）

| 服务 | 端口（host:container） | 默认 DB 名 | 默认 user |
|---|---|---|---|
| player-db | 15432:5432 | player_db | player_user |
| economy-db | 15433:5432 | economy_db | economy_user |
| match-db | 15434:5432 | match_db | match_user |
| social-db | 15435:5432 | social_db | social_user |
| admin-db | 15436:5432 | admin_db | admin_user |
| cluster-ops-db | 15437:5432 | cluster_ops_db | cluster_ops_user |

## 5 域服务（占位 image）

| 服务 | gRPC 端口 | DB |
|---|---|---|
| player-service | 50051 | player-db |
| economy-service | 50052 | economy-db |
| match-service | 50053 | match-db |
| social-service | 50054 | social-db |
| admin-service | 50055 | admin-db |

## 53.8 范围

- ✅ 6 独立 PG 18.6 容器 + healthcheck + volume 持久化
- ✅ 5 域服务占位（rust:1.98-slim 跑 cargo run --release）
- ✅ network / volume / env 模板完整
- ⚠ 5 域服务的实际业务实现待 54.5/54.6/54.7（per DTL-015~031）
- ⚠ distroless base image 待 53.13
- ⚠ k3s 部署（per DEC-010 WSL2 native）已替代 docker-compose 作为生产部署形态