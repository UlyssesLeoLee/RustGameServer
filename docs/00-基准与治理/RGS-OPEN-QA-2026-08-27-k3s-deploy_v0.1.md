# RGS-OPEN-QA-2026-08-27-k3s-deploy — 待答问题清单

> **文档 ID**: RGS-OPEN-QA-2026-08-27-k3s-deploy
> **版本**: v0.3
> **生效日期**: 2026-08-27 22:05 JST(v0.3 修订 2026-08-28 09:30 JST)
> **作者**: 架构师(Mavis 接手 agent per DEC-008,代签)
> **状态**: 🟡 OPEN(Q1/Q3/Q4/Q5/Q6 已决策/已执行;**Q2 已出 8 域 Lead 具名草案待终审**;**Q7 已出 cluster-ops 终方案决策草案 + 3 子决策待终审**;详见各条 + §4 修订历史)
> **范围**: 2026-08-27 12:43 JST 部署完成 + 16:30 JST 后续 P0/P1/P2 收尾,9 个 DDD Review blocker / 决策项
> **关联**:
> - 部署报告:`docs/deploy/.run-logs/2026-08-27-deploy-all/DEPLOY-REPORT.md` (6664 字节)
> - 代签签字记录:`docs/00-基准与治理/RGS-EXEC-2026-08-27-DEPLOY-SIGN.md`(commit `2013049` 落 `00-基础与管理/`,per Q1 决议已 merge 回 `00-基准与治理/`)
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
- [x] 保留哪个目录 / 都保留 / 合并重命名 / 不动 → **保留 `00-基准与治理/`,其余合并进去**

**决议**(2026-08-27,Ulysses per DEC-008 代签):
- `docs/README.md` 总索引第 11 行硬编码 `00-基准与治理/` 为基准分区入口,下辖 RGS-REQ/BAS/DTL/REV/HANDOFF 等 31 个文件;`00-基础与管理/` 只有本次新增的 2 个文件,系 commit `2013049` 建目录时误输入(治理→管理)
- 额外发现第三个近似目录 `00-基本与治理/`(仅含 `reviews/phase-0-5-citation-sweep/` 2 份引用扫雷报告),同属命名漂移,一并处理
- **已执行**:`git mv` 合并 3 处误落文件回 `00-基准与治理/`(本 commit),未新建分区
- **未处理(超出本次授权范围)**:`docs/12-工作流/RGS-IMPL-PLAN-{CDN,LCM}-001_*.md`、`docs/deploy/cdn-cloudflare-report.md`、`docs/deploy/cdn-it-report.md` 等文档中存在多处 `../00-基本与治理/` 裸目录链接(未指向具体文件,疑似占位符从未补全)——这是独立于本次路径合并的既有引用缺陷,建议另开 OPEN-QA 或纳入下次 DOCS-HEALTH 扫描,不在本次范围内修改

**负责**:Ulysses 决策 + 架构师执行 git mv

**阻塞影响**:
- ~~后续 DDD Review / DTL 修订 / 新增 RGS 文档时,不知道往哪个目录写~~ 已解除:统一写 `00-基准与治理/`
- ~~路径不统一会让 git blame / git log --follow 出现两条历史线~~ 已解除(git mv 保留 rename 历史)

---

### Q2. 🔴 5 域 Lead 实际具名状态(per DEC-005 兼任拒绝原则,P0)

**问题描述**:
- per 2026-08-21 Ulysses 强证据:5 域 + cluster-ops + shared-platform 等多域架构,每域配独立 Lead,拒绝兼任
- 当前状态(per OLU 报告 §7.2 + RACI 5 份 v1.1 文档):**仍是 Ulysses 兼任代签**(per DEC-008 一人公司 12 角色)
- RACI 文档位置:`docs/14-项目管理/RGS-RACI-{PLAYER,ECONOMY,MATCH,SOCIAL,ADMIN}-V1_*.md` 5 份,每份 v1.1
- 实际签字人:**目前是 5 份 RACI 的"架构师(Mavis 接手 agent per DEC-008)"代签**,不是真实 5 个 Lead 实际签字

**决策项**:
- [x] 5 域 Lead 是否本次部署前必须实际具名? → **dev 阶段不阻塞,生产部署前必须**
- [x] 如果暂不具名,生产部署前必须具名的 deadline? → **挂在生产部署 checklist 的必过 gate 项,不设独立日期**
- [x] RACI 文档 v1.1 是否升级到 v1.2,加入"实际具名 + 代签范围"声明? → **是,待 5 域 Lead 具名或下次 RACI 修订窗口时一并做**

