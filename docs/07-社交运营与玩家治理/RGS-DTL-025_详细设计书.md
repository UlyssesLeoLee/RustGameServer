# 详细设计书（詳細設計書 / Detailed Design Document）

**反作弊与作弊治理体系：物理数据库设计・事件线格式・案件聚合算法・风控规则 DSL 安全增补**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DTL-025 |
| 版本 | 0.3 |
| 父文档 | RGS-BAS-025 反作弊与作弊治理体系 基本设计书（本文档为其详细化，不改变任何既有决定，仅将逻辑设计落实为物理/实现级设计） |
| 依据标准 | IPA『共通フレーム 2013（SLCP-JCF2013）』详细设计工程 |
| 制定日 | 2026-08-17 |
| 制定者 | 架构师 |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响章节 |
|---|---|---|---|---|---|
| 0.1 | 2026-08-17 | 架构师 | — | 初版制定（负责人指示"继续"推进详细设计，本文档为RGS-DTL-002挂载脚手架落地后第一份业务域详细设计，用于验证RGS-DTL-002模板在真实域挂载中的可用性）。细化RGS-BAS-025§3.1逻辑数据模型为`admin_db`内`detection_signals`／`anticheat_cases`／`case_signal_links`三表具体DDL、§2.2/§4.2的事件为具体线格式、§3.4案件聚合逻辑为可直接翻译为Rust实现的伪代码级算法（含TBD-ANT-001聚合窗口/阈值参数的具体默认值提案）。**本版本不覆盖**：GM后台UI的具体交互细节、`anticheat-fusion`分析图内部LangGraph节点级实现。见§6 | 全部 |
| 0.2 | 2026-08-17 | 架构师 | — | 最终审核发现并修正：§2 `detection_signals`/`anticheat_cases`的`player_id`原写作`REFERENCES accounts(account_id)`，但`account_id`这一列名在`player_db.accounts`中**从不存在**（该表主键为`player_id UUID`），属成文时未核对上游DDL产生的幽灵列引用；且跨库本就不应出现物理FK子句。现改为无FK的`BIGINT`列，并注明其逻辑对应`player_db.accounts.player_seq`（RGS-DTL-001 v0.3§12新增的跨库标识映射列） | §2 |
| **0.3** | 2026-08-20 | 架构师 | — | **受控 DSL 增补**：将 RGS-REQ-028-ADD1 的 Rhai 候选实现纳入同一 DTL 编号，明确 ADR-0020 仍为“已制定・待评审”，并补齐签名制品、沙箱、版本 fencing、分区 fail-closed、回滚审计及测试追溯 | §6 |

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 | 2026-08-17 | — |
| 评审（技术） | | | DDL是否与RGS-BAS-025§3.1逻辑模型/RGS-BAS-007命名与分区规范完全一致 |
| 评审（DBA） | | | 索引设计是否覆盖§3.2既定的两个查询方向，月度分区是否与既有`admin_db`分区管理脚本兼容 |
| 审批（负责人） | | | 本文档的基准化；TBD-ANT-001默认值提案是否可直接采纳或需PH-4真实数据验证后再定 |

---

## 目录

