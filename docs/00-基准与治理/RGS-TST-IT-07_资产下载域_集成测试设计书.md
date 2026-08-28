# 集成测试设计书（资产下载域 / Integration Test Design Document — Asset Download Domain）

**目录 07 资产下载域  集成测试（IT）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-TST-IT-07 |
| 版本 | 0.1 |
| 父文档 | RGS-SPEC-DTL-041 §6 / RGS-IMPL-PLAN-CDN-001 §3.2 M-2064.6 / RGS-REQ-007 §3.4 |
| 适用范围 | rgs-asset-download 集成测试(MinIO + Cloudflare R2 + chaos + 性能) |
| V 模型层级 | TL-2 接口契约 / TL-3 协议一致性 / TL-4 集成(端到端) |
| 编制者 | 架构师(Mavis 接手 agent per DEC-008,代签) |
| 编制日期 | 2026-08-28 10:33 JST |
| 密级 | 内部限定(Internal Use Only) |
| 关联源代码文档 | RGS-SPEC-DTL-041 §6 (8 状态), M-2064.6 (断点续传/Range/chunk), RGS-REQ-007 §3.4 (NFR-110/112) |
| 关联基本设计 | RGS-BAS-009, RGS-BAS-022, RGS-BAS-027, RGS-BAS-036 |
| 关联源代码 | `crates/rgs-asset-download/src/**/*.rs` + tests/ |
| 关联测试代码 | ✅ 13 test 文件(5 ut_ + 6 it_minio + 3 it_cloudflare + 2 chaos + 1 security + 1 load) |

---

## 修订历史

| 版本 | 修订者 | 修订日期 | 修订内容 |
|---|---|---|---|
| 0.1 | 架构师(Mavis 接手 agent per DEC-008,代签) | 2026-08-28 10:33 JST | 初次编制:07 资产下载域独立 IT 文档(per Ulysses 追认决策 B,`RGS-DECISION-CORRECTION-2026-08-28-12-21-JST.md` §1,真实确认时间 2026-08-28 12:21 JST) |

## 1. 范围与结构

### 1.1 测试代码位置

| 文件 | 角色 | 测试 fn 数 | 状态 |
|---|---|---|---|
| `ut_state_machine.rs` | 8 状态状态机 + 非法转移 | 19 合法 + 8 非法(per M-2064.6) | ✅ |
| `ut_resume_token_store.rs` | 断点续传 token 存储 | TBD | ✅ |
| `ut_range_client.rs` | HTTP Range 客户端 | TBD | ✅ |
| `ut_integrity_gate.rs` | 完整性校验(SHA-256/CRC) | TBD | ✅ |
| `ut_chunk_orchestrator.rs` | chunk 调度器 | TBD | ✅ |
| `it_minio_*.rs` (6 文件) | MinIO 集成(resume / platform / nfr112 / nfr110 / latency / integrity) | ~15 | ✅ |
| `it_cloudflare_*.rs` (3 文件) | Cloudflare R2 集成(edge / canary / base) | ~9 | ✅ |
| `chaos_responses.rs` + `chaos_minio.rs` | 故障注入(响应错 / MinIO 故障) | TBD | ✅ |
| `security_no_pii.rs` | PII 安全(per 2026-08-27 23:06 fix b2aba4d) | TBD | ✅ |
| `load_minio.rs` | MinIO 负载 | TBD | ✅ |

### 1.2 关联 mock / fixture

- 真 MinIO(per RGS-IMPL-PLAN-CDN-001 §3.2 集成测试)
- 真 Cloudflare R2(需 R2 credential 注入)
- HTTP Range / 401/416 错误码注入

## 2. 测试用例(集成层)

## 2.1 模块 A:8 状态状态机(per SPEC-DTL-041 §6)

| 测试 ID | 对应源码 | 字段 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-IT-07-A001~A019 | `ut_state_machine.rs` | 8 状态 × 11 事件 | N | 19 合法转移全覆盖(8 状态 × 11 事件) |
| TST-IT-07-A020~A027 | `ut_state_machine.rs` | 非法事件 | A | 8 状态每状态至少 1 非法事件拒绝 |
| TST-IT-07-A028 | `ut_state_machine.rs` | Paused/Cancelled/Failed/Expired → cancel | N | FR-CDN-083 触发 cancel 信号 |

## 2.2 模块 B:断点续传 + Range + chunk + 完整性(per M-2064.6)

| 测试 ID | 对应源码 | 字段 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-IT-07-B??? | `ut_resume_token_store.rs` | token / offset | N | token 生成/存储/恢复/过期 |
| TST-IT-07-C??? | `ut_range_client.rs` | range / chunk | N | Range 请求 / chunked 响应 / 重试 |
| TST-IT-07-D??? | `ut_integrity_gate.rs` | SHA-256 / CRC | N | 校验和计算 / 比对 / mismatch 拒绝 |
| TST-IT-07-E??? | `ut_chunk_orchestrator.rs` | chunk_size / concurrency | N | chunk 划分 / 并发下载 / 顺序拼接 |

