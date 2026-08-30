# trigger-build-prod.ps1 —— 新式 GHCR pipeline trigger
# 目的: 用 fine-grained PAT(只 Actions:write) 调 GitHub REST API 触发 workflow_dispatch
# 关联: .github/workflows/build-prod-0.1.0.yml
# 安全: 环境变量 $env:GHCR_PAT 禁止打印, 直接 invoke
# 2026-08-30 14:30 JST Ulysses 部署决策: fine-grained PAT → Actions workflow_dispatch → GITHUB_TOKEN

[CmdletBinding()]
param(
    [string]$Tag = "0.1.0",
    [string]$PushLatest = "true",
    [string]$WorkflowFile = "build-prod-0.1.0.yml"
)

$ErrorActionPreference = 'Stop'

# 校验: GHCR_PAT 必须存在但禁止打印内容
if (-not $env:GHCR_PAT) {
    Write-Host "[FAIL] \$env:GHCR_PAT 未设置" -ForegroundColor Red
    exit 1
}
if ($env:GHCR_PAT.Length -lt 20) {
    Write-Host "[FAIL] \$env:GHCR_PAT 长度异常(< 20 字符), 拒绝触发" -ForegroundColor Red
    exit 1
}

# 校验: 不能含 echo / Write-Host GHCR_PAT 内容(per 2026-08-27 hard ban)
# 这里仅输出长度和首 4 字符用于 sanity check, 不打印 secret
Write-Host "[INFO] GHCR_PAT length = $($env:GHCR_PAT.Length) chars, prefix = $($env:GHCR_PAT.Substring(0, [Math]::Min(4, $env:GHCR_PAT.Length)))..." -ForegroundColor Yellow

# 解析 owner/repo
$remoteUrl = git remote get-url origin
# 兼容 git@github.com:Owner/Repo.git 或 https://github.com/Owner/Repo.git
if ($remoteUrl -match 'github\.com[:/]([^/]+)/([^/]+?)(?:\.git)?$') {
    $owner = $Matches[1]
    $repo = $Matches[2]
} else {
    Write-Host "[FAIL] 无法解析 remote URL: $remoteUrl" -ForegroundColor Red
    exit 1
}
Write-Host "[INFO] Repo: $owner/$repo" -ForegroundColor Green
Write-Host "[INFO] Workflow: $WorkflowFile, tag: $Tag, push_latest: $PushLatest" -ForegroundColor Green

# workflow_dispatch API: POST /repos/{owner}/{repo}/actions/workflows/{workflow}/dispatches
$apiUrl = "https://api.github.com/repos/$owner/$repo/actions/workflows/$WorkflowFile/dispatches"

# body: ref(branch) + inputs(workflow_dispatch input)
$body = @{
    ref = "main"
    inputs = @{
        tag = $Tag
        push_latest = $PushLatest
    }
} | ConvertTo-Json -Compress

Write-Host "[INFO] POST $apiUrl" -ForegroundColor Cyan

# 直接 invoke $env:GHCR_PAT, 不 echo / Write-Host
$headers = @{
    "Authorization" = "Bearer $env:GHCR_PAT"
    "Accept" = "application/vnd.github+json"
    "X-GitHub-Api-Version" = "2022-11-28"
    "Content-Type" = "application/json"
    "User-Agent" = "rgs-trigger-build-prod"
}

try {
    $response = Invoke-RestMethod -Method Post -Uri $apiUrl -Headers $headers -Body $body -TimeoutSec 30
    # 204 No Content = 成功, 不打印 env 内容
    Write-Host "[OK] workflow_dispatch 触发成功" -ForegroundColor Green
    Write-Host "[INFO] 监控: https://github.com/$owner/$repo/actions/workflows/$WorkflowFile" -ForegroundColor Cyan
} catch {
    $statusCode = $_.Exception.Response.StatusCode.value__
    $errBody = ""
    try {
        $reader = New-Object System.IO.StreamReader($_.Exception.Response.GetResponseStream())
        $errBody = $reader.ReadToEnd()
    } catch {}
    Write-Host "[FAIL] HTTP $statusCode" -ForegroundColor Red
    if ($errBody) {
        Write-Host "[FAIL body] $errBody" -ForegroundColor Red
    }
    exit 1
}
