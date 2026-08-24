# RGS-OPS-101 gRPC 健康探针 mTLS 兼容性修复设计

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-OPS-101 |
| 版本 | 0.1（初版） |
| 制定日 | 2026-08-24 |
| 最终更新日 | 2026-08-24 |
| 制定者 | 架构师（Ulysses 兼，per DEC-008 一人公司） |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |
| 关联文档 | RGS-DEC-015（mTLS fail-closed，P1）/ RGS-REV-008 AC-1 / RGS-REV-009 HI-1 / `docs/deploy/01-k8s-manifests/` / `Dockerfile` / RGS-OPS-100（K3s 部署设计，姊妹篇） |
| 配套标准 | IPA 共通フレーム 2013 + 150 工程日本 SI 业界标准；V 模型映射：UT ↔ DTL |

---

## 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-24 | 架构师（Ulysses） | 初版。Phase 0.5 部署验证发现 6 服务 CrashLoopBackOff，定位为 k8s 原生 `grpc:` 探针与 fail-closed mTLS 结构性不兼容 + Health 服务未注册两个缺陷；确定 Option A（exec + grpc_health_probe + mTLS 客户端身份复用）为修复方案。 |

---

## 0. 文档目的

Phase 0.5 部署验证阶段，`player-service`（及其余 5 域服务）Pod 持续 `CrashLoopBackOff`。进程本身未崩溃（日志显示 `mTLS ENABLED` + `binding gRPC server at 0.0.0.0:50051` 均已打印成功），但 kubelet 的 readiness/liveness 探针始终无法在超时内确认健康，导致 kubelet 反复重启容器。

本文档定义该问题的需求、方案设计、详细设计、实施规格与实施计划，目标是在**不破坏 DEC-015 P1 fail-closed mTLS 不变式**（网络上不存在任何明文 gRPC 服务端点）的前提下，让 6 个服务的存活/就绪探针可靠工作。

---

## 1. 需求定义（REQ）

### 1.1 背景 / 问题现象

定位到两个独立但叠加的缺陷：

1. **Health 服务从未注册**。全仓库 `grep -rn "tonic_health\|HealthServer\|health_reporter"` 命中为 0——6 个服务的 `main.rs` 都只 `add_service(<业务 Service>)`，从未 `add_service(<grpc.health.v1.Health>)`。任何针对 Health 服务的 RPC 都会收到 `UNIMPLEMENTED`。
2. **k8s 原生 `grpc:` 探针无法出示客户端证书**。6 份 manifest（`01~06-*.yaml`）均使用：
   ```yaml
   livenessProbe:
     grpc:
       port: 50051
   ```
   kubelet 内建 gRPC 探针（stable since 1.27）在 PodSpec 层面**没有 TLS/客户端证书配置项**，探测请求恒为明文。而 `main.rs` 通过 `shared_platform::tls::load_server_tls_config` 强制 `client_ca_root`，tonic 0.12 默认 `client_auth_optional = false`——服务端在 TLS 握手阶段即拒绝无证书连接。探针连接因此在握手层被拒绝/挂起，而非在应用层收到 RPC 错误，表现为超时而非快速失败。

### 1.2 需求

- 探针必须能准确反映 6 个服务的存活（liveness）与就绪（readiness）状态。
- **不得在网络上新增任何明文 gRPC 监听端点**（即不接受"开一个不认证的健康检查端口"这类方案，因为这与 DEC-015 P1 fail-closed mTLS 的既定安全决策相冲突）。
- 修复必须适用于全部 6 服务（player / economy / match / social / admin / cluster-ops），不允许出现服务间不一致的探针实现。
- 修复不得引入新的证书类型或新的 Secret（避免扩大 Phase 0.5 已完成的证书签发/分发工作量）。

### 1.3 范围

**包含**：6 服务 `Cargo.toml` + `main.rs`、共享 `Dockerfile`、6 份 `docs/deploy/01-k8s-manifests/0{1..6}-*.yaml` 的探针字段。

