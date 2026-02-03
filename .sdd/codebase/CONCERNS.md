# Known Concerns

> **Purpose**: Document technical debt, known risks, bugs, fragile areas, and improvement opportunities.
> **Generated**: 2026-02-03
> **Last Updated**: 2026-02-03

## Security Concerns

### High Priority

| ID | Area | Description | Risk Level | Mitigation |
|----|------|-------------|------------|------------|
| SEC-001 | WebSocket authentication | Single static token for all clients allows no per-client revocation or auditing | High | Plan token rotation mechanism or client certificates |
| SEC-002 | Token management | Subscriber token hardcoded in environment variable with no expiration or rotation | High | Implement periodic token rotation and audit logging |
| SEC-003 | CORS policy | All origins allowed, no CORS validation in place | Medium | Add configurable CORS origin whitelist |
| SEC-004 | Signature header validation | X-Signature header parsed as-is with minimal format validation | Low | Already mitigated by base64 decoding and cryptographic verification |

### Medium Priority

| ID | Area | Description | Risk Level | Mitigation |
|----|------|-------------|------------|------------|
| SEC-005 | Private key environment variable | `VIBETEA_PRIVATE_KEY` env var alternative not documented; unclear if used in production | Medium | Document usage pattern; prefer file-based keys |
| SEC-006 | Rate limiting overhead | Per-source token bucket tracking could consume memory with many unique sources | Medium | Implement stale entry cleanup (partially done); add configurable limits |
| SEC-007 | Event persistence | Events are in-memory only; no persistence means events lost on restart | Medium | Document as design decision; recommend replay mechanism at application level |

## Technical Debt

### High Priority

| ID | Area | Description | Impact | Effort |
|----|------|-------------|--------|--------|
| TD-001 | Cleanup task | Rate limiter cleanup task in main.rs never terminates; cleanup_handle is dropped without cancellation | Cleanup runs until server shutdown | Low |
| TD-002 | Error handling | Some auth errors (InvalidPublicKey) could reveal server configuration details in logs | Debugging difficulty | Low |

### Medium Priority

| ID | Area | Description | Impact | Effort |
|----|------|-------------|--------|--------|
| TD-003 | Configuration validation | VIBETEA_PUBLIC_KEYS parsing doesn't validate that decoded base64 is exactly 32 bytes | Confusing error messages at runtime | Low |
| TD-004 | Type safety | EventPayload uses untagged enum which could be fragile with certain JSON structures | API contract ambiguity | Medium |
| TD-005 | Logging | Some debug/trace logs are verbose and could impact performance under load | Performance in high-traffic scenarios | Low |

### Low Priority

| ID | Area | Description | Impact | Effort |
|----|------|-------------|--------|--------|
| TD-006 | Documentation | VIBETEA_PRIVATE_KEY environment variable mentioned in crypto.rs but not in main config docs | Developer confusion | Low |
| TD-007 | Error response codes | Health endpoint always returns 200 even during degradation; no status codes for partial failure | Monitoring complexity | Low |

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
| `monitor/src/crypto.rs` | Cryptographic key handling; file permissions matter | Tests verify file permissions on Unix; regenerate if compromised |
| `server/src/config.rs` | Configuration parsing with environment variables; tests required `#[serial]` | Never modify without running full test suite with `--test-threads=1` |

## Deprecated Code

| Area | Deprecation Reason | Removal Target | Replacement |
|------|-------------------|----------------|-------------|
| None identified | - | - | - |

## TODO Items

| Location | TODO | Priority |
|----------|------|----------|
| `monitor/tests/privacy_test.rs:319` | TODO regex in test assertion for security match | Medium |
| None other than above | - | - |

## Dependency Concerns

### At-Risk Dependencies

| Package | Concern | Action Needed |
|---------|---------|---------------|
| `ed25519_dalek` | Cryptographic library; monitor for security advisories | Subscribe to GitHub security alerts |
| `tokio` | Runtime; heavy async dependency with many transitive deps | Keep updated; monitor for CVEs |
| `base64` | Decoding; generally stable but validate error handling | No immediate action needed |

## Performance Concerns

| ID | Area | Description | Impact | Mitigation |
|----|------|-------------|--------|-----------|
| PERF-001 | Rate limiter memory | Hash map of source_id -> TokenBucket grows unbounded until cleanup | Memory leak over time | Cleanup task removes stale entries (60s timeout) |
| PERF-002 | Event broadcast | Broadcaster uses bounded channel; lagging subscribers lose events | Client experience degrades | Expected behavior; clients reconnect |
| PERF-003 | JSON serialization | Every event serialized per WebSocket subscriber | CPU under high load | No mitigation; consider compression |

## Monitoring Gaps

| Area | Missing | Impact |
|------|---------|--------|
| Private key usage | No metrics on whether file vs env var is used | Can't audit key source at runtime |
| Rate limiter state | No metrics on bucket count or token refill rates | Hard to debug rate limiting issues |
| Authentication success rate | No metrics on auth successes vs failures | Can't detect brute force attempts |
| WebSocket health | No metrics on connection duration or message throughput | Hard to diagnose subscription issues |

## Improvement Opportunities

| Area | Current State | Desired State | Benefit |
|------|---------------|---------------|---------|
| Token management | Static string in environment | JWT or token service with expiration | Better security and client management |
| Configuration validation | Happens at startup only | Happens at startup with detailed validation | Catch misconfigurations earlier |
| Error logging | Mix of debug/warn levels | Structured error types with contextual data | Better observability |
| Rate limiting | Per-source only | Support per-IP and per-endpoint limits | Finer-grained DoS protection |
| Event filtering | Client-side by query params | Server-side filtering with ACLs | Reduced bandwidth, better security |

## Security Debt Items

| ID | Area | Description | Mitigation Strategy |
|----|------|-------------|-------------|
| DEBT-001 | WebSocket token | Same token for all clients across all time | Implement token rotation every N days or on deployment |
| DEBT-002 | Key registration | Public keys hardcoded in environment variable | Implement key management API with dynamic updates |
| DEBT-003 | Audit trail | Minimal logging of auth events | Add structured audit logging to database/file |
| DEBT-004 | Client identity | WebSocket clients are anonymous beyond token | Add optional client ID/name for audit purposes |

## Potential Attack Vectors

| Vector | Mitigation | Status |
|--------|-----------|--------|
| Signature bypass (wrong message signed) | Signature verifies full request body | Mitigated |
| Timing attack on token comparison | Constant-time comparison with `subtle` crate | Mitigated |
| Rate limit bypass (multiple sources) | Per-source limiting doesn't prevent N sources | Partially mitigated by total capacity |
| WebSocket replay (reuse old token) | Static token never expires | Not mitigated |
| Source ID spoofing | Must provide valid signature for registered source | Mitigated |
| Base64 decoding errors | Handled with explicit error cases | Mitigated |

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
