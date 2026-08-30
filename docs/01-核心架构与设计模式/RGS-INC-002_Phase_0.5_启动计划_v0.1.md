# Phase 0.5 启动计划：5 业务域 K3s 部署基线

| 字段 | 值 |
|---|---|
| 文档 ID | RGS-INC-002 |
| 版本 | v0.1（启动计划草案） |
| 关联 | RGS-INC-001 v0.2 §23 Phase 0.5 / `docs/deploy/07-no-go-checklist_business_v0.1.md` / `docs/deploy/01-k8s-manifests/_status.md` / `docs/deploy/09-deploy-dev-k3s.log` |
| Owner | Ulysses（DEC-008 一人公司：5 域 Lead / SRE / DBA / Platform / QA / 业务方 / 架构师 全部兼任） |
| 周期 | 2~3 周（per RGS-INC-001 v0.2 §23.1） |
| 准入 | 07-no-go-checklist_business_v0.1.md 4 B-CODE 全部 Closed + `cargo test --workspace` 全绿 + 工具链齐备 |
| 决定权 | 5 域 Lead 联合校准 → Ulysses 实际签（per DEC-008） |

---

## 0. 背景

RGS-INC-001 v0.2 §23 插入的 **Phase 0.5 硬阻塞**：5 业务域 gRPC 互通未在 K3s 上跑通，Function Plane 化（Phase 2~7）不能建立在「5 业务域 Pod running」之前的假设上。

**当前状态**（per `09-deploy-dev-k3s.log` + `_status.md` 2026-08-23 扫描）：

- 6 业务域 + cluster-ops：A=✅ B=✅ **C=❌ D=❌**（代码就绪 / 编译通过 / **K3s 未跑** / **流量不可达**）
- 11 个 K8s manifest 全部 PLACEHOLDER（🔴 NO-GO 占位）
- NATS / OTel / Prometheus / Grafana：K3s **0 Pod**
- 4 B-CODE 全部 🔴 NO-GO（2026-08-23 制定）
- 工具链 5 项 ❌：cargo-deny / cargo-audit / cargo-llvm-cov / helm / kubectl

**本文档定位**：拆分 §23 Phase 0.5 行为 6 步可执行交付物 + 决策矩阵（19×7）+ 准入条件。**不实际实施**——决策权归 5 域 Lead + SRE + DBA 联合（per DEC-008 全部 Ulysses 实际签）。

---

## 1. Phase 0.5 关键产物（per §23 表格 Phase 0.5 行）

| # | 产物 | 输入 | 输出 | 责任人 | 状态判据 |
|---|---|---|---|---|---|
| (1) | 5 Deployment + 5 Service manifest 实际值落地 | `01-k8s-manifests/` 11 个 PLACEHOLDER yaml + §4 决策矩阵 | 5 业务域 manifest 实际值（resources / replicas / image tag / env / probes） | 5 域 Lead 联合 + SRE | `kubectl apply --dry-run=server` 全 PASS；`kubectl get deploy -n rgs` 5/5 Ready |
| (2) | NATS JetStream K8s Deployment + Service | shared-platform 已有 `async_nats` client；server side 缺失 | NATS JetStream StatefulSet + Headless Service + PVC | SRE（兼 Platform） | `nats stream ls` 列出 6 个 Stream（pl / ec / mt / gd / ad / co）；Subject 数 ≥ 现状 NATS 主题数 |
| (3) | OTel Collector + Prometheus + Grafana 3 套 K3s manifest | `docker/observability/` 已有 compose；K3s manifest 缺失 | 3 套 Deployment/Service/ConfigMap | Platform 架构师（兼 SRE） | 3/3 Pod running；Grafana UI 可访问；Prometheus scrape 5 业务域 + 自身 |
| (4) | mTLS 证书签发 + Secret 注入 | `rgs-certgen` 工具（`rcgen`）已就位；Secret 模板 PLACEHOLDER | 6 域证书（pl / ec / mt / gd / ad / co）+ CA 证书；K8s Secret 注入 | SRE | `rgs-certgen` 跑通；`kubectl get secret -n rgs` 7 个 secret 存在；`MTLS_BYPASSED_TOTAL=0` 启动 5 业务域 |
| (5) | docker image 构建流水线落地 + registry | `.github/workflows/docker-build.yml` + `Dockerfile` 已就位；registry 未接入 | ghcr.io（或自建）image 推送 + pull secret + tag 策略 | Platform 架构师 | 6 业务域镜像 `docker build` + `docker push` 成功；K3s pull secret 配通 |
| (6) | end-to-end smoke test | 上述 (1)~(5) 全部就位 | 5 业务域 HealthCheck OK + trace_id 跨域串联 | QA Lead（兼 5 域 Lead） | B-CODE-01~04 全部 Closed；`08-measure-env-setup.log` 追加 Section 7 "5 业务域 Pod status" |

