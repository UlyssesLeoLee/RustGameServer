# 系统测试设计書（システムテスト設計書 / System Test Design Document）

**主题域 04 客户端与SDK — 断点续传与可恢复下载（补强）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-TST-ST-04-ADD2 |
| 版本 | 0.2 (v0.1 → v0.2 升版 per 2026-09-07 12:35 JST 拍板) |
| 父文档 | RGS-REQ-036 v0.1 + RGS-DTL-041 v0.1 |
| V模型层级 | TL-6 负载 / TL-7 故障注入 |
| 制定日 | 2026-08-21 (v0.1) / 2026-09-07 (v0.2 升版) |
| 关联 v0.2 升版基线 | RGS-TEST-DESIGN-2026-09-07_v0.2.md (commit 583ce9e) + RGS-TEST-CASES-2026-09-07_v0.2.md (commit 3ce36f0) |
| 制定者 | Mavis 接手 agent per DEC-008 (代签 Ulysses) |

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

## 0. v0.1 → v0.2 升版范围 (Round 2 W5 综合 8 维度 + ST 特定增量)

per 2026-09-07 12:35 JST 拍板 (scope=opt4 全部 v0.2 综合 8 维度), 本 addendum (断点续传与可恢复下载) 升版增量:

| # | 维度 | v0.1 现状 (9/5 拍板) | v0.2 增量 (9/7 拍板) | 引用 |
|---|---|---|---|---|
| 1 | **8 域扩展** | §2 仅客户端 + 服务端 | 13 域 (player / economy / match / social / admin + scene / battle / network / account / sub8 / batch + 平台 + function-plane) | 1134cfd / 95e67a6 / 57edbeb / b6b19b7 / 1dd9afc / 3c79bca / 42df673 |
| 2 | **batch v0.1 + v0.2 EVAL** | §2 完全没提 batch 域 | §5.1 增 6 module × 15 用例 = 90 用例 (cron / task_templates / worker_pool / audit_logger / dlq / connector) | fd122f6 / e70ed71 / e366ff8 / 62027c9 / eb1e15d |
| 3 | **admin-coc Phase B** | §4 仅 AC-CDN 追溯 | §5.2 增 admin-coc §X 集成设计 + 7 项 admin 域 Lead 真实签字 + coc_policy 决策树 3 场景 ST 用例 | ae9702d / 6c2a786 / ab127e4 / 3695f3b |
| 4 | **plugin 集群 + app 集群架构** | §3.1 仅 MinIO + Cloudflare | §5.3 增 4 阶段用例 (per 9/5 61cf306 ARCH §4) | 61cf306 / f785f18 |
| 5 | **flash-mock v0.3** | §2 仅 "1000 客户端实例" | §5.4 增 v0.3: 60 module + 4 NEW 回归脚本 | 575f5c9 / fdba686 / 01aee71 |
| 6 | **9 域 mTLS 业务级** | §3.1 无 mTLS 字段 | §5.5 增 9 域 mTLS 业务级 E2E 11 步客户端模拟器 v3 (per 9/6 d270ab9) | d270ab9 / d15a0bb |
| 7 | **REQ/BDD/DDD v0.2 升版** | §1 仅父文档 v0.1 引用 | §1 增 3 addendum 引用 (业务逻辑逆推 + 协议号映射 + frontend 适配) | 39d817b / 96e6b3c / 554b1ef |
| 8 | **cutover 收口** | §5 仅 "0 高优事故" | §5 增 13 commit 推远端 + 派生约束 L15-L23 落地 (per 9/6 6c6839e cutover 收口) | 6c6839e / add4238 |

**ST 特定增量** (per W5 任务简报, 跟 IT-04 addendum 不同, ST 强调端到端业务级 9 域 mTLS):
- **envoy 独立 deployment 偏好**: per 9/1 13:03/13:05 JST Ulysses 决策 (所有 nginx → envoy + 独立 deployment 模式), 断点续传 §3.1 服务端部署应明确 envoy 验证
- **9 域 mTLS 业务级 11 步 v3**: per 9/6 d270ab9, §5.5 EX-ST-MTLS-9D-001 11 步 E2E 模拟器
- **派生约束守护**: L1/L11/L12/L15-L23 + 9/1 batch 12 派生约束 + B3 二审流程 (per AGENTS.md §6.2 + 9/2 10:18 JST D2 拍板)

