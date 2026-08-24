# Kubernetes Manifest Probe 段一致性核对报告

> **任务**：WF-1-55.46 verify_probe_consistency.ps1 + 6 份 manifest probe 段全核对
> **生成时间**：2026-08-25 06:14:59 +09:00
> **脚本入口**：scripts/verify_probe_consistency.ps1
> **关联疑问**：RGS-OPEN-QA-001 v0.2 Q-M-04 + ACTIONS-v0.3 §3 B-05
> **基线 manifest**：01-player-service.yaml（作为 canonical reference）

## 0. 头表 — 报告元信息

| 字段 | 值 |
|---|---|
| 报告生成时间 | 2026-08-25 06:14:59 +09:00 |
| 脚本 | `scripts/verify_probe_consistency.ps1` |
| Manifest 根目录 | docs/deploy/01-k8s-manifests |
| 报告输出路径 | docs/deploy/probe-consistency-report.md |
| PowerShell 版本 | 7.6.3 |
| 6 份 manifest | 01-player / 02-economy / 03-match / 04-social / 05-admin / 06-cluster-ops |
| 字段集差异数 | 0 |
| 阈值差异数（vs player 基线） | 8 |
| 命令结构差异数 | 0 |

## 1. 6 份 manifest probe 段实际参数表

### 1.1 01-player-service.yaml

**livenessProbe.grpc_health_probe 命令（完整）**：
```
  /bin/grpc_health_probe
  -addr=127.0.0.1:50051
  -tls
  -tls-client-cert=/etc/rgs/certs/server.pem
  -tls-client-key=/etc/rgs/certs/server.key
  -tls-ca-cert=/etc/rgs/certs/ca.pem
  -tls-server-name=player.service
  -connect-timeout=2s
```

**readinessProbe.grpc_health_probe 命令（完整）**：
```
  /bin/grpc_health_probe
  -addr=127.0.0.1:50051
  -tls
  -tls-client-cert=/etc/rgs/certs/server.pem
  -tls-client-key=/etc/rgs/certs/server.key
  -tls-ca-cert=/etc/rgs/certs/ca.pem
  -tls-server-name=player.service
  -connect-timeout=2s
```

**livenessProbe 阈值**：
| initialDelaySeconds | periodSeconds | timeoutSeconds | failureThreshold |
|---|---|---|---|
| 30 | 30 | 5 | 3 |

**readinessProbe 阈值**：
| initialDelaySeconds | periodSeconds | timeoutSeconds | failureThreshold |
|---|---|---|---|
| 10 | 10 | 3 | 3 |

**volumeMounts**（仅名字，验证存在性）：
```
  - rgs-tls
```

**Canonical 命令骨架**（剥离 `-addr` / `-tls-server-name` / `-connect-timeout` 域特定值）：
```
  liveness : /bin/grpc_health_probe -tls -tls-client-cert=/etc/rgs/certs/server.pem -tls-client-key=/etc/rgs/certs/server.key -tls-ca-cert=/etc/rgs/certs/ca.pem
  readiness: /bin/grpc_health_probe -tls -tls-client-cert=/etc/rgs/certs/server.pem -tls-client-key=/etc/rgs/certs/server.key -tls-ca-cert=/etc/rgs/certs/ca.pem
```

### 1.2 02-economy-service.yaml

**livenessProbe.grpc_health_probe 命令（完整）**：
```
  /bin/grpc_health_probe
  -addr=127.0.0.1:50052
  -tls
  -tls-client-cert=/etc/rgs/certs/server.pem
  -tls-client-key=/etc/rgs/certs/server.key
  -tls-ca-cert=/etc/rgs/certs/ca.pem
  -tls-server-name=economy.service
  -connect-timeout=2s
```

**readinessProbe.grpc_health_probe 命令（完整）**：
```
  /bin/grpc_health_probe
  -addr=127.0.0.1:50052
  -tls
  -tls-client-cert=/etc/rgs/certs/server.pem
  -tls-client-key=/etc/rgs/certs/server.key
  -tls-ca-cert=/etc/rgs/certs/ca.pem
  -tls-server-name=economy.service
  -connect-timeout=2s
```

**livenessProbe 阈值**：
| initialDelaySeconds | periodSeconds | timeoutSeconds | failureThreshold |
|---|---|---|---|
| 30 | 30 | 5 | 3 |