1. [前言](#1-前言)
2. [物理数据库设计：admin_db反作弊三表](#2-物理数据库设计admindb反作弊三表)
3. [事件线格式](#3-事件线格式)
4. [案件聚合算法详细设计](#4-案件聚合算法详细设计)
5. [TBD-ANT-001参数默认值提案](#5-tbd-ant-001参数默认值提案)
6. [风控规则 DSL（v0.3 受控增补）](#6-风控规则-dslv03-受控增补)
7. [本文档的覆盖范围与后续计划](#7-本文档的覆盖范围与后续计划)

---

## 1. 前言

### 1.1 定位

RGS-BAS-025给出了`DetectionSignal`/`AntiCheatCase`/`CaseSignalLink`的逻辑字段表与聚合逻辑的文字流程描述。本文档将其落实为：可直接执行的PostgreSQL DDL、事件总线上实际传输的消息格式、聚合逻辑的算法级伪代码（含边界条件处理），使实现人员无需再做设计判断，只需翻译为Rust代码。

### 1.2 本文档不做什么

- 不重新决定RGS-BAS-025已确定的任何结构性选择（三表依附`admin_db`不新建库、处置权收口于`AdminService`、智能层只读建议不直接执行）。
- 不覆盖`anticheat-fusion`分析图内部的LangGraph节点图结构与Prompt设计——那属于RGS-BAS-011§5A既定治理框架下、评审通过后另行提交的分析图定义文档范围，本文档只覆盖该图的输入（§3聚合窗口数据）与输出（§4.2既定的三类只读建议）在ANT域内的落位，不覆盖图内部实现。
- 不覆盖GM后台前端UI细节（案件列表排序展示、详情页布局）——属于前端/GM工具自身设计范围。

### 1.3 记述规则

沿用RGS-DTL-001§1.3与RGS-DTL-002§1.3已确立的记述规则：DDL类型以PostgreSQL为准；事件消息格式以Protobuf风格给出，字段编号固定、只增不改；伪代码可直接对应到Rust `Result`风格实现。

---

## 2. 物理数据库设计：admin_db反作弊三表

对应RGS-BAS-025§3.1/§3.2。三表均落位于既有`admin_db`（AD限界上下文），复用其既有连接池与迁移工具链，本文档只新增表结构本身。

```sql
-- 检测信号表：RT/SY既有校验判定的异步旁路记录，对应FR-ANT-001〜003
CREATE TABLE detection_signals (
    signal_id       BIGSERIAL PRIMARY KEY,
    player_id       BIGINT NOT NULL,   -- 跨库逻辑外键：player_db.accounts.player_seq（RGS-DTL-001§12跨库标识映射），此处不设物理FK，仅注释语义
    signal_type     TEXT NOT NULL CHECK (signal_type IN (
                        'SPEED_VIOLATION', 'COLLISION_VIOLATION',
                        'INPUT_ANOMALY', 'REPLAY_ANOMALY', 'PLAYER_REPORT')),
    occurred_at     TIMESTAMPTZ NOT NULL,
    context_ref     TEXT NOT NULL,               -- 场景/对局/原始举报记录标识，语义随signal_type而定
    raw_value       DOUBLE PRECISION,             -- PLAYER_REPORT类型可为空
    threshold_value DOUBLE PRECISION,             -- PLAYER_REPORT类型可为空
    case_id         BIGINT NULL REFERENCES anticheat_cases(case_id),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
) PARTITION BY RANGE (occurred_at);
-- 月度分区，保留期3年(NFR-ANT-003)，分区创建/DETACH复用既有admin_db分区管理脚本(G-005模式)

CREATE INDEX idx_detection_signals_player_time
    ON detection_signals (player_id, occurred_at);
CREATE INDEX idx_detection_signals_case
    ON detection_signals (case_id) WHERE case_id IS NOT NULL;

-- 反作弊案件表，对应FR-ANT-010
CREATE TABLE anticheat_cases (
    case_id          BIGSERIAL PRIMARY KEY,
    player_id        BIGINT NOT NULL,   -- 同上，逻辑引用player_db.accounts.player_seq，跨库不设物理FK
    status           TEXT NOT NULL DEFAULT '待审核'
                        CHECK (status IN ('待审核', '已处置', '已驳回')),
    confidence_score DOUBLE PRECISION NOT NULL DEFAULT 0,
    signal_count     INTEGER NOT NULL DEFAULT 0,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_signal_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    disposition_ref  BIGINT NULL,   -- 指向处置记录(依附既有AdminService操作日志表,非本文档新建)
    version          INTEGER NOT NULL DEFAULT 0  -- 乐观锁，防止并发信号追加与GM处置操作的竞态(见§4.3)
);

CREATE INDEX idx_anticheat_cases_player_status
    ON anticheat_cases (player_id, status);
CREATE INDEX idx_anticheat_cases_status_confidence
    ON anticheat_cases (status, confidence_score DESC)
    WHERE status = '待审核';  -- 支撑RGS-BAS-025§5.1 GM按confidence_score排序查询(NFR-ANT-002)

-- 案件-信号关联表，对应FR-ANT-013
CREATE TABLE case_signal_links (
    case_id   BIGINT NOT NULL REFERENCES anticheat_cases(case_id),
    signal_id BIGINT NOT NULL REFERENCES detection_signals(signal_id),
    linked_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (case_id, signal_id)
);
CREATE INDEX idx_case_signal_links_signal ON case_signal_links (signal_id);
-- PRIMARY KEY(case_id, signal_id)已提供"案件→信号"方向索引；本索引提供反向"信号→案件"查询，满足§3.2双向要求
```

`player_id`未设物理外键（`admin_db`与`player_db`是不同数据库，跨库物理FK在PostgreSQL中不可行），仅在注释中声明逻辑关联——这与RGS-DTL-001§2/§3多库物理隔离的既有约束一致，不是本文档新增的例外。

---

## 3. 事件线格式

对应RGS-BAS-025§2.2（信号产生事件）与§3.3（举报转化事件）。均复用ARC-009既有Outbox事件基础设施，格式如下：

```protobuf
// DetectionSignalRaised：RT/SY既有校验分支在判定异常时发布，对应§2.2
message DetectionSignalRaised {
  int64  player_id      = 1;
  string signal_type    = 2;   // 取值同detection_signals.signal_type CHECK约束
  string context_ref    = 3;
  double raw_value       = 4;  // 0表示未提供(proto3不区分未设置与0,举报类信号消费端按signal_type判断是否读取)
  double threshold_value = 5;
  int64  occurred_at_ms  = 6;  // Unix毫秒时间戳
}

// PlayerReportSubmitted：RGS-BAS-014既有举报提交完成后发布(若尚不存在,由BAS-014补充),
// 本文档只定义ANT侧消费该事件后的转化,不新增举报提交本身的发布逻辑
message PlayerReportSubmitted {
  int64  report_id        = 1;
  int64  reporter_id       = 2;
  int64  reported_player_id = 3;
  string report_category   = 4;  // 仅report_category == "CHEATING"时触发§3.3转化
  int64  submitted_at_ms    = 5;
}
```

ANT信号消费者订阅`PlayerReportSubmitted`，仅当`report_category == "CHEATING"`时按§3.3规则转化写入一条`signal_type='PLAYER_REPORT'`的`detection_signals`记录（`raw_value`/`threshold_value`置空，`context_ref`存`report_id`的字符串形式）；非作弊类举报直接忽略，不产生检测信号。

---

## 4. 案件聚合算法详细设计

对应RGS-BAS-025§3.4文字流程，落实为伪代码，覆盖并发场景与幂等语义。

### 4.1 主流程

```rust
// 每条新DetectionSignal写入后触发(信号消费者内,单条处理,不批处理,避免延迟放大)
fn on_signal_persisted(signal: &DetectionSignal, window: Duration, threshold: AggregationThreshold) -> Result<(), AggError> {
    let existing_case = find_open_case(signal.player_id)?;  // status='待审核' 的最新一条,若无则None

    match existing_case {
        Some(case) => {
            link_signal_to_case(case.case_id, signal.signal_id)?;
            let updated = recompute_case_stats(case.case_id)?;  // signal_count/last_signal_at
            recompute_confidence(case.case_id)?;  // §4.2简单规则路径,或触发§4.3智能层路径
            Ok(())
        }
        None => {
            let recent_signals = query_recent_signals(signal.player_id, signal.signal_type, window)?;
            if meets_threshold(&recent_signals, &threshold) {
                let case_id = create_case(signal.player_id, &recent_signals)?;  // 事务内: 建案件+批量link全部窗口内信号
                recompute_confidence(case_id)?;
            }
            // 未达阈值: 信号保持case_id=NULL,不创建案件,不报错(这是正常路径,非异常)
            Ok(())
        }
    }
}
```

### 4.2 简单规则置信度计算（对应RGS-BAS-025§4.1）

```rust
fn recompute_confidence_simple(case_id: CaseId) -> f64 {
    let links = query_case_signals(case_id);
    // 固定加权公式: 严重度权重(高危信号类型系数更高) × 数量对数衰减(防止单一类型刷量线性拉高分数)
    let weighted_sum: f64 = links.iter()
        .map(|s| severity_weight(s.signal_type) * s.severity_ratio())  // severity_ratio = raw_value/threshold_value,PLAYER_REPORT固定按1.0计
        .sum();
    let count_factor = (links.len() as f64 + 1.0).ln();  // 对数衰减,见§5参数提案
    (weighted_sum * count_factor).min(1.0)  // 归一化到[0,1],供§5.1 GM排序展示
}
```

多信号类型组合场景（`case`的构成信号`signal_type`去重后≥2种）触发§4.3智能层分析图评估（异步，不阻塞§4.1主流程写入），`confidence_score`以图输出为准覆盖简单规则计算值；单一类型场景则简单规则计算结果即为最终值，不调用智能层（对应RGS-BAS-025§4.1"低复杂度场景不经过智能层"的判定条件在此处被具体化为"信号类型种类数"这一可编程判断依据）。

### 4.3 并发控制

`recompute_case_stats`/`recompute_confidence`对`anticheat_cases`行的更新使用§2的`version`乐观锁列（复用RGS-DTL-001§3.2既定的OCC模式，与GM在§5.1并发提交处置操作时的更新互斥）：

```sql
UPDATE anticheat_cases
SET signal_count = signal_count + 1, last_signal_at = $now, confidence_score = $new_score, version = version + 1
WHERE case_id = $case_id AND version = $expected_version;
-- 影响行数=0则表示并发冲突(如GM同时正在处置该案件),重试查询最新version后重算,不报错给上游信号消费流程
```

若OCC冲突恰好发生在GM刚提交处置（`status`已变为`已处置`/`已驳回`）之后，信号消费者重试时应检测到`status`已非`待审核`，此时**不再**追加信号到已关闭案件，转而按§4.1"无开放案件"分支处理（可能开启新案件）——避免"已处置案件被后续信号悄悄追加"这一审计完整性问题。

---

## 5. TBD-ANT-001参数默认值提案

RGS-BAS-025§3.4与§4.1中"既定时间窗口"/"聚合阈值"/"加权系数"均标记为`TBD-ANT-001`（附件D已登记）。本文档提出以下默认值供PH-4真实数据验证前的初始上线使用，非最终值：

| 参数 | 提案默认值 | 依据 |
|---|---|---|
| 聚合窗口`window` | 24小时 | 覆盖单次游戏会话到次日的典型作弊行为发现延迟，过短易漏聚合、过长易把无关行为误关联 |
| 同类信号聚合阈值（触发新案件） | 3次 | 单次异常可能是误判/网络抖动，3次同类在窗口内重复大幅降低误判概率 |
| 高严重度信号单次触发阈值 | `severity_ratio ≥ 3.0`（即实际值达阈值3倍以上）即1次触发 | 明显偏离阈值的单次事件（如移动速度达上限5倍）本身已具备高置信度，无需等待重复 |
| `severity_weight`（各`signal_type`权重） | `SPEED_VIOLATION`=1.0／`COLLISION_VIOLATION`=1.0／`INPUT_ANOMALY`=0.6／`REPLAY_ANOMALY`=1.2／`PLAYER_REPORT`=0.8 | REPLAY_ANOMALY技术门槛更高、误判率更低故权重最高；INPUT_ANOMALY可能由网络问题引起故权重最低；PLAYER_REPORT主观性强，权重中等 |

以上默认值应在上线后按PH-4阶段真实误判率/漏判率数据校准，校准结果回写本文档新版本，不在RGS-BAS-025基本设计层面体现（属于实现参数调优，非结构性设计变更）。

---

## 6. 风控规则 DSL（v0.3 受控增补）

### 6.1 生效门禁与处置边界

本节受 RGS-ADR-0020 约束。ADR-0020 当前状态为“已制定・待评审”，故 Rhai 仅是候选受限脚本引擎；在 ADR 获批前，DSL 只可在隔离测试环境验证，生产节点的 DSL 执行与热更新开关必须为 `false`。规则只能输出检测分数或 `Observe` 建议，**不得**直接写库、发布处置事件或调用经济接口；案件创建、封禁等仍由本设计既有的确定性 `AdminService` 路径收口。

### 6.2 已签名规则制品

节点不得直接拉取分支头或未签名 `.rhai` 文件。每个可加载制品的规范 manifest 必须包含：

| 字段 | 约束 |
|---|---|
| `rule_id` / `channel` / `version` | 三元组唯一；`version` 严格单调递增，不允许覆盖已发布版本 |
| `script_sha256` | 对规范化 UTF-8 脚本文本计算；节点下载后必须重算并相等 |
| `artifact_uri` / `object_version` | 指向 Git 已签名 tag 对应提交和只读对象版本；禁止可变分支名或无版本对象 URL |
| `signing_key_id` / `signature` | 发布控制面对 manifest 作 Ed25519 签名；节点仅信任密钥管理服务/JWK 集登记的未撤销公钥 |
| `issued_at` / `expires_at` | 有效窗口；过期、尚未生效或时钟偏差超过 10 秒时拒绝加载 |
| `previous_version` / `change_ticket` | 支持受控回滚和审计；缺失即拒绝发布 |

加载顺序固定为：验证 manifest 结构/channel → 验证签名与撤销状态 → 下载指定对象版本 → 重算 SHA-256 → 编译进受限引擎 → 写入本地不可变缓存 → 收到版本围栏许可后激活。任一步失败均保留当前已验证版本，记录 `RULE_ARTIFACT_REJECTED` 审计与度量；不得降级加载裸脚本。

### 6.3 Rhai 沙箱与资源限制

每次执行使用独立、不可变的 `RuleContext` 副本，只注册 `ctx` 读取器、确定性数学/字符串/数组函数与显式审计钩子。不得注册文件系统、网络、进程、动态模块、反射、数据库、消息总线或任意时钟写入宿主函数；白名单变更须单独安全评审。

| 控制 | 实现要求 | 失败行为 |
|---|---|---|
| 能力白名单 | 编译前拒绝未知符号与宿主模块 | 返回中性分数 `0`，审计 `RULE_SANDBOX_DENIED` |
| 资源上限 | 配置最大操作数、调用/表达式深度、字符串/数组长度与每次内存预算 | 达上限中止执行，返回 `0`，不重试 |
| 时间预算 | 单规则墙钟 100 ms；工作线程可取消且不得阻塞采集/案件主路径 | 超时返回 `0`，审计 `RULE_TIMEOUT` 并增加度量 |
| 输出边界 | 仅允许有限、非负 `Score` 或 `Decision::Observe` | NaN、无穷、越界或类型错误归零并审计 `RULE_INVALID_OUTPUT` |

本域的 fail-safe 是不给规则造成自动处置：超时、异常或沙箱拒绝一律输出 `Score(0)` / `Observe`，随后由既有确定性阈值和人工流程决定是否建案。

### 6.4 版本 fencing 与一致性

控制面为每个 `(rule_id, channel)` 维护带签名的 `desired_version`、`fence_epoch`、`state`（`ACTIVE`/`ROLLED_BACK`/`REVOKED`）。NATS 仅负责通知，不能充当事实来源；节点必须定期向强一致规则清单存储重校验。每个执行请求携带已验证的 `(version, fence_epoch, manifest_hash)`；节点只能执行满足以下全部条件的缓存版本：

1. 版本和 manifest 签名有效，且 `version == desired_version`；
2. `fence_epoch` 等于最新已确认值；收到较大 epoch 时必须先停止旧版本；
3. channel 与请求路由完全一致，禁止 `canary` 落入 `stable`、跳号或回退至未签名缓存；
4. 节点已写入含规则、channel、版本、epoch、hash 和节点 ID 的 `RULE_VERSION_ACTIVATED` 审计。

缓存至多保留最近三份已验证版本，但“在缓存中”不等于“可执行”；仅当前围栏许可的版本可运行。

### 6.5 网络分区（fail-closed）与恢复

节点无法在租约到期前确认 `desired_version/fence_epoch`、收到冲突签名、清单不可读或签名密钥撤销时，必须立即停止受影响 `(rule_id, channel)` 的新 DSL 执行，输出 `Score(0)` / `Observe`，记录 `RULE_FENCE_LOST`（或对应拒绝码）并告警。不得无限期沿用旧规则、猜测最新版本，或自动将 DSL 结果升级为处置。

恢复后必须重新获取并验证清单与制品、确认当前 epoch 后才能执行；恢复前缓存命中不得补跑或改写历史案件。该策略优先保证规则一致性和不误处置，接受分区期间降低 DSL 检测覆盖率。

### 6.6 回滚、审计与测试追溯

回滚必须发布由同一受信控制面签名的 `ROLLED_BACK` manifest，包含目标历史 `version`、新的 `fence_epoch`、原因、`change_ticket`、操作者身份和审批记录；不得移动 Git 分支替代回滚。控制面原子写入新 epoch 后通知节点，节点须按 §6.2 完整验签、加载目标对象版本并确认后才切换。

必须持久化不可篡改审计：发布/批准者、源/目标版本与 hash、签名密钥 ID、原因、时间、channel、每节点接收/激活/拒绝时间及未确认节点列表。制品、manifest 和审计不得因回滚删除。回滚超过 RGS-REQ-028-ADD1 的 5 秒目标或存在未确认节点时，控制面必须维持 fail-closed 并升级人工处理，不能宣布“全节点已回滚”。

| 设计点 | 单元测试 | 集成/系统测试 | 可验收结果 |
|---|---|---|---|
| §6.2 签名制品 | TST-UT-07-R018 | TST-IT-07-R011 | 篡改 hash、无效/撤销签名、可变 URI 均拒绝并审计 |
| §6.3 沙箱 | TST-UT-07-R019 | TST-ST-07-R004/R007 | fs/net/syscall、超时、内存/操作超限返回 `0`，无副作用 |
| §6.4 围栏 | TST-UT-07-R020 | TST-IT-07-R012 | 旧 epoch、跳号、跨 channel 和未确认版本不得执行 |
| §6.5 分区 | TST-UT-07-R021 | TST-IT-07-R013、TST-ST-07-R011 | 租约失效停止 DSL；恢复验签后才执行 |
| §6.6 回滚 | TST-UT-07-R022 | TST-IT-07-R014、TST-ST-07-R012 | 未确认节点不计成功，完整审计证据可查询 |

---

## 7. 本文档的覆盖范围与后续计划

本文档覆盖：`admin_db`内反作弊三表的物理DDL（含分区/索引/OCC乐观锁列）、`DetectionSignalRaised`/`PlayerReportSubmitted`两个事件的具体线格式、案件聚合主流程与简单规则置信度计算的完整伪代码（含并发冲突处理）、TBD-ANT-001四项参数的初始默认值提案，以及 **v0.3 受 ADR-0020 待评审门禁约束的 DSL 签名、沙箱、版本围栏、分区和回滚设计**。

本版本明确不覆盖、留待后续：

- `anticheat-fusion`智能层分析图内部的LangGraph节点结构、Prompt模板、多信号类型组合评估的具体推理逻辑——按RGS-BAS-025§4.2既定治理流程，须先完成该图自身的注册评审，评审通过后另立分析图定义文档，本文档不代为设计。
- GM后台案件列表/详情页的具体UI交互细节。
- §5参数默认值的正式校准结果——当前为初始提案，非最终值，校准需等待PH-4真实运营数据。
- `AdminService`执行封禁/禁言的既有API本身的详细设计——RGS-BAS-025§5.1已明确复用既有API不新增接口，故其内部实现不属于本文档（ANT域）职责范围。

后续详细设计建议顺序：与RGS-DTL-002建议一致，可并行推进MM（匹配系统）的详细设计（RGS-DTL-026），以及RGS-DTL-001遗留的match_db／social_db／admin_db核心架构物理设计（后者与本文档的`admin_db`扩展存在交集，建议在RGS-DTL-001后续版本中显式引用本文档新增的三张表，避免同一数据库的物理设计分散在两份文档中互不知情）。

---

## 追溯性

| 需求/设计来源 | 本文档章节 |
|---|---|
| RGS-BAS-025§2.2 异步写入设计 | §3 |
| RGS-BAS-025§3.1 逻辑数据模型 | §2 |
| RGS-BAS-025§3.2 物理落位与约束 | §2 |
| RGS-BAS-025§3.3 举报作为信号来源 | §3、§4.1 |
| RGS-BAS-025§3.4 案件聚合逻辑 | §4.1 |
| RGS-BAS-025§4.1 简单规则判定 | §4.2 |
| RGS-BAS-025§4.2 智能层分析图接入 | §4.2（触发条件）、§7（明确排除内部实现） |
| TBD-ANT-001（附件D） | §5 |
| RGS-REQ-028-ADD1 FR-ANT-005~012、ARC-043-1~6 | §6.1~§6.6 |
| RGS-ADR-0020（已制定・待评审） | §6.1（生产门禁）、§6.3（沙箱） |
| RGS-DTL-002（挂载脚手架物理落地） | 前提依赖，本文档假定ANT/AD域已按RGS-DTL-002完成挂载 |
