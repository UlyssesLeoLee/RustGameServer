# 单元测试设计書（単体テスト設計書 / Unit Test Design Document）

**主题域 04 客户端与SDK — 断点续传与可恢复下载（补强）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-TST-UT-04-ADD2 |
| 版本 | 0.2 |
| 父文档 | RGS-REQ-036 v0.1 + RGS-DTL-041 v0.1 |
| V模型层级 | TL-1 单元试验 |
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

覆盖 RGS-REQ-036 §6~§8 + RGS-DTL-041 §3~§7 新增的断点续传模块——`DownloadStateMachine` / `ResumeTokenStore` / `RangeClient` / `ChunkOrchestrator` / `IntegrityGate` ——在单元试验层级（无网络、无文件系统、无 SQLite）的全部 FR-CDN-040~084 + NFR-CDN-110~114 行为。

## 2. 测试用例

### 2.1 DownloadStateMachine 状态机

| 用例 ID | 对应 FR/AC | 测试目的 | 覆盖类型 |
|---|---|---|---|
| TST-UT-04-R001 | FR-CDN-050 | `NotStarted → Probing` 合法 | N |
| TST-UT-04-R002 | FR-CDN-050 | `Probing → Downloading` 合法（无断点场景） | N |
| TST-UT-04-R003 | FR-CDN-050 | `Probing → Resuming` 合法（有断点场景） | N |
| TST-UT-04-R004 | FR-CDN-051 | `Downloading → Paused` 合法 | N |
| TST-UT-04-R005 | FR-CDN-051 | `Downloading → Canceled` 合法 | N |
| TST-UT-04-R006 | FR-CDN-051 | `Paused → Downloading` 合法 | N |
| TST-UT-04-R007 | FR-CDN-051 | `Failed → Resuming` 合法 | N |
| TST-UT-04-R008 | FR-CDN-052 | `Canceled → *` 全部非法（终态） | A |
| TST-UT-04-R009 | FR-CDN-052 | `Completed → *` 全部非法（终态） | A |
| TST-UT-04-R010 | FR-CDN-052 | 非法跳转返回 `InvalidTransition` 错误 | A |
| TST-UT-04-R011 | FR-CDN-053 | 状态变更同步到 `ResumeTokenStore.update_status` | S |

### 2.2 ResumeTokenStore 持久化

| 用例 ID | 对应 FR/AC | 测试目的 | 覆盖类型 |
|---|---|---|---|
| TST-UT-04-R020 | FR-CDN-060 | 13 字段完整序列化/反序列化 | N |
| TST-UT-04-R021 | FR-CDN-061 | upsert 原子写：JSON 在前，SQLite 索引在后 | S |
| TST-UT-04-R022 | FR-CDN-061 | 进程崩溃在 JSON 写完后 SQLite 写之前时，恢复后读取仍有效 | A |
| TST-UT-04-R023 | FR-CDN-062 | 存储路径独立于资源文件（`~/.rgs-sdk/downloads/`）| N |
| TST-UT-04-R024 | FR-CDN-063 | `is_expired` 在 `last_updated_at > 7天` 时返回 true | B |
| TST-UT-04-R025 | FR-CDN-063 | 7 天边界值（恰好 7 天 0 秒）返回 false | B |
| TST-UT-04-R026 | FR-CDN-064 | 断点记录**不**含 `player_id` / `device_id` / `ip` / `mac` 字段 | A |
| TST-UT-04-R027 | NFR-CDN-113 | LRU 清理：超过 100MB 时按策略淘汰 | B |
| TST-UT-04-R028 | NFR-CDN-113 | 优先清理 `completed` 状态超过 1 小时的记录 | B |
| TST-UT-04-R029 | NFR-CDN-113 | 优先清理 `canceled` / `failed` 状态超过 24 小时的记录 | B |

### 2.3 RangeClient HTTP 协议

| 用例 ID | 对应 FR/AC | 测试目的 | 覆盖类型 |
|---|---|---|---|
| TST-UT-04-R040 | FR-CDN-040 | 合法 Range 请求 → 206 Partial Content 解析 | N |
| TST-UT-04-R041 | FR-CDN-040 | `Range: bytes=START-`（开区间）→ 206 | N |
| TST-UT-04-R042 | FR-CDN-040 | `Range: bytes=-N`（最后 N 字节）→ 206 | N |
| TST-UT-04-R043 | FR-CDN-040 | 越界 Range → 416 错误返回 | A |
| TST-UT-04-R044 | FR-CDN-040 | 不支持 Range 的资源（`Accept-Ranges: none`）→ `RangeNotSupported` 错误 | A |
| TST-UT-04-R045 | FR-CDN-041 | `If-Range: <etag>` 匹配 → 206 续传 | N |
| TST-UT-04-R046 | FR-CDN-041 | `If-Range: <etag>` 不匹配 → 200 OK 触发全量重传 | A |
| TST-UT-04-R047 | FR-CDN-042 | HEAD 响应解析：`Content-Length` / `ETag` / `Accept-Ranges` / `Last-Modified` | N |
| TST-UT-04-R048 | FR-CDN-044 | 206 响应的 `Content-Length` 与 `Content-Range` 区间长度一致 | A |
| TST-UT-04-R049 | FR-CDN-074 | `If-Range` 头**必须**用 ETag 而非 `Last-Modified` | A |
| TST-UT-04-R050 | NFR-CDN-110 | HEAD 探测 + 签名校验 + 灰度查询总时延 < 500ms | B |

