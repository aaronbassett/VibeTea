# Known Concerns

> **Purpose**: Document technical debt, known risks, bugs, fragile areas, and improvement opportunities.
> **Generated**: 2026-02-02
> **Last Updated**: 2026-02-02

## Security Concerns

Security-related issues requiring attention:

| ID | Area | Description | Risk Level | Effort | Mitigation |
|----|------|-------------|------------|--------|-----------|
| SEC-001 | Server auth | Bearer token has no expiration | Medium | Low | Implement token TTL in configuration |
| SEC-002 | Server auth | No granular authorization/RBAC | High | High | Design per-resource permissions before scaling |
| SEC-003 | Server auth | All clients see all events (no filtering) | High | High | Implement event filtering by source/user |
| SEC-004 | All | No comprehensive audit logging | Medium | Medium | Add structured request/auth/action logging |
| SEC-005 | Server | No rate limiting middleware | High | Medium | Implement token bucket via tower middleware |
| SEC-006 | Server | No security headers configured | Medium | Low | Add HSTS, CSP, X-Frame-Options via tower-http |
| SEC-007 | Monitor | No TLS certificate validation | High | Medium | Verify CA chain in reqwest configuration |
| SEC-008 | Monitor | Private key stored unencrypted | Medium | High | Consider OS keychain integration |
| SEC-009 | Config | Development bypass enabled on startup | Medium | Low | Remove VIBETEA_UNSAFE_NO_AUTH from production |
| SEC-010 | Crypto | Base64 key validation deferred to use time | Low | Low | Validate during config parsing, not at use |
| SEC-011 | Monitor | Server URL has no format validation | Low | Low | Add URL parsing validation in monitor config |
| SEC-012 | Client | No bearer token management implementation | High | High | Implement token storage and refresh logic |
| SEC-013 | Client | No client-side authorization checks | Medium | Medium | Add event filtering before rendering |
| SEC-014 | All | No per-client session isolation | High | High | Implement user/client-based event filtering |

## Technical Debt

### High Priority

Items that should be addressed before scaling:

| ID | Area | Description | Impact | Effort |
|----|------|-------------|--------|--------|
| TD-001 | `server/src/types.rs` | Event validation incomplete - base64 key format not validated at parse time | Cryptographic failures at runtime | Low |
| TD-002 | `server/src/config.rs` | Configuration validation happens late - no base64 format validation of keys | Silent failures during signature verification | Low |
| TD-003 | Server - Auth | Signature verification logic not yet implemented | No actual auth enforcement for monitors | High |
| TD-004 | Server - Logging | Missing structured logging for auth decisions | Difficult debugging of auth issues | Medium |
| TD-005 | All | No tracing/observability for security events | Can't detect or respond to attacks | High |
| TD-006 | `monitor/src/config.rs` | Server URL validation missing | Invalid URLs accepted silently | Low |
| TD-007 | Client - Types | Client-side token management not implemented | Clients can't maintain auth state | High |
| TD-008 | Server | WebSocket authentication not fully enforced | Unauthenticated clients may connect | High |

### Medium Priority

Items to address when working in the area:

| ID | Area | Description | Impact | Effort |
|----|------|-------------|--------|--------|
| TD-010 | `monitor/src/config.rs` | Configuration validation could be stricter (URL format) | Invalid config accepted silently | Low |
| TD-011 | `server/src/error.rs` | Error messages in ServerError could expose information | Potential information disclosure in auth errors | Low |
| TD-012 | All | No integration tests for auth flows | Auth regressions not caught early | Medium |
| TD-013 | Server | Rate limiting dependency installed but not wired | Ready to implement but not integrated | Low |
| TD-014 | Client | No security-related error handling | UI doesn't guide users on auth failures | Medium |
| TD-015 | `server/src/config.rs` | Public key parsing uses manual string splitting | Fragile to changes, no structured format | Medium |
| TD-016 | Client | Event payload validation missing client-side | Malformed events not caught early | Low |

### Low Priority

Nice to have improvements:

