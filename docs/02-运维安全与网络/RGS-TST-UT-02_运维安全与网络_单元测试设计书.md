# 单元测试设计書（単体テスト設計書 / Unit Test Design Document）

**主题域 02 运维安全与网络 — 单元测试（UT）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-TST-UT-02 |
| 版本 | 0.2 |
| 父文档 | RGS-DTL-003/004/005/006/017/018/020/021 详细设计书 |
| 本主题域源文档全集（REQ/BAS/DTL） | RGS-REQ-007、RGS-REQ-008、RGS-REQ-009、RGS-REQ-010、RGS-REQ-020、RGS-REQ-021、RGS-REQ-023、RGS-REQ-024、RGS-BAS-003、RGS-BAS-004、RGS-BAS-005、RGS-BAS-006、RGS-BAS-017、RGS-BAS-018、RGS-BAS-020、RGS-BAS-021、RGS-DTL-003、RGS-DTL-004、RGS-DTL-005、RGS-DTL-006、RGS-DTL-017、RGS-DTL-018、RGS-DTL-020、RGS-DTL-021 |

| V模型层级 | TL-1 单元试验 ↔ DTL 详细设计 |
| 依据标准 | IPA『共通フレーム 2013』詳細設計工程 |
| 制定日 | 2026-08-19 |
| 制定者 | 架构师（自动化产出） |

---

## 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-19 | 架构师 | 初版制定
| **0.2** | 2026-08-19 | 架构师 | **字段级深化**：每条用例的"对应设计"列升级为"文档 ID + §X.Y + 表/图/字段"；新增"ADR 决策验证"小节覆盖本主题 ADR；新增"TBD 处置"小节 |。覆盖 GM 后台管控、埋点日志、插件热插拔、网络安全、网络拓扑容灾、账号身份合规、平台内购、GM 拓扑画布的物理/实现级测试 |

## 审批栏

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 | 2026-08-19 | — |
| 评审（技术） | | | DTL 物理设计一致性 |
| 评审（QA） | | | QA-001 覆盖率 |
| 审批（负责人） | | | 本测试设计书的基准化 |

---

## 目次（目次 / Table of Contents）

1. 前言（はじめに / Preface）
   1.1 目的（目的 / Purpose）
   1.2 适用范围（適用範囲 / Scope）
   1.3 关联文档（関連文書 / Related Documents）
   1.4 记述规则（記述規則 / Notation Rules）
   1.5 字段级映射说明
   1.6 命名约定（命名規約 / Naming Convention）
2. 测试策略（テスト戦略 / Test Strategy）
3. 测试用例（テストケース / Test Cases）
4. 追溯性矩阵（トレーサビリティ・マトリクス / Traceability Matrix）
5. 测试执行计划（テスト実行計画 / Test Execution Plan）
6. 通过判定基准（合格判定基準 / Pass Criteria）
7. 风险与未决事项（リスクと未決事項 / Risks and TBDs）

注：本文档实际章节以文中二级标题为准。


## 1. 前言

## 1.1 目的（目的 / Purpose）

TL-1 单元试验层级，对应主题 02 的 8 份详细设计书，覆盖运维管控、安全、网络、容灾、合规、画布等模块的函数/类型级正确性。

## 1.2 适用范围（適用範囲 / Scope）

| 范畴 | 说明 |
|---|---|
| 适用 | 本主题域内父文档所定义的全部功能/非功能需求 |
| 不适用 | 其他主题域的功能（见各主题 ST/IT/UT 设计书） |

## 1.3 关联文档（関連文書 / Related Documents）

| 文档编号 | 关系 |
|---|---|
| RGS-BAS-003/004/005/006/017/018/020/021 | 父基本设计 |
| RGS-REQ-007/008/009/010/020/021/023/024 | 父需求 |
| RGS-TST-UT-00/01 | 跨主题 |

