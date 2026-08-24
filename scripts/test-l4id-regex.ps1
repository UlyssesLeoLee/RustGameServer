$ErrorActionPreference = 'Stop'
$pattern = '^WF-(\d+(\.\d+)?)-(\d+)(\.\d+)?$'
$cases = @(
    @{ Id = 'WF-1-54.1';  Expect = 'PASS' }
    @{ Id = 'WF-0.5-1';  Expect = 'PASS' }
    @{ Id = 'WF-1-1';    Expect = 'PASS' }
    @{ Id = 'WF-1.5-2';  Expect = 'PASS' }
    @{ Id = 'WF-0.5-1.2'; Expect = 'PASS' }
    @{ Id = 'WF-1-54';   Expect = 'PASS' }
    @{ Id = 'WF-1';      Expect = 'FAIL' }
    @{ Id = 'WF-0.5-';   Expect = 'FAIL' }
    @{ Id = 'ABC-0.5-1'; Expect = 'FAIL' }
    @{ Id = 'WF-';       Expect = 'FAIL' }
)
$fail = 0
foreach ($c in $cases) {
    $m = [regex]::IsMatch($c.Id, $pattern)
    $status = if ($m) { 'PASS' } else { 'FAIL' }
    $icon = if ($status -eq $c.Expect) { 'OK' } else { 'X' }
    if ($status -ne $c.Expect) { $fail++ }
    Write-Host ("[{0}] {1} (regex={2}, expect={3})" -f $icon, $c.Id, $status, $c.Expect)
}
if ($fail -gt 0) { Write-Host "FAILED: $fail case(s)"; exit 1 }
Write-Host "ALL OK: 10 case(s) pass"
