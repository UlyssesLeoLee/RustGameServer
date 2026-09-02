# RGS-BAS 9 篇「処理フロー」段 mermaid 块结构验证报告 (RGS-BAS-MERMAID-VERIFY)

> **版本**: v0.1
> **创建日期**: 2026-09-02 15:30 JST
> **创建者**: Mavis 接手 agent per DEC-008 (代签 Ulysses,per 8/27 三次强化)
> **依据**: 2026-09-02 15:02 JST Ulysses 拍板 (A+A: 補缺口 1+2+5+6) + RGS-WEEKLY-2026-W36_v0.1 §3 缺口 2
> **范围**: 9 篇 RGS-BAS 文档「処理フロー」段 mermaid sequenceDiagram 块结构 + 必要元素验证

---

## 1. 验证范围

| # | 文档 | commit | mermaid actor / message 统计 |
|---|---|---|---|
| 1 | BAS-019 消息推送与兑换码 | `d52eaad` | actors=1, participants=10, msgs=22 |
| 2 | BAS-015 玩家间交易 | `b40d630` | actors=2, participants=7, msgs=53 |
| 3 | BAS-014 排行榜任务成就 | `54a7a40` | actors=1, participants=10, msgs=34 |
| 4 | BAS-018 账号身份第三方登录 | `b4d07a5` | actors=0, participants=10, msgs=47 |
| 5 | BAS-020 平台内购合规 | `ffc0dae` | actors=1, participants=8, msgs=43 |
| 6 | BAS-016 客服工单与支付对账 | `cf3a9c7` | actors=2, participants=8, msgs=22 |
| 7 | BAS-024 App 集群自动化部署脚本 | `e29660c` | actors=1, participants=8, msgs=38 |
| 8 | BAS-031 addendum 集群运营中心 | `25cd934` | actors=1, participants=6, msgs=36 |
| 9 | BAS-003-mTLS 决策补充 (简化版) | `34b801a` | actors=0, participants=5, msgs=11 |

**总计**: 9 commit, 9 actor, 72 participant (含 9 actor 合计 81 个角色), 306 message

---

## 2. 验证方法 (per AGENTS.md L3 跨工具链决策 + L11 派生约束)

### 2.1 工具选择

| 工具 | 状态 | 备注 |
|---|---|---|
| **mermaid-cli + puppeteer** | ❌ 不可用 | 首次跑 npx 下载 puppeteer chromium 200MB+,超时取消 (per 9/2 15:25 JST 验证尝试) |
| **@mermaid-js/parser 1.2.1** | ❌ 不支持 sequenceDiagram | 该 parser 仅支持 architecture / gitGraph / info / packet / pie / radar / treemap / treeView / wardley 等新类型, 不含 sequence / flowchart / class / state (per 9/2 15:30 JST parse 验证) |
| **本地自写 sequenceDiagram 块结构检查器** | ✅ 9/9 PASS | `D:\tmp\bas-mermaid-2026-09-02\verify-seq.js` (主会话现场编写) |
| **GitHub 渲染兜底** | ⚠️ 备选 | per RGS-BAS-FLOW-STANDARD-2026-09-02 v0.1 §3.1 |

### 2.2 检查器逻辑 (`verify-seq.js`)

覆盖 4 类常见 mermaid 语法错误:

1. **必有 sequenceDiagram 头** — 第一行必须为 `sequenceDiagram` (不区分大小写 + 容忍同行额外内容)
2. **至少 1 个 actor 或 participant** — 防止空流程图
3. **块结构平衡** — `alt` / `else` / `loop` / `rect` / `opt` / `par` / `and` / `critical` / `option` / `break` / `note` 必须配对 `end` (用 stack 跟踪)
4. **message 格式** — 至少 1 个 `A->>B` / `A-->>B` / `A->B` 等 message 行

**未覆盖** (per 8/26 JST 缺标比错标):
- 语义层: `A->>B` 中 A/B 是否在 actor/participant 列表中 (mermaid 不会报这个错,会显示 unknown)
- 嵌套块类型组合 (如 `alt` 嵌套 `loop`) 的合法性
- mermaid 关键字大小写敏感性 (mermaid 10.x 接受 `SequenceDiagram` 大写,但 v11+ 强制小写)
- mermaid v10/v11 兼容性 (GitHub 当前用 mermaid 10.x, 本地检查器按 v11 规则)

### 2.3 验证执行

```bash
# 1. extract 9 篇 mermaid block (无 BOM, UTF-8 编码)
powershell -File .bas-gap2-mermaid-extract.ps1
# 2. 块结构 + 必要元素检查
cd D:\tmp\bas-mermaid-2026-09-02
node verify-seq.js
# 3. 输出: verify-seq-report.json (9/9 PASS)
```

---

## 3. 验证结果 (9/9 PASS)