| ID | Area | Description | Impact | Effort |
|----|------|-------------|--------|--------|
| TD-020 | `server/src/config.rs` | Add configuration validation tests for edge cases | Better error detection | Low |
| TD-021 | Monitor | Add progress reporting for key loading | Better UX on startup | Low |
| TD-022 | All | Add security documentation to README | Improves onboarding | Low |
| TD-023 | Client | Add JSDoc for security-critical functions | Better code review | Low |

## Missing Security Controls

Critical gaps in security infrastructure:

| Area | Missing Control | Required For | Timeline | Implementation Location |
|------|-----------------|--------------|----------|------------------------|
| Authentication | Signature verification middleware | Monitor auth enforcement | Phase 2 completion | Server main handler |
| Authorization | Event filtering by source | Multi-tenant isolation | Phase 3 | Server event broadcast |
| Rate limiting | Middleware implementation | Production deployment | Phase 2 | tower-http middleware |
| Audit logging | Centralized audit log | Compliance & debugging | Phase 2/3 | New logging module |
| Security headers | CORS, CSP, HSTS headers | Production deployment | Phase 2 | tower-http configuration |
| Certificate validation | TLS validation in reqwest | Prevent MITM attacks | Phase 2 | Monitor HTTP client config |
| Client auth state | Token storage and refresh | Client authentication | Phase 2/3 | Client useAuthStore hook |

## Fragile Areas

Code areas that are brittle or risky to modify:

| Area | Why Fragile | Precautions | Files |
|------|-------------|-------------|-------|
| `server/src/config.rs:157-203` | Manual string parsing for public keys | Add comprehensive parser tests before modifying | Manual split on `:` and `,` |
| `server/src/types.rs:48-111` | Untagged enum deserialization is order-dependent | Document variant ordering, add roundtrip tests | EventPayload must maintain order |
| Auth flow (pending) | Will be critical path, not tested yet | Comprehensive unit + integration tests before shipping | server/src/main.rs (pending) |
| `monitor/src/config.rs:97-143` | Configuration validation | Add URL format validation and tests | Server URL parsing |
| Error handling | Custom ServerError type, not widely used yet | Add error handling tests before expanding usage | `server/src/error.rs` |
| Client state | Zustand store with no auth isolation | Add user-scoped state selector before multi-tenant | `client/src/hooks/useEventStore.ts` |

## Known Bugs

Active bugs that haven't been fixed:

| ID | Area | Description | Severity | Workaround | Status |
|----|------|-------------|----------|-----------|--------|
| BUG-001 | Server config | Invalid unicode in PORT env var crashes config loading | Medium | Ensure PORT contains only ASCII digits | Open |
| BUG-002 | Monitor config | No validation of server URL format (accepts invalid URLs) | Low | Provide correct VIBETEA_SERVER_URL value | Open |
| BUG-003 | Client | Event buffer has no size limit protection beyond 1000 events | Low | Monitor applies FIFO eviction at 1000 max | Open |

## Deprecated Code

Code marked for removal or replacement:

| Area | Status | Reason | Replacement | Timeline | Impact |
|------|--------|--------|-------------|----------|--------|
| `VIBETEA_UNSAFE_NO_AUTH` | Active | Development only, security risk | None - remove from production | Before Phase 1 ship | Required for production safety |
| Default bearer token handling | Pending impl | Currently basic env var, needs enhanced validation | Enhanced token validation middleware | Phase 2 | Security enforcement |

## Monitoring Gaps

Areas lacking proper observability:

| Area | Missing | Impact | Priority | Suggested Implementation |
|------|---------|--------|----------|------------------------|
| Auth decisions | Structured logging of auth events | Can't detect failed auth attempts | High | Log in `server/src/middleware/auth.rs` (pending) |
| Event ingestion | Request metrics (count, latency, size) | Can't monitor event rate or latency | High | Add tower metrics middleware |
| Rate limiting | Enforcement metrics and alerts | Can't verify rate limits are working | High | Log rate limit hits with source |
| WebSocket connections | Connection metrics (open, closed, failures) | Can't monitor client disconnect storms | Medium | Add ws::connect event logging |
| Configuration load | Startup diagnostics | Hard to debug config issues | Medium | Add detailed startup logging |
| Cryptographic operations | Signature verify traces | Can't debug key mismatches | Medium | Log signature verification attempts |

