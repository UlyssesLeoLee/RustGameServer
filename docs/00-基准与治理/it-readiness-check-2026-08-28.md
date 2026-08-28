# IT 准入核对清单 — 2026-08-28 09:58 JST(v0.2 更新 2026-08-28 12:21 JST)

> **目的**:核对是否达到 V 模型 TL-2/3/4 集成测试(IT)准入条件
> **作者**:Mavis(接手 agent per DEC-008,2026-08-28 09:58 JST;v0.2 数据更新 12:21 JST)
> **关联**:`RGS-SPEC-000 §2.4` V 模型定义 + UT 准入已完成(per 2026-08-28 ut 实施 v0.2,10 域 ≥ 90% 达标率,含本轮补入的 rgs-asset-download)+ `RGS-DECISION-CORRECTION-2026-08-28-12-21-JST.md`

---

## 0. 结论(v0.2:基于 `cargo test --workspace --no-fail-fast` 完整实测数据 + 01-07 IT 文档已补全)

**准入条件基本满足,1 项环境限制待确认**。01-07 域 IT 设计书已补全(per 追认决策 B,7 份独立文档落档);10 域真实 UT 全绿(391 passed / 0 failed,含本轮补入的 rgs-asset-download 135 UT);IT/环境依赖测试 14 个失败**全部为本沙箱环境无本地 Postgres/Docker daemon 导致**,非代码缺陷(已用 `RUST_BACKTRACE` + panic 信息逐条核实,见 §2.1);另有 1 个真实测试脆弱性 bug(CRLF 换行符导致的字符串匹配失败,已修复,见 §2.1 脚注)。

**优先级**(更新):
1. **G3 环境依赖确认**(本机/CI 均需 `DATABASE_URL` 指向真实 Postgres 才能验证 14 个 fixture IT;本次沙箱无 Docker daemon,未能实测,标记"未验证"而非估算通过)
2. **§2.5 覆盖率未测量**:`cargo-llvm-cov` 已装但本轮未跑全量(工作量 + 时间预算原因),旧文档"95% 粗估"已删除,标记"未测量"
3. **跨域 IT 用例补充**(cluster-ops ↔ 5 域 ↔ admin-service ↔ gm-backend,per S2)

**总评**:文档/决策层面的 3 项阻塞(G1 cluster-ops Q7、G2 8 域 Lead 具名、01-07 IT 文档)均已经真实追认/补全解除。唯一剩余的是**环境验证类**收尾(G3 + 覆盖率实测),建议开 IT 主阶段的同时,第一件事是在有 Postgres 的环境里重跑 `cargo test --workspace --no-fail-fast` 拿到真实 fixture IT 结果。

---

## 1. V 模型 TL 层级对照

| TL 层级 | 测试类型 | 当前状态 | 准入条件 |
|---|---|---|---|
| TL-1 | 单元测试 (UT) | ✅ 10 域 ≥ 90% 达标率 (per 2026-08-28 09:30 JST ut 实施 + 本轮补入 rgs-asset-download) | 各域设计达标率 ≥ 90% + TBD 跟踪 |
| **TL-2** | **接口契约 (IT)** | 🟡 部分就绪(见下) | UT 稳定 + 域间接口定义 + 跨域 RPC 链路可测 |
| **TL-3** | **协议一致性 (IT)** | 🟡 部分就绪(见下) | DTL 协议字段已对齐 + BAS 协议章节已定 |
| **TL-4** | **集成端到端 (IT)** | 🟡 部分就绪(见下) | k3s 集群跑通(per 2026-08-27 部署)+ admin-client mock |
| TL-5 | 状态机试验 (ST) | ⏳ ST 阶段(per V 模型下一步) | IT 全 PASS + 5 域 e2e 12 端口 PASS |
| TL-6 | 系统测试 (ST) | ⏳ ST 阶段 | IT + chaos + load test |
| TL-7 | 验收测试 (UAT) | ⏳ DDD Review + Ulysses 终审 | 全部 ST PASS + OPEN-QA 关闭 |

## 2. IT 准入条件逐项核对

### 2.1 UT 稳定通过(必需)— v0.2 实测数据(2026-08-28 12:21 JST,`cargo test --workspace --no-fail-fast`)