**决议**(2026-08-27,Ulysses per DEC-008 代签):dev k3s 部署阶段兼任代签可接受,但生产部署 gate 前必须 5 域 Lead 实际具名,列为硬性 checklist 项(而非软性 deadline);RACI v1.2 升级与具名同批次做,不单独立即执行(避免为空壳字段折腾文档版本)

**v0.3 追加决议**(2026-08-28 09:30 JST,Mavis 接手 agent per DEC-008 代签):
- 8 域 Lead 角色映射草案已落档:`docs/00-基准与治理/RGS-LEAD-NAMING-8-域-2026-08-28.md`
- 8 域(5 域 + cluster-ops + gm-backend + 工具集)各自独立角色,共享支持 4 角色(SRE/Platform/QA/PM) + 架构师 = 12 角色 (per DEC-008 一人公司 12 角色)
- 一人公司 12 角色 ↔ 8 域 Lead 数量合理(8 域 + 4 共享 = 12)
- 待 Ulysses 终审 §1 角色映射 + §1.5 共享支持 + RACI v1.2 升级窗口
- OLU §6.5 重算(预计人·天 21 → 16-18 per 8 域分配) — 8 域 Lead 具名后,工作量可分配到各域
- 关闭条件:8 域 Lead 实际具名 + RACI v1.2 升级 + OLU §6.5 重算 → Q2 可关闭

**v0.4 终审决议**(2026-08-28 10:33 JST,Ulysses 一审):
- ✅ **采纳** 8 域 + 4 共享 = 12 角色映射(per Ulysses 决策)
- 8 域 Lead 角色具名(per DEC-008 一人公司 12 角色):
  - **player-service**:玩家域 Lead(per DTL-015)
  - **economy-service**:经济域 Lead(per DTL-018/037)
  - **match-service**:对战域 Lead(per DTL-026/038)
  - **social-service**:社交域 Lead(per DTL-019/020/039)
  - **admin-service**:Admin 域 Lead(per DTL-031)
  - **cluster-ops**:集群运营 Lead(per DTL-042)
  - **gm-backend**:GM 后台域 Lead(per BAS-003)
  - **rgs-certgen**:工具链 Lead(per 09 编号域)
- 共享支持 4 角色(SRE/Platform/QA/PM)+ 架构师
- 代签透明:author=Ulysses / 审批=架构师(Mavis 接手 agent per DEC-008)+自审+日期
- 关闭条件:8 域 Lead 实际具名 + RACI v1.2 升级 + OLU §6.5 重算 → Q2 可关闭

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
- [x] 还是"接受超 5% 风险,留作 follow-up" → **采纳**
- [ ] 是否需要走 RGS-ADR-0025 NFR-OP-010 修订流程? → **暂不需要**

**决议**(2026-08-27,Ulysses per DEC-008 代签):超幅仅 5%,且 token 轨远低于上限,更可能是"人·天"指标测量噪声(per 2026-08-21 Ulysses 反馈"AI 协作下人·天失去精度")而非真实产能缺口;现在申请编制或走 ADR 修订流程成本/收益比不划算。标记 follow-up,等 Q2(5 域 Lead 具名)落地、有真实分域数据后再复核

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

**根因复核**(2026-08-27,架构师 per DEC-008 代签,**仍 OPEN,未决策,未执行任何镜像构建/推送**):
- 原文档"0.1.2 镜像从最新 source 重新编译"这句话不准确。实际链路(git 实证):`ddff002`(17:54)新建 `build-cluster-ops-0.1.2.yml`,设计上**不跑 cargo build**,而是从 base image `COPY --from` 复用二进制(commit 注释原话:"避免 LF 迁移内容与 DB CRLF hash 冲突"——说明作者当时已知有 CRLF/LF 风险);`e614515`(17:55)因 `0.1.0-cluster-ops` tag 在 ghcr.io 不存在,把 base image 改成通用 tag `0.1.0`;`bbebb02`(17:57)据此部署 0.1.2
- 关键新证据:`fb926f1` 回滚时用的是 `0.1.0-cluster-ops` 这个 tag(而非 `0.1.0`),且回滚后 3 副本正常 Running——说明当前真正能通过 migration 校验的是 `0.1.0-cluster-ops`,而 0.1.2 构建用的 base 是**另一个** tag(`0.1.0`),二者是否字节等价未经验证
- 环境层面确认存在真实 CRLF/LF 差异风险源:本机 git 系统级配置 `core.autocrlf=true`(Git for Windows 默认),本仓库 `.git/config` 局部覆盖为 `false`——即任何在没有这条局部覆盖的环境(例如另一台机器 clone、或某次 CI/本地构建未继承该覆盖)签出 `crates/cluster-ops/migrations/*.sql` 都可能得到 CRLF 内容,而 git blob 本身是纯 LF(已用 `git show HEAD:...` 核实);sqlx 的 migration checksum 是按文件字节算的,CRLF/LF 差一个字节就会导致 hash 不匹配
- **结论**:根因大概率是"哪次构建的签出环境决定了 CRLF/LF、进而决定了 sqlx checksum",而不是简单的"用了新 source"。真正修复前需要先确认:(a) 当前活着的 `0.1.0-cluster-ops` 镜像里的二进制具体来自哪次构建/哪个签出环境,(b) `0.1.0` 通用 tag 与 `0.1.0-cluster-ops` 是否字节相同。这两点未查清前,**不建议**基于现有任何 workflow 重新 build/push,以免再次产生 hash 不匹配

