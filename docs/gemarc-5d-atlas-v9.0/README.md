# Universal 5D Design Atlas v9.0

> 一个面向设计-工程协作的 5 维方法论。

## 它是什么

把"做产品"这件事分成 5 个互相咬合的层, 每层有**模板** + **出口判据** + **与其他层的接口**:

| 层 | 一句话 | 失败模式 |
|---|---|---|
| **D1** Concept & Strategy | 把"为什么、给谁、怎样算成功"钉死 | 战略缺位 → D2-D5 全部失焦 |
| **D2** Information Architecture | 把价值主张拆成可被找到/理解的内容 | 用户找不到东西 |
| **D3** Interaction & Flow | 把 IA 串成可验证的旅程 + 状态机 | 只画 happy path 掉坑 |
| **D4** Visual Language | 用 token + 组件库做成可看的页面 | 凭审美拍板难维护 |
| **D5** Engineering & Delivery | 做成可部署/可监控/可回滚/可演化 | 上线即盲盒 |

## 目录

```
gemarc-5d-atlas-v9.0/
├── index.html              ← 入口, 单文件离线可用
├── README.md               ← 本文件
├── QA_FINAL.md             ← 最终 QA 报告 + 已知缺口
├── atlas/                  ← 5 维方法论文档
│   ├── D1-concept.md
│   ├── D2-information-architecture.md
│   ├── D3-interaction.md
│   ├── D4-visual.md
│   └── D5-engineering.md
├── templates/              ← 5 套可复用模板
│   ├── D1-concept-brief.template.md
│   ├── D2-ia-map.template.md
│   ├── D3-flow-spec.template.md
│   ├── D4-component-spec.template.md
│   └── D5-adr.template.md
├── checklists/             ← 出口判据 (单层 + 集成)
│   ├── D1-checklist.md
│   ├── D2-checklist.md
│   ├── D3-checklist.md
│   ├── D4-checklist.md
│   ├── D5-checklist.md
│   └── cross-d-integration-check.md
└── examples/               ← 假设性 worked example
    └── hypothetical-product/
        ├── README.md
        ├── 01-concept.md
        ├── 02-ia-map.md
        ├── 03-flow-spec.md
        ├── 04-component-specs.md
        └── 05-engineering-summary.md
```

## 怎么用

### 当你要做一个新产品/新功能

1. **D1** — 复制 `templates/D1-concept-brief.template.md`, 填完跑 `checklists/D1-checklist.md`, 全 ☑ 才进 D2
2. **D2** — 复制 `templates/D2-ia-map.template.md`, 跑 D2 checklist
3. ... 依此类推, 每层必走两步: 填模板 + 跑清单
4. 全部完成后跑 `checklists/cross-d-integration-check.md` 做接口一致性

### 当你要给团队看"方法论是什么"

直接打开 `index.html`, 从卡片点进任何一层。

### 当你想看"方法论具体长什么样"

打开 `examples/hypothetical-product/`, 看 Arcade Control Panel 是怎么从模板填出 5 份文档的。

## 与 RGS (本仓库) 的关系

> ⚠️ 本 atlas **不替代** RGS 自身的产品文档 (RGS 文档在 `docs/14-项目治理/`, 各域 RACI)。
> 它是一份**通用方法论**, 当 RGS 团队需要新增"非游戏核心功能" (如后台 / 工具 / 控制面板)
> 时, 可以借鉴本 atlas 的 5 维流程, 但不要直接把内容当 RGS 战略用。

按 governance 设置 `keep_unresolved_in_backlog: true`, 本 atlas 的所有"已知缺口"会
显式列出, 不假装覆盖。

## 离线优先

- `index.html` 单文件可双击打开, 完全离线
- 不依赖 npm 构建 (per `execution_environment.offline_core_required: true`)
- 视觉风格自带 minimal dark theme, 减弱动效 (`prefers-reduced-motion`) 已支持
- 所有 markdown 文件之间仅靠相对链接, 文件夹搬迁不会断链

## 谁来修订

按 AGENTS.md §0 代签规则:
- **修订**: Ulysses — Mavis 接手 (临时代签)
- **审批**: 架构师 — Mavis 接手 (临时代签)
- **守门**: 8/27 11:06 JST env 硬 ban + 9/1 18:30 JST DB 横展三分类 + 9/8 15:19 JST 第 6 次强化

真人 (Ulysses / 架构师 / 5 域 Lead) 到位后, 在修订历史 +1 行追溯签字覆盖 (per 守门 #14 v3+v4)。

## 版本历史

| 版本 | 日期 | 摘要 | commit |
|---|---|---|---|
| v9.0 | 2026-10-21 | 初版: 5 维文档 + 5 模板 + 6 清单 + 1 假设性示例 + 入口 HTML | (待 commit) |

## 下版本候选 (backlog)

- [ ] 增加 D2 IA lint CI 实装模板 (perls / js / rs)
- [ ] 增加 D3 旅程 → mermaid 转换器
- [ ] 增加 D4 token → Tailwind / CSS variable / iOS / Android 转换器
- [ ] 增加 D5 ADR → 索引页面自动生成器
- [ ] 增加 multilingual 自动检测 (zh-CN / ja-JP / en-US)
- [ ] 与 visual-page / web-design skill 联动 (用户没要求, 候选)