**本主题域源文档全集**：
- REQ: RGS-REQ-007, RGS-REQ-008, RGS-REQ-009, RGS-REQ-010, RGS-REQ-020, RGS-REQ-021, RGS-REQ-023, RGS-REQ-024
- BAS: RGS-BAS-003, RGS-BAS-004, RGS-BAS-005, RGS-BAS-006, RGS-BAS-017, RGS-BAS-018, RGS-BAS-020, RGS-BAS-021
- DTL: RGS-DTL-003, RGS-DTL-004, RGS-DTL-005, RGS-DTL-006, RGS-DTL-017, RGS-DTL-018, RGS-DTL-020, RGS-DTL-021

## 1.4 记述规则（記述規則 / Notation Rules）

### 1.4.1 强度用语（强度表現 / Strength of Expression）

本文档遵循 RFC 2119 与 IPA 共通フレーム 2013 规定的强度用语：

| 中文表述 | 日文表述 | 英文 | 强度 | 含义 |
|---|---|---|---|---|
| **必须** | 必ず / 必須 | MUST | 强 | 必要条件。未满足则不予验收 |
| **应当** | すべき / 推奨 | SHOULD | 中 | 推荐条件。未满足时必须记录理由并取得批准 |
| **不得** | してはならない / 禁止 | MUST NOT | 强 | 禁止事项。违反即为设计缺陷 |
| **可以** | してもよい / 任意 | MAY | 弱 | 任意条件。是否实现不影响验收 |

### 1.4.2 优先级符号

| 符号 | 中文 | 日文 | 含义 |
|---|---|---|---|
| ◎ | 必须 | 必須 | 商用上线前必须实现 |
| ○ | 推荐 | 推奨 | 商用上线前应当实现 |
| △ | 任意 | 任意 | 上线后追加实现 |
| × | 范围外 | 範囲外 | 本次范围外 |

### 1.4.3 标识符体系

本文档遵循 RGS-REQ-001 §1.5.3 既定标识符体系：
- `RGS-TST-XX-NNN` 测试用例编号
- `RGS-{REQ|BAS|DTL}-NNN` 父文档编号
- `RGS-ADR-NNNN` 架构决策记录编号
- `NFR-<区分>-NNN` 非功能需求编号
- `AC-NNN` / `VF-NNN` / `FT-NNN` 验收/验证/故障注入编号
- `BZ-NNN` 业务规则编号
- `ST-NNN` 状态机编号

### 1.4.4 引用约定

- 全部引用以编号（如 `RGS-REQ-006`）而非文件路径
- 同一编号在本文档中首次出现时附全称，后续仅用编号

## 1.5 字段级映射说明

本版本（0.2）的核心升级是**字段级映射**：每条测试用例的"对应设计"列从"§X.Y 章节名"升级为"文档 ID + §X.Y + 表/图/字段"。

**映射规则**：
- 每个测试模块对应 1 个或多个父文档的物理/实现级章节
- 每条用例精确引用其父文档的具体字段（如 DDL 字段、gRPC 方法字段、状态机迁移名）
- 模块汇总表（§2.2）给出该文档验证的字段清单与覆盖率目标

**V 模型强化对应**：本文档对应该主题父基本设计书与详细设计书，构成"V 字"右侧的 TL-1/2/3 单元素验证。

## 1.6 命名约定（命名規約 / Naming Convention）

- 用例 ID：`TST-{UT|IT|ST}-XX-NNN`（XX 为主题编号 00-07）
- 试验级别标注：UT 无标注 / IT 用 [TL-2/3/4/5] / ST 用 [TL-6/7/8/E2E]
- 覆盖类型：N=正常 / A=异常 / B=边界 / P=属性不变条件 / S=状态机非法迁移
- 运行时机：`cargo test --workspace`（主干 CI 必跑，QA-006 ≤ 15 min 约束内）


## 2. 测试策略

```
需求 RGS-REQ-007/008/009/010/020/021/023/024  ┐ ST
基础 RGS-BAS-003/004/005/006/017/018/020/021  ┐ IT
详细 RGS-DTL-003/004/005/006/017/018/020/021  ┐ UT  ★ RGS-TST-UT-02 ★
实现 Rust 源码                              ┘
```

覆盖率目标：核心 80%、属性 1000 次、状态机 100%。