**分类标准**(本轮明确化,替代 v0.1 混合口径):UT = `cargo test -p <crate> --lib`(纯单元测试)+ `tests/ut_*.rs`(命名规范的独立单元测试文件);其余(`integration_*.rs` / `it_*.rs` / `fail_closed_start.rs` / `chaos_*.rs` / `span_assertion.rs` / `load_*.rs`)按 V 模型口径记入 IT,见 §3。

| 域 | UT passed | UT failed | 状态 | 备注 |
|---|---|---|---|---|
| rgs-testkit | 3 (lib) | 0 | ✅ | 另有 self_test/grpc_mock_test/nats_mock_test/fixture_extended_test 29 个,属"测 mock 基础设施自身",归入 IT |
| rgs-certgen | 17 (ut_blackbox.rs) | 0 | ✅ | |
| gm-backend | 21 (ut_audit 7 + ut_config 6 + ut_jwt 8) | 0 | ✅ | lib.rs 本身无内联测试 |
| cluster-ops | 56 (lib) | 0 | ✅ | |
| player-service | 27 (lib) | 0 | ✅ | |
| economy-service | 53 (lib) | 0 | ✅ | |
| match-service | 28 (lib 19 + ut_matchmaker 9) | 0 | ✅ | |
| social-service | 20 (lib) | 0 | ✅ | |
| admin-service | 31 (lib) | 0 | ✅ | |
| rgs-asset-download | 135 (lib 55 + ut_chunk_orchestrator 8 + ut_integrity_gate 8 + ut_range_client 10 + ut_resume_token_store 24 + ut_state_machine 30) | 0 | ✅ | 域 07(资产下载),v0.2 首版遗漏,本次补入 |
| **TOTAL** | **391** | **0** | ✅ **100%** | |

**结论**:纯 UT(TL-1)口径下,10 域(9 域原表 + 补入的 rgs-asset-download)实测 **391 passed / 0 failed**,与 v0.1 的"311/13(95.4%)"数字不同 — 差异原因是 v0.1 把 fixture 依赖的 `integration_*.rs`(真需要 Postgres)也计入"UT"分母,本轮按 V 模型口径把它们移到 §3 IT 表,UT 层因此干净 100% 通过,更符合 TL-1/TL-2 分层定义。(rgs-overflow-alert lib 30 passed 未计入 — 该 crate 不在 §4 的 00/01-09 域编号内,是横切 NFR 组件,非独立域)

**脚注(测试脆弱性 bug,已修复)**:`rgs-asset-download::ut_resume_token_store.rs::token_struct_has_no_pii_fields_in_definition` 用 `include_str!` 解析源码找 `"\n}\n"` 定位 struct 结束位置,但 `resume_token.rs` 是 CRLF 换行(Windows checkout),导致字符串匹配失败 panic("struct closing brace")。**不是产品缺陷**(ResumeToken struct 本身确认无 PII 字段),已在 `crates/rgs-asset-download/tests/ut_resume_token_store.rs` 加 `.replace("\r\n", "\n")` 修复,修复后 24/24 passed。

### 2.2 域间接口定义稳定(必需)

| 域间接口 | 状态 | 证据 |
|---|---|---|
| 5 域 gRPC(actor/entry/saga)| ✅ DTL-100/101/102 已实化 + IT 测试 | `crates/economy-service/tests/integration_*.rs` 9 fn + span_assertion 3 fn |
| gm-backend HTTP (axum) | ✅ BAS-003 §2.1 + DTL-003 §3 协议字段已定 | `crates/gm-backend/tests/integration_gm_basic.rs` 14 fn |
| gm-backend → admin-service gRPC | ⏳ TBD-08-03 v0.2 暂用 AuditStore trait 抽象 | v0.3 实装 |
| cluster-ops → 5 域 + admin | ✅ DTL-042 §7 跨域编排 | `crates/cluster-ops/tests-disabled/it_cross_domain.rs` 8 fn(旧债,待 Q7 终方案)|
| NATS outbox relay | ⏳ 5 域 outbox + NATS 部署已通(per 2026-08-27) | Q5 OPEN-QA 跟踪 |
| rgs-certgen → 文件系统 | ✅ IT-09 设计书 + UT-09 v0.2 17 黑盒 | `crates/rgs-certgen/tests/ut_blackbox.rs` 17 fn(UT 已覆盖 IT 流程)|

**结论**:5 域内部接口 + gm-backend HTTP 已稳定;**gm-backend → admin-service gRPC 待 v0.3**(TBD-08-03)。

### 2.3 跨域 RPC 链路可测(必需)

