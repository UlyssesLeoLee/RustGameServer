# 详细设计书（詳細設計書 / Detailed Design Document）

**核心架构：物理数据库设计・协议线格式・核心算法详细设计**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DTL-001 |
| 版本 | 0.1 |
| 父文档 | RGS-BAS-001 基本设计书（本文档为其详细化，不改变任何既有决定，仅将逻辑设计落实为物理/实现级设计） |
| 依据标准 | IPA『共通フレーム 2013（SLCP-JCF2013）』详细设计工程 |
| 制定日 | 2026-08-17 |
| 制定者 | 架构师 |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响章节 |
|---|---|---|---|---|---|
| 0.1 | 2026-08-17 | 架构师 | — | 初版制定（负责人指示"推进制作更新详细设计"）。本项目此前26个域全部止步于需求+基本设计两层，本文档是**第一份详细设计文档**，作为后续其余域详细设计的模板与先例。范围：RGS-BAS-001§5物理数据库设计的两个最核心限界上下文（player_db／economy_db）落实为具体DDL、§6接口设计落实为具体协议线格式（.proto风格）、§4.2 tick循环与§4.3 AOI算法落实为可直接翻译为Rust实现的伪代码级算法。**本版本不覆盖BAS-001全部章节**，其余限界上下文（match_db／social_db／admin_db）与MT/GD/EV/WF/OB/AD模块详细设计留待后续版本或独立DTL文档补充，见§7 | 全部 |

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 | 2026-08-17 | — |
| 评审（技术） | | | DDL是否与RGS-BAS-001§5逻辑ER图/RGS-BAS-007命名规范完全一致，协议线格式是否与RGS-BAS-001§6字段设计一一对应 |
| 评审（DBA） | | | 索引/约束设计是否满足RGS-BAS-007既定的索引/分区标准，是否有遗漏的高频查询路径未建索引 |
| 审批（负责人） | | | 本文档的基准化 |

---

## 目录

