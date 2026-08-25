# 断点续传实施计划（CDN 资源分发扩展 · 客户端可恢复下载）

**RGS-IMPL-PLAN-CDN-001**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-IMPL-PLAN-CDN-001 |
| 版本 | 0.1（初版，per 主对话 2026-08-25 12:13 孤儿 SPEC 治理延伸）|
| 状态 | 草案；待架构师（兼）签字 + Platform Engineer 兼复审后升 v0.2 |
| 制定日 | 2026-08-25 |
| 制定者 | Ulysses（架构师兼 / Platform Engineer 兼 per DEC-008）|
| 适用范围 | 客户端 SDK 层的 CDN 资源断点续传与可恢复下载；服务端只承担既有 manifest / 签名 / 灰度判定（per RGS-DTL-007）|
| 关联 | SPEC-DTL-041（实现规格 v0.2）+ DTL-041 + ARC-045 + WBS L4 #2063/#2064/#2065/#2069/#2072 + RGS-REQ-004 §3.7（AC-CDN-110~118）|
| OLU 框架 | RGS-TS-001 v0.6 §6.2 token-OLU（1 人·天 ≈ 100K-300K tokens）|
| 一人公司兼任 | per DEC-008，架构师 = Platform Engineer = SRE 兼；本计划 owner 即"架构师（兼）"|

---

## 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-25 | Ulysses（架构师兼）| 首版：拆解 WBS §16.2 L4 #2063~#2065/#2069/#2072 → 5 阶段 L4 子任务 + 18 个 M 任务 + token-OLU 估算 + 风险表 + 回滚策略 |

---

## 1. 目标 & 范围

### 1.1 目标

实现一个**纯客户端 SDK**层的可恢复下载能力：

- **4 平台可用**：iOS 17 / Android 14 / Windows 11 / macOS 14
- **不绕过 `rgs-asset-update` 的 `IntegrityGate`**（NFR-CDN-002 硬约束）
- **支持 HTTP Range** 后端（自托管 MinIO + 商业 CDN 可选对照）
- **断点续传**：进程崩溃/网络断开/手动暂停/手动取消 后，可从最近 checkpoint 恢复
- **整文件校验闸门**：下载完成后整文件 hash 校验通过才能切到 `asset-update` 灰度
- **不持有服务端凭证**（FR-CDN-001 既有）
- **断点记录不含 PII**（FR-CDN-064）

### 1.2 非目标（per SPEC-DTL-041 §1 + DTL-041）

- ❌ **不**实现服务端 Range 协议栈（用现成 MinIO/Cloudflare）
- ❌ **不**实现 manifest 拉取 / 签名校验 / 灰度判定（前置能力归 `rgs-asset-update`）
- ❌ **不**改 P2P / BT / WebRTC（仅 HTTP/1.1 + 可选 HTTP/2）
- ❌ **不**实现 QUIC（quinn 0.10+ 是目标基线，但 v0.1 阶段只用 reqwest 0.12 + HTTP Range，QUIC 待 PH-5 选型）

### 1.3 关键硬约束（per SPEC-DTL-041 §3 + §5 + §7 + §8）

| 编号 | 内容 | 类型 |
|---|---|---|
| NFR-CDN-002 | 整文件校验不可绕过 | 硬约束（代码评审 grep）|
| FR-CDN-001 | 公开资源清单 / patch 走匿名访问 | 既有 |
| FR-CDN-064 | 断点记录不含 PII 字段 | 既有（代码评审 grep）|
| FR-CDN-074 | 用 `If-Range: <ETag>` 不用 `Last-Modified` | 既有 |
| FR-CDN-083 | 暂停时必须取消 in_flight Range 请求 | 既有（代码评审 grep `cancel_request`）|
| NFR-CDN-114 | DistributionBackend 必须支持 HTTP Range | 门禁 |
| NFR-CDN-110/112 | 恢复时延 p99 < 500ms / 恶化阈值 ≤ 20% | 实测 |
| AC-CDN-110~118 | 9 项验收门槛 | 实测 |
| TBD-CDN-201/202/203 | 断点过期阈值 / 并发粒度 / LRU 上限 | PH-3 实测 |

---

## 2. 现有依赖与拟新增结构

### 2.1 现有依赖（**不变**）

