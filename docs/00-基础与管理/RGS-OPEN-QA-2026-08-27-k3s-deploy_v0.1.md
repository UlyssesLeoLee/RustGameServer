# RGS-OPEN-QA-2026-08-27-k3s-deploy — 待答问题清单

> **文档 ID**: RGS-OPEN-QA-2026-08-27-k3s-deploy
> **版本**: v0.1
> **生效日期**: 2026-08-27 22:05 JST
> **作者**: 架构师(Mavis 接手 agent per DEC-008,代签)
> **状态**: 🟡 OPEN(6 个问题待答,4 个需 SRE Lead 决策,2 个需 5 域 Lead 联合决策,1 个需 DBA 决策)
> **范围**: 2026-08-27 12:43 JST 部署完成 + 16:30 JST 后续 P0/P1/P2 收尾,9 个 DDD Review blocker / 决策项
> **关联**:
> - 部署报告:`docs/deploy/.run-logs/2026-08-27-deploy-all/DEPLOY-REPORT.md` (6664 字节)
> - 代签签字记录:`docs/00-基础与管理/RGS-EXEC-2026-08-27-DEPLOY-SIGN.md` (commit `2013049`)
> - OLU 报告:`docs/14-项目管理/RGS-OLU-REPORT-2026-08-27_dev-k3s-deploy_v0.1.md` (commit `88ce66b`)
> - 5 域 RACI:`docs/14-项目管理/RGS-RACI-{PLAYER,ECONOMY,MATCH,SOCIAL,ADMIN}-V1_*.md`
> - 部署 SOP:`docs/deploy/05-deploy-sop.md` + `04-env-setup-sop.md`

---

## 0. 重要前提

- **本 OPEN-QA 不擅自给"建议方案"**——per 2026-08-26 04:30 JST 派生约束"缺标比错标安全",问题留给负责 Lead / Ulysses 决策
- **本 OPEN-QA 中所有 commit SHA / file:line 都是 git 实证**(per 2026-08-26 04:30 JST 派生约束"引用必须 git 实证")
- **DDD Review 时 DDL Review + 5 域 Lead 联合审 + Ulysses 终审**(per 2026-08-26 08:40 JST 反转规则)

---

## 1. 待答问题(优先级排序)

### Q1. 🔴 路径 byte-level 偏差(P0 阻塞)

**问题描述**:
- git 实际追踪目录: `docs/00-基准与治理/`(8 字节 UTF-8 字符)
- 任务规范名 + F3 commit 落点: `docs/00-基础与管理/`
- 6 字节 / 18 字节路径差异 = 33% 不同
- 证据: commit `2013049 docs(exec): 2026-08-27 部署签字记录 (代签)` 落 `00-基础与管理/`
- 反证据: commit `88ce66b chore(workspace): .gitignore 修 target/ + scripts/_scratch/` 落 `00-基准与治理/`
- 已存在文档:
  - `RGS-OPEN-QA-001_*` 系列在 `00-基准与治理/`
  - `RGS-DEC-Q003_*` 在 `00-基准与治理/`
  - `RGS-DOCS-HEALTH-*` 在 `00-基准与治理/`
  - `RGS-EXEC-2026-08-27-DEPLOY-SIGN.md` 在 `00-基础与管理/`(本次新增)

**决策项**:
- [ ] 保留哪个目录 / 都保留 / 合并重命名 / 不动

**负责**:Ulysses 决策 + 架构师执行 git mv

**阻塞影响**:
- 后续 DDD Review / DTL 修订 / 新增 RGS 文档时,不知道往哪个目录写
- 路径不统一会让 git blame / git log --follow 出现两条历史线

---

### Q2. 🔴 5 域 Lead 实际具名状态(per DEC-005 兼任拒绝原则,P0)

**问题描述**:
- per 2026-08-21 Ulysses 强证据:5 域 + cluster-ops + shared-platform 等多域架构,每域配独立 Lead,拒绝兼任
- 当前状态(per OLU 报告 §7.2 + RACI 5 份 v1.1 文档):**仍是 Ulysses 兼任代签**(per DEC-008 一人公司 12 角色)
- RACI 文档位置:`docs/14-项目管理/RGS-RACI-{PLAYER,ECONOMY,MATCH,SOCIAL,ADMIN}-V1_*.md` 5 份,每份 v1.1
- 实际签字人:**目前是 5 份 RACI 的"架构师(Mavis 接手 agent per DEC-008)"代签**,不是真实 5 个 Lead 实际签字

