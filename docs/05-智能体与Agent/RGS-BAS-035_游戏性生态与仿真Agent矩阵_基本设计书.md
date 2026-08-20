# 基本设计书（基本設計書 / Basic Design Document）

**游戏性生态与仿真 Agent 矩阵 — Gameplay & Simulation Agent Matrix**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-BAS-035 |
| 版本 | 0.1 |
| 父文档 | RGS-REQ-035 需求定义书、RGS-BAS-033 Agent平台底座 |
| 制定日 | 2026-08-20 |
| 最终更新日 | 2026-08-20 |
| 制定者 | 架构师 |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |

---

## 1. Generative NPC 状态图与反思循环

```mermaid
graph TD
    Obs["外界感知 (玩家对话 / 场景事件)"] --> Mem["写入 Episodic Memory"]
    Mem --> Recall["相关性/重要性/新鲜度 混合检索"]
    Recall --> Reflect{"重要度积分 > 阈值?"}
    Reflect -- 是 --> FormInsight["提炼高阶认知 (Insight) & 修正世界观"]
    FormInsight --> Plan["生成未来行动规划 (Plan)"]
    Reflect -- 否 --> GenReply["生成即时对话与情绪反应"]
    Plan --> GenReply
    GenReply --> ActionOutput["输出对话与 NPC 动作 (只读/建议)"]
```
