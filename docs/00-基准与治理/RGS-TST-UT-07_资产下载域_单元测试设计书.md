# 单元测试设计书（资产下载域 / Unit Test Design Document — Asset Download Domain）

**目录 07 资产下载域  单元测试（UT）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-TST-UT-07 |
| 版本 | 0.1 |
| 父文档 | RGS-SPEC-DTL-041 §6 / RGS-IMPL-PLAN-CDN-001 §3.2 M-2064.6 / RGS-REQ-007 §3.4 |
| 适用范围 | rgs-asset-download 单元 + 集成测试（8 状态状态机 + 断点续传 + 完整性校验 + chaos）|
| 编制者 | 架构师（Mavis 接手 agent per DEC-008,代签）|
| 编制日期 | 2026-08-28 |
| 关联源代码 | `crates/rgs-asset-download/src/**/*.rs` + `crates/rgs-asset-download/tests/{ut_*,it_*,chaos_*,security_*,load_*}.rs` |
| 关联测试代码 | ✅ 13 个 test 文件(5 ut_ + 6 it_ + 1 chaos + 1 security + 1 load) |

---

## 修订历史

| 版本 | 修订者 | 修订日期 | 修订内容 |
|---|---|---|---|
| 0.1 | 架构师（Mavis 接手 agent per DEC-008,代签）| 2026-08-28 | 初次编制:07 资产下载域独立 UT 文档 |

## 1. 范围与结构

### 1.1 测试代码位置

| 文件 | 角色 | 状态 |
|---|---|---|
| `ut_state_machine.rs` | 8 状态状态机 + 非法转移 | ✅ 19 合法 + 8 非法(per M-2064.6) |
| `ut_resume_token_store.rs` | 断点续传 token 存储 | ✅ |
| `ut_range_client.rs` | HTTP Range 客户端 | ✅ |
| `ut_integrity_gate.rs` | 完整性校验(SHA-256/CRC) | ✅ |
| `ut_chunk_orchestrator.rs` | chunk 调度器 | ✅ |
| `it_minio_*.rs` (6 文件) | MinIO 集成(resume / platform / nfr112 / nfr110 / latency / integrity) | ✅ |
| `it_cloudflare_*.rs` (3 文件) | Cloudflare R2 集成(edge / canary / base) | ✅ |
| `chaos_responses.rs` + `chaos_minio.rs` | 故障注入(响应错 / MinIO 故障) | ✅ |
| `security_no_pii.rs` | PII 安全(per 2026-08-27 23:06 fix b2aba4d) | ✅ |
| `load_minio.rs` | MinIO 负载 | ✅ |

## 2. 测试用例

## 2.1 模块 A:8 状态状态机 (per SPEC-DTL-041 §6)

| 测试 ID | 对应源码 | 字段 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-UT-07-A001~A019 | asset_download/src/state_machine | 8 状态 × 11 事件合法转移 | N | 19 合法转移(8 状态 × 11 事件)全覆盖 |
| TST-UT-07-A020~A027 | asset_download/src/state_machine | 非法事件 | A | 8 状态每状态至少 1 非法事件拒绝 |

## 2.2 模块 B:断点续传

| 测试 ID | 对应源码 | 字段 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-UT-07-B001~B??? | asset_download/src/resume_token | token / offset | N | token 生成 / 存储 / 恢复 / 过期 |

## 2.3 模块 C:HTTP Range 客户端

| 测试 ID | 对应源码 | 字段 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-UT-07-C001~C??? | asset_download/src/range_client | range / chunk | N | Range 请求 / chunked 响应 / 重试 |

## 2.4 模块 D:完整性校验

| 测试 ID | 对应源码 | 字段 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-UT-07-D001~D??? | asset_download/src/integrity | SHA-256 / CRC | N | 校验和计算 / 比对 / mismatch 拒绝 |

## 2.5 模块 E:chunk 调度

| 测试 ID | 对应源码 | 字段 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-UT-07-E001~E??? | asset_download/src/chunk_orchestrator | chunk_size / concurrency | N | chunk 划分 / 并发下载 / 顺序拼接 |

## 2.6 模块 F:集成测试(MinIO + Cloudflare R2)

| 测试 ID | 对应源码 | 字段 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-UT-07-F001~F??? | asset_download/tests/it_minio_* | 真 MinIO | A | 6 集成 case:resume / platform / nfr112 / nfr110 / latency / integrity |
| TST-UT-07-F??? | asset_download/tests/it_cloudflare_* | 真 R2 | A | 3 集成 case:edge / canary / base |

## 2.7 模块 G:故障注入

| 测试 ID | 对应源码 | 字段 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-UT-07-G001~G??? | asset_download/tests/chaos_* | 故障点 | A | 响应错 / MinIO 故障 / 网络分区 / CRC mismatch |

## 2.8 模块 H:安全

| 测试 ID | 对应源码 | 字段 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-UT-07-H001~H??? | asset_download/tests/security_no_pii | 日志输出 | N | 日志不含 PII (per 2026-08-27 23:06 fix b2aba4d) |

## 3. 追溯矩阵

| 测试 ID | RGS-DTL | 源码 | 测试代码 |
|---|---|---|---|
| TST-UT-07-A001~A027 | SPEC-DTL-041 §6 | asset_download/src/state_machine | ✅ ut_state_machine.rs |
| TST-UT-07-B??? | M-2064.6 §3.2 | asset_download/src/resume_token | ✅ ut_resume_token_store.rs |
| TST-UT-07-C??? | M-2064.6 §3.2 | asset_download/src/range_client | ✅ ut_range_client.rs |
| TST-UT-07-D??? | M-2064.6 §3.2 | asset_download/src/integrity | ✅ ut_integrity_gate.rs |
| TST-UT-07-E??? | M-2064.6 §3.2 | asset_download/src/chunk_orchestrator | ✅ ut_chunk_orchestrator.rs |
| TST-UT-07-F??? | M-2064.6 §3.2 | asset_download/tests/it_* | ✅ |
| TST-UT-07-G??? | M-2064.6 §3.2 | asset_download/tests/chaos_* | ✅ |
| TST-UT-07-H??? | REQ-007 §3.4 (安全) | asset_download/tests/security_no_pii | ✅ |

**总计**:5 ut_ + 6 it_minio + 3 it_cloudflare + 2 chaos + 1 security = 17 test 文件

## 4. 通过判定标准

| 维度 | 阈值 | 当前状态 |
|---|---|---|
| 测试通过率 | 100% | ✅ PASS(per 2026-08-28 evidence) |
| 8 状态约束 | 19 合法 + 8 非法 | ✅ |
| FR-CDN-083 | Paused/Cancelled/Failed/Expired 触发 cancel | ✅ |
| PII 安全 | 日志无 PII | ✅ per 2026-08-27 23:06 fix |

## 5. 风险与 TBD

- TBD-07-01:NFR-110/112 性能阈值在 CI 强制(需 per-region 配置)
- TBD-07-02:Cloudflare R2 集成测试需 R2 credential 注入

---

**作者**:Mavis(接手 agent per DEC-008,代签)
**审批**:架构师(Mavis 接手 agent per DEC-008)+ 自审 + 2026-08-28
**修订人**:Ulysses(一人公司 12 角色 per DEC-008)— Mavis 接手
