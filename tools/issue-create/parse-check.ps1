$script = 'D:\RustGameServer\tools\issue-create\create-2026-09-07-batch.ps1'
$tokens = $null
$err = $null
[System.Management.Automation.Language.Parser]::ParseFile($script, [ref]$tokens, [ref]$err) | Out-Null
if ($err -and $err.Count -gt 0) {
    Write-Host "PowerShell parse FAILED:"
    $err | ForEach-Object { Write-Host $_ }
    exit 1
}
Write-Host "PowerShell parse OK"
Write-Host "Tokens count: $($tokens.Count)"
Write-Host "File size: $((Get-Item $script).Length) bytes"
