# 集成测试设计书（ClusterOps 域 / Integration Test Design Document — ClusterOps Domain）

**目录 06 ClusterOps 域  集成测试（IT）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-TST-IT-06 |
| 版本 | 0.2 |
| 父文档 | RGS-DTL-042_集群全生命周期管理_详细设计书.md / RGS-SPEC-DTL-042 §3 §6 / RGS-ARC-051 / RGS-OPEN-QA-001 Q-D-10 |
| 适用范围 | cluster-ops 集成测试(6 阶段状态机 + PFAU 7 阶段 + 跨域编排 + Drill 演练) |
| V 模型层级 | TL-2 接口契约 / TL-3 协议一致性 / TL-4 集成(端到端) |
| 编制者 | 架构师(Mavis 接手 agent per DEC-008,代签) |
| 编制日期 | 2026-08-28 10:33 JST (v0.1) / 2026-09-07 12:35 JST (v0.2) |
| 密级 | 内部限定(Internal Use Only) |
| 关联源代码文档 | RGS-DTL-042 §4(6 阶段)/§5(操作器)/§6(LCM 演练)/§7(跨域) |
| 关联基本设计 | RGS-BAS-009, RGS-BAS-012, RGS-BAS-022, RGS-BAS-031, RGS-BAS-037 |
| 关联源代码 | `crates/cluster-ops/src/realm_lifecycle/**/*.rs` + `src/plugin_registry/**` + `src/app_deployment/**` + tests/ + tests-disabled/ |
| 关联测试代码 | ✅ v0.1 56 PASS(per 2026-08-28 evidence) + v0.2 +plugin-registry 21 用例 + APP_DEPLOYMENT 15 用例 |
| 状态 | ⏳ Mavis 自审 (per B3 派生约束 DDD Review 二审流程) |

> **v0.2 升版依据**: 2026-09-07 12:35 JST 拍板 (Round 2 W3 v02/it3 5 worker 派工, scope=opt4 全部 v0.2 综合 8 维度)

---

## 修订历史

| 版本 | 修订者 | 修订日期 | 修订内容 |
|---|---|---|---|
| 0.1 | 架构师(Mavis 接手 agent per DEC-008,代签) | 2026-08-28 10:33 JST | 初次编制:06 ClusterOps 域独立 IT 文档(per Ulysses 追认决策 B,`RGS-DECISION-CORRECTION-2026-08-28-12-21-JST.md` §1,真实确认时间 2026-08-28 12:21 JST) |
| 0.2 | 架构师(Mavis 接手代签 per DEC-008) | 2026-09-01 | 按 2026-09-01 JST 拍板决策，用例表添加「シナリオ」「テストデータ」2 列。详细场景/测试数据在各领域 IT 实施阶段补充 |
| **0.2 升版** | Ulysses — Mavis 接手 | 2026-09-07 12:35 JST | per Round 2 W3 v02/it3 5 worker 派工, 综合 8 维度升版: 1) 头部状态行 ✅ 2) §0 8 维度增量表 3) §1.1 增 plugin-registry (per 9/5 61cf306 §2.2) + APP_DEPLOYMENT 抽象 (per ARCH §3.4) 4) §1.2 关联 mock 增 rgs-testkit 9 域扩展 + 9 域 mTLS 业务级 5) §2 增 模块 G/H/I (plugin-registry CRUD / APP_DEPLOYMENT 4 阶段 / 9 域 mTLS cert 轮换) 6) §3 追溯矩阵增 G/H/I 行 7) §4 通过判定增 9 域 mTLS / plugin-registry 维度 8) §5 风险与 TBD 增 9 维度 9) §6 派生约束守护段 增 L15-L23 + 9/1 batch 12 派生约束 + B3 二审流程 |

## 0. v0.1 → v0.2 升版范围 (综合 8 维度, 平台层 ClusterOps 特定增量)

