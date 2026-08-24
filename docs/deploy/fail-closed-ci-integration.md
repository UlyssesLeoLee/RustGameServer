# fail-closed 验证 CI 接入说明

**文档编号**: RGS-DEPLOY-CI-FC-001
**版本**: 0.1
**制定日**: 2026-08-24
**关联**: WF-1-55.48（B-09）/ RGS-OPEN-QA-001 v0.2 Q-M-08 + ACTIONS-v0.3 B-09

---

## 1. 目的

`scripts/verify_fail_closed.ps1`（5 域 fail-closed 验证脚本，固化了 phase-0-5 step 4 一次性手工验证）是 RGS-INC-001 v0.2 §1.4 fail-closed 防线的**持续性 CI 入口**。本文件说明如何在 CI 中接入并触发，避免 fail-closed 防线被静默降级破坏。

## 2. 触发条件

**触发源（path filter）**：

| 触发范围 | 路径 | 说明 |
|---|---|---|
| K8s manifest 变更 | `docs/deploy/01-k8s-manifests/**` | namespace / Deployment / Service / ConfigMap / Secret / RBAC / NetworkPolicy 任一变更 |
| RBAC 模板变更 | `docs/deploy/01-k8s-manifests/10-rbac-template.yaml` | 5 域 + cluster-ops 6 套 Role/RoleBinding 模板变更 |
| mTLS Secret 变更 | `docs/deploy/01-k8s-manifests/50-secret-*.yaml` | 6 域 mTLS Secret + CA 单例（50-secret-ca.yaml） |
| 验证脚本自身变更 | `scripts/verify_fail_closed.ps1` | 改脚本也应跑一遍验证（防止脚本自身 bug 引入误判 PASS） |

**不触发**（仅文档变更）：
- `docs/00-基准与治理/**/*.md`
- `docs/01-核心架构与设计模式/**/*.md`
- `docs/10-技术选型/**/*.md`（TS-001 文档变更不影响 fail-closed 行为）
- 其它与 K8s 部署无关的文档

**触发 PR 类型**（per Q-M-08 答复：**不是一次性脚本，每次 manifest/RBAC 变更 PR 都触发**）：

- ✅ 任何新增域的 manifest PR
- ✅ 任何修改 RBAC 权限的 PR
- ✅ 任何修改 mTLS Secret 模板的 PR
- ✅ 任何修改 namespace / NetworkPolicy / ResourceQuota 的 PR
- ❌ 纯文档 / 注释 PR（不触发，节省 CI 资源）

## 3. CI workflow 调用方式

### 3.1 GitHub Actions（推荐）

```yaml
# .github/workflows/fail-closed-verify.yml
name: fail-closed-verify
on:
  pull_request:
    paths:
      - 'docs/deploy/01-k8s-manifests/**'
      - 'scripts/verify_fail_closed.ps1'
  workflow_dispatch:  # 允许手动触发

jobs:
  verify:
    runs-on: ubuntu-latest
    timeout-minutes: 15
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Install k3s
        run: |
          curl -sfL https://get.k3s.io | INSTALL_K3S_EXEC="--disable=traefik --disable=servicelb" sh -
          sudo k3s kubectl wait --for=condition=Ready nodes --all --timeout=60s

      - name: Install PowerShell 7
        run: |
          sudo apt-get update
          sudo apt-get install -y powershell

      - name: Apply namespace + base manifests
        run: |
          sudo k3s kubectl apply -f docs/deploy/01-k8s-manifests/00-namespace.yaml
          sudo k3s kubectl apply -f docs/deploy/01-k8s-manifests/50-secret-ca.yaml
          # 其它 5 域 manifest 由 PR 提供,在此 apply

      - name: Run fail-closed verification
        run: |
          pwsh -NoProfile -File scripts/verify_fail_closed.ps1 -KubeCtlPath 'sudo k3s kubectl'
        # 关键:exit 1 自动阻断 PR 合并(GitHub Actions 默认行为)

      - name: Upload report on failure
        if: failure()
        uses: actions/upload-artifact@v4
        with:
          name: fail-closed-verify-report
          path: docs/deploy/fail-closed-verify-report.md
```

### 3.2 本地 / 自托管 runner

```bash
# 1. 启动 k3s 集群
sudo k3s server &
sudo k3s kubectl wait --for=condition=Ready nodes --all --timeout=60s

# 2. apply 受 PR 影响的 manifest
sudo k3s kubectl apply -f docs/deploy/01-k8s-manifests/

# 3. 跑验证(exit 1 = 失败)
pwsh -NoProfile -File scripts/verify_fail_closed.ps1 -KubeCtlPath 'sudo k3s kubectl'
```