## Performance Concerns

Potential performance issues:

| ID | Area | Description | Impact | Mitigation | Priority |
|----|------|-------------|--------|-----------|----------|
| PERF-001 | Monitor | File watching unoptimized | May miss events on busy systems | Add configurable debounce | Medium |
| PERF-002 | Server | WebSocket broadcast to all clients | O(N) per event, memory overhead | Implement event filtering by topic | High |
| PERF-003 | Server | No connection pooling for backend | May exhaust resources | Add tokio task limiting | Medium |
| PERF-004 | Config | Validation on every startup | Adds latency to boot | Lazy load, cache parsed config | Low |
| PERF-005 | Client | Event buffer unbounded growth potential | Memory leak if buffer limit breached | Add additional safeguards beyond 1000 limit | Medium |

## Dependency Risks

Dependencies that may need attention:

| Package | Concern | Action Needed | Timeline | Priority |
|---------|---------|---------------|----------|----------|
| tokio | Major async runtime, tight coupling | Monitor for breaking changes | Ongoing | High |
| axum | HTTP framework, evolving API | Pin version, test upgrades | Ongoing | High |
| ed25519-dalek | Crypto library, high security impact | Stay current with security patches | Ongoing | Critical |
| notify | File watching, platform-specific bugs | Monitor issue tracker | Ongoing | Medium |
| reqwest | HTTP client, security updates important | Keep up-to-date with patches | Ongoing | High |
| zustand | State management, no auth features built-in | Monitor for auth library integrations | Ongoing | Medium |

## Improvement Opportunities

Areas that could benefit from refactoring or enhancement:

| Area | Current State | Desired State | Benefit | Effort |
|------|---------------|---------------|---------|--------|
| Error handling | Scattered across modules | Centralized error types with context | Better debugging | Medium |
| Configuration | Environment variable parsing | Config file + env override | Easier deployment | Medium |
| Secrets management | Plain environment variables | Integration with vault/secrets manager | Stronger security | High |
| Logging | Basic tracing usage | Structured JSON logging with context | Better observability | Low |
| Testing | Unit tests in modules | Integration tests for auth flows | Regression prevention | High |
| Documentation | Code comments | Inline security considerations | Better review | Low |
| Client auth | Token in env variable | Secure token storage and refresh | Better client UX | High |
| Event filtering | Broadcast to all | Per-client filtered streams | Better security and performance | High |

## TODO Items

Active TODO comments in codebase:

| Location | TODO | Priority | Status | Implementation |
|----------|------|----------|--------|-----------------|
| `server/src/config.rs:73` | Add JWT token support | Medium | Not started | Consider for Phase 3 |
| `server/src/error.rs` | Implement error response formatting | Medium | Pending | Add HTTP response serialization |
| `monitor/src/config.rs` | Add config file support | Low | Backlog | TOML configuration file |
| `server/src/` | Wire rate limiting middleware | High | Pending | Implement tower rate limit middleware |
| `server/src/` | Implement WebSocket auth | High | Pending | Add bearer token validation on ws upgrade |
| `server/src/` | Add security headers | Medium | Pending | tower-http middleware configuration |

## Configuration Debt

Configuration-related issues:

| Issue | Impact | Resolution | Effort | Timeline |
|-------|--------|-----------|--------|----------|
| No `.env.example` file | Unclear which vars are required | Create and commit template | Low | Phase 2 |
| No validation of base64 keys | Silent failures at verification time | Parse and validate during config | Low | Phase 2 |
| Missing URL format validation | Invalid server URLs accepted | Add URL parsing validation | Low | Phase 2 |
| No configuration schema documentation | Hard for users to configure | Generate schema from code | Medium | Phase 3 |
| No production checklist | Easy to deploy insecurely | Create deployment guide | Low | Phase 2 |

## Code Quality Concerns

