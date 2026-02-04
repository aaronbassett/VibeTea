# Known Concerns

> **Purpose**: Document technical debt, known risks, bugs, fragile areas, and improvement opportunities.
> **Generated**: 2026-02-03
> **Last Updated**: 2026-02-04

## Security Concerns

### High Priority

| ID | Area | Description | Risk Level | Mitigation | Status |
|----|------|-------------|------------|------------|--------|
| SEC-001 | WebSocket authentication | Single static token for all clients allows no per-client revocation or auditing | High | Plan token rotation mechanism or client certificates | Open |
| SEC-002 | Token management | Subscriber token hardcoded in environment variable with no expiration or rotation | High | Implement periodic token rotation and audit logging | Open |
| SEC-003 | CORS policy | All origins allowed, no CORS validation in place | Medium | Add configurable CORS origin whitelist | Open |
| SEC-004 | Signature header validation | X-Signature header parsed as-is with minimal format validation | Low | Already mitigated by base64 decoding and cryptographic verification | Mitigated |

### Medium Priority

| ID | Area | Description | Risk Level | Mitigation | Status |
|----|------|-------------|------------|------------|---------|
| SEC-005 | Private key environment variable | `VIBETEA_PRIVATE_KEY` env var alternative now documented and implemented | Low | Documented in SECURITY.md; same validation as file-based keys | Resolved |
| SEC-006 | Rate limiting overhead | Per-source token bucket tracking could consume memory with many unique sources | Medium | Implement stale entry cleanup (partially done); add configurable limits | Open |
| SEC-007 | Event persistence | Events are in-memory only; no persistence means events lost on restart | Medium | Document as design decision; recommend replay mechanism at application level | Open |
| SEC-008 | Export-key stdout purity | Diagnostic/error messages must be stderr-only to enable safe piping | Low | Export-key explicitly prints errors to stderr; only key goes to stdout | Mitigated (Phase 4) |
| SEC-009 | GitHub Actions secret exposure | Private key accessible in GitHub Actions environment; potential exposure via leaked logs | Medium | Use GitHub secret masking; never log VIBETEA_PRIVATE_KEY; minimize output from monitor process | Mitigated (Phase 5) |
| SEC-010 | Composite action error handling | Action warns on network failure but continues workflow; potential silent monitoring failures | Medium | Document in README; monitor logs for warnings; consider explicit failure modes | Mitigated (Phase 6) |

## Security Improvements (Phase 3-6)

### Phase 3 Features

| ID | Feature | Implementation | Status | Location |
|----|---------|-----------------|--------|----------|
| FR-019 | Never log private key value | Private key seed never converted to string for logging | Implemented | `monitor/src/crypto.rs` - no logging of sensitive values |
| FR-020 | Memory zeroing for key material | Zeroize crate wipes intermediate buffers after SigningKey construction | Implemented | `monitor/src/crypto.rs:114,157,169,221,233,287` |
| FR-021 | Standard Base64 RFC 4648 | All key encoding uses standard (not URL-safe) base64 | Implemented | `monitor/src/crypto.rs:152,216` uses `BASE64_STANDARD` |
| FR-022 | Validate key material is exactly 32 bytes | Strict validation on load/decode, clear error messages | Implemented | `monitor/src/crypto.rs:155-162,219-226,276-283` |

### Phase 4 Features

| ID | Feature | Implementation | Status | Location |
|----|---------|-----------------|--------|----------|
| FR-003 | Export-key command | CLI subcommand outputs base64-encoded private key + single newline | Implemented | `monitor/src/main.rs:101-109, 181-202` |
| FR-023 | Stderr for diagnostics | All diagnostic/error messages go to stderr; stdout is key-only | Implemented | `monitor/src/main.rs:196-199` - errors print to eprintln! |
| FR-026 | Exit code semantics | 0 for success, 1 for configuration error (missing key) | Implemented | `monitor/src/main.rs:199` |
| FR-027 | Integration tests | Tests verify exported key roundtrips via `VIBETEA_PRIVATE_KEY` | Implemented | `monitor/tests/key_export_test.rs:148-221` |
| FR-028 | Signature consistency | Ed25519 deterministic; tests verify identical signatures after export-import | Implemented | `monitor/tests/key_export_test.rs:229-264` |