### 3.3 烟雾预检（PR review 阶段，无 K8s）

```bash
# 仅做 k3s 可达性 + namespace 预检(< 5s,适合 draft PR 早期)
pwsh -NoProfile -File scripts/verify_fail_closed.ps1 -Mode Smoke
```

## 4. 失败处理 SOP

当 `verify_fail_closed.ps1` 退出 1（**任何一项 FAIL**）时，按以下流程处理：

### 4.1 CI 拦截（自动）

- GitHub Actions：job 标红 + 阻断 PR merge（`required status check` 设为必须 pass）
- 自托管 runner：返回非零退出码，CI pipeline 终止

### 4.2 报告查看

- 报告路径：`docs/deploy/fail-closed-verify-report.md`
- 报告含：Mermaid 流程图 + 4 项结果表 + 不变量校验表 + 复现命令

### 4.3 排查路径（per 测试 ID）

| 失败 TestId | 可能原因 | 排查方向 |
|---|---|---|
| **T1**（TLS fail-closed） | CA Secret 损坏后 5 域 Pod 未 fail-closed → **降级风险** | 检查 `crates/shared-platform/src/tls.rs` 的 mTLS load；检查 `RGS_ALLOW_INSECURE_GRPC` 是否被错误设置 |
| **T2**（RBAC fail-closed） | 未授权 SA 居然能访问 5 域 Secret | 检查 `10-rbac-template.yaml` 的 Role.rules；确认 `resourceNames` 限制（不能 list all） |
| **T3**（Secret 缺失） | 删 rgs-secret-ca 后 Pod 仍能启动（不依赖 CA） | 检查 Deployment 是否挂载 `rgs-secret-ca`；检查 `load_server_tls_config` 是否在 cert 缺失时仍走 insecure |
| **T4**（默认拒绝） | 删 RoleBinding 后 SA 仍能访问 | 检查 k3s API server 配置；可能启用 Node / ClusterRole 权限未受控（违反 RBAC 隔离） |

### 4.4 PR review checklist（人工）

- [ ] 阅读 `fail-closed-verify-report.md` 的失败 TestId
- [ ] 定位根因（manifest / Rust code / k3s 配置）
- [ ] 修复后 push → 重新触发 CI
- [ ] CI pass 后 PR 允许合并

## 5. PH-2 增强路径

当前（PH-1）的 fail-closed 验证是**手工 + CI 临时集群**。PH-2 计划引入：

### 5.1 cert-manager 自动轮转（PH-2）

- 当前 CA 轮转是手工（per RGS-INC-001 v0.2 §1.4 + Q-M-05 答复）
- PH-2 引入 cert-manager 后，CA 证书自动 90 天轮转 → T1（TLS fail-closed）需适配：
  - **必须验证轮转过程中 5 域服务不中断**（in-flight mTLS 仍可用）
  - **必须验证轮转后旧 CA 仍能校验未过期 leaf 证书**（grace period）

### 5.2 持续集群验证（PH-2 起）

- 当前 PR 触发 → 临时 k3s cluster → 跑验证 → 销毁
- PH-2 计划在 staging cluster 上**持续跑**（每 6 小时）→ 早于 PR 触发发现问题
- 接入 Prometheus：`fail_closed_verify_total{result="fail"}` 指标告警

### 5.3 chaos test 增强（PH-2）

- 当前 T3（Secret 缺失）仅测"删 rgs-secret-ca"场景
- PH-2 计划扩展：随机删任意 mTLS Secret / 注入 corruption / 模拟 clock skew
- 引入 chaos-mesh 或 litmus 做更全面故障注入

## 6. 退出码约定

| 退出码 | 含义 | CI 行为 |
|---|---|---|
| 0 | 全部 PASS（含 SKIP） | 允许 PR 合并 |
| 1 | 任何一项 FAIL | 阻断 PR 合并 |
| 2 | 前置检查失败（k3s 不可达 / namespace 缺失） | 阻断 PR 合并（CI 基础设施问题） |

## 7. 相关文档

- `scripts/verify_fail_closed.ps1`（脚本本体）
- `docs/deploy/phase-0-5-step-4-validate-fail-closed.ps1`（PH-0.5 一次性验证，per commit 765930a）
- `RGS-INC-001 v0.2 §1.4`（mTLS fail-closed 规范）
- `RGS-OPEN-QA-001 v0.2 Q-M-08 + ACTIONS-v0.3 B-09`（任务来源）

## 8. 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-24 | Worker (WF-1-55.48) | 初版。CI 接入 SOP + 失败处理 + PH-2 增强路径 |
