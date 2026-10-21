# D4 出口校验清单

## A. 设计 Token

- [ ] color JSON 含 primitive (灰阶/原色) + semantic (bg/text/primary/...)
- [ ] light / dark / 高对比 模式齐全
- [ ] typography 字号阶梯 + 字重 + 行高
- [ ] spacing 4px 或 8px 基准
- [ ] radius / shadow / motion 都 JSON 化
- [ ] 全部 token 有 light/dark 两套值 (或说明为何不需要)

## B. 对比度

- [ ] 文本 vs 背景 ≥ 4.5:1 (WCAG AA)
- [ ] 组件 vs 背景 ≥ 3:1
- [ ] 焦点环在任何背景下 ≥ 3:1

## C. 组件库

- [ ] L1 ≥ 12 个
- [ ] L2 ≥ 8 个
- [ ] L3 ≥ 6 个
- [ ] L4 ≥ 4 个
- [ ] 组件总数 ≥ 30 个 (= 12 + 8 + 6 + 4)
- [ ] 每个组件都有 Component Spec
- [ ] 任意 L1 组件可独立 Storybook 展示 (无业务依赖)
- [ ] Storybook 故事覆盖 variant × size 笛卡尔积 + 全部 state

## D. 组件约束

- [ ] 没有任何组件使用裸 hex (只能 token)
- [ ] 焦点环永不被 outline:none 抹掉
- [ ] 减弱动效支持 (prefers-reduced-motion)
- [ ] 暗色模式独立视觉验证

## E. 布局

- [ ] 栅格: 12 列, gutter 16/24
- [ ] 断点 5 个 (sm/md/lg/xl/2xl)
- [ ] z-index 全局表覆盖 base / sticky / modal / toast / tooltip

## F. 图标

- [ ] 同源风格 (笔画 / 网格一致)
- [ ] 命名 kebab-case 动宾结构
- [ ] license 已审计

## G. 响应式

- [ ] 320 / 375 / 768 / 1024 / 1280 / 1440 全部测过
- [ ] 触控目标 ≥ 44×44 px
- [ ] 桌面悬停态 / 移动无悬停 兼容

## H. 减弱动效

- [ ] prefers-reduced-motion 已实测触发
- [ ] 所有动效时长缩至 0 或短至可忽略

## I. 已知缺口

- [ ] 暗色模式对比度复测
- [ ] i18n 文案扩展导致布局问题
- [ ] 图标 license 覆盖
- [ ] 减弱动效 iOS / Android 实测

---

## 通过标准
A-I 全部 ☑。

## 反模式
- ❌ 颜色就地写 hex,后期改色 100 文件
- ❌ 字号 13 / 15 / 17 任意值,完不成节奏
- ❌ 焦点环"为了好看"被删,屏读用户卡死
- ❌ 暗色模式="亮色反色",对比度爆雷
- ❌ z-index 9999 → 谁都盖不住那个右上角浮窗
