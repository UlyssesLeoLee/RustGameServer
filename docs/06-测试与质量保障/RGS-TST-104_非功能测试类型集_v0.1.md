# RGS-TST-104 非功能测试类型集（Non-functional Test Types）

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-TST-104 |
| 版本 | 0.1（占位，per 150 工程审计缺失项补全）|
| 依据 | RGS-WF-001 v0.5 §2 150 工程 80 / 82 |
| 状态 | 🔴 **占位 NO-GO**（per RGS-PLAN-001 v0.8 §3.3 + RGS-EXEC-001 v0.3）|
| 责任人 | Ulysses（一人公司 12 角色兼任 per DEC-008）|
| 关联 150 工程 | 80 / 82（性能試験 / ストレス試験）|

---

## 1. 文档目的

本文件是 RGS-WF-001 v0.5 §2 150 工程审计中**缺失 RGS 引用**的 NFR（非功能测试类型集）类工程的占位文档。

**补全范围**：覆盖 150 工程中以下编号（per WF-001 §2）：

> 80 / 82：性能試験 / ストレス試験

---

## 2. 测试范围

**覆盖范围**：
- 工程 80：性能测试（latency / throughput / resource utilization）
- 工程 82：压力测试（极限负载 / 长时间稳定性）

**NFR 目标**（per RGS-REQ-001 §12）：
- p99 latency < 50ms（read）/ 100ms（write）
- 100k CCU 同在线
- 5xx 错误率 < 0.1%
- 资源利用率：CPU < 70%，Mem < 80%
- 7x24 持续运行（hypercare 阶段 ≥ 168h）

**工具链**：
- k6（HTTP/gRPC load gen）
- criterion.rs（micro-bench）
- tokio-console（async 调优）
- flamegraph（CPU profiling）
- valgrind / heaptrack（内存分析）

---

## 3. 文档结构（待 NO-GO 解除后填充）

```markdown
# §1 测试规约
# §2 测试环境
# §3 测试用例集
# §4 工具链与脚本
# §5 通过标准
# §6 缺陷管理流程
# §7 报告与签字
```

---

## 4. 责任人

| 角色 | 责任人 | 签字日 |
|---|---|---|
| NFR Lead | Ulysses（一人公司 12 角色兼任 per DEC-008）| 2026-08-21 |
| 评审 | Ulysses（架构师兼）| 2026-08-21 |

---

## 5. 激活条件

🔴 → 🟢：

1. **G-CODE-06** Rust 1.98 + cargo build + cargo test 实际跑过全绿
2. **G-CODE-03** 5 独立 DB 拓扑图实际画过
3. NO-GO 完全解除

满足后由 Ulysses 升 v0.2 填入实际测试用例与工具链配置。

---

## 6. 关联文档

- 上游：RGS-WF-001 v0.5 §2 150 工程 / RGS-PLAN-001 v0.8 §3 / RGS-EXEC-001 v0.3
- 同类：RGS-TST-101/102/103/104/105（5 个测试类型集）
- 并行：RGS-WBS-001 v0.3 瀑布式 WBS（每条 150 工程 → 1 个 L4 任务）
- worktree：每条 150 工程可单独 worktree 分支执行（per RGS-WT-001）
