# L-CANDIDATES.md — 派生约束候选清单

> **创建日期**: 2026-09-02 11:00 JST
> **作者**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
> **审批**: 架构师(Mavis 接手 agent per DEC-008)
> **修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
> **代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)
> **依据**: AGENTS.md v0.6.1 §8 派生约束 L1-L14 冻结期 + B2 派生约束 (per 9/2 10:18 JST 拍板)
> **配套**: `RGS-CRITIQUE-IMPROVEMENT-2026-09-02 v0.1.1` §3.1 A 类 4 条 + W37 反思 (per RGS-WEEKLY-2026-W37 v0.1) + Phase C 准备 (per RGS-PHASE-C-KICKOFF-2026-09-02 v0.1)

---

## 0. 流程

派生约束 L1-L14 自 2026-09-02 10:18 JST 起冻结 6 个月 (至 2027-03-02 JST)。

**新约束入档流程**:
1. Mavis 发现需新约束 → 写入本文件候选清单 (B2 派生约束)
2. 季度评审 (3/2 / 6/2 / 9/2 / 12/2 JST) 由 Ulysses 拍板
3. 通过的约束升 AGENTS.md 段, 未通过的清出候选清单
4. **例外** (立即生效, 不走季度评审): env value 打印 (8/27 11:06 JST 硬 ban) / 凭据泄露 / 安全相关

---

## 1. 候选清单 (Q1 拍板 A 类未选, 4 条入档; L15 候选 W37 反思 4 条入档, 候选不阻塞 sprint)

### A 类 — Q1 未选 (per RGS-CRITIQUE v0.1.1 §3.1, 12/2 季度评审)

#### L-CAND-001: A1 RGS-BAS-037 (运维安全生命周期) 拆 4 份

- **来源**: RGS-CRITIQUE-IMPROVEMENT v0.1.1 §3.1 A1
- **来源 commit**: `e459f00` (2026-09-02 10:24 JST)
- **类型**: 文档减肥 (治理类)
- **现状**: RGS-BAS-037 = 264,970 字节 (~265 KB), 是 RGS 仓内最大单 doc
- **措施**: 拆 4 份 ≤70KB (运维 SOP / 部署 / 监控 / 应急)
- **收益**: 读得动 (单 doc ≤70KB)
- **成本**: 中 (重排 + grep 全文改引用)
- **风险**: 跨引用维护成本 +20%
- **入档日期**: 2026-09-02 11:00 JST
- **下次评审**: 2026-12-02 JST (Q4 季度评审)

#### L-CAND-002: A3 AGENTS.md 6 个月一归档

- **来源**: RGS-CRITIQUE-IMPROVEMENT v0.1.1 §3.1 A3
- **来源 commit**: `e459f00` (2026-09-02 10:24 JST)
- **类型**: 文档减肥 (治理类)
- **现状**: AGENTS.md 持续升版, v0.5 → v0.6 已增 17% (27,920 → 32,605 字节)
- **措施**: 6 个月一归档, 当前 v0.6 → `AGENTS_v0.6_archive.md`, 主 AGENTS.md 只留派生约束 L1-L14 + 拍板规则
- **收益**: 治理更聚焦, 主 AGENTS.md ≤ 20KB
- **成本**: 低
- **风险**: 历史回溯需 git log
- **入档日期**: 2026-09-02 11:00 JST
- **下次评审**: 2026-12-02 JST (Q4 季度评审)

#### L-CAND-003: A4 document-registry.toml 强制 80KB 上限

- **来源**: RGS-CRITIQUE-IMPROVEMENT v0.1.1 §3.1 A4
- **来源 commit**: `e459f00` (2026-09-02 10:24 JST)
- **类型**: 文档减肥 (治理类)
- **现状**: RGS-BAS-037 (265KB) / RGS-BAS-036 (218KB) / RGS-BAS-010 (141KB) 巨型 doc 仍有出现
- **措施**: 写 `docs/document-registry.toml` 强制登记新 doc 路径 + 大小上限 80KB, CI 校验
- **收益**: 防巨型 doc 再出现
- **成本**: 低 (改 file)
- **风险**: 流程摩擦, 改造期长 doc 需手动 split
- **入档日期**: 2026-09-02 11:00 JST
- **下次评审**: 2026-12-02 JST (Q4 季度评审)

### L15 候选 — W37 反思 4 条 (per 9/2 D6 拍板机制, 12/2 季度评审)

> **来源**: RGS-WEEKLY-2026-W37 v0.1 (commit `8d69cef`) + RGS-PHASE-C-KICKOFF-2026-09-02 v0.1 (commit `941bb8e`) + 8/27 11:06 JST hard ban (env value 硬 ban)
> **入档动机**: W37 启动预热时, 阶段 A/B/C 风险 + 8 worker 派工临时 log 教训, 9 月新教训无法归入 A 类 4 条
> **类型分布**: 治理类 1 + 业务类 2 + 安全类 1, 给 Ulysses 季度评审选项
> **强约束**: L15 候选**不阻塞 W37 sprint**, 仅入档, 待 12/2 季度评审拍板