**目标读者**: ST 业务级测试工程师 / Ulysses 二审 / 6 域 Lead (5 + batch) 签字 / SRE Lead 接管验证

---

## 1. 目的

端到端验证 RGS-REQ-036 AC-CDN-110~118 + NFR-CDN-110~114 在真实网络（弱网、QUIC 断连、4G/5G 切换）下的端到端表现，覆盖 9 个 AC。

**v0.2 增补**: 断点续传覆盖 envoy 独立 deployment (per 9/1 13:03/13:05 JST 偏好, 不选 nginx) + 9 域 mTLS 业务级 (per 9/6 d270ab9) + rgs-flash-mock v0.3 60 module fixture (per RGS-FLASH-MOCK-DESIGN-2026-09-04 v0.3).

## 2. 测试用例

| 用例 ID | 试验级别 | 对应 AC | 测试目的 |
|---|---|---|---|
| TST-ST-04-R001 | [E2E] | AC-CDN-110 | 60% 进度时强制断网，恢复后从 60% 续传（不重头） |
| TST-ST-04-R002 | [E2E] | AC-CDN-111 | 80% 进度时客户端进程被 kill -9，重启后自动恢复至 80% |
| TST-ST-04-R003 | [E2E] | AC-CDN-112 | 源文件 ETag 变更（运营紧急回滚），客户端收到 200 OK 全量响应，触发从头重传 |
| TST-ST-04-R004 | [E2E] | AC-CDN-113 | 30% 暂停 → 5 分钟 → 恢复，验证从 30% 续传 + 暂停期间零带宽占用 |
| TST-ST-04-R005 | [E2E] | AC-CDN-114 | GB 级（≥1GB）文件 8 路并发分片，30% 暂停后仅重试未完成 70% |
| TST-ST-04-R006 | [E2E] | AC-CDN-115 | 灰度回滚：从新版本断点恢复到旧版本 URL 重新开始（不续传新版本） |
| TST-ST-04-R007 | [TL-7] | AC-CDN-116 | 中间人攻击场景：篡改 Range 响应 → 整文件校验失败 + 触发全量重传 |
| TST-ST-04-R008 | [E2E] | AC-CDN-117 | 后端选型评估：未通过 Range 支持门禁的候选后端被部署门禁拒绝 |
| TST-ST-04-R009 | [E2E] | AC-CDN-118 | 断点记录存储超 100MB 时 LRU 清理生效 |
| TST-ST-04-R010 | [TL-6] | NFR-CDN-110 | 恢复可行性判断（HEAD + 签名 + 灰度查询）p99 < 500ms |
| TST-ST-04-R011 | [TL-6] | NFR-CDN-111 | 断点记录本地持久化毫秒级（≤ 10ms） |
| TST-ST-04-R012 | [TL-6] | NFR-CDN-112 | 总下载时间相对无中断连续下载恶化 ≤ 20% |
| TST-ST-04-R013 | [TL-6] | NFR-CDN-113 | LRU 清理策略：completed > 1h、canceled/failed > 24h、last_updated > 7d |

## 3. 最小可复现实验

### 3.1 固定基线与取证规则

| 项目 | 固定条件 |
|---|---|
| 拓扑/规格 | 客户端：iOS 17 / Android 14 / Windows 11 / macOS 14 4 平台各 5 台；服务端：MinIO 自托管（默认后端）+ Cloudflare 商业 CDN（可选对照）；每平台至少 1 GB 资源包样本（覆盖：< 8MB 不分片、8MB~1GB 标准分片、≥ 1GB GB 级分片）。 |
| 数据集与负载模型 | 1000 客户端实例分布在 4 平台，按 25% 模拟正常下载、25% 模拟 30% 暂停恢复、25% 模拟 80% kill -9 恢复、25% 模拟灰度回滚；资源类型分布 30% < 8MB、50% 8MB~1GB、20% ≥ 1GB。 |
| 预热与持续时间 | 预热 10 分钟（前置下载建立 baseline）；正式持续 60 分钟；故障注入在第 15/30/45 分钟分批执行。 |
| 故障注入 | ① QUIC 握手后 60s 强制断网；② 客户端进程 kill -9；③ 服务端 ETag 变更（运营回滚）；④ 篡改 Range 响应（中间人模拟）；⑤ 客户端版本号低于 min_supported_client_version 触发强制更新。 |
| 采样/SLO计算 | 每下载记录：file_path / 平台 / chunk_id / Range 请求 / 206 vs 416 vs 200 / ETag / 实际吞吐 / 整文件 hash 校验结果 / 暂停恢复次数；p99 为预热后每 1 分钟窗口的最差值。 |
| 原始证据路径 | `artifacts/test-results/TST-ST-04-ADD2/<run-id>/<case-id>/{topology.yaml,downloads.parquet,chunk_audit.jsonl,resume_tokens.json,integrity_report.json,summary.json}`；`summary.json` 必须含平台 / 资源类型 / 中断类型 / 恢复耗时 / 总下载时间。 |
| 清理步骤 | 停止所有下载、清除断点记录、删除 MinIO 测试桶、失效 CDN 缓存、删除临时凭据；保留 evidence 目录。 |

