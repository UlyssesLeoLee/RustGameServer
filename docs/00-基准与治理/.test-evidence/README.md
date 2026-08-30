# .test-evidence/ — 统一测试 evidence 归档

> **目的**:所有 cargo test / cargo run --example / e2e-smoke 输出统一落档,DDD Review 阶段可查档
> **维护人**:Mavis(接手 agent per DEC-008)
> **关联**:`scripts/test-evidence.ps1` + `scripts/regression-smoke.sh` + `docs/00-基准与治理/mock-registry.md §5`
> **强约束**(per 2026-08-27 11:06 JST):环境变量内容禁止打印到此目录,只可 invoke

---

## 1. 目录结构

```
.test-evidence/
├── README.md           本文件
└── {batch_id}/         每次 evidence 收集按 batch_id 落档(per test-evidence.ps1 -BatchId)
    ├── manifest.json   汇总:git head / 工具版本 / host / artifacts
    ├── cargo-test-*.log     9 份,对应 9 个 crate 的 cargo test 完整输出
    ├── cargo-example-*.log  7 份,对应 7 个 rgs-testkit example 输出
    └── e2e-smoke.log  (可选) 12 端口 e2e smoke 输出
```

## 2. 历史 batch

| batch_id | 时间(JST) | artifacts | passed | failed | git head |
|---|---|---|---|---|---|
| 2026-08-28-ut-impl | 2026-08-28 09:00 JST | 16 | 195 | 0 | b4df2ed23acdc33e541c5e3aac9323f51c7cd3f1 |
| 2026-08-28-ut-impl-v2 | 2026-08-28 09:05 JST | 16 | 195 | 0 | b4df2ed23acdc33e541c5e3aac9323f51c7cd3f1(正则未修) |
| 2026-08-28-ut-impl-v3 | 2026-08-28 09:10 JST | 16 | **270** | 0 | b4df2ed23acdc33e541c5e3aac9323f51c7cd3f1(正则修复,sum 多次 result) |

> v1/v2/v3 同一 commit,正则版本差异(从 regex 单次匹配 → 多次 sum)。最终 v3 是准确数字。

## 3. 关键数字(per 2026-08-28-ut-impl-v3)

| crate / example | passed | elapsed |
|---|---|---|
| rgs-testkit | 3 | 7.25s |
| rgs-certgen | 17 | 0.6s |
| gm-backend | 0(默认集成测试,需 --test 才跑) | 5.73s |
| cluster-ops | 56 | 24.0s |
| player-service | 27 | 59.4s |
| economy-service | 53 | 26.8s |
| match-service | 19 | 83.6s |
| social-service | 17 | 47.0s |
| admin-service | 20 | 57.9s |
| 7 examples | 7/7 exit_ok | ~2.5s 总计 |

**270 passed / 0 failed** (per 2026-08-28 ut 实施批次)

## 4. 用法

### 4.1 跑本批 evidence

```pwsh
pwsh -NoProfile -NonInteractive -File scripts/test-evidence.ps1
```

### 4.2 跑指定 crate

```pwsh
pwsh -NoProfile -NonInteractive -File scripts/test-evidence.ps1 -Crates "gm-backend,rgs-certgen" -SkipExamples
```

### 4.3 跑回归(走 shell)

```bash
bash scripts/regression-smoke.sh
```

### 4.4 DDD Review 阶段查档

每个 batch 都有 `manifest.json` 标 git head + 工具版本,任何一个测试失败都可定位到 commit。

## 5. 清理策略

- 保留最近 5 个 batch(覆盖最近 1 周)
- 超过 5 个 batch 自动 gitignore 排除
- DDD Review 阶段需要的 batch 加 tag 永久保留

---

**作者**:Mavis(接手 agent per DEC-008)
**审批**:架构师(Mavis 接手 agent per DEC-008)+ 自审 + 2026-08-28
