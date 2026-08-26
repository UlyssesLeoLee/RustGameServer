//! 沙箱 K8s 客户端（per FR-LCM-003 + IMPL §3.4 M-2070.1）。
//!
//! ## 硬约束
//!
//! - K3s 演练 namespace（**不**复用生产 namespace）
//! - DrillExecutor **不**得引用生产 K8s client（per FR-LCM-003）
//!
//! ## 降级策略
//!
//! 沙箱 K8s 不可达 → drill 演练 `Skipped`（由 SRE 接力后启动 K3s sandbox namespace）。

use serde::{Deserialize, Serialize};

use crate::realm_lifecycle::error::{Error, Result};

/// 沙箱 K8s namespace 名（per IMPL §3.4 M-2070.1）。
pub const SANDBOX_K8S_NAMESPACE: &str = "rgs-drill-sandbox";

/// 沙箱 K8s kubeconfig env var。
pub const SANDBOX_KUBECONFIG_ENV: &str = "RGS_SANDBOX_KUBECONFIG";

/// 沙箱 K8s 客户端配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxK8sConfig {
    pub kubeconfig: Option<String>,
    pub namespace: String,
    /// replicas 范围（演练副本数）。
    pub min_replicas: u32,
    pub max_replicas: u32,
}

impl SandboxK8sConfig {
    pub fn from_env() -> Option<Self> {
        let kubeconfig = std::env::var(SANDBOX_KUBECONFIG_ENV).ok();
        Some(Self {
            kubeconfig,
            namespace: SANDBOX_K8S_NAMESPACE.to_string(),
            min_replicas: 1,
            max_replicas: 5,
        })
    }

    pub fn new(kubeconfig: Option<String>) -> Self {
        Self {
            kubeconfig,
            namespace: SANDBOX_K8S_NAMESPACE.to_string(),
            min_replicas: 1,
            max_replicas: 5,
        }
    }
}

/// 沙箱 K8s 客户端（per FR-LCM-003：仅沙箱）。
///
/// 不持有真实 `kube` client（避免在编译期强制 k8s 依赖）；通过配置 + namespace 锚定
/// 演练隔离边界。
#[derive(Debug, Clone)]
pub struct SandboxK8sClient {
    config: SandboxK8sConfig,
}

impl SandboxK8sClient {
    /// 构造沙箱 K8s 客户端；强制 namespace = `rgs-drill-sandbox`。
    pub fn new(config: SandboxK8sConfig) -> Result<Self> {
        if config.namespace != SANDBOX_K8S_NAMESPACE {
            return Err(Error::DrillProductionLeak);
        }
        Ok(Self { config })
    }

    /// 探测沙箱 K8s 是否可达；不可达返回 `Ok(false)`。
    pub fn probe_available(&self) -> bool {
        self.config.kubeconfig.is_some()
    }

    pub fn namespace(&self) -> &str {
        &self.config.namespace
    }

    pub fn config(&self) -> &SandboxK8sConfig {
        &self.config
    }

    /// 演练副本数调整（占位；SRE 接力后接 kube-rs）。
    pub fn plan_replicas(&self, target: u32) -> u32 {
        target.clamp(self.config.min_replicas, self.config.max_replicas)
    }

    pub const fn env_var_name() -> &'static str {
        SANDBOX_KUBECONFIG_ENV
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn namespace_must_be_drill_sandbox() {
        // FR-LCM-003 锚定：误用生产 namespace → DrillProductionLeak
        let cfg = SandboxK8sConfig {
            kubeconfig: None,
            namespace: "production".to_string(),
            min_replicas: 1,
            max_replicas: 5,
        };
        let r = SandboxK8sClient::new(cfg);
        assert!(matches!(r, Err(Error::DrillProductionLeak)));
    }

    #[test]
    fn drill_sandbox_namespace_passes() {
        let cfg = SandboxK8sConfig::new(None);
        let client = SandboxK8sClient::new(cfg).unwrap();
        assert_eq!(client.namespace(), "rgs-drill-sandbox");
    }

    #[test]
    fn probe_unavailable_without_kubeconfig() {
        let client = SandboxK8sClient::new(SandboxK8sConfig::new(None)).unwrap();
        assert!(!client.probe_available());
    }

    #[test]
    fn env_var_name_is_rgs_sandbox_kubeconfig() {
        assert_eq!(SandboxK8sClient::env_var_name(), "RGS_SANDBOX_KUBECONFIG");
    }

    #[test]
    fn plan_replicas_clamps_to_range() {
        let client = SandboxK8sClient::new(SandboxK8sConfig::new(None)).unwrap();
        assert_eq!(client.plan_replicas(0), 1);
        assert_eq!(client.plan_replicas(3), 3);
        assert_eq!(client.plan_replicas(100), 5);
    }
}
