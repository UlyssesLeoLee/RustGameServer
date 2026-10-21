# D5 Worked Example — ACP Engineering Summary

> ⚠️ 假设性示例。

---

# ADR 索引 (示例 3 条,代替完整列表)

## ADR-001 — 选择 PostgreSQL + TimescaleDB 存遥测

**日期**: 2025-XX-XX
**修订**: Ulysses (Mavis 代签) — per AGENTS.md §0
**审批**: 架构师 (Mavis 代签)

### 状态: 通过

### 上下文
需要存:机台心跳(高频 append)、故障事件(中频)、营收快照(中频 append)、配置(低频)。需要时序查询与 OLTP 两套能力。

### 选项
- A: PostgreSQL + TimescaleDB hypertable
  - 优势: 一个 DB 引擎,一份事务语义
  - 代价: 时序查询能力不及专用时序库
- B: PostgreSQL + InfluxDB (双写)
  - 优势: 时序专业
  - 代价: 双写一致性成本高,运维两套

### 决策
选 **A**, 因为: 团队规模小,容错运维简化收益大于专业时序收益;时序查询需求用 TimescaleDB 80% 满足。

### 后果
- 正面: 一份事务语义;一个备份策略
- 负面: 时序深度分析能力受限
- 撤销条件: 时序查询效率成为瓶颈时,重审本 ADR

---

## ADR-002 — 推送通道自建 (走 WebSocket + 厂商推送兜底)

### 状态: 提议

### 上下文
硬约束 2 要求"不依赖第三方推送厂商",但厂商推送覆盖率远高于 WebSocket。需要权衡。

### 选项
- A: 纯 WebSocket (iOS/Android 桌面端常驻)
  - 优势: 不被厂商锁定
  - 代价: 后台 / 杀进程后收不到
- B: WebSocket + 厂商推送兜底 (smart fallback)
  - 优势: 兼顾自建与可达
  - 代价: 仍部分依赖厂商

### 决策
倾向 **B**, 但需要明确"何时回退到 WebSocket only"。

### 后果 (待定)
- 正面
- 负面
- 风险与缓解

### 撤销条件
当厂商推送涨价 > ___ 时,迁移到 A。

---

## ADR-003 — 故障检测用阈值 + 简单统计,不做 ML

### 状态: 通过

### 上下文
故障检测需要快速上线,MVP 阶段不必上 ML。

### 选项
- A: 阈值检测 (heartbeat 失联 X 分钟 = 故障)
  - 优势: 可解释 / 误报率可调
- B: 简单统计 (Z-score, 同类机台对比)
  - 优势: 可发现温和异常
- C: ML 模型
  - 优势: 准确率高
  - 代价: 训练 / 推理成本

### 决策
**A + B 组合**, C 留 Next-scope。

### 撤销条件
误报率超 5% 时升级 B;季度复盘评估 C。

---

# 性能预算

```markdown
## CWV (老板视角: 4G 网络 + 中端安卓平板)
- LCP ≤ 3.0s (75th)
- INP ≤ 200ms (75th)
- CLS ≤ 0.1 (75th)

## 资源预算
- 主页 JS ≤ 250 KB gzip (略高,因图表组件)
- 首屏图片 ≤ 200 KB
- 字体子集化 ≤ 50 KB per family

## API 预算
- dashboard 列表 P95 ≤ 800ms
- 故障详情 P95 ≤ 400ms
- 推送端到端 P95 ≤ 60s
```

---

# 可观测性

## 三支柱

| 支柱 | 方案 |
|---|---|
| 日志 | Loki + 结构化 JSON 含 trace_id |
| 指标 | Prometheus + RED (Rate/Error/Duration) |
| 链路 | OpenTelemetry → Tempo |

## 用户旅程事件 (per D3 J-1)
- enter_dashboard
- step_load / step_view_machine
- exit_dashboard
- error_dashboard_<code>
- abandon_dashboard (heartbeat 30s)

## 系统事件
- boot / ready / shutdown
- push_send / push_ack / push_failed
- detect_fault / resolve_fault

## 告警分级

| 级 | 含义 | 响应 |
|---|---|---|
| P0 | dashboard 不可达 | 立即 |
| P1 | 推送延迟 > 60s | 4h |
| P2 | 故障误报率 > 5% | 24h |

---

# 测试金字塔

- 单元: state machine 转移 (FaultEvent / Shift) + 纯函数 (排名、对账)
- 集成: API 契约 (OpenAPI) + 推送通道稳定性
- E2E: 6 条旅程 (per D3) 在 k3s 测试环境跑

配额: 修复 bug 前必先写复现测试。

---

# 发布策略

| 服务 | 策略 |
|---|---|
| dashboard service | 滚动 (无状态) |
| 推送 worker | 蓝绿 (上下游依赖) |
| 故障检测 job | 灰度 (写入双写对比) |
| 新功能 | Feature Flag |

数据库迁移: 7 条铁律全过。

回滚脚本: `scripts/rollback.sh`, 每季度演练一次。

---

# 安全

- 凭据走环境变量,永不打印 (per AGENTS.md 守门 #5)
- mTLS 业务级 (per D4 边界)
- 锁版本 + cargo audit 定期跑
- SBOM 入库

---

# 域划分 (本仓库 RGS 范式,假设)

| 域 | 范围 |
|---|---|
| machine | 机台心跳 / 详情 |
| fault | 故障检测 / 通知 |
| shift | 班次 / 交接 |
| report | 营收 / 对账 |
| admin | 设置 / 推送策略 |

跨域走 gRPC;接口契约版本号。

---

## D5 出口判据跑分
- ✅ ADR 全部写过
- ✅ 性能预算写入 CI
- ✅ 三支柱可观测
- ✅ 告警分级 + runbook
- ⏳ a11y 真人验证待上线前补
- ✅ 测试金字塔比例 60/30/10
- ✅ 发布策略 + 回滚脚本化
- ✅ 数据库迁移 7 铁律
- ✅ 凭据不在仓

## 已知缺口
- 性能预算基线 = 上线后回填
- 值班表 = 上线前 SRE 拍板
- a11y 真人验证待补
- 回滚演练待跑
