# CDN 商业（Cloudflare R2 可选）vs 自托管 MinIO 对比报告

**报告编号**：RGS-DEPLOY-CDN-COMPARE-001
**版本**：v0.1（首版，per RGS-IMPL-PLAN-CDN-001 v0.1 §3.5 M-2072.4）
**日期**：2026-08-25
**制定者**：AI worker 子代理（per 主对话 2026-08-25 12:13 孤儿 SPEC 治理延伸）
**审批者**：Ulysses（架构师兼 + SRE 兼，per DEC-008）— **待 `/sign`**
**适用范围**：客户端可恢复下载 SDK（`rgs-asset-download`）后端选型决策
**关联**：
- 上行：[RGS-IMPL-PLAN-CDN-001 v0.1 §3.5](../12-工作流/RGS-IMPL-PLAN-CDN-001_断点续传实施计划_v0.1.md)
- [RGS-DTL-041 详细设计](../01-核心架构与设计模式/RGS-DTL-031_集群运营管理_每功能原子升级_详细设计.md)
- [ARC-045 客户端资源分发与热更新](../00-基本与治理/)(per WBS §16.2 L4 #2072)
- [RGS-REQ-004 §3.7 AC-CDN-110~118](../../RGS-REQ-004_需求规约.md)
- M-2072.1~3 IT 测试：`crates/rgs-asset-download/tests/it_cloudflare_{edge,canary}.rs`
- 上游：M-2069 MinIO 自托管实测报告 `docs/deploy/cdn-minio-it-report.md`（per #2069）

---

## 0. 摘要

| 维度 | 自托管 MinIO（生产默认） | Cloudflare R2（PH-5 可选） | 决策 |
|---|---|---|---|
| **运维成本** | 中（SRE 兼 / DBA 兼 1 人公司分摊 30% 容量）| 低（全托管，无 OS 补丁 / 容量规划）| 倾向 R2 |
| **跨 region 边缘命中** | 差（自建机房 1~3 个；跨大区延迟高）| 优（Cloudflare 300+ PoP）| 倾向 R2 |
| **冷启动首字节时延** | 取决于机房地理距离（per region 10~200ms）| < 50ms p50（边缘命中态）| 倾向 R2 |
| **数据出口成本** | 0（自托管内部网络）| 0（R2 无出口费，仅存储 + 操作费）| 平 |
| **存储成本** | 0.02 USD/GB·月（MinIO on Hetzner）| 0.015 USD/GB·月（R2）| 倾向 R2（10% 低）|
| **单 chunk 大小限制** | 5 GiB | 5 GiB | 平（per R6 缓解）|
| **HTTP Range 协议一致性** | 100% 符合 RFC 7233 | 100% 符合（per Cloudflare SLA）| 平 |
| **可恢复下载实测** | M-2069 1000 资源 × 4 平台 | **本报告**（per M-2072.2 + 2072.3）| — |
| **PH-5 选型门禁** | — | **NFR-CDN-114**（per R6 缓解，未通过 AC-CDN-117 不可启用）| — |
| **降级策略** | — | R2 异常自动回退 MinIO（per canary service abort 路径）| — |

**核心结论**：
1. **生产默认后端 = 自托管 MinIO**（per 实施计划 §1.2 + RGS-ADR-0052 数据主权约束）
2. **PH-5 商业 CDN = Cloudflare R2（可选对照）**（per 实施计划 §3.5）
3. **PH-5 启用门禁**：M-2072.2 / 2072.3 全部 IT 通过 + NFR-CDN-114 门禁通过
4. **当前状态**：PH-3 W7-W9 阶段仅完成 **测试代码 + 报告骨架**；**未跑真 IT**（Cloudflare 账号未就位，per 任务降级策略）

---

## 1. 范围 & 非目标

### 1.1 范围（per 实施计划 §3.5 M-2072.4）

- 商业 CDN（Cloudflare R2 可选）vs 自托管 MinIO 的 6 维对比（运维 / 性能 / 成本 / 协议 / 选型门禁 / 降级路径）
- M-2072.1 R2 bucket + Range endpoint 准备脚本：落地形态
- M-2072.2 边缘命中实测契约：4 region × 10 probe 起步门槛
- M-2072.3 切流验证契约：5% → 25% → 100% 三阶段
- 选型决策的 PH-5 启用门禁（NFR-CDN-114）
- 实施计划 §6 风险表 R6 的缓解证据

### 1.2 非目标

- ❌ **不**对比 AWS S3 / 阿里 OSS（per 实施计划 §1.2 + RGS-ADR-0052 选型收敛）
- ❌ **不**对比 CDN 反向代理模式（CloudFront / Akamai / Fastly）
- ❌ **不**做 TCO 5 年预测（实施计划 §3.5 不要求；PH-5 启用后再展开）

---

## 2. 选型基线（per 实施计划 §1.2 + §3.5 + R6 风险表）

| 项 | 自托管 MinIO | Cloudflare R2 | 来源 |
|---|---|---|---|
| **生产默认** | ✓ | — | 实施计划 §2.1 + §1.2 |
| **PH-5 可选** | — | ✓ | 实施计划 §3.5 |
| **NFR-CDN-114 门禁** | 必须 | 必须 | 实施计划 §1.3 + R6 风险表 |
| **未通过 AC-CDN-117 不可启用** | n/a | 硬约束 | 实施计划 §6 R6 |
| **数据主权 / FR-CDN-001 匿名访问** | ✓ 内部网络 | ✓ 公开 bucket | FR-CDN-001 既有 |
| **SRE 兼容量占比** | 30% | 5% | DEC-008 一人公司分摊 |

---

## 3. M-2072.2 边缘命中实测（多 region）

### 3.1 测试契约

| 契约项 | 目标 | 验证方式 |
|---|---|---|
| 区域数 | ≥ 4（nrt / sfo / fra / syd）| `RGS_CF_REGIONS` env 解析 |
| 每区域探测次数 | 10 起步（生产可调 50+）| `RGS_CF_PROBES_PER_REGION` env |
| Range 行为 | 206 Partial Content + `content-range: bytes a-b/total` | HTTP HEAD + Range bytes=0-1023 |
| `accept-ranges: bytes` | 必须存在 | HTTP HEAD |
| `cf-cache-status` 命中 | 至少 1 次 HIT（首轮 MISS 暖身后）| 20 次探针 |
| 边缘机房分布 | ≥ 2 个不同 `cf-ray` / `colo` | `cf-ray` + `colo` header |
| TTFB p99 | < 500ms（NFR-CDN-110 硬约束）| 4 region × N probe ttfb 采样 |
| 暖身后 miss 比例 | ≤ 30% | 20 probe / region |

### 3.2 跑法（待 SRE 接力）

```bash
# 1. 准备 R2 endpoint（per M-2072.1）
CLOUDFLARE_ACCOUNT_ID=*** CLOUDFLARE_API_TOKEN=*** \
  ./scripts/cloudflare_r2_setup.sh --env staging --region auto

# 2. 启用 IT（默认 #[ignore]）
RGS_CF_R2_BASE=https://pub-xxx.r2.dev \
RGS_CF_SMOKE_KEY=rgs-asset-download-smoke/abc123...bin \
RGS_CF_REGIONS=nrt,sfo,fra,syd \
RGS_CF_PROBES_PER_REGION=10 \
  cargo test -p rgs-asset-download --test it_cloudflare_edge -- --ignored --nocapture
```

### 3.3 预期结果（待 SRE 接力后回填）

| Region | Probes | 206 | HIT | TTFB p50 | TTFB p99 | colo |
|---|---|---|---|---|---|---|
| nrt | 10 | 10/10 | _/_ | __ms | __ms | nrt12 (期望) |
| sfo | 10 | 10/10 | _/_ | __ms | __ms | sfo12 (期望) |
| fra | 10 | 10/10 | _/_ | __ms | __ms | fra06 (期望) |
| syd | 10 | 10/10 | _/_ | __ms | __ms | syd01 (期望) |
| **合计** | 40 | **40/40** | **__** | **__ms** | **__ms** | **4 unique** |

> **回填说明**：本表由 SRE 接力 + Cloudflare 账号就位后实测填入；当前值留空（**待回填**）。

### 3.4 NFR-CDN-114 门禁判定

| 项 | 目标 | 状态 |
|---|---|---|
| AC-CDN-114（Range 行为）| 100% 206 + `accept-ranges: bytes` | **待测**（IT 已就位）|
| AC-CDN-117（多 region 边缘命中 < 50ms p50）| per-region p50 < 50ms | **待测** |
| 实施计划 §6 R6 缓解 | 未通过 AC-CDN-117 不可启用 | **门禁就位**（IT 自动断言）|

---

## 4. M-2072.3 切流验证（5% → 25% → 100%）

### 4.1 三阶段契约

| 阶段 | R2 权重 | 自托管 MinIO 权重 | 容差 |
|---|---|---|---|
| **canary-5%** | 5% | 95% | ±2% |
| **canary-25%** | 25% | 75% | ±2% |
| **full-100%** | 100% | 0%（降为 fallback）| ≥98% |

### 4.2 不变量（per 实施计划 §1.3）

| 不变量 | 来源 | 落地 |
|---|---|---|
| 整文件 SHA-256 校验不可绕过 | NFR-CDN-002 硬约束 | `it_cloudflare_canary::canary_three_stage_full_run` 强制 integrity_fail == 0 |
| 恶化阈值 ≤ 20% | NFR-CDN-112 硬约束 | 5% 阶段 p99 设为 baseline；25% / 100% 阶段 ratio ≤ 1.20 |
| 暂停时取消 in_flight | FR-CDN-083 | **PH-3 阶段**由 M-2065.4 保证（per `chunk_orchestrator.rs`）|
| If-Range ETag（不用 Last-Modified）| FR-CDN-074 | **PH-3 阶段**由 M-2065.2 保证（per `range_client.rs`）|
| 切流 abort 触发 | 5%/25% 阶段 R2 错误率 > 1% 必须 abort | canary service 决策（per DTL-007 §4）；本 IT 锁死"abort 后不绕过 hash 校验"|

### 4.3 跑法（待 SRE 接力）

```bash
# 前置：M-2072.1 R2 endpoint + M-2069 MinIO endpoint 都已就绪
RGS_CF_R2_BASE=https://pub-xxx.r2.dev \
RGS_CF_SMOKE_KEY=rgs-asset-download-smoke/abc123...bin \
RGS_SELF_HOSTED_BASE=https://cdn-self.rgs.internal \
RGS_SELF_HOSTED_KEY=smoke/abc123...bin \
RGS_CANARY_PROBES=200 \
  cargo test -p rgs-asset-download --test it_cloudflare_canary -- --ignored --nocapture
```

### 4.4 预期结果（待 SRE 接力后回填）

| 阶段 | Probes | R2 占比 | 整文件 hash fail | p99 (vs baseline) | abort? |
|---|---|---|---|---|---|
| **canary-5%** | 200 | __% | 0 | __ms (baseline) | — |
| **canary-25%** | 200 | __% | 0 | __ms (__%) | — |
| **full-100%** | 200 | __% | 0 | __ms (__%) | — |

> **回填说明**：本表由 SRE 接力后实测填入；当前值留空。

### 4.5 切流 abort 流程

```text
[canary service] -- 读取 R2 错误率 -->
  IF error_rate > 1% AND stage IN [5%, 25%] THEN
    abort 切流 → 回退到上一阶段
    [告警] RGS-CDN-CANARY-ABORT
  ELSE IF error_rate > 1% AND stage == 100% THEN
    [告警] 但**不**abort（已是 full，触发 R2 故障 → 自动 fallback MinIO）
  END
```

> **fallback 路径**：100% 阶段 R2 故障时，客户端 SDK 通过 `rgs-asset-update` 的
> `DistributionBackend.can_fallback()` 标志位自动回退到 MinIO endpoint（per
> DTL-007 §4.2）。

---

## 5. M-2072.1 R2 endpoint 准备脚本

### 5.1 脚本落地形态

- 路径：`scripts/cloudflare_r2_setup.sh`
- 7.7 KB / 195 行
- 6 步流水线：bucket 创建 → 公开访问 → smoke 资源生成 → 上传 → Range 验证 → 输出 endpoint JSON
- 失败模式：缺 `CLOUDFLARE_ACCOUNT_ID` / `CLOUDFLARE_API_TOKEN` / `wrangler` → 退出码 2 + 提示

### 5.2 输出契约

`scripts/cloudflare_r2_setup.sh` 成功后产出 `${TMPDIR}/r2-endpoint.json`：

```json
{
  "provider": "cloudflare-r2",
  "bucket": "rgs-cdn-public-staging",
  "public_base": "https://pub-xxx.r2.dev",
  "custom_domain": "",
  "smoke_asset": {
    "key": "rgs-asset-download-smoke/<sha256>.bin",
    "size_bytes": 1048576,
    "sha256": "<sha256>"
  },
  "verified_at": "2026-08-25T...Z",
  "verified_by": "sre@host"
}
```

> **消费方**：M-2072.2 / M-2072.3 通过 `RGS_CF_R2_BASE` + `RGS_CF_SMOKE_KEY` env 读取。

---

## 6. 6 维对比表

| 维度 | 自托管 MinIO | Cloudflare R2 | 结论 |
|---|---|---|---|
| **1. 运维成本（SRE 兼）**| 中（30% SRE 容量）：OS 补丁 / 容量规划 / 故障自愈 / 备份 | 低（5% SRE 容量）：仅监控告警 + 切流配置 | **R2 优** |
| **2. 跨 region 边缘命中**| 差：1~3 个机房，跨大区 RTT 100~300ms | 优：300+ PoP，全球 p50 < 50ms | **R2 优** |
| **3. 数据出口成本**| 0（自托管内部网络）| 0（R2 无出口费）| 平 |
| **4. 存储成本**| 0.02 USD/GB·月（Hetzner）| 0.015 USD/GB·月（Cloudflare）| **R2 优 25%** |
| **5. HTTP Range 协议一致性**| 100% RFC 7233 | 100% RFC 7233 | 平 |
| **6. NFR-CDN-114 门禁**| 1000 资源 × 4 平台实测（M-2069）| **待测**（IT 已就位 + 9 个 `#[ignore]`）| **待回填** |

### 6.1 PH-5 启用决策

| 启用条件 | 状态 | 来源 |
|---|---|---|
| M-2072.2 边缘命中 IT 全部通过 | **待 SRE 接力** | §3 |
| M-2072.3 切流 IT 全部通过 | **待 SRE 接力** | §4 |
| NFR-CDN-114 门禁通过 | **待 SRE 接力** | 实施计划 §1.3 |
| AC-CDN-117 多 region p50 < 50ms | **待 SRE 接力** | §3.4 |
| 数据主权 / 隐私审批（Ulysses 显式 /sign）| **待审批** | DEC-008 |

> **最终启用决策** = 全部启用条件满足后，Ulysses（架构师兼 + SRE 兼）显式 `/sign`；
> 否则保持自托管 MinIO 单后端（per 实施计划 §1.2）。

---

## 7. 风险 & 缓解（per 实施计划 §6 R6）

| # | 风险 | 等级 | 缓解（M-2072 已落地）|
|---|---|---|---|
| R6 | 商业 CDN Range 行为差异 | 低 | **本报告** + NFR-CDN-114 门禁：未通过 AC-CDN-117 测试的候选**不得**启用 |
| R3 | 客户端 token 跨 session 失效 | 低 | `cleanup_expired()` 启动时清理 7 天前 token（per `config.rs`）|
| R7 | 断点记录被外部进程读取 | 低 | token_id 用 UUID v4 随机生成（per M-2064.2 + FR-CDN-064）|
| R-EXTRA | Cloudflare 账号不可用导致 PH-5 跳过 | 中 | **本任务降级策略**：IT 代码完整 + `#[ignore]` 标记 + 报告就位；Cloudflare 就位后 SRE 直接跑 |

---

## 8. 关联文档 & 后续任务

### 8.1 关联文档

- [RGS-IMPL-PLAN-CDN-001 v0.1 §3.5](../12-工作流/RGS-IMPL-PLAN-CDN-001_断点续传实施计划_v0.1.md) — L4 #2072 任务源
- [RGS-ADR-0052 v0.2 数据主权与商业 CDN 选型](../02-决策记录/)（per OPEN-QA-001 Q-D-06）
- [RGS-ARC-007 canary service 控制平面](../00-基本与治理/)（per §4.2 fallback 路径）
- [DTL-007 灰度判定服务](../01-核心架构与设计模式/)（per canary weight 来源）

### 8.2 后续任务（待 SRE 接力）

| 任务 | 优先级 | 截止 | 负责 |
|---|---|---|---|
| 准备 Cloudflare 账号 + R2 API token | 高 | PH-5 启动前 W11 | SRE 兼 |
| 跑 `scripts/cloudflare_r2_setup.sh --env staging` 产出 R2 endpoint | 高 | PH-5 W12 | SRE 兼 |
| 跑 M-2072.2 IT（4 region 边缘命中）| 高 | PH-5 W12 | SRE 兼 |
| 跑 M-2072.3 IT（5% → 25% → 100% 切流）| 高 | PH-5 W13 | SRE 兼 |
| 写 RGS-ADR-0053 商业 CDN 启用决议（per PH-5 启用决策）| 中 | PH-5 W14 | Ulysses（架构师兼）|
| 回填本报告 §3.3 / §4.4 数据 | 中 | 同上 | SRE 兼 |

### 8.3 跟 WBS L4 上下游

- **上游**：L4 #2069（MinIO 自托管实测）= M-2072.3 切流对照基准
- **下游**：L4 #2063 / #2064 / #2065（PH-3）= RangeClient / IntegrityGate 实现后，本报告 IT 真跑就位
- **PH-5 启用门禁**：M-2072.2 + M-2072.3 + NFR-CDN-114 全部通过

---

## 9. 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-25 | AI worker 子代理 | 首版：M-2072.1~4 代码 + IT 契约 + 6 维对比表 + PH-5 启用决策；**未跑真 IT**（Cloudflare 账号未就位，per 任务降级策略）|

---

## 附录 A：本报告引用的 IT 测试位置

- `crates/rgs-asset-download/tests/it_cloudflare_edge.rs`（M-2072.2；4 个 `#[ignore]` + 4 个契约）
- `crates/rgs-asset-download/tests/it_cloudflare_canary.rs`（M-2072.3；5 个 `#[ignore]` + 6 个契约）
- `crates/rgs-asset-download/src/lib.rs`（M-2072.PREREQ 骨架）
- `scripts/cloudflare_r2_setup.sh`（M-2072.1 R2 endpoint 准备）

## 附录 B：当前任务落地状态

- [x] **M-2072.PREREQ** 最小 crate 骨架（`Cargo.toml` + `lib.rs` + workspace member）
- [x] **M-2072.1** R2 bucket + Range endpoint 配置脚本（`scripts/cloudflare_r2_setup.sh`，195 行）
- [x] **M-2072.2** 边缘命中实测 IT 契约（`it_cloudflare_edge.rs`，8 tests / 4 ignored）
- [x] **M-2072.3** 切流验证 IT 契约（`it_cloudflare_canary.rs`，11 tests / 5 ignored）
- [x] **M-2072.4** 商业 CDN vs 自托管 MinIO 对比报告（本文件）
- [ ] **真跑 IT**（**待 SRE 接力** + Cloudflare 账号就位）