---

## 3. 测试用例

## 3.1 模块 A：GM 后台管控（DTL-003）

| 用例 ID | 对应设计 | 测试目的 | 覆盖类型 |
|---|---|---|---|
| TST-UT-02-001 | §3 AdminService | BanAccount 字段级 | N |
| TST-UT-02-002 | §3 | KickSession 强制下线 | N |
| TST-UT-02-003 | §3 | MuteChat 禁言 | N |
| TST-UT-02-004 | §3 | GrantCompensation 补偿 | N |
| TST-UT-02-005 | §3 | SetMaintenanceMode | N |
| TST-UT-02-006 | §3 | ReloadConfigTable 数值表 | N |
| TST-UT-02-007 | §3 | RequestSceneRestart 二次确认 | N |
| TST-UT-02-008 | §3 | ConfirmSceneRestart | N |
| TST-UT-02-009 | §3 | QueryOnlineStatus | N |
| TST-UT-02-010 | §3 | QuerySceneMetrics | N |
| TST-UT-02-011 | §3 | QueryAuditLog | N |
| TST-UT-02-012 | §3 | CreateOpsTicket 工单 | N |
| TST-UT-02-013 | §4 RBAC | 角色权限矩阵正确 | N |
| TST-UT-02-014 | §4 RBAC | 越权调用被拒 | A |
| TST-UT-02-015 | §5 维护模式 | 拒绝新连接 | A |
| TST-UT-02-016 | §5 维护模式 | 通知已连接玩家 | N |
| TST-UT-02-017 | §6 Webhook 签名 | 合法签名通过 | N |
| TST-UT-02-018 | §6 Webhook 签名 | 伪造签名被拒 | A |
| TST-UT-02-019 | §6 Webhook 重放 | 同 nonce 拒绝 | A |
| TST-UT-02-020 | §7 告警规则 | 阈值触发 | B |
| TST-UT-02-021 | §7 告警规则 | 抑制时段不重复 | B |
| TST-UT-02-022 | §8 ops_ticket 状态机 | 合法迁移 | N |
| TST-UT-02-023 | §8 状态机 | Closed→Open 非法 | S |
| TST-UT-02-024 | §9 爆炸半径 | 误操作不冲击实时 | A |

## 3.2 模块 B：埋点日志（DTL-004）

| 用例 ID | 对应设计 | 测试目的 | 覆盖类型 |
|---|---|---|---|
| TST-UT-02-030 | §3 指标命名 | rgs_* 前缀 | N |
| TST-UT-02-031 | §3 指标 | 4 黄金指标存在 | N |
| TST-UT-02-032 | §4 span 命名 | span.name 符合 | N |
| TST-UT-02-033 | §4 trace 字段 | 6 ID 完整 | N |
| TST-UT-02-034 | §5 脱敏 | 个人信息 redact | A |
| TST-UT-02-035 | §5 脱敏 | 凭证 redact | A |
| TST-UT-02-036 | §6 采样 | 默认 10% | B |
| TST-UT-02-037 | §6 强制全采集 | 关键路径 100% | N |
| TST-UT-02-038 | §7 静态检查 | 禁用 println | A |
| TST-UT-02-039 | §7 静态检查 | span ID 必填 | A |
| TST-UT-02-040 | §8 trace_sample_ratio | 配置加载 | N |
| TST-UT-02-041 | §9 日志格式 | JSON 结构化 | N |
| TST-UT-02-042 | §9 关联 ID 传播 | 全链路 | P |
| TST-UT-02-043 | §10 日志聚合 | 批量发送 | N |
| TST-UT-02-044 | §10 日志 | 失败重试 | A |

## 3.3 模块 C：插件热插拔（DTL-005）

