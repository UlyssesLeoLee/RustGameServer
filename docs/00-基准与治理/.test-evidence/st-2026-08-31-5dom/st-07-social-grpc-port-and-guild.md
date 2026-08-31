# ST-07 social 域 gRPC port + guild Evidence

| 字段 | 值 |
|---|---|
| 场景 ID | st-07-social-grpc-port-and-guild |
| BAS 章节 | §4.6 (GD 社交/工会) + §5.6 (social_db) |
| 执行时间 | 2026-08-31T18:45:22.7385126+09:00 |
| k3s 状态 | 8/27 JST 部署 |
| 步骤数 | 4 |
| 验证结果 | ❌ FAIL |

## 步骤详情

| # | 动作 | 预期 | 实际 | 状态 |
|---|---|---|---|---|
| 1 | social-service-grpc | PASS | PASS | ✅ |
| 2 | gm-backend-healthz | PASS | FAIL | ❌ |
| 3 | player-service-grpc | PASS | PASS | ✅ |
| 4 | postgres | PASS | PASS | ✅ |

## 关键 evidence

- 复用 e2e-smoke: scripts/e2e-smoke.ps1
- mock 数据: scripts/st/mock/st-07-social-grpc-port-and-guild.json
- 运行 log: docs/00-基准与治理/.test-evidence/st-2026-08-31-5dom/st-07-social-grpc-port-and-guild.log
- 脚本: scripts/st/st-07-social-grpc-port-and-guild.ps1

## 业务引用

- RGS-BAS-001 §4.6 GD 社交/工会 (PH-6 详细)
- RGS-BAS-001 §5.6 social_db
- 关联 UT: 3e456b4 (social)
- 关联 IT: 3f41626 (social)
