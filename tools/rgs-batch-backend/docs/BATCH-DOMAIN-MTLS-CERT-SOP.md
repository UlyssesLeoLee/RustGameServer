# Batch 域 5 域 mTLS Cert 导出 SOP (E3 L4-1, per 9/8 20:47 JST 派工)

> **作用域**: rgs-batch-backend 5 域 gRPC client mTLS 接入 (player/economy/match/social/admin)
> **依据**: 5 域 ST 业务级 mTLS 实践 commit `401ac5c` (per 8/31 W7 ST)
> **派生**: AGENTS.md v0.4 §2.4 L4 强约束 (跨多工具链场景主会话打头阵, 业务级 mTLS 跑通, k3s 不可达记 backlog)

## 1. Cert 来源

5 域 (player/economy/match/social/admin) cert 复用 8/27 ST 阶段导出的 SOP, 路径位于
k3s namespace `rust-game-server`, 5 个 secret:
- `player-service-tls`
- `economy-service-tls`
- `match-service-tls`
- `social-service-tls`
- `admin-service-tls`

## 2. 导出命令 (k3s 可达时)

```bash
# 5 域 ca.crt
for d in player economy match social admin; do
  kubectl get secret ${d}-service-tls -n rust-game-server \
    -o jsonpath='{.data.ca\.crt}' | base64 -d \
    > /etc/rgs-batch/tls/${d}-ca.crt
done

# 5 域 client cert + key (rgs-batch-backend 自己签发, 复用 8/27 ST 模式)
for d in player economy match social admin; do
  kubectl get secret rgs-batch-client-tls -n rust-game-server \
    -o jsonpath='{.data.tls\.crt}' | base64 -d \
    > /etc/rgs-batch/tls/client.crt
  kubectl get secret rgs-batch-client-tls -n rust-game-server \
    -o jsonpath='{.data.tls\.key}' | base64 -d \
    > /etc/rgs-batch/tls/client.key
done
```

## 3. Env 注入 (per 8/27 11:06 JST 硬 ban, 不打印)

rgs-batch-backend 启动时需:
- `GRPC_CA_CERT_PEM=<multi-line PEM, 5 域 ca.crt 合并>`
- `GRPC_CLIENT_CERT_PEM=<client.crt 内容>`
- `GRPC_CLIENT_KEY_PEM=<client.key 内容>`
- `GRPC_PLAYER_ENDPOINT=https://player-service:50051`
- `GRPC_ECONOMY_ENDPOINT=https://economy-service:50052`
- `GRPC_MATCH_ENDPOINT=https://match-service:50053`
- `GRPC_SOCIAL_ENDPOINT=https://social-service:50054`
- `GRPC_ADMIN_ENDPOINT=https://admin-service:50055`

5 域 cert PEM 注入 k8s Secret `rgs-batch-tls` (per `k8s/77-rgs-batch-tls.yaml`),
deployment 引用 `secretKeyRef`, 走 k8s 卷挂载 `/etc/rgs-batch/tls/`。

## 4. 业务级 mTLS 跑通验证

启动 rgs-batch-backend 后:
```bash
curl http://127.0.0.1:8790/api/v1/grpc-mtls-config
# 期望: ready=true, identity=true, ca_loaded=true

curl http://127.0.0.1:8790/api/v1/grpc-health
# 期望: 5 域 healthy=true (k3s 可达时)
# 不可达时: healthy=false + 记录 error
```

## 5. 已知缺口 (per 9/8 20:47 JST NO-GO 风险)

- k3s 集群可达性: worker 启动时若 k3s 不可达, health_check 返 unhealthy + 不假装跑通
- 5 域 gRPC Health service 端到端实测: 需集群可达后由主会话 ST 阶段补 (per L4 checklist)
- Cert 自动轮换: v0.1 不集成, v0.2 评估 (per RGS-BATCH-V0.2-EVAL-2026-09-03 v0.1 §4)

## 6. 派生约束

- 凭据永不打印 (per 8/27 11:06 JST 硬 ban)
- 5 域 gRPC endpoint 走 mTLS, 不降级到 plaintext
- Cert 路径失败 → 返回 503 + 明确错误, 不静默降级
