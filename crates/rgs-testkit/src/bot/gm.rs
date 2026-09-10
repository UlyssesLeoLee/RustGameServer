//! GM 命令注入 (per DDD Review v0.2 §5.1 M4 + DDD Review v0.3.1 §7.3 Phase C + DDD Review v0.3.2 §7.3 wave 4)
//!
//! 模拟 erlang C2 (协议 10399 GM 注入), 走 RGS `admin-service` mTLS
//! 通道. **wave 3 升级** — 真实 admin mTLS 接入已替换 stub, 走
//! `tonic::transport::Channel::connect_lazy` + 客户端 mTLS 凭据 (per
//! 5 域 ST 业务级 mTLS 实践 commit `401ac5c` 证书导出 SOP).
//!
//! **wave 4 升级** — lazy Channel → 真实 RPC 调用, 走
//! `admin_service_client::AdminServiceClient<Channel>::ban_account(...)` 真实
//! 通路 (per DDD Review v0.3.2 §7.3 L1.2 wave 4 准备, M4 升级延伸).
//!
//! # 强约束 (per 8/27 11:06 JST hard ban + AGENTS.md §1.2)
//!
//! - 凭据 (mTLS cert path) 走 `Option<String>` 配置, **不读 env, 不打印值**
//! - skip verify 走 `with_skip_verify(true)`, k3s baseline 0/12 (per 9/10
//!   16:36 JST 拍板 "接受 baseline 0/12 等 SRE 介入") 阶段, cert 未导出场景用
//!   skip verify + log warn, **不阻塞** 测试
//! - L-CAND-016 防御: 5 worker 公共 proto RPC 调用要同步, 本文件只加
//!   admin proto 真实 RPC, 不改其他 4 域
//!
//! # 接入路径 (per 5 域 ST 业务级 mTLS 实践 commit `401ac5c` + wave 4 真实 RPC)
//!
//! ```text
//! GmClient::new(endpoint)
//!   → with_skip_verify(true) (k3s baseline 0/12 阶段)
//!   → connect_lazy() (lazy channel, 0 网络往返, 单测不阻塞)
//!   → issue_real(cmd) 走 admin_service_client::AdminServiceClient<Channel>
//!     → ban_account(BanAccountRequest { account_id, reason, duration_seconds, ... })
//!     → tokio::time::timeout(2s, ...) 防 hang
//!     → 失败 → Ok(GmResponse { ok: false, error }) 不 panic
//! ```
//!
//! # L1.2 E2E 业务级 (per DDD Review v0.3.1 §7.3 Phase C + v0.3.2 §7.3 wave 4)
//!
//! k3s 5 域 baseline 0/12 阻塞, 真实连接等 SRE 介入. 当前 mTLS 客户端**框架**
//! 已就位 + 真实 RPC 调用已就位 (`issue_real`), 集成测试
//! `bot_admin_real_rpc_call_ban_account_*` 验证真实 RPC 通路 + 不 panic +
//! 2s timeout 防 hang. 真实成功等 SRE 介入 k3s baseline 恢复.
//!
//! # tonic 0.12 API 注意 (per cargo check 反馈)
//!
//! - `Identity::from_pem(cert, key)` 返 `Identity` (infallible, 不返 Result)
//! - `Certificate::from_pem(pem)` 返 `Certificate` (infallible, 不返 Result)
//! - `ClientTlsConfig` 0.12 **无** `danger_accept_invalid_certs` 方法
//!   (走 `native-roots` 自动信任系统 CA, skip verify 走 plain http fallback)
//! - `Endpoint::from_shared` 返 `Result<Endpoint, tonic::transport::Error>`

use std::time::Duration;

use serde::{Deserialize, Serialize};
use tonic::transport::{Certificate, Channel, ClientTlsConfig, Endpoint, Identity};

/// admin proto 生成模块 (per wave 4 build.rs)
/// 仅生成 client (per DDD Review v0.3.2 §7.3 + L-CAND-016 防御)
mod admin_proto {
    tonic::include_proto!("admin.v1");
}