### 3.2 用例执行矩阵与可判定预期

| 用例 | 拓扑/数据/负载 | 预热/持续/故障注入 | 可判定预期 |
|---|---|---|---|
| C001 (AC-CDN-110) | 4 平台 × 1000 客户端，60% 进度强制断网 | 10m/60m，第 15min 注入断网 | 恢复后下载进度从 60% 起，已下字节**不**重复下载；整文件 hash 校验通过。 |
| C002 (AC-CDN-111) | 同基线，80% 进度 kill -9 | 10m/60m，第 30min 注入 kill | 客户端重启后从 80% 自动恢复；恢复耗时 < 5s。 |
| C003 (AC-CDN-112) | 同基线，运营回滚 | 10m/60m，第 45min 服务端 ETag 变更 | 客户端下次 Range 请求收到 200 OK 触发全量重传，**不**续传陈旧内容。 |
| C004 (AC-CDN-113) | 同基线，30% 暂停 | 10m/60m，30% 暂停 5min | 暂停期间服务端 Range 请求数 = 0；恢复后从 30% 续传。 |
| C005 (AC-CDN-114) | 仅 ≥ 1GB 文件 × 8 路分片 | 10m/60m，30% 暂停 | 恢复时仅重试未完成的 70%，已完成 30% **未**重新下载；总下载时间 ≤ 1.2 × 无中断 baseline。 |
| C006 (AC-CDN-115) | 同基线，灰度回滚 | 10m/60m，第 30min 触发灰度回滚 | 客户端从旧版本 URL 重新开始，**不**续传新版本。 |
| C007 (AC-CDN-116) | 单客户端，篡改响应 | 10m/60m，注入篡改 | 整文件 hash 校验失败 + 触发全量重传 + 安全告警。 |
| C008 (AC-CDN-117) | 部署门禁测试 | N/A | 提交缺 Range 支持的候选后端 profile → 部署门禁 100% 拒绝。 |
| C009 (AC-CDN-118) | 单客户端累积断点 | 持续累积断点至 > 100MB | LRU 清理触发，存储不超 100MB；优先清理 completed > 1h 记录。 |
| C010 (NFR-CDN-110) | 4 平台并发恢复 | 10m/60m，无故障 | 恢复可行性判断（HEAD + 签名 + 灰度查询）p99 < 500ms。 |
| C011 (NFR-CDN-111) | 单客户端，chunk 落盘 | 10m/60m，无故障 | 断点记录本地写入 ≤ 10ms。 |
| C012 (NFR-CDN-112) | GB 级文件，无中断对照 | 10m/60m，无故障 | 总下载时间相对 baseline 恶化 ≤ 20%。 |
| C013 (NFR-CDN-113) | 累积多种状态断点 | 持续累积至 > 100MB | 清理优先级：completed > 1h > canceled/failed > 24h > 7d > LRU。 |

## 4. 追溯性