**readinessProbe 阈值**：
| initialDelaySeconds | periodSeconds | timeoutSeconds | failureThreshold |
|---|---|---|---|
| 15 | 10 | 3 | 3 |

**volumeMounts**（仅名字，验证存在性）：
```
  - rgs-tls
```

**Canonical 命令骨架**（剥离 `-addr` / `-tls-server-name` / `-connect-timeout` 域特定值）：
```
  liveness : /bin/grpc_health_probe -tls -tls-client-cert=/etc/rgs/certs/server.pem -tls-client-key=/etc/rgs/certs/server.key -tls-ca-cert=/etc/rgs/certs/ca.pem
  readiness: /bin/grpc_health_probe -tls -tls-client-cert=/etc/rgs/certs/server.pem -tls-client-key=/etc/rgs/certs/server.key -tls-ca-cert=/etc/rgs/certs/ca.pem
```

### 1.3 03-match-service.yaml

**livenessProbe.grpc_health_probe 命令（完整）**：
```
  /bin/grpc_health_probe
  -addr=127.0.0.1:50053
  -tls
  -tls-client-cert=/etc/rgs/certs/server.pem
  -tls-client-key=/etc/rgs/certs/server.key
  -tls-ca-cert=/etc/rgs/certs/ca.pem
  -tls-server-name=match.service
  -connect-timeout=2s
```

**readinessProbe.grpc_health_probe 命令（完整）**：
```
  /bin/grpc_health_probe
  -addr=127.0.0.1:50053
  -tls
  -tls-client-cert=/etc/rgs/certs/server.pem
  -tls-client-key=/etc/rgs/certs/server.key
  -tls-ca-cert=/etc/rgs/certs/ca.pem
  -tls-server-name=match.service
  -connect-timeout=2s
```

**livenessProbe 阈值**：
| initialDelaySeconds | periodSeconds | timeoutSeconds | failureThreshold |
|---|---|---|---|
| 20 | 15 | 3 | 3 |

**readinessProbe 阈值**：
| initialDelaySeconds | periodSeconds | timeoutSeconds | failureThreshold |
|---|---|---|---|
| 5 | 5 | 2 | 2 |

**volumeMounts**（仅名字，验证存在性）：
```
  - rgs-tls
```

**Canonical 命令骨架**（剥离 `-addr` / `-tls-server-name` / `-connect-timeout` 域特定值）：
```
  liveness : /bin/grpc_health_probe -tls -tls-client-cert=/etc/rgs/certs/server.pem -tls-client-key=/etc/rgs/certs/server.key -tls-ca-cert=/etc/rgs/certs/ca.pem
  readiness: /bin/grpc_health_probe -tls -tls-client-cert=/etc/rgs/certs/server.pem -tls-client-key=/etc/rgs/certs/server.key -tls-ca-cert=/etc/rgs/certs/ca.pem
```

### 1.4 04-social-service.yaml

**livenessProbe.grpc_health_probe 命令（完整）**：
```
  /bin/grpc_health_probe
  -addr=127.0.0.1:50054
  -tls
  -tls-client-cert=/etc/rgs/certs/server.pem
  -tls-client-key=/etc/rgs/certs/server.key
  -tls-ca-cert=/etc/rgs/certs/ca.pem
  -tls-server-name=social.service
  -connect-timeout=2s
```

**readinessProbe.grpc_health_probe 命令（完整）**：
```
  /bin/grpc_health_probe
  -addr=127.0.0.1:50054
  -tls
  -tls-client-cert=/etc/rgs/certs/server.pem
  -tls-client-key=/etc/rgs/certs/server.key
  -tls-ca-cert=/etc/rgs/certs/ca.pem
  -tls-server-name=social.service
  -connect-timeout=2s
```

**livenessProbe 阈值**：
| initialDelaySeconds | periodSeconds | timeoutSeconds | failureThreshold |
|---|---|---|---|
| 30 | 30 | 5 | 3 |

**readinessProbe 阈值**：
| initialDelaySeconds | periodSeconds | timeoutSeconds | failureThreshold |
|---|---|---|---|
| 10 | 10 | 3 | 3 |

**volumeMounts**（仅名字，验证存在性）：
```
  - rgs-tls
```

