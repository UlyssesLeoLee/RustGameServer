本批次 lane 全部跑在 `4f773122`（批次启动时的 dev tip）之上，判定「lane 结果 == dev 代码结果」的依据是 git 实证而非推断：

1. **lane 分支不含代码文件**
   `git diff --stat 4f773122..agent/nim3/ulys-257` 只有 `reply.md`（+16 行），
   零代码文件 → 在 lane tip 跑回归 == 在 dev 代码基线跑回归，一次运行同时充当
   「合并候选验证」+「dev 代码基线」。

2. **批次内发现并修复 3 个既有缺陷，均已在其影响的 lane 复验**

   | 缺陷 | 根因 | 复验 lane | 修复前 → 修复后 |
   |---|---|---|---|
   | network-gateway 编译失败 | `rustler::init!` 误用字节串字面量（`rustler_codegen-0.36.2/src/init.rs` L96 `concat!(#name, "\0")`） | reg-admin | lane 从编译中止 → 475 passed / 3 failed |
   | battle-service 4 个 `w41_*` 用例失败 | 丢弃 `battle_init` 返回的随机 `battle_id`，硬编码 `"x"` | reg-match | 404/7 → 408/3 |
   | rgs-testkit 掉线自愈失效 + `issue()` 契约漂移 | `stop_all()` 用 `drain(..)` 抽空 registry，`relogin_offline()` 却要从中查找 | reg-tools | 386/6 → 388/4 |

   未受影响的 lane 不需要重跑：`stop_all` / `spawn_with_stagger` / `relogin_offline`
   在全仓**仅** rgs-testkit 自身 bot 模块使用（无外部调用方），
   `bot_smoke.rs` 是 rgs-testkit 自己的集成测试文件；network-gateway 与
   battle-service 各自只被 reg-admin / reg-match 覆盖。

3. **合并后代码状态不变**
   合并 commit `5e3c46ab` 只引入 `reply.md`（非代码）。
   `git diff --stat 4f773122..5e3c46ab -- crates/` 的结果恰好是上述 3 个修复
   涉及的 4 个文件，无其他 Rust 变动 → 本报告的 lane 结果仍然描述 dev 当前
   的 Rust 代码状态。

4. **合并完整性**
   `git diff dev..agent/nim3/ulys-257` 为空 → lane 分支已 100% 含入 dev。
   rebase 保留了那条空的 baseline 提交（`--keep-empty`，680d864e），
   内容差异仍只有 `reply.md`。
