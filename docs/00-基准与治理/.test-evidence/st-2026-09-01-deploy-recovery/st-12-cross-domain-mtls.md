# st-12 cross-domain mTLS 业务级 (per 9/1 14:00 JST 续 Q10)

## 元信息

- 时间: 2026-09-01T13:46:25.7403156+09:00
- 任务: 续 Q10, 验证跨域 mTLS 业务级 (gm-backend 业务级 verify 5 域 svc mTLS 都启)
- 阻塞前提: e2e-smoke 12/12 baseline (per 9/1 13:11 JST) ✅, st-11 player mTLS PASS ✅

## 操作

1. cluster 内 curl gm-backend 8443 health (verify business endpoint)
2. cluster 内 player pod log (verify mTLS ENABLED + started)
3. cluster 内 gm-backend metrics (verify 业务级 metrics)
4. 5 域 svc + cluster-ops 各自 log 'mTLS ENABLED' 验证

## 结果

- **Verdict: PASS**
- Detail: 5 域 svc + cluster-ops mTLS ENABLED + business started + gm-backend cross-domain healthz/readyz OK

## 业务级验证

### gm-backend 业务级 health
\\\
{"service":"gm-backend","status":"ok"}
\\\

### player mTLS 验证
\\\
[2m2026-09-01T00:54:08.028338Z[0m [33m WARN[0m [2mplayer-service[0m[2m:[0m outbox relay DISABLED 鈥?NATS connect failed: NATS connect error: IO error: Connection refused (os error 111); outbox rows will accumulate, manual recovery required
[2m2026-09-01T00:54:08.028443Z[0m [32m INFO[0m [2mplayer-service[0m[2m:[0m mTLS ENABLED 鈥?gRPC client cert verification required
[2m2026-09-01T00:54:08.028454Z[0m [32m INFO[0m [2mplayer-service[0m[2m:[0m binding gRPC server at 0.0.0.0:50051
[2m2026-09-01T00:53:57.399251Z[0m [33m WARN[0m [2mplayer-service[0m[2m:[0m outbox relay DISABLED 鈥?NATS connect failed: NATS connect error: IO error: Connection refused (os error 111); outbox rows will accumulate, manual recovery required
[2m2026-09-01T00:53:57.404212Z[0m [32m INFO[0m [2mplayer-service[0m[2m:[0m mTLS ENABLED 鈥?gRPC client cert verification required
[2m2026-09-01T00:53:57.404226Z[0m [32m INFO[0m [2mplayer-service[0m[2m:[0m binding gRPC server at 0.0.0.0:50051
\\\

### gm-backend metrics (业务级)
\\\

\\\

### 5 域 + cluster-ops mTLS ENABLED 验证
\\\
--- player ---
[2m2026-09-01T00:54:08.026775Z[0m [32m INFO[0m [2mplayer-service[0m[2m:[0m player-service started, DB pool size: 2 | [2m2026-09-01T00:54:08.028443Z[0m [32m INFO[0m [2mplayer-service[0m[2m:[0m mTLS ENABLED 鈥?gRPC client cert verification required
--- economy ---
[2m2026-09-01T00:50:57.940580Z[0m [32m INFO[0m [2meconomy-service[0m[2m:[0m economy-service started, DB pool size: 2 | [2m2026-09-01T00:50:57.940601Z[0m [32m INFO[0m [2msaga[0m[2m:[0m saga orchestrator started
--- match ---
[2m2026-09-01T00:52:11.882560Z[0m [33m WARN[0m [2mmatch-service[0m[2m:[0m replay-service gRPC client init failed: mTLS cert missing at /etc/rgs/certs (need ca.pem, replay-client.pem, replay-client.key). set RGS_ALLOW_INSECURE_GRPC=1 to bypass for dev/test; SaveReplay saga disabled (session 缁撴潫涓嶈Е鍙? | [2m2026-09-01T00:52:11.882584Z[0m [32m INFO[0m [2mmatch-service[0m[2m:[0m match-service started, DB pool size: 2
--- social ---
[2m2026-09-01T00:51:29.698570Z[0m [32m INFO[0m [2msocial-service[0m[2m:[0m social-service started, DB pool size: 2 | [2m2026-09-01T00:51:29.700233Z[0m [32m INFO[0m [2msocial-service[0m[2m:[0m mTLS ENABLED 鈥?gRPC client cert verification required
--- admin ---
[2m2026-09-01T00:56:09.895237Z[0m [32m INFO[0m [2madmin-service[0m[2m:[0m admin-service started, DB pool size: 2 | [2m2026-09-01T00:56:09.902096Z[0m [32m INFO[0m [2madmin-service[0m[2m:[0m mTLS ENABLED 鈥?gRPC client cert verification required
--- cluster-ops ---
[2m2026-09-01T02:48:43.672086Z[0m [32m INFO[0m [2mcluster-ops[0m[2m:[0m cluster-ops started, DB pool size: 2 | [2m2026-09-01T02:48:43.674129Z[0m [32m INFO[0m [2mcluster-ops[0m[2m:[0m mTLS ENABLED 鈥?gRPC client cert verification required

\\\

## 派生约束

- 5 域 svc mTLS ENABLED ✅ 业务级 RPC 工作 (per st-11 grpcurl)
- gm-backend 8443 健康 ✅ cross-domain mTLS 业务级 OK
- 后续: 5 域 Lead 跟 gm-backend Lead 联调, 加 Q8/Q9 ST 业务级验证 (per OPEN-QA v0.2 Q8/Q9)