**Canonical 命令骨架**（剥离 `-addr` / `-tls-server-name` / `-connect-timeout` 域特定值）：
```
  liveness : /bin/grpc_health_probe -tls -tls-client-cert=/etc/rgs/certs/server.pem -tls-client-key=/etc/rgs/certs/server.key -tls-ca-cert=/etc/rgs/certs/ca.pem
  readiness: /bin/grpc_health_probe -tls -tls-client-cert=/etc/rgs/certs/server.pem -tls-client-key=/etc/rgs/certs/server.key -tls-ca-cert=/etc/rgs/certs/ca.pem
```

### 1.5 05-admin-service.yaml

**livenessProbe.grpc_health_probe 命令（完整）**：
```
  /bin/grpc_health_probe
  -addr=127.0.0.1:50055
  -tls
  -tls-client-cert=/etc/rgs/certs/server.pem
  -tls-client-key=/etc/rgs/certs/server.key
  -tls-ca-cert=/etc/rgs/certs/ca.pem
  -tls-server-name=admin.service
  -connect-timeout=2s
```

**readinessProbe.grpc_health_probe 命令（完整）**：
```
  /bin/grpc_health_probe
  -addr=127.0.0.1:50055
  -tls
  -tls-client-cert=/etc/rgs/certs/server.pem
  -tls-client-key=/etc/rgs/certs/server.key
  -tls-ca-cert=/etc/rgs/certs/ca.pem
  -tls-server-name=admin.service
  -connect-timeout=2s
```

**livenessProbe 阈值**：
| initialDelaySeconds | periodSeconds | timeoutSeconds | failureThreshold |
|---|---|---|---|
| 30 | 30 | 5 | 3 |

**readinessProbe 阈值**：
| initialDelaySeconds | periodSeconds | timeoutSeconds | failureThreshold |
|---|---|---|---|
| 10 | 10 | 3 | 3 |

**volumeMounts**（仅名字，验证存在性）：
```
  - rgs-tls
  - coc-tls
```

**Canonical 命令骨架**（剥离 `-addr` / `-tls-server-name` / `-connect-timeout` 域特定值）：
```
  liveness : /bin/grpc_health_probe -tls -tls-client-cert=/etc/rgs/certs/server.pem -tls-client-key=/etc/rgs/certs/server.key -tls-ca-cert=/etc/rgs/certs/ca.pem
  readiness: /bin/grpc_health_probe -tls -tls-client-cert=/etc/rgs/certs/server.pem -tls-client-key=/etc/rgs/certs/server.key -tls-ca-cert=/etc/rgs/certs/ca.pem
```

### 1.6 06-cluster-ops-service.yaml

**livenessProbe.grpc_health_probe 命令（完整）**：
```
  /bin/grpc_health_probe
  -addr=127.0.0.1:50056
  -tls
  -tls-client-cert=/etc/rgs/certs/server.pem
  -tls-client-key=/etc/rgs/certs/server.key
  -tls-ca-cert=/etc/rgs/certs/ca.pem
  -tls-server-name=cluster-ops.service
  -connect-timeout=2s
```

**readinessProbe.grpc_health_probe 命令（完整）**：
```
  /bin/grpc_health_probe
  -addr=127.0.0.1:50056
  -tls
  -tls-client-cert=/etc/rgs/certs/server.pem
  -tls-client-key=/etc/rgs/certs/server.key
  -tls-ca-cert=/etc/rgs/certs/ca.pem
  -tls-server-name=cluster-ops.service
  -connect-timeout=2s
```

**livenessProbe 阈值**：
| initialDelaySeconds | periodSeconds | timeoutSeconds | failureThreshold |
|---|---|---|---|
| 30 | 30 | 5 | 3 |

**readinessProbe 阈值**：
| initialDelaySeconds | periodSeconds | timeoutSeconds | failureThreshold |
|---|---|---|---|
| 10 | 10 | 3 | 3 |

**volumeMounts**（仅名字，验证存在性）：
```
  - rgs-tls
```

**Canonical 命令骨架**（剥离 `-addr` / `-tls-server-name` / `-connect-timeout` 域特定值）：
```
  liveness : /bin/grpc_health_probe -tls -tls-client-cert=/etc/rgs/certs/server.pem -tls-client-key=/etc/rgs/certs/server.key -tls-ca-cert=/etc/rgs/certs/ca.pem
  readiness: /bin/grpc_health_probe -tls -tls-client-cert=/etc/rgs/certs/server.pem -tls-client-key=/etc/rgs/certs/server.key -tls-ca-cert=/etc/rgs/certs/ca.pem
```

## 2. Diff 矩阵（6×6 — 阈值差异数）

