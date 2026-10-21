# D4 组件规格模板 (Component Spec Template)

---

# Component — [名称]

## 用途
> 一句话讲清"何时用、为何用、不可用时用什么替代"

## 变体 (Variant)
| 维度 | 取值 |
|---|---|
| 颜色 | primary / secondary / ghost / danger |
| 尺寸 | sm / md / lg |
| 状态 | default / hover / active / focus / disabled / loading |

## API (Props)
| Prop | 类型 | 必填 | 默认 | 备注 |
|---|---|---|---|---|
| variant | enum | ✅ | "primary" | |
| size | enum | | "md" | |
| disabled | boolean | | false | loading 时强制 true |
| loading | boolean | | false | aria-busy |
| onClick | fn | | | |
| type | enum | | "button" | submit / reset / button |

## 视觉规则 (引用 D4 tokens)
- 背景: `var(--color-{variant})`
- 文本: `var(--color-text-on-{variant})`
- 圆角: `var(--radius-{size})`
- 阴影: `var(--shadow-{state})`

## 行为
- Hover: 不变色仅亮度 +5%
- Active: scale(0.98)
- Loading: 内部 spinner + 文字保留 (屏读)
- Disabled: opacity 0.5 + cursor not-allowed

## 可访问性
- 焦点环: 永远 `var(--focus-ring)`,永不被 `outline:none` 抹掉
- aria-busy: loading 时为 true
- aria-disabled: disabled 时为 true
- 文本需有对比度 ≥ 4.5:1

## 禁用组合 (避免坏用法)
- ❌ Primary + danger 同时
- ❌ Loading 时 onClick 还能触发

## Storybook 故事清单
- [ ] 所有 variant × size 笛卡尔积
- [ ] 所有 state
- [ ] 与 Icon / Loading / Disabled 组合
- [ ] 暗色模式

---

## 组件 spec 离线校验
- [ ] 引用 token 非魔数
- [ ] 键盘可达
- [ ] 屏读朗读完整 label
- [ ] 色盲模拟下状态可识别
- [ ] 减弱动效下无 jarring 过渡
