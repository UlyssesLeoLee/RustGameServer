# st-13 economy mTLS 业务级 gRPC (per 9/1 14:00 JST 续 Q10)

## 操作
1. grpcurl 1.9.1 + economy mTLS cert (从 k3s rgs-secret-economy-tls 提取)
2. 调 economy.v1.EconomyService/HealthCheck

## 结果
- Verdict: PASS
- Detail: mTLS OK, HealthCheck 返回 status=Ok

## 输出
``n{
  "status": "STATUS_OK",
  "message": "ok"
}
``n
