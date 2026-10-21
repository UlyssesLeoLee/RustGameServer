# D4 Worked Example — ACP Key Components

> ⚠️ 假设性示例。展示 3 个关键组件,非完整组件库。

---

# Component — MachineStatusChip

## 用途
> 在 dashboard 网格中显示一台机台的状态,单字符 / 图标 / 颜色多通道编码。

## 变体
| 维度 | 取值 |
|---|---|
| 状态 | online / offline / fault / maintenance |
| 尺寸 | sm / md / lg |
| 强调度 | default / strong (悬停) |

## API
| Prop | 类型 | 必填 | 默认 |
|---|---|---|---|
| status | enum | ✅ | — |
| size | enum | | "md" |
| lastSeen | datetime | | — |
| strong | bool | | false |

## 视觉规则 (引用 D4 tokens)
- 背景: `var(--color-status-{status}-bg)`
- 文字: `var(--color-status-{status}-fg)`
- 圆角: `var(--radius-full)` (胶囊)
- 描边: `var(--color-status-{status}-border)`, focus 时 2px

## 行为
- Hover: 边框变粗;tooltip 显示 last_seen
- Loading (心跳中): pulse 动画 1.5s, 可被 reduced-motion 禁用
- Stale (lastSeen > 60s): 状态色降级为 warning + "陈旧"标签

## 可访问性
- 焦点环: 永远显示 `var(--focus-ring)`
- aria-label 完整: "机台 12, 故障 3 分钟前"
- 屏读朗读: 状态 + 持续时间
- 色盲: 状态配图标 (✓ ● ! 🔧)

## 禁用组合
- ❌ `status=online` 同时显示 "故障"
- ❌ `strong=true` 同时 `size=sm` (空间不允许)

## Storybook 故事
- [ ] status × size 全笛卡尔积
- [ ] stale 状态
- [ ] 暗色模式
- [ ] 减弱动效
- [ ] 屏读朗读 (用 NVDA 录)

---

# Component — FaultCard

## 用途
> 一台故障的卡片:机台 / 严重度 / 时间 / 操作按钮组。

## 变体
| 维度 | 取值 |
|---|---|
| severity | info / warn / error / critical |
| acked | bool |
| size | compact / full |

## API
| Prop | 必填 | 类型 |
|---|---|---|
| fault | ✅ | FaultEvent |
| onAck | | fn |
| onView | | fn |
| onClose | | fn |

## 视觉
- 左边: 严重度条状色 (4px 宽) `var(--color-severity-{s})`
- 头部: 机台号 + 严重度 chip (复用 MachineStatusChip)
- 中部: 检测时间 + 持续时间 + 描述
- 底部: 操作按钮组 (Ack / View / Close)

## 行为
- ack 后: chip 状态由"未读"→"已读",按钮组收敛
- view 后: 抽屉打开 (按 D3 J-1 走)
- close 前: 必弹二次确认 (防误操作)

## 可访问性
- aria-live: polité 由"未读"变"已读"时声明
- Tab 顺序: ack → view → close
- 焦点环全显
- 屏读: "故障 X 检测于..., 严重..., 操作..."

## Storybook
- [ ] severity × acked 全笛卡尔积
- [ ] 二次确认弹窗
- [ ] 减弱动效 (无 pulse, 无缩放)
- [ ] 屏读朗读

---

# Component — DashboardGrid

## 用途
> 网格布局,按机台状态排序,故障优先级高的占前列。

## 变体
| 维度 | 取值 |
|---|---|
| density | comfortable / compact |
| filter | 全部 / 故障 / 离线 |
| sort | severity / last_seen / revenue |

## API
| Prop | 必填 | 类型 |
|---|---|---|
| machines | ✅ | Machine[] |
| filter | | enum |
| sortBy | | enum |
| onCardClick | | fn |

## 视觉
- 4-6 列栅格 (响应式)
- 每格 = MachineStatusChip + 机台号 + 心跳时间
- 故障项 1px 红边 + pulse (可减弱)
- 数据陈旧项角标 "陈旧"

## 行为
- 心跳 5s 刷新
- 拖拽排序: 本期不做 (留 hook)
- 双击 → 详情抽屉

## 可访问性
- 整组 grid: role="grid"
- 每 cell: role="gridcell" + aria-label 完整
- 键盘: 方向键导航
- 屏读: 进入即朗读"X 台机台, Y 台故障"

## Storybook
- [ ] 50 / 200 / 1000 台机台性能
- [ ] 暗色模式
- [ ] 屏读首次进入 / 刷新后
- [ ] 减弱动效 (无 pulse)

---

## D4 出口判据跑分
- ✅ 颜色 token 引用
- ✅ 焦点环保留
- ✅ 色盲图标辅助
- ✅ 减弱动效可关
- ✅ Storybook 故事清单

## 已知缺口
- 完整 12 个 L1 组件未列 (此处仅 3 个示例)