- `rgs-asset-update`（前置，**只**依赖其 manifest API；**不**反向依赖）
- `rgs-version`（版本协商，被 SDK 引用）
- `rgs-network`（HTTP 客户端基座）
- 5 域 manifest 服务（per 既有 `rgs-asset-update` 接口）
- 自托管 MinIO（生产默认后端）
- Cloudflare（可选商业 CDN，对照组）

### 2.2 拟新增 crate：`crates/rgs-asset-download/`

```
crates/rgs-asset-download/
├── Cargo.toml                          # 独立 crate；workspace member 显式登记
├── build.rs                            # 若需 tonic-build（v0.1 不需要，预留）
├── src/
│   ├── lib.rs                          # crate 入口；re-export 公开 API
│   ├── api.rs                          # download_asset / pause_download / cancel_download / get_download_state
│   ├── state_machine.rs                # DownloadStateMachine（8 状态 + 转移表）
│   ├── resume_token.rs                 # ResumeToken 结构（13 字段 per SPEC §6）
│   ├── resume_token_store.rs           # ResumeTokenStore trait + SqliteResumeTokenStore + JsonFileResumeTokenStore
│   ├── range_client.rs                 # RangeClient（HTTP/1.1 RFC 7233 HEAD/Range；206/416/200/429 全部响应路径）
│   ├── chunk_orchestrator.rs           # ChunkOrchestrator（并发分片调度；桌面 ≤ 16 路 / 移动 ≤ 4 路）
│   ├── integrity_gate.rs               # IntegrityGate（整文件 hash 校验；NFR-CDN-002 硬约束）
│   ├── error.rs                        # 错误码（per DTL §6，不自创）
│   ├── metrics.rs                      # 10 项 rgs_asset_download_* 指标
│   ├── config.rs                       # 并发数 / LRU 上限 / 断点过期阈值（PH-3 实测填入）
│   └── platform/
│       ├── mod.rs                      # 4 平台分支（per target_os 编译期选择）
│       ├── unix.rs                     # Linux/macOS sparse file 预分配（fallocate / posix_fallocate）
│       ├── windows.rs                  # Windows sparse file + SetFileValidData 权限评估
│       ├── android.rs                  # Android sparse file（应用沙箱目录）
│       └── ios.rs                      # iOS sparse file（应用沙箱目录）
├── tests/
│   ├── ut_state_machine.rs             # 8 状态转移 UT（per SPEC §6 50 条 UT 拆分）
│   ├── ut_resume_token_store.rs        # 13 字段 + 原子写 + LRU UT
│   ├── ut_range_client.rs              # HEAD + Range 全状态码 UT
│   ├── ut_chunk_orchestrator.rs        # 并发 + 暂停取消 UT
│   ├── ut_integrity_gate.rs            # 整文件 hash UT（含篡改负例）
│   ├── it_minio.rs                     # MinIO 集成测试（AC-CDN-110~113）
│   ├── it_cloudflare.rs                # Cloudflare 集成测试（AC-CDN-114~118，仅 PH-5）
│   └── security_no_pii.rs              # 断点记录 grep 验证 PII 字段为空
├── migrations/
│   └── 0001_resume_token_index.sql     # SQLite 表 + 索引（store 用）
└── README.md                           # 公开 API + 集成步骤
```

### 2.3 Cargo.toml 关键声明

```toml
[package]
name = "rgs-asset-download"
version = "0.1.0"
edition = "2021"
rust-version = "1.98"
license = "Apache-2.0"

[dependencies]
# 既有（per RGS-IMPL-001 §2 workspace）
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "1"
tracing = "0.1"
async-trait = "0.1"

# 既有 shared-platform re-export
# （路径由 workspace 统一管理；本 crate 不直接依赖 rgs-asset-update）

# 新增（按 DTL §12 / SPEC §2）
reqwest = { version = "0.12", default-features = false, features = ["rustls-tls", "stream"] }
rusqlite = { version = "0.31", features = ["bundled"] }     # 客户端侧 SQLite（不连 PG）
sha2 = "0.10"                                              # 整文件 hash
hex = "0.4"
url = "2"
dirs = "5"                                                 # ~/.rgs-sdk/downloads/

[target.'cfg(windows)'.dependencies]
windows-sys = { version = "0.52", features = ["Win32_Storage_FileSystem", "Win32_Foundation"] }

[dev-dependencies]
tokio = { version = "1", features = ["full", "test-util"] }
wiremock = "0.6"                                           # mock Range server
tempfile = "3"
proptest = "1"                                             # 状态机 fuzz
```

