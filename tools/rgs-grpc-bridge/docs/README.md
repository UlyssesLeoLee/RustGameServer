# rgs-grpc-bridge

RGS gRPC-Web bridge: 浏览器 HTTP/1.1 + JSON (Connect Protocol) → RGS 5 域 gRPC server

**版本**: v0.1.0
**创建**: 2026-09-10 JST
**决策**: per DTL-038 决策 + 9/10 JST Ulysses 拍板 opt2 (独立 bridge binary, 不下沉 5 域 binary)

## 设计

- **server**: tonic-web 0.12 暴露 gRPC-Web (HTTP/1.1) + Connect Protocol (application/json)
- **client**: tonic 0.12 调 backend gRPC server (mTLS or insecure)
- **forwarder**: 每个 PlayerService RPC 方法都透传到 backend player-service :50051
- **CORS**: 允许浏览器 fetch 跨域 (H5 8788 → bridge 8080)
- **路由**: `/<package>.<service>/<method>` → 对应 backend

## v0.1 范围

- 只 forward `player.v1.PlayerService` (25 个 unary RPC)
- 后续 v0.2 加 `economy.v1.EconomyService` / `match.v1.MatchService` / `social.v1.SocialService` / `admin.v1.AdminService`

## 构建

```bash
# workspace 成员, 直接 build
cargo build -p rgs-grpc-bridge --release

# dev 模式 (热编译)
cargo build -p rgs-grpc-bridge
```

## env 配置

| env | 说明 | default |
|---|---|---|
| `RGS_BRIDGE_LISTEN` | bridge 监听地址 | `0.0.0.0:8080` |
| `RGS_BRIDGE_PLAYER_BACKEND` | player-service backend | `http://localhost:50051` |
| `RGS_BRIDGE_INSECURE` | 1 = 跳过 mTLS (dev) | `0` |
| `RGS_BRIDGE_CA_CERT` | CA cert PEM 路径 (mTLS) | - |
| `RGS_BRIDGE_CLIENT_CERT` | client cert PEM 路径 (mTLS) | - |
| `RGS_BRIDGE_CLIENT_KEY` | client key PEM 路径 (mTLS) | - |
| `RGS_BRIDGE_SERVER_CERT` | server cert (可选, 启用 bridge 自身 mTLS) | - |
| `RGS_BRIDGE_SERVER_KEY` | server key (可选) | - |
| `RUST_LOG` | tracing filter | `info,rgs_grpc_bridge=debug` |

## dev 启动

```bash
# 1. 启动 player-service (insecure 模式, 监听 50051)
RGS_ALLOW_INSECURE_GRPC=1 GRPC_ADDR=0.0.0.0:50051 \
    cargo run -p player-service

# 2. 启动 bridge (insecure, 监听 8080, 转发到 localhost:50051)
RGS_BRIDGE_INSECURE=1 RGS_BRIDGE_PLAYER_BACKEND=http://localhost:50051 \
    cargo run -p rgs-grpc-bridge

# 3. curl 测
curl -X POST http://127.0.0.1:8080/player.v1.PlayerService/HealthCheck \
    -H 'Content-Type: application/json' \
    -d '{"service": ""}'
```

## 路径

- `tools/rgs-grpc-bridge/Cargo.toml` — workspace 成员
- `tools/rgs-grpc-bridge/build.rs` — 复用 player-service proto
- `tools/rgs-grpc-bridge/src/main.rs` — bridge 主体
- `tools/rgs-grpc-bridge/docs/README.md` — 本文档

## 派生约束 (per AGENTS.md + 8/27 JST)

- **env value 永打印** (8/27 11:06 JST hard ban): RGS_BRIDGE_CA_CERT 等路径字符串不进 log
- **mTLS 失败 fail-closed** (per RGS-REV-007 CH4): secure 模式 cert/key 缺则启动失败
- **代签规则** (8/27 19:39/20:56/21:59 JST): author=Ulysses / 审批=架构师(Mavis 接手 agent per DEC-008) / 修订人=Ulysses — Mavis 接手
- **5 域独立 Lead** (8/21 JST): bridge 是工具层, 不算 6 域, 不参与 5 域 Lead 签字
