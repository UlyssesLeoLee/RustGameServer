$env:SHIM_PORT = "9002"
$env:RGS_PROXY = "http://127.0.0.1:8084"
$env:RUST_LOG = "info"
$exe = "D:\RustGameServer\target\shim-w2\release\rgs-shim.exe"
Start-Process -FilePath $exe -RedirectStandardOutput "D:\rgs-shim-w2\shim-w2.log" -RedirectStandardError "D:\rgs-shim-w2\shim-w2-err.log" -PassThru | Out-Null
Start-Sleep -Seconds 2
Get-Content "D:\rgs-shim-w2\shim-w2.log" -ErrorAction SilentlyContinue | Select-Object -First 3
