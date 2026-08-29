//! 配置层 — 统一从 env 读全部超限中间件配置
//!
//! 约定（per 任务 §1 + `.env.example` §8/§9）：
//! - 客服邮箱默认 `hanakagumi@gmail.com`，可由 `SUPPORT_EMAIL` 覆盖
//! - SMTP 缺密码时整体告警**不抛错**，仅落 `tracing::warn!`
//! - 软阈值 = `NATS_OVERFLOW_SOFT_RATIO`（默认 0.8），硬上限 = `<DOMAIN>_MAX_INFLIGHT`（0 = 不启用）
//! - 队列后端 = NATS JetStream，复用 `shared_platform::messaging::build_messaging_client`
//!
//! 错误模型：`ConfigError` 仅在**结构性错误**（env 解析失败、软阈值越界）时报；
//! "缺密码" / "硬上限 0" 不是错误，是"降级到日志" / "关闭限流"的合法配置。

use crate::domain::Domain;
use std::collections::HashMap;
use std::env;
use std::time::Duration;
use thiserror::Error;

/// 默认客服邮箱（per 2026-08-27 Ulysses 拍板）
pub const DEFAULT_SUPPORT_EMAIL: &str = "hanakagumi@gmail.com";

/// 默认 NATS 软阈值比例
pub const DEFAULT_SOFT_RATIO: f64 = 0.8;

/// 默认 NATS 队列最大 pending 消息数
pub const DEFAULT_MAX_PENDING: u64 = 10_000;

/// 默认告警去重窗口（秒）
pub const DEFAULT_DEDUP_WINDOW_SECS: u64 = 60;

/// 默认 SMTP 超时（毫秒）
pub const DEFAULT_SMTP_TIMEOUT_MS: u64 = 3_000;

/// 配置解析错误（**仅**结构性错误；缺密码 / 硬上限 0 不在此列）
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("invalid soft ratio {0}: must be in (0,1]")]
    InvalidSoftRatio(f64),

    #[error("invalid SMTP timeout {0} ms: must be > 0")]
    InvalidSmtpTimeout(u64),

    #[error("invalid max pending {0}: must be > 0")]
    InvalidMaxPending(u64),
}

/// 整个中间件的运行配置
///
/// 一次性 `from_env()` 解析，构造时把"业务无意义"的退化路径都落成"合理默认"，
/// 业务服务 main.rs 不需要任何条件判断就能直接 `OverflowConfig::from_env()`。
#[derive(Debug, Clone)]
pub struct OverflowConfig {
    /// 客服邮箱（告警收件人）
    pub support_email: String,
    /// NATS URI（k8s 内 = `nats://nats.<ns>.svc.cluster.local:4222`；dev = `nats://127.0.0.1:14222`）
    /// 优先读 `NATS_URL_IN_CLUSTER`（沿用 shared-platform 既有约定 per RGS-DTL-100 §5 + DEC-011），
    /// fallback 到 `NATS_URL_LOCAL`（端口转发），再 fallback 到 `localhost:4222`
    pub nats_uri: String,
    /// 软阈值比例（在 (0,1] 内；超此 → 入队）
    pub soft_ratio: f64,
    /// JS 流名（per 任务 §1：覆盖 4 域 subject filter = `rgs.*.overflow.v1`）
    pub stream_name: String,
    /// 消费者组
    pub consumer_group: String,
    /// 队列最大 pending 消息数（超此返回 QueueFull）
    pub max_pending: u64,
    /// 告警去重窗口（秒；同 (domain, kind) 窗口内只发 1 次）
    pub dedup_window: Duration,
    /// 5 域每域硬上限（0 = 关闭该域限流）
    pub per_domain: HashMap<Domain, u32>,
    /// SMTP 配置块（密码为空 → 切到 LogOnlySink）
    pub smtp: SmtpConfig,
}

/// SMTP 配置
#[derive(Debug, Clone)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    /// **空 = 缺密码 → LogOnlySink 降级**
    pub password: String,
    pub from_name: String,
    pub timeout: Duration,
}

impl SmtpConfig {
    /// `SMTP_PASSWORD` 为空 / 未设置 → true（用 LogOnlySink 替代）
    pub fn password_is_empty(&self) -> bool {
        self.password.trim().is_empty()
    }
}