| 用例 ID | 对应设计 | 测试目的 | 覆盖类型 |
|---|---|---|---|
| TST-UT-02-050 | §3 注册表 | 插件登记 | N |
| TST-UT-02-051 | §3 沙箱脚本 | Rhai 加载 | N |
| TST-UT-02-052 | §3 白名单 | 调白名单函数 OK | N |
| TST-UT-02-053 | §3 白名单 | 调非白名单被拒 | A |
| TST-UT-02-054 | §4 状态机 | Registered→Enabled→Disabled→Unregistered | N |
| TST-UT-02-055 | §4 状态机 | Registered→Enabled 跳级被拒 | S |
| TST-UT-02-056 | §5 跨节点同步 | 多节点最终一致 | P |
| TST-UT-02-057 | §6 回滚 | 启停可逆 | N |
| TST-UT-02-058 | §6 回滚 | 数据回滚 | N |
| TST-UT-02-059 | §7 故障隔离 | 插件 panic 不崩宿主 | A |
| TST-UT-02-060 | §7 断路器 | 阈值触发 | B |
| TST-UT-02-061 | §8 资源限制 | CPU/内存上限 | B |
| TST-UT-02-062 | §9 拒绝动态链接库 | dlopen 拒绝 | A |
| TST-UT-02-063 | §10 插件事件 | 产生经 EC-003 路径 | N |
| TST-UT-02-064 | §10 插件事件 | 直接写 DB 拒 | A |

## 3.4 模块 D：网络安全（DTL-006）

| 用例 ID | 对应设计 | 测试目的 | 覆盖类型 |
|---|---|---|---|
| TST-UT-02-070 | §3 NetworkPolicy | default-deny | N |
| TST-UT-02-071 | §3 NetworkPolicy | 白名单最小化 | N |
| TST-UT-02-072 | §4 DDoS | OpenResty+Coraza | N |
| TST-UT-02-073 | §4 WAF 规则 | OWASP CRS | N |
| TST-UT-02-074 | §4 WAF | SQL 注入拦截 | A |
| TST-UT-02-075 | §4 WAF | XSS 拦截 | A |
| TST-UT-02-076 | §4 WAF | 误报率 | P |
| TST-UT-02-077 | §5 密钥轮换 | 双密钥并存 | N |
| TST-UT-02-078 | §5 密钥轮换 | 旧密钥拒 | A |
| TST-UT-02-079 | §5 OpenBao | 密钥读写 | N |
| TST-UT-02-080 | §6 供应链 SBOM | 生成 | N |
| TST-UT-02-081 | §6 漏洞扫描 | High 检出 | N |
| TST-UT-02-082 | §6 CVE 修复 | 14 天内 | B |
| TST-UT-02-083 | §7 限流 | NFR-SEC-008 阈值 | B |
| TST-UT-02-084 | §7 账号级限流 | 1k/小时 | B |
| TST-UT-02-085 | §7 IP 限流 | 100/分 | B |
| TST-UT-02-086 | §7A QUIC 地址验证 | 假 IP 拒 | A |
| TST-UT-02-087 | §7A 资源配额 | 越界拒 | A |
| TST-UT-02-088 | §8 应急响应 | runbook 可执行 | N |
| TST-UT-02-089 | §8 事件定级 | P0~P3 | N |
| TST-UT-02-090 | §8 通报 | 24x7 | N |

## 3.5 模块 E：网络拓扑容灾（DTL-017）

| 用例 ID | 对应设计 | 测试目的 | 覆盖类型 |
|---|---|---|---|
| TST-UT-02-100 | §3 单区域 Multi-AZ | 拓扑结构 | N |
| TST-UT-02-101 | §3 故障切换 | AZ 故障自动切 | A |
| TST-UT-02-102 | §3 RTO | 30min | N |
| TST-UT-02-103 | §3 RPO | 0 (Lv.4) | N |
| TST-UT-02-104 | §4 AnalyticsStore | 配置加载 | N |
| TST-UT-02-105 | §4 BI 工具 | 部署配置 | N |
| TST-UT-02-106 | §5 数据流时序 | 读写分离 | N |
| TST-UT-02-107 | §5 连接配额 | 1/2/3 三档 | B |

## 3.6 模块 F：账号身份合规（DTL-018）

