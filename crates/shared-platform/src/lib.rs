//! shared-platform —— 公共服务库
//!
//! 域职责：
//!   - tracing 初始化（OpenTelemetry OTLP 导出）
//!   - config 加载（基于 envy / figment + .env 读取 per RGS-SEC-100 §7）
//!   - error 类型（thiserror + 各域 error 集中转换）
//!   - mTLS 工具（rustls + rcgen 证书生成 per WF-1-53.11）
//!   - sqlx 公共 connection pool helper
//! 规范：RGS-SPEC-CROSS-001~007 横向规范
//!        RGS-IMPL-001 §3 / RGS-IMPL-003 §3 工具链
//!
//! 不持有 DB（per ARC-008 5 独立 DB 原则）。
//!
//! 53.2 占位。
