# 04-ci-cd — GitHub Actions CI/CD 占位

> **状态：🔴 NO-GO 占位**（per `RGS-PLAN-001 v0.8 §3.3` + `RGS-ENV-001 v0.3 §6` + G-CODE-06）
>
> 本目录所有 `.yaml` workflow 在 **53 開発環境構築 启动条件**全部满足前**不得激活为可执行 workflow**。
>
> 当前文件全部为**结构骨架**（trigger 关闭 / steps 占位），仅用于提前铺好 CI/CD 管道。**禁止在 NO-GO 解除前向本目录提交实际 runner 配置、镜像 push 凭证、deploy key。**

---

## 1. 目录组织

| 序号 | 文件 | 角色 | 状态 |
|---|---|---|---|
| `rust-ci.yaml` | Rust 编译 + 测试 + clippy + fmt | CI | 占位（trigger 仅 main，禁 push） |
| `docs-ci.yaml` | 文档变更预览 / 链接检查 | CI | 占位 |
| `verify-docs-ci.yaml` | `verify_docs.py` + `check-cross-references.py` + `verify_wf_v05.py` 三脚本必跑 | CI | 占位 |
| `docker-build.yaml` | 5 域 + cluster-ops + shared-platform 镜像构建 | CI | 占位（无 push 凭证） |

---

## 2. 触发策略（占位）

| Workflow | 触发 | 行为 |
|---|---|---|
| `rust-ci.yaml` | `pull_request` (任意分支) | 仅验证，禁 push 镜像 |
| `docs-ci.yaml` | `pull_request` (docs/**) | 文档链接检查 |
| `verify-docs-ci.yaml` | `push` (main) + `pull_request` | 3 脚本必跑 |
| `docker-build.yaml` | `push` (main, tag=v*) — **未激活** | 镜像构建 + 推送（NO-GO 解除后由 Platform 架构师激活） |

> `docker-build.yaml` 的 `push` trigger 在 NO-GO 状态下保持注释状态。
> 实际激活条件：G-CODE-06 Closed（Rust 1.98 + Cargo.lock + CI 全绿）+ Platform 架构师具名签字。

---

## 3. Runner 与依赖（占位）

| 资源 | 实际值 | 状态 |
|---|---|---|
| Runner | `ubuntu-22.04`（GitHub-hosted） | 待 Platform 架构师确认 |
| Rust toolchain | `1.98.0` | **G-CODE-06 部分满足**（GA 已发，待 CI 验证） |
| Cargo 缓存 | `Swatinem/rust-cache@v2` | 占位 |
| PG 版本（CI 测试） | `postgres:18.6` | 占位（per RGS-TS-001 v0.6 §5.2） |
| 容器 registry | `PLACEHOLDER_REGISTRY` | 占位 |
| Deploy key | `PLACEHOLDER_DEPLOY_KEY` | 占位（NO-GO 解除前不得提交实际 key） |

---

## 4. 与 3 验证脚本的集成

`verify-docs-ci.yaml` 必跑 3 脚本（per handoff §5 + RGS-WF-001 v0.5）：

1. `python scripts/verify_docs.py` — doc ID 唯一性 + FILENAME 规范
2. `python scripts/check-cross-references.py` — 跨文档引用闭环
3. `python scripts/verify_wf_v05.py` — 工作流 v0.5 一致性

任一脚本非零退出 → CI 失败 → 不得合并。

---

## 5. NO-GO 解除条件

本目录从占位升级为可执行 workflow，必须满足：

1. **7 G-CODE 全部 Closed**，特别：
   - G-CODE-05 Platform 架构师具名 + CI/CD 签字
   - G-CODE-06 Rust 1.98 + Cargo.lock + CI 全绿
2. **RGS-ENV-001 v0.3 §6 12 类签字栏全部具名签字**（当前 2/12 实际签 + 10/12 所有者背书占位）
3. **3 验证脚本通过**（已在 eb2fffa 验证过，状态保留）
4. **Runner + registry 实际配置**（Platform 架构师签字）

满足后由架构师出 v0.8 删除"所有者背书"占位 → 本目录 `_status.md` 升 `🟢 GO` → 由 Platform 架构师主导激活 `docker-build.yaml` 实际 push trigger。

---

## 6. 关联文档

- 上游：`RGS-EXEC-001 v0.3`（G-CODE 突破手册）+ `RGS-TS-001 v0.6 §5.2`（PG 选型）
- 并行：`01-k8s-manifests/` + `02-helm-charts/` + `03-db-migrations/`
- 治理：`RGS-PLAN-001 v0.8 §3.3` + `RGS-ENV-001 v0.3 §6`
- 验证：`scripts/verify_docs.py` + `scripts/check-cross-references.py` + `scripts/verify_wf_v05.py`
- 自检表：`../07-no-go-checklist_v0.2.md` + `../00-prerequisites/00-no-go-checklist_v0.2.md`