| Area | Issue | Fix | Priority |
|------|-------|-----|----------|
| Public key parsing | Manual string split logic | Consider using structured format or library | Medium |
| Event deserialization | Order-dependent untagged enum | Add comprehensive roundtrip tests | Low |
| Error messages | Inconsistent across modules | Standardize error reporting | Low |
| Configuration tests | Good coverage but not exhaustive | Add edge case tests | Medium |
| Type guards | Client-side validation incomplete | Add schema validation library | Medium |

## Security Review Checklist

Items to verify before production:

- [ ] VIBETEA_UNSAFE_NO_AUTH=true removed from all production deployments
- [ ] All environment variables documented in deployment guide
- [ ] TLS/HTTPS enforced for all connections (monitor→server, server→client)
- [ ] Rate limiting middleware implemented and tested
- [ ] Audit logging captures auth failures and events
- [ ] CORS headers configured appropriately
- [ ] Security headers (HSTS, CSP, X-Frame-Options) configured
- [ ] Base64 public key validation moved to config parsing
- [ ] Signature verification integrated and tested
- [ ] No hardcoded secrets in code or config files
- [ ] Private key permissions verified (chmod 600)
- [ ] Error messages reviewed for information disclosure
- [ ] All dependencies checked with cargo audit
- [ ] Input validation comprehensive for events and config
- [ ] WebSocket connections authenticated
- [ ] Client token validation implemented and tested
- [ ] Documentation updated with security practices
- [ ] Penetration testing performed
- [ ] Dependency vulnerability scanning in CI/CD

## External Risk Factors

| Risk | Likelihood | Impact | Mitigation | Timeline |
|------|------------|--------|-----------|----------|
| Supply chain attack via dependencies | Low | Critical | cargo audit, lockfile pinning | Ongoing |
| Cryptographic key compromise | Low | Critical | Secure storage, rotation policy | Phase 3 |
| Service DoS via rate limit bypass | Medium | High | Implement rate limiting | Phase 2 |
| Data exposure through logs | Medium | High | Scrub sensitive data from logs | Phase 2 |
| Configuration misconfiguration | High | Medium | Better validation, documentation | Phase 2 |
| Unencrypted transit of sensitive data | Medium | High | Enforce HTTPS/WSS only | Phase 1 |
| Unauthorized data access | High | High | Implement proper authorization | Phase 2/3 |

## Phase 2 Security Impact Analysis

Changes introduced in Phase 2:

| Component | Changes | Security Impact | Outstanding Work |
|-----------|---------|-----------------|------------------|
| `server/src/config.rs` | Added ConfigError, validation tests | Improved config error handling | Validate base64 keys |
| `server/src/error.rs` | Comprehensive ServerError variants | Better error classification | Add error response serialization |
| `server/src/types.rs` | Event types with serde validation | Type-safe event handling | Test roundtrip serialization |
| `monitor/src/config.rs` | Configuration validation | Good env var parsing | Add URL format validation |
| `monitor/src/error.rs` | Crypto error handling | Supports future auth | Integrate with signature verification |
| `monitor/src/types.rs` | Event types for monitor | Consistent data model | Implement event signing |
| `client/src/types/events.ts` | Type-safe event types | Runtime event validation | Add ZodSchema for parsing |
| `client/src/hooks/useEventStore.ts` | Centralized state management | Event aggregation ready | Add auth token management |

---

## Concern Severity Guide

| Level | Definition | Response Time | Example |
|-------|------------|----------------|---------|
| Critical | Production impact, security breach | Immediate | Unencrypted credentials in logs |
| High | Degraded functionality, security risk | This sprint | Missing auth enforcement |
| Medium | Developer experience, moderate risk | Next sprint | Poor error messages |
| Low | Nice to have, low priority | Backlog | Configuration improvements |

---

## What Does NOT Belong Here

- Active implementation tasks → Project board/issues
- Security controls (what we do right) → SECURITY.md
- Architecture decisions → ARCHITECTURE.md
- Code conventions → CONVENTIONS.md

---

*This document tracks what needs attention. Update when concerns are resolved or discovered.*
