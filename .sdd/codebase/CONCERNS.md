# Known Concerns

> **Purpose**: Document technical debt, known risks, bugs, fragile areas, and improvement opportunities.
> **Generated**: 2026-02-02
> **Last Updated**: 2026-02-02

## Security Concerns

Security-related issues requiring attention:

| ID | Area | Description | Risk Level | Effort | Mitigation |
|----|------|-------------|------------|--------|-----------|
| SEC-001 | Server auth | Bearer token has no expiration | Medium | Low | Implement token TTL in CONCERNS tracking |
| SEC-002 | Server auth | No granular authorization/RBAC | High | High | Design per-resource permissions before scaling |
| SEC-003 | Server auth | All clients see all events | High | High | Implement event filtering by source/user |
| SEC-004 | All | No audit logging | Medium | Medium | Add detailed request/auth/action logging |
| SEC-005 | Server | No rate limiting middleware | High | Medium | Implement token bucket via tower middleware |
| SEC-006 | Server | No security headers configured | Medium | Low | Add HSTS, CSP, X-Frame-Options via tower-http |
| SEC-007 | Monitor | No TLS certificate validation | High | Medium | Verify CA chain in reqwest configuration |
| SEC-008 | Monitor | Private key stored unencrypted | Medium | High | Consider OS keychain integration |
| SEC-009 | Config | Development bypass enabled on startup | Medium | Low | Remove VIBETEA_UNSAFE_NO_AUTH from production |
| SEC-010 | Crypto | Base64 key validation deferred | Low | Low | Validate during config parsing, not at use |

## Technical Debt

### High Priority

Items that should be addressed before scaling:

| ID | Area | Description | Impact | Effort |
|----|------|-------------|--------|--------|
| TD-001 | `server/src/types.rs` | Event validation incomplete - base64 key format not validated at parse time | Cryptographic failures at runtime | Low |
| TD-002 | `server/src/config.rs` | Configuration validation happens too late - no format validation of base64 keys | Silent failures during signature verification | Medium |
| TD-003 | Server - Auth | Signature verification logic not yet implemented | No actual auth enforcement | High |
| TD-004 | Server - Logging | Missing structured logging for auth decisions | Difficult debugging of auth issues | Medium |
| TD-005 | All | No tracing/observability for security events | Can't detect or respond to attacks | High |

### Medium Priority

Items to address when working in the area:

| ID | Area | Description | Impact | Effort |
|----|------|-------------|--------|--------|
| TD-010 | `monitor/src/config.rs` | Configuration validation could be stricter (URL format) | Invalid config accepted silently | Low |
| TD-011 | `server/src/error.rs` | Error messages could expose information unnecessarily | Potential information disclosure | Low |
| TD-012 | All | No integration tests for auth flows | Auth regressions not caught early | Medium |
| TD-013 | Server | Rate limiting dependency installed but not wired | Ready to implement but not integrated | Low |
| TD-014 | Client | No security-related error handling yet | UI doesn't guide users on auth failures | Medium |

### Low Priority

Nice to have improvements:

| ID | Area | Description | Impact | Effort |
|----|------|-------------|--------|--------|
| TD-020 | `server/src/config.rs` | Add configuration validation tests for edge cases | Better error detection | Low |
| TD-021 | Monitor | Add progress reporting for key loading | Better UX on startup | Low |
| TD-022 | All | Add security documentation to README | Improves onboarding | Low |

## Missing Security Controls

Critical gaps in security infrastructure:

| Area | Missing Control | Required For | Timeline |
|------|-----------------|--------------|----------|
| Authentication | Signature verification middleware | Monitor auth enforcement | Phase 2 completion |
| Authorization | Event filtering by source | Multi-tenant isolation | Phase 3 |
| Rate limiting | Middleware implementation | Production deployment | Phase 2 |
| Audit logging | Centralized audit log | Compliance & debugging | Phase 2/3 |
| Security headers | CORS, CSP, HSTS headers | Production deployment | Phase 2 |
| Certificate validation | TLS validation in reqwest | Prevent MITM attacks | Phase 2 |

## Fragile Areas

Code areas that are brittle or risky to modify:

| Area | Why Fragile | Precautions |
|------|-------------|-------------|
| `server/src/config.rs` | Manual string parsing for public keys | Add comprehensive parser tests before modifying |
| `server/src/types.rs` | Untagged enum deserialization is order-dependent | Document variant ordering, add roundtrip tests |
| Auth flow (pending) | Will be critical path, not tested yet | Comprehensive unit + integration tests before shipping |
| Configuration validation | No centralized error handling | Standardize config error handling across modules |
| Error handling | Custom ServerError type, not widely used yet | Add error handling tests before expanding usage |