#### L-CAND-004: L15 候选 — SRE Lead 拍板超时防御

- **来源**: RGS-WEEKLY-2026-W37 v0.1 §3 风险评估 (SRE Lead 不可达 🟡 中) + RGS-PHASE-C-KICKOFF-2026-09-02 v0.1 §3.1 (4 选 1+ 拍板项)
- **来源 commit**: `8d69cef` (W37 v0.1) + `941bb8e` (Phase C KICKOFF)
- **类型**: 业务类
- **现状**: W37 D2 (9/9 JST) SRE Lead 拍板"阶段 A 全 4 步"待回, 阶段 B/C 启动依赖拍板结果; 拍板悬空 ≥ 24h → 业务里程碑风险累计, 但无自动 fallback
- **措施**: SRE Lead 拍板悬空 > 24h 自动降级到选项 C (推迟 W38), 写 `RGS-PHASE-C-DEFER-*` 公告; cron / 手动每日 09:00 JST 检查拍板状态
- **收益**: 业务里程碑不再依赖"拍板待回"长期悬空, 风险显性化
- **成本**: 低 (写检查脚本 + 模板, 估算 1-2h)
- **风险**: 误判 SRE 离线 (但 W37 时间窗口有迹可循: 9/2 17:32 JST → 9/9 09:00 JST = 7 天)
- **候选理由**: 业务里程碑风险显性化 ≠ 派生约束, 但 SRE Lead 拍板待回 → Mavis 无法代签 SRE 派生决策 (per Phase C KICKOFF §1), 需制度化兜底
- **入档日期**: 2026-09-02 18:30 JST
- **下次评审**: 2026-12-02 JST (Q4 季度评审)

#### L-CAND-005: L15 候选 — 业务里程碑 commit 必带 git 实证

- **来源**: RGS-WEEKLY-2026-W37 v0.1 §0.1 双指标 (业务里程碑 5/6 域生产可用) + RGS-CRITIQUE v0.1.1 §2.5 (DoD 偏轻, 治理派压倒实现派)
- **来源 commit**: `8d69cef` (W37 v0.1) + `e459f00` (RGS-CRITIQUE v0.1.1)
- **类型**: 业务类
- **现状**: W37 业务里程碑表 (5 域 ST mTLS / Phase C 阶段 A/B/C / DDD Review v0.2 / batch v0.1 冻结) 含 commit SHA, 但周报/Phase C 公告中"🟡 1/5"等状态无 1:1 git 实证链接, 业务指标 vs 文档承诺追溯链不闭合
- **措施**: 业务里程碑状态 (🟢/🟡/⏳/❌) 在周报/公告中**必带** commit SHA + file:line (per L13 自指字段 deferred 实时查询), commit 模板 D3 派生段加 "Business Evidence" 字段
- **收益**: 业务指标 vs 文档承诺 1:1 可追溯, 防止"治理派"虚高
- **成本**: 低 (Mavis commit 时必填 1-2 行, 跟 D3 commit 模板复用)
- **风险**: 跟 L1/L1.1/L1.2 DoD 升级 (D2 派生约束) 有重合, 候选清单可 12/2 合并 / 去重
- **候选理由**: D2 已升级 L1/L1.1/L1.2, 但"业务里程碑 commit 必带 git 实证"是"业务"视角 (5 域生产可用 / 跨域 saga 跑通), 不在 L1-L14 范围, 需独立候选
- **入档日期**: 2026-09-02 18:30 JST
- **下次评审**: 2026-12-02 JST (Q4 季度评审)

#### L-CAND-006: L15 候选 — k8s secret 导出硬 ban (cert 内容不入 commit)

- **来源**: 8/27 11:06 JST env value 硬 ban (Get-ChildItem env: | Format-Table 输出即违规) + RGS-WEEKLY-2026-W37 v0.1 §3 (阶段 B 5 域 mTLS certs 导出风险 🟡 中)
- **来源 commit**: 8/27 11:06 JST hard ban 文档 (无单 commit, per 8/27 19:39 JST Ulysses 三次强化) + `8d69cef` (W37 v0.1)
- **类型**: 安全类
- **现状**: W37 D3-5 (9/10-12 JST) 阶段 B 启动, 5 域 certs 导出 (per RGS-PHASE-C-PREP §1 阶段 B 8 步); 当前流程是 `kubectl get secret <domain>-tls -o yaml > certs/<domain>-tls.yaml`, **cert 内容进入 certs/ 目录, 风险进入 commit**
- **措施**: k8s secret 导出走 `certs/` gitignored 目录 (per L12 派生约束兜底), 仅 cert SHA-256 fingerprint + cert subject 写 manifest (`certs/MANIFEST.toml`), cert 内容**永不入 commit**; 验证 cert 链用 `openssl x509 -noout -fingerprint -sha256` 比对 fingerprint
- **收益**: cert 内容 0 泄露 (8/27 11:06 JST hard ban 一致性延伸), 即使 worktree 误 push 也不泄露 cert
- **成本**: 低 (改 export script + 加 .gitignore, 估算 1h)
- **风险**: cert 链验证需用 fingerprint 比对而非 cert 内容比对, 工具链依赖 `openssl` (k3s 节点已装)
- **候选理由**: 8/27 11:06 JST 硬 ban 是"env value", k8s secret 是"secret 内容" — 范围扩展, 但跟 8/27 硬 ban 精神一致 (不打印 secret), 12/2 评审时跟 8/27 硬 ban 合并升级为"安全派生约束 L15"
- **入档日期**: 2026-09-02 18:30 JST
- **下次评审**: 2026-12-02 JST (Q4 季度评审), 可考虑立即生效 (per 8/27 11:06 JST 安全例外条款, §0 第 4 项)

