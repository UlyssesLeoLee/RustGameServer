# ST-03 economy 域 gRPC port + postgres outbox Evidence

| 字段 | 值 |
|---|---|
| 场景 ID | st-03-economy-grpc-port-and-outbox |
| BAS 章节 | §4.5.1 (EC 确定性 API) + §5.4 (economy_db) |
| 执行时间 | 2026-08-31T18:45:22.6838350+09:00 |
| k3s 状态 | 8/27 JST 部署 |
| 步骤数 | 4 |
| 验证结果 | ✅ PASS |

## 步骤详情

| # | 动作 | 预期 | 实际 | 状态 |
|---|---|---|---|---|
| 1 | economy-service-grpc | PASS | PASS | ✅ |
| 2 | postgres | PASS | PASS | ✅ |
| 3 | player-service-grpc | PASS | PASS | ✅ |
| 4 | admin-service-grpc | PASS | PASS | ✅ |

## 关键 evidence

- 复用 e2e-smoke: scripts/e2e-smoke.ps1
- mock 数据: scripts/st/mock/st-03-economy-grpc-port-and-outbox.json
- 运行 log: docs/00-基准与治理/.test-evidence/st-2026-08-31-5dom/st-03-economy-grpc-port-and-outbox.log
- 脚本: scripts/st/st-03-economy-grpc-port-and-outbox.ps1

## 业务引用

- RGS-BAS-001 §4.5.1 EC 确定性 API (FR-EC-003 + ARC-006/009)
- RGS-BAS-001 §4.7.1 Outbox 分发 (ARC-009/010 + FR-EV-001)
- 关联 UT: 1db3249 (economy)
- 关联 IT: fd3d65 (economy)
