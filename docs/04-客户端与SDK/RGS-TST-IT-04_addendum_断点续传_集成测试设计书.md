# 集成测试设计書（統合テスト設計書 / Integration Test Design Document）

**主题域 04 客户端与SDK — 断点续传与可恢复下载（补强）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-TST-IT-04-ADD2 |
| 版本 | 0.2 |
| 父文档 | RGS-REQ-036 v0.1 + RGS-DTL-041 v0.1 |
| V模型层级 | TL-3 模块间集成 / TL-4 子系统集成 / TL-5 系统间集成 |
| 制定日 | 2026-08-21 |

---

---

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | Ulysses(架构师兼 / Admin 域 Lead兼 per DEC-008) | 2026-08-21 | 一人公司 12 角色兼任 |
| 评审（技术/架构） | Ulysses(架构师兼 per DEC-008) | 2026-08-21 | DEC-008 |
| 评审（平台/客户端/SRE/DBA/安全/合规/法务） | Ulysses(对应角色兼 per DEC-008) | 2026-08-21 | DEC-008 |
| 评审（运营） | Ulysses(运营兼 per DEC-008) | 2026-08-21 | 仅适用全生命周期文档 |
| **集体签字(per DEC-008)** | **Ulysses(一人公司 12 角色兼任)** | **2026-08-21** | **Ulysses 在审批栏各角色中具名签字,完整 12 角色兼任清单见 RGS-WBS-001 §17 集体签字声明。审批栏细化角色意见详见 RGS-REQ-004 §3.10。** |

---

## 1. 目的

覆盖断点续传模块与既有 SDK 模块（`rgs-asset-update` / `rgs-version` / `rgs-network`）+ 后端 `DistributionBackend`（自托管 MinIO + 商业 CDN 可选）的跨模块集成场景，验证 SDK 内部模块边界与跨进程边界均无破坏。

## 2. 测试用例

### 2.1 SDK 内部模块集成

| 用例 ID | 集成层级 | 对应 FR | 测试目的 | シナリオ | テストデータ |
|---|---|---|---|---|---|
| TST-IT-04-R001 | TL-3 | FR-CDN-070 | `rgs-asset-download` 调用 `rgs-asset-update` 的 `IntegrityGate` 整文件校验 | — | — |
| TST-IT-04-R002 | TL-3 | FR-CDN-071 | `rgs-asset-download` 调用 `rgs-asset-update` 的 `ManifestService` 重新拉取并校验签名 | — | — |
| TST-IT-04-R003 | TL-3 | FR-CDN-072 | `rgs-asset-download` 调用 `rgs-asset-update` 的 `GrayRolloutChecker` 灰度状态判定 | — | — |
| TST-IT-04-R004 | TL-3 | FR-CDN-051 | `rgs-asset-download` 的状态变更同步到 `rgs-asset-update` 的 Rollout 事件 | — | — |
| TST-IT-04-R005 | TL-3 | FR-CDN-073 | `rgs-asset-download` 触发限流时与 `rgs-network` 限流配额共享 | — | — |
| TST-IT-04-R006 | TL-3 | FR-CDN-040 | `rgs-asset-download` 通过 `rgs-network` (QUIC/TCP) 发起 Range 请求 | — | — |

### 2.2 与后端 DistributionBackend 集成

| 用例 ID | 集成层级 | 对应 FR | 测试目的 | シナリオ | テストデータ |
|---|---|---|---|---|---|
| TST-IT-04-R020 | TL-4 | FR-CDN-040 | 客户端 Range 请求打到 MinIO 自托管，验证 206/416/ETag/If-Range 行为 | — | — |
| TST-IT-04-R021 | TL-4 | FR-CDN-041 | ETag 变更：MinIO 上新版本对象，客户端收到 200 OK | — | — |
| TST-IT-04-R022 | TL-4 | FR-CDN-045 | Range 协议不改变 MinIO 上文件存储格式（不预切片） | — | — |
| TST-IT-04-R023 | TL-4 | FR-CDN-046 | 客户端 Range 协议与自托管对象存储契约一致（NFR-CDN-114）| — | — |
| TST-IT-04-R024 | TL-5 | FR-CDN-043 | 商业 CDN（Cloudflare 可选）Range 行为一致性测试 | — | — |
| TST-IT-04-R025 | TL-5 | NFR-CDN-114 | 商业 CDN Range 边缘命中实测（RSK-CDN-203 缓解）| — | — |
| TST-IT-04-R026 | TL-4 | FR-CDN-040 | 客户端 Range 协议不依赖任何特定后端 SDK（抽象层不变性）| — | — |

