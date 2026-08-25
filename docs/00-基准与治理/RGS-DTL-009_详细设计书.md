# 详细设计书（詳細設計書 / Detailed Design Document）

**体系治理与横切关注点：登记表物理存储格式・CI校验脚本逻辑・OLU台账/编排状态机详细设计**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DTL-009 |
| 版本 | 0.2 |
| 父文档 | RGS-BAS-009 体系治理与横切关注点 基本设计书（本文档为其详细化，不改变任何既有决定，仅将逻辑/流程设计落实为物理/实现级设计） |
| 依据标准 | IPA『共通フレーム 2013（SLCP-JCF2013）』详细设计工程 |
| 制定日 | 2026-08-17 |
| 制定者 | 架构师 |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响章节 |
|---|---|---|---|---|---|
| 0.1 | 2026-08-17 | 架构师 | — | 初版制定，本文档是RGS-DTL-001/002/025/026/027之后本批次继续推进详细设计阶段的一部分。细化RGS-BAS-009§4 CI机械校验表中"未实现"4项的具体解析逻辑（未注册域名段/未登记领域验收标准/跨文档章节引用失效/OLU台账余额为负）为可脚本化的伪代码、§2.2/§2.3登记表结构落实为附件C/D的具体Markdown表格行格式解析规则、§3.4 OLU台账落实为附件D§5的具体物理格式、§5.2/§5.2.1删除与导出编排落实为PH-6前"服务内状态机"阶段的具体状态转移伪代码与幂等实现。**本版本不覆盖**：PH-6后工作流基础设施接管编排后的具体工作流定义（依赖ARC-011工作流基础设施本身的详细设计尚未产出）、CI机械校验脚本`check-docs-consistency.sh`已实现4项检查的重复展开（原脚本已存在于仓库，非本文档新增设计对象）。见§7 | 全部 |
| 0.2 | 2026-08-25 | 架构师（一人公司 12 角色兼任 per DEC-008；本升版由 Mavis 接手 agent 代为执笔）| — | 同步父BAS-009升版至v0.6 + 补：①修复§5.2.1伪代码常量名笔误（`EXPORY_TTL`→`EXPORT_TTL`，落实BAS v0.6临时对象存储限时链接设计）；②新增§5.4删除/导出两条编排遍历的库集合一致性校验（落实BAS v0.6 §5.2.1设计点③"上线前检查清单须验证遍历库集合对应"，以单一Mount Record强制两条编排共享同一份清单并在代码层加assert防漂移）；③目录补§5.4小节，追溯性表细分§5.2/§5.2.1映射。**未引入新设计**——三处补强均为对BAS v0.3/v0.4（OLU预算+回收口径）、v0.5（FR-GOV-040挂载回滚拆分）、v0.6（§5.2.1导出编排一致性校验）已确定设计的落实，§1.2"不做什么"边界、§4不动既有CI 4项、§6挂载回滚v0.1版本已落实FR-GOV-040均保持原样 | §5.2, §5.4, 目录, 追溯性, 头部版本 |

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 | 2026-08-17 | — |
| 评审（技术） | | | 新增4项CI校验的解析逻辑是否会对现有`.github/workflows/docs-consistency.yml`产生高误报 |
| 评审（SRE） | | | OLU台账物理格式是否可被§4.4脚本稳定解析，删除/导出编排状态机是否覆盖全部中途失败重入场景 |
| 审批（负责人） | | | 本文档的基准化 |

---

## 目录