---

## 2. Phase 0.5 准入条件（per §23 表格 Phase 0.5 准入 (a)~(d)）

| # | 准入 | 验证方法 | 责任人 | 通过判据 |
|---|---|---|---|---|
| (a) | B-CODE-01~04 全部 Closed | `07-no-go-checklist_business_v0.1.md` 4 条 B-CODE 状态从 🔴 → 🟢，附实测 log（`b1-otel-pod-up.log` / `b2-player-grpc-healthcheck.log` / `b3-session-pg-trace.log` / `b4-cross-domain-trace.log`） | SRE + Platform + player 域 Lead（DEC-008 兼任） | 4/4 标注 Closed + 实测 log 引用齐全 |
| (b) | `cargo test --workspace` 全绿（含 9 crate，不只 rgs-hello） | `cargo test --workspace` exit code 0；输出覆盖 player / economy / match / social / admin / cluster-ops / shared-platform / rgs-certgen / rgs-hello | Platform + QA（兼任） | 9/9 crate 0 失败；测试总数与 baseline 一致（无回归） |
| (c) | cargo-deny / cargo-audit / cargo-llvm-cov 全部安装 + PASS | §5 工具链 4 项验证命令 | Platform 架构师 | 4/4 验证命令 exit 0 |
| (d) | helm v3.10+ 安装 + 至少一次 dry-run 通过 | `helm version` ≥ v3.10；`helm install --dry-run` 对一个 chart 跑通（参考占位 `02-helm-charts/`） | SRE | version 满足 + dry-run exit 0 |

---

## 3. 6 步执行序列

按依赖顺序（前步准入 = 后步启动条件）。

### Step 1: 5 业务域 K8s manifest 实际值落地

- **目标**：把 11 个 PLACEHOLDER yaml 中的实际值按 §4 决策矩阵填入
- **输入**：11 文件清单 + §4 决策矩阵 + `cargo build` binary size
- **产出**：5 Deployment + 5 Service + HPA + PDB + SA + ConfigMap + Secret 模板
- **责任人**：5 域 Lead 联合 + SRE 协调
- **周期**：3~5 天
- **准入判据**：`kubectl apply --dry-run=server` 11/11 PASS
- **失败回退**：5 域 Lead 不一致 → 架构师仲裁；1 周内闭环
- **对应 B-CODE**：B-CODE-02 / 03

### Step 2: NATS JetStream K8s Deployment + Service

- **目标**：K3s 上跑通 NATS JetStream（Phase 2 异步事件前置）
- **输入**：`shared-platform` 已有 `async_nats` client；K3s 无 NATS manifest（**需新建**）
- **产出**：NATS JetStream StatefulSet（1 副本 + PVC 5Gi）+ Headless Service + Cluster Service
- **责任人**：SRE（兼 Platform）
- **周期**：2~3 天
- **准入判据**：`nats stream ls` 列出 6 Stream（pl / ec / mt / gd / ad / co）
- **失败回退**：单节点 NATS 起步；cluster 需 ADR 单独登记
- **对应 B-CODE**：B-CODE-04

### Step 3: 可观测性栈（OTel Collector + Prometheus + Grafana）K3s manifest

- **目标**：把 `docker/observability/` compose 三件套移植到 K3s manifest
- **输入**：`otel-collector-config.yaml` + `prometheus.yml` + `rgs-services-overview.json`
- **产出**：OTel Collector + Prometheus + Grafana 三 Deployment + Service + ConfigMap + PVC
- **责任人**：Platform 架构师（兼 SRE）
- **周期**：3~4 天
- **准入判据**：3/3 Pod running；Prometheus `/api/v1/targets` 5 业务域 up
- **失败回退**：先不上 Loki/Tempo（ARC-014 未批准不引入）；单独 ADR 评估
- **对应 B-CODE**：B-CODE-01 / 04