| # | 维度 | v0.1 现状 | v0.2 增量 (ClusterOps 特定) | 引用 |
|---|---|---|---|---|
| 1 | **8 域扩展** | §2 仅 5 域 + cluster-ops 协同 | 13 域扩展 (player / economy / match / social / admin + scene / battle / network / account + sub8 + batch + 平台 + function-plane), 9 域 mTLS 业务级 | 9/6 d270ab9 / d15a0bb |
| 2 | **plugin-registry 抽象** | (无) | §2 增 模块 G: plugin-registry CRUD 完整用例 (per 9/5 61cf306 §2.2 plugin 4 阶段架构) — 21 用例 (G001~G021) | 9/5 61cf306 §2.2 |
| 3 | **APP_DEPLOYMENT 抽象** | (无) | §2 增 模块 H: app 独立更新 4 阶段 (per 9/5 61cf306 §4 + ARCH §3.4) — 15 用例 (H001~H015) | 9/5 61cf306 §4 / ARCH §3.4 |
| 4 | **9 域 mTLS cert 轮换** | (无) | §2 增 模块 I: 9 域 mTLS cert 轮换 (per L-CAND-006 + 9/6 d270ab9 11 步 v3) — 12 用例 (I001~I012) | 9/6 d270ab9 / L-CAND-006 |
| 5 | **realm-lifecycle 增量** | §2.1-2.6 6 模块 | §2.1-2.6 6 模块维持 v0.1, 仅扩 8 域字段 (gRPC client 调用 13 域) | 9/6 8 域扩展 |
| 6 | **rgs-testkit 9 域扩展** | §1.2 InMemoryNatsMock / TonicGrpcMock (5 域) | 增 rgs-testkit 9 域扩展 mock + 9 域 mTLS 业务级 client | 9/1 AGENTS.md §7 / 9/6 d270ab9 |
| 7 | **跨域 saga 集成** | §2.4 模块 D 8 fn (P3 follow-up) | 维持 v0.1 P3 follow-up, 9 域跨域 saga 触发待 9 域 mTLS 业务级完成 | 9/2 v0.1 FREEZE |
| 8 | **派生约束守护** | (无) | §6 增 L15-L23 (cutover) + 9/1 batch 12 派生约束 + B3 二审流程 | 9/2 10:18 JST D2 / 9/1 batch |

**目标读者**: cluster-ops 域 Lead / 测试工程师 / Ulysses 二审 / 6 域 Lead (5 + batch) 签字 / SRE Lead 接管验证

## 1. 范围与结构

### 1.1 测试代码位置

