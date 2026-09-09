# DEPRECATED — Empty directory, 21 RPC stub mock

**2026-09-09 15:10 JST Mavis 标记**: 此目录是 mock (per 2026-09-09 12:35 JST commit d99f76b 自标 deprecated), 21 RPC stub 是假数据, 跟 Ulysses 9/9 11:31 JST "前端表现和erlang版本一致的情况下，后端换成rgs" 战略冲突.

**正确路径**: rgs-shim-rust v0.3.1 (真 RGS 5 域 gRPC), 字节级 zsyz_server 协议兼容. 见 `tools/rgs-shim-rust/docs/ERLANG_TO_RGS_MIGRATION.md`.

**清理**: 原始文件已删除 (build.rs + Cargo.toml + 21 RPC stub), 目录保留空. 待 root 用户用 `Remove-Item` 删目录.
