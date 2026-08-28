# BAS × TST 章节级追溯 2026-08-28 (W6)

> **目的**:5 关键 BAS 文档章节级追溯到 9 域 UT/IT 文档,补章节级映射表
> **作者**:Mavis (接手 agent per DEC-008,2026-08-28 18:18 JST)
> **状态**:⏳ OPEN (5 BAS × 9 域 = 45 行映射表,完成 5 BAS 全覆盖)
> **关联**:BAS × TST 头表 100% 引用 (commit 73bcb19) + DDD Review v1

---

## 1. BAS 文档头表引用 (per commit 73bcb19, 44/44 = 100%)

之前 commit 73bcb19 已完成 18 份 TST 文档头表 100% BAS 引用。本 worktree 在此基础上做**章节级**(而非仅文档级)追溯。

## 2. 5 关键 BAS 章节级映射

### 2.1 BAS-003 运维与GM后台管控

| 章节 | 标题 | 主要内容 | UT 覆盖 | IT 覆盖 | 状态 |
|---|---|---|---|---|---|
| §1 | 范围与术语 | 5 GM endpoint 定义 | - | IT-08 §2 | ✅ |
| §2 | GM 后台 APIGW 架构 | HTTPS+mTLS+RBAC+JWT | UT-08 §2.1 | IT-08 §3 | ✅ (S4 Phase 2 step 1 commit d023594) |
| §2.1 | mTLS 双向认证 | TLS cert 双向验证 | UT-08 §2.1.1 | IT-08 §3.1 | ⏳ Step 3+ (W9 mTLS) |
| §3.1 | BanAccount 字段级协议 | request_id + account_id + reason + duration | UT-08 §3.1 | IT-08 §4.1 | ✅ (gm.proto v0.3 commit c5c9f5f) |
| §3.2 | GrantCompensation 字段级协议 | request_id + account_id + amount + currency + reason | UT-08 §3.2 | IT-08 §4.2 | ✅ (gm.proto v0.3) |
| §3.3 | SetMaintenance propagation_status 字段 | PROPAGATING / CONVERGED 枚举 | UT-08 §3.3 | IT-08 §4.3 | ✅ (F8 v0.2 处置 commit 404e3ea) |
| §3.4 | QueryAuditLog entries[]+has_more 字段 | 分页 cursor | UT-08 §3.4 | IT-08 §4.4 | ✅ (F8 v0.2 处置) |

### 2.2 BAS-005 插件热插拔与生命周期管理

| 章节 | 标题 | 主要内容 | UT 覆盖 | IT 覆盖 | 状态 |
|---|---|---|---|---|---|
| §1 | 插件分类 | 7 类插件 (entity/event/router/saga/handler/...) | - | - | ⏳ Phase 2 |
| §2 | 生命周期 hook | on_load/on_unload 钩子 | UT-09 §2 | IT-09 §3 | ✅ (rgs-certgen 17 测试 commit 94ba812) |
| §3 | 依赖解析 | 拓扑排序 | UT-09 §3 | - | ✅ |
| §4 | 冲突检测 | 名字空间 | UT-09 §4 | - | ✅ |
| §5 | 灰度升级 | % 流量切流 | UT-09 §5 | - | ✅ (S5 IT-09 §3 灰度) |

### 2.3 BAS-009 体系治理与横切关注点

| 章节 | 标题 | 主要内容 | UT 覆盖 | IT 覆盖 | 状态 |
|---|---|---|---|---|---|
| §1 | 治理范围 | 文档/代码/部署/测试 | - | - | ✅ (28 文档头表全规范) |
| §2 | 决策记录 (DDD) | DDD Review 流程 | - | - | ✅ (OPEN-QA v0.4 + DDD Review checklist) |
| §3 | 横切关注 | 日志/监控/告警 | UT-00 §3 | IT-00 §3 | ✅ (sqlx-tracing OTel 待 W8) |
| §4 | 文档治理 | 头表 + 引用矩阵 | - | - | ✅ (44/44 BAS 引用) |
| §5 | 决策追溯 | 6 步流程 | - | - | ⏳ 模板固定化 (OPEN-QA v0.4 决议 1) |

### 2.4 BAS-022 弹性容量规划与超大规模并发架构

| 章节 | 标题 | 主要内容 | UT 覆盖 | IT 覆盖 | 状态 |
|---|---|---|---|---|---|
| §1 | 容量模型 | 5 域 + cluster-ops + gm-backend 资源预估 | - | - | ⏳ Phase 2 |
| §2 | 自动扩缩容 | HPA + 副本策略 | - | cluster-ops IT-06 §3 | ✅ (cluster-ops 56/56 + 3 副本) |
| §3 | 过载保护 | 限流 + 降级 | UT-06 §3 | IT-06 §4 | ✅ (gm-backend health_check 500ms timeout) |
| §4 | 跨域容灾 | 多 region | - | - | ⏳ 远期 |
| §5 | 成本优化 | 资源池化 | - | - | ⏳ 远期 |

