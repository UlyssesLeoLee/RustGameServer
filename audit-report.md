## Dependency Security Audit Report (ULYS-249)

### Summary
Ran `cargo audit` and `npm audit` on the workspace. **27 vulnerabilities found in Rust dependencies** (2 critical, 1 high, 11 medium, 13 low), **0 in npm**. Many packages are significantly outdated.

---

### 🔴 Critical / High Severity Vulnerabilities

| Package | Current | Vulnerability | Severity | Fixed In |
|---------|---------|---------------|----------|----------|
| **wasmtime** | 19.0.2 | RUSTSEC-2026-0095: Sandbox-escaping memory access (Winch) | 9.0 Critical | ≥36.0.7, <37.0.0 OR ≥42.0.2, <43.0.0 OR ≥43.0.1 |
| **wasmtime** | 19.0.2 | RUSTSEC-2026-0096: Sandbox escape on aarch64 Cranelift | 9.0 Critical | Same as above |
| **wasmtime** | 19.0.2 | RUSTSEC-2026-0269: Filesystem sandbox escape (trailing slashes) | 8.8 High | ≥24.0.13, <25.0.0 OR ≥36.0.14, <37.0.0 OR ≥46.0.3, <47.0.0 OR ≥47.0.4 |
| **h2** | 0.3.27 | RUSTSEC-2026-0258: Unbounded empty DATA frames | — | ≥0.4.16 (latest 0.4.20) |
| **rustls-webpki** | 0.102.8 | RUSTSEC-2026-0049/0098/0099/0104: CRL/name constraints/panic issues | — | ≥0.103.13 (latest 0.104.0-alpha.7) |
| **rsa** | 0.9.10 | RUSTSEC-2023-0071: Marvin Attack timing sidechannel | 5.9 Medium | **No fix available** |
| **tokio-tar** | 0.3.1 | RUSTSEC-2025-0111: PAX header file smuggling | — | **No fix available** |

**wasmtime has 16 additional medium/low CVEs** (RUSTSEC-2024-0438 through RUSTSEC-2026-0316). Latest wasmtime is 50.0.0-rc.1 — **31 major versions behind**.

---

### 🟡 Medium / Low + Unmaintained

| Package | Current | Issue |
|---------|---------|-------|
| backoff | 0.4.0 | Unmaintained (RUSTSEC-2025-0012) |
| bincode | 1.3.3 | Unmaintained (RUSTSEC-2025-0141) |
| fxhash | 0.2.1 | Unmaintained (RUSTSEC-2025-0057) |
| instant | 0.1.13 | Unmaintained (RUSTSEC-2024-0384) |
| mach | 0.3.2 | Unmaintained (RUSTSEC-2020-0168) |
| paste | 1.0.15 | Unmaintained (RUSTSEC-2024-0436) |
| rustls-pemfile | 2.2.0 | Unmaintained (RUSTSEC-2025-0134) |
| wasmtime-jit-debug | 19.0.2 | Unsound (RUSTSEC-2024-0442) |

---

### 📦 Outdated Packages (>2 Major Versions Behind)

| Package | Current | Latest | Gap |
|---------|---------|--------|-----|
| **wasmtime** | 19.0.2 | 50.0.0-rc.1 | 31 major |
| **jsonwebtoken** | 9.3.1 | 11.1.0 | 2 major |
| **rand** | 0.8.8 | 0.10.3 | 2 major |
| **serial_test** | 0.5.1 | 4.0.1 | 3 major |
| **tonic** | 0.12.3 | 0.14.6 | 2 major |
| **tonic-build** | 0.12.3 | 0.14.6 | 2 major |
| **tonic-health** | 0.12.3 | 0.14.6 | 2 major |

Other notable gaps: async-nats (0.42→0.50), bcrypt (0.16→0.19), criterion (0.5→0.8), ctor (0.2→1.0), opentelemetry* (0.24→0.33), reqwest (0.12→0.13), sqlx (0.8→0.9), testcontainers (0.23→0.28), thiserror (1.0→2.0), tokio-tungstenite (0.24→0.30), tracing-opentelemetry (0.25→0.34), actix-web-lab (0.24→0.29).

---

### ✅ npm
- No vulnerabilities found
- `glob@13.0.6` is current

---

### 🎯 Recommended Action Items

1. **Immediate (Critical)**:
   - Upgrade `wasmtime` to ≥43.0.1 (or ≥36.0.7, <37.0.0) — fixes 18 CVEs including 2 critical sandbox escapes
   - Upgrade `h2` to ≥0.4.16
   - Upgrade `rustls-webpki` to ≥0.103.13 (or 0.104.0-alpha.7)

2. **High Priority**:
   - Evaluate `rsa` 0.9.10 → 0.10+ (breaking changes; Marvin Attack has no fix in 0.9)
   - Evaluate `tokio-tar` replacement (unmaintained, no fix for smuggling)

3. **Medium Priority**:
   - Migrate unmaintained crates: `backoff`→`backoff-rs`, `bincode`→`bincode2`/`postcard`, `fxhash`→`foldhash`/`ahash`, `instant`→`std::time`, `mach`→remove/replace, `paste`→`paste2`/macros, `rustls-pemfile`→`rustls-pemfile` 3.x (re-maintained)
   - Update `jsonwebtoken`, `rand`, `serial_test`, `tonic*` to latest major versions

4. **Low Priority**:
   - Batch-update remaining outdated deps (async-nats, bcrypt, criterion, ctor, opentelemetry*, reqwest, sqlx, testcontainers, thiserror, tokio-tungstenite, tracing-opentelemetry, actix-web-lab)

5. **Process**:
   - Add `cargo audit` to CI pipeline
   - Consider `cargo deny` (deny.toml exists) for license/duplicate policies
   - Schedule quarterly dependency reviews

---

### Next Steps
I can prepare PRs for the critical upgrades (wasmtime, h2, rustls-webpki) first. The wasmtime upgrade will be the largest change — it touches `function-plane` and may require WASM API updates. Want me to start with a wasmtime upgrade PR?