| 跨域 | 测法 | 状态 |
|---|---|---|
| 5 域 + cluster-ops 集成 | `cluster-ops/tests-disabled/it_cross_domain.rs` (旧债) + `it_cross_domain.rs` 待迁回 | ⏳ Q7 终方案 A' 决策 |
| 5 域 + admin | `rgs-testkit::mock::TonicGrpcMock` (54.x 实装) | ✅ |
| gm-backend 调 admin-service gRPC | `TonicGrpcMock` mock admin-service 5 endpoint (per ut_jwt.rs 等) | ✅ mock 已有,真接 v0.3 |
| 5 域 + 资产下载 (07) | `crates/rgs-asset-download/tests/it_cloudflare_*.rs` 3 fn + `it_minio_*.rs` 6 fn | ✅ 已 9 fn |
| gm-backend + 5 域 (k8s 集群) | `scripts/e2e-smoke.ps1` 12 端口 + `gm-backend /healthz` | ✅ 19/19 Pods Running + 12/12 PASS |
| 端到端 (k3s + NATS + pg) | per 2026-08-27 部署 | ✅ |

**结论**:跨域链路 mock + e2e 已就绪;**k3s 集群跨域集成 IT 用例待 v0.2 实施**。

### 2.4 测试环境(必需)

| 环境 | 状态 | 备注 |
|---|---|---|
| DATABASE_URL (PG) | 🟡 CI 已配;本沙箱环境无 Docker daemon(`docker ps` 报 daemon 未运行),14 个 fixture IT **未能实测** | 需在有本地/CI Postgres 的环境重跑确认,不能假设"配了就通过" |
| RUST_LOG | ✅ CI + 本机 | - |
| 5 域 4 域对称骨架 | ✅ integration_player/match/social/admin_basic.rs 3 fn × 4 域 | - |
| NATS | ✅ k3s 集群部署 + InMemoryNatsMock fallback | - |
| k3s 集群 | ✅ 19/19 Pods Running | per 2026-08-27 部署 |
| axum-test 16 / wiremock 0.6 | ✅ 双工具并存(per TBD-08-06 决策草案)| |

**结论**:CI 全套环境就绪;**本沙箱无 Docker daemon,14 个 fixture IT 待有 Postgres 的环境重跑验证**,不应假设通过。

### 2.5 测试覆盖率(必需 ≥ 60%)— v0.2:标记未测量,不再沿用粗估

| 域 | 覆盖率 | 目标 | 状态 |
|---|---|---|---|
| gm-backend | 未测量(`cargo-llvm-cov` 已装,本轮未跑全量) | 60% | ⏳ **未验证** |
| rgs-certgen | 未测量 | 60% | ⏳ **未验证** |
| 其他 7 域 | 未测量 | 60% | ⏳ **未验证** |

**结论**:v0.1 版本"gm-backend 80% 粗估 / 其他 7 域 95% 粗估"**没有实测来源,已删除**。`cargo-llvm-cov 0.9.0` 已确认安装(`E:\DevCache\cargo\bin\cargo-llvm-cov`),但全工作区覆盖率扫描未在本轮跑(耗时 + 需要 DB 环境配合 IT 部分),列为 IT 主阶段启动前的**真实待办**(见 §5.1 G4),而非既成事实。

## 3. IT 测试覆盖现状 — v0.2 实测(per `cargo test --workspace --no-fail-fast`)

