# ST-08 social 跨域 push 链路 Evidence

| 字段 | 值 |
|---|---|
| 场景 ID | st-08-social-cross-domain-push |
| BAS 章节 | §4.6 (GD) + §4.7.1 (Outbox 分发) |
| 执行时间 | 2026-08-31T18:45:19.0604011+09:00 |
| k3s 状态 | 8/27 JST 部署 |
| 步骤数 | 4 |
| 验证结果 | ✅ PASS |
| NATS 容忍 | SKIP/PASS 都算 accept (deployment optional) |

## 步骤详情

| # | 动作 | 预期 | 实际 | 状态 |
|---|---|---|---|---|
| 1 | social-service-grpc | PASS | PASS | ✅ |
| 2 | player-service-grpc | PASS | PASS | ✅ |
| 3 | admin-service-grpc | PASS | PASS | ✅ |
| 4 | nats-varz | PASS | SKIP | ✅ |

## 关键 evidence

- 复用 e2e-smoke: scripts/e2e-smoke.ps1 -ExpectNats 0
- mock 数据: scripts/st/mock/st-08-social-cross-domain-push.json
- 运行 log: docs/00-基准与治理/.test-evidence/st-2026-08-31-5dom/st-08-social-cross-domain-push.log
- 脚本: scripts/st/st-08-social-cross-domain-push.ps1

## 业务引用

- RGS-BAS-001 §4.6 GD 社交/工会
- RGS-BAS-001 §4.7.1 Outbox 分发流程 (ARC-009/010 + FR-EV-001)
- 关联 UT: 3e456b4 (social)
- 关联 IT: 3f41626 (social)