1. [前言](#1-前言)
2. [物理数据库设计：player_db](#2-物理数据库设计player_db)
3. [物理数据库设计：economy_db](#3-物理数据库设计economy_db)
4. [协议线格式：PlayerService／EconomyService](#4-协议线格式playerserviceeconomyservice)
5. [核心算法详细设计](#5-核心算法详细设计)
6. [错误码一览](#6-错误码一览)
7. [本文档的覆盖范围与后续计划](#7-本文档的覆盖范围与后续计划)

---

# 1. 前言

## 1.1 本文档的定位

IPA SLCP-JCF2013将设计工程分为**基本设计（システム外部から見た仕様）**与**详细设计（内部構造・実装レベルの仕様）**两个阶段。RGS-BAS-001已完成前者：限界上下文划分、组件职责、逻辑ER图（字段用`string`/`long`/`datetime`等抽象类型表述）、UML接口视图（方法签名但无线格式）。本文档完成后者：**逻辑ER图→物理DDL**（具体SQL类型、约束、索引）、**UML接口→协议线格式**（具体wire类型、字段编号，可直接生成代码）、**流程图→算法伪代码**（可直接翻译为Rust实现，而非仅表达"做什么"）。

## 1.2 本文档不做什么

- **不重新决策**：本文档不引入任何RGS-BAS-001未决定的架构选择，逻辑设计与物理设计之间若出现表面差异（如字段拆分），须能一一追溯回原逻辑设计条目，不得借详细设计之名做架构变更
- **不是实现本身**：本文档仍是设计文档，伪代码用于表达算法逻辑与边界条件，不是可编译的Rust源码；.proto风格片段用于固定线格式，实际是否采用Protobuf或其他IDL留给实现阶段，本文档固定的是**字段编号与类型**这一契约本身

## 1.3 记述规则

物理类型标注遵循PostgreSQL语法；协议字段编号规则：1〜15为高频字段（varint单字节编码），16以上为低频/可选字段，编号一经分配**不得**在后续版本变更或复用（同Protobuf既定最佳实践，即便本项目最终不采用Protobuf本身，该编号纪律仍保留作为契约稳定性的通用原则）。

---

# 2. 物理数据库设计：player_db

## 2.1 DDL

```sql
-- 复用RGS-BAS-007既定命名规范：表名snake_case复数、主键统一为<entity>_id、
-- 乐观并发列统一命名version、审计时间戳统一created_at/updated_at

CREATE TABLE accounts (
    player_id       UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    credential_hash TEXT NOT NULL,
    status          SMALLINT NOT NULL DEFAULT 0,  -- 0=Registered 1=Active 2=Suspended 3=Banned 4=Deleted（枚举值见ST-005，故意用SMALLINT而非TEXT：高频WHERE条件，避免字符串比较开销）
    version         BIGINT NOT NULL DEFAULT 0,     -- OCC，DR-007
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT chk_accounts_status CHECK (status BETWEEN 0 AND 4)
);

CREATE TABLE characters (
    character_id     UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    player_id        UUID NOT NULL REFERENCES accounts(player_id) ON DELETE RESTRICT,
    -- ON DELETE RESTRICT而非CASCADE：账号删除走FR-GOV-010〜013既定的跨库编排流程
    -- （RGS-BAS-009§5.2），不得由数据库外键级联静默删除，避免绕过审计留痕
    name              VARCHAR(32) NOT NULL,
    level             INT NOT NULL DEFAULT 1,
    current_scene_id  UUID,                        -- 可空，未在场景内时为NULL
    session_epoch     BIGINT NOT NULL DEFAULT 0,    -- ARC-005 Single-Writer核心机制
    version           BIGINT NOT NULL DEFAULT 0,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT uq_characters_name UNIQUE (name)     -- 角色名全局唯一，BR-001既定
);

CREATE TABLE ban_records (
    ban_id      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    player_id   UUID NOT NULL REFERENCES accounts(player_id) ON DELETE RESTRICT,
    reason      TEXT NOT NULL,
    issued_by   UUID NOT NULL,                      -- 逻辑引用admin_db操作者，跨库不建物理FK（同RGS-BAS-007既定跨限界上下文引用规则）
    issued_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at  TIMESTAMPTZ                          -- NULL=永久封禁
);

-- 索引设计（对应高频查询路径）
CREATE INDEX idx_characters_player_id ON characters (player_id);
    -- 支撑§4.4.1 GetCharacterList：按player_id列出全部角色
CREATE INDEX idx_characters_current_scene_id ON characters (current_scene_id)
    WHERE current_scene_id IS NOT NULL;
    -- 部分索引：支撑"某场景当前有哪些角色"查询（运维/GM工具），NULL值不入索引减小体积
CREATE INDEX idx_ban_records_player_id_active ON ban_records (player_id)
    WHERE expires_at IS NULL OR expires_at > now();
    -- 部分索引：登录鉴权路径（FR-PL-001）需要快速判定"当前是否有生效中的封禁"，
    -- 只索引未过期/永久记录，历史已过期封禁不占索引空间
```

## 2.2 与逻辑设计的对应关系

| RGS-BAS-001§5.3逻辑字段 | 物理实现 | 差异说明 |
|---|---|---|
| `Account.status`（字符串枚举） | `accounts.status SMALLINT` | 高频查询列改用整数枚举，逻辑语义不变，仅物理编码优化（同RGS-BAS-007既定"高频WHERE列避免TEXT比较"原则） |
| `Character.name`唯一性 | `uq_characters_name UNIQUE` | BAS-001 UML图未显式标注唯一性，本文档补充物理约束（角色名全局唯一是既有BR-001隐含要求，此前逻辑设计遗漏显式声明，此处不算新决策，是对既有需求的物理落实） |
| 其余字段 | 类型直译 | `string`→`UUID`/`VARCHAR`/`TEXT`视语义而定，`long`→`BIGINT`，`datetime`→`TIMESTAMPTZ`（复用RGS-BAS-007既定"时间戳一律带时区"规范） |

---

# 3. 物理数据库设计：economy_db

## 3.1 DDL

```sql
CREATE TABLE wallets (
    character_id  UUID PRIMARY KEY,   -- 逻辑引用player_db.characters，跨库不建物理FK
    balance        BIGINT NOT NULL DEFAULT 0 CHECK (balance >= 0),
    version         BIGINT NOT NULL DEFAULT 0,
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE inventory_items (
    item_instance_id  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    character_id      UUID NOT NULL,
    item_template_id  VARCHAR(64) NOT NULL,   -- 引用静态配置表（ARC-016数值表），非动态数据
    quantity          INT NOT NULL DEFAULT 1 CHECK (quantity > 0),
    version           BIGINT NOT NULL DEFAULT 0,
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE transaction_ledger (
    -- 落地FR-EC-003确定请求API的幂等键与审计追溯，仅追加，复用RGS-BAS-007§4幂等去重表设计标准
    ledger_id     UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    request_id    UUID NOT NULL,          -- 幂等键，客户端/上游生成
    character_id  UUID NOT NULL,
    operation     SMALLINT NOT NULL,      -- 0=grant_item 1=consume_item 2=grant_currency 3=consume_currency
    payload       JSONB NOT NULL,         -- 操作明细（item_template_id/quantity或currency delta），结构随operation变体
    expected_version BIGINT NOT NULL,     -- OCC校验时使用的版本号
    result_version    BIGINT NOT NULL,    -- 执行成功后的新版本号
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT uq_transaction_ledger_request_id UNIQUE (request_id)
    -- 唯一约束是幂等性的物理强制层：重复request_id的INSERT在数据库层即失败，
    -- 不依赖应用层查重逻辑单独兜底（纵深防御，同ARC-009既定幂等设计精神）
);

-- 分区：按RGS-BAS-007§4既定标准，交易流水表按时间范围月度分区
CREATE TABLE transaction_ledger_template (LIKE transaction_ledger INCLUDING ALL);
-- 实际分区创建由既有分区滚动创建自动化流程（RGS-BAS-007§4）执行，本文档仅声明分区键

CREATE INDEX idx_inventory_items_character_id ON inventory_items (character_id);
CREATE INDEX idx_transaction_ledger_character_id_created ON transaction_ledger (character_id, created_at);
    -- 支撑GM工具/玩家自查"我的交易历史"按角色+时间范围查询
```

## 3.2 确定请求API的物理执行语义

`CommitTransaction`（RGS-BAS-001§4.5.1既有时序）对应的物理事务边界：

```sql
BEGIN;
  -- 1. 幂等检查：request_id已存在则直接返回既有结果，不重复执行（应用层先查一次可选，
  --    但真正的幂等保证来自下方INSERT的唯一约束，查询只是避免不必要的失败重试）
  -- 2. OCC校验：更新前先确认version未变
  UPDATE wallets SET balance = balance + $delta, version = version + 1, updated_at = now()
    WHERE character_id = $cid AND version = $expected_version;
  -- 若UPDATE影响行数为0，说明version已被并发修改，事务回滚，向上层返回OCC冲突错误（ST-004状态转移）
  -- 3. 写入流水（幂等键的物理强制点）
  INSERT INTO transaction_ledger (request_id, character_id, operation, payload, expected_version, result_version)
    VALUES ($request_id, $cid, $op, $payload, $expected_version, $expected_version + 1);
    -- 若request_id冲突，本INSERT失败，事务整体回滚——但此时上一步wallets的UPDATE也会一并回滚，
    -- 不会出现"钱包已扣但流水未记"的不一致态，这是"同一事务边界"设计的直接物理保证
COMMIT;
```

---

# 4. 协议线格式：PlayerService／EconomyService

## 4.1 设计说明

RGS-BAS-001§6.3已定义方法签名与字段名，本节固定**字段编号**（协议演进的兼容性锚点，一旦分配不可变更/复用，同ARC-015 Expand-Contract精神在协议层的对应物）。以下用Protobuf风格表达线格式契约，不代表实现阶段必须采用Protobuf本身。

## 4.2 PlayerService

```protobuf
message AuthenticateRequest {
  string credential_token = 1;
}
message AuthenticateResponse {
  string player_id = 1;
  repeated CharacterSummary character_list = 2;
  ResultCode result_code = 3;
}
message CharacterSummary {
  string character_id = 1;
  string name = 2;
  int32 level = 3;
}

message SelectCharacterRequest {
  string player_id = 1;
  string character_id = 2;
}
message SelectCharacterResponse {
  int64 session_epoch = 1;      // 新发行的epoch，ARC-005核心字段，字段号低位优先编码
  string current_scene_id = 2;
  ResultCode result_code = 3;
}
```

## 4.3 EconomyService

```protobuf
message CommitTransactionRequest {
  string request_id = 1;        // 幂等键，编号1（最高频访问字段）
  string character_id = 2;
  int64 session_epoch = 3;      // ARC-005：请求必须携带当前epoch供服务端校验陈旧请求
  oneof operation {
    GrantItem grant_item = 10;
    ConsumeItem consume_item = 11;
    GrantCurrency grant_currency = 12;
    ConsumeCurrency consume_currency = 13;
  }
  int64 expected_version = 20;  // OCC字段编号统一置于20+区间，与业务字段区分
}
message CommitTransactionResponse {
  ResultCode result_code = 1;
  int64 new_version = 2;
  string ledger_id = 3;
}
```

## 4.4 通用`ResultCode`枚举（跨全部服务复用，不各自定义）

```protobuf
enum ResultCode {
  OK = 0;
  UNKNOWN_ERROR = 1;
  OCC_CONFLICT = 2;           // 对应ST-004乐观并发冲突
  STALE_SESSION_EPOCH = 3;    // ARC-005：epoch已被更新的SelectCharacter请求作废
  INVALID_REQUEST = 4;
  ACCOUNT_BANNED = 5;
  INSUFFICIENT_BALANCE = 6;
  DUPLICATE_REQUEST_ID = 7;   // 幂等键冲突但内容不一致（正常重放应直接返回原结果而非此错误）
}
```

---

# 5. 核心算法详细设计

## 5.1 tick循环算法（落实RGS-BAS-001§4.2.2流程图为伪代码）

```
fn scene_actor_tick(scene: &mut SceneState, tick_no: u64) {
    let tick_start = Instant::now();

    // 阶段1：输入应用（ECS System顺序执行，同一场景内严格串行，ARC-001既定）
    for input in scene.pending_inputs.drain_up_to(tick_no) {
        // 乱序/重复输入的排除：sequence_no单调性检查（FR-RT-004）
        if input.sequence_no <= scene.last_applied_sequence[input.entity_id] {
            continue;  // 静默丢弃，不报错（客户端重传是正常现象，非异常）
        }
        apply_input_system(scene, input);
        scene.last_applied_sequence[input.entity_id] = input.sequence_no;
    }

    // 阶段2：移动模拟（含FR-SEC-050未信任输入解析安全边界——本阶段全部读取均来自
    // 阶段1已校验通过的输入，不重新解析原始字节，避免重复的panic风险面）
    movement_system(scene);

    // 阶段3：战斗判定（服务器权威，ARC-002）
    combat_system(scene);

    // 阶段4：AOI更新（§5.2另行详述）
    aoi_update_system(scene);

    // 阶段5：复制生成（差分快照）
    let snapshot = generate_delta_snapshot(scene, tick_no);
    broadcast_to_clients(scene, snapshot);

    // 耗时预算校验（NFR-PE-002）
    let elapsed = tick_start.elapsed();
    if elapsed > TICK_BUDGET_SOFT_LIMIT {
        emit_metric("tick_overrun", elapsed);  // 复用RGS-BAS-004既定指标体系，不新建监控通道
    }
    if elapsed > TICK_BUDGET_HARD_LIMIT {
        // 硬预算超支：本tick已产生的快照仍然发送（不丢弃已完成的工作），
        // 但触发降级判定（是否需要临时降低AOI半径/快照频率，同ARC-013既定原则）
        trigger_degradation_check(scene);
    }
}
```

**关键边界条件说明**：
- 输入应用阶段的乱序/重复排除是**幂等的必要条件**——若不做此排除，客户端网络抖动导致的重传会被反复应用，产生移动量翻倍等错误
- 硬预算超支**不得**导致丢弃已完成的快照生成结果（宁可这一tick延迟发送，不得让客户端出现"完全没收到状态更新"的更差体验）

## 5.2 AOI计算算法（落实RGS-BAS-001§4.3.1，含G-003无饿死性设计）

```
fn aoi_update_system(scene: &mut SceneState) {
    for observer in scene.entities_with_view() {
        let mut candidates: Vec<(EntityId, f64)> = Vec::new();

        for target in scene.entities_in_grid_range(observer.position, VIEW_DISTANCE) {
            if target.id == observer.id { continue; }

            let distance_score = 1.0 - (distance(observer.position, target.position) / VIEW_DISTANCE);
            let importance_score = target.importance_weight;  // 静态权重，ARC-016数值表配置

            // G-003无饿死性设计：老化因子随"距上次更新的tick数"单调增长，
            // 权重下限恒正，保证任意实体等待足够久后必被纳入
            let ticks_since_update = scene.current_tick - target.last_aoi_update_tick;
            let aging_score = AGING_BASE_WEIGHT + AGING_GROWTH_RATE * (ticks_since_update as f64);

            let final_score = distance_score * W_DISTANCE
                             + importance_score * W_IMPORTANCE
                             + aging_score * W_AGING;
            candidates.push((target.id, final_score));
        }

        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let selected = candidates.into_iter().take(observer.aoi_budget).collect();

        update_observer_view(observer, selected, scene.current_tick);
    }
}
```

**无饿死性证明要点**：`aging_score`是`ticks_since_update`的严格单调递增函数且下限为`AGING_BASE_WEIGHT > 0`，故任一实体若持续未被选中，其`ticks_since_update`持续增长，`final_score`最终必然超过任何"新近更新过"的竞争实体的得分上限（`W_DISTANCE + W_IMPORTANCE`为常数上界），保证有限时间内必被选中——这是G-003"老化机制"要求的形式化落实，而非仅停留在文字描述。

---

# 6. 错误码一览

| `ResultCode` | 触发条件 | 对应既有设计 |
|---|---|---|
| `OCC_CONFLICT` | `expected_version`与数据库当前`version`不一致 | RGS-BAS-001§3.6.2 ST-004状态迁移 |
| `STALE_SESSION_EPOCH` | 请求携带的`session_epoch`低于该角色当前生效值 | ARC-005 Single-Writer保证 |
| `ACCOUNT_BANNED` | `accounts.status`处于封禁态且存在生效中的`ban_records` | FR-AD-001 |
| `INSUFFICIENT_BALANCE` | `ConsumeCurrency`请求的扣减量超过当前`wallets.balance` | FR-EC-002 |
| `DUPLICATE_REQUEST_ID` | 幂等键已存在但**请求内容与首次不一致**（正常重放应命中相同`payload`，直接返回原`ledger_id`而非此错误——本错误码专指内容冲突这一异常情形） | §3.2确定请求物理执行语义 |

---

# 7. 本文档的覆盖范围与后续计划

本文档是本项目**第一份详细设计文档**，作为模板与先例，覆盖范围**刻意限定**在核心架构中最基础的两个限界上下文（player_db／economy_db）与最核心的两个算法（tick循环／AOI计算），未覆盖：

- match_db／social_db／admin_db的物理DDL
- MatchService／SocialService／AdminService的协议线格式细化
- §4.6〜4.8（对局/社交、事件工作流、可观测性）的详细算法设计
- 全部其余25个域（RGS-REQ-006〜030对应的BAS-002〜027）的详细设计，目前**均未开始**

**后续计划**（留待负责人确认优先级排期，本文档不代为决定）：按既有RGS-REQ-001§11.2.1"领域文档工作的阶段归属表"的PH-1〜PH-8顺序，逐域产出对应DTL文档，命名规则为`RGS-DTL-<与BAS同编号>`，与本文档同一documentclass与记述规则。核心架构（本文档覆盖之外的部分）与挂载架构（RGS-BAS-002）建议优先于其余业务域，因其余域的详细设计普遍依赖核心架构的物理设计已经落地（如全部限界上下文的DDL都遵循本文档§2/§3确立的命名与索引纪律）。