### 2.4 ChunkOrchestrator 并发分片

| 用例 ID | 对应 FR/AC | 测试目的 | 覆盖类型 |
|---|---|---|---|
| TST-UT-04-R060 | FR-CDN-080 | 文件 < 分片粒度时不分片，走单流 | N |
| TST-UT-04-R061 | FR-CDN-080 | 文件 ≥ 分片粒度时切为 N 个不相交区间 | N |
| TST-UT-04-R062 | FR-CDN-081 | 桌面平台并发数 ≤ 16 | N |
| TST-UT-04-R063 | FR-CDN-081 | 移动平台并发数 ≤ 4 | N |
| TST-UT-04-R064 | FR-CDN-082 | 恢复时仅重试 `chunk_manifest` 中未完成区间 | S |
| TST-UT-04-R065 | FR-CDN-082 | 已完成 chunk 在恢复时**不**重新下载 | A |
| TST-UT-04-R066 | FR-CDN-083 | 暂停时取消所有 in_flight 请求（计数器归零）| A |
| TST-UT-04-R067 | FR-CDN-083 | 暂停期间不发起新的 Range 请求 | A |
| TST-UT-04-R068 | NFR-CDN-111 | chunk 完成时断点记录写入 ≤ 10ms | B |

### 2.5 IntegrityGate 整文件校验

| 用例 ID | 对应 FR/AC | 测试目的 | 覆盖类型 |
|---|---|---|---|
| TST-UT-04-R080 | FR-CDN-070 | Sha256 算法整文件 hash 与 Manifest 声明值一致 → `passed=true` | N |
| TST-UT-04-R081 | FR-CDN-070 | Blake3 算法整文件 hash 一致 | N |
| TST-UT-04-R082 | FR-CDN-070 | 篡改文件 hash 不一致 → `passed=false` + `IntegrityCheckFailed` 错误 | A |
| TST-UT-04-R083 | FR-CDN-070 | **不**做分块单独校验（避免攻击面） | A |
| TST-UT-04-R084 | FR-CDN-071 | 恢复前重新拉 Manifest 签名失败 → 状态机 `Failed`，**不**使用既有断点 | A |
| TST-UT-04-R085 | FR-CDN-072 | 灰度回滚：恢复时玩家被切回旧版本 → 状态机 `Resuming → NotStarted`，从旧 URL 重新开始 | A |
| TST-UT-04-R086 | NFR-CDN-111 | GB 级文件 hash 计算不阻塞下载主流程（异步计算） | B |

### 2.6 错误语义

| 用例 ID | 对应 FR/AC | 测试目的 | 覆盖类型 |
|---|---|---|---|
| TST-UT-04-R100 | FR-CDN-073 | HTTP 429 触发 `RateLimited` 错误，**不**绕过限流配额 | A |
| TST-UT-04-R101 | NFR-CDN-114 | Range 请求与全量 GET 共享同一限流配额 | A |
| TST-UT-04-R102 | 异常处理 | 磁盘满时 `DiskSpaceInsufficient` 错误，**不**自动重试 | A |
| TST-UT-04-R103 | 异常处理 | 重试 3 次耗尽后 `RetryExhausted { attempts: 3 }` 错误 | B |

## 3. 追溯性

| 需求 | 用例 |
|---|---|
| FR-CDN-040 | TST-UT-04-R040~R044 |
| FR-CDN-041 | TST-UT-04-R045~R046 |
| FR-CDN-042 | TST-UT-04-R047 |
| FR-CDN-044 | TST-UT-04-R048 |
| FR-CDN-050~053 | TST-UT-04-R001~R011 |
| FR-CDN-060~064 | TST-UT-04-R020~R029 |
| FR-CDN-070~074 | TST-UT-04-R080~R085, R100 |
| FR-CDN-080~084 | TST-UT-04-R060~R068 |
| NFR-CDN-110 | TST-UT-04-R050 |
| NFR-CDN-111 | TST-UT-04-R068, R086 |
| NFR-CDN-113 | TST-UT-04-R027~R029 |
| NFR-CDN-114 | TST-UT-04-R101 |
| AC-CDN-110~118 | 全部（覆盖 §2 全部用例） |

## 4. 通过判定

- §2 全部 50 条用例 PASS
- `ResumeTokenStore` 不含 PII 字段（TST-UT-04-R026 grep 验证）
- 状态机非法跳转 100% 拒绝（TST-UT-04-R010）
- 完整性校验不可绕过（TST-UT-04-R083）
- 暂停期间无 Range 请求（TST-UT-04-R067 通过 mock 验证）

---

> 与 RGS-TST-UT-04 + RGS-TST-UT-04-ADD1 共存。