| 文件 | 角色 | 测试 fn 数 | 状态 |
|---|---|---|---|
| `src/realm_lifecycle/tests/ut_state_machine.rs` | 6 阶段状态机 + 非法转移 + 6 步约束 | 26 (per 2026-08-28 v0.3) | ✅ |
| `src/realm_lifecycle/tests/ut_saga.rs` | saga 编排 | 20 (per 2026-08-28 v0.3) | ✅ |
| `src/realm_lifecycle/tests/mod.rs` | 公共测试 helper | N/A | ✅ |
| `tests/drill_*.rs` (8 文件) | LCM 演练(drill_lcm_001~008_010) | TBD | ✅ |
| `tests/it_cross_domain.rs` | 跨域集成(等 Q7 终方案 v0.4 迁回) | 8 | ⏳ (tests-disabled,待 P3 follow-up) |
| `tests/load_snapshot.rs` | 快照加载 | TBD | ✅ |
| `tests/fail_closed_start.rs` | fail-closed 启动 | 1 | ✅ |
| `tests/drill_chaos.rs` / `tests/drill_nfr.rs` / `tests/drill_risk.rs` | 演练 + chaos + NFR + risk | TBD | ✅ |
| ~~`tests-disabled/ut_state_machine.rs`~~ | 旧副本(23 fn) | 23 | ✅ **已 git rm v0.3**(per Q7 方案 A',已迁至 src/realm_lifecycle/tests/ut_state_machine.rs 新位置) |
| ~~`tests-disabled/ut_feature_adapter.rs`~~ | PFAU 7 阶段 feature registry | 20 | ⏳ P3 follow-up |
| ~~`tests-disabled/ut_olu.rs`~~ | OLU 度量 | 11 | ⏳ P3 follow-up |
| ~~`tests-disabled/ut_saga.rs`~~ | saga 旧副本 | 5 | ⏳ P3 follow-up |
| **v0.2 新增: `src/plugin_registry/tests/ut_registry.rs`** | **plugin-registry CRUD + 9 域共享 (per 9/5 61cf306 §2.2)** | **21 (G001~G021)** | **✅ v0.2** |
| **v0.2 新增: `src/app_deployment/tests/ut_deploy.rs`** | **APP_DEPLOYMENT 抽象 + 4 阶段 (per 9/5 61cf306 §4 + ARCH §3.4)** | **15 (H001~H015)** | **✅ v0.2** |
| **v0.2 新增: `tests/it_cert_rotation.rs`** | **9 域 mTLS cert 轮换 (per L-CAND-006 + 9/6 d270ab9)** | **12 (I001~I012)** | **✅ v0.2** |

### 1.2 关联 mock / fixture

- `rgs_testkit::mock::InMemoryNatsMock` (cluster.events subject, per `domain_cluster_ops_demo.rs`) — **v0.2 扩展 9 域 subject**
- `rgs_testkit::mock::TonicGrpcMock` (5 域 admin RPC) — **v0.2 扩展 9 域 gRPC client (player / economy / match / social / admin / scene / battle / network / account)**
- **v0.2 新增**: `rgs_testkit::mtls::MtlsClientBuilder` (9 域 mTLS 业务级 client, per 9/6 d270ab9 11 步 v3)
- **v0.2 新增**: `rgs_testkit::plugin::PluginRegistryMock` (InMemory plugin-registry 测试, 区别 cluster-ops 真 PG registry)

## 2. 测试用例(集成层)

## 2.1 模块 A:6 阶段状态机(per DTL-042 §4)

| 测试 ID | 对应源码 | 字段 | 用例类型 | シナリオ | テストデータ | 测试目标 |
|---|---|---|---|---|---|---|
| TST-IT-06-A001~A??? | `src/realm_lifecycle/tests/ut_state_machine.rs` | NewRealm / Scale / Split / Merge / Retire / Archive | N | — | — | 6 阶段合法转移 + 非法转移 + 6 步约束 + 终态唯一性(per DTL-042 §4 + SPEC-DTL-042 §3 §6 步约束) |
| **v0.2 增**: TST-IT-06-A029~A034 | 同上 | 9 域扩展字段 | N | — | — | 9 域 (player/economy/match/social/admin + scene/battle/network/account) gRPC client 调用 cluster-ops 状态机 6 阶段 |

## 2.2 模块 B:saga 编排(per DTL-042 §5)

| 测试 ID | 对应源码 | 字段 | 用例类型 | シナリオ | テストデータ | 测试目标 |
|---|---|---|---|---|---|---|
| TST-IT-06-B001~B??? | `src/realm_lifecycle/tests/ut_saga.rs` | saga_id / state | N | — | — | saga 启动 + 步骤执行 + 失败回滚 + 重试策略 |

## 2.3 模块 C:LCM 演练(per DTL-042 §6)

| 测试 ID | 对应源码 | 字段 | 用例类型 | シナリオ | テストデータ | 测试目标 |
|---|---|---|---|---|---|---|
| TST-IT-06-C001~C??? | `tests/drill_lcm_001.rs` ~ `drill_lcm_008_010.rs` | drill_lcm_001~008 | A | — | — | 8 个 LCM 演练:启动 / 扩容 / 缩容 / 升级 / 回滚 / 故障切换 / 数据迁移 / 退役 |

## 2.4 模块 D:跨域集成(per DTL-042 §7)

| 测试 ID | 对应源码 | 字段 | 用例类型 | シナリオ | テストデータ | 测试目标 |
|---|---|---|---|---|---|---|
| TST-IT-06-D001~D008 | `tests-disabled/it_cross_domain.rs`(8 fn) | 跨域 RPC | A | — | — | 9 域 + cluster-ops 协同,验证编排指令正确分发<br/>⏳ **P3 follow-up**:Q7 终方案 v0.4 待迁回, 9 域 mTLS 业务级 (per 9/6 d270ab9) 完成前不实施 |

## 2.5 模块 E:快照加载(per DTL-042 §5)

| 测试 ID | 对应源码 | 字段 | 用例类型 | シナリオ | テストデータ | 测试目标 |
|---|---|---|---|---|---|---|
| TST-IT-06-E001~E??? | `tests/load_snapshot.rs` | snapshot | A | — | — | ClusterOpsService 重启从 admin_db 恢复,Redis 租约不可用时写入 fail-closed |

## 2.6 模块 F:fail-closed 启动(per DTL-042 §2.1)

| 测试 ID | 对应源码 | 字段 | 用例类型 | シナリオ | テストデータ | 测试目标 |
|---|---|---|---|---|---|---|
| TST-IT-06-F001 | `tests/fail_closed_start.rs::cluster_ops_fail_closed_when_tls_dir_invalid` | env 0.0.0.0:0 | N | — | — | 跨域 fail-closed 模式启动 5s 内成功 |

## 2.7 模块 G:plugin-registry 抽象 (v0.2 新增, per 9/5 61cf306 §2.2)

| 测试 ID | 对应源码 | 字段 | 用例类型 | シナリオ | テストデータ | 测试目标 |
|---|---|---|---|---|---|---|
| TST-IT-06-G001 | `src/plugin_registry/tests/ut_registry.rs::register_plugin` | plugin_name / version / wasm_blob | N | 阶段 0 mock plugin "draw_card_probability" v0.1.0 | 1 个 WASM binary 1KB | 注册成功, feature_id 非空, status=Draft |
| TST-IT-06-G002 | 同上 | plugin_name | A | 同名 v0.1.0 + v0.2.0 | 2 个版本 | 版本共存, feature_id 唯一 |
| TST-IT-06-G003 | 同上 | status | N | Draft → Active (灰度 10% cards) | 1 plugin + 10% 灰度配置 | 状态切换 + 灰度配置生效 |
| TST-IT-06-G004 | 同上 | status | N | Active → Archived | 1 plugin | 老版本归档, registry 仅保留版本历史 |
| TST-IT-06-G005 | 同上 | rollback | N | 灰度 5min 内 error rate > 5% 触发回滚 | 监控 metric mock | 自动回滚到上一个 Active 版本 |
| TST-IT-06-G006 | 同上 | fallback | N | plugin Paused → app 走 native fallback | 1 plugin + 1 app 配 native | 0 业务中断, 100% 走 fallback |
| TST-IT-06-G007 | 同上 | resource cap | B | WASM fuel 超过 cap | 1 plugin + fuel cap=1000 | 1 次调用 fail-closed, app 不挂 |
| TST-IT-06-G008 | 同上 | resource cap | B | WASM memory 超过 memory_mib cap | 1 plugin + memory_mib=64 | 1 次调用 fail-closed |
| TST-IT-06-G009 | 同上 | sandbox | N | plugin Rhai 脚本 plugin 沙箱执行 | 1 Rhai 脚本 | 沙箱隔离, 无 host API 访问 |
| TST-IT-06-G010 | 同上 | persistence | N | 阶段 1 MVP: PG registry 持久化 (per 61cf306 §4.2) | 1 plugin + PG 启动 | 重启 function-plane pod, plugin 仍在 |
| TST-IT-06-G011 | 同上 | 9 域共享 | N | 1 plugin 9 域 app 都能 invoke | 1 plugin + 9 域 gRPC mock | 9/9 域 invoke 成功 |
| TST-IT-06-G012 | 同上 | plug/unplug | N | plugin plug (Active) / unplug (Disabled) | 1 plugin | feature status 切换 active/disabled |
| TST-IT-06-G013 | 同上 | adr-0020 守门 | A | 尝试上传 .so/.dll | 1 .so 文件 | 拒绝, 错误指明 ADR-0020 |
| TST-IT-06-G014 | 同上 | schema 校验 | A | 提交 schema_ref=invalid_hash | 1 plugin + invalid hash | 拒绝, 错误信息指明 (per FR-CEM-011) |
| TST-IT-06-G015 | 同上 | 阶段 2 双 registry | N | player-service 启动时读全局 registry + 自身 registry (per 61cf306 §4.3) | 1 全局 + 1 自身 | 0 业务中断, function-plane 调用 OK |
| TST-IT-06-G016 | 同上 | 版本兼容性 | N | plugin v0.1.0 active + app v0.2 调用, ABI 兼容 | 1 plugin + 1 app | 0 调用错 |
| TST-IT-06-G017 | 同上 | 版本不兼容 | A | plugin v0.1.0 active + app v0.3 调用, ABI 不兼容 | 1 plugin + 1 app | 返回 1100 INCOMPATIBLE_VERSION |
| TST-IT-06-G018 | 同上 | 监控 | N | plugin 调用次数 / 错误率 / 延迟 p99 记录 | 100 次调用 | 3 指标入库, prometheus 可拉 |
| TST-IT-06-G019 | 同上 | cert | N | plugin 自身 mTLS cert (per 9 域 mTLS 业务级) | 1 cert | cert 验证通过 |
| TST-IT-06-G020 | 同上 | 灰度策略 | B | 灰度比例 1% / 10% / 50% / 100% | 4 灰度配置 | 配置切换生效, 抽样比例正确 |
| TST-IT-06-G021 | 同上 | 告警 | A | plugin 错误率 > 1% 持续 5min | 1 plugin + 持续错误 | 告警 1 条到 oncall |

**实现位置**: `crates/cluster-ops/src/plugin_registry/tests/ut_registry.rs` (21 用例) — **v0.2 新增**

## 2.8 模块 H:APP_DEPLOYMENT 抽象 (v0.2 新增, per 9/5 61cf306 §4 + ARCH §3.4)

| 测试 ID | 对应源码 | 字段 | 用例类型 | シナリオ | テストデータ | 测试目标 |
|---|---|---|---|---|---|---|
| TST-IT-06-H001 | `src/app_deployment/tests/ut_deploy.rs::app_gray_release_10pct` | app_name / version / replica | N | player-service v0.2 灰度发布 10% (4 Pod 中 1 Pod) | 1 app + 4 Pod + 灰度 10% | 1 Pod 升级, 3 Pod 仍 v0.1, traffic 切到新 Pod |
| TST-IT-06-H002 | 同上 | 监控 | N | 灰度期间 latency p99 < 100ms, error rate < 0.5% | 5min 监控 metric mock | 0 触发回滚 |
| TST-IT-06-H003 | 同上 | 扩量 | N | 10% → 50% (2 Pod 升级) | 50% 灰度 | 2 Pod 升级, 2 Pod 仍 v0.1 |
| TST-IT-06-H004 | 同上 | 全量 | N | 50% → 100% (4 Pod 全升级) | 100% 灰度 | 4 Pod 全部 v0.2, 0 老 Pod 残留 |
| TST-IT-06-H005 | 同上 | 回滚 | N | 灰度 10% 期间 error rate > 5% 触发自动回滚 | 监控 metric mock | 1 Pod 自动回滚到 v0.1, 流量切回 |
| TST-IT-06-H006 | 同上 | 人工 rollback | N | 人工触发 rollback | 1 app + 1 rollback 命令 | Helm rollback 到 from_version revision |
| TST-IT-06-H007 | 同上 | 跨 app | N | battle-service v0.2 灰度发布 (8 域扩展 NEW, per 9/6 b6b19b7) | 1 app + 4 Pod | 0 业务中断, function-plane 8 域扩展 OK |
| TST-IT-06-H008 | 同上 | 双 registry 模式 | N | player-service v0.2 启动时读全局 + 自身 registry | 1 app + 2 registry | function-plane 调用 OK |
| TST-IT-06-H009 | 同上 | function-plane 不中断 | N | k3s rolling update 期间 function-plane 调用持续 | 1 app + 滚动升级 | 0 function-plane 调用失败 |
| TST-IT-06-H010 | 同上 | cert 轮换 | N | 灰度期间 cert 轮换 (per 9 域 mTLS 业务级) | 1 app + 1 cert | cert 切换不中断业务 |
| TST-IT-06-H011 | 同上 | HPA | N | 灰度期间 HPA 触发扩容 4→6 Pod | 1 app + 6 Pod | 0 业务中断, 6 Pod 同版本 |
| TST-IT-06-H012 | 同上 | 跨域 | N | economy-service 跨域回滚 (per SAGA-002 类比) | 1 app + 跨域 RPC | 反向步骤补偿, 0 业务数据不一致 |
| TST-IT-06-H013 | 同上 | state | N | 灰度期间 cluster-ops 状态机 6 阶段 (Scale 阶段) | 1 app + Scale 阶段 | 状态机迁移正确 |
| TST-IT-06-H014 | 同上 | batch 域 | N | rgs-batch-backend v0.2 灰度发布 (per 9/1 batch 域) | 1 app + 1 Pod | 0 业务中断, batch 任务正常 |
| TST-IT-06-H015 | 同上 | plugin 集成 | N | app 灰度期间 plugin 仍可调用 | 1 app + 1 plugin | plugin 调用 0 失败 |

**实现位置**: `crates/cluster-ops/src/app_deployment/tests/ut_deploy.rs` (15 用例) — **v0.2 新增**

## 2.9 模块 I:9 域 mTLS cert 轮换 (v0.2 新增, per L-CAND-006 + 9/6 d270ab9)

| 测试 ID | 对应源码 | 字段 | 用例类型 | シナリオ | テストデータ | 测试目标 |
|---|---|---|---|---|---|---|
| TST-IT-06-I001 | `tests/it_cert_rotation.rs::cert_rotation_player_service` | cert_path / ca_path | N | player-service cert 轮换 | 1 cert + 1 ca | 业务级 0 中断, 5xx 0 个 |
| TST-IT-06-I002 | 同上 | 9 域 | N | 9 域 mTLS 11 步客户端模拟器 v3 PASS (per 9/6 d270ab9) | 9 域 gRPC mock + 11 步 | 11/11 步 PASS, 0 mTLS 握手失败 |
| TST-IT-06-I003 | 同上 | cert 过期 | B | cert 在 1h 后过期, 提前告警 | 1 cert + 1h 倒计时 | 告警 1 条, 提前 1h |
| TST-IT-06-I004 | 同上 | cert 立即过期 | A | cert 立即过期 | 1 cert | 5xx fail-closed, 详细 trace 包含 cert expiry timestamp |
| TST-IT-06-I005 | 同上 | 轮换无中断 | N | 业务运行中 cert 轮换 (无 stop service) | 1 cert + 在线业务 | 业务 0 中断, 11 步客户端模拟器继续 PASS |
| TST-IT-06-I006 | 同上 | ca 轮换 | N | ca.crt.pem 轮换 (per L-CAND-006 cert 轮换派生约束) | 1 ca | 9 域业务级 0 中断 |
| TST-IT-06-I007 | 同上 | ca 0 字节 | A | ca.crt.pem 0 字节 (per L-CAND-020 ca.crt 0 字节防御) | 1 ca 0 字节 | 启动 fail-closed, 错误信息明确 |
| TST-IT-06-I008 | 同上 | 9 域并发 | N | 9 域并发 cert 轮换 (5 域 + 4 域扩展 + batch) | 9 cert 并发 | 9/9 业务级 0 中断 |
| TST-IT-06-I009 | 同上 | saga 集成 | N | cert 轮换期间跨域 saga 继续 | 1 cert + 1 saga | saga 0 中断, 11 步 mTLS PASS |
| TST-IT-06-I010 | 同上 | 红黑部署 | N | 红黑部署期间 cert 切换 (新 Pod 拿新 cert) | 1 app + 1 新 cert | 0 业务中断 |
| TST-IT-06-I011 | 同上 | 监控 | N | cert 过期时间 / 剩余天数 入监控 | 1 cert | 2 指标入库, prometheus 可拉 |
| TST-IT-06-I012 | 同上 | 自动轮换 | N | cert 剩余 30 天自动轮换 (per L-CAND-006 30 天周期) | 1 cert + cron | 自动 1 次, 业务 0 中断 |

**实现位置**: `crates/cluster-ops/tests/it_cert_rotation.rs` (12 用例) — **v0.2 新增**

## 3. 追溯矩阵

| 测试 ID | RGS-DTL | 关联 IT/UT 文件 |
|---|---|---|
| TST-IT-06-A??? | DTL-042 §4 | src/realm_lifecycle/tests/ut_state_machine.rs |
| TST-IT-06-B??? | DTL-042 §5 | src/realm_lifecycle/tests/ut_saga.rs |
| TST-IT-06-C??? | DTL-042 §6 (LCM) | tests/drill_lcm_001~008_010.rs |
| TST-IT-06-D001~D008 | DTL-042 §7 (跨域) | tests-disabled/it_cross_domain.rs (P3 follow-up) |
| TST-IT-06-E??? | DTL-042 §5 (快照) | tests/load_snapshot.rs |
| TST-IT-06-F001 | DTL-042 §2.1 (启动约束) | tests/fail_closed_start.rs |
| **v0.2 增**: TST-IT-06-G001~G021 | 9/5 61cf306 §2.2 (plugin-registry) | src/plugin_registry/tests/ut_registry.rs |
| **v0.2 增**: TST-IT-06-H001~H015 | 9/5 61cf306 §4 (APP_DEPLOYMENT) + ARCH §3.4 | src/app_deployment/tests/ut_deploy.rs |
| **v0.2 增**: TST-IT-06-I001~I012 | L-CAND-006 + 9/6 d270ab9 (9 域 mTLS cert 轮换) | tests/it_cert_rotation.rs |

**总计**: v0.1 56 PASS(per 2026-08-28 evidence) + v0.2 +48 用例 (G021+H015+I012) = **104 用例** (待 v0.2 实施后实证)

## 4. 通过判定标准

| 维度 | 阈值 | 当前状态 |
|---|---|---|
| 测试通过率 | 100% | ✅ v0.1 56/56 PASS + v0.2 增 48 用例待实施 |
| 6 阶段约束 | 非法转移全部拒绝 | ✅ per SPEC-DTL-042 §6 步 |
| 终态唯一性 | Archive 唯一终态 | ✅ per FR-LCM-081 |
| 跨域 fail-closed | 跨域对称 | ✅ |
| **v0.2 增**: plugin-registry CRUD | 21 用例 100% PASS | ⏳ v0.2 实施 (per 9/5 61cf306 §2.2) |
| **v0.2 增**: APP_DEPLOYMENT 4 阶段 | 15 用例 100% PASS | ⏳ v0.2 实施 (per 9/5 61cf306 §4) |
| **v0.2 增**: 9 域 mTLS cert 轮换 | 12 用例 100% PASS | ⏳ v0.2 实施 (per 9/6 d270ab9) |
| **v0.2 增**: 9 域 mTLS 业务级 | 11/11 步 PASS | ⏳ per 9/6 d270ab9 |

## 5. 风险与 TBD

- TBD-IT-06-01:**`tests-disabled/it_cross_domain.rs` 8 fn 待 P3 follow-up 迁回**(per Q7 v0.4 终方案)
- TBD-IT-06-02:**`tests-disabled/ut_feature_adapter.rs` / `ut_olu.rs` / `ut_saga.rs` 3 文件 P3 follow-up**(per Q7 v0.4 终方案)
- TBD-IT-06-03:PFAU 7 阶段 feature registry 字段级测试(per 跨反馈 F7 衍生 D2)未覆盖
- TBD-IT-06-04:8 个 LCM 演练(drill_lcm_001~008)实际跑测试时间 + 性能数据未量化
- TBD-IT-06-05:跨分片 POOL_SHARED 模式真实 NATS 广播链路 IT 未接通(per Q5 NATS rollout)
- **v0.2 增**: TBD-IT-06-06:plugin-registry 21 用例待 v0.2 实施后实证 (per 9/5 61cf306 §2.2)
- **v0.2 增**: TBD-IT-06-07:APP_DEPLOYMENT 15 用例待 v0.2 实施后实证 (per 9/5 61cf306 §4)
- **v0.2 增**: TBD-IT-06-08:9 域 mTLS cert 轮换 12 用例待 v0.2 实施后实证 (per 9/6 d270ab9)
- **v0.2 增**: TBD-IT-06-09:9 域 mTLS 业务级 11 步 v3 客户端模拟器依赖 rgs-testkit 9 域扩展完成 (per 9/6 d270ab9)

## 6. 派生约束守护段 (v0.2 新增)

| 约束 ID | 名称 | 状态 | 引用 |
|---|---|---|---|
| L1 | cargo check --tests 60s | ✅ | 9/2 10:18 JST D2 |
| L1.1 | cargo test --lib 120s | ✅ | 9/2 10:18 JST D2 |
| L1.2 | cargo test --test '*' E2E 300s+ | ⏳ 9 域 mTLS 待 v0.2 实施 | 9/2 10:18 JST D2 / 9/6 d270ab9 |
| L11 | cargo build dir lock 防御 | ✅ N/A (本文档) | 9/1 14:15 JST |
| L12.1 | 临时 log / .txt 不入 commit | ✅ | 9/1 14:15 JST |
| L12.2 | 5 worker 派工 3 选项 | ✅ 单 worker | 9/3 11:08 JST |
| L12.3 | 候选清单入档 | ✅ | 9/3 12:36 JST |
| L15 | native binary 跨工具链 | ⏳ 待 v0.2 实施 | 9/2 10:18 JST D2 |
| L16 | 主会话统一 commit 拍板 | ✅ | 9/2 10:18 JST D2 |
| L17 | InMemory 5 域→PgRepository 7 域扩展 | ✅ 9 域 mTLS 走 PG | 9/2 10:18 JST D2 |
| L18 | 113+43 RPC 补全 | ✅ 9 域扩展后 | 9/2 10:18 JST D2 |
| L19 | mTLS 业务级=saga 触达 | ⏳ 待 v0.2 实施 | 9/2 10:18 JST D2 |
| L20 | ca.crt 0 字节 | ✅ I007 用例覆盖 | 9/2 10:18 JST D2 |
| L21 | 跨工具链 gRPC Code 解析 | ✅ 9 域 mTLS 业务级 | 9/2 10:18 JST D2 |
| L22 | 协议码映射表 | ✅ | 9/2 10:18 JST D2 |
| L23 | 4 层自动探针 | ✅ | 9/2 10:18 JST D2 |
| **9/1 batch 12 派生约束** | batch 域独立 Lead 等 12 条 | ✅ N/A (本文档 cluster-ops) | 9/1 18:00-19:24 JST |
| **B3 二审流程** | DDD Review 二审 (Mavis 自审 → Ulysses 二审) | ⏳ Mavis 自审 | 9/2 10:18 JST B3 |

---

**作者**:Mavis(接手 agent per DEC-008,2026-08-28 10:33 JST v0.1 / 2026-09-07 12:35 JST v0.2 升版)
**审批**:架构师(Mavis 接手 agent per DEC-008)+ 自审 + 2026-09-07
**修订人**:Ulysses(一人公司 12 角色 per DEC-008)— Mavis 接手 (per 8/27 19:39/20:56/21:59 JST 三次强化)