### Phase 5 Features

| ID | Feature | Implementation | Status | Location |
|----|---------|-----------------|--------|----------|
| FR-029 | GitHub Actions workflow example | Example CI workflow showing monitor integration with export-key setup | Implemented | `.github/workflows/ci-with-monitor.yml` |
| FR-030 | Dynamic source ID in Actions | Source ID includes repo and run ID for traceability | Implemented | `.github/workflows/ci-with-monitor.yml:39` |
| FR-031 | Graceful monitor shutdown | SIGTERM handler flushes buffered events before exit | Documented | `.github/workflows/ci-with-monitor.yml:105-113` |

### Phase 6 Features

| ID | Feature | Implementation | Status | Location |
|----|---------|-----------------|--------|----------|
| FR-032 | Composite GitHub Action | Reusable action wrapper for monitor binary download and startup | Implemented | `.github/actions/vibetea-monitor/action.yml` |
| FR-033 | Action inputs/outputs | Parameterized inputs (server-url, private-key, version) and outputs (monitor-pid, started) | Implemented | `.github/actions/vibetea-monitor/action.yml:24-55` |
| FR-034 | Action documentation | README updated with action usage, inputs, outputs, and examples | Implemented | `README.md:212-292` |
| FR-035 | Non-blocking action errors | Network/config failures log warnings but don't fail workflow | Implemented | `.github/actions/vibetea-monitor/action.yml:101-120` |
| FR-036 | Dynamic source ID interpolation | Action default source ID uses repo and run_id for uniqueness | Implemented | `.github/actions/vibetea-monitor/action.yml:96` |

## Technical Debt

### High Priority

| ID | Area | Description | Impact | Effort | Status |
|----|------|-------------|--------|--------|--------|
| TD-001 | Cleanup task | Rate limiter cleanup task in main.rs never terminates; cleanup_handle is dropped without cancellation | Cleanup runs until server shutdown | Low | Open |
| TD-002 | Error handling | Some auth errors (InvalidPublicKey) could reveal server configuration details in logs | Debugging difficulty | Low | Open |

### Medium Priority

| ID | Area | Description | Impact | Effort | Status |
|----|------|-------------|--------|--------|--------|
| TD-003 | Configuration validation | VIBETEA_PUBLIC_KEYS parsing doesn't validate that decoded base64 is exactly 32 bytes | Confusing error messages at runtime | Low | Open |
| TD-004 | Type safety | EventPayload uses untagged enum which could be fragile with certain JSON structures | API contract ambiguity | Medium | Open |
| TD-005 | Logging | Some debug/trace logs are verbose and could impact performance under load | Performance in high-traffic scenarios | Low | Open |
| TD-008 | Export-key path handling | Currently requires --path flag; no automatic .env file detection for fallback keys | Developer friction | Low | Open |
| TD-009 | Composite action cleanup | Post-job cleanup requires manual SIGTERM step; no automatic cleanup mechanism | Potential zombie processes | Medium | Open |

### Low Priority

| ID | Area | Description | Impact | Effort | Status |
|----|------|-------------|--------|--------|--------|
| TD-006 | Documentation | VIBETEA_PRIVATE_KEY environment variable now documented in SECURITY.md | Developer confusion | Low | Resolved |
| TD-007 | Error response codes | Health endpoint always returns 200 even during degradation; no status codes for partial failure | Monitoring complexity | Low | Open |

## Known Bugs

| ID | Description | Workaround | Severity | Status |
|----|-------------|------------|----------|--------|
| BUG-001 | EnvGuard in tests modifies global env var state; tests must use `#[serial]` to avoid race conditions | Use `#[serial]` decorator on all env-var-touching tests | Medium | Mitigated in code |
| BUG-002 | WebSocket client lagging causes skipped events (lagged count logged but events discarded) | No workaround; clients must reconnect to resume from current position | Medium | Documented in trace log |

## Fragile Areas