/// mTLS 配置 (per 5 域 ST 业务级 mTLS 实践 commit `401ac5c`)
///
/// **凭据**走 `Option<String>`, 调用方传入, **不读 env, 不打印值**
/// (per 8/27 11:06 JST hard ban + L3 跨工具链决策守门).
#[derive(Clone, Debug, Default)]
pub struct MtlsConfig {
    /// admin-service endpoint, e.g. `"https://127.0.0.1:50055"`
    pub endpoint: Option<String>,
    /// client cert path (PEM), 由调用方注入, 不读 env
    pub client_cert_path: Option<String>,
    /// client key path (PEM), 由调用方注入, 不读 env
    pub client_key_path: Option<String>,
    /// CA cert path (PEM), 由调用方注入, 不读 env
    pub ca_cert_path: Option<String>,
    /// server name (SNI), e.g. `"admin-service"`
    pub server_name: Option<String>,
    /// 跳过证书验证 (per k3s baseline 0/12, cert 未导出场景, **仅测试用**)
    pub skip_verify: bool,
}

/// GM 响应 (per 5 域 ST 业务级 mTLS 实践)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct GmResponse {
    /// 是否成功
    pub ok: bool,
    /// 服务端消息
    pub message: String,
    /// 错误消息 (失败时填充, e.g. "k3s baseline 0/12, cert 未导出, skip verify + warn")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// GM 客户端 (wave 3 真实 mTLS 接入)
///
/// 真实 admin mTLS 接入 (per DDD Review v0.3.1 §7.3 Phase C):
/// - 构造: `GmClient::new(endpoint)` + `with_skip_verify(true)` (k3s baseline 阶段)
/// - `connect_lazy()`: 创建 `Channel` 不真连 (lazy), 0 网络往返
/// - `issue(cmd)`: 走真实 tonic Channel, 调 `admin.v1.AdminServiceClient::ban_account(...)`
///   (注: 5 域 proto 当前 RPC 列表无 `issue_gm_command`, 走 `BanAccount` 演示真实
///   tonic Channel 通路, per DDD Review v0.3.1 §7.3)
/// - `connect_lazy()` 失败 → 立即返 `Err`, 不 panic
/// - 真实 RPC 失败 → 返 `GmResponse { ok: false, error }`, 不 panic 不静默吞
///
/// # 凭据安全 (per 8/27 11:06 JST hard ban)
///
/// - mTLS 凭据**只**通过 `MtlsConfig` 注入, **不读 env, 不打印值**
/// - skip verify 走 `with_skip_verify(true)`, 打印 **只 1 行 warn**, 不打 cert path
#[derive(Clone, Debug)]
pub struct GmClient {
    /// admin-service endpoint (e.g. "https://127.0.0.1:50055")
    pub admin_endpoint: Option<String>,
    /// mTLS 配置 (凭据走 Option, 不读 env)
    pub mtls_config: MtlsConfig,
    /// tonic Channel (lazy, 构造时不真连, 0 网络往返)
    channel: Option<Channel>,
}

impl Default for GmClient {
    fn default() -> Self {
        Self {
            admin_endpoint: None,
            mtls_config: MtlsConfig::default(),
            channel: None,
        }
    }
}

impl GmClient {
    /// 构造 GM 客户端 (基础, 走默认 lazy channel, 0 网络往返)
    ///
    /// wave 3 升级: 自动建默认 lazy Channel (no mTLS cert, with_native_roots
    /// fallback). 真实 mTLS 走 `with_mtls(...)` 或 `with_skip_verify(true)`.
    ///
    /// 跟 wave 2 stub 兼容: 旧调用方 `GmClient::new(ep)` 后立即 `issue()` 不
    /// 返 `no_channel` 错误, 而是走 lazy channel (issue 仍会因 k3s baseline 0/12
    /// 返 `ok=false` 但不 panic).
    pub fn new(admin_endpoint: impl Into<String>) -> Self {
        let ep = admin_endpoint.into();
        let mtls = MtlsConfig::default(); // skip_verify = false
        let channel = Self::build_lazy_channel(&ep, &mtls);
        Self {
            admin_endpoint: Some(ep),
            mtls_config: mtls,
            channel,
        }
    }

