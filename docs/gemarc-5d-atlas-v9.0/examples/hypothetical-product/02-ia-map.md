# D2 Worked Example — ACP IA Map

> ⚠️ 假设性示例。

---

# 项目: ACP

## 1. 内容模型

### 实体清单
| # | 实体 | 必填字段 | 状态机 | 关联 |
|---|---|---|---|---|
| 1 | 机台 (ArcadeMachine) | id, store_id, model, status | online/offline/fault/maintenance | store, fault_log |
| 2 | 故障 (FaultEvent) | id, machine_id, detected_at, severity | detected/acked/resolved/closed | machine, acker |
| 3 | 班次 (Shift) | id, store_id, opened_by, opened_at | open/closed | user, summary |
| 4 | 营收快照 (RevenueSnapshot) | id, machine_id, ts, delta | (append-only) | machine |

### 字段描述示例 (机台)
| 字段 | 类型 | 必填 | 理由 |
|---|---|---|---|
| id | string | ✅ | 唯一标识 |
| status | enum | ✅ | dashboard 必显 |
| last_seen_at | datetime | ✅ | "心跳"判断失联 |
| store_id | string | ✅ | 多店 hook |

## 2. 分类法

| 维度 | 互斥 | 取值数 | 例子 |
|---|---|---|---|
| severity | ✅ | 4 | info / warn / error / critical |
| store | ❌ | N | 多选 |
| machine status | ✅ | 4 | online / offline / fault / maintenance |

## 3. 站点地图

### L0 (≤ 7)
- 概览
- 故障
- 机台
- 班次
- 报表
- 设置
- 关于

### L1 (≤ 9 / 分组)
```
- 概览 (Dashboard)
  - 实时网格
  - 故障热点
- 故障 (Fault)
  - 当前活跃
  - 历史记录
- 机台 (Machine)
  - 列表
  - 详情
- 班次 (Shift)
  - 当前班次
  - 历史
- 报表 (Report)
  - 当日对账
  - 周报
- 设置 (Settings)
  - 推送策略
  - 数据同步
- 关于 (About)
```

### 多入口

| 对象 | 入口 1 | 入口 2 | 入口 3 |
|---|---|---|---|
| 机台详情 | 概览 → 网格 → 详情 | 故障 → 关联 → 详情 | 报表 → Top 5 → 详情 |
| 故障详情 | 概览 → 故障点 | 故障列表 → 详情 | 推送 → 直达 |

## 4. 同义词表

| 规范词 | 同义词 | 用途 |
|---|---|---|
| 机台 | 机器 / 机柜 | 店门口招贴用"机柜", 内部全用"机台" |
| 故障 | 报错 / 异常 | 屏读 / 推送用"故障", 后端日志用"异常" |

## 5. 检索 5 要素
- 索引范围: 当前用户可见的机台 + 故障
- 排序: 严重度倒序 → 时间倒序
- 同义词: 已并入表
- 高亮: 故障关键字 + 机台号
- 0 结果: "猜你想找"用最近 7 天活跃机台

## 6. IA lint 规则
- 新增"机台型号"分组 ≥ 3 机台才允许创建
- 月度评审: 流量 < 0.1% 节点隐藏
- 删除必发 301 重定向

---

## 出口判据跑分 (逐条对 D2 出口判据)

| # | D2 出口判据 | 本示例 | 备注 |
|---|---|---|---|
| 1 | 内容模型每个字段有"为什么必须有"答案 | ✅ | 机台 4 字段均有理由 |
| 2 | 分类法每个维度标注互斥/非互斥 | ✅ | severity/machine status 互斥, store 非互斥 |
| 3 | L0 ≤ 7, L1 ≤ 9 | ✅ | L0 = 7, L1 ≤ 6 |
| 4 | 每个核心对象 ≥ 3 个入口 | ✅ | 机台 3 / 故障 3 |
| 5 | 同义词表覆盖搜索热词 | ✅ | 机台 / 故障 2 组 |
| 6 | 检索 5 要素全部命中 | ✅ | 索引/排序/同义/高亮/0 结果 |
| 7 | IA 演化 lint 规则可运行 | ✅ | 3 条规则已写明 |
| 8 | 标签系统无词义冲突 | ✅ | 每组仅 1 个规范词, 无词义冲突 |

## 已知缺口
- 真人搜索流量数据未采 → 上线 2 周后回填
