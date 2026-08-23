//! In-memory Function Registry (per RGS-INC-001 v0.2 §15, simplified).
//!
//! This module provides the abstract [`FunctionRegistry`] trait and a
//! [`InMemoryRegistry`] implementation that mirrors the PG-backed
//! `cluster_ops_db.function_registry` table (per §15.2 schema):
//!
//! - keyed by `(function_id, version)` — uniqueness enforced
//! - `status` field drives invokability (Gateway rejects non-Active)
//! - `get(_, version=None)` returns the latest `Active` version
//!
//! "Latest" is computed by a SemVer-ish string compare that does **not** require
//! the `semver` crate: we split on `.`, drop the leading `v` if present, parse
//! each part as `u32`, and compare as a tuple. This correctly orders
//! `"v0.2.0"` < `"v0.10.0"` — a frequent SemVer-trap.
#![allow(missing_docs)]

use crate::contract::{FunctionMetadata, FunctionStatus};
use crate::error::{FunctionPlaneError, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Abstract registry contract. PG-backed implementation is Phase 1 work.
#[async_trait]
pub trait FunctionRegistry: Send + Sync {
    /// Insert or replace a `(function_id, version)` record.
    async fn register(&self, meta: FunctionMetadata) -> Result<()>;

    /// Look up a function. When `version` is `None`, returns the latest `Active`
    /// version of the function (or [`FunctionPlaneError::NotFound`] if no
    /// `Active` version exists).
    async fn get(&self, function_id: &str, version: Option<&str>) -> Result<FunctionMetadata>;

    /// List **all** versions of a function id (any status).
    async fn list_versions(&self, function_id: &str) -> Result<Vec<FunctionMetadata>>;

    /// Transition a version to the given status. Returns
    /// [`FunctionPlaneError::VersionNotFound`] when the row does not exist.
    async fn set_status(
        &self,
        function_id: &str,
        version: &str,
        status: FunctionStatus,
    ) -> Result<()>;

    /// List every record currently in `Active` state (used by Gateway warm-up).
    async fn list_active(&self) -> Result<Vec<FunctionMetadata>>;
}

/// Thread-safe in-memory implementation backed by a `HashMap`.
#[derive(Debug, Default, Clone)]
pub struct InMemoryRegistry {
    inner: Arc<RwLock<HashMap<(String, String), FunctionMetadata>>>,
}

impl InMemoryRegistry {
    /// Build an empty in-memory registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of registered rows (for tests / metrics).
    pub async fn len(&self) -> usize {
        self.inner.read().await.len()
    }
}

#[async_trait]
impl FunctionRegistry for InMemoryRegistry {
    async fn register(&self, meta: FunctionMetadata) -> Result<()> {
        if meta.function_id.is_empty() {
            return Err(FunctionPlaneError::ContractInvalid(
                "function_id is empty".into(),
            ));
        }
        if meta.version.is_empty() {
            return Err(FunctionPlaneError::ContractInvalid(
                "version is empty".into(),
            ));
        }
        // Cheap contract check: a Wasm function must carry its bytes.
        if matches!(meta.runtime, crate::contract::Runtime::Wasm) && meta.wasm_bytes.is_none() {
            return Err(FunctionPlaneError::ContractInvalid(
                "wasm_bytes required for Wasm runtime".into(),
            ));
        }
        let key = (meta.function_id.clone(), meta.version.clone());
        let mut guard = self.inner.write().await;
        guard.insert(key, meta);
        Ok(())
    }

    async fn get(&self, function_id: &str, version: Option<&str>) -> Result<FunctionMetadata> {
        let guard = self.inner.read().await;
        match version {
            Some(v) => guard
                .get(&(function_id.to_string(), v.to_string()))
                .cloned()
                .ok_or_else(|| FunctionPlaneError::VersionNotFound {
                    function_id: function_id.to_string(),
                    version: v.to_string(),
                }),
            None => {
                // Find the latest Active version.
                let mut candidates: Vec<&FunctionMetadata> = guard
                    .iter()
                    .filter_map(|((fid, _), m)| {
                        if fid == function_id && m.status == FunctionStatus::Active {
                            Some(m)
                        } else {
                            None
                        }
                    })
                    .collect();
                if candidates.is_empty() {
                    return Err(FunctionPlaneError::NotFound(function_id.to_string()));
                }
                // Sort by parsed version tuple descending; pick first.
                candidates.sort_by(|a, b| compare_versions(&b.version, &a.version));
                Ok(candidates[0].clone())
            }
        }
    }

    async fn list_versions(&self, function_id: &str) -> Result<Vec<FunctionMetadata>> {
        let guard = self.inner.read().await;
        let mut out: Vec<FunctionMetadata> = guard
            .iter()
            .filter_map(|((fid, _), m)| {
                if fid == function_id {
                    Some(m.clone())
                } else {
                    None
                }
            })
            .collect();
        // Stable order: descending by version, then by created_at.
        out.sort_by(|a, b| {
            compare_versions(&b.version, &a.version).then(a.created_at.cmp(&b.created_at))
        });
        Ok(out)
    }

    async fn set_status(
        &self,
        function_id: &str,
        version: &str,
        status: FunctionStatus,
    ) -> Result<()> {
        let key = (function_id.to_string(), version.to_string());
        let mut guard = self.inner.write().await;
        let entry = guard.get_mut(&key).ok_or_else(|| FunctionPlaneError::VersionNotFound {
            function_id: function_id.to_string(),
            version: version.to_string(),
        })?;
        entry.status = status;
        entry.updated_at = chrono::Utc::now();
        Ok(())
    }

    async fn list_active(&self) -> Result<Vec<FunctionMetadata>> {
        let guard = self.inner.read().await;
        let mut out: Vec<FunctionMetadata> = guard
            .values()
            .filter(|m| m.status == FunctionStatus::Active)
            .cloned()
            .collect();
        out.sort_by(|a, b| {
            a.function_id
                .cmp(&b.function_id)
                .then_with(|| compare_versions(&b.version, &a.version))
        });
        Ok(out)
    }
}

/// Compare two version strings as SemVer-ish `u32` tuples.
///
/// - Strips a single leading `v` / `V` if present.
/// - Splits on `.` and parses each part as `u32`.
/// - Missing or non-numeric parts are treated as `0`.
/// - Shorter tuples pad with `0` on the right (`"1.2"` == `"1.2.0"`).
///
/// Returns `Ordering` suitable for `sort_by`.
fn compare_versions(a: &str, b: &str) -> std::cmp::Ordering {
    let mut a_t = parse_version_tuple(a);
    let mut b_t = parse_version_tuple(b);
    let max_len = a_t.len().max(b_t.len());
    a_t.resize(max_len, 0);
    b_t.resize(max_len, 0);
    a_t.cmp(&b_t)
}

fn parse_version_tuple(s: &str) -> Vec<u32> {
    let trimmed = s.strip_prefix(['v', 'V']).unwrap_or(s);
    trimmed
        .split('.')
        .map(|part| {
            // Strip pre-release / build metadata for the mock.
            let core = part.split('-').next().unwrap_or(part);
            let core = core.split('+').next().unwrap_or(core);
            core.parse::<u32>().unwrap_or(0)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compare_versions_orders_semver_correctly() {
        assert_eq!(compare_versions("v0.1.0", "v0.2.0"), std::cmp::Ordering::Less);
        assert_eq!(
            compare_versions("v0.10.0", "v0.2.0"),
            std::cmp::Ordering::Greater
        );
        assert_eq!(
            compare_versions("v1.0.0", "v0.99.99"),
            std::cmp::Ordering::Greater
        );
        assert_eq!(compare_versions("1.2", "1.2.0"), std::cmp::Ordering::Equal);
    }
}
