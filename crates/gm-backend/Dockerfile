# gm-backend Dockerfile (per RGS-IMPL-005 §3, 单 binary distroless)
# 复用顶层 Dockerfile 模式(builder + distroless runtime),只打包 /app/bin/gm-backend
FROM rust:1.98-slim AS chef
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev ca-certificates protobuf-compiler && rm -rf /var/lib/apt/lists/*
RUN cargo install cargo-chef --locked

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --locked -p gm-backend
RUN cp /app/target/release/gm-backend /tmp/gm-backend-binary

FROM gcr.io/distroless/cc-debian12:nonroot AS runtime
WORKDIR /app
COPY --from=builder /tmp/gm-backend-binary /app/bin/gm-backend
USER nonroot:nonroot
EXPOSE 8443 8081 9464
ENV RUST_LOG=info
CMD ["/app/bin/gm-backend"]
