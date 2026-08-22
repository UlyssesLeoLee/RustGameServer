//! rgs-testkit —— RustGameServer 测试套件骨架
//!
//! 三大子模块：
//! - `mock`    DB / gRPC / NATS JetStream mock 工具
//! - `helper`  config 加载 / tracing 初始化 / 公共 assert helper
//! - `fixture` sample data 工厂 + 6 DB init/teardown
//!
//! 5 域 + cluster-ops + shared-platform 测试统一引用本 crate。
//! 规范：RGS-IMPL-001 §3 + RGS-SPEC-000 §2.4
//!
//! 53.3 骨架：3 个子模块各 1-2 个最小可用 API，self-test 覆盖。

pub mod fixture;
pub mod helper;
pub mod mock;
