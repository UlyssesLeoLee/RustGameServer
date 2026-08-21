# 附件 C：责任矩阵与签字模板（详细版）

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-REV-006 |
| 版本 | 0.1（草稿）|
| 依据 | RGS-REV-003 §3 + RGS-HANDOFF-001 §4 Gate 矩阵 |

---

## §C.1 完整责任矩阵

> R = Responsible（执行）/ A = Accountable（最终责任）/ C = Consulted（咨询）/ I = Informed（知情）

### §C.1.1 架构类 Gate

| Gate | 责任人 | 角色分配 | RACI |
|---|---|---|---|
| G-CODE-02 (DTL-031 字段级) | **架构师** | 主评审 | R+A |
| | SRE Lead | 运行环境 / COC 接口 | C |
| | DBA | Schema 字段 | C |
| | Platform Engineer | 实施可行性 | C |
| | PM | 签字 | I |
| G-CODE-03 (ADR-0052 联审) | **架构师** | 容错哲学 | R+A |
| | SRE Lead | K8s 故障注入 | C |
| | Platform Engineer | 实施路径 | C |
| | PM | 风险接受 | I |

### §C.1.2 跨 DB 事务类 Gate

| Gate | 责任人 | 角色分配 | RACI |
|---|---|---|---|
| Q-003 / G-CODE-04 (Saga 真实场景) | **架构师** | Saga 方案 | R+A |
| | DBA | DB 事务 / Outbox | C |
| | **Economy 域 Lead** | Saga 跨域核心 | R（业务侧）|
| | Match 域 Lead | match 域步骤 | C |
| | Player 域 Lead（架构师兼任）| player 域步骤 | C |
| | Social 域 Lead | social 域步骤 | C |
| | SRE Lead | 监控 / 告警 | I |
| | PM | 签字 | I |

### §C.1.3 5 域 DTL 字段级 Review

| Gate | 责任人 | 角色分配 | RACI |
|---|---|---|---|
| G-CODE-02 (Player 域字段级) | **架构师**（兼任）| 实体表 / 索引 | R+A |
| | QA Lead | 测试覆盖 | C |
| | SRE Lead | 容量 | C |
| G-CODE-02 (Economy 域字段级) | **Economy 域 Lead** | 事务 / Outbox | R+A |
| | 架构师 | Saga 集成 | C |
| | DBA | Schema | C |
| | QA Lead | 测试 | C |
| G-CODE-02 (Match 域字段级) | **Match 域 Lead** | 状态机 / 算法 | R+A |
| | 架构师 | 跨域 | C |
| | SRE Lead | 性能 NFR-PT | C |
| | QA Lead | 性能测试 | C |
| G-CODE-02 (Social 域字段级) | **Social 域 Lead** | 消息 / 通知 | R+A |
| | 架构师 | 跨域 | C |
| | SRE Lead | 异步路径 | C |
| | QA Lead | 测试 | C |
| G-CODE-02 (Admin / COC 域字段级) | **SRE Lead** | COC 接口 | R+A |
| | 架构师 | ADR-0052 集成 | C |
| | DBA | Schema | C |
| | QA Lead | 测试 | C |

### §C.1.4 工具链类 Gate

| Gate | 责任人 | 角色分配 | RACI |
|---|---|---|---|
| G-CODE-06 (Rust 1.98 stable) | **Platform Engineer** | 工具链 | R+A |
| | DBA | PG 18.4 | C |
| | SRE Lead | K3s 集成 | C |
| | 架构师 | 跨工具链 | C |
| G-CODE-06 (PG 18.4 migration 演练) | **DBA** | 迁移演练 | R+A |
| | Platform Engineer | 工具链 | C |
| | SRE Lead | 部署集成 | C |
| G-CODE-06 (K3s 能力核验) | **SRE Lead** | 集群 | R+A |
| | Platform Engineer | 集成 | C |
| | DBA | 存储 | C |
| G-CODE-05 / G-CODE-07 (5 域依赖 + testkit) | **架构师** | 依赖图 | R+A |
| | 5 域 Lead | 各域 testkit 需求 | R（域侧）|
| | QA Lead | testkit 实现 | C |
| | SRE Lead | CI 集成 | C |

