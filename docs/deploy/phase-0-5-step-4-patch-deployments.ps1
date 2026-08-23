<#
.SYNOPSIS
    Phase 0.5 Step 4 —— 生成 6 域 deployment yaml 的 mTLS volumes/env 增量 patch(不修改原 yaml)

.DESCRIPTION
    背景:
      - 5 业务域 deployment yaml(01-05-*.yaml)在 WF-0-5-1 worktree 持有
      - cluster-ops deployment yaml(06-*.yaml)同样在 WF-0-5-1
      - **本 worker 严禁跨 worktree 改文件**(per 任务硬约束)
      - 主对话 WF-0.5-2/0.5-3 合入时,需把这些 patch 片段人工或脚本合并到 5+1 个 deployment yaml

    行为:
      - 输出 6 个 patch 片段(纯文本 yaml 段),写入 E:\DevCache\cargo\target\deployment-patches\
      - 输出 1 个 _merge_guide.md(主对话合并时遵循的步骤)
      - patch 段涵盖:
        1. env 增量(RGS_TLS_DIR + RGS_ALLOW_INSECURE_GRPC 显式 "0" 锁死 fail-closed)
        2. volumes 增量(2 个 secret:rgs-secret-<domain>-tls + rgs-secret-ca)
        3. volumeMounts 增量(2 个 mount:items 重映射 tls.crt→server.pem / tls.key→server.key + ca.pem)

    文件映射(容器内):
      /etc/rgs/certs/server.pem  ← secret rgs-secret-<domain>-tls 的 tls.crt (用 items key=server.pem)
      /etc/rgs/certs/server.key  ← secret rgs-secret-<domain>-tls 的 tls.key (用 items key=server.key)
      /etc/rgs/certs/ca.pem      ← secret rgs-secret-ca 的 ca.pem (直接用 key,subPath 挂载单文件)

    业务 binary 启动读 RGS_TLS_DIR=/etc/rgs/certs + load_server_tls_config(server.pem, server.key, ca.pem)
    缺失任一 → anyhow Context 上抛 → main 返 Err → 进程退 1 (fail-closed)

.PARAMETER OutputDir
    patch 片段输出目录。默认 E:\DevCache\cargo\target\deployment-patches\

.EXAMPLE
    pwsh -File phase-0-5-step-4-patch-deployments.ps1
    # 默认:输出 6 个 patch 片段 + 1 个 merge guide

.NOTES
    Author:  Worker (Phase 0.5 Step 4 deployment)
    Spec:    RGS-INC-001 v0.2 §1.4 (mTLS fail-closed) + 55.21 wire-up
    Pre:     无
    Post:    6 域 patch 片段 + 1 merge guide 在 E:\DevCache\cargo\target\deployment-patches\
#>
[CmdletBinding()]
param(
    [string]$OutputDir = 'E:\DevCache\cargo\target\deployment-patches'
)

$ErrorActionPreference = 'Stop'
$ProgressPreference    = 'SilentlyContinue'

# 1. 准备输出目录(幂等)
if (Test-Path $OutputDir) {
    Get-ChildItem -Path $OutputDir -ErrorAction SilentlyContinue | Remove-Item -Recurse -Force
}
New-Item -Path $OutputDir -ItemType Directory -Force | Out-Null
Write-Host "[INFO] Patch 片段输出目录: $OutputDir" -ForegroundColor Cyan

# 2. 6 域定义(5 业务域 + cluster-ops)
$domains = @(
    @{ Name = 'player';      SecretTls = 'rgs-secret-player-tls';      SourceYaml = '01-player-service.yaml' }
    @{ Name = 'economy';     SecretTls = 'rgs-secret-economy-tls';     SourceYaml = '02-economy-service.yaml' }
    @{ Name = 'match';       SecretTls = 'rgs-secret-match-tls';       SourceYaml = '03-match-service.yaml' }
    @{ Name = 'social';      SecretTls = 'rgs-secret-social-tls';      SourceYaml = '04-social-service.yaml' }
    @{ Name = 'admin';       SecretTls = 'rgs-secret-admin-tls';       SourceYaml = '05-admin-service.yaml' }
    @{ Name = 'cluster-ops'; SecretTls = 'rgs-secret-cluster-ops-tls'; SourceYaml = '06-cluster-ops-service.yaml' }
)

# 3. 6 域 patch 片段模板(纯文本 yaml 增量)
$patchTemplate = @"
# === Phase 0.5 Step 4 —— {DOMAIN} 域 mTLS patch 片段(per RGS-INC-001 §1.4) ===
# 合并目标:{SOURCE}
# 合并位置:
#   A. 追加到 spec.template.spec.containers[0].env 段尾
#   B. 追加到 spec.template.spec.volumes 段尾(若段不存在则新建)
#   C. 追加到 spec.template.spec.containers[0].volumeMounts 段尾(若段不存在则新建)

# A. env 增量
            - name: RGS_TLS_DIR
              value: /etc/rgs/certs
            # RGS_ALLOW_INSECURE_GRPC 显式锁死为 "0" 防 dev 镜像误传 "1" 静默降级(per RGS-REV-008 verify-C)
            - name: RGS_ALLOW_INSECURE_GRPC
              value: "0"

# B. volumes 增量
        - name: rgs-tls-server
          secret:
            secretName: {SECRET_TLS}
            defaultMode: 0600
        - name: rgs-tls-ca
          secret:
            secretName: rgs-secret-ca
            defaultMode: 0600