| Area | Why Fragile | Precautions |
|------|-------------|-------------|
| `server/src/auth.rs` | Critical security-sensitive code; base64/length validation is subtle | Extensive test coverage (43 tests); use RFC 8032 strict verification |
| `server/src/routes.rs` | High complexity with multiple auth paths and error cases | Test all auth combinations; validate error responses |
| `monitor/src/crypto.rs` | Cryptographic key handling; file permissions and memory management matter | Tests verify file permissions on Unix; tests verify zeroization; regenerate if compromised |
| `server/src/config.rs` | Configuration parsing with environment variables; tests required `#[serial]` | Never modify without running full test suite with `--test-threads=1` |
| `monitor/tests/env_key_test.rs` | Environment variable tests must serialize to avoid race conditions | All tests use `#[serial]` decorator (24 env-var-touching tests) |
| `monitor/tests/key_export_test.rs` | Export-key tests modify env vars and spawn subprocesses; must use `#[serial]` | All tests use `#[serial]` decorator (15 export-key tests) |
| `monitor/src/main.rs` | New export-key logic handles private key material and must not log it | Verify stdout purity in tests; all key writes are stderr only |
| `.github/workflows/ci-with-monitor.yml` | Workflow manages private key and process; signal handling is critical | Test with dry-run first; ensure SIGTERM properly terminates and flushes |
| `.github/actions/vibetea-monitor/action.yml` | Composite action manages binary download and monitor process lifecycle | Ensure secret masking works; test with actual GitHub Actions runner |

## Deprecated Code

| Area | Deprecation Reason | Removal Target | Replacement |
|------|-------------------|----------------|-------------|
| None identified | - | - | - |

## TODO Items

| Location | TODO | Priority | Status |
|----------|------|----------|--------|
| `monitor/tests/privacy_test.rs:319` | TODO regex in test assertion for security match | Medium | Open |

## Dependency Concerns

### At-Risk Dependencies

| Package | Concern | Action Needed | Status |
|---------|---------|---------------|--------|
| `ed25519_dalek` | Cryptographic library; monitor for security advisories | Subscribe to GitHub security alerts | Open |
| `tokio` | Runtime; heavy async dependency with many transitive deps | Keep updated; monitor for CVEs | Open |
| `base64` | Decoding; generally stable but validate error handling | No immediate action needed | Resolved |
| `zeroize` | New dependency for memory safety; critical for security | Monitor for updates and best practices | Open |

## Performance Concerns

| ID | Area | Description | Impact | Mitigation |
|----|------|-------------|--------|-----------|
| PERF-001 | Rate limiter memory | Hash map of source_id -> TokenBucket grows unbounded until cleanup | Memory leak over time | Cleanup task removes stale entries (60s timeout) |
| PERF-002 | Event broadcast | Broadcaster uses bounded channel; lagging subscribers lose events | Client experience degrades | Expected behavior; clients reconnect |
| PERF-003 | JSON serialization | Every event serialized per WebSocket subscriber | CPU under high load | No mitigation; consider compression |
| PERF-004 | GitHub Actions binary download | Release binary download on every workflow run | Network overhead | Consider caching binary or building from source |
| PERF-005 | Composite action overhead | Action adds step overhead for binary download and validation | Minimal workflow slowdown | Overhead is ~5-10 seconds per workflow; acceptable for CI |

## Monitoring Gaps

| Area | Missing | Impact | Notes |
|------|---------|--------|-------|
| Private key source tracking | No metrics on whether file vs env var is used | Can't audit key source at runtime | KeySource enum added in Phase 3; logging at startup recommended |
| Rate limiter state | No metrics on bucket count or token refill rates | Hard to debug rate limiting issues | Consider adding Prometheus metrics |
| Authentication success rate | No metrics on auth successes vs failures | Can't detect brute force attempts | Would require counter instrumentation |
| WebSocket health | No metrics on connection duration or message throughput | Hard to diagnose subscription issues | Consider adding connection metrics |
| Export-key usage | No audit trail of key exports | Can't track which systems have exported keys | Consider adding telemetry or structured logging |
| GitHub Actions monitor | No metrics on monitor process uptime/failures in CI | Can't detect if monitoring silently fails | Consider structured logging to Actions output |
| Composite action usage | No telemetry on adoption or failure rates | Can't track action usage patterns | Could add optional telemetry to action |