### Step 4: mTLS 证书签发（rgs-certgen 实跑）+ Secret 注入

- **目标**：`rgs-certgen` 跑通 6 域证书签发 + 注入 K8s Secret
- **输入**：`crates/rgs-certgen/`（`rcgen`）+ `09-secret-template.yaml` PLACEHOLDER
- **产出**：6 域证书 + CA 证书 + 7 个 K8s Secret
- **责任人**：SRE（兼 5 域 Lead）
- **周期**：1~2 天
- **准入判据**：`kubectl get secret -n rgs` 7/7 存在；5 业务域 Pod `MTLS_BYPASSED_TOTAL=0` 启动
- **失败回退**：dev 临时 `RGS_ALLOW_INSECURE_GRPC=1`（55.26 opt-out），计数器必须上报；生产前闭环 mTLS
- **对应 B-CODE**：B-CODE-02 / 03

### Step 5: docker image 构建流水线（rust-ci → ghcr.io）落地 + registry 接入

- **目标**：6 业务域镜像可被 K3s 拉起
- **输入**：`docker-build.yml` + `Dockerfile`（rust:1.98 builder → distroless runtime）已就位
- **产出**：6 业务域镜像 `ghcr.io/rust-game-server/<service>:<tag>` 推送；K3s imagePullSecret 配通
- **责任人**：Platform 架构师（兼 QA）
- **周期**：2~3 天
- **准入判据**：`docker build` + `docker push` 6/6 成功；K3s 节点能 pull
- **失败回退**：dev `imagePullPolicy: Never` + 节点预加载；Phase 0.5 不上 cosign（per 57.8 注释未解除）
- **对应 B-CODE**：B-CODE-02

### Step 6: end-to-end smoke test（5 业务域 HealthCheck OK + trace_id 跨域串联）

- **目标**：K3s 上跑通 5 业务域 gRPC 互通 + OTel trace 跨域串联
- **输入**：Step 1~5 全部就位
- **产出**：4 份实测 log（`b1-otel-pod-up.log` / `b2-player-grpc-healthcheck.log` / `b3-session-pg-trace.log` / `b4-cross-domain-trace.log`）；`08-measure-env-setup.log` 追加 Section 7
- **责任人**：QA Lead（兼 5 域 Lead + SRE）
- **周期**：1~2 天
- **准入判据**：4 B-CODE 全部 Closed；Grafana 输入 trace_id 可见 player → economy 跨域调用树
- **失败回退**：trace 断裂 → 查 `grpc_tracing` `traceparent` 注入；HealthCheck 失败 → 查 mTLS Secret
- **对应 B-CODE**：B-CODE-01 / 02 / 03 / 04 全部

---

## 4. 决策矩阵（需 5 域 Lead 联合校准）

> **状态**：全部 `TBD` —— 实际值待 5 域 Lead + SRE + DBA 在 Step 1 联合校准后填入。**本文档 v0.1 不预先编造任何数值**。
>
> 校准输入：现状 `cargo build --release` binary size + 9-crate `cargo test --workspace` 测试负载 + 06-rust-198-build.log 实测记录。
>
> 列说明：
> - **player / economy / match / social / admin** = 5 业务域 Lead 决策权
> - **cluster-ops** = SRE 决策权（per ADR-0052 PFAU Active-Active 单独考虑）