### 2.3 与既有 CDN 边缘 addendum 集成

| 用例 ID | 集成层级 | 对应 FR | 测试目的 | シナリオ | テストデータ |
|---|---|---|---|---|---|
| TST-IT-04-R040 | TL-4 | FR-CDN-040 + RGS-REQ-030-ADD1 §3 FR-CDN-030 | 边缘节点对 Range 请求的缓存键（`{channel}/{version}/{region}/{file}#range_START-END`）正确命中 | — | — |
| TST-IT-04-R041 | TL-4 | FR-CDN-040 + FR-CDN-032 | Range 请求回源时与全量 GET 共用回源策略（`DistributionBackend` 源站）| — | — |
| TST-IT-04-R042 | TL-4 | FR-CDN-041 + FR-CDN-034 | ETag 变更触发全量重传时，CDN 快速切回 stable channel ≤ 30s | — | — |

### 2.4 跨 SDK 协作场景

| 用例 ID | 集成层级 | 对应 FR | 测试目的 | シナリオ | テストデータ |
|---|---|---|---|---|---|
| TST-IT-04-R060 | TL-3 | FR-CDN-023 | 协议版本协商被拒（`result_code=协议版本过旧`）→ 自动进入断点续传下载流程 | — | — |
| TST-IT-04-R061 | TL-3 | FR-CDN-024 | 强制更新场景：从最低受支持版本 SDK 升级到当前版本，断点续传链路完整 | — | — |
| TST-IT-04-R062 | TL-3 | FR-CDN-070 | 完整性校验失败 → 触发全量重传 → 全量重传仍走断点续传（无限重试循环检测）| — | — |

## 3. 最小可复现实验

### 3.1 固定基线与取证规则

| 项目 | 固定条件 |
|---|---|
| 拓扑/规格 | SDK：3 平台（iOS 17 / Android 14 / Windows 11）；后端：MinIO 自托管（默认）+ Cloudflare 商业 CDN（可选对照）；客户端 SDK 与后端通过 QUIC over UDP 通信。 |
| 数据集与负载模型 | 100 个资源样本（覆盖 5 种断点续传触发场景：正常/暂停/杀进程/灰度回滚/篡改）；每个 SDK 实例完整跑 100 个资源样本。 |
| 预热与持续时间 | 预热 5 分钟；正式持续 30 分钟；每个样本 1 次完整下载。 |
| 故障注入 | ① SDK 实例 kill -9；② 后端 ETag 变更；③ 篡改响应；④ 灰度回滚；⑤ 服务端返回 416 / 200 OK / 206 各种状态。 |
| 采样/SLO计算 | 每下载记录：file_path / SDK 平台 / Range 序列 / 状态码 / ETag 比对结果 / 跨模块调用栈 / 集成时延 / 错误恢复路径。 |
| 原始证据路径 | `artifacts/test-results/TST-IT-04-ADD2/<run-id>/<case-id>/{topology.yaml,sdk_calls.parquet,backend_logs.parquet,range_audit.jsonl,integration_summary.json}`；`integration_summary.json` 必须含 SDK 模块调用时序图。 |
| 清理步骤 | 停止 SDK 实例、清除 MinIO 测试桶、删除临时凭据；保留 evidence 目录。 |

### 3.2 用例执行矩阵