## Improvement Opportunities

| Area | Current State | Desired State | Benefit |
|------|---------------|---------------|---------|
| Token management | Static string in environment | JWT or token service with expiration | Better security and client management |
| Configuration validation | Happens at startup only | Happens at startup with detailed validation | Catch misconfigurations earlier |
| Error logging | Mix of debug/warn levels | Structured error types with contextual data | Better observability |
| Rate limiting | Per-source only | Support per-IP and per-endpoint limits | Finer-grained DoS protection |
| Event filtering | Client-side by query params | Server-side filtering with ACLs | Reduced bandwidth, better security |
| Private key rotation | Manual process | Automated rotation with versioning | Reduced risk of key compromise |
| Export-key defaults | Explicit --path flag required | Auto-discovery of ~/.vibetea or env var | Smoother UX for end users |
| GitHub Actions integration | Manual secret setup | Documentation or automated secret creation script | Easier onboarding for CI/CD |
| Composite action | Basic functionality | Advanced features (log output, retry logic) | Better debugging and resilience |

## Security Debt Items

| ID | Area | Description | Mitigation Strategy | Status |
|----|------|-------------|------------|--------|
| DEBT-001 | WebSocket token | Same token for all clients across all time | Implement token rotation every N days or on deployment | Open |
| DEBT-002 | Key registration | Public keys hardcoded in environment variable | Implement key management API with dynamic updates | Open |
| DEBT-003 | Audit trail | Minimal logging of auth events | Add structured audit logging to database/file | Open |
| DEBT-004 | Client identity | WebSocket clients are anonymous beyond token | Add optional client ID/name for audit purposes | Open |
| DEBT-005 | Key export audit | No record of when/where keys are exported | Implement export logging with timestamp/system info | Open |
| DEBT-006 | GitHub Actions secret usage | Monitor process has access to private key; potential logging risk | Implement log filtering to never output env vars | Phase 5 risk |
| DEBT-007 | Composite action versioning | Action pinned to @main; no semantic versioning | Implement version tags and GitHub releases | Phase 6 opportunity |

## Potential Attack Vectors

| Vector | Mitigation | Status |
|--------|-----------|--------|
| Signature bypass (wrong message signed) | Signature verifies full request body | Mitigated |
| Timing attack on token comparison | Constant-time comparison with `subtle` crate | Mitigated |
| Timing attack on key material buffers | Zeroize crate wipes intermediate buffers | Mitigated (Phase 3) |
| Rate limit bypass (multiple sources) | Per-source limiting doesn't prevent N sources | Partially mitigated by total capacity |
| Private key logging | Private key never converted to string for logging | Mitigated (Phase 3) |
| Invalid key length in env var | Strict validation: exactly 32 bytes required | Mitigated (Phase 3) |
| WebSocket replay (reuse old token) | Static token never expires | Not mitigated |
| Source ID spoofing | Must provide valid signature for registered source | Mitigated |
| Base64 decoding errors | Handled with explicit error cases | Mitigated |
| Whitespace in env var key | Trimmed before base64 decoding | Mitigated (Phase 3) |
| Export-key output leakage | Diagnostic messages sent to stderr only | Mitigated (Phase 4) |
| Key export in cleartext | Keys exported as base64; assumes secure transport (HTTPS/CI secrets) | Requires operator discipline |
| GitHub Actions log leakage | Private key is env var, subject to accidental logging | Partially mitigated by GitHub secret masking (Phase 5) |
| Composite action binary tampering | Binary downloaded from GitHub releases without signature verification | Partially mitigated by HTTPS; recommend checksum verification |
| Man-in-the-middle on binary download | Binary download from GitHub releases via HTTP curl | Mitigated by HTTPS (curl -fsSL) |

---

## Concern Severity Guide

| Level | Definition | Response Time |
|-------|------------|----------------|
| Critical | Production impact, security breach | Immediate |
| High | Degraded functionality, security risk | This sprint |
| Medium | Developer experience, minor issues | Next sprint |
| Low | Nice to have, cosmetic | Backlog |

---

*This document tracks what needs attention. Update when concerns are resolved or discovered.*
