# G3 + G4 部署运行手册 — IT 主阶段第 0 天

> **目的**:在 k3s dev 集群里跑 `cargo test --workspace + cargo llvm-cov`,把 G3 fixture 真实 PASS + G4 覆盖率数字拿到
> **作者**:Mavis(接手 agent per DEC-008,2026-08-28 13:00 JST)
> **关联**:`docs/deploy/01-k8s-manifests/60-test-runner-job.yaml` + `it-readiness-check-2026-08-28.md`

---

## 0. 背景

per Ulysses 2026-08-28 12:49 JST 纠正:**真 PG + Docker 在 k3s 集群里应该确保存在**。本机无 Docker daemon 也能跑,因为:
- k3s dev 集群有 postgres statefulset(per `23-postgres-statefulset.yaml`)+ service `postgres:5432`
- 5 域 + cluster-ops 内部用 service DNS 访问
- cargo test 在 k3s pod 里跑,DATABASE_URL 走 service DNS 解析

**G3** = 5 域 fixture IT 真实 PASS 数字(替换 §3 表"🔴 未验证"标)
**G4** = workspace 真实覆盖率 + 文档同步

## 1. 前置确认

### 1.1 集群状态

```bash
# 1. k3s dev 集群 19/19 Pods Running
kubectl get pods -n rgs-dev | grep -c Running
# 期望: 19

# 2. postgres service 可达
kubectl get svc postgres -n rgs-dev
# 期望: ClusterIP 10.x.x.x:5432

# 3. postgres-secret 含 DATABASE_URL
kubectl get secret rgs-postgres-secret -n rgs-dev -o jsonpath='{.data.DATABASE_URL}' | base64 -d
# 期望: postgres://rgs:***@postgres:5432/rgs_test
```

### 1.2 镜像

`ghcr.io/ulyssesleolee/rustgameserver:0.1.0-gm-backend` 需包含:
- cargo + rust toolchain
- cargo-llvm-cov(per G4)
- 完整 workspace 源码(/workspace)
- migrations 目录

**若镜像未带源码**:改用 `docker run -v $PWD:/workspace` 挂载本地源码。

## 2. 一键运行

### 2.1 应用 Job

```bash
# 替换 PLACEHOLDER_NAMESPACE 为实际 namespace
sed -i 's/PLACEHOLDER_NAMESPACE/rgs-dev/g' docs/deploy/01-k8s-manifests/60-test-runner-job.yaml

# 应用
kubectl apply -f docs/deploy/01-k8s-manifests/60-test-runner-job.yaml -n rgs-dev

# 看 Job 状态
kubectl get job test-runner -n rgs-dev
# 期望: COMPLETIONS 1/1, AGE < 1h
```

### 2.2 等待完成(per 1h deadline)

```bash
# 阻塞等
kubectl wait --for=condition=Complete --timeout=600s job/test-runner -n rgs-dev

# 或后台看
kubectl logs -n rgs-dev -l app.kubernetes.io/name=test-runner -f
```

### 2.3 拉取结果

```bash
# 找 pod
TEST_POD=$(kubectl get pods -n rgs-dev -l app.kubernetes.io/name=test-runner -o jsonpath='{.items[0].metadata.name}')
echo "pod: $TEST_POD"

# kubectl cp 拉 evidence
kubectl cp rgs-dev/$TEST_POD:/workspace/evidence ./test-evidence-it-main-stage-$(date +%Y%m%d)
# 期望: test-evidence-it-main-stage-20260828/{cargo-test-workspace.log, cargo-llvm-cov-workspace.log, lcov-workspace.info, manifest.json, coverage-summary.json}
```

## 3. 关键产出

### 3.1 G3: fixture 真实 PASS 数字

看 `test-evidence-it-main-stage-*/cargo-test-workspace.log`:
- 找 `test result: ok. N passed; M failed`
- 期望:5 域 fixture 全部 PASS(原本 13 fail 应转 0 fail)
- 若仍有 fail → 看 stderr,定位具体 fn 失败原因

### 3.2 G4: workspace 真实覆盖率

看 `test-evidence-it-main-stage-*/coverage-summary.json`:
```json
{
  "batch": "it-main-stage-20260828-130000",
  "test_exit": 0,
  "llvm_cov_exit": 0,
  "coverage_pct": 75.5,
  "hit_lines": 12000,
  "total_lines": 16000
}
```

替换文档占位:
- `it-readiness-check-2026-08-28.md` §2.5 覆盖率"~80%/100%" 改真实数字
- `RGS-TST-08-06-axum-test-vs-wiremock-工具决策.md` §3 评估指标加真实值
- `RGS-TBD-08-05` 文档同步

## 4. 若失败回退

| 现象 | 原因 | 处置 |
|---|---|---|
| `CrashLoopBackOff` | 镜像不包含源码 | 改用 dev image + 挂载本地 workspace |
| `cargo test` timeout | 1h 不够(workspace 大)| activeDeadlineSeconds: 7200 (2h) |
| `DATABASE_URL` 未注入 | postgres-secret 缺 | kubectl edit secret rgs-postgres-secret |
| `pool timeout` | PG service 不可达 | kubectl get svc postgres -n rgs-dev |
| lcov 解析失败 | cargo-llvm-cov 未装 | 镜像 base 加 cargo-llvm-cov |

## 5. 关联文件

- `docs/deploy/01-k8s-manifests/60-test-runner-job.yaml` — Job manifest
- `docs/00-基准与治理/it-readiness-check-2026-08-28.md` — IT 准入 5 项
- `docs/00-基准与治理/test-vs-dtl-audit-2026-08-28.md` — 测试 vs 设计核对
- `crates/rgs-testkit/src/lib.rs` — 强约束 pg_test/pg_pool

---

**作者**:Mavis(接手 agent per DEC-008,2026-08-28 13:00 JST)
**审批**:架构师(Mavis 接手 agent per DEC-008)+ 自审 + 2026-08-28
**修订人**:Ulysses(一人公司 12 角色 per DEC-008)— Mavis 接手