**决策项**:
- [ ] 0.1.2 镜像资产如何处置?→ **建议标 deprecated + 附上面根因复核说明,不建议删除**(占用空间可忽略,删除 ghcr.io 包不可逆)
- [ ] 真正修复路径:待上面 (a)(b) 两点查清后再定,不要在不确定 base image 溯源前重新编译
- [ ] probe 何时改回 grpc_health_probe?(待镜像根因修复后)

**负责**:cluster-ops 域 Lead(待具名, per Q2) + SRE Lead 联合

**阻塞影响**:
- 当前 dev 跑的是 dev 妥协态(tcpSocket + 0.1.0-cluster-ops),生产前必须修复
- 0.1.2 镜像在 ghcr.io 占空间,需要明确处置(建议:标 deprecated,不删除)

---

### Q5. 🟡 5 域 outbox relay 切到 NATS(P2 follow-up)

**问题描述**:
- per F1 NATS 已部署:`nats-0 1/1 Running`,4222 端口监听
- 5 域 service 启动时 `outbox relay DISABLED — NATS connect failed: DNS error`(因为 NATS pod 起来前 5 域已启动)
- 5 域下次重启才会重新连 NATS,本会话没动(避免无关变更)

**决策项**:
- [x] 何时触发 5 域 `kubectl rollout restart`(player / economy / match / social / admin)? → **立即,串行执行**
- [x] 是否需要先在 staging 验证 NATS 持久化 + outbox relay 流? → **不需要**(dev 环境,NATS 已跑通,风险低)
- [ ] outbox relay DISABLED fallback 模式下累积的 outbox rows 何时清?→ 待重启后评估累积量再定

**决议**(2026-08-27,Ulysses per DEC-008 代签):执行前核实前提 —— `svc/nats` ClusterIP 4222 端口与 5 域 manifest 里配置的 `nats://nats:4222` 一致(排除配置错配的可能);`nats-0` 实际只运行 153 分钟,而 5 域 pod 已运行 13-23 小时,player-service 日志确认 `outbox relay DISABLED — NATS connect failed: DNS error` 至今未恢复 —— 确认是启动顺序竞态,不是配置问题,可以安全重启。

**执行记录(2026-08-27,已做,发现新阻塞,🔴 未解决)**:
- 5 域已串行 `kubectl rollout restart`(player→economy→match→social→admin),全部 rollout 成功,新 pod 均 1/1 Running
- 重启后错误从 `DNS error: failed to lookup address information` 变成 `IO error: Connection refused (os error 111)`——说明 DNS/服务发现已正常,但连接被拒绝
- 根因排查:`kubectl get networkpolicy -n rust-game-server` 显示只有 `allow-dns-and-api` / `default-deny-all` / `postgres-ingress` 三条,**没有 `nats-ingress`**——即 `docs/deploy/01-k8s-manifests/30-nats-networkpolicy.yaml` 从未被实际 apply 过,`default-deny-all`(podSelector 为空,Ingress+Egress 全拒绝)正在生效
- 且发现该 manifest 本身有两处会导致"apply 了也无效"的 bug:① podSelector 要求 nats pod 带 `app.kubernetes.io/component: message-bus` 标签,但活着的 `nats-0` 只有 `app.kubernetes.io/name=nats` + `part-of=rust-game-server`,没有这个标签,podSelector 匹配不到;② ingress 白名单只放行 `component: domain-service` 的 pod,但 `admin-service` 实际标签是 `component: coc-control-plane`,不会被放行
- **未执行**:未修改任何 NetworkPolicy / StatefulSet 标签(超出"重启"这一步授权范围,需要额外确认)
- **待决策**:(a) 给 `nats` StatefulSet 补 `component: message-bus` 标签,还是改 manifest podSelector 去匹配现有标签?(b) `nats-ingress` 规则要不要单独加一条放行 `component: coc-control-plane`(admin),还是把 admin 的 component 标签统一改成 `domain-service`?两个改动都要动 live 资源 + 源码 manifest,建议下一轮单独决策再执行

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
- [x] GM 后台相关后续工作的代签边界(谁来签/什么范围/是否需要 DDD Review 补审)? → **文档/部署/配置类可代签;跨域调用的业务逻辑首次上线需 DDD Review**
- [x] gm-backend 5 个 endpoint stub 实装时,谁来签? → **同上边界,stub→真实逻辑属于"业务逻辑首次上线",需过一次 DDD Review 才能代签合入**