---

## 3. L4 任务拆解（refines WBS §16.2 #2063/#2064/#2065/#2069/#2072）

> **WBS L4 任务** → **M 任务（可执行级）**逐条拆细。每条 M 任务可独立 worktree 化（per RGS-WT-001 §11）。

### 3.1 L4 #2063 → rgs-asset-download crate 骨架（PH-3 第 7-9 周）

| M # | 任务 | 文件 | token-OLU | 前置 |
|---|---|---|---|---|
| M-2063.1 | Cargo.toml + workspace member 登记 | `crates/rgs-asset-download/Cargo.toml` | 30K | — |
| M-2063.2 | lib.rs 骨架 + 模块目录 + 4 平台 platform/mod.rs | `src/lib.rs` + `src/platform/*.rs` | 80K | M-2063.1 |
| M-2063.3 | 公开 API trait 定义（`download_asset` / `pause_download` / `cancel_download` / `get_download_state`）| `src/api.rs` | 60K | M-2063.2 |
| M-2063.4 | 错误码定义（per DTL §6，不自创）| `src/error.rs` | 40K | M-2063.3 |
| M-2063.5 | config.rs 占位（并发数 / LRU / 断点过期 → PH-3 实测填）| `src/config.rs` | 20K | M-2063.2 |

**L4 #2063 合计**：~230K tokens ≈ 0.8-2.3 人·天

### 3.2 L4 #2064 → DownloadStateMachine + ResumeTokenStore（PH-3 第 7-9 周）

| M # | 任务 | 文件 | token-OLU | 前置 |
|---|---|---|---|---|
| M-2064.1 | 8 状态枚举（Idle / Resolving / Downloading / Paused / Completed / Failed / Cancelled / Expired）+ 转移表 | `src/state_machine.rs` | 100K | M-2063.3 |
| M-2064.2 | ResumeToken 结构（13 字段 per SPEC §6：token_id / asset_id / file_path / total_size / chunk_size / completed_chunks / etag / etc.）| `src/resume_token.rs` | 60K | M-2063.2 |
| M-2064.3 | ResumeTokenStore trait（put / get / delete / list / cleanup_expired）| `src/resume_token_store.rs` | 50K | M-2064.2 |
| M-2064.4 | JsonFileResumeTokenStore 实现（原子写 = tmp + rename + SQLite index）| `src/resume_token_store.rs` | 120K | M-2064.3 |
| M-2064.5 | SqliteResumeTokenStore 实现（LRU 100MB 上限）| `src/resume_token_store.rs` | 100K | M-2064.3 |
| M-2064.6 | UT: 状态机 8 状态转移 + 非法转移负例（per SPEC §6 50 条 UT 拆分）| `tests/ut_state_machine.rs` | 80K | M-2064.1 |
| M-2064.7 | UT: ResumeTokenStore 13 字段 + 原子写 + LRU | `tests/ut_resume_token_store.rs` | 60K | M-2064.4 + M-2064.5 |

**L4 #2064 合计**：~570K tokens ≈ 1.9-5.7 人·天

### 3.3 L4 #2065 → RangeClient + ChunkOrchestrator + IntegrityGate（PH-3 第 7-9 周）

