# D4 — Visual Language（视觉语言层）

> **5D Atlas v9.0 · D4/5**
> **目的**: 用一套**可复用、可度量、可演化的视觉规则**, 把 D1-D3 的抽象结果做成可看的页面。
> **失败模式**: 凭审美拍板, 不立 token —— 工程与设计脱节, 半年后无法维护。

---

## 0. 本层回答的核心问题

| # | 问题 | 不回答的后果 |
|---|---|---|
| Q1 | **颜色 / 字体 / 间距有哪些原子值** | 工程师各自魔改, 视觉漂移 |
| Q2 | **如何从原子值组装组件** | 设计稿与代码各说各话 |
| Q3 | **暗色模式 / 多品牌如何切换** | 二次开发时全部重做 |
| Q4 | **图标 / 插画 / 动效规范** | 越用越乱, 失去品牌识别 |
| Q5 | **响应式断点与栅格** | 移动端崩, 信息密度错位 |
| Q6 | **可访问性的视觉约束** | 色盲 / 光敏感用户无法使用 |

---

## 1. 设计令牌 (Design Tokens)

设计令牌是**所有视觉决策的源头**: 颜色、字体、间距、圆角、阴影、动效。

### 1.1 颜色

```markdown
## 原色 (Primitive)
- gray-50 ~ gray-900 (11 阶)

## 语义色 (Semantic)
- bg / surface / surface-elevated
- text / text-muted / text-disabled
- primary / primary-hover / primary-active
- success / warning / danger / info
- focus-ring (A11y 必填)

## 对比度
- 文本 vs 背景: WCAG AA ≥ 4.5:1 (WCAG AAA ≥ 7:1)
- 组件 vs 背景: ≥ 3:1

## 暗色模式
- 不"反色", 而是**重选语义色**: 亮色文字 + 弱化纯黑 + 提亮分层
```

**判据**: 颜色**不允许**就地写 hex, 必须引用 token (`var(--color-primary)` or 对应类)。

### 1.2 字体

```markdown
## 字体族
- 西文: Inter / Roboto
- 中文: 思源黑体 / 系统字体
- 等宽: JetBrains Mono / Fira Code

## 字号阶梯 (modular scale, 推荐 1.2 或 1.25)
- 12 / 14 / 16 / 20 / 24 / 32 / 48

## 行高
- 标题: 1.2
- 正文: 1.5 - 1.7
- 等宽: 1.4

## 字重
- 300 / 400 / 600 / 700 (不用 500 当 heading)
```

### 1.3 间距 / 圆角 / 阴影 / 动效

```markdown
## 间距阶梯 (4px 或 8px 基准)
- 0 / 4 / 8 / 12 / 16 / 24 / 32 / 48 / 64

## 圆角
- none / sm (4) / md (8) / lg (16) / full (9999)

## 阴影
- 0 / sm / md / lg / overlay (大遮罩)

## 动效
- 时长: instant (0) / fast (150ms) / base (250ms) / slow (400ms)
- 缓动: ease-out (进入) / ease-in (退出) / linear (循环)
- 减弱: prefers-reduced-motion: 全部时长降至 0
```

### 1.4 Token 文件结构 (推荐)

```
tokens/
├── color.json
├── typography.json
├── spacing.json
├── radius.json
├── shadow.json
├── motion.json
└── README.md
```

每 token 都有 **light / dark / 品牌 / 高对比** 等多值,运行时切换。

---

## 2. 组件库 (Component Library)

### 2.1 组件分层

```markdown
## L1 原子 (原子, 不可再分)
- Button / Icon / Input / Tag / Avatar

## L2 分子 (由原子组合)
- FormField (= Label + Input + Hint)
- ListItem (= Avatar + Title + Meta + Action)

## L3 组织 (页面级布局)
- PageHeader / CardGrid / Drawer

## L4 模板 (页面骨架)
- DetailPage / ListPage / WizardPage
```

**判据**: L1 必须**没有业务依赖**, 可独立 Storybook 展示;L4 模板页允许有业务依赖。

### 2.2 组件规格

每组件写一份规约:

