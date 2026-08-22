# RGS-DTL-041 实现规格书

**RGS-SPEC-DTL-041**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-SPEC-DTL-041 |
| 版本 | 0.2 |
| 状态 | 规格草案，待 RGS-DTL-041 具名 DD Review |
| 源详细设计 | RGS-DTL-041 |
| 实现范围 | 客户端资源分发的断点续传与可恢复下载（rgs-asset-download crate + asset_download SDK 模块） |
| 目标基线 | Rust 1.98 stable（当前基线；环境/CI Gate）、quinn 0.10+（QUIC）、tokio 1、sqlite 0.31、reqwest 0.12；环境需先核验 |
| 规格真源 | 源 DTL 的接口、字段、状态机、错误码、HTTP Range 契约和非目标 |

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

## 1. 使用规则

本规格把 RGS-DTL-041 从详细设计转成可执行的实现清单，不替代源 DTL。若本规格与 RGS-DTL-041 不一致，以 DTL 评审变更为准；不得在代码中自行调和冲突。当前工作区没有对应实现源码，本文件不代表功能已完成。

必须实现断点状态机、断点记录持久化、HTTP Range 客户端、并发分片调度、整文件校验闸门、平台特定预分配；不得在 SDK 热路径上绕过 `rgs-asset-update` 的 `IntegrityGate` 整文件校验（NFR-CDN-002 硬约束）。

## 2. 实现单元

| 类型 | 计划路径 | 要求 |
|---|---|---|
| 公共契约 | `crates/rgs-asset-download`（独立 crate，与 `rgs-asset-update` / `rgs-version` / `rgs-network` 同级） | crate 间依赖方向显式登记；不向 `rgs-asset-update` 反向依赖 |
| API/event | `rgs-asset-download` 公开 API：`download_asset` / `pause_download` / `cancel_download` / `get_download_state` | 错误码、状态枚举、断点记录 Schema 与 DTL §3~§5 一致 |
| 数据 | `ResumeTokenStore`（SQLite 索引 + JSON 文件） | 原子写（JSON 写后 SQLite 写）；LRU 清理；不含 PII |
| HTTP 协议 | `RangeClient`（HTTP/1.1 RFC 7233 HEAD / Range 请求） | 206/416/200/429 全部响应路径；`If-Range: <ETag>` 强制 |
| 平台特定 | `platform/unix.rs` / `platform/windows.rs` / `platform/android.rs` / `platform/ios.rs` | sparse file 预分配；Windows `SetFileValidData` 权限评估 |
| CI | fmt、clippy、test、deny、schema、secret、high-cardinality checks | 负例必须阻断合并 |

## 3. 实现契约