| M # | 任务 | 文件 | token-OLU | 前置 |
|---|---|---|---|---|
| M-2065.1 | RangeClient（HTTP/1.1 RFC 7233 HEAD + Range；206/416/200/429 全部响应路径）| `src/range_client.rs` | 200K | M-2063.3 |
| M-2065.2 | If-Range ETag 强制（per FR-CDN-074，不接受 Last-Modified）| `src/range_client.rs` | 40K | M-2065.1 |
| M-2065.3 | ChunkOrchestrator（并发分片调度；桌面 ≤ 16 / 移动 ≤ 4；背压重试）| `src/chunk_orchestrator.rs` | 250K | M-2065.1 |
| M-2065.4 | 暂停 / 取消信号（取消 in_flight reqwest，per FR-CDN-083）| `src/chunk_orchestrator.rs` | 60K | M-2065.3 |
| M-2065.5 | IntegrityGate（整文件 SHA-256；分块到达**不**做单独校验，per NFR-CDN-002）| `src/integrity_gate.rs` | 100K | M-2063.2 |
| M-2065.6 | 4 平台 sparse file 预分配（unix/windows/android/ios）| `src/platform/*.rs` | 200K | M-2065.5 |
| M-2065.7 | Windows SetFileValidData 权限评估 + 降级路径 | `src/platform/windows.rs` | 80K | M-2065.6 |
| M-2065.8 | 10 项 metrics（rgs_asset_download_*）| `src/metrics.rs` | 80K | M-2065.3 + M-2065.5 |
| M-2065.9 | UT: RangeClient HEAD + Range 全状态码 | `tests/ut_range_client.rs` | 80K | M-2065.1 |
| M-2065.10 | UT: ChunkOrchestrator 并发 + 暂停取消 | `tests/ut_chunk_orchestrator.rs` | 100K | M-2065.3 + M-2065.4 |
| M-2065.11 | UT: IntegrityGate 整文件 hash + 篡改负例 | `tests/ut_integrity_gate.rs` | 60K | M-2065.5 |
| M-2065.12 | Security UT: 断点记录 grep 验证 PII 字段为空（per FR-CDN-064）| `tests/security_no_pii.rs` | 30K | M-2064.5 |

**L4 #2065 合计**：~1.28M tokens ≈ 4.3-12.8 人·天

### 3.4 L4 #2069 → MinIO 自托管 Range 行为实测（PH-4 第 9-12 周）

| M # | 任务 | token-OLU | 前置 |
|---|---|---|---|
| M-2069.1 | MinIO 单节点 docker-compose 起服 + Range HEAD/Range 端到端跑通 | 60K | M-2065.11 |
| M-2069.2 | AC-CDN-110（断点续传恢复时延 p99 < 500ms）实测 1000 资源 × 4 平台 | 150K | M-2069.1 |
| M-2069.3 | AC-CDN-111（整文件校验闸门 + 篡改负例）实测 | 80K | M-2069.1 |
| M-2069.4 | AC-CDN-112（暂停/取消中途 + 重启后从 checkpoint 恢复）实测 | 100K | M-2069.1 |
| M-2069.5 | AC-CDN-113（4 平台 pre-allocate 权限 + 性能）实测 | 100K | M-2069.1 |
| M-2069.6 | NFR-CDN-110（恢复时延 p99 < 500ms）实测 | 60K | M-2069.2 |
| M-2069.7 | NFR-CDN-112（恶化阈值 ≤ 20%）实测（对比不开断点续传）| 80K | M-2069.2 |
| M-2069.8 | 故障注入 5 类：断网 / kill -9 / ETag 变更 / 篡改 / 强制更新 | 100K | M-2069.4 |
| M-2069.9 | 100 万级 chunk 落盘 + GB 级文件并发分片吞吐（Load）| 80K | M-2069.1 |
| M-2069.10 | 服务端 5 类响应（206/416/200/429/503）随机注入（Chaos）| 80K | M-2069.1 |

**L4 #2069 合计**：~890K tokens ≈ 3.0-8.9 人·天

### 3.5 L4 #2072 → Cloudflare 商业 CDN 边缘集成（PH-5 第 12-14 周，可选）

| M # | 任务 | token-OLU | 前置 |
|---|---|---|---|
| M-2072.1 | Cloudflare R2 bucket 创建 + Range endpoint 配置 | 30K | M-2069.10 |
| M-2072.2 | 边缘命中实测（多 region）| 60K | M-2072.1 |
| M-2072.3 | 切流验证（5% → 25% → 100%）| 80K | M-2072.2 |
| M-2072.4 | 商业 CDN vs 自托管 MinIO 对比报告 | 40K | M-2072.3 + M-2069.7 |

**L4 #2072 合计**：~210K tokens ≈ 0.7-2.1 人·天

> **注**：L4 #2072 是 PH-5 可选项；如商业 CDN 选型未通过 NFR-CDN-114，可整体跳过。

### 3.6 L4 汇总 + token-OLU 估算