| AC/NFR | 用例 |
|---|---|
| AC-CDN-110 | TST-ST-04-R001 |
| AC-CDN-111 | TST-ST-04-R002 |
| AC-CDN-112 | TST-ST-04-R003 |
| AC-CDN-113 | TST-ST-04-R004 |
| AC-CDN-114 | TST-ST-04-R005 |
| AC-CDN-115 | TST-ST-04-R006 |
| AC-CDN-116 | TST-ST-04-R007 |
| AC-CDN-117 | TST-ST-04-R008 |
| AC-CDN-118 | TST-ST-04-R009 |
| NFR-CDN-110 | TST-ST-04-R010 |
| NFR-CDN-111 | TST-ST-04-R011 |
| NFR-CDN-112 | TST-ST-04-R012 |
| NFR-CDN-113 | TST-ST-04-R013 |

## 5. 通过判定

- AC-CDN-110~118 全部 9 项通过
- NFR-CDN-110~114 全部 5 项达标
- 0 高优事故
- 篡改注入 100% 拦截（AC-CDN-116）
- 暂停期间零带宽占用（AC-CDN-113）

**v0.2 增补**:
- envoy 独立 deployment 验证 (per 9/1 13:03/13:05 JST 偏好)
- 9 域 mTLS 业务级 11 步 v3 端到端验证 (per 9/6 d270ab9)
- rgs-flash-mock v0.3 60 module fixture 回归 (per 9/4 v0.3 设计书)

---

## 6. v0.2 升版 ST 特定增量 (Round 2 W5)

### 6.1 batch v0.1 6 module × 15 用例 (v0.2 新增)

per 9/1 batch 4 件套 (REQ + BASIC + DETAILED + PLAN) + 9/2 v0.1 FREEZE + 9/3 v0.2 EVAL:

| 用例 ID | 试验级别 | module | 场景数 | 测试目的 | evidence |
| --- | --- | --- | --- | --- | --- |
| TST-ST-04-ADD2-BATCH-001 | [E2E] | rgs-batch-backend::cron | 3 | 定时任务调度 + worker_pool 1 个 task 完成 + audit_log 1 条 | F-1 |
| TST-ST-04-ADD2-BATCH-002 | [E2E] | rgs-batch-backend::task_templates | 2 | 模板版本化 2 版本共存 | GAP-8 |
| TST-ST-04-ADD2-BATCH-003 | [E2E] | rgs-batch-backend::worker_pool | 2 | 多 worker 并发 100 task, 100/100 完成 | F-4 |
| TST-ST-04-ADD2-BATCH-004 | [E2E] | rgs-batch-backend::audit_logger | 2 | 操作人/时间/参数 hash/结果/trace_id 永久保留 (NFR-29 T-3) | F-10 |
| TST-ST-04-ADD2-BATCH-005 | [E2E] | rgs-batch-backend::dlq | 3 | dead-letter 队列, 失败 3 次入 DLQ, payload 完整 | F-9 |
| TST-ST-04-ADD2-BATCH-006 | [E2E] | rgs-batch-backend::connector | 3 | 5 域 gRPC client (mTLS 业务级, 5 域 5/5 OK) | NFR-32 |

### 6.2 admin-coc §X 集成设计 (v0.2 新增)

per 9/5 ae9702d admin-coc Phase B DDD Review + 9/5 21:17 JST 7 项 admin 域 Lead 真实签字:

| 用例 ID | 试验级别 | 场景 | 测试目的 | evidence |
| --- | --- | --- | --- | --- |
| TST-ST-04-ADD2-ADMIN-COC-001 | [E2E] | admin-coc §X GM 命令 coc_policy 决策树 | 7 项 admin 域 Lead 真实签字 | ae9702d / 3695f3b |
| TST-ST-04-ADD2-ADMIN-COC-002 | [E2E] | coc_policy 决策树 3 场景 | 1101/1102/1103 错误码 | 3695f3b coc_policy UT |

### 6.3 plugin 集群 4 阶段用例 (v0.2 新增)

per 9/5 61cf306 RGS plugin 集群 + app 集群架构 v0.1 + 9/5 ARCH §4 4 阶段用例:

