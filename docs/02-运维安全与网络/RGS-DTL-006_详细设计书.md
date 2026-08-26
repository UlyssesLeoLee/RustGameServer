# 详细设计书（詳細設計書 / Detailed Design Document）

**网络安全：NetworkPolicy基线模板具体清单・未信任输入解析安全实现・多层速率限制与资源配额算法详细设计**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DTL-006 |
| 版本 | 0.3 |
| 父文档 | RGS-BAS-006 网络安全 基本设计书（本文档为其详细化，不改变任何既有决定，仅将逻辑设计落实为物理/实现级设计） |
| 依据标准 | IPA『共通フレーム 2013（SLCP-JCF2013）』详细设计工程 |
| 制定日 | 2026-08-17 |
| 制定者 | 架构师 |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响章节 |
|---|---|---|---|---|---|
| 0.1 | 2026-08-17 | 架构师 | — | 初版制定（负责人指示"继续"推进详细设计，本文档与RGS-DTL-003／004／005同批次产出，是02-运维安全与网络域四份详细设计中的最后一份）。细化RGS-BAS-006§4 NetworkPolicy基线原则为具体YAML模板、§7A.1未信任输入解析安全落实为具体Rust模式与CI lint配置、§7A.2多层速率限制落实为可直接翻译为Rust实现的伪代码（含Redis key设计与TBD-SEC-003阈值默认值提案）、§7A.3资源配额落实为具体算法、§6供应链安全流水线的漏洞扫描/SBOM/构建溯源落实为CI阶段配置骨架。**本版本不覆盖**：DDoS/WAF具体选型（TBD-SEC-001）、密钥管理中间件选型（TBD-SEC-002）、构建溯源签名的具体密码学方案。见§7 | 全部 |
| 0.2 | 2026-08-17 | 架构师 | — | 负责人指示"开子代理完成剩余的"（技术选型TBD收尾）。新增§7解决TBD-SEC-001（DDoS/WAF最终选型：OpenResty+Coraza+OWASP CRS，均OSI许可）与§8解决TBD-SEC-002（密钥管理中间件最终选型：OpenBao，HashiCorp Vault的Linux Foundation治理MPL-2.0开源分支，因Vault主线已转为非开源BSL许可故不选用Vault本体）。原§7覆盖范围章节顺延为§9并更新内容 | §1.2、§7（新增）、§8（新增）、原§7→§9 |
| 0.3 | 2026-08-25 | 架构师（Ulysses（一人公司 12 角色 per DEC-008））| — | 同步父 BAS-006 升版至 v0.3 + 补 AC-SEC-006/007/008 追溯性行 | 追溯性表 |

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 | 2026-08-17 | — |
| 评审（技术） | | | NetworkPolicy YAML模板是否真正默认拒绝无遗漏，速率限制Redis key设计是否与既有ARC-012缓存基础设施冲突 |
| 评审（安全） | | | §7A.1未信任输入解析安全实现是否覆盖RGS-BAS-006原文全部四类禁止操作，配额校验伪代码是否确实无TOCTOU窗口 |
| 审批（负责人） | | | 本文档的基准化；TBD-SEC-003速率限制阈值默认值提案是否可直接采纳 |

---

## 目录