| L4 | 范围 | token-OLU（人·天）| PH | 窗口 |
|---|---|---|---|---|
| #2063 | crate 骨架 | 230K（0.8-2.3）| PH-3 | W7-W9 |
| #2064 | StateMachine + TokenStore | 570K（1.9-5.7）| PH-3 | W7-W9 |
| #2065 | RangeClient + Orchestrator + IntegrityGate | 1.28M（4.3-12.8）| PH-3 | W7-W9 |
| #2069 | MinIO 端到端实测 | 890K（3.0-8.9）| PH-4 | W9-W12 |
| #2072 | Cloudflare 商业 CDN（可选）| 210K（0.7-2.1）| PH-5 | W12-W14 |
| **小计（不含 2072）** | — | **2.97M（10-30）** | PH-3~4 | W7-W12 |
| **小计（含 2072）** | — | **3.18M（10.7-32）** | PH-3~5 | W7-W14 |

**对照 NFR-OP-010（1 SRE ≤ 1 人·周 ≈ 1M tokens）**：
- 架构师兼 SRE = 0.5 SRE（1 人公司分摊 50% SRE 容量）
- 单 SRE 上限 0.5 人·周 = ~500K tokens / 周
- 3M tokens / 500K = **6 周**净工作时间
- PH-3（W7-W9 3 周）+ PH-4（W9-W12 3 周）= 6 周 → **刚好压在 1 SRE 半容量上限** ✓
- 含 PH-5 Cloudflare 再 +0.5 周 → 总 6.5 周（W7-W13.5）

### 3.7 隐含 L4 任务（暂不显式登记）

| M | 任务 | token-OLU |
|---|---|---|
| M-EXTRA.1 | 工作量评估 + RACI 复审 + SRE 接力 | 50K |
| M-EXTRA.2 | SPEC-DTL-041 状态字段升 `规格草案，待 RGS-DTL-041 具名 DD Review` → `实施中`（per RGS-SPEC-000 §1 状态机）| 5K |
| M-EXTRA.3 | 5 域 README + 1 域 SDK 示例 | 80K |
| M-EXTRA.4 | `rgs-arc-olu` 上报 OLU（per NFR-LCM-007 配套）| 20K |
| **小计** | | **155K** |

---

## 4. 文件交付物清单（每 L4 完成时同步提交）

| 类别 | 文件 | L4 |
|---|---|---|
| 代码 | `crates/rgs-asset-download/Cargo.toml` + `src/*.rs` + `tests/*.rs` | #2063~#2065 |
| 迁移 | `crates/rgs-asset-download/migrations/0001_resume_token_index.sql` | #2064 |
| 工作树 | `.wbs-task-marker` (per RGS-WT-001 §11.3) | #2063~#2065 / #2069 / #2072 |
| 测试报告 | `docs/deploy/cdn-it-report.md` + `docs/deploy/cdn-st-report.md` + `docs/deploy/cdn-load-report.md` | #2069 / #2072 |
| 文档 | `crates/rgs-asset-download/README.md` + `RGS-REQ-004 §3.7` 验收项回填 | #2063 + #2069 |
| ADR | 无（不引入新 ADR；遵循 FR-CDN-001/064/074/083 + NFR-CDN-002/110/112/114 既有）| — |

---

## 5. 验收门槛

### 5.1 必过（per SPEC-DTL-041 §7）

- [ ] RGS-DTL-041 源 DTL 的 TBD（TBD-CDN-201/202/203）有批准处置或纳入 PH-3 实测
- [ ] Cargo fmt / clippy / test / deny / schema / secret / high-cardinality 全过
- [ ] 4 平台 pre-allocate 实测通过（macOS 14 / Windows 11 / iOS 17 / Android 14）
- [ ] AC-CDN-110~118 全部 9 项达标
- [ ] NFR-CDN-110~114 全部 5 项达标
- [ ] 当前无实现文件时保持"待实现/待评审"状态（per §7 第 7 条）—— 本计划实施前 `Test-Path crates/rgs-asset-download` = False，实施后 = True 且 ≥ 1 个非空 .rs 文件

### 5.2 实测参数回填（per SPEC §8）

