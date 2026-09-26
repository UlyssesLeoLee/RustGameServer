# DEPRECATED — 历史背景说明 (2026-09-09 旧 mock 已退役)

**历史**: 2026-09-09 15:10 JST Mavis 标记 — 旧 mock 项目 21 RPC stub 是假数据,
跟 Ulysses 9/9 11:31 JST "前端表现和erlang版本一致的情况下,后端换成rgs" 战略冲突,
已迁移至 rgs-shim-rust v0.3.1 (真 RGS 5 域 gRPC, 字节级 [游戏A]_server 协议兼容).

**当前状态 (重要)**:

`rgs-flash-mock` 目录**不再**是旧 21 RPC stub 假数据 mock. 自 v0.1 (commit c5c40062)
起,该目录已重定位为 RGS **verification harness / gap matrix + 7 域 mTLS 业务级 stub**,
服务于 RGS-TEST-DESIGN v0.2 §10 DoD 整体回归测试.

**与 rgs-shim-rust 的关系**:
- `rgs-shim-rust` (D:/RustGameServer/tools/rgs-shim-rust/) — 真 RGS 5 域 gRPC 字节级 [游戏A]_server 兼容
- `rgs-flash-mock` (D:/RustGameServer/tools/rgs-flash-mock/) — verification harness + 7 域 mTLS stub
  (与上游域服务 mTLS 业务级连通, 但 stub 不真打 5 域业务逻辑)

**目录保留原因 (per 9/4 17:47 JST Ulysses user 偏好)**:
> "测试脚本+数据归入 mock 项目以备回归测试"

9 个 regression-test-*.sh + ut.sh / it.sh / st.sh + 48 个 mock_data/*.json fixture
全部归入此目录,作为 ULYS-141 整体回归测试载体.

**替代路径参考**:
- 真 RGS 业务级: `rgs-shim-rust v0.3.1` (per `tools/rgs-shim-rust/docs/ERLANG_TO_RGS_MIGRATION.md`)
- 测试设计: `../../docs/06-测试与质量保障/RGS-TST-{UT,IT,ST}-06_*.md`
- 实施报告: `docs/V0.2-IMPLEMENTATION-REPORT.md` / `V0.3-IMPLEMENTATION-REPORT.md`

---

> **维护警告**: 不要根据本文件的"原始文件已删除"措辞误判此目录为空或可清理.
> 2026-09-09 后该目录已重建为 verification harness,实际承载 6 源文件 (2743 LOC)
> + 13 个测试脚本 + 48 个 fixture JSON + 11 个 docs/ 文档.
> 任何 "remove tools/rgs-flash-mock" 的提议应先核对 `README.md` 与
> `docs/V0.3-IMPLEMENTATION-REPORT.md` 是否仍在用.