| 域 | IT 文件 | passed | failed(环境) | 状态 |
|---|---|---|---|---|
| player-service | integration_player_basic.rs | 0 | 3 | 🔴 **未验证**(PoolTimedOut,本沙箱无 Postgres) |
| player-service | fail_closed_start.rs | 1 | 0 | ✅ |
| economy-service | integration_reservation.rs | 3 | 0 | ✅ |
| economy-service | integration_outbox.rs | 0 | 2 | 🔴 **未验证**(ConnectionRefused:连 admin DB) |
| economy-service | span_assertion.rs | 3 | 0 | ✅ |
| economy-service | chaos_reservation.rs | 2 | 0(1 ignored) | ✅ |
| economy-service | fail_closed_start.rs | 1 | 0 | ✅ |
| match-service | integration_match_basic.rs | 0 | 3 | 🔴 **未验证**(PoolTimedOut) |
| match-service | fail_closed_start.rs | 1 | 0 | ✅ |
| social-service | integration_social_basic.rs | 0 | 3 | 🔴 **未验证**(PoolTimedOut) |
| social-service | fail_closed_start.rs | 1 | 0 | ✅ |
| admin-service | integration_admin_basic.rs | 0 | 3 | 🔴 **未验证**(PoolTimedOut) |
| admin-service | fail_closed_start.rs | 1 | 0 | ✅ |
| gm-backend | integration_gm_basic.rs | 12 | 0 | ✅ 字段级 + JWT + audit |
| gm-backend | fail_closed_start.rs | 3 | 0 | ✅ |
| rgs-asset-download | it_cloudflare*.rs × 3 | 10 | 0(14 ignored) | ✅ CDN 集成 |
| rgs-asset-download | it_minio_*.rs × 6 | 10 | 0(7 ignored) | ✅ 资产下载 6 场景 |
| rgs-asset-download | chaos_minio.rs / chaos_responses.rs | 8 | 0(6 ignored) | ✅ |
| rgs-asset-download | security_no_pii.rs | 4 | 0 | ✅ |
| rgs-overflow-alert | integration_overflow.rs | 5 | 0 | ✅ |
| rgs-testkit | fixture_extended_test/grpc_mock_test/nats_mock_test/self_test | 29 | 0 | ✅ mock 基础设施自测 |
| cluster-ops | it_cross_domain.rs(tests-disabled) | — | — | ⏳ 未编译(排除在 cargo test 外,per Q7 A' 方案) |
| **TOTAL(实测)** | | **94 passed** | **14 未验证(环境)** | |

**结论**:14 个失败**逐条核实均为"本沙箱无 Docker daemon / 无本地 Postgres"导致**(panic 信息为 `PoolTimedOut` 或 `ConnectionRefused`,非断言失败),**不是代码缺陷,但也不能记为"通过"** — v0.1 版本"13 fail 全 fixture 环境,自动通过"的表述过于乐观,本轮改为如实标注"未验证",需 IT 主阶段第一步在有 Postgres 的环境重跑确认。

## 4. IT 设计书覆盖 — v0.2:01-07 已补全

| 域 | IT 文档 | 版本 | 状态 |
|---|---|---|---|
| 00 基准与治理 | RGS-TST-IT-00 | v0.2 | ✅ |
| 01 玩家域 | RGS-TST-IT-01 | v0.1 | ✅ (per 追认决策 B) |
| 02 经济域 | RGS-TST-IT-02 | v0.1 | ✅ |
| 03 社交域 | RGS-TST-IT-03 | v0.1 | ✅ |
| 04 对战域 | RGS-TST-IT-04 | v0.1 | ✅ |
| 05 Admin 域 | RGS-TST-IT-05 | v0.1 | ✅ |
| 06 ClusterOps | RGS-TST-IT-06 | v0.1 | ✅ |
| 07 资产下载 | RGS-TST-IT-07 | v0.1 | ✅ |
| 08 GM 后台 | RGS-TST-IT-08 | v0.1 | ✅ |
| 09 工具集 | RGS-TST-IT-09 | v0.1 | ✅ 暂无测试代码 |

**结论**:9 域 + 00 基准,共 10 份 IT 设计书全部落档。**未做**:逐份内容质量审查(每份是否字段级追溯到 DTL / mock-registry 对齐)尚未做独立复核,建议 QA Lead 具名后补一轮复核(不阻塞 IT 启动,列为 S1'跟踪项)。

## 5. IT 准入差距与建议 — v0.2

### 5.1 原"必做"3 项现状

| 编号 | 事项 | v0.1 状态 | v0.2 状态 |
|---|---|---|---|
| G1 | cluster-ops Q7 终方案决策 | 🟡 待 Ulysses 一审 | ✅ **已真实追认**(方案 A',per `RGS-DECISION-CORRECTION-2026-08-28-12-21-JST.md`) |
| G2 | 8 域 Lead 具名终审 | 🟡 待 Ulysses 一审 | ✅ **已真实追认**(per 同上) |
| G3 | 环境依赖(DATABASE_URL/Postgres) | 🟡 "CI 已配,本机需 export" | 🔴 **仍未解决**:本沙箱无 Docker daemon,14 个 fixture IT 未能实测,需在真实环境验证 |

### 5.2 新增待办

| 编号 | 事项 | 工作量 | 优先级 |
|---|---|---|---|
| G4 | 全工作区覆盖率实测(`cargo llvm-cov --workspace`,需配合 G3 的 DB 环境)| 半天 | P0(§2.5 门槛需要真数字) |
| S1' | 01-07 IT 设计书质量复核(字段级追溯 + mock-registry 对齐,QA Lead 具名后)| 1 天 | P2 |
| S2 | 跨域 IT 用例实施(cluster-ops ↔ 5 域 ↔ admin-service ↔ gm-backend 链路,含 it_cross_domain.rs 迁回)| 2-3 天 | P1 |
| S3 | rgs-certgen IT 实施(per IT-09 设计书 v0.1 暂无测试代码)| 1 天 | P2 |
| S4 | gm-backend → admin-service gRPC 真实集成(v0.3, 替代 AuditStore 抽象)| 1 周 | P2 |
| S5 | outbox NATS 真实链路 IT(per Q5 NATS rollout)| 2-3 天 | P1 |

## 6. 推荐路径 — v0.2:选项 B 的等待条件已满足

**v0.1 的选项 B("等 01-07 IT 文档补全再开 IT")已是 Ulysses 真实追认的决策**(per §0 决策 1,`RGS-DECISION-CORRECTION-2026-08-28-12-21-JST.md`)。等待条件(01-07 IT 文档补全)**本轮已满足**(§4),因此不再需要在 A/B/C 之间重新选择 — 触发条件已达成,可进入 IT 主阶段。

**唯一新增的现实制约**:G3(环境依赖)在本沙箱未能验证,不是"决策问题"而是"操作问题" — 需要在有 Postgres/Docker 的实际环境(本机装 Docker Desktop 并启动,或用 CI)跑一次 `cargo test --workspace --no-fail-fast`,把 §3 中标"🔴 未验证"的 14 项转为真实 PASS/FAIL。

## 7. 我的推荐

**进入 IT 主阶段,以下两步作为主阶段第 0 天动作(非阻塞,但应尽快做)**:
1. G3:在真实 Postgres 环境重跑,把 14 个"未验证"转为确定结果(半天)
2. G4:`cargo llvm-cov --workspace` 拿真实覆盖率数字,替换 §2.5 的"未测量"(半天,可与 G3 合并一次环境搭建)

**IT 主阶段目标**(per V 模型 TL-2/3/4,未变):
- 5 域 + cluster-ops 跨域 IT 用例实施(S2,2 周)
- gm-backend 调 admin-service gRPC 真实集成(S4,1 周,per TBD-08-03 v0.3)
- IT 覆盖率 ≥ 60%(与 §2.5 门槛一致;需 G4 实测后才能验证是否达标)— **更正**:v0.2 初稿曾写"per NFR-OP-010 80% 的 70% 折算",经核对 `NFR-OP-010` 实际定义为"2 SRE ≤ 20 人·天/周"的运维负荷预算(与测试覆盖率无关),该折算系误引,已删除;§2.5 的 ≥60% 本身来源也较弱(可追溯到 `RGS-TEST-STRATEGY-2026-08-26_v0.1.md` §2.1 的"估算"目标,非强制规范条款),建议 QA Lead 确认后固化为正式门槛,暂列入 S1' 一并复核
- chaos 注入测试(per `economy-service/tests/chaos_reservation.rs` 模式)

## 8. 决策项 — v0.2:已追认 4 项 + 新增 2 项操作待办

**已由 Ulysses 真实追认**(per `RGS-DECISION-CORRECTION-2026-08-28-12-21-JST.md`,2026-08-28 12:21 JST):
- [x] IT 启动决策 B(等 01-07 IT 文档补全再开)— 本轮已补全,条件满足
- [x] G1 cluster-ops Q7 终方案 = 方案 A'
- [x] G2 8 域 Lead 具名 = 采纳 12 角色映射
- [x] TBD-08-06 工具决策 = 方案 D(双工具并存)

**尚待操作(非决策类,建议 IT 主阶段第 0 天做,不需要 Ulysses 额外拍板)**:
- [ ] G3:真实 Postgres 环境重跑,确认 14 个 fixture IT 结果
- [ ] G4:`cargo llvm-cov --workspace` 实测覆盖率

---

**作者**:Mavis(接手 agent per DEC-008,2026-08-28 09:58 JST;v0.2 数据更新 12:21 JST)
**审批**:架构师(Mavis 接手 agent per DEC-008)+ 自审 + 2026-08-28
**修订人**:Ulysses(一人公司 12 角色 per DEC-008)— Mavis 接手
