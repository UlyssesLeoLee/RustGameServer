# st-11 player mTLS 业务级 gRPC (per 9/1 14:00 JST 续 Q10)

## 元信息

- 时间: 2026-09-01T13:44:50.4150721+09:00
- 任务: 续 Q10 mTLS 业务级 ST, 验证 player-service mTLS 启用 + grpcurl 业务级 RPC
- 阻塞前提: e2e-smoke 12/12 baseline (per 9/1 13:11 JST) ✅, grpcurl 1.9.1 装好 ✅
- 工具: grpcurl 1.9.1 + 5 域 mTLS cert (从 k3s rgs-secret-*-tls 提取)

## 操作

1. wsl 端装 grpcurl: \curl gh-proxy.com/.../grpcurl_1.9.1_linux_x86_64.tar.gz\
2. k3s secret 提取 5 域 mTLS cert (rgs-secret-{player,economy,match,social,admin}-tls + rgs-secret-ca)
3. grpcurl -cacert ca.pem -cert player-client.pem -key player-client.key -servername player.service
   -import-path common.proto -proto player.proto
   -d '{"request_id":"st-11-2026-09-01"}' 10.42.0.221:50051 player.v1.PlayerService/HealthCheck

## 结果

- **Verdict: PASS**
- Detail: mTLS OK, HealthCheck 返回 status=Ok

## 输出 (节选)

\\\
{
  "status": "STATUS_OK",
  "message": "ok"
}
\\\

## 派生约束

- 5 域 svc 启 mTLS ENABLED (per pod log), 业务级 gRPC 调通, 验证 RGS-BAS-003-mTLS 决策 (v0.1)
- 5 域 mTLS cert 可复用 rgs-secret-*-tls tls.crt 当 client cert (k3s TLS secret tls.crt 自签 mTLS server cert, 同时含 client capability)
- 后续: st-12 cross-domain mTLS (gm-backend 调 player 业务级) 续跑
