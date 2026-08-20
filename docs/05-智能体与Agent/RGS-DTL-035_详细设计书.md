# 详细设计书（詳細設計書 / Detailed Design Document）

**游戏性生态与仿真 Agent 矩阵 — Gameplay & Simulation Agent Matrix**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DTL-035 |
| 版本 | 0.2 |
| 父文档 | RGS-BAS-035 基本设计书 |
| 制定日 | 2026-08-20 |
| 最终更新日 | 2026-08-20 |
| 制定者 | 架构师 |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |

---

## 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-20 | 架构师 | 初版制定。 |
| 0.2 | 2026-08-20 | 架构师 | 修复公式中的控制字符，恢复可渲染的 LaTeX 表达。 |

---

## 1. NPC 记忆检索评分与衰减函数定义

记忆检索得分由 **新鲜度 (Recency)**、**重要度 (Importance)**、**相关度 (Relevance)** 三者加权得到：

$$Score(m) = \alpha \cdot e^{-\lambda \Delta t} + \beta \cdot Importance(m) + \gamma \cdot CosineSimilarity(\vec{q}, \vec{m})$$

- $\Delta t$：距离记忆产生流逝的游戏内时间（小时）。
- $\lambda$：遗忘衰减系数（默认 $\lambda = 0.05$）。
- $\alpha, \beta, \gamma$：权重系数，满足 $\alpha + \beta + \gamma = 1.0$（默认 $0.3, 0.3, 0.4$）。