impl OverflowConfig {
    /// 从 `std::env` 解析全部配置（dev/test 友好：所有字段都允许"未设置"走默认）
    pub fn from_env() -> Result<Self, ConfigError> {
        let support_email =
            env::var("SUPPORT_EMAIL").unwrap_or_else(|_| DEFAULT_SUPPORT_EMAIL.to_string());
        let nats_uri = env::var("NATS_URL_IN_CLUSTER")
            .or_else(|_| env::var("NATS_URL_LOCAL"))
            .unwrap_or_else(|_| "nats://localhost:4222".to_string());
        let soft_ratio = env::var("NATS_OVERFLOW_SOFT_RATIO")
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(DEFAULT_SOFT_RATIO);
        if !(soft_ratio > 0.0 && soft_ratio <= 1.0) {
            return Err(ConfigError::InvalidSoftRatio(soft_ratio));
        }
        let stream_name =
            env::var("NATS_OVERFLOW_STREAM").unwrap_or_else(|_| "RGS_OVERFLOW".to_string());
        let consumer_group = env::var("NATS_OVERFLOW_CONSUMER_GROUP")
            .unwrap_or_else(|_| "rgs-overflow-workers".to_string());
        let max_pending = env::var("NATS_OVERFLOW_MAX_PENDING")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(DEFAULT_MAX_PENDING);
        if max_pending == 0 {
            return Err(ConfigError::InvalidMaxPending(max_pending));
        }
        let dedup_window = env::var("ALERT_DEDUP_WINDOW_SECS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(DEFAULT_DEDUP_WINDOW_SECS);

        // 5 域每域独立硬上限（0 = 不启用）
        let mut per_domain = HashMap::new();
        for d in Domain::ALL {
            let key = d.env_max_inflight();
            let v = env::var(key)
                .ok()
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(0);
            per_domain.insert(d, v);
        }

        let smtp = SmtpConfig {
            host: env::var("SMTP_HOST").unwrap_or_else(|_| "smtp.gmail.com".to_string()),
            port: env::var("SMTP_PORT")
                .ok()
                .and_then(|v| v.parse::<u16>().ok())
                .unwrap_or(587),
            user: env::var("SMTP_USER").unwrap_or_else(|_| DEFAULT_SUPPORT_EMAIL.to_string()),
            password: env::var("SMTP_PASSWORD").unwrap_or_default(),
            from_name: env::var("SMTP_FROM_NAME").unwrap_or_else(|_| "RGS-Ops-Alert".to_string()),
            timeout: Duration::from_millis(
                env::var("SMTP_TIMEOUT_MS")
                    .ok()
                    .and_then(|v| v.parse::<u64>().ok())
                    .filter(|&n| n > 0)
                    .unwrap_or(DEFAULT_SMTP_TIMEOUT_MS),
            ),
        };

        if smtp.timeout.is_zero() {
            return Err(ConfigError::InvalidSmtpTimeout(0));
        }

        Ok(Self {
            support_email,
            nats_uri,
            soft_ratio,
            stream_name,
            consumer_group,
            max_pending,
            dedup_window: Duration::from_secs(dedup_window),
            per_domain,
            smtp,
        })
    }

    /// 给定域的硬上限（per_domain 未配置 = 0 = 不启用）
    pub fn hard_cap(&self, d: Domain) -> u32 {
        self.per_domain.get(&d).copied().unwrap_or(0)
    }

    /// 给定域的软阈值（hard_cap × soft_ratio，向上取整为 u32；hard_cap=0 → 0 = 不启用）
    pub fn soft_cap(&self, d: Domain) -> u32 {
        let hard = self.hard_cap(d);
        if hard == 0 {
            return 0;
        }
        let s = (f64::from(hard) * self.soft_ratio).ceil();
        // u32::MAX 防御：理论上 hard=4_294_967_295 时可能溢出；RGS 业务 hard 远小于此
        s.min(f64::from(u32::MAX)) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{clear_all_overflow_env, lock_env, set_envs};

    #[test]
    fn defaults_when_env_unset() {
        let _g = lock_env();
        clear_all_overflow_env();
        let cfg = OverflowConfig::from_env().expect("config");
        assert_eq!(cfg.support_email, DEFAULT_SUPPORT_EMAIL);
        assert!((cfg.soft_ratio - DEFAULT_SOFT_RATIO).abs() < f64::EPSILON);
        assert_eq!(cfg.stream_name, "RGS_OVERFLOW");
        assert_eq!(cfg.consumer_group, "rgs-overflow-workers");
        assert_eq!(cfg.max_pending, DEFAULT_MAX_PENDING);
        assert_eq!(
            cfg.dedup_window,
            Duration::from_secs(DEFAULT_DEDUP_WINDOW_SECS)
        );
        assert_eq!(cfg.smtp.host, "smtp.gmail.com");
        assert_eq!(cfg.smtp.port, 587);
        assert!(cfg.smtp.password_is_empty());
        for d in Domain::ALL {
            assert_eq!(cfg.hard_cap(d), 0, "{} hard_cap", d);
            assert_eq!(cfg.soft_cap(d), 0, "{} soft_cap", d);
        }
    }

    #[test]
    fn rejects_invalid_soft_ratio() {
        let _g = lock_env();
        clear_all_overflow_env();
        set_envs(&[("NATS_OVERFLOW_SOFT_RATIO", "1.5")]);
        let err = OverflowConfig::from_env().unwrap_err();
        assert!(matches!(err, ConfigError::InvalidSoftRatio(1.5)));
        clear_all_overflow_env();
    }

    #[test]
    fn soft_cap_scales_with_hard_cap() {
        let _g = lock_env();
        clear_all_overflow_env();
        set_envs(&[
            ("PLAYER_MAX_INFLIGHT", "100"),
            ("NATS_OVERFLOW_SOFT_RATIO", "0.8"),
        ]);
        let cfg = OverflowConfig::from_env().expect("config");
        assert_eq!(cfg.hard_cap(Domain::Player), 100);
        assert_eq!(cfg.soft_cap(Domain::Player), 80);
        clear_all_overflow_env();
    }

    #[test]
    fn hard_cap_zero_disables_limiter_for_domain() {
        let _g = lock_env();
        clear_all_overflow_env();
        set_envs(&[("PLAYER_MAX_INFLIGHT", "0")]);
        let cfg = OverflowConfig::from_env().expect("config");
        assert_eq!(cfg.hard_cap(Domain::Player), 0);
        assert_eq!(cfg.soft_cap(Domain::Player), 0);
        clear_all_overflow_env();
    }

    #[test]
    fn smtp_password_empty_marks_log_only() {
        let _g = lock_env();
        clear_all_overflow_env();
        set_envs(&[("SMTP_PASSWORD", "")]);
        let cfg = OverflowConfig::from_env().expect("config");
        assert!(cfg.smtp.password_is_empty());
        clear_all_overflow_env();
    }
}
