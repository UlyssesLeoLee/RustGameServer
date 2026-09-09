// RGS gRPC client via rgs-proxy 8084 (HTTP/JSON bridge)
// Production-grade: connection pool (reqwest), 5s timeout, structured errors

use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

#[derive(Clone)]
pub struct RgsClient {
    proxy: String,
    http: Client,
}

#[derive(Debug, Deserialize)]
pub struct RgsResponse {
    pub ok: bool,
    #[serde(default)]
    pub response: Option<serde_json::Value>,
    #[serde(default)]
    pub error: Option<String>,
    #[allow(dead_code)]
    #[serde(default)]
    pub code: Option<i32>,
}

impl RgsClient {
    pub fn new(proxy: impl Into<String>) -> Self {
        let http = Client::builder()
            .timeout(Duration::from_secs(5))
            .connect_timeout(Duration::from_secs(2))
            .pool_max_idle_per_host(32)
            .build()
            .expect("reqwest client");
        RgsClient { proxy: proxy.into(), http }
    }

    pub async fn call(&self, domain: &str, rpc: &str, body: serde_json::Value) -> RgsResponse {
        let url = format!("{}/{}/{}", self.proxy, domain, rpc);
        match self.http.post(&url).json(&body).send().await {
            Ok(resp) => match resp.json::<RgsResponse>().await {
                Ok(r) => r,
                Err(e) => RgsResponse { ok: false, response: None, error: Some(format!("PARSE_ERROR: {}", e)), code: None },
            },
            Err(e) => RgsResponse { ok: false, response: None, error: Some(format!("HTTP_ERROR: {}", e)), code: None },
        }
    }

    // High-level: 5 域 HealthCheck 并发
    pub async fn healthcheck_all(&self) -> Vec<(String, bool)> {
        let domains = ["player", "economy", "match", "social", "admin"];
        let mut futs = Vec::with_capacity(5);
        for d in domains {
            let client = self.clone();
            let d = d.to_string();
            futs.push(tokio::spawn(async move {
                let r = client.call(&d, "HealthCheck", serde_json::json!({})).await;
                (d, r.ok)
            }));
        }
        let mut out = Vec::with_capacity(5);
        for f in futs {
            if let Ok(r) = f.await { out.push(r); }
        }
        out
    }
}
