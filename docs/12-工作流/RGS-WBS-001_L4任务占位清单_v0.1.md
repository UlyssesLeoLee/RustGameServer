# RGS-WBS-001 L4 任务占位清单（v0.1 占位，由 `scripts/build_wbs_v02.py` 生成）

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-WBS-001-ADD1 |
| 版本 | 0.1（占位框架）|
| 依据 | RGS-WBS-001 v0.3 §4.3 + RGS-PLAN-001 v0.8 §3.1 PH 表 |
| 配套 | RGS-WBS-001 v0.3 主文件 / RGS-TS-001 v0.6 §6.2 OLU 双轨制 / RGS-ENV-CALIB-001 |
| 保密级别 | 内部限定（Internal Use Only）|

> **本表由 `scripts/build_wbs_v02.py` 生成**。共 2048 行 L4 任务占位。
>
> **5 域 + 3 配套 Lead 在 PH-0.5 前补全每行**：人·天 / tokens / 前置 / 验收 / 回滚 5 字段。
> 签字栏位留空，由 owner 在 PH-0.5 签字时填写。
>
> **维护方式**：
> 1. 编辑本表 CSV / markdown 表格（按列填写 _占位）
> 2. PH-0.5 前 5 域 Lead 完成 256 L4/域 × 5 = 1,280 + 3 配套 256/域 × 3 = 768 → 共 2,048 L4 补全
> 3. PH-0.5 签字：5 域 Lead + SRE + 架构 + PM 按域签字
> 4. PH-1 末：每域 Lead 出 L5 工作包完整清单（per RGS-WBS-001 v0.3 §5）
>
> **生成脚本**：`scripts/build_wbs_v02.py`（可重跑保持结构一致）
> **关联主文件**：[RGS-WBS-001 v0.3 主文件](../12-工作流/RGS-WBS-001_瀑布式工作分解结构_v0.3.md)