**不包含**：证书签发流程（`rgs-certgen` / `phase-0-5-step-4-*.ps1`）、NetworkPolicy、其余 Phase 0.5 未完成事项。

### 1.4 非目标（明确拒绝的方案）

- **Option B**（开独立明文健康端口，仅供 kubelet 探测）：技术上可行、改动量更小，但会在 Pod 网络内引入一个无客户端认证的 gRPC 端点，与 DEC-015 P1 的"fail-closed，无例外"精神相悖。本次不采用。

---

## 2. 基本设计（BAS）

### 2.1 总体方案（Option A）

保持"全链路无明文 gRPC"不变式，探针改走**能出示 mTLS 客户端证书的 exec 探针**：

1. 每个服务的 tonic server 注册标准 `grpc.health.v1.Health` 服务（`tonic-health` crate），随业务 Service 一并挂在同一个 mTLS 端口上——**不新增端口**。
2. 运行时镜像内置静态编译的 `grpc_health_probe` 二进制（Go 静态二进制，无 libc 依赖，可在 distroless 镜像内直接执行）。
3. k8s 探针类型从 `grpc:` 切换为 `exec:`，探针进程即 `grpc_health_probe`，携带 `-tls` 系列参数发起一次真正的 mTLS 握手后再查询 Health 服务。
4. **客户端身份复用服务自身的 mTLS server 证书**：`rgs-certgen`（`crates/rgs-certgen/src/main.rs`）生成证书时未设置 `ExtendedKeyUsage`，因此每个域证书（`/etc/rgs/certs/server.pem` + `server.key`，已通过现有 `rgs-tls` projected volume 挂载）本身即可同时充当 serverAuth 与 clientAuth 身份，探针"自己拿自己的证书探自己"，无需签发新证书、无需新 Secret。

### 2.2 关键约束

- 探针请求路径：kubelet → Pod 网络接口 :50051（exec 探针在容器 netns 内执行，可直接连 `127.0.0.1:50051`）→ 与业务流量完全相同的 mTLS 端口/相同的 rustls `ServerTlsConfig`。
- 第三方二进制（`grpc_health_probe`）经 GitHub Release 下载，必须做 SHA-256 校验后再固化进镜像层，防止供应链投毒（详见 §4.2）。
- 保持每个服务原有的 `initialDelaySeconds` / `periodSeconds` / `timeoutSeconds` / `failureThreshold` 数值不变，只替换探测方式本身。

### 2.3 影响范围

| 层 | 改动 |
|---|---|
| Rust 代码 | workspace + 6 服务 `Cargo.toml` 加 `tonic-health`；6 服务 `main.rs` 注册 Health 服务 |
| 镜像 | `Dockerfile` 新增 `health-probe` 构建阶段，`runtime-base` 内置二进制 |
| K8s 清单 | 6 份 manifest 的 `livenessProbe` / `readinessProbe` 从 `grpc:` 改 `exec:` |
| 证书 / Secret | **无改动**（复用现有 `rgs-tls` volume） |

---

## 3. 详细设计（DTL）

### 3.1 Rust 依赖

`Cargo.toml`（workspace）新增：

```toml
tonic-health = "0.12"   # 与 tonic 0.12 对齐
```

6 服务各自 `Cargo.toml` 的 `[dependencies]` 内，紧跟 `tonic = { workspace = true }` 之后加：

```toml
tonic-health = { workspace = true }
```

### 3.2 `main.rs` 改动模式（6 服务一致）

以 `player-service` 为例（其余 5 服务替换对应的 `*ServiceServer` / `*GrpcService` 类型名即可，模式完全一致）：

```rust
// DB pool + migrations 已在此之前完成（失败已 exit(1)），
// 此时才注册 Health 服务，保证 "SERVING" 与 "DB 可用" 语义一致。
let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
health_reporter
    .set_serving::<player_service::proto::v1::player_service_server::PlayerServiceServer<PlayerGrpcService>>()
    .await;
// k8s exec 探针默认查询空 service name（对应"整体健康"语义），需显式设置。
health_reporter
    .set_service_status("", tonic_health::ServingStatus::Serving)
    .await;

...

server_builder
    .add_service(svc)
    .add_service(health_service)
    .serve(addr)
    .await
    .context("tonic server failed")?;
```