| 用例 ID | 对应设计 | 测试目的 | 覆盖类型 |
|---|---|---|---|
| TST-UT-02-110 | §3 身份联合 | OIDC 流程 | N |
| TST-UT-02-111 | §3 Apple 登录 | id_token 校验 | N |
| TST-UT-02-112 | §3 Google 登录 | id_token 校验 | N |
| TST-UT-02-113 | §3 Steam 登录 | ticket 校验 | N |
| TST-UT-02-114 | §3 伪造 token | 拒绝 | A |
| TST-UT-02-115 | §3 token 过期 | 拒绝 | A |
| TST-UT-02-116 | §4 游客进度 | 本地保存 | N |
| TST-UT-02-117 | §4 游客升级 | 绑定正式账号 | N |
| TST-UT-02-118 | §5 账号绑定 | 多个 IdP | N |
| TST-ST-02-119 | §5 解绑 | 至少 1 个 IdP | N |
| TST-UT-02-120 | §6 实名认证 | 流程完整 | N |
| TST-UT-02-121 | §6 ComplianceRuleSet | 地区取值 | N |
| TST-UT-02-122 | §7 未成年人 | 防沉迷 | N |
| TST-UT-02-123 | §7 消费限制 | 金额上限 | B |
| TST-UT-02-124 | §7 游戏时间 | 时段限制 | B |

## 3.7 模块 G：平台内购（DTL-020）

| 用例 ID | 对应设计 | 测试目的 | 覆盖类型 |
|---|---|---|---|
| TST-UT-02-130 | §3 iOS 收据 | 校验 | N |
| TST-UT-02-131 | §3 Google 收据 | 校验 | N |
| TST-UT-02-132 | §3 伪造收据 | 拒 | A |
| TST-UT-02-133 | §3 重复收据 | 幂等 | P |
| TST-UT-02-134 | §4 退款追回 | TBD-PLT-001 | A |
| TST-UT-02-135 | §5 选服 | 按 region 路由 | N |
| TST-UT-02-136 | §5 选服 | 满服拒绝 | A |
| TST-UT-02-137 | §6 合服 | 流程 | N |
| TST-UT-02-138 | §6 分服 | 流程 | N |
| TST-UT-02-139 | §6 合服幂等 | 重复执行 | P |
| TST-UT-02-140 | §6 数据校验 | 跨服差分 | P |

## 3.8 模块 H：GM 拓扑画布（DTL-021）

| 用例 ID | 对应设计 | 测试目的 | 覆盖类型 |
|---|---|---|---|
| TST-UT-02-150 | §3 拓扑聚合 | App 间流 | N |
| TST-UT-02-151 | §3 颗粒度切换 | 方法级 | N |
| TST-UT-02-152 | §3 颗粒度切换 | 插件级 | N |
| TST-UT-02-153 | §3 颗粒度切换 | App 级 | N |
| TST-UT-02-154 | §4 LangGraph 节点 | 渲染 | N |
| TST-UT-02-155 | §4 LangGraph 边 | 渲染 | N |
| TST-UT-02-156 | §4 节点聚类 | TBD-VIZ-002 | N |
| TST-UT-02-157 | §5 业务视图 | 声明式 | N |
| TST-UT-02-158 | §5 视图权限 | RBAC | N |
| TST-UT-02-159 | §6 实时性 | 增量更新 | N |
| TST-UT-02-160 | §6 大规模 | 1k 节点 | B |

## 3.9 业务规则与状态机

| 用例 ID | 对应需求 | 测试目的 | 覆盖类型 |
|---|---|---|---|
| TST-UT-02-200 | BZ-001 | 货币非负 | P |
| TST-UT-02-201 | BZ-005 | 已归档对局不可变 | P |
| TST-UT-02-202 | BZ-006 | 封禁账号不可建会话 | P |
| TST-UT-02-203 | NFR-SE-002 | TLS 1.3 | N |
| TST-UT-02-204 | NFR-SE-006 | 输入校验 | N |
| TST-UT-02-205 | NFR-SE-010 | 审计不可删 | A |
| TST-UT-02-210 | ARC-005 旧 epoch | 拒绝 | A |
| TST-UT-02-211 | ARC-006 ACK 边界 | DB 写前不 ACK | A |
| TST-UT-02-212 | ARC-007 实时不同步 DB | 0 SQL | N |
| TST-UT-02-213 | ARC-013 死锁 | 静态检查 | A |
| TST-UT-02-214 | ARC-014 中间件 | 需 ADR | A |
| TST-UT-02-215 | ARC-022 拒绝 SaaS | 部署审计 | N |