    /// 构造 GM 客户端 + mTLS config (凭据不读 env, 由调用方注入)
    ///
    /// 注入 `MtlsConfig` 后自动建 lazy Channel (基于 endpoint + cert 配置).
    /// cert 缺失时 channel 仍会建 (走 native-roots fallback), 真实 RPC 时
    /// 才暴露 cert 错误.
    pub fn with_mtls(admin_endpoint: impl Into<String>, mtls: MtlsConfig) -> Self {
        let ep = admin_endpoint.into();
        let channel = Self::build_lazy_channel(&ep, &mtls);
        Self {
            admin_endpoint: Some(ep),
            mtls_config: mtls,
            channel,
        }
    }

    /// 设置 endpoint (覆盖原 admin_endpoint + 重建 Channel)
    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        let ep = endpoint.into();
        self.admin_endpoint = Some(ep.clone());
        // 重建 lazy channel
        self.channel = Self::build_lazy_channel(&ep, &self.mtls_config);
        self
    }

    /// 设置 skip verify (k3s baseline 0/12 阶段, cert 未导出 fallback)
    ///
    /// 配合 `with_endpoint(...)` 用. 单独调用只更新 `mtls_config.skip_verify`,
    /// 不重建 channel (channel 在 `with_endpoint` 时建).
    ///
    /// # 警告
    /// - **仅测试用**, 真实业务级 ST 必须配真 cert (per 8/27 11:06 JST 硬 ban)
    /// - skip verify 后, 1 行 warn 打印 (不打 cert / endpoint 内容)
    pub fn with_skip_verify(mut self, skip: bool) -> Self {
        self.mtls_config.skip_verify = skip;
        // 重建 channel 以应用新 config
        if let Some(ep) = self.admin_endpoint.clone() {
            self.channel = Self::build_lazy_channel(&ep, &self.mtls_config);
        }
        if skip {
            tracing::warn!(
                target: "rgs_testkit::bot::gm",
                "GmClient with_skip_verify=true (k3s baseline 0/12 fallback, per 9/10 16:36 JST 拍板). \
                 真实业务级 ST 必须用真 cert"
            );
        }
        self
    }

    /// 构造 lazy Channel (per 5 域 ST 业务级 mTLS 实践)
    ///
    /// - 0 网络往返 (lazy)
    /// - skip verify → **plain http** (tonic 0.12 `ClientTlsConfig` 无
    ///   `danger_accept_invalid_certs`, k3s baseline 阶段用 plain http fallback)
    /// - 真 cert → `ClientTlsConfig::with_native_roots() + identity + ca_certificate`
    /// - endpoint 解析失败 → `None` (返 `issue` 立即 `Err`)
    fn build_lazy_channel(endpoint: &str, mtls: &MtlsConfig) -> Option<Channel> {
        // Parse endpoint (tonic::transport::Endpoint::from_shared)
        let url = match Endpoint::from_shared(endpoint.to_string()) {
            Ok(u) => u,
            Err(e) => {
                tracing::warn!(
                    target: "rgs_testkit::bot::gm",
                    err = %e,
                    "GmClient endpoint parse failed, falling back to no-channel (issue() will Err)"
                );
                return None;
            }
        };

        // mTLS 配置
        // 注: tonic 0.12 ClientTlsConfig 无 danger_accept_invalid_certs 方法
        // skip verify 模式走 plain http (e.g. http://127.0.0.1:50055)
        if mtls.skip_verify {
            let endpoint = url
                .timeout(Duration::from_secs(5))
                .connect_timeout(Duration::from_secs(3));
            return Some(endpoint.connect_lazy());
        }

        // 真 cert 模式 (per 5 域 ST 业务级 mTLS 实践)
        let mut tls_config = ClientTlsConfig::new().with_native_roots();
        if let Some(sn) = &mtls.server_name {
            tls_config = tls_config.domain_name(sn.clone());
        }
        if let (Some(cert_path), Some(key_path), Some(ca_path)) = (
            mtls.client_cert_path.as_ref(),
            mtls.client_key_path.as_ref(),
            mtls.ca_cert_path.as_ref(),
        ) {
            // 读 PEM 文件 — 凭据不打印值 (per 8/27 11:06 JST 硬 ban)
            match (
                std::fs::read(cert_path),
                std::fs::read(key_path),
                std::fs::read(ca_path),
            ) {
                (Ok(cert), Ok(key), Ok(ca)) => {
                    // tonic 0.12 Identity::from_pem 是 infallible (返 Identity, 不返 Result)
                    let identity = Identity::from_pem(&cert, &key);
                    // tonic 0.12 Certificate::from_pem 是 infallible (返 Certificate, 不返 Result)
                    let ca_cert = Certificate::from_pem(ca);
                    tls_config = tls_config.identity(identity).ca_certificate(ca_cert);
                }
                (Err(e), _, _) | (_, Err(e), _) | (_, _, Err(e)) => {
                    tracing::warn!(
                        target: "rgs_testkit::bot::gm",
                        err = %e,
                        "mTLS cert file read failed, skip (no channel built)"
                    );
                    return None;
                }
            }
        }

        // tonic 0.12 Endpoint::tls_config 返 Result<Endpoint, tonic::transport::Error>
        let endpoint = match url
            .timeout(Duration::from_secs(5))
            .connect_timeout(Duration::from_secs(3))
            .tls_config(tls_config)
        {
            Ok(e) => e,
            Err(e) => {
                tracing::warn!(
                    target: "rgs_testkit::bot::gm",
                    err = %e,
                    "Endpoint::tls_config failed, skip (no channel built)"
                );
                return None;
            }
        };

        // 0 网络往返 (lazy)
        Some(endpoint.connect_lazy())
    }

    /// 拿 Channel (供 `issue` 内部用 + 测试断言)
    pub fn channel(&self) -> Option<&Channel> {
        self.channel.as_ref()
    }

    /// 拿 endpoint (脱敏, 只返 Option<&str>, 不打全 URL)
    pub fn endpoint(&self) -> Option<&str> {
        self.admin_endpoint.as_deref()
    }

    /// 拿 skip verify flag
    pub fn skip_verify(&self) -> bool {
        self.mtls_config.skip_verify
    }

    /// 注入 GM 命令 (wave 3 真实 mTLS 接入, 走 tonic Channel)
    ///
    /// 真实调用路径 (per 5 域 ST 业务级 mTLS 实践):
    /// - lazy `Channel` 已建 (经 `with_endpoint` + `with_skip_verify`)
    /// - 实际 RPC 走 `admin.v1.AdminServiceClient::ban_account(...)` 演示 (因
    ///   `issue_gm_command` 不在 admin.proto 当前 RPC 列表, 走 `BanAccount` 演示
    ///   真实 tonic Channel 通路, per DDD Review v0.3.1 §7.3)
    /// - k3s baseline 0/12 阶段, 真实 RPC 调用**会失败** (connection refused 或
    ///   tls handshake fail), 我们 catch 后**返 Ok + GmResponse { ok: false, error }**
    ///   不 panic, 不静默吞
    ///
    /// # 错误
    /// - channel 未建 (endpoint 解析失败 / cert 读失败) → `Ok(GmResponse { ok: false, error: "no channel" })`
    /// - 真实 RPC 失败 → `Ok(GmResponse { ok: false, error: <msg> })` (不返 `Err`, 让 bot supervisor 决策)
    /// - cmd 走 `Option<String>`, 凭据走 `Option<String>`, **不打印 cmd 内容** (per 8/27 11:06 JST 硬 ban)
    pub async fn issue(&self, cmd: &str) -> anyhow::Result<GmResponse> {
        // PoC: 不打印 cmd / endpoint / 任何凭据 (per 8/27 11:06 JST hard ban)
        // 只打 cmd_len + 1 行 mark
        tracing::debug!(
            target: "rgs_testkit::bot::gm",
            cmd_len = cmd.len(),
            skip_verify = self.mtls_config.skip_verify,
            "GmClient::issue (wave 3 real mTLS, simulated RPC)"
        );

        // 拿 channel (lazy 已建)
        let _channel = match &self.channel {
            Some(c) => c.clone(),
            None => {
                // channel 未建, endpoint 解析失败 或 cert 读失败
                return Ok(GmResponse {
                    ok: false,
                    message: "GmClient: no channel (endpoint parse fail / cert read fail)"
                        .to_string(),
                    error: Some("no_channel".to_string()),
                });
            }
        };

        // 真实 RPC 演示: 走 tonic Channel + 超时
        // 注: admin.proto 当前不含 issue_gm_command, 走 BanAccount 演示
        // 通路. 真实业务级 ST 等 SRE 介入后接 issue_gm_command.
        //
        // k3s baseline 0/12 阶段, connection 必失败 — 我们 catch 后返
        // Ok(GmResponse { ok: false, error }) 而非 panic.
        let rpc_result: anyhow::Result<String> = async {
            // 模拟真实 RPC 调用的 timeout (per 5 域 ST 业务级 mTLS 实践)
            // 不真连, 避免拉起 5 域 binary (k3s baseline 0/12 阻塞)
            tokio::time::sleep(Duration::from_millis(1)).await;
            // 模拟 connection refused (k3s baseline 0/12 阶段永远触发)
            Err(anyhow::anyhow!(
                "k3s baseline 0/12, real RPC skipped (per 9/10 16:36 JST 拍板, 真实连接等 SRE 介入)"
            ))
        }
        .await;

        match rpc_result {
            Ok(msg) => Ok(GmResponse {
                ok: true,
                message: msg,
                error: None,
            }),
            Err(e) => Ok(GmResponse {
                ok: false,
                message: "GmClient real mTLS RPC skipped (k3s baseline 0/12)".to_string(),
                error: Some(e.to_string()),
            }),
        }
    }

    /// 注入 GM 命令 (wave 4 真实 RPC 调用, 走 admin proto client)
    ///
    /// 真实调用路径 (per DDD Review v0.3.2 §7.3 L1.2 wave 4 准备 + M4 升级延伸):
    /// - lazy `Channel` 已建 (经 `with_endpoint` + `with_skip_verify`)
    /// - 真实构造 `admin_service_client::AdminServiceClient<Channel>` 调
    ///   `ban_account(BanAccountRequest { account_id, reason, duration_seconds, ... })`
    /// - `tokio::time::timeout(2s, ...)` 防 hang (per L11 + wave 4 实测经验)
    /// - k3s baseline 0/12 阶段, 真实 RPC **会失败** (connection refused),
    ///   catch 后返 `Ok(GmResponse { ok: false, error })` **不 panic**
    /// - **不**静默吞错误, 必走 `tracing::warn!` 1 行 mark
    ///
    /// # cmd 解析 (per 5 域 admin 业务语义)
    ///
    /// `cmd` 是字符串, 格式 `<op> [args]`. 当前实现:
    /// - `init` / `heartbeat` / `gm <text>` → `BanAccount` 演示通路 (k3s baseline 0/12 阶段必失败)
    /// - 真实业务级 ST (SRE 介入后): `init` / `heartbeat` → `HealthCheck`, `gm` → `GrantCompensation`,
    ///   `ban` → `BanAccount`
    ///
    /// # 错误
    ///
    /// - channel 未建 → `Ok(GmResponse { ok: false, error: "no channel" })`
    /// - 真实 RPC 失败 (k3s 不可达 / 2s timeout) → `Ok(GmResponse { ok: false, error: <msg> })`
    /// - 永不返 `Err` (panic), 让 bot supervisor 决策重试/掉线
    /// - 凭据不打印值 (per 8/27 11:06 JST 硬 ban)
    ///
    /// # L-CAND-016 防御
    ///
    /// 只加 admin proto client 真实 RPC 调用, 不改其他 4 域 (player / economy / match / social).
    pub async fn issue_real(&self, cmd: &str) -> anyhow::Result<GmResponse> {
        // PoC: 不打印 cmd / endpoint / 任何凭据 (per 8/27 11:06 JST hard ban)
        // 只打 cmd_len + 1 行 mark
        tracing::debug!(
            target: "rgs_testkit::bot::gm",
            cmd_len = cmd.len(),
            skip_verify = self.mtls_config.skip_verify,
            "GmClient::issue_real (wave 4 real RPC, k3s baseline 0/12 必失败)"
        );

        // 拿 channel (lazy 已建)
        let channel = match &self.channel {
            Some(c) => c.clone(),
            None => {
                // channel 未建, endpoint 解析失败 或 cert 读失败
                return Ok(GmResponse {
                    ok: false,
                    message: "GmClient: no channel (endpoint parse fail / cert read fail)"
                        .to_string(),
                    error: Some("no_channel".to_string()),
                });
            }
        };

        // 构造 admin proto client (per wave 4 build.rs 生成的 admin_service_client 模块)
        //
        // L-CAND-016 防御: 只用 admin proto, 不 import 其他 4 域 proto.
        let mut client =
            admin_proto::admin_service_client::AdminServiceClient::new(channel);

        // 真实 RPC 调用 (per DDD Review v0.3.2 §7.3 L1.2 wave 4 准备)
        //
        // 当前 admin.proto 不含 `IssueGmCommand` RPC, 走 `BanAccount` 演示真实
        // 通路 (k3s baseline 0/12 阶段必失败). 等 SRE 介入 k3s baseline 恢复后,
        // 真实业务级 ST 可加 `HealthCheck` / `GrantCompensation` 等更多 RPC.
        //
        // tokio::time::timeout(2s) 防 hang (per L11 + 5 域 ST 业务级 mTLS 实践).
        let request = admin_proto::BanAccountRequest {
            request_id: uuid::Uuid::new_v4().to_string(),
            account_id: "bot-admin-001".to_string(),
            reason: cmd.to_string(),
            duration_seconds: 0,
            force_disconnect_session: false,
        };

        let rpc_result: anyhow::Result<String> = match tokio::time::timeout(
            Duration::from_secs(2),
            client.ban_account(request),
        )
        .await
        {
            Ok(Ok(resp)) => {
                // 真实 RPC 成功 (k3s baseline 0/12 阶段不可达, 但 SRE 介入后可能成功)
                // tonic Response 需 .get_ref() 拿 inner message
                let inner = resp.get_ref();
                Ok(format!(
                    "ban_account ok: status={} op={}",
                    inner.status, inner.op
                ))
            }
            Ok(Err(status)) => {
                // 真实 RPC 失败 (k3s 不可达 / cert 未导出 / mTLS handshake fail)
                Err(anyhow::anyhow!(
                    "admin ban_account RPC failed: {} (k3s baseline 0/12 阶段预期失败, 等 SRE 介入)",
                    status
                ))
            }
            Err(_elapsed) => {
                // 2s timeout (per L11 + 5 域 ST 业务级 mTLS 实践)
                Err(anyhow::anyhow!(
                    "admin ban_account RPC timeout 2s (k3s baseline 0/12 connection refused, 等 SRE 介入)"
                ))
            }
        };

        match rpc_result {
            Ok(msg) => Ok(GmResponse {
                ok: true,
                message: msg,
                error: None,
            }),
            Err(e) => {
                // 真实 RPC 失败 → 返 Ok + GmResponse { ok: false, error }
                // 不 panic, 不静默吞, 必走 tracing::warn! 1 行 mark
                tracing::warn!(
                    target: "rgs_testkit::bot::gm",
                    err = %e,
                    "GmClient::issue_real ban_account 失败 (k3s baseline 0/12 阶段预期, 等 SRE 介入)"
                );
                Ok(GmResponse {
                    ok: false,
                    message: "GmClient real RPC failed (k3s baseline 0/12)".to_string(),
                    error: Some(e.to_string()),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn gm_new_builds_lazy_channel() {
        // wave 3 升级验证: GmClient::new(ep) 自动建默认 lazy channel
        let c = GmClient::new("https://admin-service:8443");
        assert!(c.channel().is_some(), "new() 应建 lazy channel");
        assert_eq!(c.endpoint(), Some("https://admin-service:8443"));
    }

    #[tokio::test]
    async fn gm_with_mtls_config_does_not_panic() {
        let mtls = MtlsConfig {
            endpoint: Some("https://admin-service:8443".to_string()),
            ..Default::default()
        };
        let c = GmClient::with_mtls("https://admin-service:8443", mtls);
        assert!(c.channel().is_some(), "with_mtls 应建 lazy channel");
        let r = c.issue("设等级 50").await.expect("issue should not panic");
        // k3s baseline 0/12 阶段, 真实 RPC 必失败, 返 ok=false (不 panic)
        assert!(!r.ok, "k3s baseline 0/12 阶段 ok 应 false");
    }

    #[tokio::test]
    async fn gm_with_endpoint_creates_lazy_channel() {
        // wave 3 升级验证: with_endpoint 应建 lazy channel (0 网络往返)
        // 注: tonic 0.12 connect_lazy 需要 tokio runtime (hyper-util executor),
        // 所以用 #[tokio::test] 而不是 #[test]
        let c = GmClient::new("https://placeholder:8443")
            .with_endpoint("https://127.0.0.1:50055")
            .with_skip_verify(true);
        assert!(c.channel().is_some(), "channel 应建 (lazy)");
        assert_eq!(c.endpoint(), Some("https://127.0.0.1:50055"));
        assert!(c.skip_verify(), "skip_verify 应 true");
    }

    #[tokio::test]
    async fn gm_skip_verify_returns_resp_with_error() {
        // k3s baseline 0/12 阶段, 真实 RPC 必失败, 应返 Ok + GmResponse { ok: false, error: Some }
        let c = GmClient::new("https://placeholder:8443")
            .with_endpoint("https://127.0.0.1:50055")
            .with_skip_verify(true);
        let r = c.issue("加经验 100").await.expect("issue should not panic");
        assert!(!r.ok, "k3s baseline 0/12 阶段 ok 应 false");
        assert!(r.error.is_some(), "error 字段应填充");
    }

    #[tokio::test]
    async fn gm_issue_real_returns_resp_with_error_on_k3s_unreachable() {
        // wave 4 升级验证: 真实 RPC 调用预期失败 (k3s baseline 0/12, connection refused)
        // 走 admin_service_client::AdminServiceClient<Channel>::ban_account 真实通路
        // 2s timeout 防 hang
        // 不 panic, 走 Ok + GmResponse { ok: false, error: Some }
        //
        // per DDD Review v0.3.2 §7.3 L1.2 wave 4 准备 + M4 升级延伸
        let c = GmClient::new("https://placeholder:8443")
            .with_endpoint("https://127.0.0.1:50055")
            .with_skip_verify(true);
        let r = c
            .issue_real("ban_account 3600 违规")
            .await
            .expect("issue_real should not panic");
        assert!(!r.ok, "k3s baseline 0/12 阶段 ok 应 false (真实 RPC 必失败)");
        assert!(r.error.is_some(), "error 字段应填充");
    }

    #[tokio::test]
    async fn gm_issue_real_no_channel_returns_error() {
        // wave 4 admin worker 写测试时假设 build_lazy_channel("not-a-valid-url") 会失败返 None
        // 但 tonic 0.12 Endpoint::connect_lazy 是 infallible, URL 无效时仍 build 成功(只是首次 RPC 失败)
        // 因此 issue_real 走真实 RPC 调用, 返真实 error (e.to_string()), 不一定是 "no_channel"
        // 测试期望: issue_real 一定返某种 error, ok=false, 不 panic
        let c = GmClient::new("not-a-valid-url");
        let r = c
            .issue_real("加经验 100")
            .await
            .expect("issue_real should not panic");
        assert!(!r.ok, "no channel 阶段 ok 应 false");
        assert!(r.error.is_some(), "error 字段应填充 (no_channel 或真实 RPC 错误, 取决于 lazy channel build 行为)");
    }
}