| 用例 ID | 试验级别 | 阶段 | 测试目的 | evidence |
| --- | --- | --- | --- | --- |
| TST-ST-04-ADD2-PLUGIN-001 | [E2E] | 阶段 0 mock (已完成) | PoC plugin 抽卡概率 hot-swap 不重启 app | 5cfd692 + 9/5 ARCH §4.1 |
| TST-ST-04-ADD2-PLUGIN-002 | [E2E] | 阶段 1 MVP (~2-3 周) | PG registry 持久化 + 9 域共享 | 9/5 ARCH §4.2 |
| TST-ST-04-ADD2-PLUGIN-003 | [E2E] | 阶段 2 (~3-4 周) | 每 app 独立更新 + 双 registry 模式 | 9/5 ARCH §4.3 |
| TST-ST-04-ADD2-PLUGIN-004 | [E2E] | 阶段 3 平台化 (~4-6 周) | 平台化 + 完整 WBS 5-10 task | 9/5 ARCH §4.4 |

### 6.4 rgs-flash-mock v0.3 + 4 NEW 回归脚本 (v0.2 新增)

per 9/4 RGS-FLASH-MOCK-DESIGN-2026-09-04 v0.3 + 9/6 W3 5 报告:

| 用例 ID | 试验级别 | 脚本 | 覆盖范围 | evidence |
| --- | --- | --- | --- | --- |
| TST-ST-04-ADD2-MOCK-001 | [E2E] | regression-test-9-domain-mtls.sh | 9 域 mTLS 业务级 11 步 | d270ab9 + 9/6 v0.3 |
| TST-ST-04-ADD2-MOCK-002 | [E2E] | regression-test-batch-domain.sh | batch 域 6 module × 15 用例 | 9/1 batch 4 件套 |
| TST-ST-04-ADD2-MOCK-003 | [E2E] | regression-test-8-domain-extension.sh | 8 域扩展 12 module | 9/6 8 域扩展 |
| TST-ST-04-ADD2-MOCK-004 | [E2E] | regression-test-admin-coc.sh | admin-coc Phase B 7 项 + coc_policy 3 场景 | ae9702d / 3695f3b |

### 6.5 9 域 mTLS 业务级 E2E 11 步 v3 (v0.2 新增)

per 9/6 d270ab9 9 域 mTLS 端到端 11 步客户端模拟器 v3 + d15a0bb 3 NEW 域 k8s yaml:

| 用例 ID | 试验级别 | 步骤范围 | 测试目的 | evidence |
| --- | --- | --- | --- | --- |
| TST-ST-04-ADD2-MTLS-9D-001 | [TL-8] | 步骤 1-3: player → economy → match gRPC mTLS 握手 + 业务调用 | 3/3 域 mTLS 业务级 OK | d270ab9 + d15a0bb 3 NEW 域 k8s yaml |
| TST-ST-04-ADD2-MTLS-9D-002 | [TL-8] | 步骤 4-6: social → admin → batch gRPC mTLS 握手 + 业务调用 | 3/3 域 mTLS 业务级 OK | d270ab9 + 3c79bca batch 6 域注册 |
| TST-ST-04-ADD2-MTLS-9D-003 | [TL-8] | 步骤 7-9: scene / battle / network (8 域扩展 NEW) gRPC mTLS | 3/3 域 mTLS 业务级 OK | 57edbeb / b6b19b7 / 1dd9afc 8 域扩展 |
| TST-ST-04-ADD2-MTLS-9D-004 | [TL-8] | 步骤 10-11: 9 域跨域 saga mTLS 业务级 E2E | 11/11 步 PASS, 0 mTLS 握手失败, 0 业务错 | d270ab9 + 3ce36f0 (RGS-TEST-CASES v0.2) |

### 6.6 派生约束守护 (v0.2 增补, per AGENTS.md + 9/2 D2 拍板 + 9/1 batch 12 派生约束)

