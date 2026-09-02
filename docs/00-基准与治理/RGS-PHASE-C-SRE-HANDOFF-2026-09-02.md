# Phase C SRE 介入 Handoff v0.1 (per STATUS-SNAPSHOT v0.6.36, 2026-09-02 11:30 JST)

> **创建日期**: 2026-09-02 11:30 JST
> **创建者**: 架构师(Mavis 接手 agent per DEC-008) 代签 Ulysses
> **状态**: 🟡 Handoff 就绪 (等 SRE 介入 + k3s ulyssespc 节点注册恢复)
> **关联**: STATUS-SNAPSHOT v0.6.36 §5.1 + WBS v0.4.10 §1 + DB-PARTITIONED-REVIEW-CHECKLIST v0.1.1 + RGS-VERIFIER-COMMANDS v0.1.1 + OPEN-QA v0.3 §7.1

## 0. Handoff 目标

把 Phase C 5 域 mTLS 业务级部署的**具体步骤**文档化,SRE 介入时直接照着做,不需要回查 OPEN-QA / WBS / STATUS-SNAPSHOT 多文件。**Mavis 边界 (per OPEN-QA v0.3 §7.5)**: 不应做卸载 k3s / 重 apply manifest / 修证书 / 改 yaml — 全部由 SRE 执行。

## 1. Phase C 阻塞项现状 (per STATUS-SNAPSHOT v0.6.36 §5.1)

| 项 | 当前状态 | 解锁条件 |
|---|---|---|
| k3s ulyssespc 节点注册 | 🔒 未恢复 (per OPEN-QA v0.3 §7.1) | SRE 物理介入 |
| 5 域 gRPC 业务级 mTLS | 🔒 0/5 (5 域 binary 起来 + mTLS 业务级 ST) | k3s 节点注册恢复后 |
| E3 W2-W6 3 项依赖外部 | 🔒 0/3 (W2 task_buffer / W3 E2E 真实 sqlx + 5 域 / W4-N 灰度锁 + W6-N 跨域事件) | 5 域 mTLS 部署完成 |
| 11 UT 实际跑 | 🔒 未跑 (L1 派生约束 cargo check 60s 限时) | 不依赖 Phase C, 可立即跑 |
| 11 E2E 实际跑 | 🔒 未跑 (依赖 rgs-web + DB) | 5 域 mTLS + DB 池接通 |
| 4 DRAFT partitioned SQL 评审 | 🟡 启动材料 v0.1.1 就绪, 5 域 Lead 已派工 | SRE 介入 → DBA 主审 → 3 域 Lead 业务验证 |

## 2. Phase C SRE 介入 Checklist

### 阶段 A: k3s 节点注册恢复 (per OPEN-QA v0.3 §7.1, 阻塞根源)

- [ ] **A.1** SSH 登录 WSL 节点
  ```bash
  # Mavis 边界外, 由 SRE 执行
  wsl -d Ubuntu-22.04
  ```
- [ ] **A.2** 检查 k3s 状态
  ```bash
  systemctl status k3s
  sudo kubectl get nodes
  ```
- [ ] **A.3** 恢复 ulyssespc 节点注册
  - 检查 `/etc/rancher/k3s/k3s.yaml` 配置
  - 检查 token: `sudo cat /var/lib/rancher/k3s/server/node-token`
  - 在 ulyssespc 节点重跑 agent 注册:
    ```bash
    # ulyssespc 节点上
    curl -sfL https://get.k3s.io | K3S_URL=https://<server-ip>:6443 K3S_TOKEN=<node-token> sh -
    ```
- [ ] **A.4** 验证节点状态
  ```bash
  sudo kubectl get nodes  # ulyssespc 应为 Ready
  ```
- [ ] **A.5** 检查 5 域 binary
  ```bash
  sudo kubectl get pods -A | grep -E 'player|economy|match|social|admin'
  ```