| 用例 | 集成对象 | 测试触发 | 可判定预期 |
|---|---|---|---|
| C001 (R001) | asset-download ↔ asset-update | 整文件校验 | 跨模块调用链 `asset_download::IntegrityGate` → `asset_update::verify` 一次成功；失败时 `asset_download` 触发全量重传。 |
| C002 (R002) | asset-download ↔ asset-update | 恢复时 Manifest 校验 | 签名失败时 `asset_download` 不使用既有断点。 |
| C003 (R003) | asset-download ↔ asset-update | 灰度回滚 | 灰度不匹配时 `asset_download` 触发 `Resuming → NotStarted`。 |
| C004 (R004) | asset-download ↔ asset-update | 状态变更传播 | `DownloadState` 变更通过事件总线通知 `asset_update` 的 Rollout。 |
| C005 (R005) | asset-download ↔ network | 限流配额 | Range 与全量 GET 共享同一限流配额（按 IP）。 |
| C006 (R006) | asset-download ↔ network | QUIC Range 请求 | Range 请求通过 QUIC stream 发送，206 响应正常解析。 |
| C020 (R020) | SDK ↔ MinIO | 206/416/ETag/If-Range | 自托管 MinIO 全场景行为正确。 |
| C021 (R021) | SDK ↔ MinIO | ETag 变更 | 客户端收到 200 OK 触发全量重传。 |
| C022 (R022) | SDK ↔ MinIO | 存储格式不变 | MinIO 上文件字节级一致，Range 协议无副作用。 |
| C023 (R023) | SDK ↔ MinIO | 抽象层不变 | Range 协议不依赖 MinIO SDK，HTTP 客户端直接对接。 |
| C024 (R024) | SDK ↔ Cloudflare | Range 一致性 | 商业 CDN Range 行为与 MinIO 一致（边缘节点 / 源站路径）。 |
| C025 (R025) | SDK ↔ Cloudflare | 边缘 Range 命中 | Range 请求在边缘命中缓存（命中率 ≥ 80%）；未达标 CDN 候选**不得**启用。 |
| C040 (R040) | SDK ↔ CDN 边缘 | Range 缓存键 | 边缘缓存键 `{channel}/{version}/{region}/{file}#range_START-END` 命中。 |
| C041 (R041) | SDK ↔ CDN 边缘 | Range 回源 | Range 请求 miss 时回源至 MinIO 源站（共享回源通道）。 |
| C042 (R042) | SDK ↔ CDN 边缘 | ETag 变更 + 灰度切回 | ETag 变更时 CDN 切回 stable channel ≤ 30s。 |
| C060 (R060) | 协议版本 ↔ 资源分发 | 协议过旧引导 | 握手拒绝后 SDK 自动查询 Manifest 进入断点续传下载。 |
| C061 (R061) | 强制更新 ↔ 断点续传 | 强制更新场景 | 强制更新场景下断点续传链路完整；不会出现循环重试。 |

## 4. 追溯性

| FR | 用例 |
|---|---|
| FR-CDN-023 | TST-IT-04-R060 |
| FR-CDN-024 | TST-IT-04-R061 |
| FR-CDN-040 | TST-IT-04-R006, R020, R023, R026, R040 |
| FR-CDN-041 | TST-IT-04-R021, R042 |
| FR-CDN-043 | TST-IT-04-R024, R025 |
| FR-CDN-045 | TST-IT-04-R022 |
| FR-CDN-046 | TST-IT-04-R023 |
| FR-CDN-070 | TST-IT-04-R001, R062 |
| FR-CDN-071 | TST-IT-04-R002 |
| FR-CDN-072 | TST-IT-04-R003 |
| FR-CDN-073 | TST-IT-04-R005 |
| FR-CDN-051 | TST-IT-04-R004 |
| NFR-CDN-114 | TST-IT-04-R023, R025 |

## 5. 通过判定

- §2 全部 16 条用例 PASS
- 商业 CDN 边缘 Range 命中率 ≥ 80%（NFR-CDN-114 门禁）
- 自托管 MinIO Range 行为 100% 符合 RFC 7233
- 跨 SDK 模块调用无循环依赖
- 协议版本协商被拒后断点续传链路完整

---

> 与 RGS-TST-IT-04 + RGS-TST-IT-04-ADD1 共存。