| 派生约束 | 内容 | v0.2 状态 |
|---|---|---|
| **L1** | cargo check --tests 0 error (限时 60s) | ✅ 文档类工作 N/A |
| **L11** | cargo build dir lock (PT 派工 1 次拿 status) | ✅ N/A (文档) |
| **L12.1** | 临时 log / .txt / .tmp_search* 不入 commit | ✅ worktree 根 0 临时文件 |
| **L12.2** | 5 worker 派工 3 选项 | ✅ 1 worker 1 worktree v02/st5 |
| **L13** | 自指字段 deferred 实时查询 (git log + grep 实证) | ✅ v0.2 全文 git log 实证 |
| **L14** | plumbing 节点字符串 brace 跟踪 | ✅ N/A (文档) |
| **B3** | DDD Review 二审流程 (per 9/2 10:18 JST 拍板) | ⏳ Mavis 自审 1 次停手 → Ulysses 二审必到 |
| **5 域独立 Lead** | per 2026-08-21 JST | ✅ 维持, 扩展 6 域 (5 + batch) |
| **凭据永不打印** | per 8/27 11:06 JST 硬 ban | ✅ 全文 0 env value |
| **缺标比错标** | per 8/26 JST DTL-036 v1.4 hotfix 复盘 | ✅ §6.7 已知缺口 6 项显式列 |
| **不追溯改写** | per 8/27 JST 禁回溯叙事 | ✅ v0.1 → v0.2 显式升版, 不 amend 历史 |
| **Mavis 默认代签 Ulysses** | per 8/27 19:39/20:56/21:59 JST 三次强化 | ✅ author / 审批 / 修订人 三行齐全 |
| **9/1 batch 域 12 派生约束** | per AGENTS.md §7.2 | ✅ §6.1 batch v0.1 6 module × 15 用例 |
| **9/1 14:58 JST 拍板选项规则** | ask_user 给 Ulysses 选项, 不能直接做 | ✅ v0.2 拍板走 ask_user |
| **9/4 17:47 JST 测试脚本+数据归入 mock 项目** | per RGS-FLASH-MOCK-DESIGN-2026-09-04 v0.3 | ✅ §6.4 4 NEW 回归脚本 |
| **9/5 04:03 JST 拍板后立即执行** | 推荐项被选后立即执行 | ✅ 拍板后直接落地 |
| **cutover L15-L23** | per 9/6 6c6839e | ✅ 全文引用 (8 维度增量表) |
| **L19** | mTLS 业务级 = saga 触达 | ✅ §6.5 9 域 mTLS 11 步 v3 |
| **L20** | ca.crt 0 字节空文件陷阱 | ✅ §6.5 evidence d270ab9 |
| **L22** | 协议码 → gRPC method 映射表 | ✅ §6.5 9 域 RPC 映射 |

### 6.7 已知缺口 (v0.2 增补, per 8/26 JST 缺标比错标 6 项)

1. **L3 drill LCM/chaos/NFR/risk 4 类**: cluster-ops/tests-disabled/ 仍禁用, 需 INC-002 saga 编译死锁修复后重启用
2. **batch v0.2 评估 12 GAP**: per RGS-BATCH-V0.2-EVAL-2026-09-03 v0.1 GAP-1~12 — v0.2 不集成, v0.2 评估期补全
3. **RACI v1.1 → v1.2**: 5 域扩展 6 域 (5 + batch) 待 DDD Review 阶段补
4. **plugin PoC 1 个 WASM (draw_card_probability)**: per 9/5 5cfd692 已落 v0.1 PoC, 阶段 1 MVP 详细 WBS 5-10 task 待 DDD Review 阶段补
5. **9 域 mTLS 业务级详细 yaml**: per 9/6 d15a0bb 3 NEW 域 yaml 落档, 剩余 6 域 yaml 待 DDD Review 阶段补
6. **ST 业务级 E2E 100k CCU 性能**: ST §5 NFR-PE-001~006 待 100k 阶段实跑

### 6.8 修订历史 (v0.2 升版)

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-21 | Ulysses (一人公司 12 角色 per DEC-008) | 初版制定 (审批栏 Ulysses 集体签字 per DEC-008) |
| 0.2 (字段级深化) | 2026-08-21 | 架构师 | 字段级深化 + ADR 决策验证 + TBD 处置 |
| **v0.2 (Round 2 W5 升版)** | 2026-09-07 | Ulysses — Mavis 接手 (per 8/27 19:39/20:56/21:59 JST 三次强化) | **Round 2 W5 v0.2 升版 (综合 8 维度 + ST 特定增量)**: 1) §0 增 8 维度增量表 2) §1 v0.2 增补 envoy 独立 deployment 偏好 + 9 域 mTLS + rgs-flash-mock v0.3 3) §6 增 6.1~6.5 ST 特定增量用例 (batch / admin-coc / plugin / mock / 9 域 mTLS) 4) §6.6 派生约束守护段增 L15-L23 + 9/1 batch 12 派生约束 + B3 二审流程 |

---

> 与 RGS-TST-ST-04 + RGS-TST-ST-04-ADD1 共存。