| 参数 | 目标 | 实测位置 |
|---|---|---|
| 断点过期阈值 | 7 天 | `config.rs` `resume_token_ttl_days` |
| 并发分片粒度 | 4~16 MB | `config.rs` `chunk_size_bytes` |
| LRU 上限 | 100 MB | `config.rs` `lru_max_bytes` |
| 恢复时延 p99 | < 500 ms | M-2069.6 |
| 恶化阈值 | ≤ 20% | M-2069.7 |
| 4 平台 SetFileValidData 权限 | Windows 需 SeManageVolumePrivilege 评估 | M-2065.7 |

### 5.3 必须 grep 的 3 处代码评审检查

```bash
# 1. NFR-CDN-002: 整文件校验不可绕过
Select-String -Path crates/rgs-asset-download/src -Pattern "skip_integrity|bypass_integrity" -List
# 期望：空（无绕过标记）

# 2. FR-CDN-064: 断点记录不含 PII
Select-String -Path crates/rgs-asset-download/src/resume_token.rs -Pattern "player_id|device_id|ip|mac|email" -List
# 期望：空

# 3. FR-CDN-083: 暂停时必须取消 in_flight
Select-String -Path crates/rgs-asset-download/src/chunk_orchestrator.rs -Pattern "cancel_request|abort_request" -List
# 期望：≥ 1 处
```

---

## 6. 风险 & 缓解

| # | 风险 | 等级 | 缓解 |
|---|---|---|---|
| R1 | quinn 0.10+ QUIC 协议栈集成复杂度（v0.1 不集成，但 SPEC §12 列出）| 中 | v0.1 阶段只用 reqwest 0.12 HTTP/1.1 + Range；QUIC 列为 PH-5+ 远期 |
| R2 | Windows SetFileValidData 需要 `SeManageVolumePrivilege`（普通用户无）| 中 | 实测降级路径（fallback 到 `SetEndOfFile` + 显式填充 0），M-2065.7 强制覆盖 |
| R3 | iOS / Android 沙箱目录限制（应用被 kill 后 sandbox 目录仍在，但 token_id 需重新生成）| 低 | 启动时 `ResumeTokenStore::cleanup_expired()` 清理 7 天前；token_id 与 app session 绑定（不与 player_id 绑定，per FR-CDN-064）|
| R4 | MinIO 默认配置对 Range 请求的 size 限制（单 chunk ≤ 5GiB）| 低 | M-2065.3 `chunk_orchestrator` 单 chunk 默认 8MB（远小于 5GiB）|
| R5 | 整文件 SHA-256 计算阻塞 UI（GB 级文件 30s+）| 中 | 用 `tokio::task::spawn_blocking` 异步执行；UI 显示进度（per metrics `rgs_asset_download_integrity_duration_seconds`）|
| R6 | 商业 CDN（Cloudflare R2 / AWS S3 / 阿里 OSS）Range 行为差异 | 低 | NFR-CDN-114 门禁：未通过 AC-CDN-117 测试的候选**不得**启用 |
| R7 | 断点记录被外部进程读取（无 PII 但 token_id 可关联）| 低 | token_id 用 UUID v4 随机生成；不含文件路径以外的元数据；store 大小上限 100MB（LRU）|

---

## 7. 回滚策略

### 7.1 应用回滚

`rgs-asset-download` **不**是必选路径（服务端既有 `rgs-asset-update` 全量下载保留）。如 Range 协议在生产环境出现回归：

1. 客户端侧：通过 `rgs-version` 协商降级到 v0（无断点续传，全量重传）
2. 服务端侧：5 域 `manifest.json` 移除 `supports_resume: true` 字段
3. 监控：NFR-CDN-114 门禁自动告警

### 7.2 数据回滚

- **断点记录** = 客户端本地 SQLite + JSON 文件；删除即可，无服务端状态
- **Migration 回滚** = `sqlite3 downloads.db < migrations/0001_resume_token_index.sql` reverse（迁移本身是 ADD INDEX，可 DROP INDEX）

### 7.3 配置回滚

`config.rs` 各字段默认值保守：
- `chunk_size_bytes = 8 * 1024 * 1024`（8MB，可在 4MB~16MB 调）
- `lru_max_bytes = 100 * 1024 * 1024`（100MB）
- `resume_token_ttl_days = 7`

---

## 8. 一人公司 RACI 简表（per RGS-ADR-0055 §4）

