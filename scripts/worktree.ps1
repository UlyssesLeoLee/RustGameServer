# CYPHER-STRUCTURE-BEGIN
# CREATE
#   (file:File {name: "worktree.ps1", type: "file", language: "powershell"}),
#   (invokeGit:Function {name: "Invoke-Git", type: "function", signature: "Invoke-Git([string[]] Arguments)", visibility: "private"}),
#   (records:Function {name: "Get-WorktreeRecords", type: "function", signature: "Get-WorktreeRecords()", visibility: "private"}),
#   (primary:Function {name: "Get-PrimaryRepositoryRoot", type: "function", signature: "Get-PrimaryRepositoryRoot()", visibility: "private"}),
#   (within:Function {name: "Test-PathIsWithin", type: "function", signature: "Test-PathIsWithin([string] ChildPath, [string] ParentPath)", visibility: "private"}),
#   (taskName:Function {name: "Assert-TaskName", type: "function", signature: "Assert-TaskName([string] TaskName)", visibility: "private"}),
#   (externalRoot:Function {name: "Assert-NotNestedInGitWorktree", type: "function", signature: "Assert-NotNestedInGitWorktree([string] CandidateRoot, [string] PrimaryRoot)", visibility: "private"}),
#   (resolveRoot:Function {name: "Resolve-WorktreeRoot", type: "function", signature: "Resolve-WorktreeRoot([string] PrimaryRoot, [string] RequestedRoot)", visibility: "private"}),
#   (managedPath:Function {name: "Assert-ManagedTaskPath", type: "function", signature: "Assert-ManagedTaskPath([string] TargetPath, [string] WorktreeRoot, [string] PrimaryRoot)", visibility: "private"}),
#   (config:Function {name: "Get-WorktreeConfig", type: "function", signature: "Get-WorktreeConfig([string] Path, [string] Key)", visibility: "private"}),
#   (managed:Function {name: "Get-ManagedRecords", type: "function", signature: "Get-ManagedRecords([string] WorktreeRoot)", visibility: "private"}),
#   (port:Function {name: "Get-AvailablePortBlock", type: "function", signature: "Get-AvailablePortBlock([string] WorktreeRoot, [int] RequestedPortBlock)", visibility: "private"}),
#   (writeEnv:Function {name: "Write-WorktreeEnv", type: "function", signature: "Write-WorktreeEnv([string] TargetPath, [string] TaskName, [int] PortBlock)", visibility: "private"}),
#   (create:Function {name: "New-TaskWorktree", type: "function", signature: "New-TaskWorktree([string] PrimaryRoot, [string] WorktreeRoot, [string] TaskName, [string] BaseRef, [int] RequestedPortBlock)", visibility: "private"}),
#   (list:Function {name: "Show-TaskWorktrees", type: "function", signature: "Show-TaskWorktrees([string] WorktreeRoot)", visibility: "private"}),
#   (doctor:Function {name: "Test-TaskWorktrees", type: "function", signature: "Test-TaskWorktrees([string] PrimaryRoot, [string] WorktreeRoot)", visibility: "private"}),
#   (remove:Function {name: "Remove-TaskWorktree", type: "function", signature: "Remove-TaskWorktree([string] PrimaryRoot, [string] WorktreeRoot, [string] TaskName)", visibility: "private"}),
#   (main:Logic {name: "command-dispatch", type: "logic"}),
#   (command:Variable {name: "Action", type: "variable"}),
#   (name:Variable {name: "Name", type: "variable"}),
#   (base:Variable {name: "Base", type: "variable"}),
#   (requestedRoot:Variable {name: "WorktreeRoot", type: "variable"}),
#   (requestedPort:Variable {name: "PortBlock", type: "variable"}),
#   (primaryRoot:Variable {name: "primaryRoot", type: "variable"}),
#   (worktreeRoot:Variable {name: "worktreeRoot", type: "variable"}),
#   (targetPath:Variable {name: "targetPath", type: "variable"}),
#   (branchName:Variable {name: "branchName", type: "variable"}),
#   (file)-[:CONTAINS]->(invokeGit), (file)-[:CONTAINS]->(records), (file)-[:CONTAINS]->(primary),
#   (file)-[:CONTAINS]->(within), (file)-[:CONTAINS]->(taskName), (file)-[:CONTAINS]->(externalRoot), (file)-[:CONTAINS]->(resolveRoot),
#   (file)-[:CONTAINS]->(managedPath), (file)-[:CONTAINS]->(config), (file)-[:CONTAINS]->(managed),
#   (file)-[:CONTAINS]->(port), (file)-[:CONTAINS]->(writeEnv), (file)-[:CONTAINS]->(create),
#   (file)-[:CONTAINS]->(list), (file)-[:CONTAINS]->(doctor), (file)-[:CONTAINS]->(remove), (file)-[:CONTAINS]->(main),
#   (records)-[:CALLS]->(invokeGit), (primary)-[:CALLS]->(records),
#   (resolveRoot)-[:CALLS]->(within), (resolveRoot)-[:CALLS]->(externalRoot), (managedPath)-[:CALLS]->(within), (managed)-[:CALLS]->(records),
#   (managed)-[:CALLS]->(within), (port)-[:CALLS]->(managed), (port)-[:CALLS]->(config),
#   (create)-[:CALLS]->(taskName), (create)-[:CALLS]->(managedPath), (create)-[:CALLS]->(invokeGit),
#   (create)-[:CALLS]->(records), (create)-[:CALLS]->(port), (create)-[:CALLS]->(writeEnv),
#   (list)-[:CALLS]->(managed), (list)-[:CALLS]->(config),
#   (doctor)-[:CALLS]->(managed), (doctor)-[:CALLS]->(config),
#   (remove)-[:CALLS]->(taskName), (remove)-[:CALLS]->(managedPath), (remove)-[:CALLS]->(records),
#   (remove)-[:CALLS]->(config), (remove)-[:CALLS]->(invokeGit),
#   (main)-[:CALLS]->(primary), (main)-[:CALLS]->(invokeGit), (main)-[:CALLS]->(resolveRoot), (main)-[:CALLS]->(create),
#   (main)-[:CALLS]->(list), (main)-[:CALLS]->(doctor), (main)-[:CALLS]->(remove),
#   (main)-[:USES]->(command), (main)-[:USES]->(name), (main)-[:USES]->(base),
#   (main)-[:USES]->(requestedRoot), (main)-[:USES]->(requestedPort), (main)-[:USES]->(primaryRoot),
#   (main)-[:USES]->(worktreeRoot), (create)-[:USES]->(targetPath), (create)-[:USES]->(branchName),
#   (remove)-[:USES]->(targetPath), (remove)-[:USES]->(branchName);
# CYPHER-STRUCTURE-END