| # | PH | 窗口 | 域 | L3 任务簇 | L4 任务 | Owner | 人·天 | Tokens | 前置 | 验收 | 回滚 | 签字 |
|---:|---|---|---|---|---|---|---|---:|---:|---|---|---|
| 1 | PH-0 | 第 1-2 周 | foundation | workspace 骨架 | virtual workspace 配置 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 2 | PH-0 | 第 1-2 周 | foundation | workspace 骨架 | resolver=3 锁定 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 3 | PH-0 | 第 1-2 周 | foundation | workspace 骨架 | Edition 2024 升级 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 4 | PH-0 | 第 1-2 周 | foundation | workspace 骨架 | crate 间依赖方向 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 5 | PH-0 | 第 1-2 周 | foundation | testkit 共用 | testcontainers PG 封装 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 6 | PH-0 | 第 1-2 周 | foundation | testkit 共用 | mock helpers | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 7 | PH-0 | 第 1-2 周 | foundation | testkit 共用 | fixture builders | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 8 | PH-0 | 第 1-2 周 | foundation | testkit 共用 | coverage 报告 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 9 | PH-0 | 第 1-2 周 | foundation | CI 工具链 | GitHub Actions 配置 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 10 | PH-0 | 第 1-2 周 | foundation | CI 工具链 | cargo fmt/clippy/deny | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 11 | PH-0 | 第 1-2 周 | foundation | CI 工具链 | sqlx prepare check | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 12 | PH-0 | 第 1-2 周 | foundation | CI 工具链 | manifest 校验 CI | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 13 | PH-0 | 第 1-2 周 | foundation | DAG validator | 拓扑排序算法 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 14 | PH-0 | 第 1-2 周 | foundation | DAG validator | 环依赖检测 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 15 | PH-0 | 第 1-2 周 | foundation | DAG validator | 缺祖先检测 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 16 | PH-0 | 第 1-2 周 | foundation | DAG validator | 负例测试套件 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 17 | PH-0 | 第 1-2 周 | foundation | cargo-deny/audit | 许可证白名单 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 18 | PH-0 | 第 1-2 周 | foundation | cargo-deny/audit | 漏洞数据库 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 19 | PH-0 | 第 1-2 周 | foundation | cargo-deny/audit | 依赖来源限制 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 20 | PH-0 | 第 1-2 周 | foundation | cargo-deny/audit | CI 集成 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 21 | PH-0 | 第 1-2 周 | foundation | manifest schema | JSON Schema 起草 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 22 | PH-0 | 第 1-2 周 | foundation | manifest schema | ARC-042 字段映射 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 23 | PH-0 | 第 1-2 周 | foundation | manifest schema | schema 校验 CLI | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 24 | PH-0 | 第 1-2 周 | foundation | manifest schema | 示例 manifest | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 25 | PH-0 | 第 1-2 周 | foundation | 文档生成 | mdbook 配置 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 26 | PH-0 | 第 1-2 周 | foundation | 文档生成 | Doxygen/Rustdoc | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 27 | PH-0 | 第 1-2 周 | foundation | 文档生成 | CR 链接检查 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 28 | PH-0 | 第 1-2 周 | foundation | 文档生成 | CI 文档门禁 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 29 | PH-0 | 第 1-2 周 | foundation | 工程约定 | 错误码定义 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 30 | PH-0 | 第 1-2 周 | foundation | 工程约定 | 序列化约定 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 31 | PH-0 | 第 1-2 周 | foundation | 工程约定 | 日志规范 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 32 | PH-0 | 第 1-2 周 | foundation | 工程约定 | metrics 命名 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 33 | PH-0 | 第 1-2 周 | player | API Spec | 列出 gRPC 方法 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 34 | PH-0 | 第 1-2 周 | player | API Spec | 定义 Proto 文件 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 35 | PH-0 | 第 1-2 周 | player | API Spec | 配置 tonic-build | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 36 | PH-0 | 第 1-2 周 | player | API Spec | 编译期校验 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 37 | PH-0 | 第 1-2 周 | player | 业务逻辑 | 实体表定义 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 38 | PH-0 | 第 1-2 周 | player | 业务逻辑 | 状态机实现 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 39 | PH-0 | 第 1-2 周 | player | 业务逻辑 | 错误码 + 边界条件 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 40 | PH-0 | 第 1-2 周 | player | 业务逻辑 | 核心算法 / 决策 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 41 | PH-0 | 第 1-2 周 | player | DB migration | Schema 迁移 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 42 | PH-0 | 第 1-2 周 | player | DB migration | 索引 + 约束 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 43 | PH-0 | 第 1-2 周 | player | DB migration | 双向迁移演练 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 44 | PH-0 | 第 1-2 周 | player | DB migration | 回滚预案 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 45 | PH-0 | 第 1-2 周 | player | UT 单元测试 | testkit helper | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 46 | PH-0 | 第 1-2 周 | player | UT 单元测试 | CRUD 覆盖 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 47 | PH-0 | 第 1-2 周 | player | UT 单元测试 | 状态机覆盖 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 48 | PH-0 | 第 1-2 周 | player | UT 单元测试 | 覆盖率报告 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 49 | PH-0 | 第 1-2 周 | player | IT 集成测试 | service 启动 + health | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 50 | PH-0 | 第 1-2 周 | player | IT 集成测试 | DB 集成 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 51 | PH-0 | 第 1-2 周 | player | IT 集成测试 | 跨组件契约 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 52 | PH-0 | 第 1-2 周 | player | IT 集成测试 | 端到端集成 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 53 | PH-0 | 第 1-2 周 | player | ST 系统测试 | K8s 部署验证 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 54 | PH-0 | 第 1-2 周 | player | ST 系统测试 | 性能 / 容量 NFR | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 55 | PH-0 | 第 1-2 周 | player | ST 系统测试 | chaos 故障注入 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 56 | PH-0 | 第 1-2 周 | player | ST 系统测试 | RPO/RTO 验证 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 57 | PH-0 | 第 1-2 周 | player | Helm chart | Chart.yaml | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 58 | PH-0 | 第 1-2 周 | player | Helm chart | values.yaml | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 59 | PH-0 | 第 1-2 周 | player | Helm chart | deployment + HPA | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 60 | PH-0 | 第 1-2 周 | player | Helm chart | NetworkPolicy | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 61 | PH-0 | 第 1-2 周 | player | observability | OTel spans | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 62 | PH-0 | 第 1-2 周 | player | observability | Prometheus metrics | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 63 | PH-0 | 第 1-2 周 | player | observability | Grafana 仪表盘 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 64 | PH-0 | 第 1-2 周 | player | observability | Loki 日志 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 65 | PH-0 | 第 1-2 周 | economy | API Spec | 列出 gRPC 方法 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 66 | PH-0 | 第 1-2 周 | economy | API Spec | 定义 Proto 文件 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 67 | PH-0 | 第 1-2 周 | economy | API Spec | 配置 tonic-build | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 68 | PH-0 | 第 1-2 周 | economy | API Spec | 编译期校验 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 69 | PH-0 | 第 1-2 周 | economy | 业务逻辑 | 实体表定义 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 70 | PH-0 | 第 1-2 周 | economy | 业务逻辑 | 状态机实现 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 71 | PH-0 | 第 1-2 周 | economy | 业务逻辑 | 错误码 + 边界条件 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 72 | PH-0 | 第 1-2 周 | economy | 业务逻辑 | 核心算法 / 决策 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 73 | PH-0 | 第 1-2 周 | economy | DB migration | Schema 迁移 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 74 | PH-0 | 第 1-2 周 | economy | DB migration | 索引 + 约束 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 75 | PH-0 | 第 1-2 周 | economy | DB migration | 双向迁移演练 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 76 | PH-0 | 第 1-2 周 | economy | DB migration | 回滚预案 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 77 | PH-0 | 第 1-2 周 | economy | UT 单元测试 | testkit helper | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 78 | PH-0 | 第 1-2 周 | economy | UT 单元测试 | CRUD 覆盖 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 79 | PH-0 | 第 1-2 周 | economy | UT 单元测试 | 状态机覆盖 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 80 | PH-0 | 第 1-2 周 | economy | UT 单元测试 | 覆盖率报告 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 81 | PH-0 | 第 1-2 周 | economy | IT 集成测试 | service 启动 + health | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 82 | PH-0 | 第 1-2 周 | economy | IT 集成测试 | DB 集成 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 83 | PH-0 | 第 1-2 周 | economy | IT 集成测试 | 跨组件契约 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 84 | PH-0 | 第 1-2 周 | economy | IT 集成测试 | 端到端集成 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 85 | PH-0 | 第 1-2 周 | economy | ST 系统测试 | K8s 部署验证 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 86 | PH-0 | 第 1-2 周 | economy | ST 系统测试 | 性能 / 容量 NFR | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 87 | PH-0 | 第 1-2 周 | economy | ST 系统测试 | chaos 故障注入 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 88 | PH-0 | 第 1-2 周 | economy | ST 系统测试 | RPO/RTO 验证 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 89 | PH-0 | 第 1-2 周 | economy | Helm chart | Chart.yaml | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 90 | PH-0 | 第 1-2 周 | economy | Helm chart | values.yaml | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 91 | PH-0 | 第 1-2 周 | economy | Helm chart | deployment + HPA | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 92 | PH-0 | 第 1-2 周 | economy | Helm chart | NetworkPolicy | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 93 | PH-0 | 第 1-2 周 | economy | observability | OTel spans | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 94 | PH-0 | 第 1-2 周 | economy | observability | Prometheus metrics | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 95 | PH-0 | 第 1-2 周 | economy | observability | Grafana 仪表盘 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 96 | PH-0 | 第 1-2 周 | economy | observability | Loki 日志 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 97 | PH-0 | 第 1-2 周 | match | API Spec | 列出 gRPC 方法 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 98 | PH-0 | 第 1-2 周 | match | API Spec | 定义 Proto 文件 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 99 | PH-0 | 第 1-2 周 | match | API Spec | 配置 tonic-build | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 100 | PH-0 | 第 1-2 周 | match | API Spec | 编译期校验 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 101 | PH-0 | 第 1-2 周 | match | 业务逻辑 | 实体表定义 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 102 | PH-0 | 第 1-2 周 | match | 业务逻辑 | 状态机实现 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 103 | PH-0 | 第 1-2 周 | match | 业务逻辑 | 错误码 + 边界条件 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 104 | PH-0 | 第 1-2 周 | match | 业务逻辑 | 核心算法 / 决策 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 105 | PH-0 | 第 1-2 周 | match | DB migration | Schema 迁移 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 106 | PH-0 | 第 1-2 周 | match | DB migration | 索引 + 约束 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 107 | PH-0 | 第 1-2 周 | match | DB migration | 双向迁移演练 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 108 | PH-0 | 第 1-2 周 | match | DB migration | 回滚预案 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 109 | PH-0 | 第 1-2 周 | match | UT 单元测试 | testkit helper | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 110 | PH-0 | 第 1-2 周 | match | UT 单元测试 | CRUD 覆盖 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 111 | PH-0 | 第 1-2 周 | match | UT 单元测试 | 状态机覆盖 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 112 | PH-0 | 第 1-2 周 | match | UT 单元测试 | 覆盖率报告 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 113 | PH-0 | 第 1-2 周 | match | IT 集成测试 | service 启动 + health | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 114 | PH-0 | 第 1-2 周 | match | IT 集成测试 | DB 集成 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 115 | PH-0 | 第 1-2 周 | match | IT 集成测试 | 跨组件契约 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 116 | PH-0 | 第 1-2 周 | match | IT 集成测试 | 端到端集成 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 117 | PH-0 | 第 1-2 周 | match | ST 系统测试 | K8s 部署验证 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 118 | PH-0 | 第 1-2 周 | match | ST 系统测试 | 性能 / 容量 NFR | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 119 | PH-0 | 第 1-2 周 | match | ST 系统测试 | chaos 故障注入 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 120 | PH-0 | 第 1-2 周 | match | ST 系统测试 | RPO/RTO 验证 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 121 | PH-0 | 第 1-2 周 | match | Helm chart | Chart.yaml | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 122 | PH-0 | 第 1-2 周 | match | Helm chart | values.yaml | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 123 | PH-0 | 第 1-2 周 | match | Helm chart | deployment + HPA | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 124 | PH-0 | 第 1-2 周 | match | Helm chart | NetworkPolicy | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 125 | PH-0 | 第 1-2 周 | match | observability | OTel spans | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 126 | PH-0 | 第 1-2 周 | match | observability | Prometheus metrics | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 127 | PH-0 | 第 1-2 周 | match | observability | Grafana 仪表盘 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 128 | PH-0 | 第 1-2 周 | match | observability | Loki 日志 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 129 | PH-0 | 第 1-2 周 | social | API Spec | 列出 gRPC 方法 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 130 | PH-0 | 第 1-2 周 | social | API Spec | 定义 Proto 文件 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 131 | PH-0 | 第 1-2 周 | social | API Spec | 配置 tonic-build | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 132 | PH-0 | 第 1-2 周 | social | API Spec | 编译期校验 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 133 | PH-0 | 第 1-2 周 | social | 业务逻辑 | 实体表定义 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 134 | PH-0 | 第 1-2 周 | social | 业务逻辑 | 状态机实现 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 135 | PH-0 | 第 1-2 周 | social | 业务逻辑 | 错误码 + 边界条件 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 136 | PH-0 | 第 1-2 周 | social | 业务逻辑 | 核心算法 / 决策 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 137 | PH-0 | 第 1-2 周 | social | DB migration | Schema 迁移 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 138 | PH-0 | 第 1-2 周 | social | DB migration | 索引 + 约束 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 139 | PH-0 | 第 1-2 周 | social | DB migration | 双向迁移演练 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 140 | PH-0 | 第 1-2 周 | social | DB migration | 回滚预案 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 141 | PH-0 | 第 1-2 周 | social | UT 单元测试 | testkit helper | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 142 | PH-0 | 第 1-2 周 | social | UT 单元测试 | CRUD 覆盖 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 143 | PH-0 | 第 1-2 周 | social | UT 单元测试 | 状态机覆盖 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 144 | PH-0 | 第 1-2 周 | social | UT 单元测试 | 覆盖率报告 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 145 | PH-0 | 第 1-2 周 | social | IT 集成测试 | service 启动 + health | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 146 | PH-0 | 第 1-2 周 | social | IT 集成测试 | DB 集成 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 147 | PH-0 | 第 1-2 周 | social | IT 集成测试 | 跨组件契约 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 148 | PH-0 | 第 1-2 周 | social | IT 集成测试 | 端到端集成 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 149 | PH-0 | 第 1-2 周 | social | ST 系统测试 | K8s 部署验证 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 150 | PH-0 | 第 1-2 周 | social | ST 系统测试 | 性能 / 容量 NFR | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 151 | PH-0 | 第 1-2 周 | social | ST 系统测试 | chaos 故障注入 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 152 | PH-0 | 第 1-2 周 | social | ST 系统测试 | RPO/RTO 验证 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 153 | PH-0 | 第 1-2 周 | social | Helm chart | Chart.yaml | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 154 | PH-0 | 第 1-2 周 | social | Helm chart | values.yaml | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 155 | PH-0 | 第 1-2 周 | social | Helm chart | deployment + HPA | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 156 | PH-0 | 第 1-2 周 | social | Helm chart | NetworkPolicy | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 157 | PH-0 | 第 1-2 周 | social | observability | OTel spans | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 158 | PH-0 | 第 1-2 周 | social | observability | Prometheus metrics | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 159 | PH-0 | 第 1-2 周 | social | observability | Grafana 仪表盘 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 160 | PH-0 | 第 1-2 周 | social | observability | Loki 日志 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 161 | PH-0 | 第 1-2 周 | admin | API Spec | 列出 gRPC 方法 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 162 | PH-0 | 第 1-2 周 | admin | API Spec | 定义 Proto 文件 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 163 | PH-0 | 第 1-2 周 | admin | API Spec | 配置 tonic-build | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 164 | PH-0 | 第 1-2 周 | admin | API Spec | 编译期校验 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 165 | PH-0 | 第 1-2 周 | admin | 业务逻辑 | 实体表定义 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 166 | PH-0 | 第 1-2 周 | admin | 业务逻辑 | 状态机实现 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 167 | PH-0 | 第 1-2 周 | admin | 业务逻辑 | 错误码 + 边界条件 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 168 | PH-0 | 第 1-2 周 | admin | 业务逻辑 | 核心算法 / 决策 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 169 | PH-0 | 第 1-2 周 | admin | DB migration | Schema 迁移 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 170 | PH-0 | 第 1-2 周 | admin | DB migration | 索引 + 约束 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 171 | PH-0 | 第 1-2 周 | admin | DB migration | 双向迁移演练 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 172 | PH-0 | 第 1-2 周 | admin | DB migration | 回滚预案 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 173 | PH-0 | 第 1-2 周 | admin | UT 单元测试 | testkit helper | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 174 | PH-0 | 第 1-2 周 | admin | UT 单元测试 | CRUD 覆盖 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 175 | PH-0 | 第 1-2 周 | admin | UT 单元测试 | 状态机覆盖 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 176 | PH-0 | 第 1-2 周 | admin | UT 单元测试 | 覆盖率报告 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 177 | PH-0 | 第 1-2 周 | admin | IT 集成测试 | service 启动 + health | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 178 | PH-0 | 第 1-2 周 | admin | IT 集成测试 | DB 集成 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 179 | PH-0 | 第 1-2 周 | admin | IT 集成测试 | 跨组件契约 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 180 | PH-0 | 第 1-2 周 | admin | IT 集成测试 | 端到端集成 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 181 | PH-0 | 第 1-2 周 | admin | ST 系统测试 | K8s 部署验证 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 182 | PH-0 | 第 1-2 周 | admin | ST 系统测试 | 性能 / 容量 NFR | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 183 | PH-0 | 第 1-2 周 | admin | ST 系统测试 | chaos 故障注入 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 184 | PH-0 | 第 1-2 周 | admin | ST 系统测试 | RPO/RTO 验证 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 185 | PH-0 | 第 1-2 周 | admin | Helm chart | Chart.yaml | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 186 | PH-0 | 第 1-2 周 | admin | Helm chart | values.yaml | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 187 | PH-0 | 第 1-2 周 | admin | Helm chart | deployment + HPA | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 188 | PH-0 | 第 1-2 周 | admin | Helm chart | NetworkPolicy | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 189 | PH-0 | 第 1-2 周 | admin | observability | OTel spans | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 190 | PH-0 | 第 1-2 周 | admin | observability | Prometheus metrics | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 191 | PH-0 | 第 1-2 周 | admin | observability | Grafana 仪表盘 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 192 | PH-0 | 第 1-2 周 | admin | observability | Loki 日志 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 193 | PH-0 | 第 1-2 周 | cluster-ops | Control Plane API | ClusterOps gRPC 定义 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 194 | PH-0 | 第 1-2 周 | cluster-ops | Control Plane API | AdminService 转发 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 195 | PH-0 | 第 1-2 周 | cluster-ops | Control Plane API | request_id 幂等 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 196 | PH-0 | 第 1-2 周 | cluster-ops | Control Plane API | OCC 版本字段 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 197 | PH-0 | 第 1-2 周 | cluster-ops | CEM | Feature registry | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 198 | PH-0 | 第 1-2 周 | cluster-ops | CEM | 事件流 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 199 | PH-0 | 第 1-2 周 | cluster-ops | CEM | 订阅/取消订阅 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 200 | PH-0 | 第 1-2 周 | cluster-ops | CEM | DLQ 处理 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 201 | PH-0 | 第 1-2 周 | cluster-ops | PFAU | declared → canary → confirm → done 状态机 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 202 | PH-0 | 第 1-2 周 | cluster-ops | PFAU | all-reachable 确认 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 203 | PH-0 | 第 1-2 周 | cluster-ops | PFAU | 灰度策略 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 204 | PH-0 | 第 1-2 周 | cluster-ops | PFAU | 回滚路径 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 205 | PH-0 | 第 1-2 周 | cluster-ops | 状态机 | feature 状态定义 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 206 | PH-0 | 第 1-2 周 | cluster-ops | 状态机 | 非法转移检测 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 207 | PH-0 | 第 1-2 周 | cluster-ops | 状态机 | 状态转移图 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 208 | PH-0 | 第 1-2 周 | cluster-ops | 状态机 | 持久化方案 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 209 | PH-0 | 第 1-2 周 | cluster-ops | RBAC | GM / COC / 客户端 3 套权限 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 210 | PH-0 | 第 1-2 周 | cluster-ops | RBAC | 权限矩阵 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 211 | PH-0 | 第 1-2 周 | cluster-ops | RBAC | 审计日志 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 212 | PH-0 | 第 1-2 周 | cluster-ops | RBAC | 撤销机制 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 213 | PH-0 | 第 1-2 周 | cluster-ops | fencing | 租约机制 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 214 | PH-0 | 第 1-2 周 | cluster-ops | fencing | CAS 版本控制 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 215 | PH-0 | 第 1-2 周 | cluster-ops | fencing | stale leader 检测 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 216 | PH-0 | 第 1-2 周 | cluster-ops | fencing | 集群隔离策略 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 217 | PH-0 | 第 1-2 周 | cluster-ops | 审计 | 审计 schema | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 218 | PH-0 | 第 1-2 周 | cluster-ops | 审计 | 写入路径 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 219 | PH-0 | 第 1-2 周 | cluster-ops | 审计 | 查询接口 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 220 | PH-0 | 第 1-2 周 | cluster-ops | 审计 | 保留期 + 归档 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 221 | PH-0 | 第 1-2 周 | cluster-ops | OCC | 乐观并发控制 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 222 | PH-0 | 第 1-2 周 | cluster-ops | OCC | 重试策略 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 223 | PH-0 | 第 1-2 周 | cluster-ops | OCC | 冲突检测 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 224 | PH-0 | 第 1-2 周 | cluster-ops | OCC | 死锁恢复 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 225 | PH-0 | 第 1-2 周 | shared-platform | Rust 工具链 | rustup 1.98 锁定 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 226 | PH-0 | 第 1-2 周 | shared-platform | Rust 工具链 | rust-toolchain.toml | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 227 | PH-0 | 第 1-2 周 | shared-platform | Rust 工具链 | CI cache | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 228 | PH-0 | 第 1-2 周 | shared-platform | Rust 工具链 | 升级评审 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 229 | PH-0 | 第 1-2 周 | shared-platform | Cargo.lock 锁定 | --locked 构建 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 230 | PH-0 | 第 1-2 周 | shared-platform | Cargo.lock 锁定 | workspace 统一锁 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 231 | PH-0 | 第 1-2 周 | shared-platform | Cargo.lock 锁定 | 依赖审计 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 232 | PH-0 | 第 1-2 周 | shared-platform | Cargo.lock 锁定 | 更新策略 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 233 | PH-0 | 第 1-2 周 | shared-platform | 镜像构建 | Dockerfile.distroless | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 234 | PH-0 | 第 1-2 周 | shared-platform | 镜像构建 | 镜像大小优化 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 235 | PH-0 | 第 1-2 周 | shared-platform | 镜像构建 | SBOM 生成 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 236 | PH-0 | 第 1-2 周 | shared-platform | 镜像构建 | 漏洞扫描 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 237 | PH-0 | 第 1-2 周 | shared-platform | K3s | K3s 集群初始化 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 238 | PH-0 | 第 1-2 周 | shared-platform | K3s | kubectl 配置 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 239 | PH-0 | 第 1-2 周 | shared-platform | K3s | NetworkPolicy | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 240 | PH-0 | 第 1-2 周 | shared-platform | K3s | HPA / VPA | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 241 | PH-0 | 第 1-2 周 | shared-platform | OTel Collector | 采集器配置 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 242 | PH-0 | 第 1-2 周 | shared-platform | OTel Collector | OTLP 接收 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 243 | PH-0 | 第 1-2 周 | shared-platform | OTel Collector | 采样策略 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 244 | PH-0 | 第 1-2 周 | shared-platform | OTel Collector | 导出器配置 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 245 | PH-0 | 第 1-2 周 | shared-platform | Helm | Chart 模板 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 246 | PH-0 | 第 1-2 周 | shared-platform | Helm | values 校验 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 247 | PH-0 | 第 1-2 周 | shared-platform | Helm | 依赖管理 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 248 | PH-0 | 第 1-2 周 | shared-platform | Helm | CI render | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 249 | PH-0 | 第 1-2 周 | shared-platform | 密钥 | Vault/OpenBao 部署 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 250 | PH-0 | 第 1-2 周 | shared-platform | 密钥 | 密钥轮换策略 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 251 | PH-0 | 第 1-2 周 | shared-platform | 密钥 | 运行时注入 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 252 | PH-0 | 第 1-2 周 | shared-platform | 密钥 | 审计 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 253 | PH-0 | 第 1-2 周 | shared-platform | 灾备 | 备份策略 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 254 | PH-0 | 第 1-2 周 | shared-platform | 灾备 | 恢复演练 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 255 | PH-0 | 第 1-2 周 | shared-platform | 灾备 | 跨 AZ 复制 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 256 | PH-0 | 第 1-2 周 | shared-platform | 灾备 | RTO/RPO 验证 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 257 | PH-1 | 第 3-4 周 | foundation | workspace 骨架 | virtual workspace 配置 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 258 | PH-1 | 第 3-4 周 | foundation | workspace 骨架 | resolver=3 锁定 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 259 | PH-1 | 第 3-4 周 | foundation | workspace 骨架 | Edition 2024 升级 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 260 | PH-1 | 第 3-4 周 | foundation | workspace 骨架 | crate 间依赖方向 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 261 | PH-1 | 第 3-4 周 | foundation | testkit 共用 | testcontainers PG 封装 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 262 | PH-1 | 第 3-4 周 | foundation | testkit 共用 | mock helpers | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 263 | PH-1 | 第 3-4 周 | foundation | testkit 共用 | fixture builders | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 264 | PH-1 | 第 3-4 周 | foundation | testkit 共用 | coverage 报告 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 265 | PH-1 | 第 3-4 周 | foundation | CI 工具链 | GitHub Actions 配置 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 266 | PH-1 | 第 3-4 周 | foundation | CI 工具链 | cargo fmt/clippy/deny | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 267 | PH-1 | 第 3-4 周 | foundation | CI 工具链 | sqlx prepare check | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 268 | PH-1 | 第 3-4 周 | foundation | CI 工具链 | manifest 校验 CI | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 269 | PH-1 | 第 3-4 周 | foundation | DAG validator | 拓扑排序算法 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 270 | PH-1 | 第 3-4 周 | foundation | DAG validator | 环依赖检测 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 271 | PH-1 | 第 3-4 周 | foundation | DAG validator | 缺祖先检测 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 272 | PH-1 | 第 3-4 周 | foundation | DAG validator | 负例测试套件 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 273 | PH-1 | 第 3-4 周 | foundation | cargo-deny/audit | 许可证白名单 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 274 | PH-1 | 第 3-4 周 | foundation | cargo-deny/audit | 漏洞数据库 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 275 | PH-1 | 第 3-4 周 | foundation | cargo-deny/audit | 依赖来源限制 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 276 | PH-1 | 第 3-4 周 | foundation | cargo-deny/audit | CI 集成 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 277 | PH-1 | 第 3-4 周 | foundation | manifest schema | JSON Schema 起草 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 278 | PH-1 | 第 3-4 周 | foundation | manifest schema | ARC-042 字段映射 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 279 | PH-1 | 第 3-4 周 | foundation | manifest schema | schema 校验 CLI | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 280 | PH-1 | 第 3-4 周 | foundation | manifest schema | 示例 manifest | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 281 | PH-1 | 第 3-4 周 | foundation | 文档生成 | mdbook 配置 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 282 | PH-1 | 第 3-4 周 | foundation | 文档生成 | Doxygen/Rustdoc | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 283 | PH-1 | 第 3-4 周 | foundation | 文档生成 | CR 链接检查 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 284 | PH-1 | 第 3-4 周 | foundation | 文档生成 | CI 文档门禁 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 285 | PH-1 | 第 3-4 周 | foundation | 工程约定 | 错误码定义 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 286 | PH-1 | 第 3-4 周 | foundation | 工程约定 | 序列化约定 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 287 | PH-1 | 第 3-4 周 | foundation | 工程约定 | 日志规范 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 288 | PH-1 | 第 3-4 周 | foundation | 工程约定 | metrics 命名 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 289 | PH-1 | 第 3-4 周 | player | API Spec | 列出 gRPC 方法 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 290 | PH-1 | 第 3-4 周 | player | API Spec | 定义 Proto 文件 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 291 | PH-1 | 第 3-4 周 | player | API Spec | 配置 tonic-build | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 292 | PH-1 | 第 3-4 周 | player | API Spec | 编译期校验 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 293 | PH-1 | 第 3-4 周 | player | 业务逻辑 | 实体表定义 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 294 | PH-1 | 第 3-4 周 | player | 业务逻辑 | 状态机实现 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 295 | PH-1 | 第 3-4 周 | player | 业务逻辑 | 错误码 + 边界条件 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 296 | PH-1 | 第 3-4 周 | player | 业务逻辑 | 核心算法 / 决策 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 297 | PH-1 | 第 3-4 周 | player | DB migration | Schema 迁移 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 298 | PH-1 | 第 3-4 周 | player | DB migration | 索引 + 约束 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 299 | PH-1 | 第 3-4 周 | player | DB migration | 双向迁移演练 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 300 | PH-1 | 第 3-4 周 | player | DB migration | 回滚预案 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 301 | PH-1 | 第 3-4 周 | player | UT 单元测试 | testkit helper | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 302 | PH-1 | 第 3-4 周 | player | UT 单元测试 | CRUD 覆盖 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 303 | PH-1 | 第 3-4 周 | player | UT 单元测试 | 状态机覆盖 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 304 | PH-1 | 第 3-4 周 | player | UT 单元测试 | 覆盖率报告 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 305 | PH-1 | 第 3-4 周 | player | IT 集成测试 | service 启动 + health | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 306 | PH-1 | 第 3-4 周 | player | IT 集成测试 | DB 集成 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 307 | PH-1 | 第 3-4 周 | player | IT 集成测试 | 跨组件契约 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 308 | PH-1 | 第 3-4 周 | player | IT 集成测试 | 端到端集成 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 309 | PH-1 | 第 3-4 周 | player | ST 系统测试 | K8s 部署验证 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 310 | PH-1 | 第 3-4 周 | player | ST 系统测试 | 性能 / 容量 NFR | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 311 | PH-1 | 第 3-4 周 | player | ST 系统测试 | chaos 故障注入 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 312 | PH-1 | 第 3-4 周 | player | ST 系统测试 | RPO/RTO 验证 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 313 | PH-1 | 第 3-4 周 | player | Helm chart | Chart.yaml | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 314 | PH-1 | 第 3-4 周 | player | Helm chart | values.yaml | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 315 | PH-1 | 第 3-4 周 | player | Helm chart | deployment + HPA | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 316 | PH-1 | 第 3-4 周 | player | Helm chart | NetworkPolicy | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 317 | PH-1 | 第 3-4 周 | player | observability | OTel spans | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 318 | PH-1 | 第 3-4 周 | player | observability | Prometheus metrics | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 319 | PH-1 | 第 3-4 周 | player | observability | Grafana 仪表盘 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 320 | PH-1 | 第 3-4 周 | player | observability | Loki 日志 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 321 | PH-1 | 第 3-4 周 | economy | API Spec | 列出 gRPC 方法 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 322 | PH-1 | 第 3-4 周 | economy | API Spec | 定义 Proto 文件 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 323 | PH-1 | 第 3-4 周 | economy | API Spec | 配置 tonic-build | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 324 | PH-1 | 第 3-4 周 | economy | API Spec | 编译期校验 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 325 | PH-1 | 第 3-4 周 | economy | 业务逻辑 | 实体表定义 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 326 | PH-1 | 第 3-4 周 | economy | 业务逻辑 | 状态机实现 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 327 | PH-1 | 第 3-4 周 | economy | 业务逻辑 | 错误码 + 边界条件 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 328 | PH-1 | 第 3-4 周 | economy | 业务逻辑 | 核心算法 / 决策 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 329 | PH-1 | 第 3-4 周 | economy | DB migration | Schema 迁移 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 330 | PH-1 | 第 3-4 周 | economy | DB migration | 索引 + 约束 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 331 | PH-1 | 第 3-4 周 | economy | DB migration | 双向迁移演练 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 332 | PH-1 | 第 3-4 周 | economy | DB migration | 回滚预案 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 333 | PH-1 | 第 3-4 周 | economy | UT 单元测试 | testkit helper | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 334 | PH-1 | 第 3-4 周 | economy | UT 单元测试 | CRUD 覆盖 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 335 | PH-1 | 第 3-4 周 | economy | UT 单元测试 | 状态机覆盖 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 336 | PH-1 | 第 3-4 周 | economy | UT 单元测试 | 覆盖率报告 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 337 | PH-1 | 第 3-4 周 | economy | IT 集成测试 | service 启动 + health | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 338 | PH-1 | 第 3-4 周 | economy | IT 集成测试 | DB 集成 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 339 | PH-1 | 第 3-4 周 | economy | IT 集成测试 | 跨组件契约 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 340 | PH-1 | 第 3-4 周 | economy | IT 集成测试 | 端到端集成 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 341 | PH-1 | 第 3-4 周 | economy | ST 系统测试 | K8s 部署验证 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 342 | PH-1 | 第 3-4 周 | economy | ST 系统测试 | 性能 / 容量 NFR | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 343 | PH-1 | 第 3-4 周 | economy | ST 系统测试 | chaos 故障注入 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 344 | PH-1 | 第 3-4 周 | economy | ST 系统测试 | RPO/RTO 验证 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 345 | PH-1 | 第 3-4 周 | economy | Helm chart | Chart.yaml | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 346 | PH-1 | 第 3-4 周 | economy | Helm chart | values.yaml | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 347 | PH-1 | 第 3-4 周 | economy | Helm chart | deployment + HPA | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 348 | PH-1 | 第 3-4 周 | economy | Helm chart | NetworkPolicy | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 349 | PH-1 | 第 3-4 周 | economy | observability | OTel spans | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 350 | PH-1 | 第 3-4 周 | economy | observability | Prometheus metrics | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 351 | PH-1 | 第 3-4 周 | economy | observability | Grafana 仪表盘 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 352 | PH-1 | 第 3-4 周 | economy | observability | Loki 日志 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 353 | PH-1 | 第 3-4 周 | match | API Spec | 列出 gRPC 方法 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 354 | PH-1 | 第 3-4 周 | match | API Spec | 定义 Proto 文件 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 355 | PH-1 | 第 3-4 周 | match | API Spec | 配置 tonic-build | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 356 | PH-1 | 第 3-4 周 | match | API Spec | 编译期校验 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 357 | PH-1 | 第 3-4 周 | match | 业务逻辑 | 实体表定义 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 358 | PH-1 | 第 3-4 周 | match | 业务逻辑 | 状态机实现 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 359 | PH-1 | 第 3-4 周 | match | 业务逻辑 | 错误码 + 边界条件 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 360 | PH-1 | 第 3-4 周 | match | 业务逻辑 | 核心算法 / 决策 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 361 | PH-1 | 第 3-4 周 | match | DB migration | Schema 迁移 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 362 | PH-1 | 第 3-4 周 | match | DB migration | 索引 + 约束 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 363 | PH-1 | 第 3-4 周 | match | DB migration | 双向迁移演练 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 364 | PH-1 | 第 3-4 周 | match | DB migration | 回滚预案 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 365 | PH-1 | 第 3-4 周 | match | UT 单元测试 | testkit helper | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 366 | PH-1 | 第 3-4 周 | match | UT 单元测试 | CRUD 覆盖 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 367 | PH-1 | 第 3-4 周 | match | UT 单元测试 | 状态机覆盖 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 368 | PH-1 | 第 3-4 周 | match | UT 单元测试 | 覆盖率报告 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 369 | PH-1 | 第 3-4 周 | match | IT 集成测试 | service 启动 + health | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 370 | PH-1 | 第 3-4 周 | match | IT 集成测试 | DB 集成 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 371 | PH-1 | 第 3-4 周 | match | IT 集成测试 | 跨组件契约 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 372 | PH-1 | 第 3-4 周 | match | IT 集成测试 | 端到端集成 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 373 | PH-1 | 第 3-4 周 | match | ST 系统测试 | K8s 部署验证 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 374 | PH-1 | 第 3-4 周 | match | ST 系统测试 | 性能 / 容量 NFR | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 375 | PH-1 | 第 3-4 周 | match | ST 系统测试 | chaos 故障注入 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 376 | PH-1 | 第 3-4 周 | match | ST 系统测试 | RPO/RTO 验证 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 377 | PH-1 | 第 3-4 周 | match | Helm chart | Chart.yaml | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 378 | PH-1 | 第 3-4 周 | match | Helm chart | values.yaml | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 379 | PH-1 | 第 3-4 周 | match | Helm chart | deployment + HPA | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 380 | PH-1 | 第 3-4 周 | match | Helm chart | NetworkPolicy | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 381 | PH-1 | 第 3-4 周 | match | observability | OTel spans | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 382 | PH-1 | 第 3-4 周 | match | observability | Prometheus metrics | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 383 | PH-1 | 第 3-4 周 | match | observability | Grafana 仪表盘 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 384 | PH-1 | 第 3-4 周 | match | observability | Loki 日志 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 385 | PH-1 | 第 3-4 周 | social | API Spec | 列出 gRPC 方法 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 386 | PH-1 | 第 3-4 周 | social | API Spec | 定义 Proto 文件 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 387 | PH-1 | 第 3-4 周 | social | API Spec | 配置 tonic-build | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 388 | PH-1 | 第 3-4 周 | social | API Spec | 编译期校验 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 389 | PH-1 | 第 3-4 周 | social | 业务逻辑 | 实体表定义 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 390 | PH-1 | 第 3-4 周 | social | 业务逻辑 | 状态机实现 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 391 | PH-1 | 第 3-4 周 | social | 业务逻辑 | 错误码 + 边界条件 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 392 | PH-1 | 第 3-4 周 | social | 业务逻辑 | 核心算法 / 决策 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 393 | PH-1 | 第 3-4 周 | social | DB migration | Schema 迁移 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 394 | PH-1 | 第 3-4 周 | social | DB migration | 索引 + 约束 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 395 | PH-1 | 第 3-4 周 | social | DB migration | 双向迁移演练 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 396 | PH-1 | 第 3-4 周 | social | DB migration | 回滚预案 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 397 | PH-1 | 第 3-4 周 | social | UT 单元测试 | testkit helper | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 398 | PH-1 | 第 3-4 周 | social | UT 单元测试 | CRUD 覆盖 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 399 | PH-1 | 第 3-4 周 | social | UT 单元测试 | 状态机覆盖 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 400 | PH-1 | 第 3-4 周 | social | UT 单元测试 | 覆盖率报告 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 401 | PH-1 | 第 3-4 周 | social | IT 集成测试 | service 启动 + health | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 402 | PH-1 | 第 3-4 周 | social | IT 集成测试 | DB 集成 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 403 | PH-1 | 第 3-4 周 | social | IT 集成测试 | 跨组件契约 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 404 | PH-1 | 第 3-4 周 | social | IT 集成测试 | 端到端集成 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 405 | PH-1 | 第 3-4 周 | social | ST 系统测试 | K8s 部署验证 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 406 | PH-1 | 第 3-4 周 | social | ST 系统测试 | 性能 / 容量 NFR | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 407 | PH-1 | 第 3-4 周 | social | ST 系统测试 | chaos 故障注入 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 408 | PH-1 | 第 3-4 周 | social | ST 系统测试 | RPO/RTO 验证 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 409 | PH-1 | 第 3-4 周 | social | Helm chart | Chart.yaml | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 410 | PH-1 | 第 3-4 周 | social | Helm chart | values.yaml | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 411 | PH-1 | 第 3-4 周 | social | Helm chart | deployment + HPA | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 412 | PH-1 | 第 3-4 周 | social | Helm chart | NetworkPolicy | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 413 | PH-1 | 第 3-4 周 | social | observability | OTel spans | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 414 | PH-1 | 第 3-4 周 | social | observability | Prometheus metrics | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 415 | PH-1 | 第 3-4 周 | social | observability | Grafana 仪表盘 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 416 | PH-1 | 第 3-4 周 | social | observability | Loki 日志 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 417 | PH-1 | 第 3-4 周 | admin | API Spec | 列出 gRPC 方法 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 418 | PH-1 | 第 3-4 周 | admin | API Spec | 定义 Proto 文件 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 419 | PH-1 | 第 3-4 周 | admin | API Spec | 配置 tonic-build | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 420 | PH-1 | 第 3-4 周 | admin | API Spec | 编译期校验 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 421 | PH-1 | 第 3-4 周 | admin | 业务逻辑 | 实体表定义 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 422 | PH-1 | 第 3-4 周 | admin | 业务逻辑 | 状态机实现 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 423 | PH-1 | 第 3-4 周 | admin | 业务逻辑 | 错误码 + 边界条件 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 424 | PH-1 | 第 3-4 周 | admin | 业务逻辑 | 核心算法 / 决策 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 425 | PH-1 | 第 3-4 周 | admin | DB migration | Schema 迁移 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 426 | PH-1 | 第 3-4 周 | admin | DB migration | 索引 + 约束 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 427 | PH-1 | 第 3-4 周 | admin | DB migration | 双向迁移演练 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 428 | PH-1 | 第 3-4 周 | admin | DB migration | 回滚预案 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 429 | PH-1 | 第 3-4 周 | admin | UT 单元测试 | testkit helper | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 430 | PH-1 | 第 3-4 周 | admin | UT 单元测试 | CRUD 覆盖 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 431 | PH-1 | 第 3-4 周 | admin | UT 单元测试 | 状态机覆盖 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 432 | PH-1 | 第 3-4 周 | admin | UT 单元测试 | 覆盖率报告 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 433 | PH-1 | 第 3-4 周 | admin | IT 集成测试 | service 启动 + health | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 434 | PH-1 | 第 3-4 周 | admin | IT 集成测试 | DB 集成 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 435 | PH-1 | 第 3-4 周 | admin | IT 集成测试 | 跨组件契约 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 436 | PH-1 | 第 3-4 周 | admin | IT 集成测试 | 端到端集成 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 437 | PH-1 | 第 3-4 周 | admin | ST 系统测试 | K8s 部署验证 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 438 | PH-1 | 第 3-4 周 | admin | ST 系统测试 | 性能 / 容量 NFR | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 439 | PH-1 | 第 3-4 周 | admin | ST 系统测试 | chaos 故障注入 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 440 | PH-1 | 第 3-4 周 | admin | ST 系统测试 | RPO/RTO 验证 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 441 | PH-1 | 第 3-4 周 | admin | Helm chart | Chart.yaml | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 442 | PH-1 | 第 3-4 周 | admin | Helm chart | values.yaml | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 443 | PH-1 | 第 3-4 周 | admin | Helm chart | deployment + HPA | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 444 | PH-1 | 第 3-4 周 | admin | Helm chart | NetworkPolicy | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 445 | PH-1 | 第 3-4 周 | admin | observability | OTel spans | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 446 | PH-1 | 第 3-4 周 | admin | observability | Prometheus metrics | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 447 | PH-1 | 第 3-4 周 | admin | observability | Grafana 仪表盘 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 448 | PH-1 | 第 3-4 周 | admin | observability | Loki 日志 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 449 | PH-1 | 第 3-4 周 | cluster-ops | Control Plane API | ClusterOps gRPC 定义 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 450 | PH-1 | 第 3-4 周 | cluster-ops | Control Plane API | AdminService 转发 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 451 | PH-1 | 第 3-4 周 | cluster-ops | Control Plane API | request_id 幂等 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 452 | PH-1 | 第 3-4 周 | cluster-ops | Control Plane API | OCC 版本字段 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 453 | PH-1 | 第 3-4 周 | cluster-ops | CEM | Feature registry | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 454 | PH-1 | 第 3-4 周 | cluster-ops | CEM | 事件流 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 455 | PH-1 | 第 3-4 周 | cluster-ops | CEM | 订阅/取消订阅 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 456 | PH-1 | 第 3-4 周 | cluster-ops | CEM | DLQ 处理 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 457 | PH-1 | 第 3-4 周 | cluster-ops | PFAU | declared → canary → confirm → done 状态机 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 458 | PH-1 | 第 3-4 周 | cluster-ops | PFAU | all-reachable 确认 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 459 | PH-1 | 第 3-4 周 | cluster-ops | PFAU | 灰度策略 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 460 | PH-1 | 第 3-4 周 | cluster-ops | PFAU | 回滚路径 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 461 | PH-1 | 第 3-4 周 | cluster-ops | 状态机 | feature 状态定义 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 462 | PH-1 | 第 3-4 周 | cluster-ops | 状态机 | 非法转移检测 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 463 | PH-1 | 第 3-4 周 | cluster-ops | 状态机 | 状态转移图 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 464 | PH-1 | 第 3-4 周 | cluster-ops | 状态机 | 持久化方案 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 465 | PH-1 | 第 3-4 周 | cluster-ops | RBAC | GM / COC / 客户端 3 套权限 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 466 | PH-1 | 第 3-4 周 | cluster-ops | RBAC | 权限矩阵 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 467 | PH-1 | 第 3-4 周 | cluster-ops | RBAC | 审计日志 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 468 | PH-1 | 第 3-4 周 | cluster-ops | RBAC | 撤销机制 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 469 | PH-1 | 第 3-4 周 | cluster-ops | fencing | 租约机制 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 470 | PH-1 | 第 3-4 周 | cluster-ops | fencing | CAS 版本控制 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 471 | PH-1 | 第 3-4 周 | cluster-ops | fencing | stale leader 检测 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 472 | PH-1 | 第 3-4 周 | cluster-ops | fencing | 集群隔离策略 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 473 | PH-1 | 第 3-4 周 | cluster-ops | 审计 | 审计 schema | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 474 | PH-1 | 第 3-4 周 | cluster-ops | 审计 | 写入路径 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 475 | PH-1 | 第 3-4 周 | cluster-ops | 审计 | 查询接口 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 476 | PH-1 | 第 3-4 周 | cluster-ops | 审计 | 保留期 + 归档 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 477 | PH-1 | 第 3-4 周 | cluster-ops | OCC | 乐观并发控制 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 478 | PH-1 | 第 3-4 周 | cluster-ops | OCC | 重试策略 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 479 | PH-1 | 第 3-4 周 | cluster-ops | OCC | 冲突检测 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 480 | PH-1 | 第 3-4 周 | cluster-ops | OCC | 死锁恢复 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 481 | PH-1 | 第 3-4 周 | shared-platform | Rust 工具链 | rustup 1.98 锁定 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 482 | PH-1 | 第 3-4 周 | shared-platform | Rust 工具链 | rust-toolchain.toml | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 483 | PH-1 | 第 3-4 周 | shared-platform | Rust 工具链 | CI cache | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 484 | PH-1 | 第 3-4 周 | shared-platform | Rust 工具链 | 升级评审 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 485 | PH-1 | 第 3-4 周 | shared-platform | Cargo.lock 锁定 | --locked 构建 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 486 | PH-1 | 第 3-4 周 | shared-platform | Cargo.lock 锁定 | workspace 统一锁 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 487 | PH-1 | 第 3-4 周 | shared-platform | Cargo.lock 锁定 | 依赖审计 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 488 | PH-1 | 第 3-4 周 | shared-platform | Cargo.lock 锁定 | 更新策略 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 489 | PH-1 | 第 3-4 周 | shared-platform | 镜像构建 | Dockerfile.distroless | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 490 | PH-1 | 第 3-4 周 | shared-platform | 镜像构建 | 镜像大小优化 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 491 | PH-1 | 第 3-4 周 | shared-platform | 镜像构建 | SBOM 生成 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 492 | PH-1 | 第 3-4 周 | shared-platform | 镜像构建 | 漏洞扫描 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 493 | PH-1 | 第 3-4 周 | shared-platform | K3s | K3s 集群初始化 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 494 | PH-1 | 第 3-4 周 | shared-platform | K3s | kubectl 配置 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 495 | PH-1 | 第 3-4 周 | shared-platform | K3s | NetworkPolicy | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 496 | PH-1 | 第 3-4 周 | shared-platform | K3s | HPA / VPA | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 497 | PH-1 | 第 3-4 周 | shared-platform | OTel Collector | 采集器配置 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 498 | PH-1 | 第 3-4 周 | shared-platform | OTel Collector | OTLP 接收 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 499 | PH-1 | 第 3-4 周 | shared-platform | OTel Collector | 采样策略 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 500 | PH-1 | 第 3-4 周 | shared-platform | OTel Collector | 导出器配置 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 501 | PH-1 | 第 3-4 周 | shared-platform | Helm | Chart 模板 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 502 | PH-1 | 第 3-4 周 | shared-platform | Helm | values 校验 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 503 | PH-1 | 第 3-4 周 | shared-platform | Helm | 依赖管理 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 504 | PH-1 | 第 3-4 周 | shared-platform | Helm | CI render | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 505 | PH-1 | 第 3-4 周 | shared-platform | 密钥 | Vault/OpenBao 部署 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 506 | PH-1 | 第 3-4 周 | shared-platform | 密钥 | 密钥轮换策略 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 507 | PH-1 | 第 3-4 周 | shared-platform | 密钥 | 运行时注入 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 508 | PH-1 | 第 3-4 周 | shared-platform | 密钥 | 审计 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 509 | PH-1 | 第 3-4 周 | shared-platform | 灾备 | 备份策略 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 510 | PH-1 | 第 3-4 周 | shared-platform | 灾备 | 恢复演练 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 511 | PH-1 | 第 3-4 周 | shared-platform | 灾备 | 跨 AZ 复制 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 512 | PH-1 | 第 3-4 周 | shared-platform | 灾备 | RTO/RPO 验证 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 513 | PH-2 | 第 5-6 周 | foundation | workspace 骨架 | virtual workspace 配置 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 514 | PH-2 | 第 5-6 周 | foundation | workspace 骨架 | resolver=3 锁定 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 515 | PH-2 | 第 5-6 周 | foundation | workspace 骨架 | Edition 2024 升级 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 516 | PH-2 | 第 5-6 周 | foundation | workspace 骨架 | crate 间依赖方向 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 517 | PH-2 | 第 5-6 周 | foundation | testkit 共用 | testcontainers PG 封装 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 518 | PH-2 | 第 5-6 周 | foundation | testkit 共用 | mock helpers | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 519 | PH-2 | 第 5-6 周 | foundation | testkit 共用 | fixture builders | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 520 | PH-2 | 第 5-6 周 | foundation | testkit 共用 | coverage 报告 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 521 | PH-2 | 第 5-6 周 | foundation | CI 工具链 | GitHub Actions 配置 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 522 | PH-2 | 第 5-6 周 | foundation | CI 工具链 | cargo fmt/clippy/deny | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 523 | PH-2 | 第 5-6 周 | foundation | CI 工具链 | sqlx prepare check | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 524 | PH-2 | 第 5-6 周 | foundation | CI 工具链 | manifest 校验 CI | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 525 | PH-2 | 第 5-6 周 | foundation | DAG validator | 拓扑排序算法 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 526 | PH-2 | 第 5-6 周 | foundation | DAG validator | 环依赖检测 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 527 | PH-2 | 第 5-6 周 | foundation | DAG validator | 缺祖先检测 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 528 | PH-2 | 第 5-6 周 | foundation | DAG validator | 负例测试套件 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 529 | PH-2 | 第 5-6 周 | foundation | cargo-deny/audit | 许可证白名单 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 530 | PH-2 | 第 5-6 周 | foundation | cargo-deny/audit | 漏洞数据库 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 531 | PH-2 | 第 5-6 周 | foundation | cargo-deny/audit | 依赖来源限制 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 532 | PH-2 | 第 5-6 周 | foundation | cargo-deny/audit | CI 集成 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 533 | PH-2 | 第 5-6 周 | foundation | manifest schema | JSON Schema 起草 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 534 | PH-2 | 第 5-6 周 | foundation | manifest schema | ARC-042 字段映射 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 535 | PH-2 | 第 5-6 周 | foundation | manifest schema | schema 校验 CLI | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 536 | PH-2 | 第 5-6 周 | foundation | manifest schema | 示例 manifest | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 537 | PH-2 | 第 5-6 周 | foundation | 文档生成 | mdbook 配置 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 538 | PH-2 | 第 5-6 周 | foundation | 文档生成 | Doxygen/Rustdoc | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 539 | PH-2 | 第 5-6 周 | foundation | 文档生成 | CR 链接检查 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 540 | PH-2 | 第 5-6 周 | foundation | 文档生成 | CI 文档门禁 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 541 | PH-2 | 第 5-6 周 | foundation | 工程约定 | 错误码定义 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 542 | PH-2 | 第 5-6 周 | foundation | 工程约定 | 序列化约定 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 543 | PH-2 | 第 5-6 周 | foundation | 工程约定 | 日志规范 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 544 | PH-2 | 第 5-6 周 | foundation | 工程约定 | metrics 命名 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 545 | PH-2 | 第 5-6 周 | player | API Spec | 列出 gRPC 方法 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 546 | PH-2 | 第 5-6 周 | player | API Spec | 定义 Proto 文件 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 547 | PH-2 | 第 5-6 周 | player | API Spec | 配置 tonic-build | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 548 | PH-2 | 第 5-6 周 | player | API Spec | 编译期校验 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 549 | PH-2 | 第 5-6 周 | player | 业务逻辑 | 实体表定义 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 550 | PH-2 | 第 5-6 周 | player | 业务逻辑 | 状态机实现 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 551 | PH-2 | 第 5-6 周 | player | 业务逻辑 | 错误码 + 边界条件 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 552 | PH-2 | 第 5-6 周 | player | 业务逻辑 | 核心算法 / 决策 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 553 | PH-2 | 第 5-6 周 | player | DB migration | Schema 迁移 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 554 | PH-2 | 第 5-6 周 | player | DB migration | 索引 + 约束 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 555 | PH-2 | 第 5-6 周 | player | DB migration | 双向迁移演练 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 556 | PH-2 | 第 5-6 周 | player | DB migration | 回滚预案 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 557 | PH-2 | 第 5-6 周 | player | UT 单元测试 | testkit helper | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 558 | PH-2 | 第 5-6 周 | player | UT 单元测试 | CRUD 覆盖 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 559 | PH-2 | 第 5-6 周 | player | UT 单元测试 | 状态机覆盖 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 560 | PH-2 | 第 5-6 周 | player | UT 单元测试 | 覆盖率报告 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 561 | PH-2 | 第 5-6 周 | player | IT 集成测试 | service 启动 + health | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 562 | PH-2 | 第 5-6 周 | player | IT 集成测试 | DB 集成 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 563 | PH-2 | 第 5-6 周 | player | IT 集成测试 | 跨组件契约 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 564 | PH-2 | 第 5-6 周 | player | IT 集成测试 | 端到端集成 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 565 | PH-2 | 第 5-6 周 | player | ST 系统测试 | K8s 部署验证 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 566 | PH-2 | 第 5-6 周 | player | ST 系统测试 | 性能 / 容量 NFR | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 567 | PH-2 | 第 5-6 周 | player | ST 系统测试 | chaos 故障注入 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 568 | PH-2 | 第 5-6 周 | player | ST 系统测试 | RPO/RTO 验证 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 569 | PH-2 | 第 5-6 周 | player | Helm chart | Chart.yaml | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 570 | PH-2 | 第 5-6 周 | player | Helm chart | values.yaml | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 571 | PH-2 | 第 5-6 周 | player | Helm chart | deployment + HPA | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 572 | PH-2 | 第 5-6 周 | player | Helm chart | NetworkPolicy | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 573 | PH-2 | 第 5-6 周 | player | observability | OTel spans | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 574 | PH-2 | 第 5-6 周 | player | observability | Prometheus metrics | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 575 | PH-2 | 第 5-6 周 | player | observability | Grafana 仪表盘 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 576 | PH-2 | 第 5-6 周 | player | observability | Loki 日志 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 577 | PH-2 | 第 5-6 周 | economy | API Spec | 列出 gRPC 方法 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 578 | PH-2 | 第 5-6 周 | economy | API Spec | 定义 Proto 文件 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 579 | PH-2 | 第 5-6 周 | economy | API Spec | 配置 tonic-build | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 580 | PH-2 | 第 5-6 周 | economy | API Spec | 编译期校验 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 581 | PH-2 | 第 5-6 周 | economy | 业务逻辑 | 实体表定义 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 582 | PH-2 | 第 5-6 周 | economy | 业务逻辑 | 状态机实现 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 583 | PH-2 | 第 5-6 周 | economy | 业务逻辑 | 错误码 + 边界条件 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 584 | PH-2 | 第 5-6 周 | economy | 业务逻辑 | 核心算法 / 决策 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 585 | PH-2 | 第 5-6 周 | economy | DB migration | Schema 迁移 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 586 | PH-2 | 第 5-6 周 | economy | DB migration | 索引 + 约束 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 587 | PH-2 | 第 5-6 周 | economy | DB migration | 双向迁移演练 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 588 | PH-2 | 第 5-6 周 | economy | DB migration | 回滚预案 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 589 | PH-2 | 第 5-6 周 | economy | UT 单元测试 | testkit helper | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 590 | PH-2 | 第 5-6 周 | economy | UT 单元测试 | CRUD 覆盖 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 591 | PH-2 | 第 5-6 周 | economy | UT 单元测试 | 状态机覆盖 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 592 | PH-2 | 第 5-6 周 | economy | UT 单元测试 | 覆盖率报告 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 593 | PH-2 | 第 5-6 周 | economy | IT 集成测试 | service 启动 + health | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 594 | PH-2 | 第 5-6 周 | economy | IT 集成测试 | DB 集成 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 595 | PH-2 | 第 5-6 周 | economy | IT 集成测试 | 跨组件契约 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 596 | PH-2 | 第 5-6 周 | economy | IT 集成测试 | 端到端集成 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 597 | PH-2 | 第 5-6 周 | economy | ST 系统测试 | K8s 部署验证 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 598 | PH-2 | 第 5-6 周 | economy | ST 系统测试 | 性能 / 容量 NFR | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 599 | PH-2 | 第 5-6 周 | economy | ST 系统测试 | chaos 故障注入 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 600 | PH-2 | 第 5-6 周 | economy | ST 系统测试 | RPO/RTO 验证 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 601 | PH-2 | 第 5-6 周 | economy | Helm chart | Chart.yaml | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 602 | PH-2 | 第 5-6 周 | economy | Helm chart | values.yaml | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 603 | PH-2 | 第 5-6 周 | economy | Helm chart | deployment + HPA | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 604 | PH-2 | 第 5-6 周 | economy | Helm chart | NetworkPolicy | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 605 | PH-2 | 第 5-6 周 | economy | observability | OTel spans | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 606 | PH-2 | 第 5-6 周 | economy | observability | Prometheus metrics | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 607 | PH-2 | 第 5-6 周 | economy | observability | Grafana 仪表盘 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 608 | PH-2 | 第 5-6 周 | economy | observability | Loki 日志 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 609 | PH-2 | 第 5-6 周 | match | API Spec | 列出 gRPC 方法 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 610 | PH-2 | 第 5-6 周 | match | API Spec | 定义 Proto 文件 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 611 | PH-2 | 第 5-6 周 | match | API Spec | 配置 tonic-build | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 612 | PH-2 | 第 5-6 周 | match | API Spec | 编译期校验 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 613 | PH-2 | 第 5-6 周 | match | 业务逻辑 | 实体表定义 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 614 | PH-2 | 第 5-6 周 | match | 业务逻辑 | 状态机实现 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 615 | PH-2 | 第 5-6 周 | match | 业务逻辑 | 错误码 + 边界条件 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 616 | PH-2 | 第 5-6 周 | match | 业务逻辑 | 核心算法 / 决策 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 617 | PH-2 | 第 5-6 周 | match | DB migration | Schema 迁移 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 618 | PH-2 | 第 5-6 周 | match | DB migration | 索引 + 约束 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 619 | PH-2 | 第 5-6 周 | match | DB migration | 双向迁移演练 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 620 | PH-2 | 第 5-6 周 | match | DB migration | 回滚预案 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 621 | PH-2 | 第 5-6 周 | match | UT 单元测试 | testkit helper | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 622 | PH-2 | 第 5-6 周 | match | UT 单元测试 | CRUD 覆盖 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 623 | PH-2 | 第 5-6 周 | match | UT 单元测试 | 状态机覆盖 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 624 | PH-2 | 第 5-6 周 | match | UT 单元测试 | 覆盖率报告 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 625 | PH-2 | 第 5-6 周 | match | IT 集成测试 | service 启动 + health | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 626 | PH-2 | 第 5-6 周 | match | IT 集成测试 | DB 集成 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 627 | PH-2 | 第 5-6 周 | match | IT 集成测试 | 跨组件契约 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 628 | PH-2 | 第 5-6 周 | match | IT 集成测试 | 端到端集成 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 629 | PH-2 | 第 5-6 周 | match | ST 系统测试 | K8s 部署验证 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 630 | PH-2 | 第 5-6 周 | match | ST 系统测试 | 性能 / 容量 NFR | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 631 | PH-2 | 第 5-6 周 | match | ST 系统测试 | chaos 故障注入 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 632 | PH-2 | 第 5-6 周 | match | ST 系统测试 | RPO/RTO 验证 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 633 | PH-2 | 第 5-6 周 | match | Helm chart | Chart.yaml | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 634 | PH-2 | 第 5-6 周 | match | Helm chart | values.yaml | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 635 | PH-2 | 第 5-6 周 | match | Helm chart | deployment + HPA | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 636 | PH-2 | 第 5-6 周 | match | Helm chart | NetworkPolicy | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 637 | PH-2 | 第 5-6 周 | match | observability | OTel spans | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 638 | PH-2 | 第 5-6 周 | match | observability | Prometheus metrics | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 639 | PH-2 | 第 5-6 周 | match | observability | Grafana 仪表盘 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 640 | PH-2 | 第 5-6 周 | match | observability | Loki 日志 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 641 | PH-2 | 第 5-6 周 | social | API Spec | 列出 gRPC 方法 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 642 | PH-2 | 第 5-6 周 | social | API Spec | 定义 Proto 文件 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 643 | PH-2 | 第 5-6 周 | social | API Spec | 配置 tonic-build | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 644 | PH-2 | 第 5-6 周 | social | API Spec | 编译期校验 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 645 | PH-2 | 第 5-6 周 | social | 业务逻辑 | 实体表定义 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 646 | PH-2 | 第 5-6 周 | social | 业务逻辑 | 状态机实现 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 647 | PH-2 | 第 5-6 周 | social | 业务逻辑 | 错误码 + 边界条件 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 648 | PH-2 | 第 5-6 周 | social | 业务逻辑 | 核心算法 / 决策 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 649 | PH-2 | 第 5-6 周 | social | DB migration | Schema 迁移 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 650 | PH-2 | 第 5-6 周 | social | DB migration | 索引 + 约束 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 651 | PH-2 | 第 5-6 周 | social | DB migration | 双向迁移演练 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 652 | PH-2 | 第 5-6 周 | social | DB migration | 回滚预案 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 653 | PH-2 | 第 5-6 周 | social | UT 单元测试 | testkit helper | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 654 | PH-2 | 第 5-6 周 | social | UT 单元测试 | CRUD 覆盖 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 655 | PH-2 | 第 5-6 周 | social | UT 单元测试 | 状态机覆盖 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 656 | PH-2 | 第 5-6 周 | social | UT 单元测试 | 覆盖率报告 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 657 | PH-2 | 第 5-6 周 | social | IT 集成测试 | service 启动 + health | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 658 | PH-2 | 第 5-6 周 | social | IT 集成测试 | DB 集成 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 659 | PH-2 | 第 5-6 周 | social | IT 集成测试 | 跨组件契约 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 660 | PH-2 | 第 5-6 周 | social | IT 集成测试 | 端到端集成 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 661 | PH-2 | 第 5-6 周 | social | ST 系统测试 | K8s 部署验证 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 662 | PH-2 | 第 5-6 周 | social | ST 系统测试 | 性能 / 容量 NFR | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 663 | PH-2 | 第 5-6 周 | social | ST 系统测试 | chaos 故障注入 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 664 | PH-2 | 第 5-6 周 | social | ST 系统测试 | RPO/RTO 验证 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 665 | PH-2 | 第 5-6 周 | social | Helm chart | Chart.yaml | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 666 | PH-2 | 第 5-6 周 | social | Helm chart | values.yaml | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 667 | PH-2 | 第 5-6 周 | social | Helm chart | deployment + HPA | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 668 | PH-2 | 第 5-6 周 | social | Helm chart | NetworkPolicy | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 669 | PH-2 | 第 5-6 周 | social | observability | OTel spans | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 670 | PH-2 | 第 5-6 周 | social | observability | Prometheus metrics | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 671 | PH-2 | 第 5-6 周 | social | observability | Grafana 仪表盘 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 672 | PH-2 | 第 5-6 周 | social | observability | Loki 日志 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 673 | PH-2 | 第 5-6 周 | admin | API Spec | 列出 gRPC 方法 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 674 | PH-2 | 第 5-6 周 | admin | API Spec | 定义 Proto 文件 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 675 | PH-2 | 第 5-6 周 | admin | API Spec | 配置 tonic-build | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 676 | PH-2 | 第 5-6 周 | admin | API Spec | 编译期校验 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 677 | PH-2 | 第 5-6 周 | admin | 业务逻辑 | 实体表定义 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 678 | PH-2 | 第 5-6 周 | admin | 业务逻辑 | 状态机实现 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 679 | PH-2 | 第 5-6 周 | admin | 业务逻辑 | 错误码 + 边界条件 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 680 | PH-2 | 第 5-6 周 | admin | 业务逻辑 | 核心算法 / 决策 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 681 | PH-2 | 第 5-6 周 | admin | DB migration | Schema 迁移 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 682 | PH-2 | 第 5-6 周 | admin | DB migration | 索引 + 约束 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 683 | PH-2 | 第 5-6 周 | admin | DB migration | 双向迁移演练 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 684 | PH-2 | 第 5-6 周 | admin | DB migration | 回滚预案 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 685 | PH-2 | 第 5-6 周 | admin | UT 单元测试 | testkit helper | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 686 | PH-2 | 第 5-6 周 | admin | UT 单元测试 | CRUD 覆盖 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 687 | PH-2 | 第 5-6 周 | admin | UT 单元测试 | 状态机覆盖 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 688 | PH-2 | 第 5-6 周 | admin | UT 单元测试 | 覆盖率报告 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 689 | PH-2 | 第 5-6 周 | admin | IT 集成测试 | service 启动 + health | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 690 | PH-2 | 第 5-6 周 | admin | IT 集成测试 | DB 集成 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 691 | PH-2 | 第 5-6 周 | admin | IT 集成测试 | 跨组件契约 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 692 | PH-2 | 第 5-6 周 | admin | IT 集成测试 | 端到端集成 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 693 | PH-2 | 第 5-6 周 | admin | ST 系统测试 | K8s 部署验证 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 694 | PH-2 | 第 5-6 周 | admin | ST 系统测试 | 性能 / 容量 NFR | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 695 | PH-2 | 第 5-6 周 | admin | ST 系统测试 | chaos 故障注入 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 696 | PH-2 | 第 5-6 周 | admin | ST 系统测试 | RPO/RTO 验证 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 697 | PH-2 | 第 5-6 周 | admin | Helm chart | Chart.yaml | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 698 | PH-2 | 第 5-6 周 | admin | Helm chart | values.yaml | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 699 | PH-2 | 第 5-6 周 | admin | Helm chart | deployment + HPA | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 700 | PH-2 | 第 5-6 周 | admin | Helm chart | NetworkPolicy | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 701 | PH-2 | 第 5-6 周 | admin | observability | OTel spans | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 702 | PH-2 | 第 5-6 周 | admin | observability | Prometheus metrics | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 703 | PH-2 | 第 5-6 周 | admin | observability | Grafana 仪表盘 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 704 | PH-2 | 第 5-6 周 | admin | observability | Loki 日志 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 705 | PH-2 | 第 5-6 周 | cluster-ops | Control Plane API | ClusterOps gRPC 定义 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 706 | PH-2 | 第 5-6 周 | cluster-ops | Control Plane API | AdminService 转发 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 707 | PH-2 | 第 5-6 周 | cluster-ops | Control Plane API | request_id 幂等 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 708 | PH-2 | 第 5-6 周 | cluster-ops | Control Plane API | OCC 版本字段 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 709 | PH-2 | 第 5-6 周 | cluster-ops | CEM | Feature registry | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 710 | PH-2 | 第 5-6 周 | cluster-ops | CEM | 事件流 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 711 | PH-2 | 第 5-6 周 | cluster-ops | CEM | 订阅/取消订阅 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 712 | PH-2 | 第 5-6 周 | cluster-ops | CEM | DLQ 处理 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 713 | PH-2 | 第 5-6 周 | cluster-ops | PFAU | declared → canary → confirm → done 状态机 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 714 | PH-2 | 第 5-6 周 | cluster-ops | PFAU | all-reachable 确认 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 715 | PH-2 | 第 5-6 周 | cluster-ops | PFAU | 灰度策略 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 716 | PH-2 | 第 5-6 周 | cluster-ops | PFAU | 回滚路径 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 717 | PH-2 | 第 5-6 周 | cluster-ops | 状态机 | feature 状态定义 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 718 | PH-2 | 第 5-6 周 | cluster-ops | 状态机 | 非法转移检测 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 719 | PH-2 | 第 5-6 周 | cluster-ops | 状态机 | 状态转移图 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 720 | PH-2 | 第 5-6 周 | cluster-ops | 状态机 | 持久化方案 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 721 | PH-2 | 第 5-6 周 | cluster-ops | RBAC | GM / COC / 客户端 3 套权限 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 722 | PH-2 | 第 5-6 周 | cluster-ops | RBAC | 权限矩阵 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 723 | PH-2 | 第 5-6 周 | cluster-ops | RBAC | 审计日志 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 724 | PH-2 | 第 5-6 周 | cluster-ops | RBAC | 撤销机制 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 725 | PH-2 | 第 5-6 周 | cluster-ops | fencing | 租约机制 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 726 | PH-2 | 第 5-6 周 | cluster-ops | fencing | CAS 版本控制 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 727 | PH-2 | 第 5-6 周 | cluster-ops | fencing | stale leader 检测 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 728 | PH-2 | 第 5-6 周 | cluster-ops | fencing | 集群隔离策略 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 729 | PH-2 | 第 5-6 周 | cluster-ops | 审计 | 审计 schema | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 730 | PH-2 | 第 5-6 周 | cluster-ops | 审计 | 写入路径 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 731 | PH-2 | 第 5-6 周 | cluster-ops | 审计 | 查询接口 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 732 | PH-2 | 第 5-6 周 | cluster-ops | 审计 | 保留期 + 归档 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 733 | PH-2 | 第 5-6 周 | cluster-ops | OCC | 乐观并发控制 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 734 | PH-2 | 第 5-6 周 | cluster-ops | OCC | 重试策略 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 735 | PH-2 | 第 5-6 周 | cluster-ops | OCC | 冲突检测 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 736 | PH-2 | 第 5-6 周 | cluster-ops | OCC | 死锁恢复 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 737 | PH-2 | 第 5-6 周 | shared-platform | Rust 工具链 | rustup 1.98 锁定 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 738 | PH-2 | 第 5-6 周 | shared-platform | Rust 工具链 | rust-toolchain.toml | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 739 | PH-2 | 第 5-6 周 | shared-platform | Rust 工具链 | CI cache | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 740 | PH-2 | 第 5-6 周 | shared-platform | Rust 工具链 | 升级评审 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 741 | PH-2 | 第 5-6 周 | shared-platform | Cargo.lock 锁定 | --locked 构建 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 742 | PH-2 | 第 5-6 周 | shared-platform | Cargo.lock 锁定 | workspace 统一锁 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 743 | PH-2 | 第 5-6 周 | shared-platform | Cargo.lock 锁定 | 依赖审计 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 744 | PH-2 | 第 5-6 周 | shared-platform | Cargo.lock 锁定 | 更新策略 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 745 | PH-2 | 第 5-6 周 | shared-platform | 镜像构建 | Dockerfile.distroless | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 746 | PH-2 | 第 5-6 周 | shared-platform | 镜像构建 | 镜像大小优化 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 747 | PH-2 | 第 5-6 周 | shared-platform | 镜像构建 | SBOM 生成 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 748 | PH-2 | 第 5-6 周 | shared-platform | 镜像构建 | 漏洞扫描 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 749 | PH-2 | 第 5-6 周 | shared-platform | K3s | K3s 集群初始化 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 750 | PH-2 | 第 5-6 周 | shared-platform | K3s | kubectl 配置 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 751 | PH-2 | 第 5-6 周 | shared-platform | K3s | NetworkPolicy | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 752 | PH-2 | 第 5-6 周 | shared-platform | K3s | HPA / VPA | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 753 | PH-2 | 第 5-6 周 | shared-platform | OTel Collector | 采集器配置 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 754 | PH-2 | 第 5-6 周 | shared-platform | OTel Collector | OTLP 接收 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 755 | PH-2 | 第 5-6 周 | shared-platform | OTel Collector | 采样策略 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 756 | PH-2 | 第 5-6 周 | shared-platform | OTel Collector | 导出器配置 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 757 | PH-2 | 第 5-6 周 | shared-platform | Helm | Chart 模板 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 758 | PH-2 | 第 5-6 周 | shared-platform | Helm | values 校验 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 759 | PH-2 | 第 5-6 周 | shared-platform | Helm | 依赖管理 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 760 | PH-2 | 第 5-6 周 | shared-platform | Helm | CI render | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 761 | PH-2 | 第 5-6 周 | shared-platform | 密钥 | Vault/OpenBao 部署 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 762 | PH-2 | 第 5-6 周 | shared-platform | 密钥 | 密钥轮换策略 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 763 | PH-2 | 第 5-6 周 | shared-platform | 密钥 | 运行时注入 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 764 | PH-2 | 第 5-6 周 | shared-platform | 密钥 | 审计 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 765 | PH-2 | 第 5-6 周 | shared-platform | 灾备 | 备份策略 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 766 | PH-2 | 第 5-6 周 | shared-platform | 灾备 | 恢复演练 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 767 | PH-2 | 第 5-6 周 | shared-platform | 灾备 | 跨 AZ 复制 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 768 | PH-2 | 第 5-6 周 | shared-platform | 灾备 | RTO/RPO 验证 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 769 | PH-3 | 第 7-9 周 | foundation | workspace 骨架 | virtual workspace 配置 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 770 | PH-3 | 第 7-9 周 | foundation | workspace 骨架 | resolver=3 锁定 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 771 | PH-3 | 第 7-9 周 | foundation | workspace 骨架 | Edition 2024 升级 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 772 | PH-3 | 第 7-9 周 | foundation | workspace 骨架 | crate 间依赖方向 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 773 | PH-3 | 第 7-9 周 | foundation | testkit 共用 | testcontainers PG 封装 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 774 | PH-3 | 第 7-9 周 | foundation | testkit 共用 | mock helpers | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 775 | PH-3 | 第 7-9 周 | foundation | testkit 共用 | fixture builders | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 776 | PH-3 | 第 7-9 周 | foundation | testkit 共用 | coverage 报告 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 777 | PH-3 | 第 7-9 周 | foundation | CI 工具链 | GitHub Actions 配置 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 778 | PH-3 | 第 7-9 周 | foundation | CI 工具链 | cargo fmt/clippy/deny | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 779 | PH-3 | 第 7-9 周 | foundation | CI 工具链 | sqlx prepare check | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 780 | PH-3 | 第 7-9 周 | foundation | CI 工具链 | manifest 校验 CI | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 781 | PH-3 | 第 7-9 周 | foundation | DAG validator | 拓扑排序算法 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 782 | PH-3 | 第 7-9 周 | foundation | DAG validator | 环依赖检测 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 783 | PH-3 | 第 7-9 周 | foundation | DAG validator | 缺祖先检测 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 784 | PH-3 | 第 7-9 周 | foundation | DAG validator | 负例测试套件 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 785 | PH-3 | 第 7-9 周 | foundation | cargo-deny/audit | 许可证白名单 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 786 | PH-3 | 第 7-9 周 | foundation | cargo-deny/audit | 漏洞数据库 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 787 | PH-3 | 第 7-9 周 | foundation | cargo-deny/audit | 依赖来源限制 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 788 | PH-3 | 第 7-9 周 | foundation | cargo-deny/audit | CI 集成 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 789 | PH-3 | 第 7-9 周 | foundation | manifest schema | JSON Schema 起草 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 790 | PH-3 | 第 7-9 周 | foundation | manifest schema | ARC-042 字段映射 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 791 | PH-3 | 第 7-9 周 | foundation | manifest schema | schema 校验 CLI | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 792 | PH-3 | 第 7-9 周 | foundation | manifest schema | 示例 manifest | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 793 | PH-3 | 第 7-9 周 | foundation | 文档生成 | mdbook 配置 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 794 | PH-3 | 第 7-9 周 | foundation | 文档生成 | Doxygen/Rustdoc | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 795 | PH-3 | 第 7-9 周 | foundation | 文档生成 | CR 链接检查 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 796 | PH-3 | 第 7-9 周 | foundation | 文档生成 | CI 文档门禁 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 797 | PH-3 | 第 7-9 周 | foundation | 工程约定 | 错误码定义 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 798 | PH-3 | 第 7-9 周 | foundation | 工程约定 | 序列化约定 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 799 | PH-3 | 第 7-9 周 | foundation | 工程约定 | 日志规范 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 800 | PH-3 | 第 7-9 周 | foundation | 工程约定 | metrics 命名 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 801 | PH-3 | 第 7-9 周 | player | API Spec | 列出 gRPC 方法 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 802 | PH-3 | 第 7-9 周 | player | API Spec | 定义 Proto 文件 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 803 | PH-3 | 第 7-9 周 | player | API Spec | 配置 tonic-build | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 804 | PH-3 | 第 7-9 周 | player | API Spec | 编译期校验 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 805 | PH-3 | 第 7-9 周 | player | 业务逻辑 | 实体表定义 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 806 | PH-3 | 第 7-9 周 | player | 业务逻辑 | 状态机实现 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 807 | PH-3 | 第 7-9 周 | player | 业务逻辑 | 错误码 + 边界条件 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 808 | PH-3 | 第 7-9 周 | player | 业务逻辑 | 核心算法 / 决策 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 809 | PH-3 | 第 7-9 周 | player | DB migration | Schema 迁移 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 810 | PH-3 | 第 7-9 周 | player | DB migration | 索引 + 约束 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 811 | PH-3 | 第 7-9 周 | player | DB migration | 双向迁移演练 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 812 | PH-3 | 第 7-9 周 | player | DB migration | 回滚预案 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 813 | PH-3 | 第 7-9 周 | player | UT 单元测试 | testkit helper | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 814 | PH-3 | 第 7-9 周 | player | UT 单元测试 | CRUD 覆盖 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 815 | PH-3 | 第 7-9 周 | player | UT 单元测试 | 状态机覆盖 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 816 | PH-3 | 第 7-9 周 | player | UT 单元测试 | 覆盖率报告 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 817 | PH-3 | 第 7-9 周 | player | IT 集成测试 | service 启动 + health | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 818 | PH-3 | 第 7-9 周 | player | IT 集成测试 | DB 集成 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 819 | PH-3 | 第 7-9 周 | player | IT 集成测试 | 跨组件契约 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 820 | PH-3 | 第 7-9 周 | player | IT 集成测试 | 端到端集成 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 821 | PH-3 | 第 7-9 周 | player | ST 系统测试 | K8s 部署验证 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 822 | PH-3 | 第 7-9 周 | player | ST 系统测试 | 性能 / 容量 NFR | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 823 | PH-3 | 第 7-9 周 | player | ST 系统测试 | chaos 故障注入 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 824 | PH-3 | 第 7-9 周 | player | ST 系统测试 | RPO/RTO 验证 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 825 | PH-3 | 第 7-9 周 | player | Helm chart | Chart.yaml | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 826 | PH-3 | 第 7-9 周 | player | Helm chart | values.yaml | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 827 | PH-3 | 第 7-9 周 | player | Helm chart | deployment + HPA | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 828 | PH-3 | 第 7-9 周 | player | Helm chart | NetworkPolicy | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 829 | PH-3 | 第 7-9 周 | player | observability | OTel spans | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 830 | PH-3 | 第 7-9 周 | player | observability | Prometheus metrics | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 831 | PH-3 | 第 7-9 周 | player | observability | Grafana 仪表盘 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 832 | PH-3 | 第 7-9 周 | player | observability | Loki 日志 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 833 | PH-3 | 第 7-9 周 | economy | API Spec | 列出 gRPC 方法 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 834 | PH-3 | 第 7-9 周 | economy | API Spec | 定义 Proto 文件 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 835 | PH-3 | 第 7-9 周 | economy | API Spec | 配置 tonic-build | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 836 | PH-3 | 第 7-9 周 | economy | API Spec | 编译期校验 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 837 | PH-3 | 第 7-9 周 | economy | 业务逻辑 | 实体表定义 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 838 | PH-3 | 第 7-9 周 | economy | 业务逻辑 | 状态机实现 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 839 | PH-3 | 第 7-9 周 | economy | 业务逻辑 | 错误码 + 边界条件 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 840 | PH-3 | 第 7-9 周 | economy | 业务逻辑 | 核心算法 / 决策 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 841 | PH-3 | 第 7-9 周 | economy | DB migration | Schema 迁移 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 842 | PH-3 | 第 7-9 周 | economy | DB migration | 索引 + 约束 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 843 | PH-3 | 第 7-9 周 | economy | DB migration | 双向迁移演练 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 844 | PH-3 | 第 7-9 周 | economy | DB migration | 回滚预案 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 845 | PH-3 | 第 7-9 周 | economy | UT 单元测试 | testkit helper | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 846 | PH-3 | 第 7-9 周 | economy | UT 单元测试 | CRUD 覆盖 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 847 | PH-3 | 第 7-9 周 | economy | UT 单元测试 | 状态机覆盖 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 848 | PH-3 | 第 7-9 周 | economy | UT 单元测试 | 覆盖率报告 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 849 | PH-3 | 第 7-9 周 | economy | IT 集成测试 | service 启动 + health | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 850 | PH-3 | 第 7-9 周 | economy | IT 集成测试 | DB 集成 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 851 | PH-3 | 第 7-9 周 | economy | IT 集成测试 | 跨组件契约 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 852 | PH-3 | 第 7-9 周 | economy | IT 集成测试 | 端到端集成 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 853 | PH-3 | 第 7-9 周 | economy | ST 系统测试 | K8s 部署验证 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 854 | PH-3 | 第 7-9 周 | economy | ST 系统测试 | 性能 / 容量 NFR | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 855 | PH-3 | 第 7-9 周 | economy | ST 系统测试 | chaos 故障注入 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 856 | PH-3 | 第 7-9 周 | economy | ST 系统测试 | RPO/RTO 验证 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 857 | PH-3 | 第 7-9 周 | economy | Helm chart | Chart.yaml | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 858 | PH-3 | 第 7-9 周 | economy | Helm chart | values.yaml | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 859 | PH-3 | 第 7-9 周 | economy | Helm chart | deployment + HPA | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 860 | PH-3 | 第 7-9 周 | economy | Helm chart | NetworkPolicy | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 861 | PH-3 | 第 7-9 周 | economy | observability | OTel spans | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 862 | PH-3 | 第 7-9 周 | economy | observability | Prometheus metrics | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 863 | PH-3 | 第 7-9 周 | economy | observability | Grafana 仪表盘 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 864 | PH-3 | 第 7-9 周 | economy | observability | Loki 日志 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 865 | PH-3 | 第 7-9 周 | match | API Spec | 列出 gRPC 方法 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 866 | PH-3 | 第 7-9 周 | match | API Spec | 定义 Proto 文件 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 867 | PH-3 | 第 7-9 周 | match | API Spec | 配置 tonic-build | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 868 | PH-3 | 第 7-9 周 | match | API Spec | 编译期校验 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 869 | PH-3 | 第 7-9 周 | match | 业务逻辑 | 实体表定义 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 870 | PH-3 | 第 7-9 周 | match | 业务逻辑 | 状态机实现 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 871 | PH-3 | 第 7-9 周 | match | 业务逻辑 | 错误码 + 边界条件 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 872 | PH-3 | 第 7-9 周 | match | 业务逻辑 | 核心算法 / 决策 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 873 | PH-3 | 第 7-9 周 | match | DB migration | Schema 迁移 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 874 | PH-3 | 第 7-9 周 | match | DB migration | 索引 + 约束 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 875 | PH-3 | 第 7-9 周 | match | DB migration | 双向迁移演练 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 876 | PH-3 | 第 7-9 周 | match | DB migration | 回滚预案 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 877 | PH-3 | 第 7-9 周 | match | UT 单元测试 | testkit helper | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 878 | PH-3 | 第 7-9 周 | match | UT 单元测试 | CRUD 覆盖 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 879 | PH-3 | 第 7-9 周 | match | UT 单元测试 | 状态机覆盖 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 880 | PH-3 | 第 7-9 周 | match | UT 单元测试 | 覆盖率报告 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 881 | PH-3 | 第 7-9 周 | match | IT 集成测试 | service 启动 + health | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 882 | PH-3 | 第 7-9 周 | match | IT 集成测试 | DB 集成 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 883 | PH-3 | 第 7-9 周 | match | IT 集成测试 | 跨组件契约 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 884 | PH-3 | 第 7-9 周 | match | IT 集成测试 | 端到端集成 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 885 | PH-3 | 第 7-9 周 | match | ST 系统测试 | K8s 部署验证 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 886 | PH-3 | 第 7-9 周 | match | ST 系统测试 | 性能 / 容量 NFR | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 887 | PH-3 | 第 7-9 周 | match | ST 系统测试 | chaos 故障注入 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 888 | PH-3 | 第 7-9 周 | match | ST 系统测试 | RPO/RTO 验证 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 889 | PH-3 | 第 7-9 周 | match | Helm chart | Chart.yaml | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 890 | PH-3 | 第 7-9 周 | match | Helm chart | values.yaml | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 891 | PH-3 | 第 7-9 周 | match | Helm chart | deployment + HPA | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 892 | PH-3 | 第 7-9 周 | match | Helm chart | NetworkPolicy | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 893 | PH-3 | 第 7-9 周 | match | observability | OTel spans | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 894 | PH-3 | 第 7-9 周 | match | observability | Prometheus metrics | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 895 | PH-3 | 第 7-9 周 | match | observability | Grafana 仪表盘 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 896 | PH-3 | 第 7-9 周 | match | observability | Loki 日志 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 897 | PH-3 | 第 7-9 周 | social | API Spec | 列出 gRPC 方法 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 898 | PH-3 | 第 7-9 周 | social | API Spec | 定义 Proto 文件 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 899 | PH-3 | 第 7-9 周 | social | API Spec | 配置 tonic-build | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 900 | PH-3 | 第 7-9 周 | social | API Spec | 编译期校验 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 901 | PH-3 | 第 7-9 周 | social | 业务逻辑 | 实体表定义 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 902 | PH-3 | 第 7-9 周 | social | 业务逻辑 | 状态机实现 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 903 | PH-3 | 第 7-9 周 | social | 业务逻辑 | 错误码 + 边界条件 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 904 | PH-3 | 第 7-9 周 | social | 业务逻辑 | 核心算法 / 决策 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 905 | PH-3 | 第 7-9 周 | social | DB migration | Schema 迁移 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 906 | PH-3 | 第 7-9 周 | social | DB migration | 索引 + 约束 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 907 | PH-3 | 第 7-9 周 | social | DB migration | 双向迁移演练 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 908 | PH-3 | 第 7-9 周 | social | DB migration | 回滚预案 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 909 | PH-3 | 第 7-9 周 | social | UT 单元测试 | testkit helper | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 910 | PH-3 | 第 7-9 周 | social | UT 单元测试 | CRUD 覆盖 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 911 | PH-3 | 第 7-9 周 | social | UT 单元测试 | 状态机覆盖 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 912 | PH-3 | 第 7-9 周 | social | UT 单元测试 | 覆盖率报告 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 913 | PH-3 | 第 7-9 周 | social | IT 集成测试 | service 启动 + health | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 914 | PH-3 | 第 7-9 周 | social | IT 集成测试 | DB 集成 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 915 | PH-3 | 第 7-9 周 | social | IT 集成测试 | 跨组件契约 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 916 | PH-3 | 第 7-9 周 | social | IT 集成测试 | 端到端集成 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 917 | PH-3 | 第 7-9 周 | social | ST 系统测试 | K8s 部署验证 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 918 | PH-3 | 第 7-9 周 | social | ST 系统测试 | 性能 / 容量 NFR | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 919 | PH-3 | 第 7-9 周 | social | ST 系统测试 | chaos 故障注入 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 920 | PH-3 | 第 7-9 周 | social | ST 系统测试 | RPO/RTO 验证 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 921 | PH-3 | 第 7-9 周 | social | Helm chart | Chart.yaml | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 922 | PH-3 | 第 7-9 周 | social | Helm chart | values.yaml | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 923 | PH-3 | 第 7-9 周 | social | Helm chart | deployment + HPA | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 924 | PH-3 | 第 7-9 周 | social | Helm chart | NetworkPolicy | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 925 | PH-3 | 第 7-9 周 | social | observability | OTel spans | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 926 | PH-3 | 第 7-9 周 | social | observability | Prometheus metrics | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 927 | PH-3 | 第 7-9 周 | social | observability | Grafana 仪表盘 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 928 | PH-3 | 第 7-9 周 | social | observability | Loki 日志 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 929 | PH-3 | 第 7-9 周 | admin | API Spec | 列出 gRPC 方法 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 930 | PH-3 | 第 7-9 周 | admin | API Spec | 定义 Proto 文件 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 931 | PH-3 | 第 7-9 周 | admin | API Spec | 配置 tonic-build | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 932 | PH-3 | 第 7-9 周 | admin | API Spec | 编译期校验 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 933 | PH-3 | 第 7-9 周 | admin | 业务逻辑 | 实体表定义 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 934 | PH-3 | 第 7-9 周 | admin | 业务逻辑 | 状态机实现 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 935 | PH-3 | 第 7-9 周 | admin | 业务逻辑 | 错误码 + 边界条件 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 936 | PH-3 | 第 7-9 周 | admin | 业务逻辑 | 核心算法 / 决策 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 937 | PH-3 | 第 7-9 周 | admin | DB migration | Schema 迁移 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 938 | PH-3 | 第 7-9 周 | admin | DB migration | 索引 + 约束 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 939 | PH-3 | 第 7-9 周 | admin | DB migration | 双向迁移演练 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 940 | PH-3 | 第 7-9 周 | admin | DB migration | 回滚预案 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 941 | PH-3 | 第 7-9 周 | admin | UT 单元测试 | testkit helper | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 942 | PH-3 | 第 7-9 周 | admin | UT 单元测试 | CRUD 覆盖 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 943 | PH-3 | 第 7-9 周 | admin | UT 单元测试 | 状态机覆盖 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 944 | PH-3 | 第 7-9 周 | admin | UT 单元测试 | 覆盖率报告 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 945 | PH-3 | 第 7-9 周 | admin | IT 集成测试 | service 启动 + health | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 946 | PH-3 | 第 7-9 周 | admin | IT 集成测试 | DB 集成 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 947 | PH-3 | 第 7-9 周 | admin | IT 集成测试 | 跨组件契约 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 948 | PH-3 | 第 7-9 周 | admin | IT 集成测试 | 端到端集成 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 949 | PH-3 | 第 7-9 周 | admin | ST 系统测试 | K8s 部署验证 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 950 | PH-3 | 第 7-9 周 | admin | ST 系统测试 | 性能 / 容量 NFR | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 951 | PH-3 | 第 7-9 周 | admin | ST 系统测试 | chaos 故障注入 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 952 | PH-3 | 第 7-9 周 | admin | ST 系统测试 | RPO/RTO 验证 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 953 | PH-3 | 第 7-9 周 | admin | Helm chart | Chart.yaml | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 954 | PH-3 | 第 7-9 周 | admin | Helm chart | values.yaml | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 955 | PH-3 | 第 7-9 周 | admin | Helm chart | deployment + HPA | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 956 | PH-3 | 第 7-9 周 | admin | Helm chart | NetworkPolicy | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 957 | PH-3 | 第 7-9 周 | admin | observability | OTel spans | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 958 | PH-3 | 第 7-9 周 | admin | observability | Prometheus metrics | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 959 | PH-3 | 第 7-9 周 | admin | observability | Grafana 仪表盘 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 960 | PH-3 | 第 7-9 周 | admin | observability | Loki 日志 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 961 | PH-3 | 第 7-9 周 | cluster-ops | Control Plane API | ClusterOps gRPC 定义 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 962 | PH-3 | 第 7-9 周 | cluster-ops | Control Plane API | AdminService 转发 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 963 | PH-3 | 第 7-9 周 | cluster-ops | Control Plane API | request_id 幂等 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 964 | PH-3 | 第 7-9 周 | cluster-ops | Control Plane API | OCC 版本字段 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 965 | PH-3 | 第 7-9 周 | cluster-ops | CEM | Feature registry | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 966 | PH-3 | 第 7-9 周 | cluster-ops | CEM | 事件流 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 967 | PH-3 | 第 7-9 周 | cluster-ops | CEM | 订阅/取消订阅 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 968 | PH-3 | 第 7-9 周 | cluster-ops | CEM | DLQ 处理 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 969 | PH-3 | 第 7-9 周 | cluster-ops | PFAU | declared → canary → confirm → done 状态机 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 970 | PH-3 | 第 7-9 周 | cluster-ops | PFAU | all-reachable 确认 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 971 | PH-3 | 第 7-9 周 | cluster-ops | PFAU | 灰度策略 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 972 | PH-3 | 第 7-9 周 | cluster-ops | PFAU | 回滚路径 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 973 | PH-3 | 第 7-9 周 | cluster-ops | 状态机 | feature 状态定义 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 974 | PH-3 | 第 7-9 周 | cluster-ops | 状态机 | 非法转移检测 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 975 | PH-3 | 第 7-9 周 | cluster-ops | 状态机 | 状态转移图 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 976 | PH-3 | 第 7-9 周 | cluster-ops | 状态机 | 持久化方案 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 977 | PH-3 | 第 7-9 周 | cluster-ops | RBAC | GM / COC / 客户端 3 套权限 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 978 | PH-3 | 第 7-9 周 | cluster-ops | RBAC | 权限矩阵 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 979 | PH-3 | 第 7-9 周 | cluster-ops | RBAC | 审计日志 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 980 | PH-3 | 第 7-9 周 | cluster-ops | RBAC | 撤销机制 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 981 | PH-3 | 第 7-9 周 | cluster-ops | fencing | 租约机制 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 982 | PH-3 | 第 7-9 周 | cluster-ops | fencing | CAS 版本控制 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 983 | PH-3 | 第 7-9 周 | cluster-ops | fencing | stale leader 检测 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 984 | PH-3 | 第 7-9 周 | cluster-ops | fencing | 集群隔离策略 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 985 | PH-3 | 第 7-9 周 | cluster-ops | 审计 | 审计 schema | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 986 | PH-3 | 第 7-9 周 | cluster-ops | 审计 | 写入路径 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 987 | PH-3 | 第 7-9 周 | cluster-ops | 审计 | 查询接口 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 988 | PH-3 | 第 7-9 周 | cluster-ops | 审计 | 保留期 + 归档 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 989 | PH-3 | 第 7-9 周 | cluster-ops | OCC | 乐观并发控制 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 990 | PH-3 | 第 7-9 周 | cluster-ops | OCC | 重试策略 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 991 | PH-3 | 第 7-9 周 | cluster-ops | OCC | 冲突检测 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 992 | PH-3 | 第 7-9 周 | cluster-ops | OCC | 死锁恢复 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 993 | PH-3 | 第 7-9 周 | shared-platform | Rust 工具链 | rustup 1.98 锁定 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 994 | PH-3 | 第 7-9 周 | shared-platform | Rust 工具链 | rust-toolchain.toml | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 995 | PH-3 | 第 7-9 周 | shared-platform | Rust 工具链 | CI cache | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 996 | PH-3 | 第 7-9 周 | shared-platform | Rust 工具链 | 升级评审 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 997 | PH-3 | 第 7-9 周 | shared-platform | Cargo.lock 锁定 | --locked 构建 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 998 | PH-3 | 第 7-9 周 | shared-platform | Cargo.lock 锁定 | workspace 统一锁 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 999 | PH-3 | 第 7-9 周 | shared-platform | Cargo.lock 锁定 | 依赖审计 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1000 | PH-3 | 第 7-9 周 | shared-platform | Cargo.lock 锁定 | 更新策略 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1001 | PH-3 | 第 7-9 周 | shared-platform | 镜像构建 | Dockerfile.distroless | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1002 | PH-3 | 第 7-9 周 | shared-platform | 镜像构建 | 镜像大小优化 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1003 | PH-3 | 第 7-9 周 | shared-platform | 镜像构建 | SBOM 生成 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1004 | PH-3 | 第 7-9 周 | shared-platform | 镜像构建 | 漏洞扫描 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1005 | PH-3 | 第 7-9 周 | shared-platform | K3s | K3s 集群初始化 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1006 | PH-3 | 第 7-9 周 | shared-platform | K3s | kubectl 配置 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1007 | PH-3 | 第 7-9 周 | shared-platform | K3s | NetworkPolicy | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1008 | PH-3 | 第 7-9 周 | shared-platform | K3s | HPA / VPA | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1009 | PH-3 | 第 7-9 周 | shared-platform | OTel Collector | 采集器配置 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1010 | PH-3 | 第 7-9 周 | shared-platform | OTel Collector | OTLP 接收 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1011 | PH-3 | 第 7-9 周 | shared-platform | OTel Collector | 采样策略 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1012 | PH-3 | 第 7-9 周 | shared-platform | OTel Collector | 导出器配置 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1013 | PH-3 | 第 7-9 周 | shared-platform | Helm | Chart 模板 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1014 | PH-3 | 第 7-9 周 | shared-platform | Helm | values 校验 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1015 | PH-3 | 第 7-9 周 | shared-platform | Helm | 依赖管理 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1016 | PH-3 | 第 7-9 周 | shared-platform | Helm | CI render | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1017 | PH-3 | 第 7-9 周 | shared-platform | 密钥 | Vault/OpenBao 部署 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1018 | PH-3 | 第 7-9 周 | shared-platform | 密钥 | 密钥轮换策略 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1019 | PH-3 | 第 7-9 周 | shared-platform | 密钥 | 运行时注入 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1020 | PH-3 | 第 7-9 周 | shared-platform | 密钥 | 审计 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1021 | PH-3 | 第 7-9 周 | shared-platform | 灾备 | 备份策略 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1022 | PH-3 | 第 7-9 周 | shared-platform | 灾备 | 恢复演练 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1023 | PH-3 | 第 7-9 周 | shared-platform | 灾备 | 跨 AZ 复制 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1024 | PH-3 | 第 7-9 周 | shared-platform | 灾备 | RTO/RPO 验证 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1025 | PH-4 | 第 9-12 周 | foundation | workspace 骨架 | virtual workspace 配置 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1026 | PH-4 | 第 9-12 周 | foundation | workspace 骨架 | resolver=3 锁定 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1027 | PH-4 | 第 9-12 周 | foundation | workspace 骨架 | Edition 2024 升级 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1028 | PH-4 | 第 9-12 周 | foundation | workspace 骨架 | crate 间依赖方向 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1029 | PH-4 | 第 9-12 周 | foundation | testkit 共用 | testcontainers PG 封装 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1030 | PH-4 | 第 9-12 周 | foundation | testkit 共用 | mock helpers | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1031 | PH-4 | 第 9-12 周 | foundation | testkit 共用 | fixture builders | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1032 | PH-4 | 第 9-12 周 | foundation | testkit 共用 | coverage 报告 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1033 | PH-4 | 第 9-12 周 | foundation | CI 工具链 | GitHub Actions 配置 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1034 | PH-4 | 第 9-12 周 | foundation | CI 工具链 | cargo fmt/clippy/deny | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1035 | PH-4 | 第 9-12 周 | foundation | CI 工具链 | sqlx prepare check | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1036 | PH-4 | 第 9-12 周 | foundation | CI 工具链 | manifest 校验 CI | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1037 | PH-4 | 第 9-12 周 | foundation | DAG validator | 拓扑排序算法 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1038 | PH-4 | 第 9-12 周 | foundation | DAG validator | 环依赖检测 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1039 | PH-4 | 第 9-12 周 | foundation | DAG validator | 缺祖先检测 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1040 | PH-4 | 第 9-12 周 | foundation | DAG validator | 负例测试套件 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1041 | PH-4 | 第 9-12 周 | foundation | cargo-deny/audit | 许可证白名单 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1042 | PH-4 | 第 9-12 周 | foundation | cargo-deny/audit | 漏洞数据库 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1043 | PH-4 | 第 9-12 周 | foundation | cargo-deny/audit | 依赖来源限制 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1044 | PH-4 | 第 9-12 周 | foundation | cargo-deny/audit | CI 集成 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1045 | PH-4 | 第 9-12 周 | foundation | manifest schema | JSON Schema 起草 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1046 | PH-4 | 第 9-12 周 | foundation | manifest schema | ARC-042 字段映射 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1047 | PH-4 | 第 9-12 周 | foundation | manifest schema | schema 校验 CLI | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1048 | PH-4 | 第 9-12 周 | foundation | manifest schema | 示例 manifest | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1049 | PH-4 | 第 9-12 周 | foundation | 文档生成 | mdbook 配置 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1050 | PH-4 | 第 9-12 周 | foundation | 文档生成 | Doxygen/Rustdoc | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1051 | PH-4 | 第 9-12 周 | foundation | 文档生成 | CR 链接检查 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1052 | PH-4 | 第 9-12 周 | foundation | 文档生成 | CI 文档门禁 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1053 | PH-4 | 第 9-12 周 | foundation | 工程约定 | 错误码定义 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1054 | PH-4 | 第 9-12 周 | foundation | 工程约定 | 序列化约定 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1055 | PH-4 | 第 9-12 周 | foundation | 工程约定 | 日志规范 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1056 | PH-4 | 第 9-12 周 | foundation | 工程约定 | metrics 命名 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1057 | PH-4 | 第 9-12 周 | player | API Spec | 列出 gRPC 方法 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1058 | PH-4 | 第 9-12 周 | player | API Spec | 定义 Proto 文件 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1059 | PH-4 | 第 9-12 周 | player | API Spec | 配置 tonic-build | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1060 | PH-4 | 第 9-12 周 | player | API Spec | 编译期校验 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1061 | PH-4 | 第 9-12 周 | player | 业务逻辑 | 实体表定义 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1062 | PH-4 | 第 9-12 周 | player | 业务逻辑 | 状态机实现 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1063 | PH-4 | 第 9-12 周 | player | 业务逻辑 | 错误码 + 边界条件 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1064 | PH-4 | 第 9-12 周 | player | 业务逻辑 | 核心算法 / 决策 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1065 | PH-4 | 第 9-12 周 | player | DB migration | Schema 迁移 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1066 | PH-4 | 第 9-12 周 | player | DB migration | 索引 + 约束 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1067 | PH-4 | 第 9-12 周 | player | DB migration | 双向迁移演练 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1068 | PH-4 | 第 9-12 周 | player | DB migration | 回滚预案 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1069 | PH-4 | 第 9-12 周 | player | UT 单元测试 | testkit helper | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1070 | PH-4 | 第 9-12 周 | player | UT 单元测试 | CRUD 覆盖 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1071 | PH-4 | 第 9-12 周 | player | UT 单元测试 | 状态机覆盖 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1072 | PH-4 | 第 9-12 周 | player | UT 单元测试 | 覆盖率报告 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1073 | PH-4 | 第 9-12 周 | player | IT 集成测试 | service 启动 + health | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1074 | PH-4 | 第 9-12 周 | player | IT 集成测试 | DB 集成 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1075 | PH-4 | 第 9-12 周 | player | IT 集成测试 | 跨组件契约 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1076 | PH-4 | 第 9-12 周 | player | IT 集成测试 | 端到端集成 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1077 | PH-4 | 第 9-12 周 | player | ST 系统测试 | K8s 部署验证 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1078 | PH-4 | 第 9-12 周 | player | ST 系统测试 | 性能 / 容量 NFR | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1079 | PH-4 | 第 9-12 周 | player | ST 系统测试 | chaos 故障注入 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1080 | PH-4 | 第 9-12 周 | player | ST 系统测试 | RPO/RTO 验证 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1081 | PH-4 | 第 9-12 周 | player | Helm chart | Chart.yaml | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1082 | PH-4 | 第 9-12 周 | player | Helm chart | values.yaml | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1083 | PH-4 | 第 9-12 周 | player | Helm chart | deployment + HPA | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1084 | PH-4 | 第 9-12 周 | player | Helm chart | NetworkPolicy | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1085 | PH-4 | 第 9-12 周 | player | observability | OTel spans | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1086 | PH-4 | 第 9-12 周 | player | observability | Prometheus metrics | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1087 | PH-4 | 第 9-12 周 | player | observability | Grafana 仪表盘 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1088 | PH-4 | 第 9-12 周 | player | observability | Loki 日志 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1089 | PH-4 | 第 9-12 周 | economy | API Spec | 列出 gRPC 方法 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1090 | PH-4 | 第 9-12 周 | economy | API Spec | 定义 Proto 文件 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1091 | PH-4 | 第 9-12 周 | economy | API Spec | 配置 tonic-build | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1092 | PH-4 | 第 9-12 周 | economy | API Spec | 编译期校验 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1093 | PH-4 | 第 9-12 周 | economy | 业务逻辑 | 实体表定义 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1094 | PH-4 | 第 9-12 周 | economy | 业务逻辑 | 状态机实现 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1095 | PH-4 | 第 9-12 周 | economy | 业务逻辑 | 错误码 + 边界条件 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1096 | PH-4 | 第 9-12 周 | economy | 业务逻辑 | 核心算法 / 决策 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1097 | PH-4 | 第 9-12 周 | economy | DB migration | Schema 迁移 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1098 | PH-4 | 第 9-12 周 | economy | DB migration | 索引 + 约束 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1099 | PH-4 | 第 9-12 周 | economy | DB migration | 双向迁移演练 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1100 | PH-4 | 第 9-12 周 | economy | DB migration | 回滚预案 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1101 | PH-4 | 第 9-12 周 | economy | UT 单元测试 | testkit helper | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1102 | PH-4 | 第 9-12 周 | economy | UT 单元测试 | CRUD 覆盖 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1103 | PH-4 | 第 9-12 周 | economy | UT 单元测试 | 状态机覆盖 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1104 | PH-4 | 第 9-12 周 | economy | UT 单元测试 | 覆盖率报告 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1105 | PH-4 | 第 9-12 周 | economy | IT 集成测试 | service 启动 + health | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1106 | PH-4 | 第 9-12 周 | economy | IT 集成测试 | DB 集成 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1107 | PH-4 | 第 9-12 周 | economy | IT 集成测试 | 跨组件契约 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1108 | PH-4 | 第 9-12 周 | economy | IT 集成测试 | 端到端集成 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1109 | PH-4 | 第 9-12 周 | economy | ST 系统测试 | K8s 部署验证 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1110 | PH-4 | 第 9-12 周 | economy | ST 系统测试 | 性能 / 容量 NFR | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1111 | PH-4 | 第 9-12 周 | economy | ST 系统测试 | chaos 故障注入 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1112 | PH-4 | 第 9-12 周 | economy | ST 系统测试 | RPO/RTO 验证 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1113 | PH-4 | 第 9-12 周 | economy | Helm chart | Chart.yaml | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1114 | PH-4 | 第 9-12 周 | economy | Helm chart | values.yaml | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1115 | PH-4 | 第 9-12 周 | economy | Helm chart | deployment + HPA | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1116 | PH-4 | 第 9-12 周 | economy | Helm chart | NetworkPolicy | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1117 | PH-4 | 第 9-12 周 | economy | observability | OTel spans | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1118 | PH-4 | 第 9-12 周 | economy | observability | Prometheus metrics | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1119 | PH-4 | 第 9-12 周 | economy | observability | Grafana 仪表盘 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1120 | PH-4 | 第 9-12 周 | economy | observability | Loki 日志 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1121 | PH-4 | 第 9-12 周 | match | API Spec | 列出 gRPC 方法 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1122 | PH-4 | 第 9-12 周 | match | API Spec | 定义 Proto 文件 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1123 | PH-4 | 第 9-12 周 | match | API Spec | 配置 tonic-build | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1124 | PH-4 | 第 9-12 周 | match | API Spec | 编译期校验 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1125 | PH-4 | 第 9-12 周 | match | 业务逻辑 | 实体表定义 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1126 | PH-4 | 第 9-12 周 | match | 业务逻辑 | 状态机实现 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1127 | PH-4 | 第 9-12 周 | match | 业务逻辑 | 错误码 + 边界条件 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1128 | PH-4 | 第 9-12 周 | match | 业务逻辑 | 核心算法 / 决策 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1129 | PH-4 | 第 9-12 周 | match | DB migration | Schema 迁移 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1130 | PH-4 | 第 9-12 周 | match | DB migration | 索引 + 约束 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1131 | PH-4 | 第 9-12 周 | match | DB migration | 双向迁移演练 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1132 | PH-4 | 第 9-12 周 | match | DB migration | 回滚预案 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1133 | PH-4 | 第 9-12 周 | match | UT 单元测试 | testkit helper | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1134 | PH-4 | 第 9-12 周 | match | UT 单元测试 | CRUD 覆盖 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1135 | PH-4 | 第 9-12 周 | match | UT 单元测试 | 状态机覆盖 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1136 | PH-4 | 第 9-12 周 | match | UT 单元测试 | 覆盖率报告 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1137 | PH-4 | 第 9-12 周 | match | IT 集成测试 | service 启动 + health | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1138 | PH-4 | 第 9-12 周 | match | IT 集成测试 | DB 集成 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1139 | PH-4 | 第 9-12 周 | match | IT 集成测试 | 跨组件契约 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1140 | PH-4 | 第 9-12 周 | match | IT 集成测试 | 端到端集成 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1141 | PH-4 | 第 9-12 周 | match | ST 系统测试 | K8s 部署验证 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1142 | PH-4 | 第 9-12 周 | match | ST 系统测试 | 性能 / 容量 NFR | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1143 | PH-4 | 第 9-12 周 | match | ST 系统测试 | chaos 故障注入 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1144 | PH-4 | 第 9-12 周 | match | ST 系统测试 | RPO/RTO 验证 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1145 | PH-4 | 第 9-12 周 | match | Helm chart | Chart.yaml | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1146 | PH-4 | 第 9-12 周 | match | Helm chart | values.yaml | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1147 | PH-4 | 第 9-12 周 | match | Helm chart | deployment + HPA | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1148 | PH-4 | 第 9-12 周 | match | Helm chart | NetworkPolicy | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1149 | PH-4 | 第 9-12 周 | match | observability | OTel spans | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1150 | PH-4 | 第 9-12 周 | match | observability | Prometheus metrics | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1151 | PH-4 | 第 9-12 周 | match | observability | Grafana 仪表盘 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1152 | PH-4 | 第 9-12 周 | match | observability | Loki 日志 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1153 | PH-4 | 第 9-12 周 | social | API Spec | 列出 gRPC 方法 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1154 | PH-4 | 第 9-12 周 | social | API Spec | 定义 Proto 文件 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1155 | PH-4 | 第 9-12 周 | social | API Spec | 配置 tonic-build | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1156 | PH-4 | 第 9-12 周 | social | API Spec | 编译期校验 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1157 | PH-4 | 第 9-12 周 | social | 业务逻辑 | 实体表定义 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1158 | PH-4 | 第 9-12 周 | social | 业务逻辑 | 状态机实现 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1159 | PH-4 | 第 9-12 周 | social | 业务逻辑 | 错误码 + 边界条件 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1160 | PH-4 | 第 9-12 周 | social | 业务逻辑 | 核心算法 / 决策 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1161 | PH-4 | 第 9-12 周 | social | DB migration | Schema 迁移 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1162 | PH-4 | 第 9-12 周 | social | DB migration | 索引 + 约束 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1163 | PH-4 | 第 9-12 周 | social | DB migration | 双向迁移演练 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1164 | PH-4 | 第 9-12 周 | social | DB migration | 回滚预案 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1165 | PH-4 | 第 9-12 周 | social | UT 单元测试 | testkit helper | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1166 | PH-4 | 第 9-12 周 | social | UT 单元测试 | CRUD 覆盖 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1167 | PH-4 | 第 9-12 周 | social | UT 单元测试 | 状态机覆盖 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1168 | PH-4 | 第 9-12 周 | social | UT 单元测试 | 覆盖率报告 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1169 | PH-4 | 第 9-12 周 | social | IT 集成测试 | service 启动 + health | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1170 | PH-4 | 第 9-12 周 | social | IT 集成测试 | DB 集成 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1171 | PH-4 | 第 9-12 周 | social | IT 集成测试 | 跨组件契约 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1172 | PH-4 | 第 9-12 周 | social | IT 集成测试 | 端到端集成 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1173 | PH-4 | 第 9-12 周 | social | ST 系统测试 | K8s 部署验证 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1174 | PH-4 | 第 9-12 周 | social | ST 系统测试 | 性能 / 容量 NFR | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1175 | PH-4 | 第 9-12 周 | social | ST 系统测试 | chaos 故障注入 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1176 | PH-4 | 第 9-12 周 | social | ST 系统测试 | RPO/RTO 验证 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1177 | PH-4 | 第 9-12 周 | social | Helm chart | Chart.yaml | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1178 | PH-4 | 第 9-12 周 | social | Helm chart | values.yaml | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1179 | PH-4 | 第 9-12 周 | social | Helm chart | deployment + HPA | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1180 | PH-4 | 第 9-12 周 | social | Helm chart | NetworkPolicy | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1181 | PH-4 | 第 9-12 周 | social | observability | OTel spans | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1182 | PH-4 | 第 9-12 周 | social | observability | Prometheus metrics | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1183 | PH-4 | 第 9-12 周 | social | observability | Grafana 仪表盘 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1184 | PH-4 | 第 9-12 周 | social | observability | Loki 日志 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1185 | PH-4 | 第 9-12 周 | admin | API Spec | 列出 gRPC 方法 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1186 | PH-4 | 第 9-12 周 | admin | API Spec | 定义 Proto 文件 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1187 | PH-4 | 第 9-12 周 | admin | API Spec | 配置 tonic-build | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1188 | PH-4 | 第 9-12 周 | admin | API Spec | 编译期校验 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1189 | PH-4 | 第 9-12 周 | admin | 业务逻辑 | 实体表定义 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1190 | PH-4 | 第 9-12 周 | admin | 业务逻辑 | 状态机实现 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1191 | PH-4 | 第 9-12 周 | admin | 业务逻辑 | 错误码 + 边界条件 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1192 | PH-4 | 第 9-12 周 | admin | 业务逻辑 | 核心算法 / 决策 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1193 | PH-4 | 第 9-12 周 | admin | DB migration | Schema 迁移 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1194 | PH-4 | 第 9-12 周 | admin | DB migration | 索引 + 约束 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1195 | PH-4 | 第 9-12 周 | admin | DB migration | 双向迁移演练 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1196 | PH-4 | 第 9-12 周 | admin | DB migration | 回滚预案 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1197 | PH-4 | 第 9-12 周 | admin | UT 单元测试 | testkit helper | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1198 | PH-4 | 第 9-12 周 | admin | UT 单元测试 | CRUD 覆盖 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1199 | PH-4 | 第 9-12 周 | admin | UT 单元测试 | 状态机覆盖 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1200 | PH-4 | 第 9-12 周 | admin | UT 单元测试 | 覆盖率报告 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1201 | PH-4 | 第 9-12 周 | admin | IT 集成测试 | service 启动 + health | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1202 | PH-4 | 第 9-12 周 | admin | IT 集成测试 | DB 集成 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1203 | PH-4 | 第 9-12 周 | admin | IT 集成测试 | 跨组件契约 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1204 | PH-4 | 第 9-12 周 | admin | IT 集成测试 | 端到端集成 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1205 | PH-4 | 第 9-12 周 | admin | ST 系统测试 | K8s 部署验证 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1206 | PH-4 | 第 9-12 周 | admin | ST 系统测试 | 性能 / 容量 NFR | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1207 | PH-4 | 第 9-12 周 | admin | ST 系统测试 | chaos 故障注入 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1208 | PH-4 | 第 9-12 周 | admin | ST 系统测试 | RPO/RTO 验证 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1209 | PH-4 | 第 9-12 周 | admin | Helm chart | Chart.yaml | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1210 | PH-4 | 第 9-12 周 | admin | Helm chart | values.yaml | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1211 | PH-4 | 第 9-12 周 | admin | Helm chart | deployment + HPA | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1212 | PH-4 | 第 9-12 周 | admin | Helm chart | NetworkPolicy | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1213 | PH-4 | 第 9-12 周 | admin | observability | OTel spans | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1214 | PH-4 | 第 9-12 周 | admin | observability | Prometheus metrics | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1215 | PH-4 | 第 9-12 周 | admin | observability | Grafana 仪表盘 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1216 | PH-4 | 第 9-12 周 | admin | observability | Loki 日志 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1217 | PH-4 | 第 9-12 周 | cluster-ops | Control Plane API | ClusterOps gRPC 定义 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1218 | PH-4 | 第 9-12 周 | cluster-ops | Control Plane API | AdminService 转发 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1219 | PH-4 | 第 9-12 周 | cluster-ops | Control Plane API | request_id 幂等 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1220 | PH-4 | 第 9-12 周 | cluster-ops | Control Plane API | OCC 版本字段 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1221 | PH-4 | 第 9-12 周 | cluster-ops | CEM | Feature registry | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1222 | PH-4 | 第 9-12 周 | cluster-ops | CEM | 事件流 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1223 | PH-4 | 第 9-12 周 | cluster-ops | CEM | 订阅/取消订阅 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1224 | PH-4 | 第 9-12 周 | cluster-ops | CEM | DLQ 处理 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1225 | PH-4 | 第 9-12 周 | cluster-ops | PFAU | declared → canary → confirm → done 状态机 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1226 | PH-4 | 第 9-12 周 | cluster-ops | PFAU | all-reachable 确认 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1227 | PH-4 | 第 9-12 周 | cluster-ops | PFAU | 灰度策略 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1228 | PH-4 | 第 9-12 周 | cluster-ops | PFAU | 回滚路径 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1229 | PH-4 | 第 9-12 周 | cluster-ops | 状态机 | feature 状态定义 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1230 | PH-4 | 第 9-12 周 | cluster-ops | 状态机 | 非法转移检测 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1231 | PH-4 | 第 9-12 周 | cluster-ops | 状态机 | 状态转移图 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1232 | PH-4 | 第 9-12 周 | cluster-ops | 状态机 | 持久化方案 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1233 | PH-4 | 第 9-12 周 | cluster-ops | RBAC | GM / COC / 客户端 3 套权限 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1234 | PH-4 | 第 9-12 周 | cluster-ops | RBAC | 权限矩阵 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1235 | PH-4 | 第 9-12 周 | cluster-ops | RBAC | 审计日志 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1236 | PH-4 | 第 9-12 周 | cluster-ops | RBAC | 撤销机制 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1237 | PH-4 | 第 9-12 周 | cluster-ops | fencing | 租约机制 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1238 | PH-4 | 第 9-12 周 | cluster-ops | fencing | CAS 版本控制 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1239 | PH-4 | 第 9-12 周 | cluster-ops | fencing | stale leader 检测 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1240 | PH-4 | 第 9-12 周 | cluster-ops | fencing | 集群隔离策略 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1241 | PH-4 | 第 9-12 周 | cluster-ops | 审计 | 审计 schema | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1242 | PH-4 | 第 9-12 周 | cluster-ops | 审计 | 写入路径 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1243 | PH-4 | 第 9-12 周 | cluster-ops | 审计 | 查询接口 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1244 | PH-4 | 第 9-12 周 | cluster-ops | 审计 | 保留期 + 归档 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1245 | PH-4 | 第 9-12 周 | cluster-ops | OCC | 乐观并发控制 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1246 | PH-4 | 第 9-12 周 | cluster-ops | OCC | 重试策略 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1247 | PH-4 | 第 9-12 周 | cluster-ops | OCC | 冲突检测 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1248 | PH-4 | 第 9-12 周 | cluster-ops | OCC | 死锁恢复 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1249 | PH-4 | 第 9-12 周 | shared-platform | Rust 工具链 | rustup 1.98 锁定 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1250 | PH-4 | 第 9-12 周 | shared-platform | Rust 工具链 | rust-toolchain.toml | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1251 | PH-4 | 第 9-12 周 | shared-platform | Rust 工具链 | CI cache | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1252 | PH-4 | 第 9-12 周 | shared-platform | Rust 工具链 | 升级评审 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1253 | PH-4 | 第 9-12 周 | shared-platform | Cargo.lock 锁定 | --locked 构建 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1254 | PH-4 | 第 9-12 周 | shared-platform | Cargo.lock 锁定 | workspace 统一锁 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1255 | PH-4 | 第 9-12 周 | shared-platform | Cargo.lock 锁定 | 依赖审计 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1256 | PH-4 | 第 9-12 周 | shared-platform | Cargo.lock 锁定 | 更新策略 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1257 | PH-4 | 第 9-12 周 | shared-platform | 镜像构建 | Dockerfile.distroless | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1258 | PH-4 | 第 9-12 周 | shared-platform | 镜像构建 | 镜像大小优化 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1259 | PH-4 | 第 9-12 周 | shared-platform | 镜像构建 | SBOM 生成 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1260 | PH-4 | 第 9-12 周 | shared-platform | 镜像构建 | 漏洞扫描 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1261 | PH-4 | 第 9-12 周 | shared-platform | K3s | K3s 集群初始化 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1262 | PH-4 | 第 9-12 周 | shared-platform | K3s | kubectl 配置 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1263 | PH-4 | 第 9-12 周 | shared-platform | K3s | NetworkPolicy | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1264 | PH-4 | 第 9-12 周 | shared-platform | K3s | HPA / VPA | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1265 | PH-4 | 第 9-12 周 | shared-platform | OTel Collector | 采集器配置 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1266 | PH-4 | 第 9-12 周 | shared-platform | OTel Collector | OTLP 接收 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1267 | PH-4 | 第 9-12 周 | shared-platform | OTel Collector | 采样策略 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1268 | PH-4 | 第 9-12 周 | shared-platform | OTel Collector | 导出器配置 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1269 | PH-4 | 第 9-12 周 | shared-platform | Helm | Chart 模板 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1270 | PH-4 | 第 9-12 周 | shared-platform | Helm | values 校验 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1271 | PH-4 | 第 9-12 周 | shared-platform | Helm | 依赖管理 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1272 | PH-4 | 第 9-12 周 | shared-platform | Helm | CI render | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1273 | PH-4 | 第 9-12 周 | shared-platform | 密钥 | Vault/OpenBao 部署 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1274 | PH-4 | 第 9-12 周 | shared-platform | 密钥 | 密钥轮换策略 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1275 | PH-4 | 第 9-12 周 | shared-platform | 密钥 | 运行时注入 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1276 | PH-4 | 第 9-12 周 | shared-platform | 密钥 | 审计 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1277 | PH-4 | 第 9-12 周 | shared-platform | 灾备 | 备份策略 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1278 | PH-4 | 第 9-12 周 | shared-platform | 灾备 | 恢复演练 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1279 | PH-4 | 第 9-12 周 | shared-platform | 灾备 | 跨 AZ 复制 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1280 | PH-4 | 第 9-12 周 | shared-platform | 灾备 | RTO/RPO 验证 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1281 | PH-5 | 第 12-14 周 | foundation | workspace 骨架 | virtual workspace 配置 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1282 | PH-5 | 第 12-14 周 | foundation | workspace 骨架 | resolver=3 锁定 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1283 | PH-5 | 第 12-14 周 | foundation | workspace 骨架 | Edition 2024 升级 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1284 | PH-5 | 第 12-14 周 | foundation | workspace 骨架 | crate 间依赖方向 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1285 | PH-5 | 第 12-14 周 | foundation | testkit 共用 | testcontainers PG 封装 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1286 | PH-5 | 第 12-14 周 | foundation | testkit 共用 | mock helpers | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1287 | PH-5 | 第 12-14 周 | foundation | testkit 共用 | fixture builders | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1288 | PH-5 | 第 12-14 周 | foundation | testkit 共用 | coverage 报告 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1289 | PH-5 | 第 12-14 周 | foundation | CI 工具链 | GitHub Actions 配置 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1290 | PH-5 | 第 12-14 周 | foundation | CI 工具链 | cargo fmt/clippy/deny | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1291 | PH-5 | 第 12-14 周 | foundation | CI 工具链 | sqlx prepare check | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1292 | PH-5 | 第 12-14 周 | foundation | CI 工具链 | manifest 校验 CI | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1293 | PH-5 | 第 12-14 周 | foundation | DAG validator | 拓扑排序算法 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1294 | PH-5 | 第 12-14 周 | foundation | DAG validator | 环依赖检测 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1295 | PH-5 | 第 12-14 周 | foundation | DAG validator | 缺祖先检测 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1296 | PH-5 | 第 12-14 周 | foundation | DAG validator | 负例测试套件 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1297 | PH-5 | 第 12-14 周 | foundation | cargo-deny/audit | 许可证白名单 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1298 | PH-5 | 第 12-14 周 | foundation | cargo-deny/audit | 漏洞数据库 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1299 | PH-5 | 第 12-14 周 | foundation | cargo-deny/audit | 依赖来源限制 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1300 | PH-5 | 第 12-14 周 | foundation | cargo-deny/audit | CI 集成 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1301 | PH-5 | 第 12-14 周 | foundation | manifest schema | JSON Schema 起草 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1302 | PH-5 | 第 12-14 周 | foundation | manifest schema | ARC-042 字段映射 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1303 | PH-5 | 第 12-14 周 | foundation | manifest schema | schema 校验 CLI | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1304 | PH-5 | 第 12-14 周 | foundation | manifest schema | 示例 manifest | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1305 | PH-5 | 第 12-14 周 | foundation | 文档生成 | mdbook 配置 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1306 | PH-5 | 第 12-14 周 | foundation | 文档生成 | Doxygen/Rustdoc | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1307 | PH-5 | 第 12-14 周 | foundation | 文档生成 | CR 链接检查 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1308 | PH-5 | 第 12-14 周 | foundation | 文档生成 | CI 文档门禁 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1309 | PH-5 | 第 12-14 周 | foundation | 工程约定 | 错误码定义 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1310 | PH-5 | 第 12-14 周 | foundation | 工程约定 | 序列化约定 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1311 | PH-5 | 第 12-14 周 | foundation | 工程约定 | 日志规范 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1312 | PH-5 | 第 12-14 周 | foundation | 工程约定 | metrics 命名 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1313 | PH-5 | 第 12-14 周 | player | API Spec | 列出 gRPC 方法 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1314 | PH-5 | 第 12-14 周 | player | API Spec | 定义 Proto 文件 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1315 | PH-5 | 第 12-14 周 | player | API Spec | 配置 tonic-build | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1316 | PH-5 | 第 12-14 周 | player | API Spec | 编译期校验 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1317 | PH-5 | 第 12-14 周 | player | 业务逻辑 | 实体表定义 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1318 | PH-5 | 第 12-14 周 | player | 业务逻辑 | 状态机实现 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1319 | PH-5 | 第 12-14 周 | player | 业务逻辑 | 错误码 + 边界条件 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1320 | PH-5 | 第 12-14 周 | player | 业务逻辑 | 核心算法 / 决策 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1321 | PH-5 | 第 12-14 周 | player | DB migration | Schema 迁移 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1322 | PH-5 | 第 12-14 周 | player | DB migration | 索引 + 约束 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1323 | PH-5 | 第 12-14 周 | player | DB migration | 双向迁移演练 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1324 | PH-5 | 第 12-14 周 | player | DB migration | 回滚预案 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1325 | PH-5 | 第 12-14 周 | player | UT 单元测试 | testkit helper | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1326 | PH-5 | 第 12-14 周 | player | UT 单元测试 | CRUD 覆盖 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1327 | PH-5 | 第 12-14 周 | player | UT 单元测试 | 状态机覆盖 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1328 | PH-5 | 第 12-14 周 | player | UT 单元测试 | 覆盖率报告 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1329 | PH-5 | 第 12-14 周 | player | IT 集成测试 | service 启动 + health | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1330 | PH-5 | 第 12-14 周 | player | IT 集成测试 | DB 集成 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1331 | PH-5 | 第 12-14 周 | player | IT 集成测试 | 跨组件契约 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1332 | PH-5 | 第 12-14 周 | player | IT 集成测试 | 端到端集成 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1333 | PH-5 | 第 12-14 周 | player | ST 系统测试 | K8s 部署验证 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1334 | PH-5 | 第 12-14 周 | player | ST 系统测试 | 性能 / 容量 NFR | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1335 | PH-5 | 第 12-14 周 | player | ST 系统测试 | chaos 故障注入 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1336 | PH-5 | 第 12-14 周 | player | ST 系统测试 | RPO/RTO 验证 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1337 | PH-5 | 第 12-14 周 | player | Helm chart | Chart.yaml | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1338 | PH-5 | 第 12-14 周 | player | Helm chart | values.yaml | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1339 | PH-5 | 第 12-14 周 | player | Helm chart | deployment + HPA | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1340 | PH-5 | 第 12-14 周 | player | Helm chart | NetworkPolicy | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1341 | PH-5 | 第 12-14 周 | player | observability | OTel spans | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1342 | PH-5 | 第 12-14 周 | player | observability | Prometheus metrics | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1343 | PH-5 | 第 12-14 周 | player | observability | Grafana 仪表盘 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1344 | PH-5 | 第 12-14 周 | player | observability | Loki 日志 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1345 | PH-5 | 第 12-14 周 | economy | API Spec | 列出 gRPC 方法 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1346 | PH-5 | 第 12-14 周 | economy | API Spec | 定义 Proto 文件 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1347 | PH-5 | 第 12-14 周 | economy | API Spec | 配置 tonic-build | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1348 | PH-5 | 第 12-14 周 | economy | API Spec | 编译期校验 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1349 | PH-5 | 第 12-14 周 | economy | 业务逻辑 | 实体表定义 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1350 | PH-5 | 第 12-14 周 | economy | 业务逻辑 | 状态机实现 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1351 | PH-5 | 第 12-14 周 | economy | 业务逻辑 | 错误码 + 边界条件 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1352 | PH-5 | 第 12-14 周 | economy | 业务逻辑 | 核心算法 / 决策 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1353 | PH-5 | 第 12-14 周 | economy | DB migration | Schema 迁移 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1354 | PH-5 | 第 12-14 周 | economy | DB migration | 索引 + 约束 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1355 | PH-5 | 第 12-14 周 | economy | DB migration | 双向迁移演练 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1356 | PH-5 | 第 12-14 周 | economy | DB migration | 回滚预案 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1357 | PH-5 | 第 12-14 周 | economy | UT 单元测试 | testkit helper | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1358 | PH-5 | 第 12-14 周 | economy | UT 单元测试 | CRUD 覆盖 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1359 | PH-5 | 第 12-14 周 | economy | UT 单元测试 | 状态机覆盖 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1360 | PH-5 | 第 12-14 周 | economy | UT 单元测试 | 覆盖率报告 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1361 | PH-5 | 第 12-14 周 | economy | IT 集成测试 | service 启动 + health | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1362 | PH-5 | 第 12-14 周 | economy | IT 集成测试 | DB 集成 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1363 | PH-5 | 第 12-14 周 | economy | IT 集成测试 | 跨组件契约 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1364 | PH-5 | 第 12-14 周 | economy | IT 集成测试 | 端到端集成 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1365 | PH-5 | 第 12-14 周 | economy | ST 系统测试 | K8s 部署验证 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1366 | PH-5 | 第 12-14 周 | economy | ST 系统测试 | 性能 / 容量 NFR | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1367 | PH-5 | 第 12-14 周 | economy | ST 系统测试 | chaos 故障注入 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1368 | PH-5 | 第 12-14 周 | economy | ST 系统测试 | RPO/RTO 验证 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1369 | PH-5 | 第 12-14 周 | economy | Helm chart | Chart.yaml | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1370 | PH-5 | 第 12-14 周 | economy | Helm chart | values.yaml | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1371 | PH-5 | 第 12-14 周 | economy | Helm chart | deployment + HPA | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1372 | PH-5 | 第 12-14 周 | economy | Helm chart | NetworkPolicy | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1373 | PH-5 | 第 12-14 周 | economy | observability | OTel spans | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1374 | PH-5 | 第 12-14 周 | economy | observability | Prometheus metrics | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1375 | PH-5 | 第 12-14 周 | economy | observability | Grafana 仪表盘 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1376 | PH-5 | 第 12-14 周 | economy | observability | Loki 日志 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1377 | PH-5 | 第 12-14 周 | match | API Spec | 列出 gRPC 方法 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1378 | PH-5 | 第 12-14 周 | match | API Spec | 定义 Proto 文件 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1379 | PH-5 | 第 12-14 周 | match | API Spec | 配置 tonic-build | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1380 | PH-5 | 第 12-14 周 | match | API Spec | 编译期校验 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1381 | PH-5 | 第 12-14 周 | match | 业务逻辑 | 实体表定义 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1382 | PH-5 | 第 12-14 周 | match | 业务逻辑 | 状态机实现 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1383 | PH-5 | 第 12-14 周 | match | 业务逻辑 | 错误码 + 边界条件 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1384 | PH-5 | 第 12-14 周 | match | 业务逻辑 | 核心算法 / 决策 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1385 | PH-5 | 第 12-14 周 | match | DB migration | Schema 迁移 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1386 | PH-5 | 第 12-14 周 | match | DB migration | 索引 + 约束 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1387 | PH-5 | 第 12-14 周 | match | DB migration | 双向迁移演练 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1388 | PH-5 | 第 12-14 周 | match | DB migration | 回滚预案 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1389 | PH-5 | 第 12-14 周 | match | UT 单元测试 | testkit helper | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1390 | PH-5 | 第 12-14 周 | match | UT 单元测试 | CRUD 覆盖 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1391 | PH-5 | 第 12-14 周 | match | UT 单元测试 | 状态机覆盖 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1392 | PH-5 | 第 12-14 周 | match | UT 单元测试 | 覆盖率报告 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1393 | PH-5 | 第 12-14 周 | match | IT 集成测试 | service 启动 + health | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1394 | PH-5 | 第 12-14 周 | match | IT 集成测试 | DB 集成 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1395 | PH-5 | 第 12-14 周 | match | IT 集成测试 | 跨组件契约 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1396 | PH-5 | 第 12-14 周 | match | IT 集成测试 | 端到端集成 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1397 | PH-5 | 第 12-14 周 | match | ST 系统测试 | K8s 部署验证 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1398 | PH-5 | 第 12-14 周 | match | ST 系统测试 | 性能 / 容量 NFR | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1399 | PH-5 | 第 12-14 周 | match | ST 系统测试 | chaos 故障注入 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1400 | PH-5 | 第 12-14 周 | match | ST 系统测试 | RPO/RTO 验证 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1401 | PH-5 | 第 12-14 周 | match | Helm chart | Chart.yaml | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1402 | PH-5 | 第 12-14 周 | match | Helm chart | values.yaml | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1403 | PH-5 | 第 12-14 周 | match | Helm chart | deployment + HPA | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1404 | PH-5 | 第 12-14 周 | match | Helm chart | NetworkPolicy | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1405 | PH-5 | 第 12-14 周 | match | observability | OTel spans | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1406 | PH-5 | 第 12-14 周 | match | observability | Prometheus metrics | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1407 | PH-5 | 第 12-14 周 | match | observability | Grafana 仪表盘 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1408 | PH-5 | 第 12-14 周 | match | observability | Loki 日志 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1409 | PH-5 | 第 12-14 周 | social | API Spec | 列出 gRPC 方法 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1410 | PH-5 | 第 12-14 周 | social | API Spec | 定义 Proto 文件 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1411 | PH-5 | 第 12-14 周 | social | API Spec | 配置 tonic-build | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1412 | PH-5 | 第 12-14 周 | social | API Spec | 编译期校验 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1413 | PH-5 | 第 12-14 周 | social | 业务逻辑 | 实体表定义 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1414 | PH-5 | 第 12-14 周 | social | 业务逻辑 | 状态机实现 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1415 | PH-5 | 第 12-14 周 | social | 业务逻辑 | 错误码 + 边界条件 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1416 | PH-5 | 第 12-14 周 | social | 业务逻辑 | 核心算法 / 决策 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1417 | PH-5 | 第 12-14 周 | social | DB migration | Schema 迁移 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1418 | PH-5 | 第 12-14 周 | social | DB migration | 索引 + 约束 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1419 | PH-5 | 第 12-14 周 | social | DB migration | 双向迁移演练 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1420 | PH-5 | 第 12-14 周 | social | DB migration | 回滚预案 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1421 | PH-5 | 第 12-14 周 | social | UT 单元测试 | testkit helper | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1422 | PH-5 | 第 12-14 周 | social | UT 单元测试 | CRUD 覆盖 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1423 | PH-5 | 第 12-14 周 | social | UT 单元测试 | 状态机覆盖 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1424 | PH-5 | 第 12-14 周 | social | UT 单元测试 | 覆盖率报告 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1425 | PH-5 | 第 12-14 周 | social | IT 集成测试 | service 启动 + health | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1426 | PH-5 | 第 12-14 周 | social | IT 集成测试 | DB 集成 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1427 | PH-5 | 第 12-14 周 | social | IT 集成测试 | 跨组件契约 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1428 | PH-5 | 第 12-14 周 | social | IT 集成测试 | 端到端集成 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1429 | PH-5 | 第 12-14 周 | social | ST 系统测试 | K8s 部署验证 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1430 | PH-5 | 第 12-14 周 | social | ST 系统测试 | 性能 / 容量 NFR | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1431 | PH-5 | 第 12-14 周 | social | ST 系统测试 | chaos 故障注入 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1432 | PH-5 | 第 12-14 周 | social | ST 系统测试 | RPO/RTO 验证 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1433 | PH-5 | 第 12-14 周 | social | Helm chart | Chart.yaml | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1434 | PH-5 | 第 12-14 周 | social | Helm chart | values.yaml | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1435 | PH-5 | 第 12-14 周 | social | Helm chart | deployment + HPA | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1436 | PH-5 | 第 12-14 周 | social | Helm chart | NetworkPolicy | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1437 | PH-5 | 第 12-14 周 | social | observability | OTel spans | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1438 | PH-5 | 第 12-14 周 | social | observability | Prometheus metrics | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1439 | PH-5 | 第 12-14 周 | social | observability | Grafana 仪表盘 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1440 | PH-5 | 第 12-14 周 | social | observability | Loki 日志 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1441 | PH-5 | 第 12-14 周 | admin | API Spec | 列出 gRPC 方法 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1442 | PH-5 | 第 12-14 周 | admin | API Spec | 定义 Proto 文件 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1443 | PH-5 | 第 12-14 周 | admin | API Spec | 配置 tonic-build | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1444 | PH-5 | 第 12-14 周 | admin | API Spec | 编译期校验 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1445 | PH-5 | 第 12-14 周 | admin | 业务逻辑 | 实体表定义 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1446 | PH-5 | 第 12-14 周 | admin | 业务逻辑 | 状态机实现 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1447 | PH-5 | 第 12-14 周 | admin | 业务逻辑 | 错误码 + 边界条件 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1448 | PH-5 | 第 12-14 周 | admin | 业务逻辑 | 核心算法 / 决策 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1449 | PH-5 | 第 12-14 周 | admin | DB migration | Schema 迁移 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1450 | PH-5 | 第 12-14 周 | admin | DB migration | 索引 + 约束 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1451 | PH-5 | 第 12-14 周 | admin | DB migration | 双向迁移演练 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1452 | PH-5 | 第 12-14 周 | admin | DB migration | 回滚预案 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1453 | PH-5 | 第 12-14 周 | admin | UT 单元测试 | testkit helper | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1454 | PH-5 | 第 12-14 周 | admin | UT 单元测试 | CRUD 覆盖 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1455 | PH-5 | 第 12-14 周 | admin | UT 单元测试 | 状态机覆盖 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1456 | PH-5 | 第 12-14 周 | admin | UT 单元测试 | 覆盖率报告 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1457 | PH-5 | 第 12-14 周 | admin | IT 集成测试 | service 启动 + health | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1458 | PH-5 | 第 12-14 周 | admin | IT 集成测试 | DB 集成 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1459 | PH-5 | 第 12-14 周 | admin | IT 集成测试 | 跨组件契约 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1460 | PH-5 | 第 12-14 周 | admin | IT 集成测试 | 端到端集成 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1461 | PH-5 | 第 12-14 周 | admin | ST 系统测试 | K8s 部署验证 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1462 | PH-5 | 第 12-14 周 | admin | ST 系统测试 | 性能 / 容量 NFR | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1463 | PH-5 | 第 12-14 周 | admin | ST 系统测试 | chaos 故障注入 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1464 | PH-5 | 第 12-14 周 | admin | ST 系统测试 | RPO/RTO 验证 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1465 | PH-5 | 第 12-14 周 | admin | Helm chart | Chart.yaml | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1466 | PH-5 | 第 12-14 周 | admin | Helm chart | values.yaml | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1467 | PH-5 | 第 12-14 周 | admin | Helm chart | deployment + HPA | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1468 | PH-5 | 第 12-14 周 | admin | Helm chart | NetworkPolicy | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1469 | PH-5 | 第 12-14 周 | admin | observability | OTel spans | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1470 | PH-5 | 第 12-14 周 | admin | observability | Prometheus metrics | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1471 | PH-5 | 第 12-14 周 | admin | observability | Grafana 仪表盘 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1472 | PH-5 | 第 12-14 周 | admin | observability | Loki 日志 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1473 | PH-5 | 第 12-14 周 | cluster-ops | Control Plane API | ClusterOps gRPC 定义 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1474 | PH-5 | 第 12-14 周 | cluster-ops | Control Plane API | AdminService 转发 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1475 | PH-5 | 第 12-14 周 | cluster-ops | Control Plane API | request_id 幂等 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1476 | PH-5 | 第 12-14 周 | cluster-ops | Control Plane API | OCC 版本字段 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1477 | PH-5 | 第 12-14 周 | cluster-ops | CEM | Feature registry | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1478 | PH-5 | 第 12-14 周 | cluster-ops | CEM | 事件流 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1479 | PH-5 | 第 12-14 周 | cluster-ops | CEM | 订阅/取消订阅 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1480 | PH-5 | 第 12-14 周 | cluster-ops | CEM | DLQ 处理 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1481 | PH-5 | 第 12-14 周 | cluster-ops | PFAU | declared → canary → confirm → done 状态机 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1482 | PH-5 | 第 12-14 周 | cluster-ops | PFAU | all-reachable 确认 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1483 | PH-5 | 第 12-14 周 | cluster-ops | PFAU | 灰度策略 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1484 | PH-5 | 第 12-14 周 | cluster-ops | PFAU | 回滚路径 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1485 | PH-5 | 第 12-14 周 | cluster-ops | 状态机 | feature 状态定义 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1486 | PH-5 | 第 12-14 周 | cluster-ops | 状态机 | 非法转移检测 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1487 | PH-5 | 第 12-14 周 | cluster-ops | 状态机 | 状态转移图 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1488 | PH-5 | 第 12-14 周 | cluster-ops | 状态机 | 持久化方案 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1489 | PH-5 | 第 12-14 周 | cluster-ops | RBAC | GM / COC / 客户端 3 套权限 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1490 | PH-5 | 第 12-14 周 | cluster-ops | RBAC | 权限矩阵 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1491 | PH-5 | 第 12-14 周 | cluster-ops | RBAC | 审计日志 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1492 | PH-5 | 第 12-14 周 | cluster-ops | RBAC | 撤销机制 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1493 | PH-5 | 第 12-14 周 | cluster-ops | fencing | 租约机制 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1494 | PH-5 | 第 12-14 周 | cluster-ops | fencing | CAS 版本控制 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1495 | PH-5 | 第 12-14 周 | cluster-ops | fencing | stale leader 检测 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1496 | PH-5 | 第 12-14 周 | cluster-ops | fencing | 集群隔离策略 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1497 | PH-5 | 第 12-14 周 | cluster-ops | 审计 | 审计 schema | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1498 | PH-5 | 第 12-14 周 | cluster-ops | 审计 | 写入路径 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1499 | PH-5 | 第 12-14 周 | cluster-ops | 审计 | 查询接口 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1500 | PH-5 | 第 12-14 周 | cluster-ops | 审计 | 保留期 + 归档 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1501 | PH-5 | 第 12-14 周 | cluster-ops | OCC | 乐观并发控制 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1502 | PH-5 | 第 12-14 周 | cluster-ops | OCC | 重试策略 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1503 | PH-5 | 第 12-14 周 | cluster-ops | OCC | 冲突检测 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1504 | PH-5 | 第 12-14 周 | cluster-ops | OCC | 死锁恢复 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1505 | PH-5 | 第 12-14 周 | shared-platform | Rust 工具链 | rustup 1.98 锁定 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1506 | PH-5 | 第 12-14 周 | shared-platform | Rust 工具链 | rust-toolchain.toml | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1507 | PH-5 | 第 12-14 周 | shared-platform | Rust 工具链 | CI cache | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1508 | PH-5 | 第 12-14 周 | shared-platform | Rust 工具链 | 升级评审 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1509 | PH-5 | 第 12-14 周 | shared-platform | Cargo.lock 锁定 | --locked 构建 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1510 | PH-5 | 第 12-14 周 | shared-platform | Cargo.lock 锁定 | workspace 统一锁 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1511 | PH-5 | 第 12-14 周 | shared-platform | Cargo.lock 锁定 | 依赖审计 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1512 | PH-5 | 第 12-14 周 | shared-platform | Cargo.lock 锁定 | 更新策略 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1513 | PH-5 | 第 12-14 周 | shared-platform | 镜像构建 | Dockerfile.distroless | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1514 | PH-5 | 第 12-14 周 | shared-platform | 镜像构建 | 镜像大小优化 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1515 | PH-5 | 第 12-14 周 | shared-platform | 镜像构建 | SBOM 生成 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1516 | PH-5 | 第 12-14 周 | shared-platform | 镜像构建 | 漏洞扫描 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1517 | PH-5 | 第 12-14 周 | shared-platform | K3s | K3s 集群初始化 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1518 | PH-5 | 第 12-14 周 | shared-platform | K3s | kubectl 配置 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1519 | PH-5 | 第 12-14 周 | shared-platform | K3s | NetworkPolicy | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1520 | PH-5 | 第 12-14 周 | shared-platform | K3s | HPA / VPA | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1521 | PH-5 | 第 12-14 周 | shared-platform | OTel Collector | 采集器配置 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1522 | PH-5 | 第 12-14 周 | shared-platform | OTel Collector | OTLP 接收 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1523 | PH-5 | 第 12-14 周 | shared-platform | OTel Collector | 采样策略 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1524 | PH-5 | 第 12-14 周 | shared-platform | OTel Collector | 导出器配置 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1525 | PH-5 | 第 12-14 周 | shared-platform | Helm | Chart 模板 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1526 | PH-5 | 第 12-14 周 | shared-platform | Helm | values 校验 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1527 | PH-5 | 第 12-14 周 | shared-platform | Helm | 依赖管理 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1528 | PH-5 | 第 12-14 周 | shared-platform | Helm | CI render | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1529 | PH-5 | 第 12-14 周 | shared-platform | 密钥 | Vault/OpenBao 部署 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1530 | PH-5 | 第 12-14 周 | shared-platform | 密钥 | 密钥轮换策略 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1531 | PH-5 | 第 12-14 周 | shared-platform | 密钥 | 运行时注入 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1532 | PH-5 | 第 12-14 周 | shared-platform | 密钥 | 审计 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1533 | PH-5 | 第 12-14 周 | shared-platform | 灾备 | 备份策略 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1534 | PH-5 | 第 12-14 周 | shared-platform | 灾备 | 恢复演练 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1535 | PH-5 | 第 12-14 周 | shared-platform | 灾备 | 跨 AZ 复制 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1536 | PH-5 | 第 12-14 周 | shared-platform | 灾备 | RTO/RPO 验证 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1537 | PH-6 | 第 14-16 周 | foundation | workspace 骨架 | virtual workspace 配置 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1538 | PH-6 | 第 14-16 周 | foundation | workspace 骨架 | resolver=3 锁定 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1539 | PH-6 | 第 14-16 周 | foundation | workspace 骨架 | Edition 2024 升级 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1540 | PH-6 | 第 14-16 周 | foundation | workspace 骨架 | crate 间依赖方向 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1541 | PH-6 | 第 14-16 周 | foundation | testkit 共用 | testcontainers PG 封装 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1542 | PH-6 | 第 14-16 周 | foundation | testkit 共用 | mock helpers | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1543 | PH-6 | 第 14-16 周 | foundation | testkit 共用 | fixture builders | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1544 | PH-6 | 第 14-16 周 | foundation | testkit 共用 | coverage 报告 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1545 | PH-6 | 第 14-16 周 | foundation | CI 工具链 | GitHub Actions 配置 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1546 | PH-6 | 第 14-16 周 | foundation | CI 工具链 | cargo fmt/clippy/deny | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1547 | PH-6 | 第 14-16 周 | foundation | CI 工具链 | sqlx prepare check | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1548 | PH-6 | 第 14-16 周 | foundation | CI 工具链 | manifest 校验 CI | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1549 | PH-6 | 第 14-16 周 | foundation | DAG validator | 拓扑排序算法 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1550 | PH-6 | 第 14-16 周 | foundation | DAG validator | 环依赖检测 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1551 | PH-6 | 第 14-16 周 | foundation | DAG validator | 缺祖先检测 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1552 | PH-6 | 第 14-16 周 | foundation | DAG validator | 负例测试套件 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1553 | PH-6 | 第 14-16 周 | foundation | cargo-deny/audit | 许可证白名单 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1554 | PH-6 | 第 14-16 周 | foundation | cargo-deny/audit | 漏洞数据库 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1555 | PH-6 | 第 14-16 周 | foundation | cargo-deny/audit | 依赖来源限制 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1556 | PH-6 | 第 14-16 周 | foundation | cargo-deny/audit | CI 集成 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1557 | PH-6 | 第 14-16 周 | foundation | manifest schema | JSON Schema 起草 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1558 | PH-6 | 第 14-16 周 | foundation | manifest schema | ARC-042 字段映射 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1559 | PH-6 | 第 14-16 周 | foundation | manifest schema | schema 校验 CLI | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1560 | PH-6 | 第 14-16 周 | foundation | manifest schema | 示例 manifest | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1561 | PH-6 | 第 14-16 周 | foundation | 文档生成 | mdbook 配置 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1562 | PH-6 | 第 14-16 周 | foundation | 文档生成 | Doxygen/Rustdoc | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1563 | PH-6 | 第 14-16 周 | foundation | 文档生成 | CR 链接检查 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1564 | PH-6 | 第 14-16 周 | foundation | 文档生成 | CI 文档门禁 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1565 | PH-6 | 第 14-16 周 | foundation | 工程约定 | 错误码定义 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1566 | PH-6 | 第 14-16 周 | foundation | 工程约定 | 序列化约定 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1567 | PH-6 | 第 14-16 周 | foundation | 工程约定 | 日志规范 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1568 | PH-6 | 第 14-16 周 | foundation | 工程约定 | metrics 命名 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1569 | PH-6 | 第 14-16 周 | player | API Spec | 列出 gRPC 方法 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1570 | PH-6 | 第 14-16 周 | player | API Spec | 定义 Proto 文件 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1571 | PH-6 | 第 14-16 周 | player | API Spec | 配置 tonic-build | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1572 | PH-6 | 第 14-16 周 | player | API Spec | 编译期校验 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1573 | PH-6 | 第 14-16 周 | player | 业务逻辑 | 实体表定义 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1574 | PH-6 | 第 14-16 周 | player | 业务逻辑 | 状态机实现 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1575 | PH-6 | 第 14-16 周 | player | 业务逻辑 | 错误码 + 边界条件 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1576 | PH-6 | 第 14-16 周 | player | 业务逻辑 | 核心算法 / 决策 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1577 | PH-6 | 第 14-16 周 | player | DB migration | Schema 迁移 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1578 | PH-6 | 第 14-16 周 | player | DB migration | 索引 + 约束 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1579 | PH-6 | 第 14-16 周 | player | DB migration | 双向迁移演练 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1580 | PH-6 | 第 14-16 周 | player | DB migration | 回滚预案 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1581 | PH-6 | 第 14-16 周 | player | UT 单元测试 | testkit helper | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1582 | PH-6 | 第 14-16 周 | player | UT 单元测试 | CRUD 覆盖 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1583 | PH-6 | 第 14-16 周 | player | UT 单元测试 | 状态机覆盖 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1584 | PH-6 | 第 14-16 周 | player | UT 单元测试 | 覆盖率报告 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1585 | PH-6 | 第 14-16 周 | player | IT 集成测试 | service 启动 + health | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1586 | PH-6 | 第 14-16 周 | player | IT 集成测试 | DB 集成 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1587 | PH-6 | 第 14-16 周 | player | IT 集成测试 | 跨组件契约 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1588 | PH-6 | 第 14-16 周 | player | IT 集成测试 | 端到端集成 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1589 | PH-6 | 第 14-16 周 | player | ST 系统测试 | K8s 部署验证 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1590 | PH-6 | 第 14-16 周 | player | ST 系统测试 | 性能 / 容量 NFR | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1591 | PH-6 | 第 14-16 周 | player | ST 系统测试 | chaos 故障注入 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1592 | PH-6 | 第 14-16 周 | player | ST 系统测试 | RPO/RTO 验证 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1593 | PH-6 | 第 14-16 周 | player | Helm chart | Chart.yaml | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1594 | PH-6 | 第 14-16 周 | player | Helm chart | values.yaml | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1595 | PH-6 | 第 14-16 周 | player | Helm chart | deployment + HPA | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1596 | PH-6 | 第 14-16 周 | player | Helm chart | NetworkPolicy | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1597 | PH-6 | 第 14-16 周 | player | observability | OTel spans | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1598 | PH-6 | 第 14-16 周 | player | observability | Prometheus metrics | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1599 | PH-6 | 第 14-16 周 | player | observability | Grafana 仪表盘 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1600 | PH-6 | 第 14-16 周 | player | observability | Loki 日志 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1601 | PH-6 | 第 14-16 周 | economy | API Spec | 列出 gRPC 方法 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1602 | PH-6 | 第 14-16 周 | economy | API Spec | 定义 Proto 文件 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1603 | PH-6 | 第 14-16 周 | economy | API Spec | 配置 tonic-build | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1604 | PH-6 | 第 14-16 周 | economy | API Spec | 编译期校验 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1605 | PH-6 | 第 14-16 周 | economy | 业务逻辑 | 实体表定义 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1606 | PH-6 | 第 14-16 周 | economy | 业务逻辑 | 状态机实现 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1607 | PH-6 | 第 14-16 周 | economy | 业务逻辑 | 错误码 + 边界条件 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1608 | PH-6 | 第 14-16 周 | economy | 业务逻辑 | 核心算法 / 决策 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1609 | PH-6 | 第 14-16 周 | economy | DB migration | Schema 迁移 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1610 | PH-6 | 第 14-16 周 | economy | DB migration | 索引 + 约束 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1611 | PH-6 | 第 14-16 周 | economy | DB migration | 双向迁移演练 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1612 | PH-6 | 第 14-16 周 | economy | DB migration | 回滚预案 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1613 | PH-6 | 第 14-16 周 | economy | UT 单元测试 | testkit helper | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1614 | PH-6 | 第 14-16 周 | economy | UT 单元测试 | CRUD 覆盖 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1615 | PH-6 | 第 14-16 周 | economy | UT 单元测试 | 状态机覆盖 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1616 | PH-6 | 第 14-16 周 | economy | UT 单元测试 | 覆盖率报告 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1617 | PH-6 | 第 14-16 周 | economy | IT 集成测试 | service 启动 + health | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1618 | PH-6 | 第 14-16 周 | economy | IT 集成测试 | DB 集成 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1619 | PH-6 | 第 14-16 周 | economy | IT 集成测试 | 跨组件契约 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1620 | PH-6 | 第 14-16 周 | economy | IT 集成测试 | 端到端集成 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1621 | PH-6 | 第 14-16 周 | economy | ST 系统测试 | K8s 部署验证 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1622 | PH-6 | 第 14-16 周 | economy | ST 系统测试 | 性能 / 容量 NFR | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1623 | PH-6 | 第 14-16 周 | economy | ST 系统测试 | chaos 故障注入 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1624 | PH-6 | 第 14-16 周 | economy | ST 系统测试 | RPO/RTO 验证 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1625 | PH-6 | 第 14-16 周 | economy | Helm chart | Chart.yaml | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1626 | PH-6 | 第 14-16 周 | economy | Helm chart | values.yaml | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1627 | PH-6 | 第 14-16 周 | economy | Helm chart | deployment + HPA | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1628 | PH-6 | 第 14-16 周 | economy | Helm chart | NetworkPolicy | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1629 | PH-6 | 第 14-16 周 | economy | observability | OTel spans | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1630 | PH-6 | 第 14-16 周 | economy | observability | Prometheus metrics | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1631 | PH-6 | 第 14-16 周 | economy | observability | Grafana 仪表盘 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1632 | PH-6 | 第 14-16 周 | economy | observability | Loki 日志 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1633 | PH-6 | 第 14-16 周 | match | API Spec | 列出 gRPC 方法 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1634 | PH-6 | 第 14-16 周 | match | API Spec | 定义 Proto 文件 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1635 | PH-6 | 第 14-16 周 | match | API Spec | 配置 tonic-build | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1636 | PH-6 | 第 14-16 周 | match | API Spec | 编译期校验 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1637 | PH-6 | 第 14-16 周 | match | 业务逻辑 | 实体表定义 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1638 | PH-6 | 第 14-16 周 | match | 业务逻辑 | 状态机实现 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1639 | PH-6 | 第 14-16 周 | match | 业务逻辑 | 错误码 + 边界条件 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1640 | PH-6 | 第 14-16 周 | match | 业务逻辑 | 核心算法 / 决策 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1641 | PH-6 | 第 14-16 周 | match | DB migration | Schema 迁移 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1642 | PH-6 | 第 14-16 周 | match | DB migration | 索引 + 约束 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1643 | PH-6 | 第 14-16 周 | match | DB migration | 双向迁移演练 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1644 | PH-6 | 第 14-16 周 | match | DB migration | 回滚预案 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1645 | PH-6 | 第 14-16 周 | match | UT 单元测试 | testkit helper | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1646 | PH-6 | 第 14-16 周 | match | UT 单元测试 | CRUD 覆盖 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1647 | PH-6 | 第 14-16 周 | match | UT 单元测试 | 状态机覆盖 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1648 | PH-6 | 第 14-16 周 | match | UT 单元测试 | 覆盖率报告 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1649 | PH-6 | 第 14-16 周 | match | IT 集成测试 | service 启动 + health | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1650 | PH-6 | 第 14-16 周 | match | IT 集成测试 | DB 集成 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1651 | PH-6 | 第 14-16 周 | match | IT 集成测试 | 跨组件契约 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1652 | PH-6 | 第 14-16 周 | match | IT 集成测试 | 端到端集成 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1653 | PH-6 | 第 14-16 周 | match | ST 系统测试 | K8s 部署验证 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1654 | PH-6 | 第 14-16 周 | match | ST 系统测试 | 性能 / 容量 NFR | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1655 | PH-6 | 第 14-16 周 | match | ST 系统测试 | chaos 故障注入 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1656 | PH-6 | 第 14-16 周 | match | ST 系统测试 | RPO/RTO 验证 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1657 | PH-6 | 第 14-16 周 | match | Helm chart | Chart.yaml | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1658 | PH-6 | 第 14-16 周 | match | Helm chart | values.yaml | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1659 | PH-6 | 第 14-16 周 | match | Helm chart | deployment + HPA | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1660 | PH-6 | 第 14-16 周 | match | Helm chart | NetworkPolicy | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1661 | PH-6 | 第 14-16 周 | match | observability | OTel spans | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1662 | PH-6 | 第 14-16 周 | match | observability | Prometheus metrics | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1663 | PH-6 | 第 14-16 周 | match | observability | Grafana 仪表盘 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1664 | PH-6 | 第 14-16 周 | match | observability | Loki 日志 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1665 | PH-6 | 第 14-16 周 | social | API Spec | 列出 gRPC 方法 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1666 | PH-6 | 第 14-16 周 | social | API Spec | 定义 Proto 文件 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1667 | PH-6 | 第 14-16 周 | social | API Spec | 配置 tonic-build | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1668 | PH-6 | 第 14-16 周 | social | API Spec | 编译期校验 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1669 | PH-6 | 第 14-16 周 | social | 业务逻辑 | 实体表定义 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1670 | PH-6 | 第 14-16 周 | social | 业务逻辑 | 状态机实现 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1671 | PH-6 | 第 14-16 周 | social | 业务逻辑 | 错误码 + 边界条件 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1672 | PH-6 | 第 14-16 周 | social | 业务逻辑 | 核心算法 / 决策 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1673 | PH-6 | 第 14-16 周 | social | DB migration | Schema 迁移 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1674 | PH-6 | 第 14-16 周 | social | DB migration | 索引 + 约束 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1675 | PH-6 | 第 14-16 周 | social | DB migration | 双向迁移演练 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1676 | PH-6 | 第 14-16 周 | social | DB migration | 回滚预案 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1677 | PH-6 | 第 14-16 周 | social | UT 单元测试 | testkit helper | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1678 | PH-6 | 第 14-16 周 | social | UT 单元测试 | CRUD 覆盖 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1679 | PH-6 | 第 14-16 周 | social | UT 单元测试 | 状态机覆盖 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1680 | PH-6 | 第 14-16 周 | social | UT 单元测试 | 覆盖率报告 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1681 | PH-6 | 第 14-16 周 | social | IT 集成测试 | service 启动 + health | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1682 | PH-6 | 第 14-16 周 | social | IT 集成测试 | DB 集成 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1683 | PH-6 | 第 14-16 周 | social | IT 集成测试 | 跨组件契约 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1684 | PH-6 | 第 14-16 周 | social | IT 集成测试 | 端到端集成 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1685 | PH-6 | 第 14-16 周 | social | ST 系统测试 | K8s 部署验证 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1686 | PH-6 | 第 14-16 周 | social | ST 系统测试 | 性能 / 容量 NFR | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1687 | PH-6 | 第 14-16 周 | social | ST 系统测试 | chaos 故障注入 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1688 | PH-6 | 第 14-16 周 | social | ST 系统测试 | RPO/RTO 验证 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1689 | PH-6 | 第 14-16 周 | social | Helm chart | Chart.yaml | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1690 | PH-6 | 第 14-16 周 | social | Helm chart | values.yaml | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1691 | PH-6 | 第 14-16 周 | social | Helm chart | deployment + HPA | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1692 | PH-6 | 第 14-16 周 | social | Helm chart | NetworkPolicy | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1693 | PH-6 | 第 14-16 周 | social | observability | OTel spans | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1694 | PH-6 | 第 14-16 周 | social | observability | Prometheus metrics | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1695 | PH-6 | 第 14-16 周 | social | observability | Grafana 仪表盘 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1696 | PH-6 | 第 14-16 周 | social | observability | Loki 日志 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1697 | PH-6 | 第 14-16 周 | admin | API Spec | 列出 gRPC 方法 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1698 | PH-6 | 第 14-16 周 | admin | API Spec | 定义 Proto 文件 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1699 | PH-6 | 第 14-16 周 | admin | API Spec | 配置 tonic-build | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1700 | PH-6 | 第 14-16 周 | admin | API Spec | 编译期校验 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1701 | PH-6 | 第 14-16 周 | admin | 业务逻辑 | 实体表定义 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1702 | PH-6 | 第 14-16 周 | admin | 业务逻辑 | 状态机实现 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1703 | PH-6 | 第 14-16 周 | admin | 业务逻辑 | 错误码 + 边界条件 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1704 | PH-6 | 第 14-16 周 | admin | 业务逻辑 | 核心算法 / 决策 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1705 | PH-6 | 第 14-16 周 | admin | DB migration | Schema 迁移 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1706 | PH-6 | 第 14-16 周 | admin | DB migration | 索引 + 约束 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1707 | PH-6 | 第 14-16 周 | admin | DB migration | 双向迁移演练 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1708 | PH-6 | 第 14-16 周 | admin | DB migration | 回滚预案 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1709 | PH-6 | 第 14-16 周 | admin | UT 单元测试 | testkit helper | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1710 | PH-6 | 第 14-16 周 | admin | UT 单元测试 | CRUD 覆盖 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1711 | PH-6 | 第 14-16 周 | admin | UT 单元测试 | 状态机覆盖 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1712 | PH-6 | 第 14-16 周 | admin | UT 单元测试 | 覆盖率报告 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1713 | PH-6 | 第 14-16 周 | admin | IT 集成测试 | service 启动 + health | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1714 | PH-6 | 第 14-16 周 | admin | IT 集成测试 | DB 集成 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1715 | PH-6 | 第 14-16 周 | admin | IT 集成测试 | 跨组件契约 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1716 | PH-6 | 第 14-16 周 | admin | IT 集成测试 | 端到端集成 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1717 | PH-6 | 第 14-16 周 | admin | ST 系统测试 | K8s 部署验证 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1718 | PH-6 | 第 14-16 周 | admin | ST 系统测试 | 性能 / 容量 NFR | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1719 | PH-6 | 第 14-16 周 | admin | ST 系统测试 | chaos 故障注入 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1720 | PH-6 | 第 14-16 周 | admin | ST 系统测试 | RPO/RTO 验证 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1721 | PH-6 | 第 14-16 周 | admin | Helm chart | Chart.yaml | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1722 | PH-6 | 第 14-16 周 | admin | Helm chart | values.yaml | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1723 | PH-6 | 第 14-16 周 | admin | Helm chart | deployment + HPA | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1724 | PH-6 | 第 14-16 周 | admin | Helm chart | NetworkPolicy | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1725 | PH-6 | 第 14-16 周 | admin | observability | OTel spans | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1726 | PH-6 | 第 14-16 周 | admin | observability | Prometheus metrics | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1727 | PH-6 | 第 14-16 周 | admin | observability | Grafana 仪表盘 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1728 | PH-6 | 第 14-16 周 | admin | observability | Loki 日志 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1729 | PH-6 | 第 14-16 周 | cluster-ops | Control Plane API | ClusterOps gRPC 定义 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1730 | PH-6 | 第 14-16 周 | cluster-ops | Control Plane API | AdminService 转发 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1731 | PH-6 | 第 14-16 周 | cluster-ops | Control Plane API | request_id 幂等 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1732 | PH-6 | 第 14-16 周 | cluster-ops | Control Plane API | OCC 版本字段 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1733 | PH-6 | 第 14-16 周 | cluster-ops | CEM | Feature registry | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1734 | PH-6 | 第 14-16 周 | cluster-ops | CEM | 事件流 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1735 | PH-6 | 第 14-16 周 | cluster-ops | CEM | 订阅/取消订阅 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1736 | PH-6 | 第 14-16 周 | cluster-ops | CEM | DLQ 处理 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1737 | PH-6 | 第 14-16 周 | cluster-ops | PFAU | declared → canary → confirm → done 状态机 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1738 | PH-6 | 第 14-16 周 | cluster-ops | PFAU | all-reachable 确认 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1739 | PH-6 | 第 14-16 周 | cluster-ops | PFAU | 灰度策略 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1740 | PH-6 | 第 14-16 周 | cluster-ops | PFAU | 回滚路径 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1741 | PH-6 | 第 14-16 周 | cluster-ops | 状态机 | feature 状态定义 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1742 | PH-6 | 第 14-16 周 | cluster-ops | 状态机 | 非法转移检测 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1743 | PH-6 | 第 14-16 周 | cluster-ops | 状态机 | 状态转移图 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1744 | PH-6 | 第 14-16 周 | cluster-ops | 状态机 | 持久化方案 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1745 | PH-6 | 第 14-16 周 | cluster-ops | RBAC | GM / COC / 客户端 3 套权限 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1746 | PH-6 | 第 14-16 周 | cluster-ops | RBAC | 权限矩阵 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1747 | PH-6 | 第 14-16 周 | cluster-ops | RBAC | 审计日志 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1748 | PH-6 | 第 14-16 周 | cluster-ops | RBAC | 撤销机制 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1749 | PH-6 | 第 14-16 周 | cluster-ops | fencing | 租约机制 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1750 | PH-6 | 第 14-16 周 | cluster-ops | fencing | CAS 版本控制 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1751 | PH-6 | 第 14-16 周 | cluster-ops | fencing | stale leader 检测 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1752 | PH-6 | 第 14-16 周 | cluster-ops | fencing | 集群隔离策略 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1753 | PH-6 | 第 14-16 周 | cluster-ops | 审计 | 审计 schema | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1754 | PH-6 | 第 14-16 周 | cluster-ops | 审计 | 写入路径 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1755 | PH-6 | 第 14-16 周 | cluster-ops | 审计 | 查询接口 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1756 | PH-6 | 第 14-16 周 | cluster-ops | 审计 | 保留期 + 归档 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1757 | PH-6 | 第 14-16 周 | cluster-ops | OCC | 乐观并发控制 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1758 | PH-6 | 第 14-16 周 | cluster-ops | OCC | 重试策略 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1759 | PH-6 | 第 14-16 周 | cluster-ops | OCC | 冲突检测 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1760 | PH-6 | 第 14-16 周 | cluster-ops | OCC | 死锁恢复 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1761 | PH-6 | 第 14-16 周 | shared-platform | Rust 工具链 | rustup 1.98 锁定 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1762 | PH-6 | 第 14-16 周 | shared-platform | Rust 工具链 | rust-toolchain.toml | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1763 | PH-6 | 第 14-16 周 | shared-platform | Rust 工具链 | CI cache | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1764 | PH-6 | 第 14-16 周 | shared-platform | Rust 工具链 | 升级评审 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1765 | PH-6 | 第 14-16 周 | shared-platform | Cargo.lock 锁定 | --locked 构建 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1766 | PH-6 | 第 14-16 周 | shared-platform | Cargo.lock 锁定 | workspace 统一锁 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1767 | PH-6 | 第 14-16 周 | shared-platform | Cargo.lock 锁定 | 依赖审计 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1768 | PH-6 | 第 14-16 周 | shared-platform | Cargo.lock 锁定 | 更新策略 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1769 | PH-6 | 第 14-16 周 | shared-platform | 镜像构建 | Dockerfile.distroless | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1770 | PH-6 | 第 14-16 周 | shared-platform | 镜像构建 | 镜像大小优化 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1771 | PH-6 | 第 14-16 周 | shared-platform | 镜像构建 | SBOM 生成 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1772 | PH-6 | 第 14-16 周 | shared-platform | 镜像构建 | 漏洞扫描 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1773 | PH-6 | 第 14-16 周 | shared-platform | K3s | K3s 集群初始化 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1774 | PH-6 | 第 14-16 周 | shared-platform | K3s | kubectl 配置 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1775 | PH-6 | 第 14-16 周 | shared-platform | K3s | NetworkPolicy | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1776 | PH-6 | 第 14-16 周 | shared-platform | K3s | HPA / VPA | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1777 | PH-6 | 第 14-16 周 | shared-platform | OTel Collector | 采集器配置 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1778 | PH-6 | 第 14-16 周 | shared-platform | OTel Collector | OTLP 接收 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1779 | PH-6 | 第 14-16 周 | shared-platform | OTel Collector | 采样策略 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1780 | PH-6 | 第 14-16 周 | shared-platform | OTel Collector | 导出器配置 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1781 | PH-6 | 第 14-16 周 | shared-platform | Helm | Chart 模板 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1782 | PH-6 | 第 14-16 周 | shared-platform | Helm | values 校验 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1783 | PH-6 | 第 14-16 周 | shared-platform | Helm | 依赖管理 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1784 | PH-6 | 第 14-16 周 | shared-platform | Helm | CI render | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1785 | PH-6 | 第 14-16 周 | shared-platform | 密钥 | Vault/OpenBao 部署 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1786 | PH-6 | 第 14-16 周 | shared-platform | 密钥 | 密钥轮换策略 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1787 | PH-6 | 第 14-16 周 | shared-platform | 密钥 | 运行时注入 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1788 | PH-6 | 第 14-16 周 | shared-platform | 密钥 | 审计 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1789 | PH-6 | 第 14-16 周 | shared-platform | 灾备 | 备份策略 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1790 | PH-6 | 第 14-16 周 | shared-platform | 灾备 | 恢复演练 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1791 | PH-6 | 第 14-16 周 | shared-platform | 灾备 | 跨 AZ 复制 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1792 | PH-6 | 第 14-16 周 | shared-platform | 灾备 | RTO/RPO 验证 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1793 | PH-7 | 第 17-18 周 | foundation | workspace 骨架 | virtual workspace 配置 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1794 | PH-7 | 第 17-18 周 | foundation | workspace 骨架 | resolver=3 锁定 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1795 | PH-7 | 第 17-18 周 | foundation | workspace 骨架 | Edition 2024 升级 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1796 | PH-7 | 第 17-18 周 | foundation | workspace 骨架 | crate 间依赖方向 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1797 | PH-7 | 第 17-18 周 | foundation | testkit 共用 | testcontainers PG 封装 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1798 | PH-7 | 第 17-18 周 | foundation | testkit 共用 | mock helpers | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1799 | PH-7 | 第 17-18 周 | foundation | testkit 共用 | fixture builders | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1800 | PH-7 | 第 17-18 周 | foundation | testkit 共用 | coverage 报告 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1801 | PH-7 | 第 17-18 周 | foundation | CI 工具链 | GitHub Actions 配置 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1802 | PH-7 | 第 17-18 周 | foundation | CI 工具链 | cargo fmt/clippy/deny | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1803 | PH-7 | 第 17-18 周 | foundation | CI 工具链 | sqlx prepare check | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1804 | PH-7 | 第 17-18 周 | foundation | CI 工具链 | manifest 校验 CI | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1805 | PH-7 | 第 17-18 周 | foundation | DAG validator | 拓扑排序算法 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1806 | PH-7 | 第 17-18 周 | foundation | DAG validator | 环依赖检测 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1807 | PH-7 | 第 17-18 周 | foundation | DAG validator | 缺祖先检测 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1808 | PH-7 | 第 17-18 周 | foundation | DAG validator | 负例测试套件 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1809 | PH-7 | 第 17-18 周 | foundation | cargo-deny/audit | 许可证白名单 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1810 | PH-7 | 第 17-18 周 | foundation | cargo-deny/audit | 漏洞数据库 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1811 | PH-7 | 第 17-18 周 | foundation | cargo-deny/audit | 依赖来源限制 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1812 | PH-7 | 第 17-18 周 | foundation | cargo-deny/audit | CI 集成 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1813 | PH-7 | 第 17-18 周 | foundation | manifest schema | JSON Schema 起草 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1814 | PH-7 | 第 17-18 周 | foundation | manifest schema | ARC-042 字段映射 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1815 | PH-7 | 第 17-18 周 | foundation | manifest schema | schema 校验 CLI | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1816 | PH-7 | 第 17-18 周 | foundation | manifest schema | 示例 manifest | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1817 | PH-7 | 第 17-18 周 | foundation | 文档生成 | mdbook 配置 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1818 | PH-7 | 第 17-18 周 | foundation | 文档生成 | Doxygen/Rustdoc | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1819 | PH-7 | 第 17-18 周 | foundation | 文档生成 | CR 链接检查 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1820 | PH-7 | 第 17-18 周 | foundation | 文档生成 | CI 文档门禁 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1821 | PH-7 | 第 17-18 周 | foundation | 工程约定 | 错误码定义 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1822 | PH-7 | 第 17-18 周 | foundation | 工程约定 | 序列化约定 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1823 | PH-7 | 第 17-18 周 | foundation | 工程约定 | 日志规范 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1824 | PH-7 | 第 17-18 周 | foundation | 工程约定 | metrics 命名 | 架构师（兼） | _人·天 | _tokens | _ | _ | _ | _ |
| 1825 | PH-7 | 第 17-18 周 | player | API Spec | 列出 gRPC 方法 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1826 | PH-7 | 第 17-18 周 | player | API Spec | 定义 Proto 文件 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1827 | PH-7 | 第 17-18 周 | player | API Spec | 配置 tonic-build | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1828 | PH-7 | 第 17-18 周 | player | API Spec | 编译期校验 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1829 | PH-7 | 第 17-18 周 | player | 业务逻辑 | 实体表定义 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1830 | PH-7 | 第 17-18 周 | player | 业务逻辑 | 状态机实现 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1831 | PH-7 | 第 17-18 周 | player | 业务逻辑 | 错误码 + 边界条件 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1832 | PH-7 | 第 17-18 周 | player | 业务逻辑 | 核心算法 / 决策 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1833 | PH-7 | 第 17-18 周 | player | DB migration | Schema 迁移 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1834 | PH-7 | 第 17-18 周 | player | DB migration | 索引 + 约束 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1835 | PH-7 | 第 17-18 周 | player | DB migration | 双向迁移演练 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1836 | PH-7 | 第 17-18 周 | player | DB migration | 回滚预案 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1837 | PH-7 | 第 17-18 周 | player | UT 单元测试 | testkit helper | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1838 | PH-7 | 第 17-18 周 | player | UT 单元测试 | CRUD 覆盖 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1839 | PH-7 | 第 17-18 周 | player | UT 单元测试 | 状态机覆盖 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1840 | PH-7 | 第 17-18 周 | player | UT 单元测试 | 覆盖率报告 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1841 | PH-7 | 第 17-18 周 | player | IT 集成测试 | service 启动 + health | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1842 | PH-7 | 第 17-18 周 | player | IT 集成测试 | DB 集成 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1843 | PH-7 | 第 17-18 周 | player | IT 集成测试 | 跨组件契约 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1844 | PH-7 | 第 17-18 周 | player | IT 集成测试 | 端到端集成 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1845 | PH-7 | 第 17-18 周 | player | ST 系统测试 | K8s 部署验证 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1846 | PH-7 | 第 17-18 周 | player | ST 系统测试 | 性能 / 容量 NFR | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1847 | PH-7 | 第 17-18 周 | player | ST 系统测试 | chaos 故障注入 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1848 | PH-7 | 第 17-18 周 | player | ST 系统测试 | RPO/RTO 验证 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1849 | PH-7 | 第 17-18 周 | player | Helm chart | Chart.yaml | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1850 | PH-7 | 第 17-18 周 | player | Helm chart | values.yaml | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1851 | PH-7 | 第 17-18 周 | player | Helm chart | deployment + HPA | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1852 | PH-7 | 第 17-18 周 | player | Helm chart | NetworkPolicy | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1853 | PH-7 | 第 17-18 周 | player | observability | OTel spans | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1854 | PH-7 | 第 17-18 周 | player | observability | Prometheus metrics | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1855 | PH-7 | 第 17-18 周 | player | observability | Grafana 仪表盘 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1856 | PH-7 | 第 17-18 周 | player | observability | Loki 日志 | Player 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1857 | PH-7 | 第 17-18 周 | economy | API Spec | 列出 gRPC 方法 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1858 | PH-7 | 第 17-18 周 | economy | API Spec | 定义 Proto 文件 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1859 | PH-7 | 第 17-18 周 | economy | API Spec | 配置 tonic-build | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1860 | PH-7 | 第 17-18 周 | economy | API Spec | 编译期校验 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1861 | PH-7 | 第 17-18 周 | economy | 业务逻辑 | 实体表定义 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1862 | PH-7 | 第 17-18 周 | economy | 业务逻辑 | 状态机实现 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1863 | PH-7 | 第 17-18 周 | economy | 业务逻辑 | 错误码 + 边界条件 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1864 | PH-7 | 第 17-18 周 | economy | 业务逻辑 | 核心算法 / 决策 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1865 | PH-7 | 第 17-18 周 | economy | DB migration | Schema 迁移 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1866 | PH-7 | 第 17-18 周 | economy | DB migration | 索引 + 约束 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1867 | PH-7 | 第 17-18 周 | economy | DB migration | 双向迁移演练 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1868 | PH-7 | 第 17-18 周 | economy | DB migration | 回滚预案 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1869 | PH-7 | 第 17-18 周 | economy | UT 单元测试 | testkit helper | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1870 | PH-7 | 第 17-18 周 | economy | UT 单元测试 | CRUD 覆盖 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1871 | PH-7 | 第 17-18 周 | economy | UT 单元测试 | 状态机覆盖 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1872 | PH-7 | 第 17-18 周 | economy | UT 单元测试 | 覆盖率报告 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1873 | PH-7 | 第 17-18 周 | economy | IT 集成测试 | service 启动 + health | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1874 | PH-7 | 第 17-18 周 | economy | IT 集成测试 | DB 集成 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1875 | PH-7 | 第 17-18 周 | economy | IT 集成测试 | 跨组件契约 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1876 | PH-7 | 第 17-18 周 | economy | IT 集成测试 | 端到端集成 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1877 | PH-7 | 第 17-18 周 | economy | ST 系统测试 | K8s 部署验证 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1878 | PH-7 | 第 17-18 周 | economy | ST 系统测试 | 性能 / 容量 NFR | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1879 | PH-7 | 第 17-18 周 | economy | ST 系统测试 | chaos 故障注入 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1880 | PH-7 | 第 17-18 周 | economy | ST 系统测试 | RPO/RTO 验证 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1881 | PH-7 | 第 17-18 周 | economy | Helm chart | Chart.yaml | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1882 | PH-7 | 第 17-18 周 | economy | Helm chart | values.yaml | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1883 | PH-7 | 第 17-18 周 | economy | Helm chart | deployment + HPA | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1884 | PH-7 | 第 17-18 周 | economy | Helm chart | NetworkPolicy | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1885 | PH-7 | 第 17-18 周 | economy | observability | OTel spans | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1886 | PH-7 | 第 17-18 周 | economy | observability | Prometheus metrics | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1887 | PH-7 | 第 17-18 周 | economy | observability | Grafana 仪表盘 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1888 | PH-7 | 第 17-18 周 | economy | observability | Loki 日志 | Economy 域 Lead（独立 + Q-003 二次确认） | _人·天 | _tokens | _ | _ | _ | _ |
| 1889 | PH-7 | 第 17-18 周 | match | API Spec | 列出 gRPC 方法 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1890 | PH-7 | 第 17-18 周 | match | API Spec | 定义 Proto 文件 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1891 | PH-7 | 第 17-18 周 | match | API Spec | 配置 tonic-build | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1892 | PH-7 | 第 17-18 周 | match | API Spec | 编译期校验 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1893 | PH-7 | 第 17-18 周 | match | 业务逻辑 | 实体表定义 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1894 | PH-7 | 第 17-18 周 | match | 业务逻辑 | 状态机实现 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1895 | PH-7 | 第 17-18 周 | match | 业务逻辑 | 错误码 + 边界条件 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1896 | PH-7 | 第 17-18 周 | match | 业务逻辑 | 核心算法 / 决策 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1897 | PH-7 | 第 17-18 周 | match | DB migration | Schema 迁移 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1898 | PH-7 | 第 17-18 周 | match | DB migration | 索引 + 约束 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1899 | PH-7 | 第 17-18 周 | match | DB migration | 双向迁移演练 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1900 | PH-7 | 第 17-18 周 | match | DB migration | 回滚预案 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1901 | PH-7 | 第 17-18 周 | match | UT 单元测试 | testkit helper | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1902 | PH-7 | 第 17-18 周 | match | UT 单元测试 | CRUD 覆盖 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1903 | PH-7 | 第 17-18 周 | match | UT 单元测试 | 状态机覆盖 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1904 | PH-7 | 第 17-18 周 | match | UT 单元测试 | 覆盖率报告 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1905 | PH-7 | 第 17-18 周 | match | IT 集成测试 | service 启动 + health | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1906 | PH-7 | 第 17-18 周 | match | IT 集成测试 | DB 集成 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1907 | PH-7 | 第 17-18 周 | match | IT 集成测试 | 跨组件契约 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1908 | PH-7 | 第 17-18 周 | match | IT 集成测试 | 端到端集成 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1909 | PH-7 | 第 17-18 周 | match | ST 系统测试 | K8s 部署验证 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1910 | PH-7 | 第 17-18 周 | match | ST 系统测试 | 性能 / 容量 NFR | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1911 | PH-7 | 第 17-18 周 | match | ST 系统测试 | chaos 故障注入 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1912 | PH-7 | 第 17-18 周 | match | ST 系统测试 | RPO/RTO 验证 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1913 | PH-7 | 第 17-18 周 | match | Helm chart | Chart.yaml | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1914 | PH-7 | 第 17-18 周 | match | Helm chart | values.yaml | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1915 | PH-7 | 第 17-18 周 | match | Helm chart | deployment + HPA | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1916 | PH-7 | 第 17-18 周 | match | Helm chart | NetworkPolicy | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1917 | PH-7 | 第 17-18 周 | match | observability | OTel spans | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1918 | PH-7 | 第 17-18 周 | match | observability | Prometheus metrics | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1919 | PH-7 | 第 17-18 周 | match | observability | Grafana 仪表盘 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1920 | PH-7 | 第 17-18 周 | match | observability | Loki 日志 | Match 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1921 | PH-7 | 第 17-18 周 | social | API Spec | 列出 gRPC 方法 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1922 | PH-7 | 第 17-18 周 | social | API Spec | 定义 Proto 文件 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1923 | PH-7 | 第 17-18 周 | social | API Spec | 配置 tonic-build | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1924 | PH-7 | 第 17-18 周 | social | API Spec | 编译期校验 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1925 | PH-7 | 第 17-18 周 | social | 业务逻辑 | 实体表定义 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1926 | PH-7 | 第 17-18 周 | social | 业务逻辑 | 状态机实现 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1927 | PH-7 | 第 17-18 周 | social | 业务逻辑 | 错误码 + 边界条件 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1928 | PH-7 | 第 17-18 周 | social | 业务逻辑 | 核心算法 / 决策 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1929 | PH-7 | 第 17-18 周 | social | DB migration | Schema 迁移 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1930 | PH-7 | 第 17-18 周 | social | DB migration | 索引 + 约束 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1931 | PH-7 | 第 17-18 周 | social | DB migration | 双向迁移演练 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1932 | PH-7 | 第 17-18 周 | social | DB migration | 回滚预案 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1933 | PH-7 | 第 17-18 周 | social | UT 单元测试 | testkit helper | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1934 | PH-7 | 第 17-18 周 | social | UT 单元测试 | CRUD 覆盖 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1935 | PH-7 | 第 17-18 周 | social | UT 单元测试 | 状态机覆盖 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1936 | PH-7 | 第 17-18 周 | social | UT 单元测试 | 覆盖率报告 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1937 | PH-7 | 第 17-18 周 | social | IT 集成测试 | service 启动 + health | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1938 | PH-7 | 第 17-18 周 | social | IT 集成测试 | DB 集成 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1939 | PH-7 | 第 17-18 周 | social | IT 集成测试 | 跨组件契约 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1940 | PH-7 | 第 17-18 周 | social | IT 集成测试 | 端到端集成 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1941 | PH-7 | 第 17-18 周 | social | ST 系统测试 | K8s 部署验证 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1942 | PH-7 | 第 17-18 周 | social | ST 系统测试 | 性能 / 容量 NFR | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1943 | PH-7 | 第 17-18 周 | social | ST 系统测试 | chaos 故障注入 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1944 | PH-7 | 第 17-18 周 | social | ST 系统测试 | RPO/RTO 验证 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1945 | PH-7 | 第 17-18 周 | social | Helm chart | Chart.yaml | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1946 | PH-7 | 第 17-18 周 | social | Helm chart | values.yaml | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1947 | PH-7 | 第 17-18 周 | social | Helm chart | deployment + HPA | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1948 | PH-7 | 第 17-18 周 | social | Helm chart | NetworkPolicy | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1949 | PH-7 | 第 17-18 周 | social | observability | OTel spans | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1950 | PH-7 | 第 17-18 周 | social | observability | Prometheus metrics | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1951 | PH-7 | 第 17-18 周 | social | observability | Grafana 仪表盘 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1952 | PH-7 | 第 17-18 周 | social | observability | Loki 日志 | Social 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1953 | PH-7 | 第 17-18 周 | admin | API Spec | 列出 gRPC 方法 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1954 | PH-7 | 第 17-18 周 | admin | API Spec | 定义 Proto 文件 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1955 | PH-7 | 第 17-18 周 | admin | API Spec | 配置 tonic-build | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1956 | PH-7 | 第 17-18 周 | admin | API Spec | 编译期校验 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1957 | PH-7 | 第 17-18 周 | admin | 业务逻辑 | 实体表定义 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1958 | PH-7 | 第 17-18 周 | admin | 业务逻辑 | 状态机实现 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1959 | PH-7 | 第 17-18 周 | admin | 业务逻辑 | 错误码 + 边界条件 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1960 | PH-7 | 第 17-18 周 | admin | 业务逻辑 | 核心算法 / 决策 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1961 | PH-7 | 第 17-18 周 | admin | DB migration | Schema 迁移 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1962 | PH-7 | 第 17-18 周 | admin | DB migration | 索引 + 约束 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1963 | PH-7 | 第 17-18 周 | admin | DB migration | 双向迁移演练 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1964 | PH-7 | 第 17-18 周 | admin | DB migration | 回滚预案 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1965 | PH-7 | 第 17-18 周 | admin | UT 单元测试 | testkit helper | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1966 | PH-7 | 第 17-18 周 | admin | UT 单元测试 | CRUD 覆盖 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1967 | PH-7 | 第 17-18 周 | admin | UT 单元测试 | 状态机覆盖 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1968 | PH-7 | 第 17-18 周 | admin | UT 单元测试 | 覆盖率报告 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1969 | PH-7 | 第 17-18 周 | admin | IT 集成测试 | service 启动 + health | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1970 | PH-7 | 第 17-18 周 | admin | IT 集成测试 | DB 集成 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1971 | PH-7 | 第 17-18 周 | admin | IT 集成测试 | 跨组件契约 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1972 | PH-7 | 第 17-18 周 | admin | IT 集成测试 | 端到端集成 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1973 | PH-7 | 第 17-18 周 | admin | ST 系统测试 | K8s 部署验证 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1974 | PH-7 | 第 17-18 周 | admin | ST 系统测试 | 性能 / 容量 NFR | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1975 | PH-7 | 第 17-18 周 | admin | ST 系统测试 | chaos 故障注入 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1976 | PH-7 | 第 17-18 周 | admin | ST 系统测试 | RPO/RTO 验证 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1977 | PH-7 | 第 17-18 周 | admin | Helm chart | Chart.yaml | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1978 | PH-7 | 第 17-18 周 | admin | Helm chart | values.yaml | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1979 | PH-7 | 第 17-18 周 | admin | Helm chart | deployment + HPA | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1980 | PH-7 | 第 17-18 周 | admin | Helm chart | NetworkPolicy | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1981 | PH-7 | 第 17-18 周 | admin | observability | OTel spans | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1982 | PH-7 | 第 17-18 周 | admin | observability | Prometheus metrics | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1983 | PH-7 | 第 17-18 周 | admin | observability | Grafana 仪表盘 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1984 | PH-7 | 第 17-18 周 | admin | observability | Loki 日志 | Admin 域 Lead（独立，不兼任 SRE） | _人·天 | _tokens | _ | _ | _ | _ |
| 1985 | PH-7 | 第 17-18 周 | cluster-ops | Control Plane API | ClusterOps gRPC 定义 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1986 | PH-7 | 第 17-18 周 | cluster-ops | Control Plane API | AdminService 转发 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1987 | PH-7 | 第 17-18 周 | cluster-ops | Control Plane API | request_id 幂等 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1988 | PH-7 | 第 17-18 周 | cluster-ops | Control Plane API | OCC 版本字段 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1989 | PH-7 | 第 17-18 周 | cluster-ops | CEM | Feature registry | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1990 | PH-7 | 第 17-18 周 | cluster-ops | CEM | 事件流 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1991 | PH-7 | 第 17-18 周 | cluster-ops | CEM | 订阅/取消订阅 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1992 | PH-7 | 第 17-18 周 | cluster-ops | CEM | DLQ 处理 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1993 | PH-7 | 第 17-18 周 | cluster-ops | PFAU | declared → canary → confirm → done 状态机 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1994 | PH-7 | 第 17-18 周 | cluster-ops | PFAU | all-reachable 确认 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1995 | PH-7 | 第 17-18 周 | cluster-ops | PFAU | 灰度策略 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1996 | PH-7 | 第 17-18 周 | cluster-ops | PFAU | 回滚路径 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1997 | PH-7 | 第 17-18 周 | cluster-ops | 状态机 | feature 状态定义 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1998 | PH-7 | 第 17-18 周 | cluster-ops | 状态机 | 非法转移检测 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 1999 | PH-7 | 第 17-18 周 | cluster-ops | 状态机 | 状态转移图 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 2000 | PH-7 | 第 17-18 周 | cluster-ops | 状态机 | 持久化方案 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 2001 | PH-7 | 第 17-18 周 | cluster-ops | RBAC | GM / COC / 客户端 3 套权限 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 2002 | PH-7 | 第 17-18 周 | cluster-ops | RBAC | 权限矩阵 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 2003 | PH-7 | 第 17-18 周 | cluster-ops | RBAC | 审计日志 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 2004 | PH-7 | 第 17-18 周 | cluster-ops | RBAC | 撤销机制 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 2005 | PH-7 | 第 17-18 周 | cluster-ops | fencing | 租约机制 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 2006 | PH-7 | 第 17-18 周 | cluster-ops | fencing | CAS 版本控制 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 2007 | PH-7 | 第 17-18 周 | cluster-ops | fencing | stale leader 检测 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 2008 | PH-7 | 第 17-18 周 | cluster-ops | fencing | 集群隔离策略 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 2009 | PH-7 | 第 17-18 周 | cluster-ops | 审计 | 审计 schema | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 2010 | PH-7 | 第 17-18 周 | cluster-ops | 审计 | 写入路径 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 2011 | PH-7 | 第 17-18 周 | cluster-ops | 审计 | 查询接口 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 2012 | PH-7 | 第 17-18 周 | cluster-ops | 审计 | 保留期 + 归档 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 2013 | PH-7 | 第 17-18 周 | cluster-ops | OCC | 乐观并发控制 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 2014 | PH-7 | 第 17-18 周 | cluster-ops | OCC | 重试策略 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 2015 | PH-7 | 第 17-18 周 | cluster-ops | OCC | 冲突检测 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 2016 | PH-7 | 第 17-18 周 | cluster-ops | OCC | 死锁恢复 | cluster-ops 域 Lead（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 2017 | PH-7 | 第 17-18 周 | shared-platform | Rust 工具链 | rustup 1.98 锁定 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 2018 | PH-7 | 第 17-18 周 | shared-platform | Rust 工具链 | rust-toolchain.toml | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 2019 | PH-7 | 第 17-18 周 | shared-platform | Rust 工具链 | CI cache | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 2020 | PH-7 | 第 17-18 周 | shared-platform | Rust 工具链 | 升级评审 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 2021 | PH-7 | 第 17-18 周 | shared-platform | Cargo.lock 锁定 | --locked 构建 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 2022 | PH-7 | 第 17-18 周 | shared-platform | Cargo.lock 锁定 | workspace 统一锁 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 2023 | PH-7 | 第 17-18 周 | shared-platform | Cargo.lock 锁定 | 依赖审计 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 2024 | PH-7 | 第 17-18 周 | shared-platform | Cargo.lock 锁定 | 更新策略 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 2025 | PH-7 | 第 17-18 周 | shared-platform | 镜像构建 | Dockerfile.distroless | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 2026 | PH-7 | 第 17-18 周 | shared-platform | 镜像构建 | 镜像大小优化 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 2027 | PH-7 | 第 17-18 周 | shared-platform | 镜像构建 | SBOM 生成 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 2028 | PH-7 | 第 17-18 周 | shared-platform | 镜像构建 | 漏洞扫描 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 2029 | PH-7 | 第 17-18 周 | shared-platform | K3s | K3s 集群初始化 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 2030 | PH-7 | 第 17-18 周 | shared-platform | K3s | kubectl 配置 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 2031 | PH-7 | 第 17-18 周 | shared-platform | K3s | NetworkPolicy | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 2032 | PH-7 | 第 17-18 周 | shared-platform | K3s | HPA / VPA | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 2033 | PH-7 | 第 17-18 周 | shared-platform | OTel Collector | 采集器配置 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 2034 | PH-7 | 第 17-18 周 | shared-platform | OTel Collector | OTLP 接收 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 2035 | PH-7 | 第 17-18 周 | shared-platform | OTel Collector | 采样策略 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 2036 | PH-7 | 第 17-18 周 | shared-platform | OTel Collector | 导出器配置 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 2037 | PH-7 | 第 17-18 周 | shared-platform | Helm | Chart 模板 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 2038 | PH-7 | 第 17-18 周 | shared-platform | Helm | values 校验 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 2039 | PH-7 | 第 17-18 周 | shared-platform | Helm | 依赖管理 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 2040 | PH-7 | 第 17-18 周 | shared-platform | Helm | CI render | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 2041 | PH-7 | 第 17-18 周 | shared-platform | 密钥 | Vault/OpenBao 部署 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 2042 | PH-7 | 第 17-18 周 | shared-platform | 密钥 | 密钥轮换策略 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 2043 | PH-7 | 第 17-18 周 | shared-platform | 密钥 | 运行时注入 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 2044 | PH-7 | 第 17-18 周 | shared-platform | 密钥 | 审计 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 2045 | PH-7 | 第 17-18 周 | shared-platform | 灾备 | 备份策略 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 2046 | PH-7 | 第 17-18 周 | shared-platform | 灾备 | 恢复演练 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 2047 | PH-7 | 第 17-18 周 | shared-platform | 灾备 | 跨 AZ 复制 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |
| 2048 | PH-7 | 第 17-18 周 | shared-platform | 灾备 | RTO/RPO 验证 | Platform Engineer（独立） | _人·天 | _tokens | _ | _ | _ | _ |