---

## 4. 追溯性矩阵

| 详细设计 | 用例范围 |
|---|---|
| DTL-003 GM 后台 | TST-UT-02-001〜024 |
| DTL-004 埋点 | TST-UT-02-030〜044 |
| DTL-005 插件 | TST-UT-02-050〜064 |
| DTL-006 网络安全 | TST-UT-02-070〜090 |
| DTL-017 容灾 | TST-UT-02-100〜107 |
| DTL-018 身份合规 | TST-UT-02-110〜124 |
| DTL-020 平台内购 | TST-UT-02-130〜140 |
| DTL-021 画布 | TST-UT-02-150〜160 |
| BZ-* NFR-* ARC-* | TST-UT-02-200〜215 |
| AC-008 FT-001~010 | 跨域 |
| AC-015 OSI 100% | TST-UT-02-080 |
| AC-019 领域验收 | 跨域 |

---

## 5. 测试执行计划

| 触发 | 范围 | 时限 |
|---|---|---|
| commit | 受影响 crate | < 30s |
| PR | 全 workspace | < 5 min |
| 合并 | 全 + 属性 1000 | < 10 min |
| nightly | + 10000 次 | 不阻塞 |

## 6. 通过判定基准

- 全部用例 PASS
- 核心代码语句覆盖 ≥ 80%
- BZ-* 1000 次无失败
- ST-* 100% 拒绝
- 静态检查 0 命中（CON-008 / ARC-013 / ARC-022）


## 6.5 NFR 覆盖索引

本主题域覆盖的非功能需求编号全集（按 RGS-REQ-003 等级 Lv.2/3/4 全覆盖）：

- **NFR-AV-***：NFR-AV-001, NFR-AV-002, NFR-AV-003, NFR-AV-004, NFR-AV-007, NFR-AV-008, NFR-AV-009, NFR-AV-010
- **NFR-DBS-***：NFR-DBS-001, NFR-DBS-002, NFR-DBS-003, NFR-DBS-010, NFR-DBS-011, NFR-DBS-020, NFR-DBS-021, NFR-DBS-022, NFR-DBS-040, NFR-DBS-041
- **NFR-EN-***：NFR-EN-003
- **NFR-GM-***：NFR-GM-001, NFR-GM-002, NFR-GM-003, NFR-GM-004, NFR-GM-010, NFR-GM-011, NFR-GM-012, NFR-GM-013, NFR-GM-020, NFR-GM-021, NFR-GM-022, NFR-GM-023, NFR-GM-024, NFR-GM-025, NFR-GM-030, NFR-GM-031, NFR-GM-032
- **NFR-IDN-***：NFR-IDN-001, NFR-IDN-002, NFR-IDN-003, NFR-IDN-004
- **NFR-INF-***：NFR-INF-001, NFR-INF-002, NFR-INF-003, NFR-INF-004, NFR-INF-005, NFR-INF-006
- **NFR-LOG-***：NFR-LOG-001, NFR-LOG-002, NFR-LOG-003, NFR-LOG-004, NFR-LOG-005, NFR-LOG-010, NFR-LOG-011, NFR-LOG-012, NFR-LOG-013, NFR-LOG-020, NFR-LOG-021, NFR-LOG-022, NFR-LOG-040
- **NFR-MI-***：NFR-MI-005
- **NFR-OP-***：NFR-OP-001, NFR-OP-002, NFR-OP-003, NFR-OP-004, NFR-OP-005, NFR-OP-006, NFR-OP-008, NFR-OP-010
- **NFR-OPS-***：NFR-OPS-001, NFR-OPS-002, NFR-OPS-003, NFR-OPS-004
- **NFR-PLG-***：NFR-PLG-001, NFR-PLG-002, NFR-PLG-003, NFR-PLG-004
- **NFR-PLT-***：NFR-PLT-001, NFR-PLT-002, NFR-PLT-003, NFR-PLT-004
- **NFR-SE-***：NFR-SE-001, NFR-SE-002, NFR-SE-003, NFR-SE-004, NFR-SE-005, NFR-SE-006, NFR-SE-007, NFR-SE-008, NFR-SE-009, NFR-SE-010, NFR-SE-011, NFR-SE-012
- **NFR-SEC-***：NFR-SEC-001, NFR-SEC-002, NFR-SEC-003, NFR-SEC-004, NFR-SEC-005, NFR-SEC-006, NFR-SEC-007, NFR-SEC-008, NFR-SEC-009
- **NFR-VIZ-***：NFR-VIZ-001, NFR-VIZ-002, NFR-VIZ-003, NFR-VIZ-004, NFR-VIZ-005