param(
    [ValidateSet('create', 'list', 'doctor', 'remove')]
    [string]$Action = 'list',

    [string]$Name,

    [string]$Base = 'main',

    [string]$WorktreeRoot,

    [ValidateRange(0, 99)]
    [int]$PortBlock = 0
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Invoke-Git {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)]
        [string[]]$Arguments
    )

    $output = & git @Arguments 2>&1
    if ($LASTEXITCODE -ne 0) {
        $details = ($output | ForEach-Object { $_.ToString() }) -join [Environment]::NewLine
        throw "git $($Arguments -join ' ') failed with exit code $LASTEXITCODE. $details"
    }

    return $output
}

function Get-WorktreeRecords {
    [CmdletBinding()]
    param()

    $lines = @(Invoke-Git -Arguments @('worktree', 'list', '--porcelain') | ForEach-Object { $_.ToString() })
    $lines += ''
    $records = @()
    $current = @{}

    foreach ($line in $lines) {
        if ([string]::IsNullOrWhiteSpace($line)) {
            if ($current.Count -gt 0) {
                $records += [PSCustomObject]@{
                    Path   = $current['worktree']
                    Branch = $current['branch']
                    Locked = $current.ContainsKey('locked')
                }
                $current = @{}
            }
            continue
        }

        if ($line.StartsWith('worktree ')) {
            $current['worktree'] = $line.Substring(9)
        }
        elseif ($line.StartsWith('branch ')) {
            $current['branch'] = $line.Substring(7)
        }
        elseif ($line.StartsWith('locked')) {
            $current['locked'] = $true
        }
    }

    return $records
}

function Get-PrimaryRepositoryRoot {
    [CmdletBinding()]
    param()

    $records = @(Get-WorktreeRecords)
    if ($records.Count -eq 0 -or [string]::IsNullOrWhiteSpace($records[0].Path)) {
        throw 'Unable to determine the primary Git worktree.'
    }

    return [System.IO.Path]::GetFullPath($records[0].Path)
}