**决议**(2026-08-27,Ulysses per DEC-008 代签):延续已三次强化的"Mavis 接手默认代签"规则,但划定边界 —— 代签覆盖文档 / 部署 / 配置类变更;凡涉及**写库或跨域 gRPC 调用的业务逻辑首次上线**(即 5 个 endpoint 从 stub 变真实实现),合入前仍需过一次 DDD Review,不纳入默认代签范围

**负责**:Ulysses 决策 + GM 后台域 Lead(待具名, per Q2)

**阻塞影响**:
- F8 gm-backend 集成测试(per FOLLOW-UP-PLAN P3 长线)需要先确定代签边界

---

### Q7. 🟡 cluster-ops/tests-disabled/ 4 ut_*.rs 旧债处置 + TBD-08-NN/TBD-09-NN 排期(P2 治理,2026-08-28 追加)

**问题描述**:
- per 2026-08-28 08:40 JST Ulysses "实施ut" 指令 + ut 实施批次,发现 2 类遗留:
  1. **cluster-ops/tests-disabled/ 4 ut_*.rs 旧债**(ut_feature_adapter / ut_olu / ut_saga / ut_state_machine)
     - 来源:commit `b74ccc3` (2026-08-27 08:00 JST) RGS-INC-002 v0.1 复盘,saga 编译死锁临时禁用
     - 现状:源码已搬至 `src/realm_lifecycle/`,旧测试 fn 引用旧路径,无法直接迁回
     - 决策记录:`crates/cluster-ops/tests-disabled/OLD-DEBT.md`(本次新增,3 处置方案候选)
  2. **TBD-08-NN (8 条) + TBD-09-NN (44 条) 实装排期**(per 2026-08-28 跨反馈 F7/F8 衍生 D4)
     - TBD-08-01~08:gm-backend 8 条(JWT / mTLS / gRPC client / audit_log / coverage / etc)
     - TBD-09-01 已关闭(本轮实装 17 黑盒),剩 TBD-09-02~04 + TBD-09-08(per UT-09 v0.2)

**决策项**:
- [ ] cluster-ops/tests-disabled/ 处置方案(迁回 tests/ / 移到 git 历史 / 保留 + 文档化) → **临时方案 C(保留 + 文档化)**,待 DDD Review 阶段决策
- [ ] TBD-08-NN + TBD-09-NN 排期(v0.2 / v0.3 / 长期) → **待 8 域 Lead 联合排期**

**决议**(2026-08-28,Mavis 接手 agent per DEC-008 代签临时方案):
- 临时采用方案 C(保留 + `OLD-DEBT.md` 文档化),不动 Cargo.toml,保持 `cargo build --tests -p cluster-ops` 0 error
- 跟踪到本 OPEN-QA Q7,DDD Review 阶段由 Ulysses + cluster-ops 域 Lead(per Q2 待具名)+ SRE Lead 联合决策 A/B/C 终方案
- TBD-08/09 排期依赖域 Lead 具名 + 8 域 Lead 联合协调,本轮仅关闭 TBD-09-01

**v0.3 追加决议**(2026-08-28 09:30 JST,Mavis 接手 agent per DEC-008 代签):
- TBD-08-01/02/04/05/07 + UT-08 模块 D 字段级 v0.2 已实装(per commit `404e3ea`)
- TBD-08-03 (admin-service gRPC client) 暂留 v0.3,v0.2 用 AuditStore trait 抽象 + InMemory 实现
- TBD-08-06 (axum-test vs wiremock 工具决策) 草案已落档 `RGS-TST-08-06-axum-test-vs-wiremock-工具决策.md`,方案 D (双工具并存) 短期推荐
- cluster-ops/tests-disabled/ 终方案决策草案已落档 `RGS-TST-CLUSTER-OPS-OLD-DEBT-终方案决策.md`,方案 A' (单文件 ut_state_machine 迁回, P3 follow-up 其余 3 文件) 推荐
- 8 域 Lead 具名草案已落档 `RGS-LEAD-NAMING-8-域-2026-08-28.md`(per Q2 解决)
- 待 Ulysses 终审 3 决策草案