1. [前言](#1-前言)
2. [NetworkPolicy基线模板具体清单](#2-networkpolicy基线模板具体清单)
3. [未信任输入解析安全实现详细设计](#3-未信任输入解析安全实现详细设计)
4. [多层速率限制算法详细设计](#4-多层速率限制算法详细设计)
5. [游戏内资源配额算法详细设计](#5-游戏内资源配额算法详细设计)
6. [供应链安全CI阶段配置骨架](#6-供应链安全ci阶段配置骨架)
7. [DDoS/WAF选型（TBD-SEC-001）](#7-ddoswaf选型tbd-sec-001最终决定)
8. [密钥管理中间件选型（TBD-SEC-002）](#8-密钥管理中间件选型tbd-sec-002最终决定)
9. [本文档的覆盖范围与后续计划](#9-本文档的覆盖范围与后续计划)

---

## 1. 前言

### 1.1 定位

RGS-BAS-006给出了分层安全架构总览、NetworkPolicy基线原则的文字表述、密钥轮换时序图、供应链安全流水线的阶段表格、§7A未信任输入解析安全的禁止操作清单与多层速率限制/资源配额的流程图与设计要点文字描述。本文档将其落实为：具体的NetworkPolicy YAML模板、Rust层面强制未信任输入安全规则的CI lint配置与代码模式、速率限制的具体Redis数据结构与算法伪代码、资源配额校验的算法级实现、供应链安全CI阶段的具体配置骨架。

### 1.2 本文档不做什么

- 不重新决定RGS-BAS-006已确定的任何结构性选择（默认拒绝的零信任网络基线、纵深防御分层架构、配额校验寄生于既有确定请求路径而非独立机制、QUIC地址验证复用协议自身Retry机制）。
- DDoS/WAF具体选型（TBD-SEC-001）与密钥管理中间件选型（TBD-SEC-002）已于v0.2在§7/§8给出最终决定（OpenResty+Coraza+OWASP CRS；OpenBao），不再是本文档遗留缺口。
- 不覆盖构建溯源签名的具体密码学方案（签名算法/密钥管理）——RGS-BAS-006§6已注明"具体实现留详细设计"，本文档只固定CI流水线中该阶段的位置与校验触点（§6），不选定签名算法本身。
- 不覆盖RGS-DTL-003/004/005中已详细设计的内容（如GM指令强制全采集判定、维护模式收敛算法）——本文档聚焦网络安全域自身职责范围，跨域复用点仅标注引用关系不重复展开。

### 1.3 记述规则

沿用既有DTL文档记述规则：NetworkPolicy以Kubernetes YAML给出，算法伪代码可直接对应Rust `Result`实现，CI配置以GitHub Actions风格骨架给出（复用RGS-BAS-002§4.2既有CI/CD骨架约定，不新建流水线体系）。

---

## 2. NetworkPolicy基线模板具体清单

对应RGS-BAS-006§4.1"每个服务的Helm chart必须包含其自身的networkpolicy.yaml"，落实为具体模板结构。

### 2.1 Namespace级默认拒绝基线

```yaml
# 由脚手架在Namespace创建时自动生成(RGS-BAS-006§4.1"由脚手架自动生成"，
# 复用RGS-BAS-002§4.1骨架产出机制，不新建独立生成流程)
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: default-deny-all
  namespace: "{{ .Values.namespace }}"
spec:
  podSelector: {}          # 空选择器 = 匹配该Namespace内全部Pod
  policyTypes:
    - Ingress
    - Egress
  # 无ingress/egress规则条目 = 默认拒绝全部流量(K8s NetworkPolicy语义:存在policyTypes但无对应rules即为该方向全拒绝)
```

### 2.2 服务级显式声明模板（Helm chart随服务生成，复用RGS-BAS-002§5.2模板结构）

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: "{{ .Values.serviceName }}-allow"
  namespace: "{{ .Values.namespace }}"
spec:
  podSelector:
    matchLabels:
      app: "{{ .Values.serviceName }}"
  policyTypes:
    - Ingress
    - Egress
  ingress:
    # 上游调用方列表由服务自身声明的依赖清单驱动生成(同RGS-BAS-005 declared_dependencies的同类思路:
    # 显式声明而非隐式放通)，本骨架以经济服务(EC)为例
    - from:
        - podSelector:
            matchLabels:
              app: admin-service       # 示例上游:AdminService经补偿发放gRPC调用EC
      ports:
        - protocol: TCP
          port: 50051                  # gRPC端口，具体值随实际部署配置
  egress:
    - to:
        - podSelector:
            matchLabels:
              app: economy-db-proxy    # 示例下游:所依附数据库连接池代理
      ports:
        - protocol: TCP
          port: 5432
    - to:
        - podSelector:
            matchLabels:
              app: otel-collector      # 全部服务均须放通至OTel Collector的出站(RGS-BAS-004既有埋点数据流)
      ports:
        - protocol: TCP
          port: 4317
```

**模板参数化说明**：`ingress.from`与`egress.to`条目由Helm chart的`values.yaml`中该服务声明的上游调用方/下游依赖列表渲染生成，非本文档模板本身写死具体服务名——本文档固定的是**模板结构与渲染规则**，具体服务的上下游关系随各服务自身设计文档（如RGS-DTL-001§4 PlayerService/EconomyService的调用关系）而定。已有实例：RGS-BAS-003§4.4"运行时受限控制通道"的NetworkPolicy即按本模板生成，入站仅放通`AdminService`所在Pod标签，出站不适用（该通道被动接收，不主动发起出站）。

### 2.3 CI覆盖率检查具体实现

```yaml
# 复用RGS-BAS-002§4.2既有CI/CD骨架的lint阶段，新增检查步骤
- name: verify-networkpolicy-exists
  run: |
    if [ ! -f "charts/${SERVICE_NAME}/templates/networkpolicy.yaml" ]; then
      echo "::error::networkpolicy.yaml missing for ${SERVICE_NAME}, blocked by RGS-BAS-006§4.2"
      exit 1
    fi
    # 非空校验:确认渲染后的YAML确实包含非空的ingress/egress rules,而非仅有policyTypes声明的空壳
    helm template "charts/${SERVICE_NAME}" | yq eval-all 'select(.kind == "NetworkPolicy")' - \
      | grep -q 'ingress:\|egress:' || { echo "::error::networkpolicy.yaml renders empty rules"; exit 1; }
```

定期审计（RGS-BAS-006§4.2"每周扫描"）的判定逻辑：扫描集群内全部Namespace/Pod标签组合，核对每个`app`标签值是否存在对应的`NetworkPolicy`资源匹配其`podSelector`，缺失项经RGS-DTL-003§3已定义的`QueryHealthView`同类告警链路（复用RGS-BAS-003§6告警推送机制）上报，本文档不新建独立告警通道。

---

## 3. 未信任输入解析安全实现详细设计

对应RGS-BAS-006§7A.1禁止操作清单与CI强制手段。

### 3.1 CI lint强制配置

```rust
// 解析模块(客户端QUIC消息/API网关HTTP请求体的解析代码路径)顶部声明,
// 复用RGS-BAS-002§4.2既有CI测试门禁基础设施新增lint规则,而非新建检查工具(RGS-BAS-006原文既定约束)
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::integer_arithmetic,   // 覆盖"未检查的整数运算"类别,强制使用checked_*系列
)]
```

### 3.2 四类禁止操作的替代实现模式

```rust
// 类别1: 数组/切片裸索引替代
fn parse_header_field(bytes: &[u8], offset: usize) -> Result<u8, ParseError> {
    bytes.get(offset).copied().ok_or(ParseError::TruncatedInput { offset })
    // 替代 bytes[offset]，越界返回Option::None后经ok_or显式转换为ParseError，符合"Result<T,ParseError>唯一出口"要求
}

// 类别2: 未检查整数运算替代
fn compute_payload_length(header_len: u32, declared_len: u32) -> Result<u32, ParseError> {
    header_len.checked_add(declared_len).ok_or(ParseError::LengthOverflow)
    // 替代 header_len + declared_len，溢出即判定为畸形输入而非panic或wrap-around静默错误
}

// 类别3: 递归深度受限解析
struct ParseDepthGuard {
    current_depth: u32,
    max_depth: u32,   // 具体上限值随消息schema而定,本文档不预设统一常量,由各解析模块按自身嵌套语义声明
}
impl ParseDepthGuard {
    fn enter(&mut self) -> Result<(), ParseError> {
        self.current_depth += 1;
        if self.current_depth > self.max_depth {
            return Err(ParseError::MaxDepthExceeded { max: self.max_depth });
        }
        Ok(())
    }
    fn exit(&mut self) {
        self.current_depth -= 1;   // 减法在guard自身语义下不会下溢(enter先于exit调用是不变式)，本行不属于"未信任输入运算"范畴，无需checked_sub
    }
}

// 类别4: .unwrap()/.expect()替代——统一以?操作符向上传播
fn parse_quic_frame(bytes: &[u8]) -> Result<QuicFrame, ParseError> {
    let frame_type = bytes.first().ok_or(ParseError::EmptyInput)?;
    let body = parse_frame_body(*frame_type, &bytes[1..])?;  // 注意: 此处切片bytes[1..]范围已由上一行get()保证bytes非空,
                                                                // 但仍应使用bytes.get(1..).unwrap_or(&[])以完全消除裸索引(即便逻辑上安全,
                                                                // clippy::indexing_slicing仍会标记，CI层面统一按"禁止裸索引"处理，不做例外豁免)
    Ok(QuicFrame { frame_type: *frame_type, body })
}
```

### 3.3 模糊测试CI接入骨架

```yaml
# 复用RGS-BAS-002§4.2既有CI骨架，新增定期(非每次提交)执行的模糊测试job
# 具体周期(如每日/每周一次)留待详细设计确定的运维排期，本骨架仅固定触发方式与失败判定
- name: scheduled-fuzz
  schedule: "daily"          # 提案:每日一次,非最终值,可随CI资源预算调整
  run: |
    cargo fuzz run parse_quic_frame -- -max_total_time=1800  # 提案:单次运行30分钟，非最终值
    cargo fuzz run parse_http_request_body -- -max_total_time=1800
  on_panic: fail_build         # 任何panic产生即视为构建失败级别缺陷(RGS-BAS-006原文明确"不得降级为已知问题延后修复")
```

---

## 4. 多层速率限制算法详细设计

对应RGS-BAS-006§7A.2三层限流表格，落实为账号级限流的具体算法与存储设计。

### 4.1 Redis数据结构

```
key: ratelimit:{player_id}:{api_category}
value: 滑动窗口内的请求计数(整数，Redis INCR原子操作)
TTL: 与窗口大小一致(如60秒)，到期自然清零，不引入额外定时清理任务(RGS-BAS-006原文既定设计)
```

### 4.2 账号级限流判定算法

```rust
fn check_account_rate_limit(
    player_id: &str,
    api_category: ApiCategory,
    redis: &RedisClient,
    limits: &RateLimitConfig,
) -> Result<RateLimitDecision, RateLimitError> {
    let key = format!("ratelimit:{}:{}", player_id, api_category.as_str());
    let window_secs = limits.window_secs_for(api_category);
    let threshold = limits.threshold_for(api_category);

    // INCR+EXPIRE的原子性: 使用Redis Lua脚本或MULTI/EXEC保证"计数递增"与"首次设置TTL"不产生竞态
    // (若INCR后单独判断是否首次出现再EXPIRE，两步操作间存在极短的竞态窗口，虽然影响有限但仍应避免)
    let current_count: i64 = redis.eval(
        r#"
        local c = redis.call('INCR', KEYS[1])
        if c == 1 then
            redis.call('EXPIRE', KEYS[1], ARGV[1])
        end
        return c
        "#,
        &[key.as_str()],
        &[window_secs.to_string()],
    )?;

    if current_count > threshold {
        emit_metric("rgs_rate_limit_rejected_total", 1.0);  // 复用RGS-DTL-004既定指标API,标签为api_category(非高基数)
        // 记录安全埋点(RGS-BAS-006§7A.2 REJ2既定"记录FR-SEC-040安全埋点")
        log_warn_security_event("account_rate_limit_exceeded", player_id, api_category);
        return Ok(RateLimitDecision::Reject);
    }
    Ok(RateLimitDecision::Allow)
}
```

**边界条件说明**：`api_category`按类别独立计数（RGS-BAS-006原文既定"不同类别正常频率差异巨大，共用计数器会互相干扰"），本文档在§4.3给出该维度划分与阈值的初始提案；连接级限流（L1）复用ARC-013既有背压机制，其判定发生在协议层握手/连接建立阶段，早于本节账号级判定，本文档不重复展开该层；IP级限流（L3）复用边界层DDoS/WAF内建能力（§3边界防护，具体产品选型TBD-SEC-001），本文档同样不重复展开。

### 4.3 TBD-SEC-003：API类别划分与阈值初始提案

RGS-BAS-006§7A.2标注"具体类别划分与阈值TBD-SEC-003确定"。本文档提出以下初始默认值供上线使用，非最终值，与RGS-DTL-025§5、RGS-DTL-026§4.1、RGS-DTL-003§6同类做法一致：

| API类别 | 提案窗口 | 提案阈值 | 依据 |
|---|---|---|---|
| `TRADE`（交易/经济操作） | 60秒 | 30次/窗口 | 正常玩家交易频率远低于该值，留出充分余量避免误伤，同时显著低于脚本刷量的典型频率 |
| `CHAT`（聊天消息） | 10秒 | 20次/窗口 | 覆盖正常连续对话场景，防止刷屏类滥用 |
| `PARTY_INVITE`（组队邀请） | 60秒 | 10次/窗口 | 组队邀请属低频操作，正常场景很少短时间内连续邀请超过个位数 |
| `SOCIAL_ACTION`（好友申请/公会操作等） | 60秒 | 15次/窗口 | 与`PARTY_INVITE`同量级社交类操作，略高阈值容纳好友批量处理场景 |

移动/战斗指令类高频操作**不**在本表范围内——已由连接级限流（L1，复用ARC-013背压）覆盖，同RGS-BAS-006原文"移动指令已由连接级限流覆盖"的既定说明，账号级限流刻意不重复覆盖该类别，避免两层限流对同一高频路径产生冗余判定开销。以上阈值应在PH-4阶段结合真实误伤率数据（AC-SEC-007验收标准）校准。

---

## 5. 游戏内资源配额算法详细设计

对应RGS-BAS-006§7A.3，落实为具体校验点插入位置的伪代码，确保TOCTOU-free。

### 5.1 经济类资源配额（寄生于CommitTransaction）

```rust
// 配额校验作为CommitTransaction既有事务的前置条件之一,与expected_version OCC校验在同一UPDATE语句的WHERE子句内完成,
// 复用RGS-DTL-001§3.2既定"确定请求物理执行语义"事务边界,不新增独立配额检查步骤
fn commit_transaction_with_quota(req: &CommitTransactionRequest) -> Result<CommitTransactionResponse, EcError> {
    if let TransactionOp::GrantItem { quantity, .. } = &req.operation {
        // 配额上限查询与后续UPDATE处于同一事务(BEGIN已开启,同RGS-DTL-001§3.2 BEGIN/COMMIT边界)，
        // 避免"查询配额已用量"与"实际写入"之间出现可被并发请求利用的窗口——
        // 关键点: 配额判定条件同样写入UPDATE的WHERE子句，而非查询后在应用层if判断再决定是否继续，
        // 这样即便高并发下多个请求同时查到"配额未满"，最终只有WHERE条件仍成立的那些请求能实际生效
        let quota_check_sql = "UPDATE inventory_quota SET used = used + $1 
                                WHERE character_id = $2 AND item_category = $3 
                                  AND used + $1 <= quota_limit";
        let rows = execute_in_tx(quota_check_sql, &[*quantity, req.character_id.clone(), item_category(&req.operation)])?;
        if rows == 0 {
            return Err(EcError::QuotaExceeded);  // 事务内失败即整体回滚(同RGS-DTL-001§3.2既定"整个事务边界"设计)
        }
    }
    execute_commit_transaction_tx(req)  // 复用既有OCC+流水写入语义，不重复展开
}
```

### 5.2 场景内资源配额（寄生于场景Actor单一写入者）

```rust
// 配额计数是场景Actor内存状态的一部分(ARC-005单一写入者特性天然保证无并发),
// 校验与生成在同一tick内的同一同步代码路径完成,复用RGS-DTL-001§5.1 tick循环结构中的战斗判定阶段(阶段3)
fn spawn_entity_with_quota(scene: &mut SceneState, spawner_id: EntityId, entity_kind: EntityKind) -> Result<EntityId, SpawnError> {
    let current_count = scene.entity_count_by_spawner(spawner_id, entity_kind);
    let limit = quota_limit_for(entity_kind);   // 静态配置，同ARC-016数值表来源，非动态查询

    if current_count >= limit {
        // O(1)校验:current_count来自场景Actor内存中已维护的计数(非逐一遍历实体列表统计),生成时同步递增该计数
        return Err(SpawnError::QuotaExceeded { entity_kind, limit });
    }
    let entity_id = scene.spawn_entity(spawner_id, entity_kind);  // 生成与计数递增同一同步调用内完成，无中间可插入并发操作的窗口
    scene.increment_spawner_count(spawner_id, entity_kind);
    Ok(entity_id)
}
```

**O(1)校验的具体落地方式**（RGS-BAS-006原文NFR-SEC-009要求）：`current_count`不通过实时扫描/聚合查询获得，而是场景Actor在每次生成/销毁实体时增量维护的计数器（`HashMap<(EntityId, EntityKind), u32>`同量级的内存结构），校验本身是一次哈希表查找，量级为O(1)，而非O(该spawner已生成实体数)的遍历统计。

---

## 6. 供应链安全CI阶段配置骨架

对应RGS-BAS-006§6表格，落实为CI阶段的具体配置骨架（纳入RGS-BAS-002§4.2既有骨架，不新建独立流水线）。

```yaml
# 复用RGS-BAS-002§4.2既有CI/CD骨架，本文档新增以下阶段（加粗内容对应原骨架"lint/test"与"镜像构建"之间）

- name: dependency-vulnerability-scan
  run: cargo audit --deny warnings   # 或cargo-deny，具体工具二选一留待实现阶段（均OSI许可，符合CON-001）
  # 阻断级别: High以上14天内须处理、Critical 72小时内须处理(NFR-SEC-003)
  # 本文档不在CI阶段直接阻断构建("发现漏洞"与"必须在N天内处理"是两个不同的时间尺度约束，
  # 前者适合CI即时报告，后者适合独立的漏洞跟踪工单流程，故本步骤失败策略为:Critical发现即failed_build，
  # High发现记录但not fail_build，转入漏洞跟踪(具体跟踪机制留待详细设计)

- name: sbom-generation
  run: |
    syft packages dir:. -o spdx-json > sbom.spdx.json   # 或等价OSI许可SBOM工具
  artifact: sbom.spdx.json   # 随镜像一同归档

- name: build-provenance-signing
  run: |
    # 附加来源证明:构建流水线ID/源码commit/构建时间
    # 具体签名算法(如Sigstore cosign/自建方案)留待详细设计确定，此处仅固定"生成一份可校验的来源声明文件"这一契约
    cosign attest --predicate provenance.json "$IMAGE_REF"   # 示例命令，非最终工具选型结论

- name: image-build   # 既有
- name: helm-lint-dryrun   # 既有

- name: deploy-provenance-gate
  # 生产环境部署前的准入控制:校验镜像携带的来源证明存在且来自可信流水线，
  # 未经证明的镜像不得部署(具体准入控制机制——K8s ValidatingAdmissionPolicy或OPA Gatekeeper等——留待详细设计确定)
  run: |
    cosign verify-attestation --key "$TRUSTED_PUBKEY" "$IMAGE_REF" || exit 1
```

---

## 7. DDoS/WAF选型（TBD-SEC-001，最终决定）

按"全部采用开源免费策略"约束，选型为**OpenResty（nginx+LuaJIT，BSD-2-Clause）前置层 ＋ Coraza WAF引擎（Apache-2.0，OWASP CRS兼容）**，取代此前未定的商业WAF假设：

- **DDoS抗性层**：OpenResty作为边界反向代理，接入既有`limit_req`/`limit_conn`模块做连接级速率整形（早于本文档§4账号级限流生效，对应§4.3边界条件说明既定的L3拓扑位置），SYN flood等L3/L4层攻击依赖既有云基础设施/自托管BGP黑洞或`iptables`+`fail2ban`组合，不额外引入商业清洗服务。
- **WAF层**：Coraza（Apache-2.0，Go编写，可编译为独立模块接入OpenResty或作为standalone反向代理）加载OWASP Core Rule Set（CRS，Apache-2.0）作为规则基线，覆盖SQL注入/XSS/已知CVE模式等通用Web攻击特征——本系统客户端-服务端主协议为自定义二进制/gRPC而非HTTP表单，故WAF主要防护面是GM后台Web界面与§3清单查询等HTTP接口（RGS-DTL-027§3），非游戏主协议本身（游戏主协议的滥用防护由本文档§4/§5账号级限流与资源配额承担，两者互补而非重叠）。
- **部署位置**：OpenResty+Coraza部署于RGS-BAS-006§2既有边界层（NetworkPolicy拓扑中DDoS/WAF既定位置不变，本次只是给该位置填入具体产品），随集群Ingress一并管理，不改变§2模板结构本身。

license确认：OpenResty（BSD-2-Clause）、Coraza（Apache-2.0）、OWASP CRS（Apache-2.0）均为OSI认可宽松许可，无CON-001顾虑，无需与MinIO类AGPL场景一样额外论证。

## 8. 密钥管理中间件选型（TBD-SEC-002，最终决定）

选型为**OpenBao**（HashiCorp Vault的Linux Foundation治理开源分支，MPL-2.0，2023年HashiCorp将Vault改为BSL非开源许可后由社区fork延续的开源版本），而非HashiCorp Vault本体（其当前主线许可BSL-1.1不满足CON-001开源要求）。

- **接入方式**：OpenBao作为RGS-DTL-002§2.4`ExternalSecret`资源指向的`ClusterSecretStore`后端实现，其K8s Secrets同步能力通过External Secrets Operator既有的Vault-provider适配器直接兼容（OpenBao保持与Vault API的向后兼容，无需额外适配层）。
- **密钥轮换落地**：RGS-BAS-006§5既有轮换时序图中"密钥管理系统"角色由OpenBao承担，轮换动作通过OpenBao的动态密钥引擎（database secrets engine等）或版本化KV存储配合既有轮换调度触发，具体轮换脚本留待实现阶段。
- **自托管形态**：OpenBao以自身的Raft存储后端自托管（不依赖外部KV如Consul），随集群一并部署，符合"全部采用开源免费策略"与"不假设付费SaaS"两项约束。

license确认：OpenBao MPL-2.0（弱copyleft，仅约束对OpenBao自身源码文件的修改需开源，不影响调用方/本项目其余代码许可），OSI认可，无CON-001顾虑。

---

## 9. 本文档的覆盖范围与后续计划

本文档覆盖：NetworkPolicy基线模板的具体YAML结构（Namespace级默认拒绝+服务级显式声明模板+CI覆盖率检查实现）、未信任输入解析安全的具体CI lint配置与四类禁止操作的Rust替代实现模式、模糊测试CI接入骨架、账号级速率限制的Redis数据结构与判定算法（含TBD-SEC-003 API类别划分与阈值初始提案）、经济类/场景类资源配额校验的TOCTOU-free算法实现、供应链安全CI阶段的具体配置骨架、**DDoS/WAF最终选型（TBD-SEC-001：OpenResty+Coraza+OWASP CRS）**、**密钥管理中间件最终选型（TBD-SEC-002：OpenBao）**。

本版本明确不覆盖、留待后续：

- TBD-SEC-003限流阈值的最终校准数值——本文档§4.3给出的是初始提案，需PH-4/PH-5实测数据支撑最终校准，本次选型解决的是TBD-SEC-001/002而非TBD-SEC-003。
- OpenResty/Coraza/OpenBao的具体K8s部署manifest（Helm values细节）——本文档只确定选型与接入契约，具体部署配置留待实现阶段按RGS-DTL-002§2既定模板套用。
- OpenBao轮换脚本的具体实现代码。
- 构建溯源签名的具体密码学方案与准入控制机制的最终选型（§6骨架中的`cosign`/`ValidatingAdmissionPolicy`均为示例，非最终工具选型结论）。
- §4.3速率限制阈值与§6模糊测试运行周期/时长的正式校准值——均为初始提案，需PH-4/PH-5实测数据支撑校准。
- 安全事件响应流程骨架（RGS-BAS-006§7）人工判断标准与升级路径——RGS-BAS-006原文已明确该部分属RGS-OPS-001运维手顺书职责范围，本文档不涉及。

后续详细设计建议顺序：本文档§4速率限制的安全埋点与RGS-DTL-004§5强制全量采集判定存在耦合（速率限制拒绝事件应计入"降级/背压拒绝路径"类别），建议与RGS-DTL-004交叉核对；§5资源配额校验依赖的经济事务边界已由RGS-DTL-001§3.2详细设计覆盖，两者已具备一致的物理执行语义基础，无需额外协调。

---

## 追溯性

| 需求/设计来源 | 本文档章节 |
|---|---|
| RGS-BAS-006§2 分层安全架构总览 | 前提依赖，本文档假定分层拓扑不变 |
| RGS-BAS-006§3 边界防护设计 | §4.2（IP级限流复用点，不重复展开） |
| RGS-BAS-006§4 NetworkPolicy基线模板 | §2 |
| RGS-BAS-006§5 密钥与证书轮换设计 | §8（TBD-SEC-002最终选型：OpenBao） |
| RGS-BAS-006§6 供应链安全流水线设计 | §6 |
| RGS-BAS-006§7 安全事件响应流程骨架 | §9（明确排除，属RGS-OPS-001） |
| RGS-BAS-006§3 边界防护设计（DDoS/WAF拓扑位置） | §7（TBD-SEC-001最终选型：OpenResty+Coraza+OWASP CRS） |
| RGS-BAS-006§7A.1 未信任输入解析安全 | §3 |
| RGS-BAS-006§7A.2 多层速率限制设计 | §4 |
| RGS-BAS-006§7A.3 游戏内资源配额设计 | §5 |
| RGS-BAS-006§7A.4 QUIC地址验证设计 | 复用协议自身能力，本文档不展开（原文已明确"不需要自定义实现"） |
| RGS-BAS-006§7A.5 崩溃循环退避确认 | 复用RGS-DTL-005§6同款熔断退避算法思路，本文档不重复 |
| TBD-SEC-003（速率限制类别与阈值） | §4.3 |
| RGS-DTL-001§3.2 CommitTransaction物理执行语义 | §5.1（复用） |
| RGS-DTL-001§5.1 tick循环结构 | §5.2（复用） |
| AC-SEC-006（模糊测试无panic） | §3.3 模糊测试CI接入骨架（cargo fuzz + on_panic: fail_build） |
| AC-SEC-007（账号级限流生效且不误伤其他账号） | §4.3 TBD-SEC-003 API类别划分与阈值初始提案（按api_category独立计数） |
| AC-SEC-008（服务器侧资源配额上限生效） | §5.1 经济类配额（CommitTransaction同事务内完成）+ §5.2 场景内配额（场景Actor内存计数） |