```markdown
## Component — Button
### 用途
- 触发即时的、明确的、本地操作

### 变体 (Variant)
- primary / secondary / ghost / danger
- size: sm / md / lg
- state: default / hover / active / focus / disabled / loading

### API (props)
- variant, size, disabled, loading, icon, onClick, type
- 必填: variant, children
- 禁用规则: loading 时禁用 onClick

### 可访问性
- 焦点环: var(--focus-ring), 不得被 outline: none 抹掉
- 加载中: aria-busy, 不重渲染 label

### 已知状态
- 每个 state 都有视觉 + 屏读标注
```

---

## 3. 布局 (Layout)

### 3.1 栅格

```markdown
## 栅格
- 12 列栅格
- gutter: 16 / 24 (桌面 / 移动)
- max-width: 1200 / 1440

## 断点
| 名称 | min-width | 列数变化 |
|---|---|---|
| sm | 640 | 4 列堆叠 |
| md | 768 | 8 列 |
| lg | 1024 | 12 列 |
| xl | 1280 | 12 + 内容居中 |
```

### 3.2 Z-Index 地图

```markdown
## z-index 全局表
- base: 0
- sticky: 10
- dropdown: 100
- sticky-bar: 200
- modal-backdrop: 1000
- modal: 1001
- toast: 2000
- tooltip: 3000
```

**判据**: z-index **禁**用魔术数字;必须引用该表。

---

## 4. 图标与插画 (Iconography)

```markdown
## 图标体系
- 风格: 线性 / 双色 / 面状, 全产品一致
- 网格: 24×24 (UI) / 16×16 (小标)
- 笔画: 1.5 / 2px (粗细一致)
- 命名: kebab-case, 动宾结构 (icon-add-user)

## 插画风格
- 几何 / 等距 / 抽象, 保持 token 配色
- 不抢戏: 装饰性, 不承载信息
```

---

## 5. 响应式与设备适配

```markdown
## 视口
- 320 / 375 / 414 / 768 / 1024 / 1280 / 1440 / 1920
- 优先测: 375 / 768 / 1440

## 触控
- 触控目标 ≥ 44×44 px
- 留白 ≥ 8px 防误触

## 桌面专属
- 悬停态必填 (移动不可有)
- 键盘焦点必填 (仅桌面?)
```

---

## 6. 视觉与品牌一致性

```markdown
## 品牌一致性自检
- L1 组件是否全产品统一变体 (主色)?
- 空 / 错 / 加载三态是否一致?
- 字体阶梯是否全产品统一?
- 错误图标是否同一风格?
```

---

## 7. D4 出口判据 (Exit Criteria)

未通过即**禁止进入 D5**:

- [ ] 设计 token JSON 文件齐备 (color/typo/space/radius/shadow/motion)
- [ ] 组件 ≥ 20 个, 每个都有规格
- [ ] 任意 L1 组件可独立 Storybook 展示
- [ ] 暗色 / 高对比 token 已配
- [ ] z-index 全局表与栅格断点文档化
- [ ] 对比度全部 ≥ WCAG AA
- [ ] 响应式断点全过, 触控 ≥ 44px
- [ ] 动效减弱 (reduced-motion) 已支持

---

## 8. 已知缺口 (Known Gaps)

| 缺口 | 影响 | 补足方式 |
|---|---|---|
| 暗色模式未做对比度复测 | 部分用户难读 | 上线前色盲模拟器跑 |
| 组件库未做 i18n 适配 | 中文 / 英文宽度不同 | 上线前跑 2 语言 |
| 图标资产未审计 | 违反版权 | 上线前 license 扫表 |
| 减弱动效未实装 | 光敏感用户不适 | prefers-reduced-motion 实测 |

---

## 9. 接口与下游

| 输出物 | 给 D5 工程 |
|---|---|
| Tokens (JSON) | CSS 变量 / Tailwind / theme object |
| 组件规格 | Storybook / 设计系统仓库 |
| 栅格断点 | 媒体查询 / 容器查询 |
| z-index 表 | 全局 less/scss 文件 |

---

## 附录 A: 一页交付物清单

```markdown
# Token 列表
- color: 11 阶 + 4 语义 ✕ 2 模式 ✅
- typography: 7 阶 ✕ 4 字重 ✅
- spacing: 9 阶 ✅
- radius: 5 阶 ✅
- shadow: 5 阶 ✅
- motion: 3 时长 + 4 缓动 ✅

# 组件库
- L1: 12 / L2: 8 / L3: 6 / L4: 4

# 视觉自检
- 对比度: AA ✅
- 减弱动效: ✅
- 暗色模式: ✅
```
