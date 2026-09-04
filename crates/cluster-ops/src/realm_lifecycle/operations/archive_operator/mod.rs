//! ArchiveOperator 子模块 (per P1-3 第二步, 2026-09-05)
//!
//! 4 个 helper file, 各 1 个 free function, 接收 &ArchiveOperator
//! 实现 4 个最大 method body (cold_archive / gdpr / query / saga).
//! pub API 不变, archive.rs impl ArchiveOperator 4 个 method 各 1 行委派到 self::xxx_impl.

pub mod cold_archive;
pub mod gdpr;
pub mod query;
pub mod saga;