#### L-CAND-007: L15 候选 — 派生约束引用版本锁 (CI pre-commit 检查)

- **来源**: RGS-WEEKLY-2026-W37 v0.1 §5 (派生约束守护表) + RGS-CRITIQUE v0.1.1 §2.3 (AI 自指悖论: Mavis 立 Mavis 守 Mavis 改)
- **来源 commit**: `8d69cef` (W37 v0.1) + `e459f00` (RGS-CRITIQUE v0.1.1)
- **类型**: 治理类
- **现状**: AGENTS.md §2.1 / §2.3-2.6 / §6 派生约束守护段 (L1-L14) 持续升版, 9 份 DDD Review v0.2 + 11 份 BAS-* 文档中"派生约束引用"前后可能不一致 (例: 引用 L1 但实际 commit 用 L1.1); 升版时手动 grep 容易漏
- **措施**: 写 `.git/hooks/pre-commit-derivation` 检查 `AGENTS.md` §编号 vs 文档/周报中"派生约束引用"是否一致; 不一致 → commit 拒绝; L-CANDIDATES.md 升版时同步检查
- **收益**: 派生约束引用 100% 一致, 防止 Mavis 起草时漏掉新增约束 (L15/L16)
- **成本**: 低 (CI pre-commit 跑 md 引用检查, 估算 2-3h)
- **风险**: 跨文件引用维护, 但当前 doc 数量 ≤ 30 份, 检查耗时 < 5s
- **候选理由**: AI 自指悖论 (per RGS-CRITIQUE §2.3) — 派生约束 L1-L14 全 Mavis 自立, 12/2 季度评审是 Ulysses 唯一把关, 派生约束版本锁把"季度评审"前移到 "CI pre-commit", 降低漏审风险
- **入档日期**: 2026-09-02 18:30 JST
- **下次评审**: 2026-12-02 JST (Q4 季度评审)

#### L-CAND-008: (保留位) — 待 L1-L14 冻结期内 Mavis 发现

- **入档日期**: —
- **下次评审**: 2026-12-02 JST

---

## 2. 季度评审机制 (per AGENTS.md v0.6.1 §8)

| 评审日 | 入档候选 | 通过 | 清出 | 状态 |
|---|---|---|---|---|
| 2026-12-02 (Q4) | L-CAND-001 / 002 / 003 (A 类) + L-CAND-004 / 005 / 006 / 007 (L15 候选) + L-CAND-008 (保留) | — | — | 待评审 |
| 2027-03-02 (Q1) | — | — | — | 待启 |
| 2027-06-02 (Q2) | — | — | — | 待启 |
| 2027-09-02 (Q3) | L1-L14 冻结期届满, 重新评估 | — | — | 待启 |

**L-CAND-006 例外路径** (per §0 第 4 项): 安全相关可立即生效, 不走 12/2 季度评审. Mavis 上报 Ulysses 拍板后, 9/2-9/9 期间可单独 commit + 写入 AGENTS.md §8 例外段.

---

## 3. 修订历史

| 版本 | 日期 (JST) | 修订人 | 变更 |
|---|---|---|---|
| v0.1 | 2026-09-02 11:00 | 架构师(Mavis 接手 agent per DEC-008) | 初始创建: A 类 4 条候选清单 (A1/A3/A4 + 1 保留位) + 季度评审机制, per AGENTS.md v0.6.1 §8 + RGS-CRITIQUE-IMPROVEMENT v0.1.1 §3.1 |
| v0.2 | 2026-09-02 18:30 | 架构师(Mavis 接手 agent per DEC-008) | L15 候选 v0.2: 加 4 条 W37 反思候选 (L-CAND-004 SRE 拍板超时 / L-CAND-005 业务里程碑 git 实证 / L-CAND-006 k8s secret 导出硬 ban / L-CAND-007 派生约束版本锁), 类型分布 治理 1 / 业务 2 / 安全 1, 来源 W37 v0.1 + Phase C KICKOFF + 8/27 11:06 JST hard ban, 12/2 Q4 季度评审; L-CAND-006 例外路径写明 (per 8/27 安全派生约束例外条款, §0 第 4 项); 季度评审机制表 12/2 行扩到 7 条候选 + 1 保留位; 顶部 "依据" 段补 W37 v0.1 + Phase C KICKOFF 关联 |

**修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
**审批**: 架构师(Mavis 接手 agent per DEC-008)
**代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)