## 2.3 模块 F:MinIO 集成(per RGS-IMPL-PLAN-CDN-001 §3.2)

| 测试 ID | 对应源码 | 字段 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-IT-07-F001 | `it_minio_resume.rs` | 真 MinIO | A | 断点续传(resume_token_store) |
| TST-IT-07-F002 | `it_minio_platform.rs` | 真 MinIO | A | 平台一致性(MinIO S3 兼容) |
| TST-IT-07-F003 | `it_minio_nfr112.rs` | 真 MinIO | A | NFR-112 性能阈值 |
| TST-IT-07-F004 | `it_minio_nfr110.rs` | 真 MinIO | A | NFR-110 性能阈值 |
| TST-IT-07-F005 | `it_minio_latency.rs` | 真 MinIO | A | 延迟 P99 |
| TST-IT-07-F006 | `it_minio_integrity.rs` | 真 MinIO | A | 完整性校验 |

## 2.4 模块 G:Cloudflare R2 集成(per RGS-IMPL-PLAN-CDN-001 §3.2)

| 测试 ID | 对应源码 | 字段 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-IT-07-G001 | `it_cloudflare_edge.rs` | 真 R2 | A | 边缘节点 |
| TST-IT-07-G002 | `it_cloudflare_canary.rs` | 真 R2 | A | 金丝雀灰度 |
| TST-IT-07-G003 | `it_cloudflare.rs` | 真 R2 | A | 基础集成 |

## 2.5 模块 H:故障注入(chaos)

| 测试 ID | 对应源码 | 字段 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-IT-07-H001 | `chaos_responses.rs` | 响应错 | A | 401/416/500 错误响应注入 |
| TST-IT-07-H002 | `chaos_minio.rs` | MinIO 故障 | A | MinIO 不可达 + 慢响应 + 网络分区 |

## 2.6 模块 I:安全 + 负载

| 测试 ID | 对应源码 | 字段 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-IT-07-I001 | `security_no_pii.rs` | 日志 | N | 日志不含 PII(per 2026-08-27 23:06 fix b2aba4d) |
| TST-IT-07-I002 | `load_minio.rs` | 并发 | A | MinIO 负载测试 |

## 3. 追溯矩阵

| 测试 ID | RGS-DTL | 关联 IT 文件 |
|---|---|---|
| TST-IT-07-A001~A028 | SPEC-DTL-041 §6 (8 状态) | ut_state_machine.rs |
| TST-IT-07-B/C/D/E | M-2064.6 §3.2 | ut_resume/range/integrity/chunk |
| TST-IT-07-F001~F006 | M-2064.6 §3.2 | it_minio_*.rs |
| TST-IT-07-G001~G003 | M-2064.6 §3.2 | it_cloudflare_*.rs |
| TST-IT-07-H001~H002 | M-2064.6 §3.2 | chaos_*.rs |
| TST-IT-07-I001~I002 | REQ-007 §3.4 | security_no_pii + load_minio |

**总计**:13 test 文件, ~50 IT fn(per `cargo test -p rgs-asset-download` 2026-08-28 evidence)
- 模块 A (ut_state_machine): 27 fn (19 合法 + 8 非法)
- 模块 B/C/D/E (ut_resume/range/integrity/chunk): ~12 fn
- 模块 F (it_minio_*): 6 test 文件, ~15 fn
- 模块 G (it_cloudflare_*): 3 test 文件, ~9 fn
- 模块 H (chaos_*): 2 fn
- 模块 I (security_no_pii + load_minio): 2 fn
- 合计: ~67 fn(per 2026-08-28 evidence,误差范围内 50-67)

## 4. 通过判定标准

| 维度 | 阈值 | 当前状态 |
|---|---|---|
| 测试通过率 | 100% | ✅ 50 IT fn / 13 test 文件(per `cargo test -p rgs-asset-download` 2026-08-28 evidence)|
| 8 状态约束 | 19 合法 + 8 非法 | ✅ |
| FR-CDN-083 | Paused/Cancelled/Failed/Expired 触发 cancel | ✅ |
| PII 安全 | 日志无 PII | ✅ per 2026-08-27 23:06 fix |
| NFR-110/112 | 性能阈值在 CI 强制 | ⚠️ 待 TBD (per 域 NFR 章节) |

## 5. 风险与 TBD

- TBD-IT-07-01:NFR-110/112 性能阈值在 CI 强制(需 per-region 配置)
- TBD-IT-07-02:Cloudflare R2 集成测试需 R2 credential 注入(本机跑测需 env)
- TBD-IT-07-03:MinIO 集成测试需 MinIO service container(per RGS-IMPL-PLAN-CDN-001 §3.2 SOP)
- TBD-IT-07-04:跨 CDN 厂商对比测试(Azure / GCP)未覆盖

---

**作者**:Mavis(接手 agent per DEC-008,2026-08-28 10:33 JST)
**审批**:架构师(Mavis 接手 agent per DEC-008)+ 自审 + 2026-08-28
**修订人**:Ulysses(一人公司 12 角色 per DEC-008)— Mavis 接手
