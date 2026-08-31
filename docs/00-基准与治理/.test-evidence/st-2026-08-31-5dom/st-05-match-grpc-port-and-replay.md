# ST-05 match 域 gRPC port + replay 链路 Evidence

| 字段 | 值 |
|---|---|
| 场景 ID | st-05-match-grpc-port-and-replay |
| BAS 章节 | §4.2 (RT 实时场景) + §5.5 (match_db) |
| 执行时间 | 2026-08-31T18:45:22.6721164+09:00 |
| k3s 状态 | 8/27 JST 部署 |
| 步骤数 | 4 |
| 验证结果 | ❌ FAIL |

## 步骤详情

| # | 动作 | 预期 | 实际 | 状态 |
|---|---|---|---|---|
| 1 | match-service-grpc | PASS | PASS | ✅ |
| 2 | gm-backend-healthz | PASS | FAIL | ❌ |
| 3 | player-service-grpc | PASS | PASS | ✅ |
| 4 | postgres | PASS | PASS | ✅ |

## 关键 evidence

- 复用 e2e-smoke: scripts/e2e-smoke.ps1
- mock 数据: scripts/st/mock/st-05-match-grpc-port-and-replay.json
- 运行 log: docs/00-基准与治理/.test-evidence/st-2026-08-31-5dom/st-05-match-grpc-port-and-replay.log
- 脚本: scripts/st/st-05-match-grpc-port-and-replay.ps1

## 业务引用

- RGS-BAS-001 §4.2 RT 实时场景时环 (NFR-PE-002 tick 循环)
- RGS-BAS-001 §4.5.1 EC 确定性 (SaveReplay saga)
- 关联 UT: 5070547 (match)
- 关联 IT: c70ef64 (match)