---

## v0.4 manual_addition（2026-08-21 手动追加，待脚本重生成时合入）

> **本节为手动追加**，`scripts/build_wbs_v02.py` 后续重生成 v0.4 时应将这些 L4 任务并入对应域/任务簇。v0.3 主体 2048 L4 + v0.4 增量 26 = **2074 L4**。

### 14 份新文档对应 L4（document_task，状态 = done）

| L4 # | PH | 窗口 | 域 | L3 任务簇 | L4 任务 | Owner | 人·天 | Tokens | 前置 | 验收 | 回滚 | 签字 | 文档编号 | 状态 |
|---:|---|---|---|---|---|---|---|---:|---:|---|---|---|---|---|
| 2049 | PH-0.5 | 第 1-2 周 | shared-platform | CDN 资源分发扩展 | 客户端断点续传 需求定义 | 架构师（兼） | _人·天 | _tokens | _ | RGS-REQ-036 §10~§12 验收 | _ | Ulysses(per DEC-008, see WBS-001 §17) | RGS-REQ-036 | done |
| 2050 | PH-0.5 | 第 1-2 周 | shared-platform | CDN 资源分发扩展 | 客户端断点续传 基本设计 | 架构师（兼） | _人·天 | _tokens | L4 #2049 | RGS-BAS-036 §12 验收 | _ | Ulysses(per DEC-008, see WBS-001 §17) | RGS-BAS-036 | done |
| 2051 | PH-0.5 | 第 1-2 周 | shared-platform | CDN 资源分发扩展 | 客户端断点续传 详细设计 | 架构师（兼） | _人·天 | _tokens | L4 #2050 | RGS-DTL-041 §12 验收 | _ | Ulysses(per DEC-008, see WBS-001 §17) | RGS-DTL-041 | done |
| 2052 | PH-0.5 | 第 1-2 周 | shared-platform | CDN 资源分发扩展 | 客户端断点续传 实现规格 | 架构师（兼） | _人·天 | _tokens | L4 #2051 | RGS-SPEC-DTL-041 §7 Definition of Done | _ | Ulysses(per DEC-008, see WBS-001 §17) | RGS-SPEC-DTL-041 | done |
| 2053 | PH-2 | 第 5-6 周 | shared-platform | CDN 资源分发扩展 | 客户端断点续传 单元测试 | 架构师（兼） | _人·天 | _tokens | L4 #2051 | RGS-TST-UT-04-ADD2 50 条用例 | _ | Ulysses(per DEC-008, see WBS-001 §17) | RGS-TST-UT-04-ADD2 | done |
| 2054 | PH-2 | 第 5-6 周 | shared-platform | CDN 资源分发扩展 | 客户端断点续传 系统测试 | 架构师（兼） | _人·天 | _tokens | L4 #2051 | RGS-TST-ST-04-ADD2 13 条用例 | _ | Ulysses(per DEC-008, see WBS-001 §17) | RGS-TST-ST-04-ADD2 | done |
| 2055 | PH-2 | 第 5-6 周 | shared-platform | CDN 资源分发扩展 | 客户端断点续传 集成测试 | 架构师（兼） | _人·天 | _tokens | L4 #2051 | RGS-TST-IT-04-ADD2 16 条用例 | _ | Ulysses(per DEC-008, see WBS-001 §17) | RGS-TST-IT-04-ADD2 | done |
| 2056 | PH-0.5 | 第 1-2 周 | admin | LCM 全生命周期 | 服务器全生命周期 需求定义 | Admin 域 Lead（独立） | _人·天 | _tokens | _ | RGS-REQ-037 §10~§12 验收 | _ | Ulysses(per DEC-008, see WBS-001 §17) | RGS-REQ-037 | done |
| 2057 | PH-0.5 | 第 1-2 周 | admin | LCM 全生命周期 | 服务器全生命周期 基本设计 | Admin 域 Lead（独立） | _人·天 | _tokens | L4 #2056 | RGS-BAS-037 §14 验收 | _ | Ulysses(per DEC-008, see WBS-001 §17) | RGS-BAS-037 | done |
| 2058 | PH-0.5 | 第 1-2 周 | admin | LCM 全生命周期 | 服务器全生命周期 详细设计 | Admin 域 Lead（独立） | _人·天 | _tokens | L4 #2057 | RGS-DTL-042 §12 验收 | _ | Ulysses(per DEC-008, see WBS-001 §17) | RGS-DTL-042 | done |
| 2059 | PH-0.5 | 第 1-2 周 | admin | LCM 全生命周期 | 服务器全生命周期 实现规格 | Admin 域 Lead（独立） | _人·天 | _tokens | L4 #2058 | RGS-SPEC-DTL-042 §7 Definition of Done | _ | Ulysses(per DEC-008, see WBS-001 §17) | RGS-SPEC-DTL-042 | done |
| 2060 | PH-2 | 第 5-6 周 | admin | LCM 全生命周期 | 服务器全生命周期 单元测试 | Admin 域 Lead（独立） | _人·天 | _tokens | L4 #2058 | RGS-TST-UT-02-ADD3 56 条用例 | _ | Ulysses(per DEC-008, see WBS-001 §17) | RGS-TST-UT-02-ADD3 | done |
| 2061 | PH-2 | 第 5-6 周 | admin | LCM 全生命周期 | 服务器全生命周期 系统测试 | Admin 域 Lead（独立） | _人·天 | _tokens | L4 #2058 | RGS-TST-ST-02-ADD3 15 条用例 | _ | Ulysses(per DEC-008, see WBS-001 §17) | RGS-TST-ST-02-ADD3 | done |
| 2062 | PH-2 | 第 5-6 周 | admin | LCM 全生命周期 | 服务器全生命周期 集成测试 | Admin 域 Lead（独立） | _人·天 | _tokens | L4 #2058 | RGS-TST-IT-02-ADD3 33 条用例 | _ | Ulysses(per DEC-008, see WBS-001 §17) | RGS-TST-IT-02-ADD3 | done |