function Test-PathIsWithin {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)]
        [string]$ChildPath,

        [Parameter(Mandatory = $true)]
        [string]$ParentPath
    )

    $child = [System.IO.Path]::GetFullPath($ChildPath).TrimEnd([char[]]@('\', '/'))
    $parent = [System.IO.Path]::GetFullPath($ParentPath).TrimEnd([char[]]@('\', '/'))

    if ([string]::Equals($child, $parent, [System.StringComparison]::OrdinalIgnoreCase)) {
        return $true
    }

    return $child.StartsWith(
        $parent + [System.IO.Path]::DirectorySeparatorChar,
        [System.StringComparison]::OrdinalIgnoreCase
    )
}

function Assert-TaskName {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)]
        [string]$TaskName
    )

    if ($TaskName -notmatch '^[a-z0-9][a-z0-9-]{1,47}$') {
        throw 'Name must be 2-48 lowercase ASCII letters, digits, or hyphens, beginning with a letter or digit.'
    }
}

function Assert-NotNestedInGitWorktree {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)]
        [string]$CandidateRoot,

        [Parameter(Mandatory = $true)]
        [string]$PrimaryRoot
    )

    $probePath = [System.IO.Path]::GetFullPath($CandidateRoot)
    while (-not (Test-Path -LiteralPath $probePath)) {
        $parentInfo = [System.IO.Directory]::GetParent($probePath)
        if ($null -eq $parentInfo) {
            return
        }
        $probePath = $parentInfo.FullName
    }

    $previousErrorActionPreference = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'
    try {
        $insideGitWorktree = & git -C $probePath rev-parse --is-inside-work-tree 2>$null
        $insideGitWorktreeExitCode = $LASTEXITCODE
        if ($insideGitWorktreeExitCode -ne 0 -or (@($insideGitWorktree)[0]).ToString().Trim() -ne 'true') {
            return
        }

        $containingRoot = & git -C $probePath rev-parse --show-toplevel 2>$null
        $containingRootExitCode = $LASTEXITCODE
        if ($containingRootExitCode -ne 0) {
            throw "Unable to verify whether WorktreeRoot is nested in another Git worktree: $CandidateRoot"
        }
        if (-not [string]::Equals(
                [System.IO.Path]::GetFullPath((@($containingRoot)[0]).ToString().Trim()),
                [System.IO.Path]::GetFullPath($PrimaryRoot),
                [System.StringComparison]::OrdinalIgnoreCase
            )) {
            throw "WorktreeRoot must not be located in another Git repository or linked worktree: $CandidateRoot"
        }
    }
    finally {
        $ErrorActionPreference = $previousErrorActionPreference
    }
}

function Resolve-WorktreeRoot {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)]
        [string]$PrimaryRoot,

        [string]$RequestedRoot
    )

    if ([string]::IsNullOrWhiteSpace($RequestedRoot)) {
        $parentInfo = [System.IO.Directory]::GetParent($PrimaryRoot)
        if ($null -eq $parentInfo) {
            throw "Unable to determine the parent directory for $PrimaryRoot"
        }
        $parent = $parentInfo.FullName
        $leaf = [System.IO.Path]::GetFileName($PrimaryRoot.TrimEnd([char[]]@('\', '/')))
        $resolved = Join-Path -Path $parent -ChildPath ("{0}-worktrees" -f $leaf)
    }
    else {
        $resolved = [System.IO.Path]::GetFullPath($RequestedRoot)
    }

    if (Test-PathIsWithin -ChildPath $resolved -ParentPath $PrimaryRoot) {
        throw "WorktreeRoot must be outside the primary repository: $PrimaryRoot"
    }
    Assert-NotNestedInGitWorktree -CandidateRoot $resolved -PrimaryRoot $PrimaryRoot

    return [System.IO.Path]::GetFullPath($resolved)
}

function Assert-ManagedTaskPath {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)]
        [string]$TargetPath,

        [Parameter(Mandatory = $true)]
        [string]$ManagedRoot,

        [Parameter(Mandatory = $true)]
        [string]$PrimaryRoot
    )

    if (-not (Test-PathIsWithin -ChildPath $TargetPath -ParentPath $ManagedRoot)) {
        throw "Refusing to operate outside managed WorktreeRoot: $ManagedRoot"
    }

    if (Test-PathIsWithin -ChildPath $TargetPath -ParentPath $PrimaryRoot) {
        throw "Refusing to operate inside the primary repository: $PrimaryRoot"
    }
}