- 入口统一经由 `rgs-asset-update` 的 Manifest 拉取 / 签名校验 / 灰度判定；`rgs-asset-download` **不**重新实现这些前置能力。
- 所有 Range 请求携带 `If-Range: <ETag>` 头（**不**用 `Last-Modified`，FR-CDN-074）；ETag 不匹配触发全量重传。
- 断点记录本地持久化路径：`~/.rgs-sdk/downloads/`（Windows: `%APPDATA%\rgs-sdk\downloads\`，移动平台：应用沙箱目录）。
- 断点记录**不**含 PII 字段（FR-CDN-064）；代码评审 grep 验证。
- 暂停时**必须**取消所有 in_flight Range 请求（FR-CDN-083）；代码评审 grep `cancel_request` / `abort_request` 验证。
- 整文件校验不可绕过（NFR-CDN-002）；分块到达**不**做分块单独校验。
- 任意 `DistributionBackend` 实现（含自托管对象存储与商业 CDN）**必须**支持 HTTP Range（NFR-CDN-114）；后端选型门禁在 AC-CDN-117 验证。
- 错误码、状态枚举、HTTP Range 响应头契约严格按 DTL §3 / §5 / §6 实现，不自创额外枚举值。
- 所有 timeout、retry、backoff 有界：单 chunk 重试 3 次、指数退避 100ms 起步；重试耗尽返回 `RetryExhausted`。

## 4. 可观测性规格

- 业务代码只能调用统一 observability façade；禁止直接调用裸 OTel、tracing、log。
- 10 项 `rgs_asset_download_*` 指标（按 DTL §10 落地）：状态机转移 / active count / bytes received / resume success/failure / chunk retry / duration / throughput / integrity failure / ETag mismatch / resume token store bytes。
- 指标标签：仅 `from` / `to` / `file_path` / `status` / `reason` 等低基数标签；`player_id` / `device_id` / `ip` / `mac` **不**作为 metric label。
- 关键请求必须能用 `file_path` + `token_id` + `request_id` 反查 dashboard、trace、日志。
- 普通结构化日志与 OPERATION_AUDIT 分离；断点记录路径、token_id 不视为敏感（不含 PII）。
- 暂停 / 取消 / 恢复 / 灰度回退 4 类关键状态变更必须产生结构化日志事件。

## 5. 安全、容错与发布

| 领域 | 必须验证 |
|---|---|
| 认证授权 | 公开资源清单 / patch 走匿名访问（FR-CDN-001 既有）；SDK 不持有任何服务端凭证 |
| 幂等一致性 | Range 请求重试时携带相同 `If-Range` 头；分块落盘原子更新断点记录 |
| 故障 | 网络瞬时失败（指数退避）/ 服务端 416 / ETag 变更 200 OK / 整文件校验失败 / Manifest 签名失败 / 灰度回滚 / 磁盘满 / 断点过期 |
| 背压 | Range 并发数可配（桌面 ≤ 16 路、移动 ≤ 4 路）；失败 chunk 进入重试队列而非无限重试 |
| 发布 | `DistributionBackend` 后端选型门禁：未通过 Range 支持测试的候选**不得**启用（NFR-CDN-114 / AC-CDN-117） |
| 数据治理 | 断点记录不含 PII（FR-CDN-064）；ETag 携带**不**视为敏感；store 大小上限 100MB（LRU 清理） |

## 6. 测试规格

- UT：50 条用例覆盖 `DownloadStateMachine` 8 状态转移 / `ResumeTokenStore` 13 字段 + 原子写 + LRU / `RangeClient` HEAD + Range 全状态码 / `ChunkOrchestrator` 并发 + 暂停取消 / `IntegrityGate` 整文件 hash / 错误语义。
- IT：16 条用例覆盖 SDK 内部模块集成（asset-download ↔ asset-update / network）+ 后端 MinIO/Cloudflare 集成 + 跨 CDN 边缘 addendum 集成 + 协议版本协商被拒后链路完整。
- ST：13 条用例覆盖 AC-CDN-110~118 + NFR-CDN-110~114；4 平台（iOS 17 / Android 14 / Windows 11 / macOS 14）× 1000 客户端 × 1000 资源样本；故障注入 5 类（断网 / kill / ETag 变更 / 篡改 / 强制更新）。
- Load：100 万级 chunk 落盘 + GB 级文件并发分片吞吐。
- Chaos：服务端 5 类响应（206/416/200/429/503）随机注入。
- Security：断点记录 grep 验证 PII 字段为空。
- Rollback：完整功能回滚至 `rgs-asset-update` 既有全量下载路径（Range 协议禁用时）。

测试必须回填 RGS-REQ-004 追踪矩阵（AC-CDN-110~118）和对应 DTL 的验收项；不能只证明"服务启动"。

## 7. Definition of Done

- RGS-DTL-041 的审批/风险条件已满足；源 DTL 的 TBD（TBD-CDN-201/202/203）已有批准处置或纳入 PH-3 实测。
- 代码、HTTP Range 客户端、断点状态机、ResumeTokenStore、ChunkOrchestrator、IntegrityGate 实现与 DTL §3~§7 逐项对账。
- Cargo fmt、clippy、test、deny、schema、secret、high-cardinality 检查通过。
- 4 平台（iOS/Android/Windows/macOS）平台特定预分配（sparse file / SetFileValidData）实测通过。
- 集成测试在 4 平台 + MinIO + Cloudflare 6 套环境全部通过。
- AC-CDN-110~118 全部 9 项 + NFR-CDN-110~114 全部 5 项达标。
- 当前无实现文件时保持"待实现/待评审"，不得标记生产完成。

## 8. Gate 证据与实测参数

RGS-IMPL-001 已固定 workspace、crate、协议、迁移、错误、Saga、CI、镜像与可观测性后端边界；本规格不再保留这些工程选择的平行候选。进入实现前必须取得：① 源 DTL RGS-DTL-041 的具名 DD Review；② Rust 1.98 stable 的锁定依赖完整 CI、quinn 0.10+ QUIC 协议栈、reqwest 0.12 兼容性核验；③ MinIO 自托管（默认）+ Cloudflare 商业 CDN（可选对照）Range 行为实测；④ 4 平台 SDK sparse file / SetFileValidData 实测；⑤ 针对本实现范围，以 PH 基线和测试结果确定的：断点过期阈值（7 天）/ 并发分片粒度（4~16 MB）/ LRU 上限（100 MB）/ NFR-CDN-112 恶化阈值（≤ 20%）/ NFR-CDN-110 恢复时延（p99 < 500 ms）。上述均为实测参数和具名 Gate 证据，不是尚未选择的技术方案。
