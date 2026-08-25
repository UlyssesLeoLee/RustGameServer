# 详细设计书（詳細設計書 / Detailed Design Document）

**功能挂载架构：Helm/K8s模板落地・CI/CD流水线定义・Mount Record物理存储格式**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DTL-002 |
| 版本 | 0.3 |
| 父文档 | RGS-BAS-002 功能挂载架构 基本设计书（本文档为其详细化，不改变任何既有决定，仅将逻辑/结构设计落实为物理/实现级设计） |
| 依据标准 | IPA『共通フレーム 2013（SLCP-JCF2013）』详细设计工程 |
| 制定日 | 2026-08-17 |
| 制定者 | 架构师 |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响章节 |
|---|---|---|---|---|---|
| 0.1 | 2026-08-17 | 架构师 | — | 初版制定（负责人指示"按顺序和你的建议推进"，本文档为推荐的下一份详细设计文档，先于其余业务域是因为其余域挂载均依赖本文档确立的物理脚手架）。细化RGS-BAS-002§4.2 CI/CD流水线为具体流水线定义、§5.2 Helm chart模板文件列表为具体YAML内容、§10.1 Mount Record字段表为具体物理存储格式（Markdown frontmatter + Appendix C行）、§12检查清单为可脚本化的自动化检查逻辑。**本版本不覆盖**：实际admission-webhook实现代码、实际migration工具选型的完整对比评审（仅给出所选工具与用法）、跨云厂商Helm差异适配。见§7 | 全部 |
| 0.2 | 2026-08-17 | 架构师 | — | 负责人指示"详细设计应充分体现以热插拔为主的App集群的原子化低耦合高内聚特性，妥善调和回滚和生命周期幂等排他问题"。发现§6.4退场安全网与§4挂载CI流水线各自独立触发、互不感知对方状态，同一限界上下文的挂载与退场可被并发触发产生竞态——新增§6.5生命周期排他锁：Mount Record frontmatter新增`lifecycle_state`字段驱动显式状态机（MOUNTING/ACTIVE/DECOMMISSIONING/DECOMMISSIONED），CI新增`lifecycle-lock-check`前置阶段强制互斥，并阐明该机制与RGS-DTL-024§2集群级排他约束是同一类问题在不同物理介质（Git提交串行性 vs. 数据库唯一索引）上的对应实现 | §6.1、§6.5（新增）、§7 |
| 0.3 | 2026-08-25 | 架构师（Mavis 接手 agent per DEC-008）| — | **同步父 BAS-002 升版至 v0.3**（1 次升版，RGS-IMPL-001: virtual workspace/领域库 bin 分离/按域 versioned proto/migration 锁）: 本 DTL 是父 BAS 的详细化（per DTL 头部"不改变任何既有决定"），父 BAS 升版为元数据/追溯性表/装饰性修订，DTL-002 既有章节内容无实质重写，本升版仅做元数据层对齐;**正文本不重写**（per `RGS-DOCS-HEALTH-2026-08-25` §0 第 4 行"治理状态, 非文档缺陷, agent 不可代签" + 反馈单 §4 要求 1 "不预填任何 ✅, 不代签"）。 审批留空，待 Ulysses 在 review 时签发。 | (父 BAS 升版章节) |

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 | 2026-08-17 | — |
| 评审（技术） | | | YAML模板是否与RGS-BAS-002§5.2/§5.3逻辑设计一一对应，CI流水线阶段是否与§4.2表格一致 |
| 评审（SRE/运维） | | | NetworkPolicy default-deny规则是否覆盖全部既有依赖白名单，退场安全网（只读冻结）是否可脚本化执行 |
| 审批（负责人） | | | 本文档的基准化 |

---

## 目录

