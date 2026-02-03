# Known Concerns

> **Purpose**: Document technical debt, known risks, bugs, fragile areas, and improvement opportunities.
> **Generated**: 2026-02-03
> **Last Updated**: 2026-02-03

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

## Security Improvements (Phase 3)

| ID | Feature | Implementation | Status | Location |
|----|---------|-----------------|--------|----------|
| FR-019 | Never log private key value | Private key seed never converted to string for logging | Implemented | `monitor/src/crypto.rs` - no logging of sensitive values |
| FR-020 | Memory zeroing for key material | Zeroize crate wipes intermediate buffers after SigningKey construction | Implemented | `monitor/src/crypto.rs:114,157,169,221,233,287` |
| FR-021 | Standard Base64 RFC 4648 | All key encoding uses standard (not URL-safe) base64 | Implemented | `monitor/src/crypto.rs:152,216` uses `BASE64_STANDARD` |
| FR-022 | Validate key material is exactly 32 bytes | Strict validation on load/decode, clear error messages | Implemented | `monitor/src/crypto.rs:155-162,219-226,276-283` |

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

## Monitoring Gaps

| Area | Missing | Impact | Notes |
|------|---------|--------|-------|
| Private key source tracking | No metrics on whether file vs env var is used | Can't audit key source at runtime | KeySource enum added in Phase 3; logging at startup recommended |
| Rate limiter state | No metrics on bucket count or token refill rates | Hard to debug rate limiting issues | Consider adding Prometheus metrics |
| Authentication success rate | No metrics on auth successes vs failures | Can't detect brute force attempts | Would require counter instrumentation |
| WebSocket health | No metrics on connection duration or message throughput | Hard to diagnose subscription issues | Consider adding connection metrics |

## Improvement Opportunities

| Area | Current State | Desired State | Benefit |
|------|---------------|---------------|---------|
| Token management | Static string in environment | JWT or token service with expiration | Better security and client management |
| Configuration validation | Happens at startup only | Happens at startup with detailed validation | Catch misconfigurations earlier |
| Error logging | Mix of debug/warn levels | Structured error types with contextual data | Better observability |
| Rate limiting | Per-source only | Support per-IP and per-endpoint limits | Finer-grained DoS protection |
| Event filtering | Client-side by query params | Server-side filtering with ACLs | Reduced bandwidth, better security |
| Private key rotation | Manual process | Automated rotation with versioning | Reduced risk of key compromise |

## Security Debt Items

| ID | Area | Description | Mitigation Strategy | Status |
|----|------|-------------|-------------|--------|
| DEBT-001 | WebSocket token | Same token for all clients across all time | Implement token rotation every N days or on deployment | Open |
| DEBT-002 | Key registration | Public keys hardcoded in environment variable | Implement key management API with dynamic updates | Open |
| DEBT-003 | Audit trail | Minimal logging of auth events | Add structured audit logging to database/file | Open |
| DEBT-004 | Client identity | WebSocket clients are anonymous beyond token | Add optional client ID/name for audit purposes | Open |

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