| # | 字段 | player | economy | match | social | admin | cluster-ops |
|---|---|---|---|---|---|---|---|
| 1 | replicas | TBD | TBD | TBD | TBD | TBD | TBD |
| 2 | resources.requests.cpu | TBD | TBD | TBD | TBD | TBD | TBD |
| 3 | resources.requests.memory | TBD | TBD | TBD | TBD | TBD | TBD |
| 4 | resources.limits.cpu | TBD | TBD | TBD | TBD | TBD | TBD |
| 5 | resources.limits.memory | TBD | TBD | TBD | TBD | TBD | TBD |
| 6 | image tag | TBD | TBD | TBD | TBD | TBD | TBD |
| 7 | imagePullPolicy | TBD | TBD | TBD | TBD | TBD | TBD |
| 8 | env.GRPC_ADDR | TBD | TBD | TBD | TBD | TBD | TBD |
| 9 | env.DATABASE_URL | TBD | TBD | TBD | TBD | TBD | TBD |
| 10 | env.NATS_URI | TBD | TBD | TBD | TBD | TBD | TBD |
| 11 | env.RGS_TLS_DIR | TBD | TBD | TBD | TBD | TBD | TBD |
| 12 | livenessProbe.initialDelaySeconds | TBD | TBD | TBD | TBD | TBD | TBD |
| 13 | readinessProbe.periodSeconds | TBD | TBD | TBD | TBD | TBD | TBD |
| 14 | ServiceAccount | TBD | TBD | TBD | TBD | TBD | TBD |
| 15 | HPA minReplicas | TBD | TBD | TBD | TBD | TBD | TBD |
| 16 | HPA maxReplicas | TBD | TBD | TBD | TBD | TBD | TBD |
| 17 | HPA target CPU% | TBD | TBD | TBD | TBD | TBD | TBD |
| 18 | PDB minAvailable | TBD | TBD | TBD | TBD | TBD | TBD |
| 19 | 网络 egress 是否需要 | TBD | TBD | TBD | TBD | TBD | TBD |

**校准建议（per ARC-005/007/008/021 + §3.2 实时主路径保护原则）**：

- player / economy / cluster-ops 至少 2 副本（minReplicas ≥ 2）—— 失败影响"极高"或"高"
- match / social 1→N（minReplicas=1 起步）—— 失败影响"高"或"中"
- admin 1→N —— 失败影响"中"（审计链不断即可容忍短暂 GM 不可用）
- realtime 域（match 核心对局撮合）**不部署 KEDA**（per §3.2 不可插入 Knative Activator）
- env.GRPC_ADDR 走 0.0.0.0 + Service 端口 50051~50056（per §1.3）
- env.DATABASE_URL 走 K8s Secret 注入（per ARC-007 禁止直连业务 DB 字符串写在 manifest）
- HPA 默认 CPU 60%（per RGS-BAS-001 §4 经验值，待实测校准）

---

## 5. 工具链补齐清单（per 07-no-go-checklist_business_v0.1.md §2）

| 工具 | 当前 | 装法 | 验证 |
|---|---|---|---|
| cargo-deny | ❌ NOT_INSTALLED | `cargo install cargo-deny --locked` | `cargo deny check` exit 0 |
| cargo-audit | ❌ NOT_INSTALLED | `cargo install cargo-audit --locked` | `cargo audit` 无 RUSTSEC 公告 |
| cargo-llvm-cov | ❌ NOT_INSTALLED | `cargo install cargo-llvm-cov --locked` | `cargo llvm-cov --workspace` 报告生成 |
| helm v3.10+ | ❌ WSL_ERROR | `curl https://raw.githubusercontent.com/helm/helm/main/scripts/get-helm-3 -o get_helm.sh && bash get_helm.sh` | `helm version` ≥ v3.10 |
| kubectl ≥ v1.30 | ❌ WSL_ERROR | `az aks install-cli` 或 `curl -LO https://dl.k8s.io/release/v1.30.0/bin/linux/amd64/kubectl && chmod +x kubectl && mv kubectl /usr/local/bin/` | `kubectl version --client` ≥ v1.30 |

> 数据来源：`docs/deploy/07-no-go-checklist_business_v0.1.md` §2（2026-08-23 制定，照搬状态）。

---

## 6. 文档更新义务（per §23.4 Phase 0.5 三条）

Phase 0.5 完成时**必须**更新以下 3 份文档：

1. **`docs/deploy/01-k8s-manifests/_status.md`**：11 个 manifest 从占位 → 🟢（附实际值 + kubectl apply 日志 + 责任人签字）
2. **`docs/deploy/07-no-go-checklist_business_v0.1.md`**：4 条 B-CODE 从 🔴 → 🟢（附 4 份实测 log 引用：`b1-otel-pod-up.log` / `b2-player-grpc-healthcheck.log` / `b3-session-pg-trace.log` / `b4-cross-domain-trace.log`）
3. **`docs/deploy/08-measure-env-setup.log`** 追加 Section 7 "5 业务域 Pod status"（`kubectl get pods -n rgs` 输出 + `kubectl get svc -n rgs` 输出）