设计要点：

- `health_reporter` 的注册点放在 DB pool / migrations 成功**之后**、`serve()` 之前——与现有"DB 失败即 `exit(1)`，进程根本起不到 serve 这一步"的 fail-fast 模式天然一致，无需额外的 readiness 状态机。
- 同时设置具名 service（`PlayerServiceServer<...>`）与空字符串 `""` 两个 key：前者供未来更细粒度探测（如只查业务 Service）使用，后者匹配 `grpc_health_probe` 默认不传 `-service` 时的查询目标。

### 3.3 Dockerfile 改动

```dockerfile
# ==================== grpc_health_probe（mTLS exec 探针，per RGS-OPS-101）====================
FROM debian:12-slim AS health-probe
ARG GRPC_HEALTH_PROBE_VERSION=v0.4.56
ARG GRPC_HEALTH_PROBE_SHA256=dc13e24d92cdd05d1eb9faf7192c65057dc5b52d38b01aa56188e6899604ec93
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl && \
    curl -fsSL -o /bin/grpc_health_probe \
      "https://github.com/grpc-ecosystem/grpc-health-probe/releases/download/${GRPC_HEALTH_PROBE_VERSION}/grpc_health_probe-linux-amd64" && \
    echo "${GRPC_HEALTH_PROBE_SHA256}  /bin/grpc_health_probe" | sha256sum -c - && \
    chmod +x /bin/grpc_health_probe && \
    rm -rf /var/lib/apt/lists/*
```

`runtime-base` 阶段追加一行：

```dockerfile
COPY --from=health-probe /bin/grpc_health_probe /bin/grpc_health_probe
```

要点：`grpc_health_probe` 是 CGO_ENABLED=0 的纯静态 Go 二进制，在 `gcr.io/distroless/cc-debian12`（无 shell、有 glibc 供 cc 类应用用）下可直接 `exec` 运行，无需 shell 包装。

### 3.4 K8s 探针改动（6 服务通用模板）

以 `01-player-service.yaml`（端口 50051，CN/SAN=`player.service`）为例：

```yaml
livenessProbe:
  exec:
    command:
      - /bin/grpc_health_probe
      - -addr=127.0.0.1:50051
      - -tls
      - -tls-client-cert=/etc/rgs/certs/server.pem
      - -tls-client-key=/etc/rgs/certs/server.key
      - -tls-ca-cert=/etc/rgs/certs/ca.pem
      - -tls-server-name=player.service
      - -connect-timeout=2s
  initialDelaySeconds: 30
  periodSeconds: 30
  timeoutSeconds: 5
  failureThreshold: 3
readinessProbe:
  exec:
    command:
      - /bin/grpc_health_probe
      - -addr=127.0.0.1:50051
      - -tls
      - -tls-client-cert=/etc/rgs/certs/server.pem
      - -tls-client-key=/etc/rgs/certs/server.key
      - -tls-ca-cert=/etc/rgs/certs/ca.pem
      - -tls-server-name=player.service
      - -connect-timeout=2s
  initialDelaySeconds: 10
  periodSeconds: 10
  timeoutSeconds: 3
  failureThreshold: 3
```

其余 5 服务仅替换 `-addr` 端口号与 `-tls-server-name` 域名，`initialDelaySeconds` 等数值保留各自原有配置不变：

| 服务 | 端口 | `-tls-server-name` |
|---|---|---|
| player | 50051 | `player.service` |
| economy | 50052 | `economy.service` |
| match | 50053 | `match.service` |
| social | 50054 | `social.service` |
| admin | 50055 | `admin.service` |
| cluster-ops | 50056 | `cluster-ops.service` |

（实际端口号以各 manifest 现值为准，实施时逐份读取后对齐，不假设线性递增。）

---

## 4. 实装规格（SPEC）

### 4.1 变更文件清单

