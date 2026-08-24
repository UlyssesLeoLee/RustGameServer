# distroless base image Dockerfile（per WBS v0.3 §2A.5 WF-1-53.13）
# 多阶段构建：rust:1.98 builder + gcr.io/distroless/cc-debian12 runtime
# 3 个 target：dev（带 cargo/clippy/rustfmt）/ staging / prod
# 规范：RGS-IMPL-005 §3 + RGS-OPS-001 §3.2 Dockerfile 模板

# ==================== 通用 builder ====================
FROM rust:1.98-slim AS chef
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev ca-certificates protobuf-compiler && \
    rm -rf /var/lib/apt/lists/*
RUN cargo install cargo-chef --locked

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ==================== builder stage（共享） ====================
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --workspace --locked

# ==================== grpc_health_probe（mTLS exec 探针，per RGS-OPS-101） ====================
FROM debian:12-slim AS health-probe
ARG GRPC_HEALTH_PROBE_VERSION=v0.4.56
ARG GRPC_HEALTH_PROBE_SHA256=dc13e24d92cdd05d1eb9faf7192c65057dc5b52d38b01aa56188e6899604ec93
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl && \
    curl -fsSL -o /bin/grpc_health_probe \
      "https://github.com/grpc-ecosystem/grpc-health-probe/releases/download/${GRPC_HEALTH_PROBE_VERSION}/grpc_health_probe-linux-amd64" && \
    echo "${GRPC_HEALTH_PROBE_SHA256}  /bin/grpc_health_probe" | sha256sum -c - && \
    chmod +x /bin/grpc_health_probe && \
    rm -rf /var/lib/apt/lists/*

# ==================== runtime base（distroless cc） ====================
FROM gcr.io/distroless/cc-debian12:nonroot AS runtime-base
WORKDIR /app
COPY --from=builder /app/target/release/ /app/bin/
COPY --from=health-probe /bin/grpc_health_probe /bin/grpc_health_probe
USER nonroot:nonroot

# ==================== dev target ====================
FROM rust:1.98-slim AS dev
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev ca-certificates git protobuf-compiler && \
    rm -rf /var/lib/apt/lists/*
COPY . .
ENV CARGO_HOME=/usr/local/cargo \
    RUST_BACKTRACE=1 \
    RUST_LOG=info
CMD ["cargo", "run", "--release"]

# ==================== staging target ====================
FROM runtime-base AS staging
ENV RUST_LOG=info \
    RUST_BACKTRACE=1
# 53.13 接受：staging 默认启动 player-service；多 service 拆分待 54.x
CMD ["/app/bin/player-service"]

# ==================== prod target（默认） ====================
FROM runtime-base AS prod
ENV RUST_LOG=warn \
    RUST_BACKTRACE=0
# prod 默认启动 player-service；多 service 用 k8s Deployment 单独跑
CMD ["/app/bin/player-service"]

# ==================== build ====================
# 用法：
#   docker build --target dev      -t ghcr.io/ulyssesleolee/rustgameserver:dev         .
#   docker build --target staging  -t ghcr.io/ulyssesleolee/rustgameserver:0.1.0-staging .
#   docker build --target prod     -t ghcr.io/ulyssesleolee/rustgameserver:0.1.0        .
# 实际启用：docker-build.yml 待 53.7 注释解除（per commit 621aa0c）+ WF-1-57.8 cosign keyless