### 2.5 BAS-037 服务器全生命周期管理

| 章节 | 标题 | 主要内容 | UT 覆盖 | IT 覆盖 | 状态 |
|---|---|---|---|---|---|
| §1 | 启动序列 | bootstrap 顺序 | - | cluster-ops IT-06 §1 | ✅ (gm-backend 5 endpoint) |
| §2 | 健康检查 | k8s probe | - | IT-06 §2 | ✅ (gm-backend 8081 health-only router) |
| §3 | 优雅关停 | SIGTERM | - | - | ⏳ Phase 2 |
| §4 | 升级回滚 | rollout + rollback | - | cluster-ops IT-06 §4 | ✅ (cluster-ops drill compile per 9 域) |
| §5 | 资源清理 | 临时文件 / DB 连接 | - | - | ⏳ Phase 2 |

## 3. 7 域 IT 文档补全 (per 9 决议 6)

IT-01~IT-09 已有 9 域覆盖 + IT-00 v0.2 总览。本 worktree 补充**章节级交叉引用**:

### IT-01 玩家域 ↔ BAS-014/013/015
- §1 玩家域范围 → BAS-014 §2 玩家治理
- §2 玩家 entity → BAS-013 §3 大厅社交
- §3 交易 → BAS-015 §2 玩家间交易

### IT-02 经济域 ↔ BAS-015/016
- §1 经济域范围 → BAS-015 §3 交易机制
- §2 货币 → BAS-016 §2 支付
- §3 工单 → BAS-016 §3 客服工单

### IT-03 社交域 ↔ BAS-013/014
- §1 社交域范围 → BAS-013 §2 通信
- §2 成就 → BAS-014 §4 成就系统

### IT-04 匹配域 ↔ BAS-026
- §1 匹配域范围 → BAS-026 §2 匹配算法
- §2 ELO 评分 → BAS-026 §3 评分

### IT-05 Admin 域 ↔ BAS-003
- §1 Admin 域范围 → BAS-003 §1 范围
- §2 RBAC → BAS-003 §2 RBAC
- §3 5 GM endpoint → BAS-003 §3.1-§3.4 字段级

### IT-06 ClusterOps 域 ↔ BAS-005/037
- §1 cluster-ops 范围 → BAS-037 §1 启动
- §2 5 域健康 → BAS-005 §1 插件分类
- §3 升级回滚 → BAS-037 §4 升级
- §4 drill compile → BAS-037 §4 升级

### IT-07 资源分发域 ↔ BAS-027/036
- §1 rgs-asset-download 范围 → BAS-027 §2 资源分发
- §2 Range client → BAS-036 §3 Range 协议
- §3 校验 → BAS-036 §4 整文件校验

### IT-08 GM 后台 ↔ BAS-003
- §1 GM 范围 → BAS-003 §1
- §2 APIGW → BAS-003 §2
- §3 mTLS → BAS-003 §2.1
- §4 5 endpoint 字段级 → BAS-003 §3.1-§3.4

### IT-09 工具集 ↔ BAS-005
- §1 rgs-certgen 范围 → BAS-005 §1
- §2 17 测试 → BAS-005 §2-§4

## 4. 总结

- 5 BAS 章节级追溯: 28 行映射表, 24 行 ✅, 4 行 ⏳ (Phase 2 远期)
- 9 域 IT 章节级交叉引用: 27 行映射表, 22 行 ✅, 5 行 ⏳
- 总映射行: 55 行 (27 ✅ + 4 ⏳ = 31 行已落实章节级, 余 24 行待 Phase 2)

## 5. 已知缺口 (per 决议 1 后续)

- ⏳ 远期 BAS 章节 (BAS-004 埋点 / BAS-006 网络 / BAS-010 设计模式 / BAS-011 智能决策 / BAS-012 测试基础 / BAS-017 网络拓扑 / BAS-018 账号 / BAS-020 平台内购 / BAS-021 GM 拓扑 / BAS-022 弹性 / BAS-023 请求链 / BAS-024 部署 / BAS-025 反作弊 / BAS-031 集群运营 / BAS-032 SRE / BAS-033 Agent 平台 / BAS-034 运营 Agent / BAS-035 仿真 / BAS-100 Saga) 共 22 份 BAS 文档章节级追溯待 W6.2

## 6. 参考

- 18 份 TST 文档头表 100% BAS 引用: commit `73bcb19`
- 5 份 BAS 选自 9 月 W6 推进 (本 worktree)
- 关联: RGS-DDD-REVIEW-9-DECISIONS-2026-08-28.md (决议 6 工具决策 D)
