# ST-01 player 域 gRPC port + gm-backend health Evidence

| 字段 | 值 |
|---|---|
| 场景 ID | st-01-player-grpc-port-and-gm-backend |
| BAS 章节 | §4.4 (PL 玩家/账号) |
| 执行时间 | 2026-08-31T18:33:36.9015083+09:00 |
| k3s 状态 | 8/27 JST 部署 |
| 步骤数 | 3 |
| 验证结果 | ❌ FAIL |

## 步骤详情

| # | 动作 | 预期 | 实际 | 状态 |
|---|---|---|---|---|
| 1 | player-service-grpc | PASS | PASS | ✅ |
| 2 | gm-backend-healthz | PASS | FAIL | ❌ |
| 3 | gm-backend-readyz | PASS | FAIL | ❌ |

## 关键 evidence

- 复用 e2e-smoke: scripts/e2e-smoke.ps1
- mock 数据: scripts/st/mock/st-01-player-grpc-port-and-gm-backend.json
- 运行 log: docs/00-基准与治理/.test-evidence/st-2026-08-31-5dom/st-01-player-grpc-port-and-gm-backend.log
- 脚本: scripts/st/st-01-player-grpc-port-and-gm-backend.ps1

## 业务引用

- RGS-BAS-001 §4.4.1 登录授权流程 (FR-PL-001/002)
- RGS-BAS-001 §4.4.2 session_epoch 流程 (FR-PL-003 + ARC-005)
- 关联 UT: 3cfeedb (player UT, 137 tests)
- 关联 IT: d83fb3 (player IT, 12 tests)