CI 强制：`docs-ci.yml` 校验链接 / TOC / 编号（per RGS-INC-001 v0.2 §23.4）。

---

## 7. 风险与回退（per RGS-INC-001 v0.2 §25 RISK-INC-*）

| 风险 ID | 描述 | Phase 0.5 涉及 | 缓解 |
|---|---|---|---|
| **RISK-INC-001** | KEDA 引入 5 域 OLU 不足 | 间接（Phase 4）| Phase 0.5 不引入 KEDA；§11.3 OLU 申领 + 季度 review |
| **RISK-INC-006** | OLU 超 NFR-OP-010 2 SRE·周 | **直接** | token-OLU 框架 per `RGS-TS-001 v0.4 §6.2`（1 SRE·周 ≈ 1M tokens，1 人·天 ≈ 100K-300K）；任何新组件必须先 ADR |
| **RISK-INC-011** | mTLS fail-closed 与 Function Gateway 冲突 | **直接**（Step 4）| Phase 0.5 必须先解 mTLS；§16.3 复用 `shared_platform::tls` |
| **RISK-INC-012** | KEDA / Wasmtime 后 OTel 链路断裂 | **直接**（Step 3）| Phase 0.5 必须先建 OTel 栈；§18.2 traceparent 透传强制；端到端 = Step 6 |
| **RISK-INC-013** | 命名空间 / RBAC 误配 | 间接 | `deny-all` default NetworkPolicy + 季度审计（§17 + §33）|
| **RISK-INC-015** | 文档/Saga/Contract 不一致 | **直接** | §23.4 文档更新义务 + `docs-ci.yml` 强制 |

**总周期 OLU 估算**（per RISK-INC-006）：

- Phase 0.5：2~3 周 × 1 SRE（Ulysses 实际） ≈ 1.0~1.5 SRE·周 ≈ 1.0~1.5M tokens
- Phase 0.5 + Phase 1（Benchmark 2 周）合计：~2.0~2.5M tokens
- NFR-OP-010：2 SRE × 1 周 ≈ 2.0M tokens/周
- 结论：Phase 0.5 单周峰值接近上限，**不可并行 Phase 1 任何子任务**

---

## 8. Phase 1 启动条件（衔接）

Phase 0.5 全闭环后，必须**同时满足**以下 5 条才能进 RGS-INC-001 v0.2 §23 Phase 1 Benchmark：

1. 4 B-CODE 全部 🟢 Closed（附实测 log）
2. 11/11 manifest 🟢（附责任人签字 + apply 日志）
3. `cargo test --workspace` 0 回归（9/9 crate 全绿）
4. 工具链 5/5 安装 + 验证 PASS
5. Wasmtime 集成已在 mock 验证（per commit `4b5526e`）

**禁止** Phase 0.5 跨过任何一条准入直接进 Phase 1。

---

## 9. 修订历史

| 版本 | 日期 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-23 | 架构师（Ulysses）| 初版启动计划草案：拆分 §23 Phase 0.5 6 关键产物 + 4 准入 + 6 步执行 + 19×7 决策矩阵（全部 TBD） + 工具链 + 文档义务 + 风险 + Phase 1 衔接 |

---

## 附录 A. 关联文档

- 上游：`RGS-INC-001 v0.2 §23 Phase 0.5` + §23.1 周期估算 + §23.2 依赖图 + §23.4 文档义务 + §25 RISK-INC-*
- 同级：`docs/deploy/07-no-go-checklist_business_v0.1.md`（4 B-CODE）
- 状态源：`docs/deploy/01-k8s-manifests/_status.md`（11 manifest 占位）
- 部署 log：`docs/deploy/09-deploy-dev-k3s.log` + `docs/deploy/08-measure-env-setup.log`
- 治理：`RGS-PLAN-001 v0.8 §3.3` + `RGS-ENV-001 v0.3 §6` + `RGS-EXEC-001 v0.3`
- 决策：`DEC-008`（一人公司治理基线）+ `DEC-009` + `DEC-010` + `DEC-005`
- OLU 框架：`RGS-TS-001 v0.4 §6.2`（token-OLU 草案）

> **本文档 v0.1 状态**：草案待 5 域 Lead + SRE + DBA 联合校准（实际签均 Ulysses，per DEC-008）。实际值校准入 Step 1 → 决策矩阵填入 → 出 v0.2 替换本草案。