function Get-WorktreeConfig {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)]
        [string]$Path,

        [Parameter(Mandatory = $true)]
        [string]$Key
    )

    $value = & git -C $Path config --worktree --get $Key 2>$null
    if ($LASTEXITCODE -ne 0) {
        return $null
    }

    return (@($value)[0]).ToString().Trim()
}

function Get-ManagedRecords {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)]
        [string]$ManagedRoot
    )

    return @(Get-WorktreeRecords | Where-Object {
            Test-PathIsWithin -ChildPath $_.Path -ParentPath $ManagedRoot
        })
}

function Get-AvailablePortBlock {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)]
        [string]$ManagedRoot,

        [Parameter(Mandatory = $true)]
        [int]$RequestedPortBlock
    )

    $used = @{}
    foreach ($record in @(Get-ManagedRecords -ManagedRoot $ManagedRoot)) {
        $configuredBlock = Get-WorktreeConfig -Path $record.Path -Key 'rgs.worktree.portBlock'
        $parsedBlock = 0
        if ($null -ne $configuredBlock -and [int]::TryParse($configuredBlock, [ref]$parsedBlock) -and $parsedBlock -ge 1 -and $parsedBlock -le 99) {
            $used[$parsedBlock] = $record.Path
        }
    }

    if ($RequestedPortBlock -ne 0) {
        if ($used.ContainsKey($RequestedPortBlock)) {
            throw "Port block $RequestedPortBlock is already assigned to $($used[$RequestedPortBlock])."
        }
        return $RequestedPortBlock
    }

    foreach ($candidate in 1..99) {
        if (-not $used.ContainsKey($candidate)) {
            return $candidate
        }
    }

    throw 'No free port blocks remain in the range 1..99.'
}

function Write-WorktreeEnv {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)]
        [string]$TargetPath,

        [Parameter(Mandatory = $true)]
        [string]$TaskName,

        [Parameter(Mandatory = $true)]
        [int]$AssignedPortBlock
    )

    $envFile = Join-Path -Path $TargetPath -ChildPath '.worktree.env'
    $portOffset = $AssignedPortBlock * 100
    $content = @(
        '# Generated by scripts/worktree.ps1. This file is ignored by Git.',
        '# Do not put passwords, tokens, connection strings, or production endpoints here.',
        "RGS_WORKTREE_ID=$TaskName",
        "RGS_PORT_BLOCK=$AssignedPortBlock",
        "RGS_PORT_OFFSET=$portOffset",
        "COMPOSE_PROJECT_NAME=rgs_$TaskName",
        "RGS_DATABASE_NAMESPACE=rgs_$TaskName"
    ) -join [Environment]::NewLine

    $utf8NoBom = New-Object -TypeName System.Text.UTF8Encoding -ArgumentList $false
    [System.IO.File]::WriteAllText($envFile, ($content + [Environment]::NewLine), $utf8NoBom)
}