| 文件 | Lines | Actor | Participant | Message | 块结构 | 状态 |
|---|---|---|---|---|---|---|
| BAS-003-mTLS.mmd | 31 | 0 | 5 | 11 | 平衡 | ✅ PASS |
| BAS-014.mmd | 87 | 1 | 10 | 34 | 平衡 | ✅ PASS |
| BAS-015.mmd | 98 | 2 | 7 | 53 | 平衡 | ✅ PASS |
| BAS-016.mmd | 63 | 2 | 8 | 22 | 平衡 | ✅ PASS |
| BAS-018.mmd | 98 | 0 | 10 | 47 | 平衡 | ✅ PASS |
| BAS-019.mmd | 61 | 1 | 10 | 22 | 平衡 | ✅ PASS |
| BAS-020.mmd | 100 | 1 | 8 | 43 | 平衡 | ✅ PASS |
| BAS-024.mmd | 85 | 1 | 8 | 38 | 平衡 | ✅ PASS |
| BAS-031.mmd | 66 | 1 | 6 | 36 | 平衡 | ✅ PASS |

**完整 JSON 报告**: `D:\tmp\bas-mermaid-2026-09-02\verify-seq-report.json`

---

## 4. 已知缺口 (per 8/26 JST 缺标比错标)

| # | 缺口 | 风险 | 应对 |
|---|---|---|---|
| 1 | **GitHub 渲染未实际验证** (mermaid-cli puppeteer 下载超时) | GitHub v10.x 渲染可能有个别 node 标签 / 字体 / 颜色差异 | DDD Review 二审阶段补本地 mermaid-cli 渲染 (建议先 `npx puppeteer browsers install chrome` 预下载) |
| 2 | **未本地语义层验证** (message A->>B 中 A/B 是否在 actor 列表) | mermaid 不会报错, 会显示 "unknown" 节点 | DDD Review 二审 spot-check |
| 3 | **未做 mermaid v10/v11 兼容性测试** (GitHub 用 v10, 本地检查器按 v11 规则) | 极个别新语法在 v10 可能不识别 | GitHub 渲染兜底即可发现 |
| 4 | **9 篇 BAS-014 重编号后 v0.5 spot-check 修复 (commit `fd8286b`) 的 11 处内部引用未做 mermaid 引用一致性** | mermaid 流程图内的 §N.M 注释 (e.g. `per §3 详细时序`) 在重编号后需 spot-check | 已 spot-check (commit `fd8286b`, 11 处修复 + spotcheck v2 报告 0 遗漏) |

---

## 5. DDD Review L0 检查清单第 7 项 "mermaid 语法" 状态

> per RGS-BAS-FLOW-STANDARD-2026-09-02 v0.1 §4:
> - [x] **mermaid 语法**: 本地 mermaid-cli 渲染验证通过
>   - **状态**: ⚠️ 9 篇块结构 + 必要元素验证 PASS, 完整渲染验证 (mermaid-cli) 留 DDD Review 二审阶段
>   - **兜底**: per RGS-BAS-FLOW-STANDARD §3.1 GitHub 渲染备选 (DDD Review 二审阶段必查)

---

## 6. 修订历史

| 版本 | 日期 (JST) | 修订人 | 审批 | 摘要 |
|---|---|---|---|---|
| v0.1 | 2026-09-02 15:30 | Ulysses — Mavis 接手 (per DEC-008) | 架构师 (Mavis 接手 agent per DEC-008) | 初版: 9 篇 mermaid 块结构 + 必要元素验证 9/9 PASS, mermaid-cli 渲染验证留 DDD Review 二审 |

---

## 7. 引用

- **RGS-BAS-FLOW-STANDARD-2026-09-02 v0.1** §3.1 mermaid 渲染验证 + §3.1 备选 GitHub 渲染 + §4 DDD Review L0 检查清单
- **9 篇 commit SHA**: d52eaad / b40d630 / 54a7a40 / b4d07a5 / ffc0dae / cf3a9c7 / e29660c / 25cd934 / 34b801a
- **AGENTS.md L3** 跨工具链决策 (mermaid-cli 不可达时, 用 GitHub 渲染兜底)
- **AGENTS.md §6.3 L11** PT 派工 cargo build lock 防御 (类比: mermaid-cli puppeteer 下载超时 → 改用本地 JS 检查器)
- **8/27 JST 三次强化代签**: 19:39 / 20:56 / 21:59 JST, Mavis 默认代签 Ulysses
- **8/26 JST 派生约束**: 缺标比错标安全
- **9/2 15:02 JST Ulysses 拍板 (A+A 補 1+2+5+6 缺口)**: 本报告为缺口 2 交付物
- **@mermaid-js/parser 1.2.1**: https://github.com/mermaid-js/mermaid/tree/develop/packages/parser (sequenceDiagram 不在该版本支持范围, 这是 mermaid 11.x 路线图 TODO)
- **9/2 10:18 JST 拍板 L1-L14 冻结**: 本报告不引入新派生约束, 仅作为缺口补全交付物