## 6.6 ADR 决策验证（本主题）

本主题涉及的 ADR 决定项的"实现位置 + 测试位置 + 守门位置"是否完备：

| ADR 编号 | 决定项摘要 | 实现位置 | 测试位置（本文档） | 守门位置 |
|---|---|---|---|---|
| RGS-ADR-0008 | 中间件导入判定基准 | DTL-006 §3 准入 | 本主题 TST-UT 对应模块 | CI 静态检查 |
| RGS-ADR-0020 | 插件热插拔拒绝动态链接库加载 | DTL-005 §3 沙箱 | 本主题 TST-UT 对应模块 | CI 静态检查 |
| RGS-ADR-0022 | 业务逻辑不入库 | DTL-007 §7 存储过程 | 本主题 TST-UT 对应模块 | CI 静态检查 |
| RGS-ADR-0024 | 治理闭环的重新闭合 | DTL-009 §5 治理 CI | 本主题 TST-UT 对应模块 | CI 静态检查 |
| RGS-ADR-0025 | 运维负荷预算 | DTL-009 §4 OLU 台账 | 本主题 TST-UT 对应模块 | CI 静态检查 |
| RGS-ADR-0033 | 部署区域方针 | DTL-017 §3 单区域 Multi-AZ | 本主题 TST-UT 对应模块 | CI 静态检查 |

## 7. 风险与未决事项

| ID | 内容 | 处理 |
|---|---|---|
| TBD-SEC-001 | DDoS/WAF 选型 | 由 DTL-006 决议 |
| TBD-SEC-002 | OpenBao 选型 | 由 DTL-006 决议 |
| TBD-SEC-003 | 限流阈值 | PH-2 实测 |
| TBD-PLT-001 | 追回方式 | 留 PH-6 |
| TBD-VIZ-001 | 渲染库 | 留 PH-6 |
| TBD-VIZ-002 | 聚类算法 | 留 PH-6 |
| TBD-IDN-001 | 地区取值 | 法务审查 |
| TBD-INF-002 | 日志后端 | PH-2 |

---

> 本文档为 RGS-TST 系列主题 02 单元测试设计书。

## 7.5 TBD 处置

本主题涉及的 TBD 处置方式：

| TBD 编号 | 描述 | 处置 |
|---|---|---|
| TBD-SEC-001 | DDoS/WAF 选型（OpenResty+Coraza+OWASP CRS） | 保守按既定选型实施，PH-4 实测校准 |
| TBD-SEC-002 | OpenBao 密钥管理 | 保守按既定选型实施 |
| TBD-SEC-003 | 限流阈值 | 用 NFR-SEC-008 保守值，PH-2 实测校准 |
| TBD-PLT-001 | 退款追回方式 | 留待 PH-6 |
| TBD-VIZ-001 | 画布渲染库 | 留待 PH-6 |
| TBD-VIZ-002 | 节点聚类算法 | 留待 PH-6 |
| TBD-IDN-001 | ComplianceRuleSet 地区取值 | 法务审查后定 |
| TBD-INF-002 | 日志聚合后端选型 | PH-2 决定 |

