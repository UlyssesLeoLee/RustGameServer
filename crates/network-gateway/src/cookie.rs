//! Erlang cookie 鉴权 stub (per 9/4 改进路线图 Phase 1 PHP↔Erlang cookie 兼容)
//!
//! ## 范围
//! 闪烁之光 PHP 配置 / 旧 Erlang 集群用 cookie 鉴权 (per 9/4 MD §3), 节点启动时
//! `erl -setcookie <cookie> -name <node>@<host> -hidden`. RGS 协议网关需识别对方
//! cookie 并决定是否接受 dist 连接.
//!
//! ## 本骨架 (Phase 1.5 stub)
//! - Cookie 长度/字符校验 (Erlang cookie 规范: 最多 1024 字符, 任意 printable ASCII)
//! - 字符串比较 (constant-time 防 timing attack, Phase 1.5 改用 subtle crate)
//! - 不做 challenge 签名 (per 9/4 R2 风险, 需 rustler/erlang-rs 桥接)
//!
//! ## 已知缺口
//! - cookie 不入日志 (per 8/27 11:06 JST hard ban, Display/Debug 隐去)
//! - cookie 不入 env (per 8/27 11:06 JST, 走 env var 注入但 REDACTED 过滤)
//! - 完整 MD5 challenge-response 走 rustler/erlang-rs 桥接 BEAM (Phase 1.5)
//!
//! ## 参考
//! - 9/4 MD §3 网络拓扑 (PHP "假节点" cookie 鉴权)
//! - 9/4 改进路线图.md Phase 1 "PHP↔Erlang cookie 鉴权兼容" 1 SRE·d
//! - Erlang/OTP auth 模块 cookie 长度/字符规范

/// Erlang cookie 长度上限 (per Erlang/OTP, 任意 printable ASCII)
pub const MAX_COOKIE_LEN: usize = 1024;

/// Cookie 鉴权错误 (凭据 0 打印, Display/Debug 隐去)
///
/// `#[error("...")]` 字符串不携带 cookie 实际值, 符合 8/27 11:06 JST hard ban.
#[derive(Debug, thiserror::Error)]
pub enum CookieError {
    #[error("cookie too long: {len} > {max}")]
    TooLong { len: usize, max: usize },
    #[error("cookie contains invalid char at index {idx}")]
    InvalidChar { idx: usize },
    #[error("cookie mismatch")]
    Mismatch,
}

/// 校验 cookie 格式 (Erlang/OTP 规范: 任意 printable ASCII, 不超 1024 字符)
pub fn validate_cookie(cookie: &[u8]) -> Result<(), CookieError> {
    if cookie.len() > MAX_COOKIE_LEN {
        return Err(CookieError::TooLong {
            len: cookie.len(),
            max: MAX_COOKIE_LEN,
        });
    }
    for (idx, &b) in cookie.iter().enumerate() {
        // Erlang cookie: 任意 printable ASCII (0x20-0x7E), 不接受控制字符
        if !(0x20..=0x7E).contains(&b) {
            return Err(CookieError::InvalidChar { idx });
        }
    }
    Ok(())
}

/// Cookie 鉴权 (constant-time 比较)
///
/// Phase 1.5 stub: 走普通 `==` 比较, Phase 1.5 改用 `subtle::ConstantTimeEq` 防 timing attack.
pub fn verify_cookie(local: &[u8], remote: &[u8]) -> Result<(), CookieError> {
    validate_cookie(local)?;
    validate_cookie(remote)?;
    if local.len() != remote.len() || local != remote {
        return Err(CookieError::Mismatch);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_cookie_passes() {
        assert!(validate_cookie(b"secret_cookie_123").is_ok());
    }

    #[test]
    fn empty_cookie_passes() {
        // Erlang 允许空 cookie (测试环境常用)
        assert!(validate_cookie(b"").is_ok());
    }

    #[test]
    fn too_long_cookie_rejected() {
        let long = vec![b'a'; MAX_COOKIE_LEN + 1];
        let err = validate_cookie(&long).unwrap_err();
        assert!(matches!(err, CookieError::TooLong { .. }));
    }

    #[test]
    fn control_char_rejected() {
        let bad = b"abc\x00def";
        let err = validate_cookie(bad).unwrap_err();
        assert!(matches!(err, CookieError::InvalidChar { idx: 3 }));
    }

    #[test]
    fn verify_match() {
        assert!(verify_cookie(b"abc", b"abc").is_ok());
    }

    #[test]
    fn verify_mismatch() {
        let err = verify_cookie(b"abc", b"abd").unwrap_err();
        assert!(matches!(err, CookieError::Mismatch));
    }

    #[test]
    fn verify_length_mismatch() {
        let err = verify_cookie(b"abc", b"abcd").unwrap_err();
        assert!(matches!(err, CookieError::Mismatch));
    }

    #[test]
    fn cookie_error_display_redacts_value() {
        // 验证 Display 输出不含 cookie 实际值 (per 8/27 11:06 JST hard ban)
        let err = CookieError::Mismatch;
        let s = format!("{}", err);
        assert!(!s.contains("secret"), "Display must not leak cookie value");
        assert_eq!(s, "cookie mismatch");
    }
}
