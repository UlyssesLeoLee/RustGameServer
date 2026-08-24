# Probe 段一致性 CI 接入说明

> **任务**：WF-1-55.46（per RGS-OPEN-QA-001 v0.2 Q-M-04 + ACTIONS-v0.3 §3 B-05）
> **脚本**：`scripts/verify_probe_consistency.ps1`
> **报告**：`docs/deploy/probe-consistency-report.md`

## 1. CI Workflow 调用方式

### 1.1 PowerShell 直接调用（推荐）

```yaml
# .github/workflows/probe-consistency.yml
name: probe-consistency
on:
  pull_request:
    paths:
      - 'docs/deploy/01-k8s-manifests/0?-*-service.yaml'
  push:
    branches: [main]
    paths:
      - 'docs/deploy/01-k8s-manifests/0?-*-service.yaml'
jobs:
  verify:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install PowerShell 7
        uses: microsoft/powershell@v1
      - name: Run verify_probe_consistency
        shell: pwsh
        run: |
          pwsh -NoProfile -File scripts/verify_probe_consistency.ps1
```

### 1.2 本地开发循环调用

```powershell
# 任意 manifest 改动后,本机预检
pwsh -NoProfile -File scripts\verify_probe_consistency.ps1

# exit 0 = 一致,可继续
# exit 1 = 不一致,先修 manifest 再 commit
Write-Host "exit code: $LASTEXITCODE"
```

### 1.3 pre-commit hook（可选）

```yaml
# .pre-commit-config.yaml
repos:
  - repo: local
    hooks:
      - id: probe-consistency
        name: Verify Kubernetes probe consistency
        entry: pwsh -NoProfile -File scripts/verify_probe_consistency.ps1
        language: system
        files: 'docs/deploy/01-k8s-manifests/0[0-9]-.*-service\.yaml$'
        pass_filenames: false
```

## 2. 触发条件

| 触发场景 | 检查 | 说明 |
|---|---|---|
| 任何 PR 修改 `0?-*-service.yaml` | ✅ 必跑 | 防止 probe 段漂移 |
| `main` 分支 push 包含 manifest 改动 | ✅ 必跑 | 防止基线漂移 |
| 仅修改 `07-shared-platform.yaml` / PG / NATS / OTel 等 | ⏭️ 跳过 | 与域 probe 段无关 |
| 本地手动跑 | ✅ 随时 | 调试用 |

## 3. 失败处理 SOP

### 3.1 CI 失败（exit 1）

1. **下载 artifacts**：从 CI 拉取 `probe-consistency-report.md` 看具体 diff
2. **判断 diff 类型**：
   - **阈值差异**（initialDelaySeconds / periodSeconds / timeoutSeconds / failureThreshold）：
     - 如果是有意为之（如 match 实时业务频次更密、economy 事务更重启动更慢），**手动更新报告的"结论"段**，说明差异是有意设计，并 PR 注明
     - 如果是无意 drift，**修改 manifest 至统一基线**（建议参考 `01-player-service.yaml`），重跑脚本至 exit 0
   - **命令结构差异**（grpc_health_probe 参数）：**必须**修一致，不允许 drift
   - **字段集差异**（4 个阈值字段缺失）：**必须**修，不允许漂移
3. **重新跑脚本验证**：确认 exit 0
4. **PR 描述**里附上脚本运行日志，注明哪个 manifest 改了、为什么改

### 3.2 阻止 merge

- 脚本 `exit 1` → PR 检查 status check 标红 → 阻止 merge（per `gh pr merge --require-status-checks`）
- 任何 probe 段改动必须经 reviewer 显式确认 + 脚本 exit 0

### 3.3 Ulysses 终审场景

当发现有意差异（如 match 实时频次、economy 慢启动）：
1. Ulysses 在 PR 评论里 **明示同意** 该差异
2. 在报告"结论"段 **手写一段说明**（per diff 编号）："本差异为有意设计，per RGS-INC-002 §4 match 实时业务 5s readiness 频次"
3. CI 仍保持 fail 状态（脚本不知道"故意 vs 漂移"），但 PR 走人工 override 流程

## 4. 长期目标：PH-2 引入 Helm

per Q-M-04 父疑问答复：

> **PH-1 暂不引入 Helm**，本 CI 脚本作为过渡方案。
> **PH-2（待 5 域全上线 + SRE 介入）后**：
> - 把 6 份 manifest 拆为 Helm chart 的 6 个 values 文件
> - probe 段收敛为 chart template `_helpers.tpl` 里的 1 份定义
> - 6 份 Deployment 由 `helm template` 派生，自动保证一致
> - 本脚本降级为辅助工具（验证 chart 渲染后的输出与基线一致）

迁移路径：
1. 写 chart（per RGS-EXEC-001 §4 Phase 2）
2. 用 `helm template` 渲染 6 份 Deployment，与当前 yaml 做 diff
3. diff 为 0 → 删除手写 manifest，仅保留 chart
4. 本脚本修改为：对 chart 渲染输出做一致性核对

## 5. 关联文档

- RGS-OPEN-QA-001 v0.2 §Q-M-04（父疑问）
- RGS-OPEN-QA-001-ACTIONS-v0.3 §3 B-05（跟踪表条目）
- RGS-WBS-001 v0.3 / v0.4（WF-1-55.46 任务编号）
- RGS-WT-001 §11.5（PowerShell 7.0+ 兼容）
- `scripts/verify_probe_consistency.ps1`（脚本本体）
- `docs/deploy/probe-consistency-report.md`（最新报告）

## 6. 已知限制

| 限制 | 影响 | 缓解 |
|---|---|---|
| 纯文本 YAML 解析 | 不支持 anchor / alias / merge keys | 当前 6 份 manifest 都不使用,无影响 |
| 单 Deployment 假设 | 同文件多 Deployment 会取第一个 | 当前 6 份 manifest 都只 1 个 Deployment |
| 中文 Windows 终端 CP936 | 脚本输出乱码（功能不受影响） | 推荐 GitHub Actions ubuntu-latest runner |
| 不修改 manifest | 脚本只读,需要人手改 | 这是有意设计（per RGS-DEC-008 Ulysses 终审权） |