- `Cargo.toml`（workspace，加 `tonic-health`）
- `crates/{player,economy,match,social,admin,cluster-ops}-service*/Cargo.toml`（或 `crates/cluster-ops/Cargo.toml`，加依赖）
- `crates/{player,economy,match,social,admin}-service/src/main.rs` + `crates/cluster-ops/src/main.rs`（注册 Health 服务）
- `Dockerfile`（新增 `health-probe` stage + `runtime-base` COPY）
- `docs/deploy/01-k8s-manifests/01~06-*.yaml`（探针 `grpc:` → `exec:`）

### 4.2 供应链完整性（新增约束）

`grpc_health_probe` 二进制来自 GitHub Release，非本仓库产物，必须固定版本号 + SHA-256 校验后写入 `Dockerfile`（见 §3.3），任何 checksum 不匹配都必须让 `docker build` 失败（`sha256sum -c -` 天然满足）。版本升级时需同步更新 `GRPC_HEALTH_PROBE_VERSION` 与 `GRPC_HEALTH_PROBE_SHA256` 两个 ARG。

### 4.3 验收标准（AC）

- AC-1：`cargo build --workspace --locked` 全绿。
- AC-2：`docker build --target prod .` 成功，且镜像内 `/bin/grpc_health_probe` 可执行（`docker run --rm --entrypoint /bin/grpc_health_probe <image> -h` 返回用法说明而非报错）。
- AC-3：本地或测试集群部署后，6 个 Pod 的 `kubectl describe pod` 不再出现 `CrashLoopBackOff`，`READY 1/1`。
- AC-4：`RGS_ALLOW_INSECURE_GRPC=1`（dev/test opt-out 场景）下探针仍能正常工作（`grpc_health_probe` 在此模式下应去掉 `-tls` 系列参数——两种模式互斥，不在本次范围内自动切换，留作已知限制，见 §6）。
- AC-5：`docs/deploy/phase-0-5-step-4-validate-fail-closed.ps1` 复跑通过（确认本次改动未削弱既有 fail-closed 校验）。

---

## 5. 实施计划

| 步骤 | 内容 | 产出 |
|---|---|---|
| 1 | workspace `Cargo.toml` + 6 服务 `Cargo.toml` 加 `tonic-health` 依赖 | 依赖声明 |
| 2 | 6 服务 `main.rs` 注册 `health_reporter` / `health_service`，`add_service` 挂载 | 代码改动 |
| 3 | `cargo build --workspace --locked` 本地验证编译通过 | 编译通过 |
| 4 | `Dockerfile` 加 `health-probe` stage（pin 版本 + sha256）+ `runtime-base` COPY | 镜像改动 |
| 5 | 6 份 k8s manifest 探针 `grpc:` → `exec:`（按 §3.4 表格填入各服务端口/域名） | 清单改动 |
| 6 | `docker build --target staging .` 验证镜像内二进制可执行 | 镜像验证 |
| 7 | 部署到测试集群，观察 6 个 Pod 从 `CrashLoopBackOff` 恢复到 `Running/Ready` | 部署验证 |
| 8 | 复跑 `phase-0-5-step-4-validate-fail-closed.ps1`，确认 fail-closed 不变式未被破坏 | 回归验证 |

**回滚方案**：k8s manifest / Dockerfile 变更均可通过 `git revert` 还原；`kubectl apply` 幂等，回滚即重新 apply 旧版 manifest。无数据库 schema 变更，无需数据回滚。

---

## 6. 已知限制 / 后续事项

- `RGS_ALLOW_INSECURE_GRPC=1`（dev/test 明文 opt-out）场景下，探针命令仍写死 `-tls` 参数，会导致该模式下探针失败。当前 6 份生产/staging manifest 均未启用该 opt-out，故不阻塞本次修复；若未来需要同一套 manifest 同时支持两种模式，需通过 initContainer 或 ConfigMap 参数化探针命令，留待后续 Phase 处理。
- `-tls-server-name` 依赖各域证书 SAN 与 manifest 硬编码值保持一致；若 `rgs-certgen` 未来改域名命名规则，需同步更新本文档 §3.4 表格与对应 manifest。
