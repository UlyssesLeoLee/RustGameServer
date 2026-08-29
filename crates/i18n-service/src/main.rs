//! i18n-service 入口 (per RGS-DTL-038 §4.1 + DEC-038-05)
//!
//! ## 状态
//! 桶 14 部分进展,缺完整 service / 6 UT / 3 IT (推 W34+ 补完)
//! 当前 main.rs 仅占位,让 cargo build 通过

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("i18n-service starting (skeleton, per RGS-DTL-038 §4.1)");
    tracing::warn!("i18n-service 未完成,推 W34+ 补完 service / UT / IT");
}