---

## §C.2 签字流程

### §C.2.1 签字前置

签字前必须：
- [ ] 评审 §1-§7 全部完成
- [ ] §6 异议登记表全部闭环或升级
- [ ] 附件 A / B / C 全部完成
- [ ] 附件 B 所有场景跑通
- [ ] 监控指标在 SLA 范围内

### §C.2.2 签字顺序

按依赖关系顺序签字（不可跳签）：

1. **DBA** 先签 G-CODE-06 PG 18.4 部分
2. **SRE Lead** 再签 G-CODE-06 K3s + G-CODE-02 Admin 域
3. **5 域 Lead** 各自签 G-CODE-02 域 DTL
4. **架构师** 签 G-CODE-02 DTL-031 + G-CODE-03 ADR-0052 + G-CODE-04 Saga + G-CODE-05 依赖
5. **Economy 域 Lead** 签 G-CODE-04 Saga 经济域
6. **Platform Engineer** 签 G-CODE-06 Rust 1.98
7. **PM** 最后总签字

### §C.2.3 签字格式

每个签字必须包含：
```
签字人：_______（手写签名 / 加密签名 / Git 签字均可）
日期：YYYY-MM-DD
角色：_______
Gate ID：_______
评审意见：__________
遗留事项：__________
```

---

## §C.3 异议处理流程

### §C.3.1 异议登记

- 在 RGS-REV-003 §6 异议登记表填入异议 ID
- 严重度分：🔴 Blocker / 🟠 重要 / 🟡 应当 / 🟢 Nice

### §C.3.2 闭环方式

| 闭环方式 | 适用 |
|---|---|
| **A. 文档修订** | 异议在 DTL/SPEC/ADR 范围内；修订后回 §4 重新走流程 |
| **B. ADR 修订** | 异议涉及架构决策；新增/修订 ADR 后回 §4 |
| **C. 升级为 NO-GO** | 第 2 轮仍未闭环；按 handoff §1 升为 NO-GO，53 不可启动 |

### §C.3.3 时限

- 🔴 Blocker：评审后 3 天内闭环
- 🟠 重要：评审后 7 天内闭环
- 🟡 应当：评审后 14 天内闭环
- 🟢 Nice：可在 Phase 1 内闭环，不阻塞 53

---

## §C.4 签字栏汇总

> 这是 RGS-REV-003 §7.3 的详细版。

### §C.4.1 Gate 签字

| Gate ID | 责任人 | 签字 | 日期 |
|---|---|---|---|
| G-CODE-02 (DTL-031 字段级) | 架构师 | | |
| G-CODE-02 (Player 域) | 架构师（兼任）| | |
| G-CODE-02 (Economy 域) | Economy 域 Lead | | |
| G-CODE-02 (Match 域) | Match 域 Lead | | |
| G-CODE-02 (Social 域) | Social 域 Lead | | |
| G-CODE-02 (Admin / COC 域) | SRE Lead（兼任）| | |
| G-CODE-03 (ADR-0052 联审) | 架构师 | | |
| G-CODE-04 (Saga 真实场景) | 架构师 + DBA + Economy 域 Lead | | |
| G-CODE-05 (5 域依赖图) | 架构师 | | |
| G-CODE-06 (Rust 1.98) | Platform Engineer | | |
| G-CODE-06 (PG 18.4) | DBA | | |
| G-CODE-06 (K3s) | SRE Lead | | |
| G-CODE-07 (testkit) | 架构师 + QA Lead | | |

### §C.4.2 总签字

```
评审总签字：
签字人：________________（PM）
日期：________________
意见：________________________________________________________
__________________________________________________________________
```

---

> 本附件是 RGS-REV-003 的执行手册。签字栏空白是预期状态——签字时由责任人填入。
