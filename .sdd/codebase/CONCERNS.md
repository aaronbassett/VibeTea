# Known Concerns

> **Purpose**: Document technical debt, known risks, bugs, fragile areas, and improvement opportunities.
> **Generated**: 2026-02-04
> **Last Updated**: 2026-02-04

## Technical Debt

### High Priority

Security and reliability issues requiring near-term attention:

| ID | Area | Description | Impact | Effort |
|----|------|-------------|--------|--------|
| TD-001 | `client/src/hooks/useWebSocket.ts` | Bearer token stored in localStorage vulnerable to XSS; no expiration or rotation | Security risk (token theft) | Medium |
| TD-002 | `server/src/routes.rs` | WebSocket authentication via URL query parameter (token visible in logs, history) | Information disclosure | Medium |
| TD-003 | `server/src/` | No HTTPS enforcement at application level; depends entirely on reverse proxy | Security risk (man-in-the-middle) | Low |
| TD-004 | `server/src/` | No security headers configured (CSP, X-Frame-Options, HSTS) | Security risk (XSS, clickjacking) | Low |
| TD-005 | `server/src/` | No per-connection WebSocket message rate limiting after authentication | DoS risk | Medium |

### Medium Priority

Improvements to make when working in the area:

| ID | Area | Description | Impact | Effort |
|----|------|-------------|--------|--------|
| TD-010 | `server/src/rate_limit.rs` | In-memory rate limiter lost on restart; no persistence or distributed support | Operational limitation | High |
| TD-011 | `server/src/routes.rs` | Error messages may leak source_id in responses | Information disclosure (minor) | Low |
| TD-012 | `server/src/config.rs` | Public keys loaded without runtime cryptographic validation | Config risk | Low |
| TD-013 | `server/src/` | Event broadcaster has no capacity limits; could exhaust memory under load | DoS risk | Medium |
| TD-014 | `client/src/hooks/useWebSocket.ts` | No Content-Security-Policy; vulnerable to DOM-based XSS | Security risk | Low |

### Low Priority

Nice-to-have improvements:

| ID | Area | Description | Impact | Effort |
|----|------|-------------|--------|--------|
| TD-020 | `server/src/auth.rs` | No key rotation mechanism documented or implemented | Operational complexity | High |
| TD-021 | `client/src/components/TokenForm.tsx` | Token input form doesn't mask paste events | Information disclosure (minor) | Low |
| TD-022 | `server/src/routes.rs` | POST /events accepts any Content-Type; should enforce application/json | Data validation | Low |

## Known Bugs

Currently active issues:

| ID | Description | Workaround | Severity |
|----|-------------|------------|----------|
| None documented | No known security bugs reported | N/A | N/A |

## Security Concerns

Security-related issues requiring attention (prioritized by risk):

| ID | Area | Description | Risk Level | Mitigation |
|----|------|-------------|------------|------------|
| SEC-001 | `client/src/` | Bearer token stored in localStorage without expiration or secure transport wrapper | High | Implement token TTL, refresh mechanism, or WebAuthn |
| SEC-002 | `server/src/routes.rs` | Token passed in WebSocket URL query parameter (visible in logs, history, autocomplete) | High | Move to Authorization header or WebSocket subprotocol |
| SEC-003 | `server/src/` | Development mode `VIBETEA_UNSAFE_NO_AUTH=true` disables all authentication | High | Never use in production; enforce in deployment config |
| SEC-004 | `server/src/routes.rs` | No rate limiting on WebSocket connections themselves (only on POST /events) | Medium | Add per-connection message rate limiting |
| SEC-005 | `server/src/` | No TLS/HTTPS enforcement at application layer | Medium | Configure reverse proxy with HSTS header |
| SEC-006 | `server/src/` | No security headers middleware configured | Medium | Add CSP, X-Frame-Options, X-Content-Type-Options at reverse proxy |
| SEC-007 | `server/src/rate_limit.rs` | Unlimited unique source IDs can exhaust memory (DoS vector) | Medium | Add configurable limit on unique source ID count |
| SEC-008 | `client/src/` | No certificate pinning for WSS connections | Medium | Implement certificate pinning for production deployments |

## Performance Concerns

Known performance issues and bottlenecks:

| ID | Area | Description | Impact | Mitigation |
|----|------|-------------|--------|------------|
| PERF-001 | `server/src/rate_limit.rs` | Fixed 30-second cleanup interval could accumulate stale entries | Memory growth under high source variation | Consider lazy cleanup or configurable intervals |
| PERF-002 | `server/src/broadcast/` | In-memory event broadcaster with no bounds checking | Memory exhaustion under sustained load | Add bounded channel with drop policy |
| PERF-003 | `client/src/hooks/useWebSocket.ts` | Exponential backoff with jitter could cause thundering herd with many clients | Connection spike at similar times | Add distributed jitter per client |

## Fragile Areas

Code areas that are brittle or risky to modify:

| Area | Why Fragile | Precautions |
|------|-------------|-------------|
| `server/src/auth.rs` | Cryptographic signature verification is security-critical; any change risks introducing vulnerabilities | Maintain >95% test coverage, RFC 8032 compliance required, peer review all changes |
| `server/src/config.rs` | Environment variable parsing with custom format; breaks if format changes | Document parsing rules, add comprehensive tests, consider schema validation |
| `server/src/rate_limit.rs` | Concurrent access with RwLock; race conditions could break rate limiting | Add stress tests, monitor lock contention in production |
| `client/src/hooks/useWebSocket.ts` | Complex state machine with refs and timeouts; reconnection logic is intricate | Test all state transitions, reconnection scenarios, cleanup on unmount |

## Deprecated Code

Code marked for removal:

| Area | Deprecation Reason | Removal Target | Replacement |
|------|-------------------|----------------|-------------|
| None | N/A | N/A | N/A |

## TODO Items

Active TODO comments found in codebase:

| Location | TODO | Priority |
|----------|------|----------|
| `monitor/tests/privacy_test.rs` | TODO comment pattern detected (security-related) | Medium |

Note: Main source code appears free of unresolved TODO comments. Check git history for items moved to issues.

## External Dependencies at Risk

Dependencies that may need attention:

| Package | Version | Concern | Action Needed |
|---------|---------|---------|---------------|
| `ed25519-dalek` | 2.1 | Cryptographic library; essential security component | Monitor for updates, test before upgrading |
| `tokio` | 1.43 | Async runtime; concurrency safety critical | Monitor for updates, test release candidates |
| `axum` | 0.8 | Web framework; HTTP implementation | Verify HTTPS/TLS configuration at deployment |
| `subtle` | 2.6 | Timing-attack protection; must not be abandoned | Ensure crate is actively maintained |
| `serde` | 1.0 | Deserialization; potential DoS via complex inputs | Keep updated for security patches |

All dependencies current as of Cargo.toml snapshot (2026-02-04).

## Improvement Opportunities

Areas that could benefit from refactoring or enhancement:

| Area | Current State | Desired State | Benefit |
|------|---------------|---------------|---------|
| Token management | Static bearer token with no expiration | Token with TTL, refresh tokens, revocation list | Better security posture, audit trail |
| WebSocket auth | Query parameter token transmission | Authorization header in WebSocket handshake (RFC 6455) | Prevents token leakage in logs/history |
| Rate limiting | In-memory, single instance only | Redis-backed, distributed across instances | Scales to multi-instance deployments |
| Security headers | Not configured in application | Reverse proxy with CSP/HSTS middleware | Defense in depth, standards compliance |
| Audit logging | Limited authentication event logging | Centralized audit log (syslog/CloudWatch/DataDog) | Compliance, incident investigation |
| Input validation | Permissive JSON parsing | Strict schema validation at API boundary | Prevents malformed events from propagating |
| Error handling | Some errors leak information | Consistent error classification without enumeration | Prevents source discovery attacks |

## Monitoring Gaps

Areas lacking proper observability:

| Area | Missing | Impact |
|------|---------|--------|
| Authentication | Failed auth attempt metrics/counters | Can't detect brute-force or DoS attacks |
| Rate limiting | Metrics on rate-limited sources and events dropped | Can't identify which monitors are hitting limits |
| WebSocket | Connected client count, messages/sec, lag metrics | Can't detect connection issues or performance degradation |
| Event processing | Latency per event, queue depth, memory usage | Can't identify bottlenecks or capacity planning |
| Token usage | Audit log of which clients connected when | Can't audit client activity or detect unauthorized access |

## Recommendations by Timeline

### Immediate (Before Production)

1. Move WebSocket token from URL query parameter to HTTP Authorization header
2. Configure security headers (CSP, HSTS, X-Frame-Options) at reverse proxy
3. Ensure `VIBETEA_UNSAFE_NO_AUTH` is never set in production configuration
4. Add structured audit logging of all authentication events (success and failure)

### Short-term (Next Sprint)

1. Implement bearer token expiration and refresh mechanism
2. Add per-connection WebSocket message rate limiting
3. Enforce HTTPS/TLS 1.2+ with strong cipher suites
4. Add Content-Type validation (must be application/json) on POST /events

### Medium-term (Next Quarter)

1. Implement distributed rate limiting (Redis or equivalent)
2. Add certificate pinning for client WebSocket connections
3. Implement comprehensive security header policy middleware
4. Add per-message-type rate limiting (different limits for different event types)
5. Implement graceful token rotation with transition period

---

## Concern Severity Guide

| Level | Definition | Response Time |
|-------|------------|----------------|
| Critical | Production impact, security breach | Immediate |
| High | Degraded security posture, missing controls | This sprint |
| Medium | Developer experience, audit concerns, DoS vectors | Next sprint |
| Low | Nice to have, cosmetic, low-risk issues | Backlog |

---

## What Does NOT Belong Here

- Active implementation tasks → Project board/GitHub issues
- Security controls (what we do right) → SECURITY.md
- Architecture decisions → ARCHITECTURE.md
- Code conventions → CONVENTIONS.md

---

*This document tracks what needs attention. Update when concerns are resolved or discovered.*