function New-TaskWorktree {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)]
        [string]$PrimaryRoot,

        [Parameter(Mandatory = $true)]
        [string]$ManagedRoot,

        [Parameter(Mandatory = $true)]
        [string]$TaskName,

        [Parameter(Mandatory = $true)]
        [string]$BaseRef,

        [Parameter(Mandatory = $true)]
        [int]$RequestedPortBlock
    )

    Assert-TaskName -TaskName $TaskName
    $targetPath = Join-Path -Path $ManagedRoot -ChildPath $TaskName
    Assert-ManagedTaskPath -TargetPath $targetPath -ManagedRoot $ManagedRoot -PrimaryRoot $PrimaryRoot

    if (Test-Path -LiteralPath $targetPath) {
        throw "Target path already exists: $targetPath"
    }

    $registered = @(Get-WorktreeRecords | Where-Object {
            [string]::Equals(
                [System.IO.Path]::GetFullPath($_.Path),
                [System.IO.Path]::GetFullPath($targetPath),
                [System.StringComparison]::OrdinalIgnoreCase
            )
        })
    if ($registered.Count -gt 0) {
        throw "Git already has an administrative record for $targetPath. Inspect 'git worktree prune --dry-run' before retrying."
    }

    Invoke-Git -Arguments @('rev-parse', '--verify', ("{0}^{{commit}}" -f $BaseRef)) | Out-Null

    $branchName = "codex/wt-$TaskName"
    $branchProbe = & git show-ref --verify --quiet ("refs/heads/{0}" -f $branchName) 2>&1
    $branchProbeExitCode = $LASTEXITCODE
    if ($branchProbeExitCode -eq 0) {
        throw "Branch already exists: $branchName. Use a new task name; this tool never reuses or overwrites a branch."
    }
    if ($branchProbeExitCode -ne 1) {
        $details = ($branchProbe | ForEach-Object { $_.ToString() }) -join [Environment]::NewLine
        throw "Unable to check whether $branchName already exists. $details"
    }

    $assignedPortBlock = Get-AvailablePortBlock -ManagedRoot $ManagedRoot -RequestedPortBlock $RequestedPortBlock
    New-Item -ItemType Directory -Path $ManagedRoot -Force | Out-Null

    # Enables a per-worktree config file for identity and namespace metadata.
    Invoke-Git -Arguments @('config', 'extensions.worktreeConfig', 'true') | Out-Null
    Invoke-Git -Arguments @(
        'worktree', 'add', '--lock', '--reason', "active task: $TaskName",
        '-b', $branchName, $targetPath, $BaseRef
    ) | Out-Null

    try {
        Invoke-Git -Arguments @('-C', $targetPath, 'config', '--worktree', 'rgs.worktree.id', $TaskName) | Out-Null
        Invoke-Git -Arguments @('-C', $targetPath, 'config', '--worktree', 'rgs.worktree.portBlock', $assignedPortBlock.ToString()) | Out-Null
        Invoke-Git -Arguments @('-C', $targetPath, 'config', '--worktree', 'rgs.worktree.composeProject', "rgs_$TaskName") | Out-Null
        Write-WorktreeEnv -TargetPath $targetPath -TaskName $TaskName -AssignedPortBlock $assignedPortBlock
    }
    catch {
        Write-Warning "The worktree was created but its isolation metadata is incomplete. Do not use it until this error is resolved: $($_.Exception.Message)"
        throw
    }

    Write-Host "Created $targetPath"
    Write-Host "Branch: $branchName"
    Write-Host "Port block: $assignedPortBlock (offset $($assignedPortBlock * 100))"
    Write-Host "The worktree is locked. Run this script's doctor command before development."
}

function Show-TaskWorktrees {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)]
        [string]$ManagedRoot
    )

    $records = @(Get-ManagedRecords -ManagedRoot $ManagedRoot)
    if ($records.Count -eq 0) {
        Write-Host "No managed worktrees under $ManagedRoot"
        return
    }

    $rows = foreach ($record in $records) {
        [PSCustomObject]@{
            Id        = Get-WorktreeConfig -Path $record.Path -Key 'rgs.worktree.id'
            PortBlock = Get-WorktreeConfig -Path $record.Path -Key 'rgs.worktree.portBlock'
            Locked    = $record.Locked
            Branch    = $record.Branch
            Path      = $record.Path
        }
    }

    $rows | Sort-Object -Property Id, Path | Format-Table -AutoSize
}

function Test-TaskWorktrees {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)]
        [string]$PrimaryRoot,

        [Parameter(Mandatory = $true)]
        [string]$ManagedRoot
    )

    $healthy = $true
    $extension = & git -C $PrimaryRoot config --get extensions.worktreeConfig 2>$null
    $extensionExitCode = $LASTEXITCODE
    $extensionValue = if ($extensionExitCode -eq 0) { (@($extension)[0]).ToString().Trim() } else { '' }
    $records = @(Get-ManagedRecords -ManagedRoot $ManagedRoot)

    if ($extensionValue -ne 'true') {
        if ($records.Count -eq 0) {
            Write-Warning 'extensions.worktreeConfig is not enabled yet; create enables it before the first task worktree.'
        }
        else {
            Write-Warning 'extensions.worktreeConfig is not enabled while managed worktrees exist.'
            $healthy = $false
        }
    }

    $portOwners = @{}
    foreach ($record in $records) {
        $id = Get-WorktreeConfig -Path $record.Path -Key 'rgs.worktree.id'
        $portBlock = Get-WorktreeConfig -Path $record.Path -Key 'rgs.worktree.portBlock'

        if (-not (Test-Path -LiteralPath $record.Path)) {
            Write-Warning "Missing worktree directory: $($record.Path)"
            $healthy = $false
        }
        if (-not $record.Locked) {
            Write-Warning "Unlocked worktree: $($record.Path)"
            $healthy = $false
        }
        if ([string]::IsNullOrWhiteSpace($id)) {
            Write-Warning "Missing rgs.worktree.id metadata: $($record.Path)"
            $healthy = $false
        }
        if ([string]::IsNullOrWhiteSpace($portBlock)) {
            Write-Warning "Missing rgs.worktree.portBlock metadata: $($record.Path)"
            $healthy = $false
            continue
        }
        if ($portOwners.ContainsKey($portBlock)) {
            Write-Warning "Port block collision ${portBlock}: $($portOwners[$portBlock]) and $($record.Path)"
            $healthy = $false
        }
        else {
            $portOwners[$portBlock] = $record.Path
        }
    }

    if ($healthy) {
        Write-Host "doctor: OK ($($records.Count) managed worktree(s), root: $ManagedRoot)"
    }

    return $healthy
}