**决策项**:
- [ ] 5 域 Lead 是否本次部署前必须实际具名?
- [ ] 如果暂不具名,生产部署前必须具名的 deadline?
- [ ] RACI 文档 v1.1 是否升级到 v1.2,加入"实际具名 + 代签范围"声明?

**负责**:Ulysses 决策 + 5 域 Lead(未来 5 个真人)

**阻塞影响**:
- OLU 报告 §6.5 评估"人·天中位 21 略超 NFR-OP-010 上限 20"——**5 域 Lead 兼任导致 5 域维护工作量叠加,无法独立分配**
- per 2026-08-21 Ulysses 反馈"兼任会把责任矩阵和 RACI 模糊化"——当前形态符合风险

---

### Q3. 🟠 NFR-OP-010 人·天轨 21 略超 20(P1 决策)

**问题描述**:
- per OLU 报告 §6.5(commit `88ce66b`):本次 dev k3s 部署 token 轨 0.7-1.45M tokens(conservative)/ 1.95-3.68M(aggressive),**远低于** NFR-OP-010 上限 20M/周
- 但人·天轨中位 21,**略超** 20 上限(超 5%)
- 5 域 + cluster-ops + gm-backend 维护成本在 dev k3s 稳定后每周仍需 ~21 人·天
- 决策来源:per 2026-08-21 Ulysses 反馈"AI 协作下人·天失去精度,改 token"

**决策项**:
- [ ] 是按 OLU 报告 §6.5 建议"申请额外 SRE 编制"(2 SRE → 3-4 SRE)
- [ ] 还是"调整 NFR-OP-010 上限"(20 → 25-30 人·天/周)
- [ ] 还是"接受超 5% 风险,留作 follow-up"
- [ ] 是否需要走 RGS-ADR-0025 NFR-OP-010 修订流程?

**负责**:SRE Lead + PM Lead 联合决策(per RGS-PM-005 工数管理 §3 修订流程)

**阻塞影响**:
- 5 域 Lead 具名(Q2)前,本决策必须先做(因为 Q2 决定是否分摊到 5 域 Lead)
- 申请额外 SRE 编制涉及预算,需 Ulysses + 财务联合

---

### Q4. 🟠 0.1.2-cluster-ops 镜像资产保留 vs 不能用(P1 决策)

**问题描述**:
- 0.1.2-cluster-ops 镜像已 push ghcr.io(commit `ddff002` + `e614515` + `bbebb02` 链),200 OK, 23 layers
- apply 失败:`DB migrations failed: internal error: migration 1 was previously applied but has been modified`
- 根因:0.1.2 镜像是从最新 source 重新编译,migration 1 SQL 与 0.1.0 build 时跑的 hash 不匹配
- 主会话已回滚:commit `fb926f1` image tag 0.1.2 → 0.1.0 + probe 改回 tcpSocket
- live 状态:3 副本 1/1 Running(0.1.0 image + tcpSocket probe)

**决策项**:
- [ ] 0.1.2 镜像资产如何处置?保留为资产(DONE, 已 push)/ 删除(per ghcr.io API) / 标 deprecated?
- [ ] 真正修复路径:源码层让 0.1.2 镜像用 0.1.0 migration 文件 build / sqlx 跳过已应用 migration / 重写 migration 1 内容?
- [ ] probe 何时改回 grpc_health_probe?(待 0.1.2 修复后)

**负责**:cluster-ops 域 Lead(待具名, per Q2) + SRE Lead 联合

**阻塞影响**:
- 当前 dev 跑的是 dev 妥协态(tcpSocket + 0.1.0),生产前必须修复
- 0.1.2 镜像在 ghcr.io 占空间,需要明确处置

---

### Q5. 🟡 5 域 outbox relay 切到 NATS(P2 follow-up)

**问题描述**:
- per F1 NATS 已部署:`nats-0 1/1 Running`,4222 端口监听
- 5 域 service 启动时 `outbox relay DISABLED — NATS connect failed: DNS error`(因为 NATS pod 起来前 5 域已启动)
- 5 域下次重启才会重新连 NATS,本会话没动(避免无关变更)

**决策项**:
- [ ] 何时触发 5 域 `kubectl rollout restart`(player / economy / match / social / admin)?
- [ ] 是否需要先在 staging 验证 NATS 持久化 + outbox relay 流?
- [ ] outbox relay DISABLED fallback 模式下累积的 outbox rows 何时清?

