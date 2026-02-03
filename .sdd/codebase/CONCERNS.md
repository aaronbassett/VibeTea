# Known Concerns

> **Purpose**: Document technical debt, known risks, bugs, fragile areas, and improvement opportunities.
> **Generated**: 2026-02-03
> **Last Updated**: 2026-02-03

## Technical Debt

### High Priority

Items that should be addressed soon:

| ID | Area | Description | Impact | Effort |
|----|------|-------------|--------|--------|
| TD-001 | `server/src/routes.rs` | No HTTPS enforcement at application level | Security risk | High |
| TD-002 | `server/src/` | No request/response size validation for WebSocket messages | DoS risk | Medium |
| TD-003 | `monitor/src/crypto.rs` | Private key file stored unencrypted on disk | Data compromise risk | High |

### Medium Priority

Items to address when working in the area:

| ID | Area | Description | Impact | Effort |
|----|------|-------------|--------|--------|
| TD-010 | `server/src/` | No logging of successful authentications (only failures) | Audit visibility | Low |
| TD-011 | `server/src/rate_limit.rs` | Rate limiter in-memory only; no persistence across restarts | State loss | Medium |
| TD-012 | `server/src/` | WebSocket frame fragmentation handling is implicit (rely on axum) | Reliability | Medium |
| TD-013 | `server/src/auth.rs` | No key rotation mechanism documented | Operational complexity | Medium |

### Low Priority

Nice to have improvements:

| ID | Area | Description | Impact | Effort |
|----|------|-------------|--------|--------|
| TD-020 | `server/src/` | HTTP/2 upgrade could improve performance | Performance | Medium |
| TD-021 | `monitor/src/` | No key backup/recovery mechanism documented | Operational risk | Low |
| TD-022 | `server/src/routes.rs` | Error responses could be more granular for debugging | Developer experience | Low |

## Security Concerns

Security-related issues requiring attention:

| ID | Area | Description | Risk Level | Mitigation |
|----|------|-------------|------------|------------|
| SEC-001 | `server/src/config.rs` | `VIBETEA_UNSAFE_NO_AUTH` mode disables all authentication | High | Only use in development; never in production |
| SEC-002 | `monitor/src/crypto.rs` | Private keys stored as plaintext bytes on disk | High | Encrypt keys at rest; restrict filesystem access to mode 0600 |
| SEC-003 | `server/src/` | No TLS enforcement at application level | High | Enforce HTTPS via reverse proxy; use HSTS headers |
| SEC-004 | `server/src/routes.rs` | Event source validation happens after deserialization | Medium | Validate earlier if possible; document order of checks |
| SEC-005 | `server/src/` | No metrics/monitoring for suspicious patterns | Medium | Add rate limit bypass detection; log authentication failures centrally |
| SEC-006 | `server/src/` | Constant-time comparison only for WebSocket token | Medium | Extend to all sensitive string comparisons |
| SEC-007 | `server/src/rate_limit.rs` | DoS vector: unlimited unique source IDs can exhaust memory | Medium | Add per-endpoint limit on unique source ID count |

## Known Bugs

Active bugs that haven't been fixed:

| ID | Description | Workaround | Severity |
|----|-------------|------------|----------|
| BUG-001 | WebSocket clients can receive events from unsubscribed sources if filter is not applied | Always specify `source` parameter in WebSocket query | Low |
| BUG-002 | Rate limiter NaN handling: saturating_mul used but edge case with very high rates possible | Keep rates < 1e10 tokens/second | Low |

## Performance Concerns

Known performance issues:

| ID | Area | Description | Impact | Mitigation |
|----|------|-------------|--------|------------|
| PERF-001 | `server/src/rate_limit.rs` | HashMap lookup for each request (O(1) amortized but non-zero overhead) | Latency increase | Acceptable for typical workloads |
| PERF-002 | `server/src/routes.rs` | JSON deserialization on every request | CPU usage | Consider msgpack if bandwidth is concern |
| PERF-003 | `monitor/src/crypto.rs` | File I/O for key loading on each signing operation | Monitor startup latency | Load keys once at startup |

## Fragile Areas

Code areas that are brittle or risky to modify:

| Area | Why Fragile | Precautions |
|------|-------------|-------------|
| `server/src/auth.rs` | Cryptographic signature verification is security-critical | Add tests for every code path; never skip RFC 8032 strict verification |
| `server/src/config.rs` | Configuration parsing affects entire server security posture | Test with invalid inputs; document parsing rules |
| `server/src/routes.rs` | Source validation happens in multiple places; easy to miss one | Centralize source validation logic; add integration tests |
| `monitor/src/crypto.rs` | Signing is security-critical; keys must not leak | Never log keys; use constant-time operations only |
| `monitor/src/trackers/agent_tracker.rs` | Privacy-critical: must never extract or transmit prompt content | Maintain type-safe design (no prompt field in struct); review any struct field additions |

## Deprecated Code

Code marked for removal:

| Area | Deprecation Reason | Removal Target | Replacement |
|------|-------------------|----------------|-------------|
| None currently | N/A | N/A | N/A |

## TODO Items

Active TODO comments in codebase:

| Location | TODO | Priority |
|----------|------|----------|
| `server/src/config.rs:96` | Review VIBETEA_UNSAFE_NO_AUTH warning message | Low |
| `server/src/main.rs:214` | Consider adding graceful shutdown timeout metrics | Low |

## External Dependencies at Risk

Dependencies that may need attention:

| Package | Concern | Action Needed |
|---------|---------|---------------|
| `ed25519_dalek` | Cryptographic library requires correct version (check for updates) | Monitor for security advisories |
| `tokio` | Heavy dependency; ensure async patterns are correct | Monitor for performance regressions |
| `axum` | HTTP framework; ensure HTTPS enforcement at proxy | Verify proxy configuration in deployment |

## Monitoring Gaps

Areas lacking proper observability:

| Area | Missing | Impact |
|------|---------|--------|
| `server/src/` | Per-endpoint latency metrics | Can't detect performance degradation |
| `server/src/auth.rs` | Signature verification success/failure ratio | Can't detect attack patterns |
| `server/src/rate_limit.rs` | Memory usage of rate limiter state | Can't predict capacity exhaustion |
| `monitor/` | Event submission success/failure metrics | Can't detect monitor connectivity issues |

## Improvement Opportunities

Areas that could benefit from refactoring:

| Area | Current State | Desired State | Benefit |
|------|---------------|---------------|---------|
| `server/src/routes.rs` | Multiple error code strings scattered | Centralized error code enum | Consistency, maintainability |
| `server/src/` | Limited validation of configuration values | Schema validation at startup | Catch config errors earlier |
| `server/src/auth.rs` | Signature verification is monolithic | Break into sub-functions | Easier testing, readability |
| `server/src/` | No request tracing/correlation IDs | Add X-Request-ID support | Better debugging |

## Potential Vulnerabilities to Review

These are not confirmed vulnerabilities but areas that should be reviewed:

1. **Timing attacks on token comparison**: While `subtle::ConstantTimeEq` is used for WebSocket tokens, ensure all sensitive comparisons use it.

2. **Public key validation**: Public keys from `VIBETEA_PUBLIC_KEYS` are not validated to be valid Ed25519 keys at startup (only at verification time).

3. **Request body size**: Maximum body size is 1 MB; consider if this is sufficient for use cases.

4. **JSON parsing**: Malicious JSON with deeply nested structures could cause stack overflow; serde has protections but should be verified.

5. **WebSocket upgrade**: Verify that WebSocket upgrade doesn't accept invalid protocols.

6. **Rate limiter state**: HashMap can grow unbounded if many unique source IDs are used; stale entry cleanup helps but may not be sufficient under attack.

## Privacy-Related Concerns

### Phase 4: Agent Tracking Privacy

| ID | Area | Description | Status | Notes |
|----|------|-------------|--------|-------|
| PRIV-001 | `monitor/src/trackers/agent_tracker.rs` | Task tool prompt extraction eliminated | Resolved (Phase 4) | `TaskToolInput` struct intentionally lacks prompt field |
| PRIV-002 | `monitor/src/trackers/agent_tracker.rs` | Type-safe privacy enforcement | Implemented | Privacy guaranteed at compile-time via struct definition |
| PRIV-003 | `monitor/src/trackers/agent_tracker.rs` | Only metadata extracted | Implemented | Extracts: subagent_type, description (non-sensitive fields) |

### Privacy Design Pattern

The agent tracker implements privacy-by-design:
- Struct definition prevents prompt extraction: `TaskToolInput` has no `prompt` field
- Parser silently ignores prompt field in JSON (serde default behavior)
- Type system enforces that prompts cannot be included in events
- Test coverage verifies prompt field is ignored (`tests` module, line 378-393)

This approach is more robust than runtime validation because it's impossible to accidentally transmit sensitive data.

## Compliance Notes

- No formal security audit has been performed
- Code follows Rust best practices and uses safe APIs
- No hardcoded secrets found in codebase
- Cryptographic operations use well-tested libraries (`ed25519_dalek`, `subtle`)
- No SQL injection vectors (no SQL used)
- No code injection vectors (no eval/exec)
- Privacy controls built into type system (no prompt field in task tracking)

---

## Concern Severity Guide

| Level | Definition | Response Time |
|-------|------------|----------------|
| Critical | Production impact, security breach | Immediate |
| High | Degraded functionality, security risk | This sprint |
| Medium | Developer experience, minor issues | Next sprint |
| Low | Nice to have, cosmetic | Backlog |

---

## What Does NOT Belong Here

- Active implementation tasks → Project board/issues
- Security controls (what we do right) → SECURITY.md
- Architecture decisions → ARCHITECTURE.md
- Code conventions → CONVENTIONS.md

---

*This document tracks what needs attention. Update when concerns are resolved or discovered.*