# C. volumeMounts 增量
          - name: rgs-tls-server
            mountPath: /etc/rgs/certs
            readOnly: true
            # items 重映射:kubernetes.io/tls Secret 默认 key=tls.crt/tls.key,业务 binary 读 server.pem/server.key
            items:
              - key: tls.crt
                path: server.pem
              - key: tls.key
                path: server.key
          - name: rgs-tls-ca
            # ca Secret 是 Opaque,key=ca.pem;subPath 单文件挂载避免覆盖 server.pem/server.key
            mountPath: /etc/rgs/certs/ca.pem
            subPath: ca.pem
            readOnly: true
"@

foreach ($d in $domains) {
    $patch = $patchTemplate.Replace('{DOMAIN}',     $d.Name)
    $patch = $patch.Replace('{SOURCE}',     $d.SourceYaml)
    $patch = $patch.Replace('{SECRET_TLS}', $d.SecretTls)
    $outFile = Join-Path $OutputDir ("patch-{0}.yaml" -f $d.Name)
    [System.IO.File]::WriteAllText($outFile, $patch, [System.Text.Encoding]::UTF8)
    Write-Host "[OK] 生成 $outFile ($((Get-Item $outFile).Length) bytes)"
}

# 4. 合并指南(主对话人工 + yq 流程)
# 单引号 here-string @'...'@ 不展开变量、不 escape 反引号,跨 PS 5.1/7 安全
$mergeGuide = @'
# Phase 0.5 Step 4 —— 6 域 Deployment mTLS Patch 合并指南

## 何时合并

- 由主对话在 WF-0.5-1(worktree 写 5 业务域 deployment)合入 main 后执行
- 或主对话在 WF-0.5-1..WF-0.5-3 三 worktree 整合阶段手工合入

## 合并流程(以 player 域为例)

### Step 1:确认 5+1 域 deployment yaml 在 WF-0-5-1 worktree 已就位

```
ls D:\RustGameServer-worktrees\WF-0-5-1\docs\deploy\01-k8s-manifests\{01..05,06}-*.yaml
```

### Step 2:对每域应用 patch 片段

**方案 A:手工合并(5 域 + 1 cluster-ops,推荐)**

打开 patch-{domain}.yaml,按注释"合并位置"提示:
- A 段(env 增量)→ 追加到 spec.template.spec.containers[0].env 列表尾
- B 段(volumes 增量)→ 追加到 spec.template.spec.volumes 列表尾
- C 段(volumeMounts 增量)→ 追加到 spec.template.spec.containers[0].volumeMounts 列表尾(若段不存在,在 containers[0] 下新建)

**方案 B:yq 脚本合并(需要 yq v4+)**

```bash
# 例:player 域
yq eval-all '
  select(fileIndex == 0).spec.template.spec.containers[0].env +=
    [{"name":"RGS_TLS_DIR","value":"/etc/rgs/certs"},
     {"name":"RGS_ALLOW_INSECURE_GRPC","value":"0"}] |
  select(fileIndex == 0).spec.template.spec.volumes +=
    [{"name":"rgs-tls-server","secret":{"secretName":"rgs-secret-player-tls","defaultMode":384}},
     {"name":"rgs-tls-ca","secret":{"secretName":"rgs-secret-ca","defaultMode":384}}] |
  select(fileIndex == 0).spec.template.spec.containers[0].volumeMounts +=
    [{"name":"rgs-tls-server","mountPath":"/etc/rgs/certs","readOnly":true,
      "items":[{"key":"tls.crt","path":"server.pem"},{"key":"tls.key","path":"server.key"}]},
     {"name":"rgs-tls-ca","mountPath":"/etc/rgs/certs/ca.pem","subPath":"ca.pem","readOnly":true}]
' \
  D:\RustGameServer-worktrees\WF-0-5-1\docs\deploy\01-k8s-manifests\01-player-service.yaml \
  E:\DevCache\cargo\target\deployment-patches\patch-player.yaml
```

### Step 3:验证合并结果

```bash
# 每域 deployment 应包含:
grep -A1 "RGS_TLS_DIR" D:\RustGameServer-worktrees\WF-0-5-1\docs\deploy\01-k8s-manifests\0X-*.yaml
grep "rgs-secret-" D:\RustGameServer-worktrees\WF-0-5-1\docs\deploy\01-k8s-manifests\0X-*.yaml
```

## 合并后产物(per domain)

合并完成后,每域 deployment 模板应含:
- env: RGS_TLS_DIR=/etc/rgs/certs + RGS_ALLOW_INSECURE_GRPC=0
- volumes: rgs-tls-server(secretName=rgs-secret-<domain>-tls)+ rgs-tls-ca(secretName=rgs-secret-ca)
- volumeMounts: rgs-tls-server(items 映射 tls.crt→server.pem, tls.key→server.key)+ rgs-tls-ca(subPath=ca.pem)

## 验证

合并后跑 phase-0-5-step-4-validate-fail-closed.ps1 验证 fail-closed 逻辑,
或在 K3s cluster apply 后跑:
```bash
kubectl -n rgs exec deploy/player-service -- ls -la /etc/rgs/certs/
# 期望:server.pem server.key ca.pem
```
'@

$guidePath = Join-Path $OutputDir '_merge_guide.md'
[System.IO.File]::WriteAllText($guidePath, $mergeGuide, [System.Text.Encoding]::UTF8)
Write-Host "[OK] 生成 $guidePath ($((Get-Item $guidePath).Length) bytes)"

Write-Host "`n[OK] Patch 片段生成完成" -ForegroundColor Green
Write-Host "     6 域 patch + 1 merge guide 在 $OutputDir"
Write-Host ""
Write-Host "[CONSTRAINT]" -ForegroundColor Magenta
Write-Host "  - 本 worker 不修改 WF-0-5-1 持有的 deployment yaml"
Write-Host "  - 主对话在 WF-0.5-2/0.5-3 合入时按 _merge_guide.md 合并"
Write-Host "  - 合并后建议跑 phase-0-5-step-4-validate-fail-closed.ps1 二次验证"