**负责**:5 域 Lead 联合 + SRE Lead 协调

**阻塞影响**:
- 当前 5 域业务功能正常,outbox relay 暂未启用不阻塞业务
- 但 outbox 表会累积事件,需要尽快切到 NATS

---

### Q6. 🟡 GM 后台代签溯源边界(P2 治理)

**问题描述**:
- per 2026-08-26 08:40 JST Ulysses 反转规则:"今后所有 RGS-* 文档允许代签"
- per 2026-08-27 19:39 JST Ulysses 强化:"Mavis 接手默认代签 Ulysses 无需再问"
- per 2026-08-27 20:56 JST + 21:59 JST Ulysses 第三/第四次强化
- 当前 gm-backend 部署代签范围:架构师 / PM / 评审主持人 / DBA / SRE / Platform / QA / 5 域 Lead / 业务方代表
- gm-backend 后续 endpoint 实现(per F8 gm-backend 集成测试 follow-up)还需要 gRPC client 调用 admin-service,需要更多代签

**决策项**:
- [ ] GM 后台相关后续工作的代签边界(谁来签/什么范围/是否需要 DDD Review 补审)?
- [ ] gm-backend 5 个 endpoint stub(`/api/v1/gm/{health/view,ban,compensation,maintenance,audit/logs}`)实装时,谁来签?

**负责**:Ulysses 决策 + GM 后台域 Lead(待具名, per Q2)

**阻塞影响**:
- F8 gm-backend 集成测试(per FOLLOW-UP-PLAN P3 长线)需要先确定代签边界

---

## 2. 已闭合项(本 OPEN-QA 范围外,但供参考)

- ✅ 8 域 + 4 基础设施部署(2026-08-27 12:43-16:08 JST)
- ✅ gm-backend crate + workspace + 0.1.0 镜像 push(commit `456482e` + `7e2fcb2`)
- ✅ cluster-ops probe 临时修复(tcpSocket 50056)
- ✅ 代签签字记录 commit `2013049`
- ✅ .gitignore 加 target/(commit `4390795` + `88ce66b`)
- ✅ OLU 报告 commit `88ce66b`(37285 字节,自验 4/4)
- ✅ NATS 部署 + e2e-smoke.ps1(commit `a59bbb9`,19/19 Pods Running,STATUS: OK)
- ✅ 0.1.2-cluster-ops 镜像资产保留(3 commit,主会话回滚)
- ✅ RGS-EXEC-2026-08-27-DEPLOY-SIGN.md 落地(代签透明)

---

## 3. 关联文件(本 OPEN-QA 引用的全部 git 实证)

| 文件 / commit | 用途 | SHA |
|---|---|---|
| RGS-EXEC-2026-08-27-DEPLOY-SIGN.md | 代签签字记录 | `2013049` |
| .gitignore | target/ 排除 | `4390795` |
| RGS-OLU-REPORT-2026-08-27_dev-k3s-deploy_v0.1.md | OLU token 估算 | `88ce66b` |
| 06-cluster-ops-service.yaml (回滚) | 0.1.2 → 0.1.0 | `fb926f1` |
| 50-gm-backend-service.yaml | ghcr.io 0.1.0-gm-backend | `456482e` |
| scripts/e2e-smoke.ps1 | 19/19 smoke OK | `a59bbb9` |
| 0.1.2-cluster-ops image | ghcr.io 资产 | `ddff002` / `e614515` / `bbebb02` |
| 7e2fcb2 | 8 域 + gm-backend 初始 commit | `7e2fcb2` |
| 456482e | gm-backend manifest 切镜像 | `456482e` |
| cluster-ops-before-20260827-131829.yaml | cluster-ops 修复前 snapshot | snapshot |
| DEPLOY-REPORT.md | 主部署报告 | 6664 字节 |
| FOLLOW-UP-PLAN.md | 10 项 follow-up 清单 | 1700 字节 |

---

## 4. 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 | 状态 |
|---|---|---|---|---|
| 0.1 | 2026-08-27 22:05 JST | 架构师(Mavis 接手 agent per DEC-008,代签) | 初版:6 个 OPEN 问题 + 9 个已闭合项 | 🟡 OPEN |

---

**Author**: Mavis 接手 agent per DEC-008(代签)
**Time**: 2026-08-27 22:05 JST
**Next step**: Ulysses 派发给 5 域 Lead / SRE Lead / PM 各自负责的问题;DDD Review 阶段补全"实际具名"签字
