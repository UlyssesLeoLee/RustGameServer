$env:GM_HTTP_ADDR = "127.0.0.1:0"
$env:GM_HEALTH_ADDR = "127.0.0.1:0"
$env:RGS_ALLOW_INSECURE_GRPC = "1"
$env:RUST_LOG = "info"
$output = & E:\DevCache\cargo\target\debug\gm-backend.exe 2>&1 | Out-String
$output
$env:RGS_ALLOW_INSECURE_GRPC = $null
$env:GM_HTTP_ADDR = $null
$env:GM_HEALTH_ADDR = $null
$env:RUST_LOG = $null