## Known Bugs

Active bugs that haven't been fixed:

| ID | Area | Description | Severity | Workaround |
|----|------|-------------|----------|-----------|
| BUG-001 | Config parsing | Invalid unicode in PORT env var crashes config loading | Medium | Ensure PORT contains only ASCII digits |
| BUG-002 | Monitor config | No validation of server URL format (accepts invalid URLs) | Low | Provide correct VIBETEA_SERVER_URL value |

## Deprecated Code

Code marked for removal or replacement:

| Area | Status | Reason | Replacement | Timeline |
|------|--------|--------|-------------|----------|
| `VIBETEA_UNSAFE_NO_AUTH` | Active | Development only, security risk | None - remove from production | Before Phase 1 ship |
| Default bearer token handling | Pending impl | Currently hardcoded, needs env config | Environment variable parsing | Phase 2 |

## Monitoring Gaps

Areas lacking proper observability:

| Area | Missing | Impact | Priority |
|------|---------|--------|----------|
| Auth decisions | Structured logging | Can't detect failed auth attempts | High |
| Event ingestion | Request metrics | Can't monitor event rate or latency | High |
| Rate limiting | Enforcement metrics | Can't verify rate limits are working | High |
| WebSocket connections | Connection metrics | Can't monitor client disconnect storms | Medium |
| Configuration load | Startup diagnostics | Hard to debug config issues | Medium |
| Cryptographic operations | Signature verify traces | Can't debug key mismatches | Medium |

## Performance Concerns

Potential performance issues:

| ID | Area | Description | Impact | Mitigation |
|----|------|-------------|--------|-----------|
| PERF-001 | Monitor | File watching unoptimized | May miss events on busy systems | Add configurable debounce |
| PERF-002 | Server | WebSocket broadcast to all clients | O(N) per event | Implement event filtering |
| PERF-003 | Server | No connection pooling for backend | May exhaust resources | Add tokio task limiting |
| PERF-004 | Monitor | Config validation on every startup | Adds latency | Lazy load, cache parsed config |

## Dependency Risks

Dependencies that may need attention:

| Package | Concern | Action Needed | Timeline |
|---------|---------|---------------|----------|
| tokio | Major async runtime, tight coupling | Monitor for breaking changes | Ongoing |
| axum | HTTP framework, evolving API | Pin version, test upgrades | Ongoing |
| ed25519-dalek | Crypto library, high security impact | Stay current with security patches | Ongoing |
| notify | File watching, platform-specific bugs | Monitor issue tracker | Ongoing |

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

## TODO Items

Active TODO comments in codebase:

| Location | TODO | Priority | Status |
|----------|------|----------|--------|
| `server/src/config.rs:73` | Add JWT token support | Medium | Not started |
| `server/src/error.rs:` | Implement error response formatting | Medium | Pending |
| `monitor/src/config.rs:` | Add config file support | Low | Backlog |
| `server/src/` | Wire rate limiting middleware | High | Pending |
| `server/src/` | Implement WebSocket auth | High | Pending |
| `server/src/` | Add security headers | Medium | Pending |

## Configuration Debt

Configuration-related issues:

| Issue | Impact | Resolution |
|-------|--------|-----------|
| No `.env.example` file | Unclear which vars are required | Create and commit template |
| No validation of base64 keys | Silent failures at verification time | Parse and validate during config |
| Missing URL format validation | Invalid server URLs accepted | Add URL parsing validation |
| No configuration schema documentation | Hard for users to configure | Generate schema from code |

## Code Quality Concerns

| Area | Issue | Fix |
|------|-------|-----|
| Public key parsing | Manual string split logic | Consider using structured format or library |
| Event deserialization | Order-dependent untagged enum | Add comprehensive roundtrip tests |
| Error messages | Inconsistent across modules | Standardize error reporting |
| Configuration tests | Good coverage but not exhaustive | Add edge case tests |

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
- [ ] Client token validation implemented
- [ ] Documentation updated with security practices

## External Risk Factors

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|-----------|
| Supply chain attack via dependencies | Low | Critical | cargo audit, lockfile pinning |
| Cryptographic key compromise | Low | Critical | Secure storage, rotation policy |
| Service DoS via rate limit bypass | Medium | High | Implement rate limiting |
| Data exposure through logs | Medium | High | Scrub sensitive data from logs |
| Configuration misconfiguration | High | Medium | Better validation, documentation |

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