- [ ] **A.6** 验证 PostgreSQL 池
  ```bash
  sudo kubectl get pods -A | grep -E 'postgres|pg-pool'
  PGPASSWORD=rgs_admin psql -h <pg-host> -p 5544 -U rgs_admin -d rgs_main -c "SELECT 1"
  ```

### 阶段 B: 5 域 gRPC 业务级 mTLS 部署 (per Phase C 5/5 桶)

- [ ] **B.1** 5 域 cert 重新签发 (per WBS v0.4.10 + BA-W1-4 rgs-certgen)
  ```bash
  # Mavis 边界外, 由 SRE 执行
  cd /opt/rgs/certs
  ./rgs-certgen.sh --domain all --modes mtls-business  # 业务级 mTLS
  ```
- [ ] **B.2** 5 域 binary 重启 (应用新 cert)
  ```bash
  sudo kubectl rollout restart deployment player-service -n rgs
  sudo kubectl rollout restart deployment economy-service -n rgs
  sudo kubectl rollout restart deployment match-service -n rgs
  sudo kubectl rollout restart deployment social-service -n rgs
  sudo kubectl rollout restart deployment admin-service -n rgs
  ```
- [ ] **B.3** 验证 5 域 mTLS 业务级 ST
  ```bash
  # 跑 5 域 ST 业务级场景 (per BATCH-PLAN v0.2 §10)
  ./rgs-st-business-mtls.sh --domain all --verify-st-pass
  ```
- [ ] **B.4** 检查 Prometheus + Grafana
  ```bash
  curl http://prometheus:9090/api/v1/targets | jq '.data.activeTargets[] | select(.labels.job == "rgs-batch-backend")'
  curl http://grafana:3000/api/dashboards/rgs
  ```
- [ ] **B.5** 检查 rgs-web 8788
  ```bash
  curl http://rgs-web:8788/api/v1/health
  curl http://rgs-web:8788/api/v1/version
  ```
- [ ] **B.6** 检查 rgs-batch-backend 8789
  ```bash
  curl http://rgs-batch-backend:8789/api/v1/health
  curl http://rgs-batch-backend:8789/api/v1/version
  ```

### 阶段 C: cargo test 22 测试函数实际跑 (per TEST-RUN-PLAN v0.1)

- [ ] **C.1** 11 UT 单独跑 (Phase C 不依赖, 可立即跑)
  ```bash
  cd /opt/rgs/tools/rgs-batch-backend
  Start-Process cargo -ArgumentList @('test','--lib','exponential_backoff','endpoint_json_schema','--no-fail-fast') -RedirectStandardOutput 'cargo-test-ut-2026-09-02.log' -RedirectStandardError 'cargo-test-ut-2026-09-02.err' -PassThru
  # 60s 后看 log 0 error = 状态正确 (L1 派生约束)
  ```
  **预期结果**: 11/11 PASS
- [ ] **C.2** 11 E2E 完整跑 (Phase C 部署完成后)
  ```bash
  Start-Process cargo -ArgumentList @('test','--test','integration_tests','e2e_','--no-fail-fast') -RedirectStandardOutput 'cargo-test-e2e-2026-09-02.log' -RedirectStandardError 'cargo-test-e2e-2026-09-02.err' -PassThru
  ```
  **预期结果**: 11/11 PASS
- [ ] **C.3** 22 测试函数全跑
  ```bash
  Start-Process cargo -ArgumentList @('test','--tests','--no-fail-fast') -RedirectStandardOutput 'cargo-test-all-2026-09-02.log' -RedirectStandardError 'cargo-test-all-2026-09-02.err' -PassThru
  ```
  **预期结果**: 22/22 PASS
- [ ] **C.4** commit 模板 (per TEST-RUN-PLAN v0.1 §4)
  - 11 UT PASS: `test(batch-backend): UT 实际跑 11/11 PASS (per BA-W3-10 9/2 验证 cargo test --lib), 派生约束 L1 1 worker 1 crate`
  - 11 E2E PASS: `test(batch-backend): E2E 实际跑 11/11 PASS (per BA-W3-11 9/2 验证 cargo test --test integration_tests), 派生约束 L1 1 worker 1 crate + Phase C 5 域 mTLS 部署完成`
  - 22 全 PASS: `test(batch-backend): 22 测试函数全 PASS (per WBS v0.4.7 §1.1 3 项外部依赖全解锁), 派生约束 L1 + L11 + L14`

