### 本批次做了什么

- **7 个 lane 并行**覆盖根 workspace 全部 31 个 crate，按 crate 分片、每 lane 一个
  受管 worktree（`scripts/worktree.ps1`）+ 一个隔离 `CARGO_TARGET_DIR`
  （`E:\DevCache\cargo\rgs-reg-<lane>`）+ 一个子代理。
- **归因口径**：每条失败必须由 log 原文证据支撑；无法归因的记 `code_defect` /
  `unclassified`，**不扫进环境类**（缺标比错标安全）。本批次 `code_defect` 0 条。

### 发现的 3 个既有缺陷（均已修复并复验）

1. **`network-gateway` 编译失败**（阻断整个 31 crate 的 CI 门禁）
   `crates/network-gateway/src/nif_demo.rs` 的 `rustler::init!(b"rgs_nif\0", ...)`
   用了字节串字面量。根因从 vendored 源码确认：`rustler_codegen-0.36.2/src/init.rs`
   L96 `name: concat!(#name, "\0")`，`concat!` 不接受字节串。
   该缺陷在 dev 上已存在，被更早一步的 `cargo fmt --check` 失败掩盖 —— CI 一直
   红在 fmt 步骤，从未走到暴露它的 clippy/test 步骤。

2. **`battle-service` 4 个 `w41_*` 用例必然失败**（测试自身缺陷，非产品缺陷）
   用例丢弃 `battle_init` 返回的随机 `battle_id`（`service.rs` L1840
   `Uuid::new_v4()`）却硬编码 `"x"` 调 `battle_start`（L1887 按 id 精确查表
   → `BattleNotFound`）。同文件通过的 `battle_start_advances_turn` 写法正确，
   4 处已对齐。

3. **`rgs-testkit` 掉线自愈路径形同虚设**（真实产品缺陷）
   `stop_all()` 用 `self.bots.drain(..)` 抽空 registry，而 `relogin_offline()`
   要按 id 从同一个 registry 找回 bot 重新 `start()` —— 两者互相矛盾，
   `recovered` 恒为 0。git 实证该缺陷自 `16bfb951` 引入，从未绿过。
   同一 lane 还暴露 `issue()` 的契约漂移（`9788404f` 改成恒返 `ok:false` 后
   测试断言未同步），已改为守 fail-closed 契约 + 补一个以缺口命名的
   `#[ignore]` 用例，集群恢复后解除 ignore 即可变红提醒能力闭环。

### 过程记录（供后续复跑参考）

- **子代理传输层不稳定**：8 次子代理派工中 6 次会话中断
  （`net::ERR_CONNECTION_CLOSED` / `net::ERR_CONNECTION_RESET`），与任务内容无关。
  降为 2 并发 + 主会话兜底重跑后全部完成。
- **并行链接会撞 `LNK1104`**：同一 target dir 下 `-Jobs 4` 并行链接 test binary 时
  出现 `LINK : fatal error LNK1104: 无法打开文件 ...exe`（link.exe 输出句柄/内存
  竞争）。串行 `-Jobs 1` 重试即全部链接成功 —— 属环境资源问题，不是代码缺陷。
- **RAM 极度紧张**（整机空闲 1.8–3.8 GB）：`cargo` 默认 `-j 24` 会把 rustc 挤爆，
  表现为 `can't find crate for std` 这类**假失败**。每 lane 限流 `-Jobs 3~5`、
  最多 2–3 lane 并发、stagger 启动才跑得动。
- **`Cargo.lock` 被 `.gitignore` 忽略且未追踪**：新 worktree 必须先播种，
  否则 `--locked` 直接报 `cannot create the lock file`。
- **cargo 指纹与 `-p` 集合绑定**：`-p` 集合不同 → feature unification 不同 →
  依赖全量重编。复用暖缓存必须保持相同 `-p` 集合与相同源路径。