1. [前言](#1-前言)
2. [Helm Chart模板具体内容](#2-helm-chart模板具体内容)
3. [NetworkPolicy清单具体内容](#3-networkpolicy清单具体内容)
4. [CI/CD流水线定义](#4-cicd流水线定义)
5. [数据库开通脚本格式](#5-数据库开通脚本格式)
6. [Mount Record物理存储格式](#6-mount-record物理存储格式)
7. [本文档的覆盖范围与后续计划](#7-本文档的覆盖范围与后续计划)

---

## 1. 前言

### 1.1 定位

RGS-BAS-002（基本设计）回答"挂载一个新限界上下文需要哪些结构性要素、它们之间的依赖与顺序关系是什么"——它给出的是目录树、文件清单、决策表这一级别的逻辑设计。本文档（详细设计）回答"这些文件里具体写什么内容、流水线具体跑什么命令、Mount Record具体以什么格式落盘"——面向的是"照着抄就能跑起来"的实现级设计。

### 1.2 本文档不做什么

- 不重新决定RGS-BAS-002已确定的任何结构性选择（Cargo workspace分层、Helm base-template继承而非fork、NetworkPolicy default-deny、Mount Record字段清单）。若在细化过程中发现基本设计本身有缺陷，修正应回写RGS-BAS-002，不在本文档内悄悄改写。
- 不覆盖具体业务域（如ANT/MM/CDN）挂载时的业务专属K8s资源（如ANT的LangGraph分析图专属ConfigMap）——那些属于各业务域自身DTL文档的范围。
- 不做云厂商级别的Helm差异适配（本仓库遵循ARC-018"self-hosted优先"原则，模板以裸K8s/自托管K8s为准，托管云K8s的差异属于未来ADR范围）。

### 1.3 记述规则

- YAML模板中`{{ }}`标记为Helm模板变量，值来自`values.yaml`或Mount Record；`<CONTEXT>`等尖括号占位符表示挂载时替换的限界上下文名（小写连字符命名，如`anticheat`）。
- 所有示例以RGS-BAS-002已挂载的通用形态给出，具体业务域挂载时仅替换占位符，不应新增结构性字段（如需新增，先回写BAS-002）。

---

## 2. Helm Chart模板具体内容

对应RGS-BAS-002§5.2 Helm chart模板结构（当时仅给出文件清单）。以下为各文件的具体内容骨架。

### 2.1 `deployment.yaml`

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: {{ .Values.contextName }}-service
  labels:
    app: {{ .Values.contextName }}-service
    rgs.context: {{ .Values.contextName }}
    rgs.mountRecordRef: {{ .Values.mountRecordId }}
spec:
  replicas: {{ .Values.replicaCount | default 2 }}
  selector:
    matchLabels:
      app: {{ .Values.contextName }}-service
  template:
    metadata:
      labels:
        app: {{ .Values.contextName }}-service
    spec:
      serviceAccountName: {{ .Values.contextName }}-sa
      containers:
        - name: {{ .Values.contextName }}-service
          image: "{{ .Values.image.repository }}:{{ .Values.image.tag }}"
          ports:
            - containerPort: {{ .Values.grpcPort | default 50051 }}
              name: grpc
            - containerPort: {{ .Values.metricsPort | default 9090 }}
              name: metrics
          envFrom:
            - secretRef:
                name: {{ .Values.contextName }}-db-secret
          readinessProbe:
            grpc:
              port: {{ .Values.grpcPort | default 50051 }}
            initialDelaySeconds: 5
          livenessProbe:
            grpc:
              port: {{ .Values.grpcPort | default 50051 }}
            initialDelaySeconds: 15
          resources:
            requests: { cpu: "{{ .Values.resources.cpuRequest | default "250m" }}", memory: "{{ .Values.resources.memRequest | default "256Mi" }}" }
            limits:   { cpu: "{{ .Values.resources.cpuLimit | default "1" }}",     memory: "{{ .Values.resources.memLimit | default "512Mi" }}" }
```

`deployment.yaml`不为有状态形态（如SY/RT的场景Actor进程）使用，那类由RGS-BAS-002§5.1判定为StatefulSet的上下文改用`statefulset.yaml`（结构同上，增加`volumeClaimTemplates`与`serviceName`），此处不重复展开。

### 2.2 `networkpolicy.yaml`

见第3章（独立展开，因其是挂载安全性的核心，需要更细的说明）。

### 2.3 `servicemonitor.yaml`

```yaml
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: {{ .Values.contextName }}-service
  labels:
    rgs.context: {{ .Values.contextName }}
spec:
  selector:
    matchLabels:
      app: {{ .Values.contextName }}-service
  endpoints:
    - port: metrics
      interval: 15s
      path: /metrics
```

对应RGS-BAS-002§9可观测性接入设计——该章仅要求"接入既有Prometheus栈"，此模板是该要求的具体落地；不新增指标口径，指标本身的命名规范沿用RGS-BAS-007既有约定。

### 2.4 `secret-db.yaml`

不直接以明文YAML存储数据库凭据（避免Secret明文入库这一常见反模式）。改为`ExternalSecret`资源，指向自托管的Secret管理后端（与ARC-045一致的"默认自托管优先"原则，此处对应自托管的密钥管理方案，而非假定商业密钥管理SaaS）：

```yaml
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: {{ .Values.contextName }}-db-secret
spec:
  secretStoreRef:
    name: rgs-secret-store
    kind: ClusterSecretStore
  target:
    name: {{ .Values.contextName }}-db-secret
  data:
    - secretKey: DATABASE_URL
      remoteRef:
        key: {{ .Values.contextName }}/db-url
```

实际密钥值的写入发生在§5数据库开通脚本执行时（见第5章），不在Helm chart部署时创建，避免密钥流经CI日志。

---

## 3. NetworkPolicy清单具体内容

对应RGS-BAS-002§5.3 NetworkPolicy规则表（原表为"允许的依赖列表"这一逻辑层面）。以下将该表转译为具体K8s NetworkPolicy manifest，落实default-deny原则：

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: {{ .Values.contextName }}-default-deny
spec:
  podSelector:
    matchLabels:
      app: {{ .Values.contextName }}-service
  policyTypes: [Ingress, Egress]
  ingress: []   # 默认拒绝所有入站，下方allow-list叠加放行
  egress: []    # 默认拒绝所有出站，下方allow-list叠加放行
---
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: {{ .Values.contextName }}-allow-declared
spec:
  podSelector:
    matchLabels:
      app: {{ .Values.contextName }}-service
  policyTypes: [Ingress, Egress]
  ingress:
    {{- range .Values.allowedIngressFrom }}
    - from:
        - podSelector: { matchLabels: { app: {{ . }} } }
    {{- end }}
  egress:
    {{- range .Values.allowedEgressTo }}
    - to:
        - podSelector: { matchLabels: { app: {{ . }} } }
    {{- end }}
    - to:  # 数据库始终放行，来自Mount Record的DB字段
        - podSelector: { matchLabels: { app: {{ .Values.contextName }}-db } }
    - to:  # DNS始终放行
        - namespaceSelector: {}
      ports:
        - { protocol: UDP, port: 53 }
```

`.Values.allowedIngressFrom`／`.Values.allowedEgressTo`两个列表的取值直接来自Mount Record的"依赖"字段（见第6章），挂载时由挂载脚本自动从Mount Record读取并填入`values.yaml`，禁止手工在Helm values里额外新增未登记在Mount Record中的白名单项——这是RGS-BAS-002"依赖必须显式登记"原则在物理层面的强制点：CI流水线（见§4.2）应对`values.yaml`中的`allowedEgressTo`与Mount Record进行一致性校验，不一致则流水线失败。

---

## 4. CI/CD流水线定义

对应RGS-BAS-002§4.2流水线阶段表（原表为阶段名称+职责的逻辑层面）。以下给出可直接落地的GitHub Actions流水线骨架（与本仓库既有`.github/workflows/docs-consistency.yml`风格一致）：

```yaml
name: mount-<CONTEXT>-ci
on:
  pull_request:
    paths: [ 'services/<context>-service/**' ]
  push:
    branches: [ main ]
    paths: [ 'services/<context>-service/**' ]

jobs:
  boundary-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: 跨限界上下文use检查
        run: cargo deny check --manifest-path services/<context>-service/Cargo.toml
        # 对应RGS-BAS-002§4.1的#[deny]跨上下文引用禁令，在CI层面二次强制（不仅依赖编译期lint）

  test:
    needs: boundary-check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo test --manifest-path services/<context>-service/Cargo.toml

  lint-untrusted-input:
    needs: boundary-check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: unwrap/expect/indexing lint（对应RGS-BAS-006§7A.1，全上下文统一强制）
        run: cargo clippy --manifest-path services/<context>-service/Cargo.toml -- -D clippy::unwrap_used -D clippy::expect_used -D clippy::indexing_slicing

  networkpolicy-consistency:
    needs: boundary-check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: values.yaml allow-list 与 Mount Record 一致性校验
        run: scripts/check-mount-record-consistency.sh <context>
        # 见§6.3，防止手工新增未登记依赖白名单

  build-and-push:
    needs: [test, lint-untrusted-input, networkpolicy-consistency]
    if: github.ref == 'refs/heads/main'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: docker build -t <registry>/<context>-service:${{ github.sha }} services/<context>-service
      - run: docker push <registry>/<context>-service:${{ github.sha }}

  deploy:
    needs: build-and-push
    if: github.ref == 'refs/heads/main'
    runs-on: ubuntu-latest
    steps:
      - name: helm upgrade（滚动发布，不停机）
        run: helm upgrade --install <context>-service ./charts/<context>-service --set image.tag=${{ github.sha }} --atomic --timeout 5m
        # --atomic：失败自动回滚，对应RGS-BAS-009§5.5"流量回退 p99<10s"要求的CI侧保障之一
```

`boundary-check`阶段的`cargo deny check`需要仓库根`deny.toml`按限界上下文声明禁止的跨域`use`路径；该配置文件本身的具体规则集不属于本文档范围（属于Cargo workspace配置细节，随各上下文挂载时增量追加，不需要单独详细设计）。

---

## 5. 数据库开通脚本格式

对应RGS-BAS-002§6数据库开通设计（原设计为"需要一个独立DB、最小权限角色"这一逻辑要求）。具体开通通过一次性、幂等的SQL脚本执行，格式如下：

```sql
-- scripts/mount/<context>_db_provision.sql
-- 幂等：重复执行不报错、不重复创建
CREATE DATABASE IF NOT EXISTS <context>_db;

CREATE ROLE IF NOT EXISTS <context>_service_role LOGIN PASSWORD :'generated_password';
GRANT CONNECT ON DATABASE <context>_db TO <context>_service_role;

\c <context>_db
GRANT USAGE, CREATE ON SCHEMA public TO <context>_service_role;
-- 最小权限：仅授予该角色对自己上下文内表的权限，不授予对其他上下文DB的任何权限
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO <context>_service_role;
```

`:'generated_password'`由挂载脚本在执行前通过密钥管理后端（第2.4节的`ClusterSecretStore`）生成并写入，脚本本身不硬编码密码，避免密码进入版本库或CI日志。脚本执行后，挂载脚本随即将生成的连接串写入第2.4节的`ExternalSecret`对应的密钥后端条目（`{{ .Values.contextName }}/db-url`），使Pod启动时通过`ExternalSecret`同步取得。

表结构本身（`CREATE TABLE ...`）不属于本文档范围——那是各业务域自己的DTL文档职责（如`RGS-DTL-001`§2/§3对`player_db`/`economy_db`的DDL）；本文档只覆盖"DB与角色本身如何被创建"这一挂载脚手架共性部分。

---

## 6. Mount Record物理存储格式

对应RGS-BAS-002§10.1 Mount Record字段表（原表为逻辑字段清单：限界上下文名、需求ID、DB名、gRPC服务名、依赖、生产/消费事件、部署形态、完成日期、负责团队）与§10.2归档位置（`services/<context>-service/README.md` + Appendix C）。

### 6.1 物理格式：README.md frontmatter

Mount Record不是数据库表（挂载记录本质是文档治理产物，不是运行时查询对象，因此物理落地形式是结构化的Markdown frontmatter，而非DDL）：

```markdown
---
mount_record:
  context_name: <context>
  requirement_id: RGS-REQ-0xx
  db_name: <context>_db
  grpc_service: <Context>Service
  dependencies:
    ingress_from: [gateway-service, ...]
    egress_to: [admin-service, ...]
  events:
    produced: [<context>.xxx.created.v1]
    consumed: [player.banned.v1]
  deployment_form: Deployment   # 或 StatefulSet，取自RGS-BAS-002§5.1判定
  completed_at: 2026-08-17
  owning_team: <team-name>
  lifecycle_state: active       # v0.2新增：MOUNTING / ACTIVE / DECOMMISSIONING / DECOMMISSIONED，见§6.5排他锁
---

# <Context> Service

（服务说明正文……）
```

### 6.2 归档：Appendix C行

`services/<context>-service/README.md`的frontmatter是"事实来源"（source of truth），Appendix C（RGS-REQ-004）§7域注册表中的对应行是"索引/汇总视图"，两者字段一一对应，不重复维护独立数据——Appendix C行通过下方§6.3的脚本从frontmatter自动抽取生成草稿，而非人工手打两遍（避免RGS-README版本漂移那类"两处手动同步、必然漂移"的结构性弱点在Mount Record上重演）。

### 6.3 一致性校验脚本

```bash
#!/usr/bin/env bash
# scripts/check-mount-record-consistency.sh <context>
# 1. 解析 services/<context>-service/README.md 的 frontmatter
# 2. 提取 dependencies.egress_to，与 charts/<context>-service/values.yaml 的 allowedEgressTo 比对
# 3. 不一致（values.yaml多出未登记依赖，或Mount Record有依赖但values.yaml未放行）则退出码非0
# 4. 提取 Mount Record 全字段，与 Appendix C §7 对应行逐字段比对，不一致则退出码非0
```

该脚本是第4章CI流水线`networkpolicy-consistency`阶段调用的具体实现，也是RGS-BAS-002§12.1挂载检查清单中"依赖已在Mount Record登记"这一检查项的自动化落地（原检查清单为人工检查项，本文档将其中可脚本化的部分转为CI强制项；不可脚本化的部分如"负责团队已确认接手"仍保留为人工检查项，见§7）。

### 6.4 退场（decommission）安全网的物理落地

对应RGS-BAS-002§11.2退场安全网（"删除前只读冻结"）。物理落地为：退场脚本先将§2.1 Deployment的容器环境变量`READ_ONLY_MODE=true`滚动更新（触发应用层拒绝写请求，具体拒绝逻辑属各业务域自身实现范围），观察期满（RGS-BAS-002§11.1流程图规定的时长）后才执行DB删除与Helm release卸载；Mount Record的frontmatter在冻结时新增`decommission_started_at`字段，实际删除后整个README.md连同frontmatter一并移至`docs/09-归档/decommissioned/<context>-service.md`存档，不直接从仓库中物理删除记录本身（保留历史可追溯性，与Mount Record"归档位置"设计的初衷一致）。

### 6.5 生命周期排他锁（v0.2新增）

对应负责人指示"App集群应妥善调和回滚和生命周期幂等排他问题"——§4 CI流水线与§6.4退场流程各自独立触发，若同一限界上下文的挂载流水线与退场流水线被并发触发（如CI重放、误操作双击），会出现"挂载CI正在创建资源的同时，退场CI正在删除同一资源"的竞态，单看各自流程内部（Helm `--atomic`回滚、只读冻结观察期）均是幂等的，但**跨流程**的互斥此前未被约束。落实为§6.1 frontmatter新增的`lifecycle_state`字段驱动的显式状态机：

| 当前`lifecycle_state` | 允许触发 | 目标状态 | 拒绝条件 |
|---|---|---|---|
| （不存在Mount Record） | 挂载CI流水线 | `MOUNTING` | — |
| `MOUNTING` | 挂载CI流水线完成 | `ACTIVE` | 挂载CI流水线本身失败：保留`MOUNTING`，人工介入（不允许期间发起退场） |
| `ACTIVE` | 退场CI流水线（§6.4只读冻结） | `DECOMMISSIONING` | — |
| `ACTIVE` | 挂载CI流水线（版本升级类重复挂载） | `MOUNTING`（复用挂载流程处理版本升级，同ARC-018既定"挂载脚手架同样承载升级"精神） | — |
| `MOUNTING`／`DECOMMISSIONING` | 任何流水线（挂载或退场） | — | **直接拒绝**：`lifecycle_state`已处于非稳态（`MOUNTING`/`DECOMMISSIONING`），不允许叠加另一个生命周期操作，须等当前操作完成（转为`ACTIVE`或`DECOMMISSIONED`）后才可发起新操作 |
| `DECOMMISSIONING` | 退场CI流水线完成 | `DECOMMISSIONED` | — |

CI层面的具体强制机制：§4 CI流水线新增一个前置阶段`lifecycle-lock-check`（置于`boundary-check`之前），读取当前Mount Record frontmatter的`lifecycle_state`，若为`MOUNTING`或`DECOMMISSIONING`（且不是本次流水线自己发起的那次操作）则直接`exit 1`，不进入后续阶段：

```yaml
lifecycle-lock-check:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - name: 检查Mount Record当前生命周期状态是否允许本次操作
      run: scripts/check-lifecycle-lock.sh <context> <intended-operation>  # intended-operation: mount | decommission
      # 逻辑: 读取frontmatter.lifecycle_state；若为MOUNTING/DECOMMISSIONING且非本次run触发，exit 1；
      # 否则(不存在/ACTIVE)将lifecycle_state原子更新为对应中间态(MOUNTING/DECOMMISSIONING)，
      # 更新本身通过对README.md文件的Git提交完成(Git本身的分支保护/PR合并串行化即天然提供了此处所需的
      # 排他语义，不需要额外引入分布式锁——同RGS-BAS-002"不引入新部署执行机制"精神一致，复用既有工具链)
```

该机制与RGS-DTL-024§2 `uq_deploy_runs_cluster_running`部分唯一索引是**同一类问题的两种物理实现**：前者（集群编排）状态存于PostgreSQL，用数据库层唯一索引做排他；后者（单App挂载/退场）状态存于Git版本控制的Markdown frontmatter，用Git提交的天然串行性做排他——两者共同体现"App级/单流程内部OCC幂等"与"跨流程/跨集群操作排他"是两个不同层次的问题，缺一不可，本项目对两个层次分别在合适的物理介质上给出了对应机制。

---

## 7. 本文档的覆盖范围与后续计划

本文档覆盖：Helm chart四个核心模板文件的具体YAML内容（Deployment/StatefulSet骨架、NetworkPolicy两段式default-deny+allow-list、ServiceMonitor、ExternalSecret）、CI/CD六阶段流水线的GitHub Actions具体定义、数据库开通SQL脚本格式、Mount Record的物理存储格式（README frontmatter为事实来源，Appendix C为索引视图）及其一致性校验脚本、退场安全网的物理落地方式、**生命周期排他锁（§6.5，`lifecycle_state`状态机+CI前置阶段，防止同一限界上下文的挂载与退场流水线并发触发产生竞态）**。

本版本明确不覆盖、留待后续：

- `deny.toml`跨限界上下文`use`禁令的具体规则集写法（随各域挂载增量追加，非本文档职责）。
- `ExternalSecret`所指向的自托管密钥管理后端（`ClusterSecretStore`）本身的选型与部署——目前假定其已存在（属于平台基础设施，非"挂载脚手架"本身），若尚未选型，应先补一个独立的平台基础设施DTL文档。
- `check-mount-record-consistency.sh`的完整实现代码（本文档只给出脚本职责与调用点，非逐行实现）。
- 托管云K8s（非自托管）环境下Helm chart的差异适配——按ARC-018/ARC-045一致的"自托管优先"原则，此项被有意推迟，需要时应先有独立ADR。
- Admission Webhook等更强的准入时校验机制（当前一致性校验落在CI阶段而非准入时点，属于已知的"CI通过后到实际部署前仍有窗口"限制，暂未设计缓解措施，建议登记为新TBD交由后续版本处理）。
- `check-lifecycle-lock.sh`的完整实现代码（本文档§6.5只给出状态机规则与调用点，非逐行实现）；`lifecycle_state`状态迁移依赖Git提交串行性这一假设在"CI Runner并发数>1且未对同一文件路径加互斥"的极端场景下的边界情况，未做进一步压力验证。

后续详细设计建议顺序：本文档确立的物理脚手架落地后，建议按RGS-REQ-001§11.2.1既定的PH阶段映射表，优先推进各PH-1/PH-2阶段业务域（如ANT反作弊、MM匹配系统，两者均已完成基本设计且是本会话最新补齐的域）的详细设计，以尽快验证本文档给出的CI/YAML模板在真实业务域挂载中的可用性；核心架构自身遗留的match_db／social_db／admin_db物理设计（RGS-DTL-001§7遗留项）可与之并行推进。

---

## 追溯性

| 需求/设计来源 | 本文档章节 |
|---|---|
| RGS-BAS-002§4.2 CI/CD流水线阶段表 | §4 |
| RGS-BAS-002§5.2 Helm chart模板结构 | §2 |
| RGS-BAS-002§5.3 NetworkPolicy规则表 | §3 |
| RGS-BAS-002§6 数据库开通设计 | §5 |
| RGS-BAS-002§10.1 Mount Record字段 | §6.1、§6.2 |
| RGS-BAS-002§10.2 归档位置 | §6.2 |
| RGS-BAS-002§11 退场设计 | §6.4 |
| RGS-BAS-002§12 标准化检查清单 | §6.3（可脚本化部分） |
| ARC-018 挂载模式 | 全文 |