### 阶段 D: 4 DRAFT partitioned SQL 评审启动 (per DB-CHECKLIST v0.1.1 + SEQUENCE v0.1)

- [ ] **D.1** Phase 0: 架构师发出评审召集通知 (per SEQUENCE v0.1 §2 Phase 0)
- [ ] **D.2** Phase 1: SRE Lead 签字 (k3s ulyssespc Ready + 5 域 mTLS 部署完成, per A.4 + B.3)
- [ ] **D.3** Phase 2: DBA Lead 签字 (Schema + 保留期 + 索引 3 维度主审通过)
- [ ] **D.4** Phase 3a: admin Lead 签字 (PH-2 业务验证)
- [ ] **D.5** Phase 3b: economy Lead 签字 (PH-3 业务验证)
- [ ] **D.6** Phase 3c: match Lead 签字 (PH-3 业务验证)
- [ ] **D.7** Phase 3 双写期验证: 3 域 × SRE 联合签字
- [ ] **D.8** Phase 4: 架构师总审批 + DRAFT→v1.0 commit

## 3. 派生约束守护 (per L11 + L12 + L13)

- **L11** (build dir lock 防御): SRE 介入不涉及 cargo build, 隔离 target dir 不冲突
- **L12** (临时 log / .txt 不入 commit): cargo test log + err 文件放 L12 临时目录, 不入 commit
- **L13** (自指字段全 deferred 实时查询): 本文档阶段 A/B/C/D 都是流程步骤, 不带具体数字; 进度数字用实时 git 查询
- **L14** (plumbing brace 跟踪): SRE 介入不涉及 plumbing patch, 跟 Mavis 边界对齐
- **OPEN-QA v0.3 §7.5** (Mavis 边界): Mavis 不做卸载 k3s / 重 apply manifest / 修证书 / 改 yaml — 全部由 SRE 执行

## 4. 关联文档

- `RGS-STATUS-SNAPSHOT-2026-09-02.md` v0.6.36 (主跟踪快照, §5.1 Phase C 阻塞段)
- `RGS-PLAN-WBS-token-bucket-v0.4.md` v0.4.10 (WBS 跟踪表, E3 W2-W6 3 项外部依赖)
- `RGS-DB-PARTITIONED-DRAFT-REVIEW-CHECKLIST-2026-09-02.md` v0.1.1 (评审启动材料)
- `RGS-DB-PARTITIONED-DRAFT-REVIEW-SEQUENCE-2026-09-02.md` v0.1 (评审召集时序)
- `tools/rgs-batch-backend/TEST-RUN-PLAN-2026-09-02.md` v0.1 (22 测试函数运行计划)
- `RGS-VERIFIER-COMMANDS-2026-09-02.md` v0.1.1 (verifier 取数命令)
- `OPEN-QA-001 v0.3` (per 9/1 拍板 4 全 A, Mavis 边界 §7.5)
- `BATCH-PLAN v0.2` (5 域 ST 业务级场景)

**修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
**审批**: 架构师(Mavis 接手 agent per DEC-008)
**代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)

## 5. 修订历史

| 版本 | 日期 (JST) | 修订人 | 变更 |
|---|---|---|---|
| v0.1 | 2026-09-02 11:30 | 架构师(Mavis 接手 agent per DEC-008) | 初版: Phase C SRE 介入 Handoff 文档 (4 阶段: A k3s 节点 / B 5 域 mTLS / C 22 测试 / D 评审启动), 23 项 checklist 步骤, L11 + L12 + L13 + L14 + OPEN-QA §7.5 派生约束守护, 代签 per 8/27 19:39/20:56/21:59 JST 三次强化 |