function Remove-TaskWorktree {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)]
        [string]$PrimaryRoot,

        [Parameter(Mandatory = $true)]
        [string]$ManagedRoot,

        [Parameter(Mandatory = $true)]
        [string]$TaskName
    )

    Assert-TaskName -TaskName $TaskName
    $targetPath = Join-Path -Path $ManagedRoot -ChildPath $TaskName
    Assert-ManagedTaskPath -TargetPath $targetPath -ManagedRoot $ManagedRoot -PrimaryRoot $PrimaryRoot

    $record = @(Get-WorktreeRecords | Where-Object {
            [string]::Equals(
                [System.IO.Path]::GetFullPath($_.Path),
                [System.IO.Path]::GetFullPath($targetPath),
                [System.StringComparison]::OrdinalIgnoreCase
            )
        } | Select-Object -First 1)
    if ($record.Count -eq 0) {
        throw "No Git worktree is registered at $targetPath"
    }

    $id = Get-WorktreeConfig -Path $targetPath -Key 'rgs.worktree.id'
    if ($id -ne $TaskName) {
        throw "Refusing to remove an unmanaged or mismatched worktree at $targetPath"
    }

    $statusLines = @(& git -C $targetPath status --porcelain 2>&1 | Where-Object {
            -not [string]::IsNullOrWhiteSpace($_.ToString())
        })
    if ($LASTEXITCODE -ne 0) {
        $details = ($statusLines | ForEach-Object { $_.ToString() }) -join [Environment]::NewLine
        throw "Unable to inspect worktree status at $targetPath. $details"
    }
    if ($statusLines.Count -gt 0) {
        throw "Refusing to remove a dirty worktree: $targetPath. Commit, stash, or move the changes first."
    }

    if ($record[0].Locked) {
        Invoke-Git -Arguments @('worktree', 'unlock', $targetPath) | Out-Null
    }
    Invoke-Git -Arguments @('worktree', 'remove', $targetPath) | Out-Null

    $branchName = if ($record[0].Branch) { $record[0].Branch -replace '^refs/heads/', '' } else { '(detached HEAD)' }
    Write-Host "Removed $targetPath"
    Write-Host "Retained branch: $branchName"
}

$scriptRepositoryRoot = [System.IO.Directory]::GetParent($PSScriptRoot).FullName
Push-Location -LiteralPath $scriptRepositoryRoot
try {
    $primaryRoot = Get-PrimaryRepositoryRoot
    $currentRoot = [System.IO.Path]::GetFullPath((Invoke-Git -Arguments @('rev-parse', '--show-toplevel') | Select-Object -First 1).ToString().Trim())
    if (-not [string]::Equals($currentRoot, $primaryRoot, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "Run this manager from the primary worktree only: $primaryRoot"
    }

    $worktreeRoot = Resolve-WorktreeRoot -PrimaryRoot $primaryRoot -RequestedRoot $WorktreeRoot
    switch ($Action) {
        'create' { New-TaskWorktree -PrimaryRoot $primaryRoot -ManagedRoot $worktreeRoot -TaskName $Name -BaseRef $Base -RequestedPortBlock $PortBlock }
        'list'   { Show-TaskWorktrees -ManagedRoot $worktreeRoot }
        'doctor' {
            if (-not (Test-TaskWorktrees -PrimaryRoot $primaryRoot -ManagedRoot $worktreeRoot)) {
                exit 1
            }
        }
        'remove' { Remove-TaskWorktree -PrimaryRoot $primaryRoot -ManagedRoot $worktreeRoot -TaskName $Name }
    }
}
finally {
    Pop-Location
}
