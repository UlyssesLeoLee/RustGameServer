# 系统测试设计書（システムテスト設計書 / System Test Design Document）

**主题域 04 客户端与SDK — CDN 边缘策略（补强）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-TST-ST-04-ADD1 |
| 版本 | 0.2 |
| 父文档 | RGS-REQ-030-ADD1 v0.2 + RGS-DTL-027 v0.2 §6.3〜§6.4 |
| V模型层级 | TL-6 负载 / TL-7 故障注入 |
| 制定日 | 2026-08-19 |

---

## 1. 目的

端到端验证 CDN 边缘策略在 100k CCU 下的表现。

## 2. 测试用例

| 用例 ID | 试验级别 | 测试目的 |
|---|---|---|
| TST-ST-04-C001 | [TL-6] | 100k CCU CDN 命中延迟 p99 < 50ms（AC-CDN-101） |
| TST-ST-04-C002 | [TL-7] | 边缘节点宕机 → 已批准源站继续服务 |
| TST-ST-04-C003 | [E2E] | 灰度 100% 切回 ≤ 30s（AC-CDN-103） |
| TST-ST-04-C004 | [E2E] | 强制更新判定 < 100ms（AC-CDN-104） |
| TST-ST-04-C005 | [E2E] | 100% 资源签名校验（AC-CDN-105） |
| TST-ST-04-C006 | [E2E] | 命中率 ≥ 80% manifest / 95% patch（NFR-CDN-101） |
| TST-ST-04-C007 | [E2E] | 回源成功率 ≥ 99.9%（NFR-CDN-102） |
| TST-ST-04-C008 | [E2E] | 生产激活后端 100% 具备 BOM、许可证/商业条款审查与 ADR（NFR-CDN-105／AC-CDN-106） |
| TST-ST-04-C009 | [E2E] | 跨 region 一致性 |
| TST-ST-04-C010 | [E2E] | 强制更新全网生效 |

## 3. 最小可复现实验

### 3.1 固定基线与取证规则

| 项目 | 固定条件 |
|---|---|
| 拓扑/规格 | 3 个 region，每区 3 个边缘缓存节点（8 vCPU、16 GiB RAM），每区至少 2 个经 `ApprovedBackendProfile` 批准的源站实例（16 vCPU、64 GiB RAM）；测试 profile 必须包含 BOM、许可证/商业条款审查和 ADR 引用。 |
| 数据集与负载模型 | 10,000 份 manifest、1,000 个 patch 版本和 50 GiB 不可变资源；100,000 虚拟客户端按 40%／35%／25% 分布在三个 region。请求混合为 60% manifest、30% patch、5% 资源签名校验、5% 强制更新/灰度查询。 |
| 预热与持续时间 | 先预热 15 分钟以填满缓存，再持续 60 分钟；灰度/强制更新在第 20 分钟执行，故障注入在第 35 分钟执行。 |
| 故障注入 | 分别隔离一个边缘节点、使一个批准源站返回 5xx 60 秒、以及提交缺失审批引用的 profile；不得以未批准候选作为回源替代。 |
| 采样/SLO计算 | 每请求记录 region、cache-status、profile ID、版本、结果码与 HDR histogram。命中 p99 为预热后每 1 分钟窗口 cache-status=HIT 的最差 p99；命中率为 HIT/(HIT+MISS)；回源成功率为成功回源/回源尝试。 |
| 原始证据路径 | `artifacts/test-results/TST-ST-04-ADD1/<run-id>/<case-id>/{topology.yaml,profile.json,load.hdr,edge-access.parquet,origin-access.parquet,events.jsonl,summary.json}`；`summary.json` 必须含 profile 审批引用、镜像 digest 与起止时间。 |
| 清理步骤 | 导出并校验原始证据后，停止负载、撤销网络/5xx 注入、恢复 stable channel、失效测试缓存、删除临时资源和凭据；保留 evidence 目录与审批 profile 快照。 |

### 3.2 用例执行矩阵与可判定预期

| 用例 | 拓扑、数据与负载 | 预热/持续与故障注入 | 可判定预期 |
|---|---|---|---|
| C001 | 3 region 基线、100,000 客户端，60% manifest/30% patch | 15m/60m，无故障 | 预热后每个 region 的 HIT 最差 1 分钟窗口 p99 < 50ms。 |
| C002 | 同基线 | 第 35 分钟隔离一个边缘节点 | 请求经批准源站持续服务；源站回源成功率按 C007 口径 ≥ 99.9%。 |
| C003 | 同基线，canary 开始有 100% 流量 | 第 20 分钟提交 stable=100% 配置 | 从配置接受到三 region 的 canary 观测流量为 0 的最长时间 ≤ 30s。 |
| C004 | 同基线，5% 强制更新查询 | 15m/60m，无故障 | 客户端强制更新判定的最差 1 分钟窗口 p99 < 100ms。 |
| C005 | 同基线，5% 签名校验 | 15m/60m，注入一份篡改资源 | 合法资源签名校验通过率 100%，篡改资源拒绝率 100%。 |
| C006 | 同 C001 | 15m/60m，无故障 | manifest 命中率 ≥ 80%，patch 命中率 ≥ 95%，按完整持续窗口复算。 |
| C007 | 同基线 | 第 35 分钟使一个批准源站返回 5xx 60s | 完整持续窗口回源成功率 ≥ 99.9%，且未使用未批准候选回源。 |
| C008 | 任一基线 profile | 部署前提交 profile 正/负样本 | 所有激活 profile 的 BOM、许可证/商业条款审查和 ADR 引用均非空且状态为 Approved；缺任一项的部署 100% 被拒绝。 |
| C009 | 同基线，三 region 同版本数据 | 15m/60m，无故障 | 相同 `{channel,version,region,file}` 的响应 checksum 在相同 region 内一致率 100%；跨 region 仅返回各自批准配置的目标版本。 |
| C010 | 同基线，5% 强制更新查询 | 第 20 分钟发布新的 `min_supported_version` | 三 region 的观测版本在 30s 内一致，旧版本请求均返回强制更新引导。 |

## 4. 追溯性

| AC | 用例 |
|---|---|
| AC-CDN-101 | TST-ST-04-C001 |
| AC-CDN-102 | TST-ST-04-C002 |
| AC-CDN-103 | TST-ST-04-C003 |
| AC-CDN-104 | TST-ST-04-C004 |
| AC-CDN-105 | TST-ST-04-C005 |
| AC-CDN-106 | TST-ST-04-C008 |
| NFR-CDN-101~105 | TST-ST-04-C006~C010 |

## 5. 通过判定

- AC-CDN-101~106 全部通过
- 100k CCU 延迟达标
- 0 高优事故

---

> 与 RGS-TST-ST-04 共存。
