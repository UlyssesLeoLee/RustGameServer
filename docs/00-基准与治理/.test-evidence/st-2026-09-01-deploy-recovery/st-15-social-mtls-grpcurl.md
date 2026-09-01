# st-15 social mTLS 业务级 gRPC (per 9/1 14:00 JST 续 Q10)

## 操作
1. grpcurl 1.9.1 + social mTLS cert
2. 调 social.v1.SocialService/HealthCheck

## 结果
- Verdict: PASS
- Detail: mTLS OK + HealthCheck 返回 status=Ok

## 输出
``n{
  "status": "STATUS_OK",
  "message": "ok"
}
``n
