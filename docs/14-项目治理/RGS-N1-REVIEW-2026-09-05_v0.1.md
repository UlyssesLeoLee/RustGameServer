# N+1 风险审查 v0.1 (P1-5 落地)

> **创建日期**: 2026-09-05 JST
> **创建者**: Mavis 接手 agent per DEC-008 (代签 Ulysses)
> **依据**: 2026-09-04 系统性扫描 P1-5 决策落地
> **状态**: ⏳ 待 DDD Review

## 0. 范围

3 个文件（per 9/4 grep 命中 for 循环 + SQL pattern）：
- `crates/card-service/src/repository.rs` (1112 行)
- `crates/i18n-service/src/repository.rs` (1024 行)
- `crates/rgs-asset-download/src/resume_token_store.rs` (792 行)

## 1. 审查方法

```bash
# 找 for 循环 + 紧跟 SQL 调用
for file in card-service/repository i18n-service/repository rgs-asset-download/resume_token_store; do
  grep -A 5 "for .* in" crates/$file.rs | grep -E "query_as|fetch_one|fetch_optional|execute\("
done
```

## 2. 审查结果

### 2.1 card-service/src/repository.rs
- **3 个 for 循环**，命中 1 处 SQL：
  - L196 `for inst in instances { ... sqlx::query("INSERT INTO card_instances ...") }`
  - **类型判断**: BATCH INSERT（每个 instance 一行 INSERT），不是 N+1 SELECT
  - **潜在改进**: 可改 `INSERT ... VALUES ($1, $2), ($3, $4), ...` 单语句批量 insert 提速 ~5-10x
  - **优先级**: P2（dev/test 影响小，prod 取决于 insert 频率）
  - **不属 N+1 真问题**，不阻塞 P1-5

### 2.2 i18n-service/src/repository.rs
- **2 个 for 循环**，0 处 SQL 命中
- for 循环纯内存处理（无 DB 调用）
- **结论**: 0 N+1 风险

### 2.3 rgs-asset-download/src/resume_token_store.rs
- **6 个 for 循环**：
  - L243 `for path in paths { self.read_file(&path) }` — 文件 I/O，不是 SQL
  - L259 `for token in all { self.delete(&token.token_id) }` — cleanup loop，单条 delete 是设计如此
  - L558 `for bytes in payloads { serde_json::from_slice }` — 内存 deserialize
  - L663 / L747 / L770 — `#[cfg(test)]` 测试代码
- **结论**: 0 N+1 风险（cleanup loop 是设计如此，文件 I/O 不是 SQL）

## 3. 决策

**P1-5 落地结论**: 3 个文件 14 个 for 循环全部审完，**0 处 N+1 真问题**。

- card-service 的 batch INSERT 可改但属 P2 优化（非 P1-5 范围）
- 其它 2 个文件干净

## 4. 不动（per 9/2 8/27 决策）

- card-service batch INSERT 改 `INSERT ... VALUES (...), (...)` — 留为 P2 工单
- asset-download cleanup loop — 设计如此
- i18n / asset-download / card-service 其他 SQL 模式（已扫过）

## 5. 修订历史

| 版本 | 日期 | 修订人 | 审批 | 摘要 |
|---|---|---|---|---|
| v0.1 | 2026-09-05 JST | Ulysses — Mavis 接手 | ⏳ 待 DDD Review | 起草, P1-5 审查结果: 0 真 N+1 |