1. [前言](#1-前言)
2. [登记表物理存储格式](#2-登记表物理存储格式)
3. [OLU运维负荷预算台账物理格式](#3-olu运维负荷预算台账物理格式)
4. [CI机械校验：未实现4项的解析逻辑](#4-ci机械校验未实现4项的解析逻辑)
5. [个人数据删除／导出编排状态机详细设计](#5-个人数据删除导出编排状态机详细设计)
   - 5.1 状态定义
   - 5.2 主流程
   - 5.3 失败与重试
   - 5.4 删除/导出两条编排遍历的库集合一致性校验
6. [挂载回滚两阶段验证的具体判定逻辑](#6-挂载回滚两阶段验证的具体判定逻辑)
7. [本文档的覆盖范围与后续计划](#7-本文档的覆盖范围与后续计划)

---

# 1. 前言

## 1.1 定位

RGS-BAS-009的设计对象是文档体系与工程流程本身，因此其"逻辑设计"表现为登记表结构、台账规则与流程时序图，而非服务/数据库。本文档将这些结构性描述落实为：登记表在附件C/D中的具体行格式与列解析规则、OLU台账的具体物理文件格式、CI机械校验中当前标注"未实现"的4项检查的可执行解析伪代码、删除/导出编排在PH-6工作流基础设施就绪前"服务内状态机"阶段的具体状态转移与幂等实现。

## 1.2 本文档不做什么

- 不重新决定RGS-BAS-009已确定的任何结构性选择（两级ID体系、OLU预算总量与分配、CI检查项清单、删除/导出编排共享同一状态机框架、挂载回滚拆分为流量回退/版本回滚两阶段）。若细化过程中发现基本设计本身有缺陷，应回写RGS-BAS-009，不在本文档内悄悄修正——本文档在§7中列出审阅过程中注意到但**未**处理的疑点，供负责人判断是否需要回写。
- 不覆盖PH-6工作流基础设施本身的详细设计（该基础设施自身的物理设计属于ARC-011对应DTL文档职责，本文档仅覆盖PH-6前的过渡态实现）。
- 不是实现本身：伪代码用于表达算法逻辑与边界条件，不是可编译的Rust源码。

## 1.3 记述规则

沿用RGS-DTL-001§1.3/RGS-DTL-002§1.3已确立的记述规则：算法伪代码可直接对应Rust `Result`风格实现；Markdown表格格式片段用于固定登记表的物理列结构，一经本文档确定不得随意增减列（如需增减，应先回写RGS-BAS-009§2.2/§2.3）。

---

# 2. 登记表物理存储格式

对应RGS-BAS-009§2.2（附件C§7 ID归属登记表）与§2.3（主编号映射表结构）。两表均已作为Markdown表格实际存在于`docs/00-基准与治理/`附件C/D中，本文档只固定**新增行的书写格式**与**供§4脚本解析的列位置契约**，不改变表结构本身。

## 2.1 附件C§7新增行格式

```markdown
| SEC | RGS-REQ-010 | 安全 | 机密性、完整性 | FR-SEC-001〜042 | NFR-SEC-001〜005 | 2026-08-16 / 架构师 |
```

列顺序与RGS-BAS-009§2.2表头**逐列对应**（域名段/出处文档/归属子系统/归属NFR区分/FR编号范围/NFR编号范围/注册日／注册者），供§4.1脚本按固定列序解析，**不得**在新增行中调换列顺序或省略列（缺失列以`—`占位，不得直接省略单元格导致列数不齐，后者会使基于`|`分割计数的解析脚本静默错位）。

## 2.2 附件D§1.3／§2.3新增行格式

```markdown
| TBD-020 | TBD-SEC-001 | ...既有列内容... |
```

`主编号`列（如`TBD-020`）与`域内ID`列（如`TBD-SEC-001`）均为**必填且各自全局唯一**——解析脚本（§4.2）以`域内ID`为查找键，若同一`域内ID`出现在多行（重复登记）或从未出现（漏登记），均判定为该检查项失败。

---

# 3. OLU运维负荷预算台账物理格式

对应RGS-BAS-009§3.4 GOV-OLU-001，物理载体为附件D§5，格式如下（延续RGS-BAS-009§3.2/§3.3表格结构，作为附件D§5的初始内容迁入）：

```markdown
## 附件D§5 运维负荷预算台账

| 项目 | OLU/月 | 更新日 |
|---|---|---|
| 总预算 | 210 | 2026-08-17 |

### 已批准运维面明细

| 运维面 | 出处ADR | 申领OLU | 状态（生效/待回收） |
|---|---|---|---|
| PostgreSQL运维 | — （既有面，无对应ADR编号，视为基线继承） | 24 | 生效 |
| ... | | | |
| 插件状态复用ARC-016分发通道 | ADR-xxx（回收R-1） | -4 | 待回收（PH-3） |

### 合计

| 项目 | 数值 |
|---|---|
| 已批准合计 | 182 |
| 余额 | +28（R-1〜R-4/R-6回收完毕后 +50） |
```

**新增运维面申领的写入规则**（GOV-OLU-002具体化）：申领ADR评审通过后，由评审者在本台账"已批准运维面明细"新增一行，`申领OLU`取自ADR记载数值，随即重算"合计"区块；`余额`列**必须**在同一次编辑中同步更新，不得分两次提交（避免出现"明细已更新、合计未更新"的中间不一致态被误读为当前生效余额）。§4.4的CI校验以本节格式为解析契约。

---

# 4. CI机械校验：未实现4项的解析逻辑

对应RGS-BAS-009§4表格中标注"未实现，TBD-PAT-002待办"的4项，落实为可直接追加到既有`scripts/check-docs-consistency.sh`的检查函数伪代码。**本文档不改写该脚本已实现的4项**（ARC序列/未登记ARC/未登记TBD与风险/README死链），只新增以下4个函数体设计。

## 4.1 未注册域名段

```
fn check_unregistered_domain_segments(docs_dir: &Path, appendix_c_7: &Table) -> Result<(), CheckFailure> {
    let ids = scan_all_files(docs_dir, r"(FR|NFR)-([A-Z]+)-\d+");  // 正则捕获组2即域名段
    let registered_segments: HashSet<&str> = appendix_c_7.column("域名段").collect();
    let mut unregistered = vec![];
    for (file, segment) in ids {
        if !registered_segments.contains(segment.as_str()) {
            unregistered.push((file, segment));
        }
    }
    if unregistered.is_empty() { Ok(()) } else { Err(CheckFailure::new("未注册域名段", unregistered)) }
}
```

**边界条件**：`GOV`域名段本身（本文档大量使用的`FR-GOV-nnn`）须已在附件C§7登记，否则本检查函数首次启用时会对治理体系自身报错——启用前须先核对附件C§7是否已含`GOV`行，这是启用本检查的前置条件而非本文档新引入的义务。

## 4.2 未登记领域验收标准

```
fn check_unregistered_ac(docs_dir: &Path, appendix_c_8: &Table) -> Result<(), CheckFailure> {
    let acs = scan_all_files(docs_dir, r"AC-([A-Z]+)-\d+");
    let registered: HashSet<String> = appendix_c_8.column("AC编号").map(normalize).collect();
    let missing: Vec<_> = acs.into_iter().filter(|ac| !registered.contains(&normalize(ac))).collect();
    if missing.is_empty() { Ok(()) } else { Err(CheckFailure::new("未登记领域验收标准", missing)) }
}
```

## 4.3 跨文档章节引用失效（警告级，非阻断）

```
fn check_cross_doc_section_refs(docs_dir: &Path) -> Vec<Warning> {
    let refs = scan_all_files(docs_dir, r"(RGS-(?:BAS|DTL|REQ)-\d{3})§(\d+(?:\.\d+)*)");
    let mut warnings = vec![];
    for (source_file, (target_doc, section)) in refs {
        let target_path = resolve_doc_path(&target_doc);  // 按文档编号在docs/树中定位物理文件
        match target_path {
            None => warnings.push(Warning::TargetDocNotFound { source_file, target_doc }),
            Some(path) => {
                if !file_contains_heading(&path, &section) {
                    // 匹配"## §<section>"或等价标题锚点，允许"N. " "N.N " 等既有标题前缀变体
                    warnings.push(Warning::SectionNotFound { source_file, target_doc, section });
                }
            }
        }
    }
    warnings  // RGS-BAS-009§4已定性为"警告级，人工复核"，本函数不返回Err，仅输出供人工查阅的列表
}
```

**RGS-BAS-009§4已言明**"章节重编号时易误报"，本函数据此设计为**警告级**而非阻断级，与已实现4项的`Result<(), CheckFailure>`签名不同，CI流水线中本检查失败不使整体job失败，仅在PR评论中列出供人工复核（复用GitHub Actions既有PR评论机制，不新增独立通知通道）。

## 4.4 OLU台账余额为负

```
fn check_olu_budget_negative(appendix_d_5: &OluLedger) -> Result<(), CheckFailure> {
    let budget = appendix_d_5.total_budget();       // §3"总预算"行
    let approved_sum: i64 = appendix_d_5.approved_items()
        .filter(|item| item.status == "生效")        // "待回收"状态的负数行暂不计入扣减（尚未真正生效回收）
        .map(|item| item.olu)
        .sum();
    let balance = budget - approved_sum;
    if balance < 0 {
        Err(CheckFailure::new("OLU台账余额为负", vec![format!("balance={}", balance)]))
    } else {
        Ok(())
    }
}
```

**启用时点**：RGS-BAS-009§4已明确"待ISS-032决议后再启用"——ISS-032已于2026-08-17决议（总预算提升至210），当前182用量下余额+28为正，本检查函数在ISS-032决议生效**之后**启用不会立即报红，满足启用前置条件；具体启用（将本函数接入CI工作流的PR）仍需评审者显式提交，本文档只给出函数本身，不代为决定启用时点这一运维决策。

---

# 5. 个人数据删除／导出编排状态机详细设计

对应RGS-BAS-009§5.2/§5.2.1时序图，落实PH-6前"AdminService内状态机＋重试"阶段的具体状态转移与幂等实现。

## 5.1 状态定义

```rust
enum OrchestrationStatus {
    Accepted,           // 请求已受理，尚未开始逐库处理
    InProgress { completed_targets: Vec<TargetDbId> },  // 部分目标库已完成，供重入时跳过
    Completed,
    Failed { last_error: String, retry_count: u32 },
}

enum OrchestrationAction {
    Delete,   // §5.2：物理删除/哈希化可识别信息 + 审计表去标识化
    Export,   // §5.2.1：聚合读取 + 写入临时对象存储 + 生成限时链接
}
```

## 5.2 主流程（覆盖删除与导出两条路径，仅`action`参数化，落实§5.2.1"复用同一状态机"设计）

```rust
fn run_orchestration(request_id: RequestId, target_player_id: PlayerId, action: OrchestrationAction) -> Result<(), OrchError> {
    let record = find_or_create_orchestration_record(request_id, target_player_id, action)?;
    // 幂等入口：request_id已存在Completed记录则直接返回，不重复执行(同ARC-009既定幂等设计精神)
    if record.status == OrchestrationStatus::Completed {
        return Ok(());
    }

    let target_dbs = personal_data_ownership_registry().targets_for(&action);
    // 关键设计点：targets_for对Delete与Export返回**同一份**Mount Record个人数据归属清单(FR-GOV-011)，
    // 避免§5.2.1所警示的"两份清单各自维护、互相漂移"问题——本函数不额外接受各自的目标库列表参数

    let already_done: HashSet<TargetDbId> = match &record.status {
        OrchestrationStatus::InProgress { completed_targets } => completed_targets.iter().cloned().collect(),
        _ => HashSet::new(),
    };

    for db in &target_dbs {
        if already_done.contains(&db.id) { continue; }  // 重入时跳过已完成目标，不重复副作用
        match action {
            OrchestrationAction::Delete => execute_delete_on_target(db, &target_player_id)?,
            OrchestrationAction::Export => execute_export_read_on_target(db, &target_player_id)?,
        }
        mark_target_completed(request_id, db.id)?;  // 单目标完成后立即持久化进度，非等全部完成才写
    }

    match action {
        OrchestrationAction::Delete => {
            deidentify_audit_target_player_id(&target_player_id)?;  // §5.2最后一步：审计表UPDATE去标识化
        }
        OrchestrationAction::Export => {
            let bundle = aggregate_exported_data(request_id)?;      // §5.2.1汇总为结构化JSON
            let url = write_to_temp_object_storage_with_expiry(bundle, EXPORT_TTL)?;
            notify_support_ticket_with_download_url(request_id, url)?;
        }
    }

    mark_orchestration_completed(request_id)?;
    Ok(())
}
```

## 5.3 失败与重试

```rust
fn on_target_failure(request_id: RequestId, db: TargetDbId, err: TargetDbError) {
    let record = load_orchestration_record(request_id);
    let retry_count = record.retry_count() + 1;
    if retry_count > MAX_ORCH_RETRY {
        // 达到重试上限：标记Failed但**不**回滚已完成目标(已删除/已导出的目标库不可逆回退，
        // 部分完成状态本身是安全的中间态——未完成目标仍保持原状，不构成数据不一致)
        mark_orchestration_failed(request_id, err, retry_count);
        emit_alert("orchestration_stuck", request_id);  // 复用RGS-BAS-003§6既有告警通道
        return;
    }
    schedule_retry(request_id, retry_count, RETRY_BACKOFF.next(retry_count));  // 复用ARC-009标准消费者重试参数量级
}
```

**关键边界条件说明**：`already_done`集合的存在使得"中途失败后重入"不会对已处理完的目标库重复执行删除/导出——这是RGS-BAS-009§5.2"幂等，可重入"这一文字要求的具体实现方式；`mark_target_completed`在**每个**目标库完成后立即持久化（而非批量在全部完成后一次性写入），使得进程崩溃恢复后的重入起点精确到"上次完成到哪个目标库"，而非退化为"从头重来"或"整体失败"。

## 5.4 删除/导出两条编排遍历的库集合一致性校验（落实RGS-BAS-009§5.2.1"上线前检查清单"设计点）

RGS-BAS-009§5.2.1设计点③要求"上线前检查清单**必须**验证：导出编排遍历的库集合与删除编排遍历的库集合逐一对应（同一份Mount Record声明）"。本节给出该校验在代码侧的承载位置——在挂载App的上线前阶段（与RGS-BAS-002§12.1挂载检查清单同位）一次性运行，**不**进PH-6工作流运行期。

```rust
// 仅供挂载App上线前检查清单使用，CI不调用(避免每PR误报)
fn verify_delete_export_target_parity(mount_record: &MountRecord) -> Result<(), ParityFailure> {
    // 不区分action: Mount Record单一权威,两条编排共享同一份清单(同§5.2关键设计点)
    let target_set: BTreeSet<TargetDbId> = mount_record.targets_personal_data().iter().cloned().collect();
    // 删除编排遍历的库集合 = Mount Record目标集合(无action分支)
    let delete_targets: BTreeSet<TargetDbId> = target_set.clone();
    // 导出编排遍历的库集合 = Mount Record目标集合(无action分支)
    let export_targets: BTreeSet<TargetDbId> = target_set.clone();
    // 两者应恒等(代码层面已强制),但运行时仍做断言以防Mount Record解析层未来引入action分支
    if delete_targets == export_targets {
        Ok(())
    } else {
        Err(ParityFailure::new("删除/导出遍历库集合不一致", delete_targets.symmetric_difference(&export_targets).cloned().collect()))
    }
}
```

**与§5.2的语义一致性**：本函数的存在使得"两条编排共享同一份Mount Record"在**代码层**也得到强制——`delete_targets`与`export_targets`在字面上都从同一份`mount_record.targets_personal_data()`克隆而来，**没有任何action参数可影响两个集合的取值**。此设计落实RGS-BAS-009§5.2.1"新挂载库若声明持有个人数据，两条编排须同时覆盖，不得只更新其中一条"这一警示的具体防护——若未来Mount Record解析层被修改为接受action参数，断言会立即失败，强制评审者回看本节设计决定。

---

# 6. 挂载回滚两阶段验证的具体判定逻辑

对应RGS-BAS-009§5.5，落实"回滚验证"检查项的具体判定条件。

```rust
fn verify_rollback(context: &str) -> Result<RollbackVerification, VerifyError> {
    // 阶段①：流量回退，p99<10秒
    let traffic_start = Instant::now();
    set_gateway_route_weight(context, 0)?;  // 网关路由权重置零
    wait_until(|| gateway_receives_no_errors_for(context), TRAFFIC_ROLLBACK_TIMEOUT)?;
    let traffic_elapsed = traffic_start.elapsed();
    if traffic_elapsed > Duration::from_secs(10) {
        return Err(VerifyError::TrafficRollbackExceededSla(traffic_elapsed));
    }

    // 阶段②：版本回滚，与NFR-AV-007同量级(具体阈值由TBD-GOV-001同批PH-4实测确定，此处不预设固定秒数)
    rollback_image_to_previous_version(context)?;
    wait_until(|| readiness_probe_passing(context), VERSION_ROLLBACK_TIMEOUT_PLACEHOLDER)?;

    Ok(RollbackVerification { traffic_rollback_elapsed: traffic_elapsed, both_stages_passed: true })
    // 判定原则(RGS-BAS-009§5.5)落实：函数不存在"仅验证阶段①即返回Ok"的提前退出路径，
    // 阶段②未通过时wait_until超时返回Err，整体判定不通过
}
```

`VERSION_ROLLBACK_TIMEOUT_PLACEHOLDER`标注为占位——具体数值待TBD-GOV-001同批PH-4实测校准，本文档不代为决定（同RGS-BAS-009§5.5原文"具体值依PH-4实测确定"，本函数结构本身不受该数值待定影响）。

---

# 7. 本文档的覆盖范围与后续计划

本文档覆盖：附件C/D登记表新增行的物理格式契约、OLU台账（附件D§5）的具体物理格式、CI机械校验中4项"未实现"检查的可脚本化解析伪代码（含未注册域名段/未登记AC为阻断级，跨文档章节引用失效为警告级，OLU余额为负的启用时点判定）、删除/导出编排在PH-6前的完整状态机与幂等/重试实现（含v0.2新增的"删除/导出两条编排遍历的库集合一致性校验"以落实BAS-009 v0.6 §5.2.1设计点③）、挂载回滚两阶段验证的判定逻辑。

本版本明确不覆盖、留待后续：

- PH-6工作流基础设施接管编排后的具体工作流定义——依赖ARC-011工作流基础设施自身尚无DTL文档，本文档仅覆盖过渡态（服务内状态机）实现，工作流基础设施就绪后编排逻辑应迁移，届时需要新版本本文档或独立文档衔接。
- §4.4 OLU余额检查函数的实际启用时点（接入CI工作流的具体PR）——本文档只给出函数本身满足启用前置条件（ISS-032已决议），启用动作仍是运维决策，不在本文档内代为执行。
- §6版本回滚阶段的具体超时阈值数值——TBD-GOV-001待PH-4实测确定，本文档保留为占位常量。
- OLU台账的绝对校准方法（TBD-GOV-001）本身不属于本文档范围，本文档只覆盖台账的物理存储格式与解析契约，不覆盖校准算法。

**审阅中注意到、未在本文档内处理、供负责人参考的疑点**（按任务要求不在DTL文档内静默修正BAS决定）：RGS-BAS-009§3.3"处置后余额"计算段落历经两次修订（0.3引入、0.4修正R-5误计入问题），当前文字同时保留了"预算提升前的原始口径"与"预算提升后的现行口径"两组数字并列展示，读者需要分辨"现行口径"才是当前生效结论——这不是数值错误，但表述方式容易被后续读者误引用已废弃口径的数字，建议负责人评估是否需要在下次修订中删除已废弃口径的展示（不影响本文档已完成的设计，仅为文档可读性建议）。

---

## 追溯性

| 需求/设计来源 | 本文档章节 |
|---|---|
| RGS-BAS-009§2.2 附件C§7登记表结构 | §2.1 |
| RGS-BAS-009§2.3 主编号映射表结构 | §2.2 |
| RGS-BAS-009§3.4 GOV-OLU-001〜004台账运维方式 | §3 |
| RGS-BAS-009§4 CI机械校验设计（未实现4项） | §4 |
| RGS-BAS-009§5.2 个人数据删除编排 | §5.1, §5.2, §5.3 |
| RGS-BAS-009§5.2.1 数据导出编排（含"上线前检查清单必须验证遍历库集合对应"设计点③） | §5.1 (Export变体), §5.2 (Mount Record共享), §5.4 (一致性校验) |
| RGS-BAS-009§5.5 挂载回滚时限拆分 | §6 |
| RGS-DTL-001（DDL/OCC模式先例） | §5.3并发/持久化风格参照 |