### 12 项后续实施 L4（implementation_task，状态 = pending）

| L4 # | PH | 窗口 | 域 | L3 任务簇 | L4 任务 | Owner | 人·天 | Tokens | 前置 | 验收 | 回滚 | 签字 | 状态 |
|---:|---|---|---|---|---|---|---|---:|---:|---|---|---|---|
| 2063 | PH-3 | 第 7-9 周 | shared-platform | 客户端 SDK 编码 | rgs-asset-download crate 骨架（Cargo.toml + 模块目录 + 公开 API） | 架构师（兼） | _人·天 | _tokens | L4 #2051 + L4 #2052 | `cargo build -p rgs-asset-download` 通过 | `git revert` + 依赖移除 | Ulysses(per DEC-008, see WBS-001 §17) | pending |
| 2064 | PH-3 | 第 7-9 周 | shared-platform | 客户端 SDK 编码 | DownloadStateMachine + ResumeTokenStore（SQLite + JSON 原子写） | 架构师（兼） | _人·天 | _tokens | L4 #2063 | AC-CDN-110/111 + UT 50 条 PASS | feature flag 关闭 | Ulysses(per DEC-008, see WBS-001 §17) | pending |
| 2065 | PH-3 | 第 7-9 周 | shared-platform | 客户端 SDK 编码 | RangeClient + ChunkOrchestrator + IntegrityGate + 4 平台 pre-allocate | 架构师（兼） | _人·天 | _tokens | L4 #2063 | AC-CDN-112~118 + IT 16 条 PASS | feature flag 关闭 | Ulysses(per DEC-008, see WBS-001 §17) | pending |
| 2066 | PH-3 | 第 7-9 周 | admin | 6 阶段操作器编码 | RealmLifecycleService 6 操作器骨架（NewRealm/Split/Merge/Retire/Archive） | Admin 域 Lead（独立） | _人·天 | _tokens | L4 #2058 + L4 #2059 | `cargo build -p rgs-cluster-ops` 通过 + 6 阶段状态机 UT PASS | feature flag 关闭 | Ulysses(per DEC-008, see WBS-001 §17) | pending |
| 2067 | PH-3 | 第 7-9 周 | admin | Saga 编排 | SagaOrchestrator + 6 阶段 Saga 步骤实现（含反向补偿） | Admin 域 Lead（独立） | _人·天 | _tokens | L4 #2066 | UT 56 条 PASS + Saga 补偿 100% 正确 | 降级为单步手动执行 | Ulysses(per DEC-008, see WBS-001 §17) | pending |
| 2068 | PH-3 | 第 7-9 周 | admin | admin_db 迁移 | 6 张新表 migration 上线（Expand-Contract 双向演练） | DBA + Admin 域 Lead | _人·天 | _tokens | L4 #2058 | 双向迁移演练 100% 通过 + 索引 / 外键校验 | `sqlx migrate revert` | Ulysses(per DEC-008, see WBS-001 §17) | pending |
| 2069 | PH-4 | 第 9-12 周 | shared-platform | Range 实测 | MinIO 自托管 Range 行为 4 平台端到端实测（AC-CDN-110~118 9 项全通过） | SRE + 架构师 | _人·天 | _tokens | L4 #2065 | AC-CDN-110~118 全部 9 项 | 切回全量 GET fallback | Ulysses(per DEC-008, see WBS-001 §17) | pending |
| 2070 | PH-4 | 第 9-12 周 | admin | LCM 实测 | 6 阶段操作器演练环境 5 类各执行 1 次（AC-LCM-001~010 10 项全通过） | SRE + Admin Lead | _人·天 | _tokens | L4 #2067 + L4 #2068 | AC-LCM-001~010 全部 10 项 | 演练环境隔离无影响 | Ulysses(per DEC-008, see WBS-001 §17) | pending |
| 2071 | PH-4 | 第 9-12 周 | admin | LCM 集成 | ClusterOpsService `realm_lifecycle` Feature 7 子类 + OLU 上报集成 | Admin Lead | _人·天 | _tokens | L4 #2067 + L4 #2068 | 7 子类注册 + OLU 上报 100% 命中 | feature flag 关闭 | Ulysses(per DEC-008, see WBS-001 §17) | pending |
| 2072 | PH-5 | 第 12-14 周 | shared-platform | CDN 边缘集成 | 商业 CDN（Cloudflare 可选）Range 边缘命中实测 + 切流验证 | SRE | _人·天 | _tokens | L4 #2069 | 边缘命中 ≥ 80% + 切流 ≤ 30s | 关闭商业 CDN 走自托管 | Ulysses(per DEC-008, see WBS-001 §17) | pending |
| 2073 | PH-5 | 第 12-14 周 | admin | 跨域联动 | LCM 与业务 service gRPC 集成（player/economy/social）+ 退场后 RBAC 通道开启 | Admin Lead + 各域 Lead | _人·天 | _tokens | L4 #2071 | IT 33 条 PASS | LCM 关闭走人工 | Ulysses(per DEC-008, see WBS-001 §17) | pending |
| 2074 | PH-6 | 第 14-16 周 | admin | 归档实测 | 归档冷热分层 + N+2 冗余 + GDPR "被遗忘权"删除通路实测 | DBA + 法务 | _人·天 | _tokens | L4 #2071 | AC-LCM-005 通过 + GDPR 删除 100% 命中 | 保留现有归档策略 | Ulysses(per DEC-008, see WBS-001 §17) | pending |

### 合计

- v0.3 主体：2048 L4
- v0.4 manual_addition：26 L4（14 document_task done + 12 implementation_task pending）
- **总计：2074 L4**

### v0.4 manual_addition 责任分配（per DEC-008 一人公司 12 角色兼任）

| 责任域 | 任务数 | owner | 备注 |
|---|---|---|---|
| shared-platform（断点续传 7 项） | 7 | 架构师（兼） | 设计 4 + 实施 3 |
| admin（LCM 9 项 + 实施 9 项 + 1 实施）| 19 | Admin 域 Lead（独立） | 设计 4 + 实施 15 |
| **合计 v0.4** | 26 | — | — |
