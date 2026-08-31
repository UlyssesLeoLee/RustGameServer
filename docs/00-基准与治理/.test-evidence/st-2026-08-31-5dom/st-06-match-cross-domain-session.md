# ST-06 match 跨域 session 链路 Evidence

| 字段 | 值 |
|---|---|
| 场景 ID | st-06-match-cross-domain-session |
| BAS 章节 | §4.2 (RT) + §4.4 (PL) + §4.6 (MT/GD) |
| 执行时间 | 2026-08-31T18:45:22.7347533+09:00 |
| k3s 状态 | 8/27 JST 部署 |
| 步骤数 | 4 |
| 验证结果 | ✅ PASS |

## 步骤详情

| # | 动作 | 预期 | 实际 | 状态 |
|---|---|---|---|---|
| 1 | match-service-grpc | PASS | PASS | ✅ |
| 2 | player-service-grpc | PASS | PASS | ✅ |
| 3 | social-service-grpc | PASS | PASS | ✅ |
| 4 | admin-service-grpc | PASS | PASS | ✅ |

## 关键 evidence

- 复用 e2e-smoke: scripts/e2e-smoke.ps1
- mock 数据: scripts/st/mock/st-06-match-cross-domain-session.json
- 运行 log: docs/00-基准与治理/.test-evidence/st-2026-08-31-5dom/st-06-match-cross-domain-session.log
- 脚本: scripts/st/st-06-match-cross-domain-session.ps1

## 业务引用

- RGS-BAS-001 §4.2 RT 实时场景时环 + §4.6 MT/GD 对局/社交
- 关联 UT: 5070547 (match)
- 关联 IT: c70ef64 (match)