| 决策 | R（执行）| A（最终批准）| C（咨询）| I（知会）|
|---|---|---|---|---|
| **代码合并**（#2063~#2065）| AI worker 子代理 | **Ulysses（架构师兼）** 显式 PR merge | CI 4 workflow + OTel | 全员（git log）|
| **DTL 升版**（SPEC-DTL-041 v0.2→v0.3）| AI worker | **Ulysses** 显式签字 | 架构师兼 + Platform Engineer 兼 | 全员 |
| **PH-4 实测启动**（#2069）| AI worker | **Ulysses（SRE 兼）** 显式 `/sign` | Platform Engineer 兼 | DBA 兼 + QA 兼 |
| **PH-5 商业 CDN 选型**（#2072）| AI worker | **Ulysses（SRE 兼 + 架构师兼）** 显式 `/sign` | Platform Engineer 兼 | 全部 5 域 Lead 兼 |

---

## 9. 关联文档

- 上行：
  - [SPEC-DTL-041 实现规格书 v0.2](../13-实现规格/RGS-SPEC-DTL-041_实现规格书.md)
  - [RGS-DTL-041 详细设计](../01-核心架构与设计模式/RGS-DTL-031_集群运营管理_每功能原子升级_详细设计.md)（DTL-031 的衍生，按 ARC-051 Feature 注册）
  - [ARC-045 客户端资源分发与热更新](../00-基本与治理/)
  - [WBS-001 §16.2 L4 #2063/#2064/#2065/#2069/#2072](RGS-WBS-001_瀑布式工作分解结构_v0.3.md)
  - [RGS-PLAN-001 v1.1 项目实施计划](RGS-PLAN-001_项目实施计划_v1.0.md)
  - [RGS-TS-001 v0.6 §6.2 token-OLU](../10-技术选型/RGS-TS-001_主要技术选型报告.md)
  - [RGS-ADR-0055 v0.1 DEC-005/008 兼容论证 + RACI 简表](../08-架构决策记录/RGS-ADR-0055_DEC-005_008_兼容论证_v0.1.md)
  - [RGS-IMPL-001 实施约定与工程边界](../13-实现规格/RGS-IMPL-001_实施约定与工程边界.md)
  - [RGS-ANTIPATTERN-001 孤儿 SPEC 自查清单 v0.1](RGS-ANTIPATTERN-001_孤儿SPEC自查清单_v0.1.md)
- 下行：
  - `crates/rgs-asset-download/README.md`（实施时新建）
  - `docs/deploy/cdn-it-report.md` + `cdn-st-report.md` + `cdn-load-report.md`（实测时新建）
  - `crates/rgs-asset-download/migrations/0001_resume_token_index.sql`（实施时新建）

---

## 10. 审批栏（per DEC-008 一人公司 12 角色兼任）

| # | 角色 | 姓名 | 审批日 | 结论/条件 |
|---|---|---|---|---|
| 1 | 架构负责人 | **Ulysses**（架构师兼 per DEC-008）| _pending_ | ✅ 草案待签字 |
| 2 | Platform Engineer | **Ulysses**（Platform 兼 per DEC-008）| _pending_ | ✅ 草案待签字 |
| 3 | SRE Lead | **Ulysses**（SRE 兼 per DEC-008）| _pending_ | ✅ 草案待签字 |
| 4 | QA Lead | **Ulysses**（QA 兼 per DEC-008）| _pending_ | ✅ 草案待签字 |
| 5 | 安全负责人 | **Ulysses**（安全兼 per DEC-008）| _pending_ | ✅ 草案待签字（FR-CDN-064 / NFR-CDN-002 硬约束）|
| 6 | PM/项目负责人 | **Ulysses**（PM 兼 per DEC-008）| _pending_ | ✅ 草案待签字（含 token-OLU 估算批准）|

> **本计划升 v0.2 条件**：6 角色签字 + 1 次 worktree 试跑（M-2063.1~M-2063.5 完成 + 编译通过） + WBS L4 #2063 状态由 pending → in_progress

---

> **本计划是 living document**。每次 M 任务完成后，在 §3 各表追加 `commit hash` + `实测数据` 链接；OLU 估算与实测偏差 > 30% 时在 §3.6 加修正行。