**负责**:cluster-ops 域 Lead(待具名 per Q2)+ Ulysses 决策

**阻塞影响**:
- 方案 C 保留不删,0 阻塞,只是新增接手 agent 需先读 OLD-DEBT.md 才会知道 tests-disabled/ 不在 cargo test 范围
- TBD-08/09 排期延后会拖慢 v0.2 实装节奏

**关联 commit**:
- `94ba812` UT-09 rgs-certgen 17 黑盒实装 + 7 域 example + mock-registry
- `de86d80` 6 域独立 UT 文档 + evidence + 旧债决策
- `3c7d670` 核对报告 + test-evidence.ps1 v4
- `404e3ea` TBD-08-01~05/07 + UT-08 模块 D 字段级 v0.2 + match §4.1/§5 + social §3 + admin §4.2 PFAU
- TBD-08-03 (gRPC client) + TBD-08-06 (工具决策) 留 v0.3

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
| 0.1(原地追记) | 2026-08-27 | 架构师(Mavis 接手 agent per DEC-008,代签) | Q1:git mv 合并 3 处误落目录回 `00-基准与治理/`,已执行;Q2/Q3/Q6:记录决议(判断性,未改动其他文档);Q4:根因诊断修正(CRLF/LF hash 风险,非"重新编译"),仍 OPEN,未构建/推送镜像;Q5:5 域已串行重启,发现 NetworkPolicy 缺口(`nats-ingress` 从未 apply + manifest 自身 2 处标签 bug),仍 🔴 未解决 | 🟡 OPEN(Q4/Q5 待续) |
| 0.2 | 2026-08-28 | 架构师(Mavis 接手 agent per DEC-008,代签) | **Q7 新增**:per 2026-08-28 08:40 JST "实施ut" 指令 + ut 实施批次,发现 2 类遗留:① cluster-ops/tests-disabled/ 4 ut_*.rs 旧债(commit `b74ccc3` RGS-INC-002 复盘临时禁用,源码已搬至 `src/realm_lifecycle/`,决策记录 `OLD-DEBT.md`,临时方案 C 保留 + 文档化) ② TBD-08-NN (8 条) + TBD-09-NN (剩 3 条) 实装排期(per 2026-08-28 跨反馈 F7/F8 衍生 D4)。本批同时关闭 TBD-09-01(per UT-09 v0.2 实装 17/17 PASS) | 🟡 OPEN(Q7 临时方案 C,DDC Review 阶段决策 A/B/C 终方案 + TBD 排期) |
| 0.3 | 2026-08-28 09:30 JST | 架构师(Mavis 接手 agent per DEC-008,代签) | **Q2 v0.3 追加**:8 域 Lead 角色映射草案落档 `RGS-LEAD-NAMING-8-域-2026-08-28.md` (5 域 + cluster-ops + gm-backend + 工具集 = 8 域,共享支持 4 角色 SRE/Platform/QA/PM,总 12 角色 per DEC-008)。**Q7 v0.3 追加**:TBD-08-01/02/04/05/07 + UT-08 模块 D 字段级 v0.2 已实装(per commit `404e3ea`);TBD-08-03 暂留 v0.3 + TBD-08-06 工具决策草案落档 + cluster-ops 终方案决策草案落档(方案 A' 推荐)。**3 草案待 Ulysses 终审**。TBD-08-08 (gm-backend 域 Lead 具名) = Q2 解决(per RGS-LEAD-NAMING-8-域)。 | 🟡 OPEN(Q2/Q7 草案待终审,TBD-08-03/06 留 v0.3) |

---

**作者**:架构师(Mavis 接手 agent per DEC-008,代签)
**审批**:架构师(Mavis 接手 agent per DEC-008)+ 自审 + 2026-08-27 (v0.1) / 2026-08-28 09:30 JST (v0.3)
**修订人**:Ulysses(一人公司 12 角色 per DEC-008)— Mavis 接手

**Next step**: Ulysses 派发给 5 域 Lead / SRE Lead / PM 各自负责的问题;DDD Review 阶段补全"实际具名"签字