> 单元格 = 行列两 manifest 的 8 个阈值字段（4 liveness + 4 readiness）差异总数。
> 对角线为 0（自比），矩阵对称。

|  | 01-player-service.yaml | 02-economy-service.yaml | 03-match-service.yaml | 04-social-service.yaml | 05-admin-service.yaml | 06-cluster-ops-service.yaml | 
|---|---|---|---|---|---|---|
| 01-player-service.yaml |  0 |  **1** |  **7** |  0 |  0 |  0 | 
| 02-economy-service.yaml |  **1** |  0 |  **7** |  **1** |  **1** |  **1** | 
| 03-match-service.yaml |  **7** |  **7** |  0 |  **7** |  **7** |  **7** | 
| 04-social-service.yaml |  0 |  **1** |  **7** |  0 |  0 |  0 | 
| 05-admin-service.yaml |  0 |  **1** |  **7** |  0 |  0 |  0 | 
| 06-cluster-ops-service.yaml |  0 |  **1** |  **7** |  0 |  0 |  0 | 

## 3. 字段集差异（liveness / readiness 必须含 4 个阈值字段）

✅ **无差异** — 6 份 manifest 的 liveness/readiness 都含 `initialDelaySeconds` / `periodSeconds` / `timeoutSeconds` / `failureThreshold` 全 4 字段

## 4. 关键阈值差异清单（vs 01-player-service.yaml 基线）

⚠️ 共发现 **8** 处阈值差异：

- readiness.initialDelaySeconds  01-player-service.yaml=10 vs 02-economy-service.yaml=15
- liveness.initialDelaySeconds  01-player-service.yaml=30 vs 03-match-service.yaml=20
- readiness.initialDelaySeconds  01-player-service.yaml=10 vs 03-match-service.yaml=5
- liveness.periodSeconds  01-player-service.yaml=30 vs 03-match-service.yaml=15
- readiness.periodSeconds  01-player-service.yaml=10 vs 03-match-service.yaml=5
- liveness.timeoutSeconds  01-player-service.yaml=5 vs 03-match-service.yaml=3
- readiness.timeoutSeconds  01-player-service.yaml=3 vs 03-match-service.yaml=2
- readiness.failureThreshold  01-player-service.yaml=3 vs 03-match-service.yaml=2

## 5. 命令结构差异清单（canonical 骨架对比）

✅ **无差异** — 6 份 manifest 的 `grpc_health_probe` 命令骨架（除 `-addr` / `-tls-server-name` / `-connect-timeout` 域特定值外）完全一致

## 6. 结论

⚠️ **6 份 manifest probe 段存在 8 处不一致**，建议 Ulysses 终审后用以下方式收敛：

1. **短期（PH-1 暂不引入 Helm）**：手动修改 6 份 manifest 至统一基线（建议参考 01-player），跑本脚本验证 exit 0
2. **长期（PH-2 引入 Helm）**：用 Helm template + values 收敛 probe 段，6 份 Deployment 由 chart 派生

> **重要**：本脚本**仅做核对，不修改**任何 manifest。发现 diff 后必须由 Ulysses 终审后人工处理。

## 7. 附录 — Q-M-04 上下文

### 7.1 原始疑问（Q-M-04）

> 6 份 manifest（player / economy / match / social / admin / cluster-ops）的 livenessProbe / readinessProbe
> 段是手写而非 Helm template 派生。任何一份 probe 段修改必须同步到其余 5 份。
> 现状：仅抽查 2 份（01-player / 02-economy），不能断言 6 份一致。

### 7.2 父疑问答复（已确认）

> **PH-1 暂不引入 Helm**（per Q-M-04 答复）。
> 改用本 CI 脚本做"结构化 diff + 阈值一致性全 6 份核对"，作为 Helm 引入前的过渡方案。

### 7.3 完成判据

- [x] 脚本 `scripts/verify_probe_consistency.ps1` 存在
- [x] 脚本可独立运行（`pwsh -File scripts/verify_probe_consistency.ps1`）
- [x] 报告 `docs/deploy/probe-consistency-report.md` 存在
- [x] 报告含 6 份 manifest probe 段完整参数表
- [x] CI 接入说明 `docs/deploy/probe-ci-integration.md` 存在
- [x] commit message: `WF-1-55.46: verify_probe_consistency.ps1 + 6 份 manifest 全核对（per OPEN-QA-001 Q-M-04）`

